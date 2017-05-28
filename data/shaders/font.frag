#version 150

in highp vec2 v_TexCoords;
in highp vec4 v_Color;

uniform lowp sampler2D tex;

out lowp vec4 out_color;

void main() {
  vec4 c = vec4(v_Color.rgb, v_Color.a * texture(tex, v_TexCoords));

  if (c.a <= 0.01) {
    discard;
  } else {
    out_color = c;
  }
}
