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
    let t1 = f32(index);
    let t0 = 1.0 - t1;
    out.clip_position = vec4<f32>(
        2.0 * (t0 * instance.p[0] + t1 * instance.q[0]) - 1.0,
        2.0 * (t0 * instance.p[1] + t1 * instance.q[1]) - 1.0,
        t0 * instance.p[2] + t1 * instance.q[2],
        1.0
    );
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}