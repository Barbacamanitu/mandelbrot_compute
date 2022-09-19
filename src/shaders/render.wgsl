struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};



@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_mandelbrot: texture_2d<f32>;
@group(0) @binding(1)
var s_mandelbrot: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(t_mandelbrot);
    let coords = vec2<f32>(in.tex_coords.x ,in.tex_coords.y);
    var color = textureSample(t_mandelbrot, s_mandelbrot, coords);
    color.r = 1.0;
    return color;
}