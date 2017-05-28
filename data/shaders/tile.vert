#version 150

in uvec2 position;

in vec2 tex_offset;
in uvec2 map_coord;
in int quadrant;
in int autotile;
in int autotile_index;

uniform uvec2 tile_size;
uniform mat4 matrix;
uniform vec2 tex_ratio;

out highp vec2 v_TexCoords;

vec2 quadrant_offset(int quadrant) {
  int qx = (quadrant % 2);
  int qy = (quadrant / 2);
  return vec2(qx, qy);
}

vec2 autotile_offset(int index) {
  float ax = (index % 4);
  float ay = (index / 4);
  return vec2(ax, ay);
}

vec2 normal_tile(vec2 pos, vec2 qoffset) {
  float u = pos.x * tex_ratio.x + tex_offset.x;
  float v = 1.0 - (pos.y * tex_ratio.y + tex_offset.y);
  return vec2(u, v);
}

vec2 autotile_tile(vec2 pos, vec2 qoffset) {
  return normal_tile(pos + autotile_offset(autotile_index), qoffset);
}

void main() {
  vec2 qoffset = quadrant_offset(quadrant);
  gl_Position = matrix * vec4(map_coord * tile_size * vec2(2.0, 2.0) + position * tile_size + qoffset * tile_size, 0.0, 2.0);
  if (autotile > 0) {
    v_TexCoords = autotile_tile(position, qoffset);
  } else {
    v_TexCoords = normal_tile(position, qoffset);
  }
}
