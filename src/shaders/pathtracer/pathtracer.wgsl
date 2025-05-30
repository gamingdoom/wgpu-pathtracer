//!#define pub
//!#include "../shader_definitions.rs"

//!#include "../common/uniforms.wgsl"
//!#include "common/shader_resources.wgsl"

//!#include "../common/random.wgsl"

//!#include "../common/helpers.wgsl"
//!#include "../common/color.wgsl"

//!#include "../common/bsdf.wgsl"

//!#include "../common/sky.wgsl"

//!#include "common/rt_helpers.wgsl"

//!#include "pathtracer/restir_di.wgsl"
//!#include "pathtracer/pathtracer.wgsl"

@compute @workgroup_size(WORKGROUP_DIM, WORKGROUP_DIM)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let target_size = textureDimensions(output);

    let x = global_id.x;
    let y = global_id.y;

    rand_init(vec3<u32>(x, y, 1), vec2<u32>(target_size.x, target_size.y), uniforms.camera.frame);

    var color = pixel_color(vec2<u32>(x, y), acc_struct);

    var prev_color = max(vec4<f32>(0.0), textureLoad(output, vec2<u32>(x, y)));
    
    var new_color: vec4<f32>;

    if (uniforms.is_grabbed == 0) {
        new_color = vec4<f32>(((prev_color.rgb * prev_color.a) + (color.rgb)) / (f32(prev_color.a + 1.0)), prev_color.a + 1.0);
    } else {
        new_color = vec4<f32>(color.rgb, 1.0);
    }
    
    textureStore(output, global_id.xy, new_color);
}