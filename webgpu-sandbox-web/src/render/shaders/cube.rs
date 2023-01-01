
use std::sync::Arc;

use super::Shaders;
use crate::render::device::Device;
use crate::render::shader::ShaderModule;

impl Shaders {
    pub fn cube(device: &Arc<Device>) -> Arc<ShaderModule> {
        let code = "
struct Uniforms {
    projection_view: mat4x4<f32>,
}
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
}

@vertex
fn vert_main(
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>
) -> VertexOut {
    var p = vec4<f32>(position, 1.0);
    var out: VertexOut;
    out.position = uniforms.projection_view * p;
    out.normal = position;
    return out;
}

@fragment
fn frag_main(
    @builtin(position) coord_in: vec4<f32>,
    @location(0) normal: vec3<f32>
) -> @location(0) vec4<f32> {
    var light = vec3<f32>(0.5, 5.0, -0.25);
    var l = normalize(light);
    var n = normalize(normal);
    var d = dot(n, l);
    var intensity = vec3<f32>(max(d, 0.5));
    return vec4<f32>(intensity, 1.0);
}
        ";
        ShaderModule::with_code(device, code)
    }
}
