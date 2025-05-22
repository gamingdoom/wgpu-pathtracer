//!#pragma once
//!#include "uniforms.wgsl"
//!#include "shader_resources.wgsl"
//!#include "random.wgsl"
//!#include "helpers.wgsl"
//!#include "brdf.wgsl"
//!#include "sky.wgsl"

struct Path {
    x1: vec3<f32>,
    x2: vec3<f32>,
    x3: vec3<f32>
}

struct Sample {
    x: Path,
    w: f32
}

fn ray_color_restir_di(ro: vec3<f32>, rd: vec3<f32>, acc_struct: acceleration_structure) -> vec4<f32> {
    var curr_ro = ro;
    var curr_rd = rd;

    var termination_depth = 0u;

    var color_mask = vec3<f32>(1.0, 1.0, 1.0);
    var accumulated_color = vec3<f32>(0.0, 0.0, 0.0);
    var first_t: f32 = 0.0;

    for (var i = 0u; i <= MAX_BOUNCES; i++) {
        termination_depth += 1u;

        let intersection: RayIntersectionCustom = trace_ray(curr_ro, curr_rd, acc_struct);

        if intersection.ri.kind != RAY_QUERY_INTERSECTION_NONE {
            let light = lights[rand_int() % uniforms.num_lights];

            if i == 0u {
                first_t = intersection.ri.t;
            }

            accumulated_color += intersection.material.emissive * color_mask;

            if intersection.hit_light == true {
                break;
            }

            let v0 = intersection.vertices[0];
            let v1 = intersection.vertices[1];
            let v2 = intersection.vertices[2];

            var n = (v0.normal * intersection.uvw.x + v1.normal * intersection.uvw.y + v2.normal * intersection.uvw.z);
            var area = length(n) * 0.5;

            n = normalize(n);

            let next_ro = fma(curr_rd, vec3<f32>(intersection.ri.t, intersection.ri.t, intersection.ri.t), curr_ro);

            let on_light = random_point_on_triangle(light);

            let to_light = on_light - next_ro;
            


            // if dot(n, -curr_rd) < 0.0 {
            //     n *= -1.0;
            // }

            let d_squared = dot(to_light, to_light);

            let next_rd = normalize(to_light);

            let cosine = dot(next_rd, n);

            let brdf = disney_brdf_sample(next_ro, curr_rd, next_rd, n, intersection.material);

            curr_rd = next_rd;
            curr_ro = next_ro;

            color_mask *= brdf.color * (1.0 / d_squared) * area * cosine;

            // return vec4<f32>(n, 1.0);

        } else {
            accumulated_color += sky_color(curr_rd) * color_mask;
            break;
        }
    }

    return vec4<f32>(accumulated_color, first_t);
}
