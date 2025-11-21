@group(0) @binding(0) var myTexture: texture_2d<f32>;
@group(0) @binding(1) var mySampler: sampler;
@group(1) @binding(0) var<uniform> model: mat4x4<f32>;

struct PushConsts {
    view_projection: mat4x4<f32>,
};

var<push_constant> pc: PushConsts;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(vertex: VertexIn) -> VertexPayload {
    var out: VertexPayload;
    out.position = pc.view_projection * model * vec4<f32>(vertex.position, 1.0);
    out.tex_coord = vertex.tex_coord;
    out.normal = (model * vec4<f32>(vertex.normal, 0.0)).xyz;
    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    let sun_direction = normalize(vec3<f32>(1.0, 1.0, -1.0));
    let light_strength = max(0.0, dot(in.normal, sun_direction));
    let base = textureSample(myTexture, mySampler, in.tex_coord);
    return vec4<f32>(light_strength * base.rgb, base.a);
}
