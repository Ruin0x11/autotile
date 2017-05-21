use glium;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use cgmath;

use board::Board;
use point::Direction;
use point::Point;
use point;
use atlas_frame::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [u32; 2],
}

implement_vertex!(Vertex, position);

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

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];
const QUAD: [Vertex; 4] = [
    Vertex { position: [0, 1], },
    Vertex { position: [1, 1], },
    Vertex { position: [0, 0], },
    Vertex { position: [1, 0], },
];

struct DrawTile {
    tile_idx: usize,
    edges: u8,
}

pub struct Tilemap {
    map: Vec<(DrawTile, Point)>,

    indices: glium::IndexBuffer<u16>,
    vertices: glium::VertexBuffer<Vertex>,
    program: glium::Program,

    tile_manager: TileManager,
    tex_ratio: [f32; 2],
}

fn make_map(map: &Board) -> Vec<(DrawTile, Point)> {
    let mut res = Vec::new();
    for i in 0..(map.width()) {
        for j in 0..(map.height()) {
            let pos = Point::new(i, j);
            let tile = DrawTile {
                tile_idx: map.get(&pos).n(),
                edges: get_neighboring_edges(map, pos),
            };
            res.push((tile, pos));
        }
    }
    res
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
    let check_dir = |dir: Direction| (edges & (1 << dir_to_bit(dir))) > 0;

    if !check_dir(N) && !check_dir(W) && !check_dir(E) && !check_dir(S) {
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
    let lookup_tile_idx = |horiz: Direction, vert: Direction, corner: Direction, tiles: [i8; 4], corner_piece: i8| {
        if !check_dir(horiz) && !check_dir(vert) {
            tiles[0]
        } else if !check_dir(horiz) && check_dir(vert) {
            tiles[1]
        } else if check_dir(horiz) && !check_dir(vert) {
            tiles[2]
        } else {
            if !check_dir(corner) {
                corner_piece
            } else {
                tiles[3]
            }
        }
    };

    match quadrant {
        QUAD_NW => {
            lookup_tile_idx(N, W, NW, [8, 9, 12, 13], 2)
        },
        QUAD_NE => {
            lookup_tile_idx(N, E, NE, [11, 10, 15, 14], 3)
        },
        QUAD_SW => {
            lookup_tile_idx(S, W, SW, [20, 21, 16, 17], 6)
        },
        QUAD_SE => {
            lookup_tile_idx(S, E, SE, [23, 22, 19, 18], 7)
        },
        _ => -1,
    }
}

fn read_string(path: &str) -> String {
    use std::io::Read;
    use std::fs::File;

    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

impl Tilemap {
    pub fn new<F: Facade>(display: &F, map: &Board, image_filename: &str) -> Self {
        let tile_manager = TileManagerBuilder::new()
            .add_tile(image_filename, 0, AtlasTile { offset: (12, 0), is_autotile: true } )
            .build(display);

        let vertices = glium::VertexBuffer::immutable(display, &QUAD).unwrap();
        let indices = glium::IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &QUAD_INDICES).unwrap();

        let vertex_shader = read_string("./data/tile.vert");
        let fragment_shader = read_string("./data/tile.frag");
        let program = glium::Program::from_source(display, &vertex_shader, &fragment_shader, None).unwrap();

        let cols: u32 = 2048 / (48 / 2);
        let rows: u32 = 2048 / (48 / 2);

        let tex_ratio = [1.0 / cols as f32, 1.0 / rows as f32];

        let map = make_map(map);

        Tilemap {
            map: map,
            indices: indices,
            vertices: vertices,
            program: program,
            tile_manager: tile_manager,
            tex_ratio: tex_ratio,
        }
    }

    fn create_instances<F>(&self, display: &F, pass: usize) -> glium::VertexBuffer<Instance>
        where F: glium::backend::Facade {

        let data = self.map.iter()
            .filter(|&&(ref tile, _)| {
                let texture_idx = self.tile_manager.get_tile_texture_idx(tile.tile_idx);
                texture_idx == pass
            })
            .flat_map(|&(ref tile, c)| {
                let mut res = Vec::new();
                for quadrant in 0..4 {
                    let (x, y) = (c.x, c.y);

                    let (tx, ty) = self.tile_manager.get_texture_offset(tile.tile_idx);
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

impl<'a> ::Renderable for Tilemap {
    fn render<F, S>(&self, display: &F, target: &mut S, viewport: &::Viewport)
        where F: glium::backend::Facade, S: glium::Surface {

        let (w, h) = (viewport.size.0 as f32, viewport.size.1 as f32);
        let proj: [[f32; 4]; 4] = cgmath::ortho(0.0, w, h, 0.0, -1.0, 1.0).into();

        for pass in 0..self.tile_manager.passes() {
            let texture = self.tile_manager.get_texture(pass);
            let tex_ratio = self.tile_manager.get_tex_ratio(pass);

            let uniforms = uniform! {
                matrix: proj,
                tile_size: [48u32; 2],
                tex: texture.sampled()
                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                tex_ratio: tex_ratio,
            };

            let instances = self.create_instances(display, pass);

            // TODO move to arguments?
            let params = glium::DrawParameters {
                viewport: {
                    let (x, y) = viewport.position;
                    let (w, h) = viewport.size;
                    Some(glium::Rect { left: x, bottom: y, width: w, height: h })
                },
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
