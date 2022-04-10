[[block]]
struct CameraUniform {
    position: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] tex_index: i32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] tex_index: i32;
};

[[stage(vertex)]]
fn main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(vertex.position, 1.0);
    out.uv = vertex.tex_coords;
    out.tex_index = vertex.tex_index;

    return out;
}

[[group(1), binding(0)]] var texture: texture_2d_array<f32>;
[[group(1), binding(1)]] var t_samlper: sampler;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let texture_index: i32 = in.tex_index;
    var color = textureSample(texture, t_samlper, in.uv, texture_index);

    if (color.a == 0.0) { discard; }

    return color;
}
