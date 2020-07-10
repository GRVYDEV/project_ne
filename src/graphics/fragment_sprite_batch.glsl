uniform sampler2D u_texture_sampler;

in vec2 f_tex_coord;

out vec4 Target0;

void main() {
    Target0 = texture(u_texture_sampler, f_tex_coord);
}