#version 150

in highp vec2 v_TexCoords;
in highp vec4 v_Color;

uniform lowp sampler2D tex;

out lowp vec4 out_color;

void main() {
  out_color = v_Color * texture(tex, v_TexCoords);
}
