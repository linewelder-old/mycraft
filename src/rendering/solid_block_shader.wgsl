struct VertexInput {
    @location(0) val: vec2<u32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) sun_light: f32,
    @location(2) block_light: f32,
    @location(3) diffused_light: f32,
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

    let position_x     = (in.val.x) & 0x1FFu;
    let position_y     = (in.val.x >> 9u) & 0x1FFu;
    let position_z     = (in.val.x >> 18u) & 0x1FFu;
    let tex_coords_x   = (in.val.x >> 27u) & 0x1Fu;
    let tex_coords_y   = (in.val.y) & 0x1Fu;
    let texture_id     = (in.val.y >> 5u) & 0xFFFFu;
    let sun_light      = (in.val.y >> 21u) & 0xFu;
    let block_light    = (in.val.y >> 25u) & 0xFu;
    let diffused_light = (in.val.y >> 29u) & 0x3u;

    let position = vec3(f32(position_x), f32(position_y), f32(position_z)) / 16.;
    let tex_coords = vec2(f32(tex_coords_x), f32(tex_coords_y)) / 16.;

    let tex_base = vec2(
        f32(texture_id % 4u),
        f32(texture_id / 4u),
    ) / 4.;

    out.position = camera.matrix * vec4<f32>(chunk_offset + position, 1.0);
    out.tex_coords = tex_base + tex_coords / 4.;
    out.sun_light = f32(sun_light) / 15.;
    out.block_light = f32(block_light) / 15.;
    out.diffused_light = f32(diffused_light) / 3. * 0.6 + 0.4;
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

    let world_light_unmapped = in.diffused_light * max(sky_uniform.sun_light * in.sun_light, in.block_light);
    let world_light = world_light_unmapped * world_light_unmapped;

    return world_light * texture_color;
}
