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

struct SkyUniform {
    sun_direction: vec3<f32>,
    time: f32,
    sun_light: f32,
}

@group(1) @binding(0)
var<uniform> sky_uniform: SkyUniform;

@group(2) @binding(0)
var sky_texture: texture_2d<f32>;
@group(2) @binding(1)
var sky_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let direction = normalize(in.unnormalized_direction);
    let uv = vec2(sky_uniform.time * 2., direction.y / -2. + 0.5);
    let sky_color = textureSample(sky_texture, sky_sampler, uv);

    let sun_dot = dot(direction, sky_uniform.sun_direction);
    let sunness = min(1., 1. / 256. / (1. - sun_dot));
    let sun_color = vec4(1.0, 1.0, 0.9, 1.);
    return mix(sky_color, sun_color, sunness);
}
