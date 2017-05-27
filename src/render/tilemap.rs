use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use cgmath;

use atlas::*;
use board::Board;
use point::Direction;
use point::Point;
use point;
use util;
use render::{Renderable, Viewport, Vertex, QUAD, QUAD_INDICES};

#[derive(Copy, Clone)]
struct Instance {
    map_coord: [u32; 2],
    tex_offset: [f32; 2],
    quadrant: i8,
    autotile: i8,
    autotile_index: i8,
}

implement_vertex!(Instance, map_coord, tex_offset, quadrant, autotile,
                  autotile_index);

struct DrawTile {
    idx: usize,
    edges: u8,
}

pub struct TileMap {
    map: Vec<(DrawTile, Point)>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    program: glium::Program,

    tile_manager: TileManager,
}

fn dir_to_bit(dir: Direction) -> u8 {
    match dir {
        Direction::NE => 0,
        Direction::N  => 1,
        Direction::NW => 2,
        Direction::E  => 3,
        Direction::W  => 4,
        Direction::SE => 5,
        Direction::S  => 6,
        Direction::SW => 7,
    }
}

fn get_neighboring_edges(map: &Board, pos: Point) -> u8 {
    let my_type = map.get(&pos);

    let mut res: u8 = 0;
    for dir in point::DIRECTIONS.iter() {
        let new_pos = pos + *dir;
        println!("{} {:?} {} {:?}", pos, dir, new_pos, map.get(&new_pos));
        let same_type = map.get(&new_pos) == my_type;
        if same_type {
            res |= 1 << dir_to_bit(*dir);
        }
    }
    println!("Done");
    res
}

const QUAD_NW: i8 = 0;
const QUAD_NE: i8 = 1;
const QUAD_SW: i8 = 2;
const QUAD_SE: i8 = 3;

use point::Direction::*;

fn get_autotile_index(edges: u8, quadrant: i8) -> i8 {
    let is_connected = |dir: Direction| (edges & (1 << dir_to_bit(dir))) > 0;

    if !is_connected(N) && !is_connected(W) && !is_connected(E) && !is_connected(S) {
        let ret = match quadrant {
            QUAD_NW => {
                0
            },
            QUAD_NE => {
                1
            },
            QUAD_SW => {
                4
            },
            QUAD_SE => {
                5
            },
            _ => -1,
        };
        return ret;
    }

    // The tiles are in order from the corner inside.
    let lookup_idx = |horiz: Direction, vert: Direction, corner: Direction, tiles: [i8; 4], corner_piece: i8| {
        if !is_connected(horiz) && !is_connected(vert) {
            tiles[0]
        } else if !is_connected(horiz) && is_connected(vert) {
            tiles[1]
        } else if is_connected(horiz) && !is_connected(vert) {
            tiles[2]
        } else {
            if !is_connected(corner) {
                corner_piece
            } else {
                tiles[3]
            }
        }
    };

    match quadrant {
        QUAD_NW => {
            lookup_idx(N, W, NW, [8, 9, 12, 13], 2)
        },
        QUAD_NE => {
            lookup_idx(N, E, NE, [11, 10, 15, 14], 3)
        },
        QUAD_SW => {
            lookup_idx(S, W, SW, [20, 21, 16, 17], 6)
        },
        QUAD_SE => {
            lookup_idx(S, E, SE, [23, 22, 19, 18], 7)
        },
        _ => -1,
    }
}

fn make_map(map: &Board) -> Vec<(DrawTile, Point)> {
    let mut res = Vec::new();
    for i in 0..(map.width()) {
        for j in 0..(map.height()) {
            let pos = Point::new(i, j);
            let tile = DrawTile {
                idx: map.get(&pos).n(),
                edges: get_neighboring_edges(map, pos),
            };
            res.push((tile, pos));
        }
    }
    res
}

impl TileMap {
    pub fn new<F: Facade>(display: &F) -> Self {
        let tile_manager = TileManager::from_config(display, "data/tiles.toml");

        let vertices = glium::VertexBuffer::immutable(display, &QUAD).unwrap();
        let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();

        let vertex_shader = util::read_string("data/tile.vert");
        let fragment_shader = util::read_string("data/tile.frag");
        let program = glium::Program::from_source(display, &vertex_shader, &fragment_shader, None).unwrap();

        TileMap {
            map: Vec::new(),
            indices: indices,
            vertices: vertices,
            program: program,
            tile_manager: tile_manager,
        }
    }

    pub fn update(&mut self, board: &Board) {
        self.map = make_map(board);
    }

    fn create_instances<F>(&self, display: &F, pass: usize, msecs: u64) -> glium::VertexBuffer<Instance>
        where F: glium::backend::Facade {

        let data = self.map.iter()
            .filter(|&&(ref tile, _)| {
                let texture_idx = self.tile_manager.get_tile_texture_idx(tile.idx);
                texture_idx == pass
            })
            .flat_map(|&(ref tile, c)| {
                let mut res = Vec::new();
                for quadrant in 0..4 {
                    let (x, y) = (c.x, c.y);
                    let (tx, ty) = self.tile_manager.get_texture_offset(tile.idx, msecs);

                    let autotile_index = get_autotile_index(tile.edges, quadrant);

                    res.push(Instance { map_coord: [x as u32, y as u32],
                                        tex_offset: [tx, ty],
                                        quadrant: quadrant,
                                        autotile: 1,
                                        autotile_index: autotile_index, });
                }
                res
            }).collect::<Vec<Instance>>();

        glium::VertexBuffer::dynamic(display, &data).unwrap()
    }
}

impl<'a> Renderable for TileMap {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &Viewport, msecs: u64)
        where F: glium::backend::Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let (x, y) = (viewport.camera.0 as f32, viewport.camera.1 as f32);
        let proj: [[f32; 4]; 4] = cgmath::ortho(x, w + x, h + y, y, -1.0, 1.0).into();

        for pass in 0..self.tile_manager.passes() {
            let texture = self.tile_manager.get_texture(pass);
            let tex_ratio = self.tile_manager.get_tilemap_tex_ratio(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [48u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                tex_ratio: tex_ratio,
            };

            let instances = self.create_instances(display, pass, msecs);

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

            target.draw((&self.vertices, instances.per_instance().unwrap()),
                        &self.indices,
                        &self.program,
                        &uniforms,
                        &params).unwrap();
        }
    }
}
