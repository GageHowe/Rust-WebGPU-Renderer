@group(0) @binding(0) var<uniform> u_color: vec4<f32>;

struct PushConsts {
    view_projection: mat4x4<f32>,
};
var<push_constant> pc: PushConsts;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,     // unused
    @location(2) normal: vec3<f32>,

    @location(3) i_m0: vec4<f32>,
    @location(4) i_m1: vec4<f32>,
    @location(5) i_m2: vec4<f32>,
    @location(6) i_m3: vec4<f32>,
};

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(v: VertexIn) -> VertexPayload {
    let model = mat4x4<f32>(v.i_m0, v.i_m1, v.i_m2, v.i_m3);

    var out: VertexPayload;
    out.position = pc.view_projection * model * vec4<f32>(v.position, 1.0);
    out.color = u_color;
    out.normal = (model * vec4<f32>(v.normal, 0.0)).xyz;

    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    let sun_dir = normalize(vec3<f32>(1.0, 1.0, -1.0));
    let light = max(0.0, dot(in.normal, sun_dir));
    return vec4<f32>(light * in.color.rgb, in.color.a);
}
