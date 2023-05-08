struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) light: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) light: u32,
    @location(2) world_position: vec3<f32>,
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
    out.world_position =  chunk_offset + in.position;
    out.position = camera.matrix * vec4<f32>(out.world_position, 1.);
    out.tex_coords = in.tex_coords;
    out.light = in.light;

    return out;
}

const TWO_PI: f32 = 6.283185307;

fn hash(seed: vec2<f32>) -> f32 {
    let M1: vec2<f32> = vec2<f32>(3.1251, 17.8737);
    let M2: f32 = 43758.545312;
    return fract(sin(dot(seed, M1)) * M2);
}

fn interpolate(a: f32, b: f32, offset: f32) -> f32 {
    return mix(a, b, smoothstep(0., 1., offset));
}

fn interpolate2d(a: f32, b: f32, c: f32, d: f32, offset: vec2<f32>) -> f32 {
    let ab: f32 = interpolate(a, b, offset.x);
    let cd: f32 = interpolate(c, d, offset.x);
    
    return interpolate(ab, cd, offset.y);
}

fn random_direction(seed: vec2<f32>) -> vec2<f32> {
    let direction: f32 = hash(seed) * TWO_PI;
    return vec2(cos(direction), sin(direction));
}

fn gradient(base: vec2<f32>, sample_coords: vec2<f32>) -> f32 {
    let direction: vec2<f32> = random_direction(base);
    return dot(base - sample_coords, direction);
}

fn perlin(seed: vec2<f32>) -> f32 {
    let base: vec2<f32> = floor(seed);
    let offset: vec2<f32> = fract(seed);
    
    let unmapped: f32 = interpolate2d(
        gradient(base, seed),
        gradient(base + vec2(1., 0.), seed),
        gradient(base + vec2(0., 1.), seed),
        gradient(base + vec2(1., 1.), seed),
        offset
    );
    
    return unmapped;
}

fn normal_at(pos: vec2<f32>) -> vec3<f32> {
    let seed = pos * 4.;
    return normalize(vec3<f32>(perlin(seed), 10., perlin(seed)));
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
    let normal = normal_at(in.world_position.xz);

    let look_dir = normalize(in.world_position - camera.position);
    let reflected = reflect(look_dir, normal);
    let specular_light = 1. + pow(max(dot(sky_uniform.sun_direction, reflected), 0.), 128.);

    let block_light = f32(in.light & 0x0Fu) / 15.;
    let sun_light = f32((in.light >> 4u) & 0x0Fu) / 15.;
    let diffuse_light = f32((in.light >> 8u) & 0x0Fu) / 15.;
    let world_light_unmapped = diffuse_light * max(sky_uniform.sun_light * sun_light, block_light);
    let world_light = world_light_unmapped * world_light_unmapped * specular_light;

    let texture_color = textureSample(texture_test, sampler_test, in.tex_coords).xyz;
    return vec4<f32>(world_light * texture_color, 0.8);
}
