#version 150

in uvec2 position;

in vec2 tex_offset;
in uvec2 map_coord;
in vec2 tex_ratio;
in uvec2 sprite_size;

uniform mat4 matrix;
uniform uvec2 tile_size;

out highp vec2 v_TexCoords;

vec2 sprite_texture(vec2 pos) {
  float u = pos.x * tex_ratio.x + tex_offset.x;
  float v = 1.0 - (pos.y * tex_ratio.y + tex_offset.y);
  return vec2(u, v);
}

vec2 sprite_offset(vec2 size) {
  return (vec2(48, 48) - size) / vec2(2, 2);
}

void main() {
  vec2 soffset = sprite_offset(sprite_size);
  gl_Position = matrix * vec4(map_coord * tile_size + position * sprite_size + soffset, 0.0, 1.0);
  v_TexCoords = sprite_texture(position);
}
