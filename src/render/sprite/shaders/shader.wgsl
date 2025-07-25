@group(0) @binding(0)
var<uniform> view_projection: mat4x4<f32>;

@group(1) @binding(0)
var<storage, read> instances: array<InstanceData>;

@group(2) @binding(0)
var texture: texture_2d<f32>;

@group(2) @binding(1)
var texture_sampler: sampler;


struct VertexData {
    @location(0) position: vec2<f32>,
    @location(1) texture_coordinates: vec2<f32>,
}

struct InstanceData {
    transform: mat4x4<f32>,
}

struct FragmentData {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>
}

@vertex
fn vs_main(vertex: VertexData, @builtin(instance_index) instance_index: u32) -> FragmentData {
    var fragment: FragmentData;
    let instance = instances[instance_index];

    fragment.clip_position = view_projection * instance.transform * vec4<f32>(vertex.position, 0.0, 1.0);
    fragment.texture_coordinates = vertex.texture_coordinates;

    return fragment;
}

@fragment
fn fs_main(fragment: FragmentData) -> @location(0) vec4<f32> {
    // return textureSample(texture, texture_sampler, fragment.texture_coordinates);
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
