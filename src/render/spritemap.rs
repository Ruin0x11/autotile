use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use cgmath;

use atlas::*;
use point::Point;
use render::{self, Renderable, Viewport, Vertex, QUAD, QUAD_INDICES};

#[derive(Copy, Clone)]
struct Instance {
    map_coord: [u32; 2],
    tex_offset: [f32; 2],
    tex_ratio: [f32; 2],
    sprite_size: [u32; 2],
}

implement_vertex!(Instance, map_coord, tex_offset, tex_ratio, sprite_size);

pub struct SpriteMap {
    sprites: Vec<(DrawSprite, Point)>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    program: glium::Program,

    tile_manager: TileManager,
}

struct DrawSprite {
    idx: usize,
    color_mod: usize,
}

fn make_map() -> Vec<(DrawSprite, Point)> {
    let mut res = Vec::new();

    res.push((DrawSprite { idx: 0, color_mod: 0 }, Point::new(6, 6) ));
    res.push((DrawSprite { idx: 1, color_mod: 0 }, Point::new(3, 2) ));

    res
}

impl SpriteMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let tile_manager = TileManager::from_config(display, "data/sprites.toml");

        let vertices = glium::VertexBuffer::immutable(display, &QUAD).unwrap();
        let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();

        let program = render::load_program(display, "sprite.vert", "sprite.frag").unwrap();

        let sprites = make_map();

        SpriteMap {
            sprites: sprites,
            indices: indices,
            vertices: vertices,
            program: program,
            tile_manager: tile_manager,
        }
    }

    fn create_instances<F>(&self, display: &F, pass: usize, msecs: u64) -> glium::VertexBuffer<Instance>
        where F: glium::backend::Facade {

        let data = self.sprites.iter()
            .filter(|&&(ref sprite, _)| {
                let texture_idx = self.tile_manager.get_tile_texture_idx(sprite.idx);
                texture_idx == pass
            })
            .map(|&(ref sprite, c)| {
                let (x, y) = (c.x, c.y);
                let (tx, ty) = self.tile_manager.get_texture_offset(sprite.idx, msecs);
                let (sx, sy) = self.tile_manager.get_tile_texture_size(sprite.idx);
                let tex_ratio = self.tile_manager.get_sprite_tex_ratio(sprite.idx);

                Instance { map_coord: [x as u32, y as u32],
                           tex_offset: [tx, ty],
                           tex_ratio: tex_ratio,
                           sprite_size: [sx, sy], }
            }).collect::<Vec<Instance>>();

        glium::VertexBuffer::dynamic(display, &data).unwrap()
    }
}

impl<'a> Renderable for SpriteMap {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        for pass in 0..self.tile_manager.passes() {
            let texture = self.tile_manager.get_texture(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [48u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let instances = self.create_instances(display, pass, msecs);

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: Some(scissor),
                .. Default::default()
            };

            target.draw((&self.vertices, instances.per_instance().unwrap()),
                        &self.indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}
