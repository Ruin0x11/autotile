use std::fs::File;
use std::path::Path;

use cgmath;
use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium_text::FontTexture;
use image;

use atlas_frame::Texture2d;
use texture_atlas::*;
use util;

struct UiDrawList {
    commands: Vec<UiDrawCmd>,
    vertices: Vec<UiVertex>,
    indices: Vec<u16>,
}

impl UiDrawList {
    pub fn new() -> Self {
        UiDrawList {
            commands: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.vertices.clear();
        self.indices.clear();
    }
}

#[derive(Clone, Copy)]
struct UiDrawCmd {
    elem_count: u32,
    is_font: bool,
}

#[derive(Clone, Copy)]
struct UiVertex {
    pos: [f32; 2],
    tex_coords: [f32; 2],
    color: [u8; 4],
}

implement_vertex!(UiVertex, pos, tex_coords, color);

// 1. update state somehow
// 2. output vertices of texture coordinates inside UI texture atlas

// self.bar.update(BarData { current: 100, max: 1000 }):
// drawlist.extend(self.bar.output());

// For text, use the font atlas and output one texture piece for each glyph

pub struct UiRenderer {
    ui_atlas: TextureAtlas,
    font: FontTexture,
    draw_list: UiDrawList,
    last_index: u16,
}

impl UiRenderer {
    pub fn new<F: Facade>(display: &F) -> Self {
        let font = FontTexture::new(display,
                                    File::open(&Path::new("./data/gohufont-14.ttf")).unwrap(),
                                    24).unwrap();
        let atlas = TextureAtlasBuilder::new()
            .add_texture("win")
            .build(display);

        UiRenderer {
            ui_atlas: atlas,
            font: font,
            draw_list: UiDrawList::new(),
            last_index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.draw_list.clear();
        self.last_index = 0;
    }

    pub fn add_tex(&mut self, key: &str,
                   screen_area: (u32, u32, u32, u32),
                   tex_subarea: (u32, u32, u32, u32)) {
        let area = self.ui_atlas.get_texture_area(key);

        let cmd = UiDrawCmd {
            elem_count: 6,
            is_font: false,
        };

        let (pxa, pya, pxb, pyb) = tex_subarea;
        let (sxa, sya, sxb, syb) = screen_area;

        let offset_xa = pxa as f32 / area.w as f32;
        let offset_ya = pya as f32 / area.h as f32;
        let offset_xb = pxb as f32 / area.w as f32;
        let offset_yb = pyb as f32 / area.h as f32;

        let color = [255, 255, 255, 255];

        let vertices = vec! [
            UiVertex { pos: [sxa as f32, sya as f32],
                       tex_coords: [area.x as f32 + offset_xa,
                                    area.y as f32 + offset_ya],
                       color: color.clone() },
            UiVertex { pos: [sxa as f32, syb as f32],
                       tex_coords: [area.x as f32 + offset_xa,
                                    area.y as f32 + offset_yb],
                       color: color.clone() },
            UiVertex { pos: [sxb as f32, syb as f32],
                       tex_coords: [area.x as f32 + offset_xb,
                                    area.y as f32 + offset_yb],
                       color: color.clone() },
            UiVertex { pos: [sxb as f32, sya as f32],
                       tex_coords: [area.x as f32 + offset_xb,
                                    area.y as f32 + offset_ya],
                       color: color.clone()},
        ];

        let next_indices = |i| vec![i, i+1, i+2, i, i+2, i+3];

        let indices = next_indices(self.last_index);
        self.last_index += 4;

        self.draw_list.vertices.extend(vertices);
        self.draw_list.indices.extend(indices);
        self.draw_list.commands.push(cmd);
    }
}

impl<'a> ::Renderable for UiRenderer {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &::Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let proj: [[f32; 4]; 4] = cgmath::ortho(0.0, w, h, 0.0, -1.0, 1.0).into();

        let indices = glium::IndexBuffer::dynamic(display, PrimitiveType::TrianglesList, &self.draw_list.indices).unwrap();
        let vertices = glium::VertexBuffer::dynamic(display, &self.draw_list.vertices).unwrap();

        let vertex_shader = util::read_string("./data/identity.vert");
        let fragment_shader = util::read_string("./data/identity.frag");
        let program = glium::Program::from_source(display, &vertex_shader, &fragment_shader, None).unwrap();

        let uniforms = uniform! {
            matrix: proj,
            tex: self.ui_atlas.get_texture().sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        // TODO move to arguments?
        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            // viewport: {
            //     let (x, y) = viewport.position;
            //     let (w, h) = viewport.size;
            //     Some(glium::Rect { left: x, bottom: y, width: w, height: h })
            // },
            .. Default::default()
        };

        target.draw(&vertices,
                    &indices,
                    &program,
                    &uniforms,
                    &params).unwrap();
    }
}

pub struct UiWindow {
    pos: (u32, u32),
    size: (u32, u32),
}

impl UiWindow {
    pub fn new<F: Facade>(pos: (u32, u32), display: &F) -> Self {
        UiWindow {
            pos: pos,
            size: (300, 100),
        }
    }

    pub fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        renderer.add_tex("win", (x + 0, y + 0, x + 32, y + 32), (0, 0, 32, 32));
        renderer.add_tex("win", (x + 0, y + 32, x + 32, y + 64), (0, 32, 32, 64));
        renderer.add_tex("win", (x + 32, y + 0, x + 64, y + 32), (32, 0, 64, 32));
        renderer.add_tex("win", (x + 32, y + 32, x + 64, y + 64), (32, 32, 64, 64));
    }
}
