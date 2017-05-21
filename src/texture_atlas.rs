use std;
use glium;
use glium::backend::Facade;
use image::{self, DynamicImage};

pub type Texture2D = glium::texture::CompressedSrgbTexture2d;

#[derive(Debug)]
pub enum TextureAtlasError {
    IO(String),
    Image(String),
    Texture(String),
}

impl From<image::ImageError> for TextureAtlasError {
    fn from(e: image::ImageError) -> Self {
        let s = format!("{:?}", e);
        TextureAtlasError::Image(s)
    }
}

impl From<glium::texture::TextureCreationError> for TextureAtlasError {
    fn from(e: glium::texture::TextureCreationError) -> Self {
        let s = format!("{:?}", e);
        TextureAtlasError::Texture(s)
    }
}

impl From<std::io::Error> for TextureAtlasError {
    fn from(e: std::io::Error) -> Self {
        let s = format!("{:?}", e);
        TextureAtlasError::IO(s)
    }
}


pub struct TextureAtlas {
    texture: Texture2D,
    tex_ratio: [f32; 2],
    tile_cols: u32,
    tile_rows: u32,
}

impl TextureAtlas {
    pub fn load<F: Facade>(display: &F, image_filename: &str) -> Result<Self, TextureAtlasError> {
        let image = (image::open(image_filename))?.to_rgba();
        let dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), dimensions);
        let texture = (Texture2D::new(display, image))?;
        let tile_cols = 4;
        let tile_rows = 2;
        let tex_ratio = [1.0 / tile_cols as f32, 1.0 / tile_rows as f32];

        Ok(TextureAtlas {
            texture: texture,
            tex_ratio: tex_ratio,
            tile_cols: tile_cols,
            tile_rows: tile_rows,
        })
    }
}
