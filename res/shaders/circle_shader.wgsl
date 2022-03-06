[[block]]
struct CameraUniform {
    position: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(vertex.position, 1.0);
    out.uv = vertex.uv - 0.5;

    return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32>{
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    let origin = vec2<f32>(0.0, 0.0);

    let max_shade = 0.5; // Only blocking _some_ of the light...
    let min_rad = 0.1; // The shadow has a bias at the center.
    let max_rad = 1.0;

    // Double the distance, because we're zero-centered. Radius of the quad in uv is 0.5.
    let t = distance(origin, in.uv) * 2.0;
    color.a = max_shade * smoothStep(max_rad, min_rad, t);

    return color;
}
