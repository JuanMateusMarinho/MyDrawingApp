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

struct BrushParams {
    size: f32,
    opacity: f32,
    flow: f32,
    hardness: f32,
    spacing: f32,
    rotation: f32,
    scatter: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(1) var<uniform> brush_params: BrushParams;

@group(1) @binding(0) var brush_texture: texture_2d<f32>;
@group(1) @binding(1) var brush_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let pos = vec4<f32>(input.position, 0.0, 1.0);
    output.position = uniforms.view_proj * pos;
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    return output;
}

fn brush_shape(uv: vec2<f32>) -> f32 {
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center) * 2.0;
    return smoothstep(brush_params.hardness, 1.0, 1.0 - dist);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let shape = brush_shape(input.tex_coord);
    let tex_color = textureSample(brush_texture, brush_sampler, input.tex_coord);
    let alpha = shape * tex_color.a * input.color.a * brush_params.opacity * brush_params.flow;
    return vec4<f32>(input.color.rgb, alpha);
}