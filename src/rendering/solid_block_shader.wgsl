struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) light: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) light: u32,
}

struct Camera {
    matrix: mat4x4<f32>,
    inverse_matrix: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(3) @binding(0)
var<uniform> chunk_offset: vec3<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.matrix * vec4<f32>(chunk_offset + in.position, 1.0);
    out.tex_coords = in.tex_coords;
    out.light = in.light;
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
var texture_test: texture_2d<f32>;
@group(2) @binding(1)
var sampler_test: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(texture_test, sampler_test, in.tex_coords);
    if (texture_color.a == 0.) {
        discard;
    }

    let block_light = f32(in.light & 0x0Fu) / 15.;
    let sun_light = f32((in.light >> 4u) & 0x0Fu) / 15.;
    let diffused_light = f32((in.light >> 8u) & 0x0Fu) / 15.;
    let world_light_unmapped = diffused_light * max(sky_uniform.sun_light * sun_light, block_light);
    let world_light = world_light_unmapped * world_light_unmapped;

    return world_light * texture_color;
}
