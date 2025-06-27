@group(0)
@binding(0)
var<uniform> view_projection: mat4x4<f32>;


struct VertexData {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>
}


@vertex
fn vs_main(data: VertexData) -> VertexOutput {
    var output: VertexOutput;

    output.position = view_projection * vec4<f32>(data.position, 0.0, 1.0);
    output.color = vec4<f32>(data.color, 1.0);

    return output;
}

@fragment
fn fs_main(data: VertexOutput) -> @location(0) vec4<f32> {
    return data.color;
}
