use std::collections::HashSet;

use cgmath;
use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;

use point::Point;
use point::RectangleIter;
use render::{self, Renderable, Vertex, Viewport, QUAD, QUAD_INDICES};

#[derive(Clone, Copy)]
struct Instance {
    map_coord: [i32; 2],
    tile_index: i8,
}

implement_vertex!(Instance, map_coord, tile_index);

pub struct ShadowMap {
    instances: glium::VertexBuffer<Instance>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    program: glium::Program,
}

fn make_instances<F: Facade>(display: &F, area: RectangleIter, visible: HashSet<Point>) -> glium::VertexBuffer<Instance> {
    let mut instances = Vec::new();
    for point in area {
        if !visible.contains(&point) {
            instances.push(Instance {
                map_coord: [point.x, point.y],
                tile_index: 4,
            })
        }
    }
    glium::VertexBuffer::immutable(display, &instances).unwrap()
}

impl ShadowMap {
    pub fn new<F: Facade>(display: &F, area: RectangleIter, visible: HashSet<Point>) -> Self {
        let vertices = glium::VertexBuffer::immutable(display, &QUAD).unwrap();
        let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();
        let program = render::load_program(display, "shadow.vert", "shadow.frag").unwrap();

        ShadowMap {
            instances: make_instances(display, area, visible),
            vertices: vertices,
            indices: indices,
            program: program,
        }
    }
}

impl Renderable for ShadowMap {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (proj, scissor) = viewport.main_window();

        let uniforms = uniform! {
            matrix: proj,
            tile_size: [48u32; 2],
        };

        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            scissor: Some(scissor),
            .. Default::default()
        };

        target.draw((&self.vertices, self.instances.per_instance().unwrap()),
                    &self.indices,
                    &self.program,
                    &uniforms,
                    &params).unwrap();
    }
}
