uniform mat4 u_transform;

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec2 v_tex_coord;

out vec2 f_tex_coord;

void main(){
    f_tex_coord = v_tex_coord;

    gl_Position = u_transform * vec4(v_position, 1.0);
}
