#import bevy_pbr::forward_io::VertexOutput

struct FrameData {
    frame: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> frame_data: FrameData;

@group(1) @binding(1)
var skill_texture: texture_2d<f32>;

@group(1) @binding(2)
var skill_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * frame_data.frame.zw + frame_data.frame.xy;
    return textureSample(skill_texture, skill_sampler, uv);
}
