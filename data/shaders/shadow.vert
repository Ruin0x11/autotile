#version 150

in uvec2 position;
in uvec2 map_coord;

uniform uvec2 tile_size;
uniform mat4 matrix;

void main() {
  gl_Position = matrix * vec4(map_coord * tile_size + position * tile_size, 0.0, 1.0);
}
