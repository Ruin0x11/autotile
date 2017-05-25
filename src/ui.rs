use std::fs::File;
use std::path::Path;

use cgmath;
use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::Rect;
use texture_packer;

use font::FontTexture;
use texture_atlas::*;
use util;

#[derive(Clone, Copy, Debug)]
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
            last.is_text == cmd.is_text && last.clip_rect == cmd.clip_rect
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
    is_text: bool,
    clip_rect: Option<(f32, f32, f32, f32)>,
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
    program: glium::Program,
    font_program: glium::Program,
}

pub enum TexDir {
    Horizontal,
    Vertical,
    Area,
}

pub enum TexKind {
    Elem(&'static str, (u32, u32), (u32, u32)),
    Font(AreaRect, (u32, u32)),
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

        let font_fragment_shader = util::read_string("./data/font.frag");
        let font_program = glium::Program::from_source(display, &vertex_shader, &font_fragment_shader, None).unwrap();

        UiRenderer {
            ui_atlas: atlas,
            font: font,
            draw_list: UiDrawList::new(),
            program: program,
            font_program: font_program,
        }
    }

    pub fn clear(&mut self) {
        self.draw_list.clear();
    }

    pub fn repeat_tex(&mut self, key: &'static str,
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
                repeats_v = 0;
            },
            TexDir::Vertical => {
                repeats_h = 0;
                repeats_v = clipping_height / th;
            },
            TexDir::Area => {
                repeats_h = clipping_width / tw;
                repeats_v =  clipping_height / th;
            }
        }

        let mut x = cxa;
        let mut y = cya;

        for _ in 0..(repeats_h + 1) {
            for _ in 0..(repeats_v + 1) {
                let screen_pos = (x as i32, y as i32);

                self.add_tex_internal(TexKind::Elem(key, tex_pos, tex_area),
                                      screen_pos,
                                      Some(clipping_rect),
                                      (255, 255, 255, 255));

                y += th;
            }
            x += tw;
            y = cya;
        }
    }

    fn add_tex_internal(&mut self, kind: TexKind,
                        screen_pos: (i32, i32),
                        clip_rect: Option<(u32, u32, u32, u32)>,
                        color: (u8, u8, u8, u8)) {
        let tex_coords = match kind {
            TexKind::Elem(key, tex_pos, tex_area) => {
                let area = self.ui_atlas.get_texture_area(key);
                let tex_subarea = (tex_pos.0, tex_pos.1, tex_pos.0 + tex_area.0, tex_pos.1 + tex_area.1);
                calc_tex_subarea(area, tex_subarea)
            },
            TexKind::Font(coords, _) => coords,
        };

        let is_text = match kind {
            TexKind::Elem(..) => false,
            TexKind::Font(..) => true,
        };

        let clip_rect = match clip_rect {
            Some(r) => Some((r.0 as f32, r.1 as f32, r.2 as f32, r.3 as f32)),
            None => None
        };

        let cmd = UiDrawCmd {
            elem_count: 6,
            is_text: is_text,
            clip_rect: clip_rect,
        };

        let (sx, sy) = screen_pos;
        let (tw, th) = match kind {
            TexKind::Elem(_, _, tex_area) => tex_area,
            TexKind::Font(_, char_size) => char_size,
        };
        let tw = tw as i32;
        let th = th as i32;

        let color = [color.0, color.1, color.2, color.3];

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

        let indices = next_indices(self.draw_list.vertices.len() as u16);

        self.draw_list.vertices.extend(vertices);
        self.draw_list.indices.extend(indices);

        // Between a draw call for every texture and merged draw calls, it is a
        // nearly 800% speed difference.

        // self.draw_list.commands.push(cmd);
        self.draw_list.add_command(cmd);
    }

    pub fn add_tex(&mut self, key: &'static str,
                   screen_pos: (i32, i32),
                   clip_rect: Option<(u32, u32, u32, u32)>,
                   tex_pos: (u32, u32),
                   tex_area: (u32, u32)) {
        self.add_tex_internal(TexKind::Elem(key, tex_pos, tex_area),
                              screen_pos,
                              clip_rect,
                              (255, 255, 255, 255));
    }

    pub fn add_string(&mut self, screen_pos: (i32, i32),
                      clipping_rect: Option<(u32, u32, u32, u32)>,
                      color: (u8, u8, u8, u8),
                      text: &str) {
        if text.len() == 0 {
            return;
        }

        let mut total_text_width = 0.0;
        for ch in text.chars() { // FIXME: apparently wrong, but only thing stable
            let added = self.add_char(screen_pos, clipping_rect, total_text_width, color, ch);
            total_text_width += added;
        }
    }

    // Returns the width of the character printed in EMs.
    fn add_char(&mut self, screen_pos: (i32, i32),
                clipping_rect: Option<(u32, u32, u32, u32)>,
                total_text_width: f32,
                color: (u8, u8, u8, u8),
                ch: char) -> f32 {
        let infos = match self.font.find_infos(ch) {
            Some(infos) => infos,
            None => return 0.0,
        };

        let area = AreaRect {
            x1: infos.tex_coords.0,
            y1: infos.tex_coords.1,
            x2: infos.tex_coords.0 + infos.tex_size.0,
            y2: infos.tex_coords.1 + infos.tex_size.1,
        };

        let pt = 14.0;

        let (ch_width, ch_height) = ((infos.size.0 * pt) as u32, (infos.size.1 * pt) as u32);
        let added_width = infos.size.0 + infos.left_padding;

        // check overflow
        if screen_pos.1 < (infos.height_over_line * pt) as i32 {
            return added_width;
        }

        let true_pos = (screen_pos.0 + (total_text_width * pt) as i32,
                        screen_pos.1 - (infos.height_over_line * pt) as i32);

        self.add_tex_internal(TexKind::Font(area, (ch_width, ch_height)), true_pos, clipping_rect, color);

        added_width
    }
}

impl<'a> ::Renderable for UiRenderer {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &::Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let proj: [[f32; 4]; 4] = cgmath::ortho(0.0, w, h, 0.0, -1.0, 1.0).into();

        let vertices = glium::VertexBuffer::dynamic(display, &self.draw_list.vertices).unwrap();

        let height = viewport.size.1 as f32;
        let scale = viewport.scale;

        let mut idx_start = 0;

        if (msecs / 1000) % 2 == 0 {
            println!("Draw commands: {}", self.draw_list.commands.len());
        }

        for cmd in self.draw_list.commands.iter() {
            let idx_end = idx_start + cmd.elem_count;

            let indices = glium::IndexBuffer::dynamic(display,
                                                      PrimitiveType::TrianglesList,
                                                      &self.draw_list
                                                      .indices[idx_start..idx_end]).unwrap();
            idx_start = idx_end;

            let uniforms = if cmd.is_text {
                uniform! {
                    matrix: proj,
                    tex: self.font.get_texture().sampled()
                        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                        .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                }
            } else {
                uniform! {
                    matrix: proj,
                    tex: self.ui_atlas.get_texture().sampled()
                        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                        .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                }
            };

            let scissor = cmd.clip_rect.map(|rect| {
                Rect {
                    left: (rect.0 * scale) as u32,
                    bottom: ((height - rect.3) * scale) as u32,
                    width: ((rect.2 - rect.0) * scale) as u32,
                    height: ((rect.3 - rect.1) * scale) as u32,
                }
            });

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                scissor: scissor,
                .. Default::default()
            };

            if cmd.is_text {
                target.draw(&vertices,
                            &indices,
                            &self.font_program,
                            &uniforms,
                            &params).unwrap();
            } else {
                target.draw(&vertices,
                            &indices,
                            &self.program,
                            &uniforms,
                            &params).unwrap();
            }
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
            size: (300, 400),
        }
    }

    pub fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        let (w, h) = self.size;

        // center
        renderer.repeat_tex("win", TexDir::Area,
                            (x + 32,       y + 32,
                             x + (w - 32), y + (h - 32)),
                            (16, 16), (32, 32));

        // corners
        renderer.add_tex("win",  (x as i32,              y as i32),               None, (0,  0),  (32, 32));
        renderer.add_tex("win",  (x as i32,              (y + (h - 32)) as i32),  None, (0,  32), (32, 32));
        renderer.add_tex("win",  ((x + (w - 32)) as i32, y as i32),               None, (32, 0),  (32, 32));
        renderer.add_tex("win",  ((x + (w - 32)) as i32, (y + (h - 32)) as i32),  None, (32, 32), (32, 32));

        // borders
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 32,       y,            x + (w - 32), y + 32),            (16, 0),  (32, 32));
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 32,       y + (h - 32), x + (w - 32), y + h), (16, 32), (32, 32));
        renderer.repeat_tex("win", TexDir::Vertical,   (x,            y + 32,       x + 32, y + (h - 32)),       (0,  16), (32, 32));
        renderer.repeat_tex("win", TexDir::Vertical,   (x + (w - 32), y + 32,       x + w, y + (h - 32)),       (32, 16), (32, 32));

        for i in 0..10 {
            UiText::new((x as i32 + 32, y as i32 + 32 * i), "Nyanko! Nyanko! Nyanko!").draw(renderer);
        }
    }

    fn rect(&self) -> (u32, u32, u32, u32) {
        let conv = |i| {
            if i < 0 {
                0
            } else {
                (i + 1) as u32
            }
        };

        (conv(self.pos.0), conv(self.pos.1), conv(self.pos.0 + self.size.0), conv(self.pos.1 + self.size.1))
    }
}

pub struct UiText {
    pos: (i32, i32),
    text: String,
}

impl UiText {
    pub fn new(pos: (i32, i32), text: &str) -> Self {
        UiText {
            pos: pos,
            text: text.to_string(),
        }
    }

    pub fn draw(&self, renderer: &mut UiRenderer) {
        renderer.add_string(self.pos, None, (0, 0, 0, 255), &self.text);
    }
}
