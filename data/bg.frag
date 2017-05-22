#version 150 core

uniform vec2 u_resolution;
uniform float u_time;

out lowp vec4 color;

void main() {
    vec2 st = gl_FragCoord.xy/u_resolution.xy;
    st.x *= u_resolution.x/u_resolution.y;

    color = vec4(st.y, 0.0, st.x, 1.0);
}
