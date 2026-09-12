struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    canvas_size: vec2<f32>,
    time: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct LayerUniforms {
    transform: mat4x4<f32>,
    opacity: f32,
    blend_mode: u32,
    visible: u32,
    _padding: vec2<f32>,
}

@group(1) @binding(0) var layer_texture: texture_2d<f32>;
@group(1) @binding(1) var layer_sampler: sampler;
@group(1) @binding(2) var<uniform> layer_uniforms: LayerUniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let pos = vec4<f32>(input.position, 0.0, 1.0);
    let transformed = layer_uniforms.transform * pos;
    output.position = uniforms.view_proj * transformed;
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(layer_texture, layer_sampler, input.tex_coord);
    let color = tex_color * input.color * layer_uniforms.opacity;
    
    // Apply blend mode
    // This is a simplified version - real implementation would handle all blend modes
    return color;
}