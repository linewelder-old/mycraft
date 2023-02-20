struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera_matrix: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> model_matrix: mat4x4<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera_matrix * model_matrix * vec4<f32>(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    out.normal = in.normal;
    return out;
}

@group(2) @binding(0)
var texture_test: texture_2d<f32>;
@group(2) @binding(1)
var sampler_test: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sun_direction = normalize(vec3<f32>(1., 2., 0.));
    let light = dot(in.normal, sun_direction) * 0.4 + 0.6;
    return light * textureSample(texture_test, sampler_test, in.tex_coords);
}
