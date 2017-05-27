use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};

use bincode;
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use glium::backend::Facade;
use glob;
use image;
use toml::Value;

use atlas::*;
use util;

#[derive(Serialize, Deserialize)]
pub struct TileManagerConfig {
    pub locations: HashMap<TileIndex, String>,
    pub frames: HashMap<String, AtlasFrame>,
    pub file_hash: String,
}

pub fn get_config_cache_path(config_name: &str) -> PathBuf {
    let cache_filepath_str = format!("data/.packed/{}", config_name);
    PathBuf::from(&cache_filepath_str)
}

pub fn load_tile_manager_config(config_name: &str) -> TileManagerConfig {
    let mut path = get_config_cache_path(config_name);
    path.push("cache.bin");

    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    bincode::deserialize(buf.as_slice()).unwrap()
}

pub fn write_tile_manager_config(config: &TileManagerConfig, config_name: &str) {
    let mut path = get_config_cache_path(config_name);
    path.push("cache.bin");

    let data = bincode::serialize(config, bincode::Infinite).unwrap();
    let mut file = File::create(path).unwrap();
    file.write(data.as_slice()).unwrap();
}

fn hash_str(s: &str) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(s);
    hasher.result_str()
}

impl TileManager {
    pub fn from_config<F: Facade>(display: &F, filename: &str) -> Self {
        let toml_str = util::toml::toml_string_from_file(filename);

        let packed_folder = Path::new(filename).file_stem().unwrap().to_str().unwrap();
        let cache_filepath = get_config_cache_path(packed_folder);

        if !Path::exists(cache_filepath.as_path()) {
            return TileManager::build_from_toml(display, packed_folder, &toml_str);
        }

        // check if tile definitions were changed and only repack textures if
        // so, saving startup time.

        let cached_config = load_tile_manager_config(packed_folder);

        let hash = hash_str(&toml_str);

        if cached_config.file_hash != hash {
            return TileManager::build_from_toml(display, packed_folder, &toml_str);
        }

        println!("Using cached tile atlas config at {}/cache.bin", cache_filepath.display());

        let mut textures = Vec::new();

        for entry in glob::glob(&format!("{}/*.png", cache_filepath.display())).unwrap() {
            match entry {
                Ok(path) => {
                    let image = image::open(&path).unwrap();
                    let texture = make_texture(display, image);
                    textures.push(texture);
                },
                Err(..) => (),
            }
        }

        TileManager::new(cached_config, textures)
    }

    fn build_from_toml<F: Facade>(display: &F, packed_folder: &str, toml_str: &str) -> Self {
        println!("Rebuilding tile atlas config \"{}\"", packed_folder);

        let val = util::toml::toml_value_from_string(toml_str);

        let mut idx = 0;

        let mut builder = TileManagerBuilder::new();

        let maps = match util::toml::expect_value_in_table(&val, "maps") {
            Value::Array(array) => array,
            _                   => panic!("Atlas config array wasn't an array."),
        };

        for map in maps.iter() {
            let name: String = util::toml::expect_value_in_table(&map, "name");
            let tile_size: [u32; 2] = util::toml::expect_value_in_table(&map, "tile_size");
            let file_path = format!("data/texture/{}", name);
            println!("Load: {}", file_path);
            builder.add_frame(&file_path, (tile_size[0], tile_size[1]));
        }

        let tiles = match util::toml::expect_value_in_table(&val, "tiles") {
            Value::Array(array) => array,
            _                   => panic!("Atlas config array wasn't an array."),
        };

        for tile in tiles.iter() {
            let atlas: String = util::toml::expect_value_in_table(&tile, "atlas");
            let offset: [u32; 2] = util::toml::expect_value_in_table(&tile, "offset");
            let is_autotile: bool = util::toml::expect_value_in_table(&tile, "is_autotile");
            let tile_kind = match util::toml::get_value_in_table(&tile, "anim_frames") {
                Some(anim_frames) => {
                    let anim_frames = anim_frames.clone().try_into().unwrap();
                    let anim_delay = util::toml::expect_value_in_table(&tile, "anim_delay");
                    TileKind::Animated(anim_frames, anim_delay)
                },
                None => TileKind::Static,
            };

            let tile = AtlasTile {
                offset: (offset[0], offset[1]),
                is_autotile: is_autotile,
                tile_kind: tile_kind,
            };

            let file_path = format!("data/texture/{}", atlas);
            builder.add_tile(&file_path, idx, tile);

            idx += 1;
        }

        let hash = hash_str(toml_str);

        builder.file_hash = hash;

        builder.build(display, packed_folder)
    }
}
