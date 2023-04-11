struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) unnormalized_direction: vec3<f32>,
}

struct Camera {
    matrix: mat4x4<f32>,
    inverse_matrix: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(in.position, 0., 1.);
    out.unnormalized_direction = (camera.inverse_matrix * vec4(in.position, 1., 1.)).xyz;
    return out;
}

@group(1) @binding(0)
var sky_texture: texture_2d<f32>;
@group(1) @binding(1)
var sky_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let direction = normalize(in.unnormalized_direction);
    return textureSample(sky_texture, sky_sampler, vec2(0., direction.y / -2. + 0.5));
}
