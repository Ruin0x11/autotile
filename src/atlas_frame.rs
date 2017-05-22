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

type TileOffset = (u32, u32);
type TileIndex = usize;

pub type Texture2d = glium::texture::CompressedSrgbTexture2d;

type AnimFrames = u64;
type AnimMillisDelay = u64;
#[derive(Clone)]
pub enum TileKind {
    Static,
    Animated(AnimFrames, AnimMillisDelay),
}

#[derive(Clone)]
pub struct AtlasTile {
    pub offset: TileOffset,
    pub is_autotile: bool,
    pub tile_kind: TileKind,
}

#[derive(Clone)]
pub struct AtlasFrame {
    tile_size: (u32, u32),
    texture_idx: usize,
    rect: Rect,
    offsets: HashMap<TileIndex, AtlasTile>,
}

impl AtlasFrame {
    pub fn new(texture_idx: usize, rect: Rect) -> Self {
        AtlasFrame {
            tile_size: (48, 48),
            texture_idx: texture_idx,
            rect: rect,
            offsets: HashMap::new(),
        }
    }
}

pub type TilePacker<'a> = TexturePacker<'a, DynamicImage, SkylinePacker<Rgba<u8>>>;

pub struct TileManager {
    locations: HashMap<TileIndex, String>,
    frames: HashMap<String, AtlasFrame>,
    textures: Vec<Texture2d>,
}

pub struct TileManagerBuilder<'a> {
    locations: HashMap<TileIndex, String>,
    frames: HashMap<String, AtlasFrame>,
    packers: Vec<TilePacker<'a>>,
}

impl <'a> TileManagerBuilder<'a> {
    pub fn new() -> Self {
        let mut manager = TileManagerBuilder {
            locations: HashMap::new(),
            frames: HashMap::new(),
            packers: Vec::new(),
        };
        manager.add_packer();
        manager
    }

    pub fn add_tile(&'a mut self, path_str: &str, index: TileIndex, tile_data: AtlasTile) -> &'a mut Self {
        let key = path_str.to_string();
        if !self.frames.contains_key(&key) {
            self.add_image(key.clone());
        }

        {
            let mut frame = self.frames.get_mut(&key).unwrap();
            frame.offsets.insert(index, tile_data);
            self.locations.insert(index, key);
        }

        self
    }

    fn add_image(&mut self, path_string: String) {
        let path = Path::new(&path_string);
        let texture = ImageImporter::import_from_file(&path).unwrap();

        for (idx, packer) in self.packers.iter_mut().enumerate() {
            if packer.can_pack(&texture) {
                packer.pack_own(path_string.clone(), texture).unwrap();
                let rect = packer.get_frame(&path_string).unwrap().frame.clone();
                self.frames.insert(path_string.clone(), AtlasFrame::new(idx, rect));
                return;
            }
        }

        self.add_packer();

        {
            // complains that borrow doesn't last long enough
            // len mut packer = self.newest_packer_mut();

            let packer_idx = self.packers.len() - 1;
            let mut packer = self.packers.get_mut(packer_idx).unwrap();
            packer.pack_own(path_string.clone(), texture).unwrap();
            let rect = packer.get_frame(&path_string).unwrap().frame.clone();
            self.frames.insert(path_string.clone(), AtlasFrame::new(packer_idx, rect));
        }
    }

    fn add_packer(&mut self) {
        let config = TexturePackerConfig {
            max_width: 2048,
            max_height: 2048,
            allow_rotation: false,
            texture_outlines: false,
            trim: false,
            texture_padding: 0,
            ..Default::default()
        };

        self.packers.push(TexturePacker::new_skyline(config));
    }

    pub fn build<F: Facade>(&self, display: &F) -> TileManager {
        let mut textures = Vec::new();

        for packer in self.packers.iter() {
            let image = ImageExporter::export(packer).unwrap();
            let mut file = File::create("data/pack.png").unwrap();
            image.save(&mut file, image::PNG).unwrap();
            textures.push(make_texture(display, image));
        }

        TileManager {
            locations: self.locations.clone(),
            frames: self.frames.clone(),
            textures: textures,
        }
    }
}

fn make_texture<F: Facade>(display: &F, image: DynamicImage) -> Texture2d {
    let dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.to_rgba().into_raw(), dimensions);
    Texture2d::new(display, image).unwrap()
}

impl TileManager {
    fn get_frame(&self, tile_type: TileIndex) -> &AtlasFrame {
        let tex_name = self.locations.get(&tile_type).unwrap();
        self.frames.get(tex_name).unwrap()
    }

    pub fn get(&self, tile_type: TileIndex) -> &AtlasTile {
        let frame = self.get_frame(tile_type);
        frame.offsets.get(&tile_type).unwrap()
    }

    pub fn get_tile_texture_idx(&self, tile_type: TileIndex) -> usize {
        self.get_frame(tile_type).texture_idx
    }

    pub fn get_tex_ratio(&self, texture_idx: usize) -> [f32; 2] {
        let dimensions = self.textures.get(texture_idx).unwrap().dimensions();

        let cols: u32 = dimensions.0 / 24;
        let rows: u32 = dimensions.1 / 24;
        [1.0 / cols as f32, 1.0 / rows as f32]
    }

    pub fn get_texture_offset(&self, tile_type: TileIndex, msecs: u64) -> (f32, f32) {
        let frame = self.get_frame(tile_type);
        let tile = frame.offsets.get(&tile_type).unwrap();

        let get_tex_coords = |index: (u32, u32)| {
            let tex_ratio = self.get_tex_ratio(frame.texture_idx);
            let mut add_offset = get_add_offset(&frame.rect);

            match tile.tile_kind {
                TileKind::Static => (),
                TileKind::Animated(frame_count, delay) => {
                    let current_frame = msecs / delay;
                    let x_index_offset = if tile.is_autotile {
                        (4 * current_frame) % frame_count
                    } else {
                        current_frame % frame_count
                    };
                    add_offset.0 += x_index_offset as u32;
                }
            }

            let tx = ((index.0 + add_offset.0) * 2) as f32 * tex_ratio[0];
            let ty = ((index.1 + add_offset.1) * 2) as f32 * tex_ratio[1];

            (tx, ty)
        };

        get_tex_coords(tile.offset)
    }

    pub fn get_texture(&self, idx: usize) -> &Texture2d {
        self.textures.get(idx).unwrap()
    }

    pub fn passes(&self) -> usize {
        self.textures.len()
    }
}

fn get_anim_offset(anim_frame: u32, is_autotile: bool) -> (u32, u32) {
    if is_autotile {
        (5 * anim_frame, 0)
    } else {
        (anim_frame, 0)
    }
}

fn get_add_offset(rect: &Rect) -> (u32, u32) {
    let cols: u32 = rect.x / 48;
    let rows: u32 = rect.y / 48;
    (cols, rows)
}
