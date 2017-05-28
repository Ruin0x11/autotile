#version 150 core

uniform vec2 u_resolution;
uniform float u_time;

out lowp vec4 out_color;

void main() {
  vec2 st = gl_FragCoord.xy/vec2(1024 * 2, 768 * 2);

    float y = st.y;
    float percent = 0.5;
    vec3 color =  vec3(y) + percent * vec3(0.0, 0.67, 1.075);

    out_color = vec4(color, 1.0);
}
