// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> @builtin(position) vec4<f32> {
    return vec4<f32>(model.position, 1.0);
}

// Fragment shader

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0, 0, 1.0);
}
