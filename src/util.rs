use glium;
use glium::backend::Facade;
use image::{DynamicImage, GenericImage};

use atlas_frame::Texture2d;

pub fn read_string(path: &str) -> String {
    use std::io::Read;
    use std::fs::File;

    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

pub fn make_texture<F: Facade>(display: &F, image: DynamicImage) -> Texture2d {
    let dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.to_rgba().into_raw(), dimensions);
    Texture2d::new(display, image).unwrap()
}
