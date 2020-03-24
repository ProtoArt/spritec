#version 150
out vec2 v_texcoords;

void main() {
  gl_Position = vec4(((gl_VertexID == 0)? -3 : 1), ((gl_VertexID == 2)? 3 : -1), 0.0, 1.0);
  v_texcoords = vec2(((gl_VertexID == 0)? -1 : 1), ((gl_VertexID == 2)? 2 : 0));
}
