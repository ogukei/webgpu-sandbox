
use std::sync::Arc;

use super::Shaders;
use crate::render::device::Device;
use crate::render::shader::ShaderModule;

impl Shaders {
    pub fn common(device: &Arc<Device>) -> Arc<ShaderModule> {
        let code = "
struct Uniforms {
    projection_view: mat4x4<f32>,
    camera_position: vec4<f32>,
}
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

struct Object {
    model: mat4x4<f32>,
    padding0: mat4x4<f32>,
    padding1: mat4x4<f32>,
    padding2: mat4x4<f32>,
}
@binding(0) @group(1) var<uniform> object: Object;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) object_normal: vec4<f32>,
    @location(1) object_position: vec4<f32>,
    @location(2) camera_position: vec4<f32>,
}

@vertex
fn vert_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOut {
    var p = vec4<f32>(position, 1.0);
    var n = vec4<f32>(normal, 0.0);
    var out: VertexOut;
    out.position = uniforms.projection_view * object.model * p;
    out.object_normal = object.model * n;
    out.object_position = object.model * p;
    out.camera_position = uniforms.camera_position;
    return out;
}

@fragment
fn frag_main(
    @builtin(position) coord_in: vec4<f32>,
    @location(0) object_normal: vec4<f32>,
    @location(1) object_position: vec4<f32>,
    @location(2) camera_position: vec4<f32>,
) -> @location(0) vec4<f32> {
    var l = normalize(camera_position.xyz - object_position.xyz);
    var n = normalize(object_normal.xyz);
    var d = dot(n, l);
    var intensity = vec3<f32>(max(d, 0.3));
    return vec4<f32>(intensity, 1.0);
}
        ";
        ShaderModule::with_code(device, code)
    }
}
