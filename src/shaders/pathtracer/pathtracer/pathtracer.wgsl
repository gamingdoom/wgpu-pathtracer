//!#pragma once
//!#include "../../common/uniforms.wgsl"
//!#include "../common/shader_resources.wgsl"
//!#include "../../common/random.wgsl"
//!#include "../../common/helpers.wgsl"
//!#include "../../common/bsdf.wgsl"
//!#include "../../common/sky.wgsl"
//!#include "restir_di.wgsl"

struct Fraction {
    numerator: f32,
    denominator: f32
}

fn cosine_pdf(ro_i: vec3<f32>, rd_i: vec3<f32>, scattered: ScatteredRay) -> f32 {
    let cos_theta = dot(normalize(scattered.normal), normalize(scattered.direction));

    return max(0.0, cos_theta);
}

fn triangle_pdf(ro: vec3<f32>, rd: vec3<f32>, tri: Triangle) -> Fraction {
    let intersection: RayIntersectionCustom = trace_ray(ro, rd, acc_struct);

    if intersection.ri.kind == RAY_QUERY_INTERSECTION_NONE {
        return Fraction(0.0, 1.0);
    }

    if equals(intersection.vertices[0].position, tri.v0) && equals(intersection.vertices[1].position, tri.v1) && equals(intersection.vertices[2].position, tri.v2) {
        var n = cross(tri.v1 - tri.v0, tri.v2 - tri.v0);
    
        var area = length(n) * 0.5;

        n = faceForward(normalize(n), rd, n);
        
        var p = fma(rd, vec3<f32>(intersection.ri.t, intersection.ri.t, intersection.ri.t), ro);

        // We hit this triangle
        let d_squared = length(p - ro) * length(p - ro);
        let cosine = abs(dot((p - ro), n) / length(p - ro));

        return Fraction(cosine * area, d_squared);
    }

    return Fraction(0.0, 1.0);

}

fn light_scatter(ro: vec3<f32>, rd: vec3<f32>, tri: Triangle, intersection: RayIntersectionCustom) -> ScatteredRay {
    let to_point = random_point_on_triangle(tri);

    let p = fma(rd, vec3<f32>(intersection.ri.t, intersection.ri.t, intersection.ri.t), ro);

    let v0 = intersection.vertices[0];
    let v1 = intersection.vertices[1];
    let v2 = intersection.vertices[2];
    
    var n = v0.normal;

    var to_p_dir = normalize(to_point - p);

    // if light is behind triangle, then it will not be a hit
    var should_hit = dot(to_p_dir, n) > 0.0;

    let scattered: ScatteredRay = ScatteredRay(
        p,
        to_p_dir,
        intersection.material.albedo,
        n,
        should_hit
    );

    return scattered;
}


fn scatter(ro: vec3<f32>, rd: vec3<f32>, intersection: RayIntersectionCustom) -> ScatteredRay {
    //let p = ro + rd * intersection.t;
    let p = fma(rd, vec3<f32>(intersection.ri.t, intersection.ri.t, intersection.ri.t), ro);

    let v0 = intersection.vertices[0];
    let v1 = intersection.vertices[1];
    let v2 = intersection.vertices[2];
    
    var n = normalize((v0.normal * intersection.uvw.x + v1.normal * intersection.uvw.y + v2.normal * intersection.uvw.z));

    let material = intersection.material;

    let result = disney_bsdf(p, rd, n, material);

    n = faceForward(n, rd, n);

    var scattered: ScatteredRay = ScatteredRay(
        result.p,
        result.ray_outgoing,
        result.color / result.pdf,
        n,
        true
    );

    return scattered;
}

struct SentRay {
    scattered: ScatteredRay,
    scattered_pdf: f32,
    importance: Fraction
};

fn send_ray_regular(ro_i: vec3<f32>, rd_i: vec3<f32>, intersection: RayIntersectionCustom) -> SentRay {
    let scattered = scatter(ro_i, rd_i, intersection);
    let scattered_pdf = 1.0;

    let importance = Fraction(
        1.0,
        1.0
    );
    
    var sr: SentRay = SentRay(
        scattered,
        scattered_pdf, 
        importance
    );

    return sr;
}

fn send_ray_to_light(ro_i: vec3<f32>, rd_i: vec3<f32>, intersection: RayIntersectionCustom, light: Triangle) -> SentRay {

    let scattered = light_scatter(ro_i, rd_i, light, intersection);
    let scattered_pdf = cosine_pdf(ro_i, rd_i, scattered);
    let importance = triangle_pdf(scattered.origin, scattered.direction, light);

    var sr: SentRay = SentRay(
        scattered,
        scattered_pdf,
        importance
    );

    return sr;
}

fn ray_color(ro: vec3<f32>, rd: vec3<f32>, acc_struct: acceleration_structure) -> vec4<f32> {
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
            if i == 0u {
                first_t = intersection.ri.t;
            }

            var scattered: ScatteredRay;
            var importance: Fraction;
            var sent_to_light = false;
            var scattered_pdf: f32;

            if rand_float() < 1.0 - SEND_TO_LIGHT_PROBABILITY {
                let sent_ray = send_ray_regular(curr_ro, curr_rd, intersection);
                scattered = sent_ray.scattered;
                scattered_pdf = sent_ray.scattered_pdf;
                importance = sent_ray.importance;
            } else {
                let light = lights[rand_int() % uniforms.num_lights];
                let sent_ray = send_ray_to_light(curr_ro, curr_rd, intersection, light);
                scattered = sent_ray.scattered;
                scattered_pdf = sent_ray.scattered_pdf;
                importance = sent_ray.importance;

                sent_to_light = true;

                if scattered.hit_prediction == false {
                    let sent_ray = send_ray_regular(curr_ro, curr_rd, intersection);
                    scattered = sent_ray.scattered;
                    scattered_pdf = sent_ray.scattered_pdf;
                    importance = sent_ray.importance;
                    sent_to_light = false;
                }
            }

            curr_ro = scattered.origin;
            curr_rd = scattered.direction;

            let material = intersection.material;

            accumulated_color += material.emissive * color_mask;

            color_mask *= scattered.attenuation;

            if intersection.hit_light == true {
                break;
            }

            //return vec4<f32>(scattered.attenuation, 1.0);
            //return vec4<f32>(curr_rd, 1.0);
            //return vec4<f32>(scattered.normal, 1.0);
            //return vec4<f32>(dot(curr_rd, scattered.normal), dot(curr_rd, scattered.normal), dot(curr_rd, scattered.normal), 1.0);
        } else {
            accumulated_color += sky_color(curr_rd) * color_mask;
            break;
        }
    }

    return vec4<f32>(accumulated_color, first_t);
}

fn pixel_color(xy: vec2<u32>, acc_struct: acceleration_structure) -> vec4<f32> {
    // Get ray origin and direction
    let ro = uniforms.camera.position;
    let rd = normalize(
        (
            uniforms.camera.first_pixel_pos 
            + uniforms.camera.pixel_space_x * (f32(xy.x) + (0.5 * rand_float()))
            + uniforms.camera.pixel_space_y * (f32(xy.y) + (0.5 * rand_float()))
        ) 
        - ro
    );

    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    
    for (var i = 0u; i < SAMPLES_PER_PIXEL; i++) {
        color += ray_color(ro, rd, acc_struct);
    }

    color /= f32(SAMPLES_PER_PIXEL);

    textureStore(depth_output, xy, vec4<f32>(color.a, 0.0, 0.0, 1.0));

    color.w = 1.0;
    
    return color;
}