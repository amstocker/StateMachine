struct InstanceInput {
    @location(1) position: vec3<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>
}

struct VertexInput {
    @location(0) v_position: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        instance.position[0] + (model.v_position[0] * instance.size[0]),
        instance.position[1] + (model.v_position[1] * instance.size[1]),
        instance.position[2],
        1.0
    );
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}