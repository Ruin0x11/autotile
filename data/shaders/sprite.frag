#version 150

in highp vec2 v_TexCoords;

uniform lowp sampler2D tex;

out lowp vec4 color;

void main() {
  color = texture(tex, v_TexCoords);
}
