#version 150

in vec2 pos;
in vec2 tex_coords;
in vec4 color;

uniform mat4 matrix;

out vec2 v_TexCoords;
out vec4 v_Color;

void main() {
  v_TexCoords = tex_coords;
  v_Color = color / 255.0;
  gl_Position = matrix * vec4(pos.xy, 0, 1);
}
