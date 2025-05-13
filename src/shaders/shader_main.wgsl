//!#include "uniforms.wgsl"
//!#include "shader_resources.wgsl"

//!#define pub
//!#include "shader_definitions.rs"

//!#include "random.wgsl"

//!#include "helpers.wgsl"

//!#include "brdf.wgsl"
//!#include "raytracing.wgsl"

@compute @workgroup_size(WORKGROUP_DIM, WORKGROUP_DIM)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let target_size = textureDimensions(output);

    let x = global_id.x;
    let y = global_id.y;

    rand_init(vec2<u32>(x, y), vec2<u32>(target_size.x, target_size.y), uniforms.camera.frame);

    // Get ray origin and direction
    let ro = uniforms.camera.position;
    let rd = normalize((uniforms.camera.first_pixel_pos + uniforms.camera.pixel_space_x * f32(x) + uniforms.camera.pixel_space_y * f32(y)) - ro);

    var color = pixel_color(ro, rd, acc_struct);

    let prev_color = textureLoad(output, vec2<u32>(x, y));
    //let new_color = vec4<f32>(prev_color.rgb * 0.9 + color.rgb * 0.1, 1.0);
    //let new_color = vec4<f32>(mix(prev_color.rgb, color.rgb, 0.1), 1.0);
    //let new_color = color;
    let new_color = vec4<f32>(((prev_color.rgb * f32(SAMPLES_PER_PIXEL) * (f32(uniforms.camera.frame) - 1)) + (color.rgb * f32(SAMPLES_PER_PIXEL))) / (f32(uniforms.camera.frame) * f32(SAMPLES_PER_PIXEL)), 1.0);

    //var color = ray_color(ro, rd, acc_struct);
    //color = vec4<f32>(rd, 1.0);

    // let pixel_center = vec2<f32>(global_id.xy) + vec2<f32>(0.5);
    // let in_uv = pixel_center / vec2<f32>(target_size.xy);
    // let d = in_uv * 2.0 - 1.0;

    // let origin = vec3<f32>(278.0, 278.0, -800.0);
    // let direction = (vec3<f32>(d, 1.0));

    // var rq: ray_query;
    // rayQueryInitialize(&rq, acc_struct, RayDesc(0u, 0xFFu, 0.1, 10000.0, origin, direction));
    // rayQueryProceed(&rq);

    // var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    // let intersection: RayIntersection = rayQueryGetCommittedIntersection(&rq);
    // if intersection.kind != RAY_QUERY_INTERSECTION_NONE {
    //     //color = vec4<f32>(intersection.barycentrics, 1.0 - intersection.barycentrics.x - intersection.barycentrics.y, 1.0);
    //     color = vec4<f32>(materials[intersection.instance_custom_index].albedo, 1.0);
    //     //color = vec4<f32>(f32(intersection.instance_custom_index)/5.0, f32(intersection.instance_custom_index)/5.0, f32(intersection.instance_custom_index)/5.0, 1.0);
    // }

    //textureStore(output, global_id.xy, vec4<f32>(d, 0.0, 0.0));
    textureStore(output, global_id.xy, new_color);
}