struct VertexInput {
    @location(0) v_position: vec2<f32>
}

struct InstanceInput {
    @location(1) position: vec3<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>
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
    var x = -1.0 + (2.0 * instance.position[0]);
    var y = -1.0 + (2.0 * instance.position[1]);
    var w = instance.size[0] * 2.0;
    var h = instance.size[1] * 2.0;
    out.clip_position = vec4<f32>(
        x + (model.v_position[0] * w),
        y + (model.v_position[1] * h),
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