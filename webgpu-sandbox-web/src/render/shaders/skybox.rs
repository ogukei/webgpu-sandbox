
use std::sync::Arc;

use super::Shaders;
use crate::render::device::Device;
use crate::render::shader::ShaderModule;

impl Shaders {
    pub fn skybox(device: &Arc<Device>) -> Arc<ShaderModule> {
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
    // applies sky color (Ray Tracing in One Weekend, 4.2)
    var direction = normalize(normal);
    var t = 0.5 * (direction.y + 1.0);
    var sky = vec3<f32>(0.5, 0.7, 1.0);
    var bottom = vec3<f32>(1.0);
    var diffuse = mix(bottom, sky, t);
    return vec4<f32>(diffuse, 1.0);
}
        ";
        ShaderModule::with_code(device, code)
    }
}
