use std::fs::File;
use std::path::Path;
use std::ops::Deref;
use std::rc::Rc;

use cgmath;
use glium;
use glium::glutin;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::Rect;
use texture_packer;

use font::FontTexture;
use texture_atlas::*;
use util;

#[derive(Clone, Copy, Debug)]
pub struct AreaRect {
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
                    tex_pos: (u32, u32),
                    tex_area: (u32, u32)) -> AreaRect {

    let subarea = (tex_pos.0, tex_pos.1, tex_pos.0 + tex_area.0, tex_pos.1 + tex_area.1);

    let offset_xa = subarea.0 as f32 / area.w as f32;
    let offset_ya = subarea.1 as f32 / area.h as f32;
    let offset_xb = subarea.2 as f32 / area.w as f32;
    let offset_yb = subarea.3 as f32 / area.h as f32;

    AreaRect {
        x1: area.x as f32 + offset_xa,
        y1: 1.0 - (area.y as f32 + offset_ya),
        x2: area.x as f32 + offset_xb,
        y2: 1.0 - (area.y as f32 + offset_yb),
    }
}

// 1. update state somehow
// 2. output vertices of texture coordinates inside UI texture atlas

// self.bar.update(BarData { current: 100, max: 1000 }):
// drawlist.extend(self.bar.output());

// For text, use the font atlas and output one texture piece for each glyph

pub struct Ui<'a> {
    backend: &'a ::glium::backend::glutin_backend::GlutinFacade,
    renderer: UiRenderer,
    valid: bool,
    layers: Vec<Box<UiLayer>>,
}

impl<'a> Ui<'a> {
    pub fn new(window: &'a ::glium::backend::glutin_backend::GlutinFacade) -> Self {
        Ui {
            backend: window,
            renderer: UiRenderer::new(window),
            valid: true,
            layers: Vec::new(),
        }
    }

    pub fn query<R, T: 'static + UiQuery<QueryResult=R>>(&mut self, layer: &mut T) -> R {
        loop {
            for event in self.backend.poll_events() {
                match layer.on_event(event) {
                    EventResult::Done => return layer.result(),
                    _ => (),
                }
            }
        }
        layer.result()
    }

    pub fn is_active(&self) -> bool {
        !self.layers.is_empty()
    }

    pub fn push_layer<T: 'static + UiLayer>(&mut self, layer: T) {
        self.layers.push(Box::new(layer));
        self.invalidate();
    }

    pub fn pop_layer(&mut self) {
        self.layers.pop();
        self.invalidate();
    }

    pub fn clear(&mut self) {
        self.renderer.clear();
        self.valid = true;
    }

    pub fn update(&mut self, event: glutin::Event) {
        let result = match self.layers.last_mut() {
            None => EventResult::Ignored,
            Some(layer) => layer.on_event(event),
        };

        match result {
            EventResult::Ignored => (),
            EventResult::Consumed(callback) => {
                self.invalidate();
                match callback {
                    None => (),
                    Some(cb) => cb(self)
                }
            }
            EventResult::Done => self.pop_layer(),
        }

        self.redraw();
    }

    fn redraw(&mut self) {
        if !self.valid {
            self.renderer.clear();
            for layer in self.layers.iter() {
                layer.draw(&mut self.renderer);
            }

            self.valid = true;
        }
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
        println!("invalidate ui");
    }
}

impl<'a> ::Renderable for Ui<'a> {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &::Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        self.renderer.render(display, target, viewport, msecs);
    }
}

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
    Font(AreaRect),
}

impl UiRenderer {
    pub fn new<F: Facade>(display: &F) -> Self {
        let font_size = 14;

        let font = FontTexture::new(display,
                                    File::open(&Path::new("./data/gohufont-14.ttf")).unwrap(),
                                    font_size,
                                    FontTexture::ascii_character_list()).unwrap();

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

        let mut x = cxa as i32;
        let mut y = cya as i32;
        let tw = tw as i32;
        let th = th as i32;

        for _ in 0..(repeats_h + 1) {
            for _ in 0..(repeats_v + 1) {
                let screen_pos = (x, y, x + tw, y + th);

                self.add_tex_internal(TexKind::Elem(key, tex_pos, tex_area),
                                      screen_pos,
                                      Some(clipping_rect),
                                      (255, 255, 255, 255));

                y += th;
            }
            x += tw;
            y = cya as i32;
        }
    }

    fn add_tex_internal(&mut self, kind: TexKind,
                        screen_pos: (i32, i32, i32, i32),
                        clip_rect: Option<(u32, u32, u32, u32)>,
                        color: (u8, u8, u8, u8)) {
        let tex_coords = match kind {
            TexKind::Elem(key, tex_pos, tex_area) => {
                let atlas_area = self.ui_atlas.get_texture_area(key);
                calc_tex_subarea(atlas_area, tex_pos, tex_area)
            },
            TexKind::Font(coords) => coords,
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

        let (sxa, sya, sxb, syb) = screen_pos;

        let color = [color.0, color.1, color.2, color.3];

        // 0---3
        // |\  |
        // | \ |
        // |  \|
        // 1---2

        let vertices = vec! [
            UiVertex { pos: [sxa as f32, sya as f32],
                       tex_coords: [tex_coords.x1,
                                    tex_coords.y1],
                       color: color.clone() },
            UiVertex { pos: [sxa as f32, syb as f32],
                       tex_coords: [tex_coords.x1,
                                    tex_coords.y2],
                       color: color.clone() },
            UiVertex { pos: [sxb as f32, syb as f32],
                       tex_coords: [tex_coords.x2,
                                    tex_coords.y2],
                       color: color.clone() },
            UiVertex { pos: [sxb as f32, sya as f32],
                       tex_coords: [tex_coords.x2,
                                    tex_coords.y1],
                       color: color.clone() },
        ];

        let next_indices = |i| vec![i, i+1, i+2, i, i+2, i+3];

        let indices = next_indices(self.draw_list.vertices.len() as u16);

        self.draw_list.vertices.extend(vertices);
        self.draw_list.indices.extend(indices);

        // Between a draw call for every texture and merged draw calls, it is a
        // nearly 800% speed difference (or more).

        // self.draw_list.commands.push(cmd);
        self.draw_list.add_command(cmd);
    }

    pub fn add_tex(&mut self, key: &'static str,
                   screen_pos: (i32, i32),
                   clip_rect: Option<(u32, u32, u32, u32)>,
                   tex_pos: (u32, u32),
                   tex_area: (u32, u32)) {
        let (sx, sy) = screen_pos;
        let (tw, th) = tex_area;

        let true_screen_pos = (sx, sy, sx + tw as i32, sy + th as i32);

        self.add_tex_internal(TexKind::Elem(key, tex_pos, tex_area),
                              true_screen_pos,
                              clip_rect,
                              (255, 255, 255, 255));
    }

    pub fn add_tex_stretch(&mut self, key: &'static str,
                           screen_pos: (i32, i32, i32, i32),
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

    // Returns the width of the character that was printed in EMs.
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

        let pt = self.font.get_font_size() as f32;

        let (ch_width, ch_height) = ((infos.size.0 * pt) as u32, (infos.size.1 * pt) as u32);

        // check overflow
        // if screen_pos.1 < (infos.height_over_line * pt) as i32 {
        //     return added_width;
        // }

        let (sx, sy) = (screen_pos.0 + ((total_text_width + infos.left_padding) * pt) as i32,
                        screen_pos.1 - (infos.height_over_line * pt) as i32);

        let true_pos = (sx, sy, sx + ch_width as i32, sy + ch_height as i32);

        self.add_tex_internal(TexKind::Font(area), true_pos, clipping_rect, color);

        infos.size.0 + infos.left_padding + infos.right_padding
    }
}

fn make_scissor(clip_rect: (f32, f32, f32, f32), height: f32, scale: f32) -> Rect {
    let conv = |i| (i * scale) as u32;
    Rect {
        left:   conv(clip_rect.0),
        bottom: conv(height      - clip_rect.3),
        width:  conv(clip_rect.2 - clip_rect.0),
        height: conv(clip_rect.3 - clip_rect.1),
    }
}

impl<'a> ::Renderable for UiRenderer {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &::Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let proj: [[f32; 4]; 4] = cgmath::ortho(0.0, w, h, 0.0, -1.0, 1.0).into();

        let vertices = glium::VertexBuffer::dynamic(display, &self.draw_list.vertices).unwrap();

        let height = viewport.size.1 as f32;

        let mut idx_start = 0;

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

            let scissor = cmd.clip_rect.map(|rect| make_scissor(rect, height, viewport.scale));

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

pub trait UiElement {
    fn draw(&self, renderer: &mut UiRenderer);
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

    fn rect(&self) -> (u32, u32, u32, u32) {
        (self.pos.0, self.pos.1, self.pos.0 + self.size.0, self.pos.1 + self.size.1)
    }
}

impl UiElement for UiWindow {
    fn draw(&self, renderer: &mut UiRenderer) {
        let (x, y) = self.pos;
        let (w, h) = self.size;

        // center
        renderer.add_tex_stretch("win",
                                 (x as i32,     y as i32,
                                  (x + w) as i32, (y + h) as i32),
                                 None,
                                 (0, 0), (64, 64));

        renderer.repeat_tex("win", TexDir::Area,
                            (x,     y,
                            x + w,  y + h),
                            (0, 64), (64, 64));

        // corners
        renderer.add_tex("win",  (x as i32,              y as i32),               None, (64,  0), (16, 16));
        renderer.add_tex("win",  (x as i32,              (y + (h - 16)) as i32),  None, (64, 48), (16, 16));
        renderer.add_tex("win",  ((x + (w - 16)) as i32, y as i32),               None, (112, 0),  (16, 16));
        renderer.add_tex("win",  ((x + (w - 16)) as i32, (y + (h - 16)) as i32),  None, (112, 48), (16, 16));

        // borders
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 16,       y,            x + (w - 16), y + 16),       (80, 0),  (16, 16));
        renderer.repeat_tex("win", TexDir::Horizontal, (x + 16,       y + (h - 16), x + (w - 16), y + h),        (80, 48), (16, 16));
        renderer.repeat_tex("win", TexDir::Vertical,   (x,            y + 16,       x + 16,       y + (h - 16)), (64, 16), (16, 16));
        renderer.repeat_tex("win", TexDir::Vertical,   (x + (w - 16), y + 16,       x + w,        y + (h - 16)), (112, 16), (16, 16));
    }
}

pub struct UiText {
    pub pos: (i32, i32),
    text_lines: Vec<String>,
}

impl UiText {
    pub fn new(pos: (i32, i32), text: &str) -> Self {
        let split = text.split("\n").map(|s| s.to_string()).collect::<Vec<String>>();
        UiText {
            pos: pos,
            text_lines: split,
        }
    }

    pub fn text(&self) -> String {
        self.text_lines.join("\n")
    }
}

impl UiElement for UiText {
    fn draw(&self, renderer: &mut UiRenderer) {
        for (idx, line) in self.text_lines.iter().enumerate() {
            let pos = (self.pos.0, self.pos.1 + (idx as u32 * renderer.font.get_font_size()) as i32);
            renderer.add_string(pos, None, (0, 0, 0, 255), line);
        }
    }
}

pub struct UiList {
    window: UiWindow,
    items: Vec<UiText>,
    selected: usize,
}

impl UiList {
    pub fn new(pos: (u32, u32), items: Vec<&str>) -> Self {
        let item_height = 20;
        let mut text_items = Vec::new();
        for (idx, item) in items.into_iter().enumerate() {
            let pos = (pos.0 as i32 + 32, pos.1 as i32 + 32 + (item_height * idx as u32) as i32);
            let text = UiText::new(pos, &item);
            text_items.push(text);
        }

        let win = UiWindow::new(pos);

        UiList {
            window: win,
            items: text_items,
            selected: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected == self.items.len() - 1 {
            return;
        }
        self.selected += 1;
    }

    pub fn select_prev(&mut self) {
        if self.selected == 0 {
            return;
        }
        self.selected -= 1;
    }

    pub fn get_selected(&self) -> Option<&UiText> {
        self.items.get(self.selected)
    }

    pub fn set_selected(&mut self, idx: usize) {
        assert!(idx < self.items.len());
        self.selected = idx;
    }

}

impl UiElement for UiList {
    fn draw(&self, renderer: &mut UiRenderer) {
        self.window.draw(renderer);
        for item in self.items.iter() {
            item.draw(renderer);
        }
        if let Some(item) = self.get_selected() {
            let (ix, iy) = item.pos;
            renderer.add_tex("win", (ix - 16, iy - 12), None, (96, 24), (16, 16));
        }
    }
}

pub struct Callback(Rc<Box<Fn(&mut Ui)>>);

impl Callback {
    pub fn from_fn<F: Fn(&mut Ui) + 'static>(f: F) -> Self {
        Callback(Rc::new(Box::new(f)))
    }
}

impl Deref for Callback {
    type Target = Box<Fn(&mut Ui)>;
    fn deref<'a>(&'a self) -> &'a Box<Fn(&mut Ui)> {
        &self.0
    }
}

impl From<Rc<Box<Fn(&mut Ui)>>> for Callback {
    fn from(f: Rc<Box<Fn(&mut Ui)>>) -> Self {
        Callback(f)
    }
}

impl From<Box<Fn(&mut Ui) + Send>> for Callback {
    fn from(f: Box<Fn(&mut Ui) + Send>) -> Self {
        Callback(Rc::new(f))
    }
}

impl From<Box<Fn(&mut Ui)>> for Callback {
    fn from(f: Box<Fn(&mut Ui)>) -> Self {
        Callback(Rc::new(f))
    }
}

pub enum EventResult {
    Ignored,
    Consumed(Option<Callback>),
    Done
}

pub trait UiLayer: UiElement {
    fn on_event(&mut self, event: glutin::Event) -> EventResult;
}

pub struct InvLayer {
    list: UiList,
}

impl InvLayer {
    pub fn new() -> Self {
        InvLayer {
            list: UiList::new((100, 100), vec!["Dood", "Hello, my dear", "end of days", "something", "something else", "starfruit"]),
        }
    }
}

impl UiElement for InvLayer {
    fn draw(&self, renderer: &mut UiRenderer) {
        self.list.draw(renderer);
    }
}

use glium::glutin::{VirtualKeyCode, ElementState};

impl UiLayer for InvLayer {
    fn on_event(&mut self, event: glutin::Event) -> EventResult {
        match event {
            glutin::Event::KeyboardInput(ElementState::Pressed, _, Some(code)) => {
                match code {
                    VirtualKeyCode::Escape |
                    VirtualKeyCode::Return |
                    VirtualKeyCode::Q => {
                        EventResult::Done
                    },
                    VirtualKeyCode::Up => {
                        self.list.select_prev();
                        EventResult::Consumed(None)
                    },
                    VirtualKeyCode::Down => {
                        self.list.select_next();
                        println!("{}", self.list.selected);
                        EventResult::Consumed(None)
                    },
                    _ => EventResult::Ignored,
                }
            },
            _ => EventResult::Ignored,
        }
    }
}

impl UiQuery for InvLayer {
    type QueryResult = String;

    fn result(&self) -> String {
        self.list.get_selected().unwrap().text()
    }
}

pub trait UiQuery: UiLayer {
    type QueryResult;

    fn result(&self) -> Self::QueryResult;
}
