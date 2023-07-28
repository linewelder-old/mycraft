struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

struct Camera {
    matrix: mat4x4<f32>,
    inverse_matrix: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct LineMeshUniform {
    color: vec3<f32>,
    offset: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> line_mesh_uniform: LineMeshUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.matrix * vec4(in.position + line_mesh_uniform.offset, 1.);
    out.color = line_mesh_uniform.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.color, 1.);
}
