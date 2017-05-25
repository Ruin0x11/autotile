use std::fs::File;
use std::path::Path;

use cgmath;
use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium_text::FontTexture;
use glium::Rect;
use texture_packer;

use texture_atlas::*;
use util;

struct AreaRect {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

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

    pub fn add_command(&mut self, cmd: UiDrawCmd) {
        if self.commands.is_empty() {
            self.commands.push(cmd);
            return;
        }

        let should_merge = {
            let last = self.commands.get(self.commands.len() - 1).unwrap();
            last.is_font == cmd.is_font && last.clip_rect == cmd.clip_rect
        };

        if should_merge {
            let last_idx = self.commands.len() - 1;
            let last_mut = self.commands.get_mut(last_idx).unwrap();
            last_mut.elem_count += cmd.elem_count;
        } else {
            self.commands.push(cmd);
        }
    }
}

#[derive(Clone, Copy)]
struct UiDrawCmd {
    elem_count: usize,
    is_font: bool,
    clip_rect: (f32, f32, f32, f32),
}

#[derive(Clone, Copy)]
struct UiVertex {
    pos: [f32; 2],
    tex_coords: [f32; 2],
    color: [u8; 4],
}

implement_vertex!(UiVertex, pos, tex_coords, color);

fn calc_tex_subarea(area: &texture_packer::Rect,
                    subarea: (u32, u32, u32, u32)) -> AreaRect {

    let offset_xa = subarea.0 as f32 / area.w as f32;
    let offset_ya = subarea.1 as f32 / area.h as f32;
    let offset_xb = subarea.2 as f32 / area.w as f32;
    let offset_yb = subarea.3 as f32 / area.h as f32;

    AreaRect {
        x1: area.x as f32 + offset_xa,
        y1: area.y as f32 + offset_ya,
        x2: area.x as f32 + offset_xb,
        y2: area.y as f32 + offset_yb,
    }
}

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
    program: glium::Program,
}

pub enum TexDir {
    Horizontal,
    Vertical,
    Area,
}

impl UiRenderer {
    pub fn new<F: Facade>(display: &F) -> Self {
        let font = FontTexture::new(display,
                                    File::open(&Path::new("./data/gohufont-14.ttf")).unwrap(),
                                    24).unwrap();
        let atlas = TextureAtlasBuilder::new()
            .add_texture("win")
            .build(display);

        let vertex_shader = util::read_string("./data/identity.vert");
        let fragment_shader = util::read_string("./data/identity.frag");
        let program = glium::Program::from_source(display, &vertex_shader, &fragment_shader, None).unwrap();

        UiRenderer {
            ui_atlas: atlas,
            font: font,
            draw_list: UiDrawList::new(),
            last_index: 0,
            program: program,
        }
    }

    pub fn clear(&mut self) {
        self.draw_list.clear();
        self.last_index = 0;
    }

    pub fn repeat_tex(&mut self, key: &str,
                      dir: TexDir,
                      clipping_rect: (u32, u32, u32, u32),
                      tex_pos: (u32, u32),
                      tex_area: (u32, u32)) {
        let (cxa, cya, cxb, cyb) = clipping_rect;
        let clipping_width = cxb - cxa;
        let clipping_height = cyb - cya;

        let (tw, th) = tex_area;
        let repeats_h;
        let repeats_v;
        match dir {
            TexDir::Horizontal => {
                repeats_h = clipping_width / tw;
                repeats_v = 1;
            },
            TexDir::Vertical => {
                repeats_h = 1;
                repeats_v = clipping_height / th;
            },
            TexDir::Area => {
                repeats_h = clipping_width / tw;
                repeats_v =  clipping_height / th;
            }
        }

        let mut x = cxa;
        let mut y = cya;

        for _ in 0..repeats_h {
            for _ in 0..repeats_v {
                let screen_pos = (x, y);

                self.add_tex_internal(key, screen_pos, tex_pos, tex_area, clipping_rect);

                y += th;
            }
            x += tw;
            y = cya;
        }
    }

    fn add_tex_internal(&mut self, key: &str,
                        screen_pos: (u32, u32),
                        tex_pos: (u32, u32),
                        tex_area: (u32, u32),
                        clip_rect: (u32, u32, u32, u32)) {
        let area = self.ui_atlas.get_texture_area(key);
        let tex_subarea = (tex_pos.0, tex_pos.1, tex_pos.0 + tex_area.0, tex_pos.1 + tex_area.1);

        let cmd = UiDrawCmd {
            elem_count: 6,
            is_font: false,
            clip_rect: (clip_rect.0 as f32, clip_rect.1 as f32,
                        clip_rect.2 as f32, clip_rect.3 as f32),
        };

        let tex_coords = calc_tex_subarea(area, tex_subarea);
        let (sx, sy) = screen_pos;
        let (tw, th) = tex_area;

        let color = [255, 255, 255, 255];

        let vertices = vec! [
            UiVertex { pos: [sx as f32, sy as f32],
                       tex_coords: [tex_coords.x1,
                                    tex_coords.y1],
                       color: color.clone() },
            UiVertex { pos: [sx as f32, (sy + th) as f32],
                       tex_coords: [tex_coords.x1,
                                    tex_coords.y2],
                       color: color.clone() },
            UiVertex { pos: [(sx + tw) as f32, (sy + th) as f32],
                       tex_coords: [tex_coords.x2,
                                    tex_coords.y2],
                       color: color.clone() },
            UiVertex { pos: [(sx + tw) as f32, sy as f32],
                       tex_coords: [tex_coords.x2,
                                    tex_coords.y1],
                       color: color.clone()},
        ];

        let next_indices = |i| vec![i, i+1, i+2, i, i+2, i+3];

        let indices = next_indices(self.last_index);
        self.last_index += 4;

        self.draw_list.vertices.extend(vertices);
        self.draw_list.indices.extend(indices);

        // Between a draw call for every texture and merged draw calls, it is a
        // nearly 800% speed difference.

        // self.draw_list.commands.push(cmd);
        self.draw_list.add_command(cmd);
    }

    pub fn add_tex(&mut self, key: &str,
                   screen_pos: (u32, u32),
                   tex_pos: (u32, u32),
                   tex_area: (u32, u32)) {
        let (sx, sy) = screen_pos;
        let (tw, th) = tex_area;
        let clip_rect = (sx, sy, sx + tw, sy + th);
        self.add_tex_internal(key, screen_pos, tex_pos, tex_area, clip_rect);
    }
}

impl<'a> ::Renderable for UiRenderer {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &::Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let proj: [[f32; 4]; 4] = cgmath::ortho(0.0, w, h, 0.0, -1.0, 1.0).into();

        let vertices = glium::VertexBuffer::dynamic(display, &self.draw_list.vertices).unwrap();

        let uniforms = uniform! {
            matrix: proj,
            tex: self.ui_atlas.get_texture().sampled()
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
        };

        let height = viewport.size.1 as f32;
        let scale = viewport.scale;

        let mut idx_start = 0;

        if msecs % 1000 == 0 {
            println!("Draw commands: {}", self.draw_list.commands.len());
        }

        for cmd in self.draw_list.commands.iter() {
            let idx_end = idx_start + cmd.elem_count;

            let indices = glium::IndexBuffer::dynamic(display,
                                                      PrimitiveType::TrianglesList,
                                                      &self.draw_list
                                                      .indices[idx_start..idx_end]).unwrap();
            idx_start = idx_end;

            let scissor = Rect {
                left: (cmd.clip_rect.0 * scale) as u32,
                bottom: ((height - cmd.clip_rect.3) * scale) as u32,
                width: ((cmd.clip_rect.2 - cmd.clip_rect.0) * scale) as u32,
                height: ((cmd.clip_rect.3 - cmd.clip_rect.1) * scale) as u32,
            };

            let sc_now = if (msecs / 1000) % 2 == 0 {
                None
            } else {
                Some(scissor)
            };

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: sc_now,
                .. Default::default()
            };

            target.draw(&vertices,
                        &indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}

pub struct UiWindow {
    pos: (u32, u32),
    size: (u32, u32),
}

impl UiWindow {
    pub fn new(pos: (u32, u32)) -> Self {
        UiWindow {
            pos: pos,
            size: (1280, 768),
        }
    }

    pub fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        let (w, h) = self.size;

        // corners
        renderer.add_tex("win",  (x,            y),             (0,  0),  (32, 32));
        renderer.add_tex("win",  (x,            y + (h - 32)),  (0,  32), (32, 32));
        renderer.add_tex("win",  (x + (w - 32), y),             (32, 0),  (32, 32));
        renderer.add_tex("win",  (x + (w - 32), y + (h - 32)),  (32, 32), (32, 32));

        // borders
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 32,       y,            x + (w - 32), y),            (16, 0),  (32, 32));
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 32,       y + (h - 32), x + (w - 32), y + (h - 32)), (16, 32), (32, 32));
        renderer.repeat_tex("win", TexDir::Vertical,   (x,            y + 32, x + (w - 32), y + (h - 32)),       (0,  16), (32, 32));
        renderer.repeat_tex("win", TexDir::Vertical,   (x + (w - 32), y + 32, x + (w - 32), y + (h - 32)),       (32, 16), (32, 32));

        // center
        renderer.repeat_tex("win", TexDir::Area,       (x + 32,       y + 32, x + (w - 32), y + (h - 32)),       (16, 16), (32, 32));
    }
}
