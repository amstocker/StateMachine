struct InstaceInput {
    @location(0) p: vec3<f32>,
    @location(1) q: vec3<f32>,
    @location(2) color: vec4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
    instance: InstaceInput
) -> VertexOutput {
    var out: VertexOutput;
    var pos = (1.0 - f32(index)) * instance.p + f32(index) * instance.q;
    out.clip_position = vec4<f32>(
        2.0 * pos - 1.0,
        1.0
    );
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}