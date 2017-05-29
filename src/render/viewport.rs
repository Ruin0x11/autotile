use cgmath;
use glium;

#[derive(Debug)]
pub struct Viewport {
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub scale: f32,
    pub camera: (i32, i32),
}

pub type RendererSubarea = ([[f32; 4]; 4], glium::Rect);

impl Viewport {
    pub fn main_window(&self) -> RendererSubarea {
        let (w, h) = self.scaled_size();
        self.make_subarea((0, 0, w, h - 120))
    }

    pub fn scaled_size(&self) -> (u32, u32) {
        ((self.size.0 as f32 * self.scale) as u32, (self.size.1 as f32 * self.scale) as u32)
    }

    fn make_subarea(&self, area: (u32, u32, u32, u32)) -> RendererSubarea {
        (self.camera_projection(), self.scissor(area))
    }

    pub fn static_projection(&self) -> [[f32; 4]; 4] {
        self.make_projection_matrix((0, 0))
    }

    pub fn camera_projection(&self) -> [[f32; 4]; 4] {
        self.make_projection_matrix(self.camera)
    }

    fn make_projection_matrix(&self, offset: (i32, i32)) -> [[f32; 4]; 4] {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let (x, y) = (offset.0 as f32, offset.1 as f32);

        let left = x;
        let right = x + w;
        let bottom = y + h;
        let top = y;

        cgmath::ortho(left, right, bottom, top, -1.0, 1.0).into()
    }

    fn scissor(&self, area: (u32, u32, u32, u32)) -> glium::Rect {
        let (ax, ay, aw, ah) = area;
        let (_, h) = self.scaled_size();
        let conv = |i| (i as f32 * self.scale) as u32;

        glium::Rect { left:   conv(ax),
                      bottom: conv(ay) + conv(h - ah),
                      width:  conv(aw - ax),
                      height: conv(ah) - conv(ay * 2),
        }
    }
}
