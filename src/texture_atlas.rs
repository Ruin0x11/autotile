use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use glium;
use glium::backend::Facade;
use image::{self, DynamicImage, GenericImage, Rgba};
use texture_packer::Rect;
use texture_packer::SkylinePacker;
use texture_packer::{TexturePacker, TexturePackerConfig};
use texture_packer::importer::ImageImporter;
use texture_packer::exporter::ImageExporter;
use atlas_frame::Texture2d;

use util;

pub struct TextureAtlas {
    texture: Texture2d,
    frames: HashMap<String, Rect>,
}

type TextureAtlasPacker<'a> = TexturePacker<'a, DynamicImage, SkylinePacker<Rgba<u8>>>;

pub struct TextureAtlasBuilder<'a> {
    packer: TextureAtlasPacker<'a>,
    frames: HashMap<String, Rect>,
}

pub struct TextureRect {
    uv: (f32, f32, f32, f32)
}

impl TextureRect {
    fn from_atlas_rect(dim: (u32, u32), rect: &Rect) -> Self {
        TextureRect {
            uv: (rect.x as f32 / dim.0 as f32, rect.y as f32 / dim.1 as f32,
                 rect.w as f32 / dim.0 as f32, rect.h as f32 / dim.1 as f32),
        }
    }
}

impl<'a> TextureAtlasBuilder<'a> {
    pub fn new() -> Self {
        let config = TexturePackerConfig {
            max_width: 4096,
            max_height: 4096,
            allow_rotation: false,
            texture_outlines: false,
            trim: false,
            texture_padding: 0,
            ..Default::default()
        };

        TextureAtlasBuilder {
            packer: TexturePacker::new_skyline(config),
            frames: HashMap::new(),
        }
    }

    pub fn add_texture(&'a mut self, texture_name: &str) -> &'a mut Self {
        let path_str = format!("./data/texture/{}.png", &texture_name);
        let path = Path::new(&path_str);
        let texture = ImageImporter::import_from_file(&path).unwrap();

        self.packer.pack_own(path_str.to_string(), texture).unwrap();

        let rect = self.packer.get_frame(&path_str).unwrap().frame.clone();
        self.frames.insert(texture_name.to_string(), rect);

        self
    }

    pub fn build<F: Facade>(&self, display: &F) -> TextureAtlas {
        let image = ImageExporter::export(&self.packer).unwrap();
        let mut file = File::create("data/pack.png").unwrap();
        image.save(&mut file, image::PNG).unwrap();

        let texture = util::make_texture(display, image);
        let dim = texture.dimensions();

        let mut frames = HashMap::new();
        for (key, frame) in self.frames.iter() {
            frames.insert(key.clone(), TextureRect::from_atlas_rect(dim, frame));
        }

        TextureAtlas {
            texture: texture,
            frames: self.frames.clone(),
        }
    }
}

impl TextureAtlas {
    pub fn get_texture(&self) -> &Texture2d {
        &self.texture
    }

    pub fn get_texture_area(&self, key: &str) -> &Rect {
        self.frames.get(key).unwrap()
    }
}
