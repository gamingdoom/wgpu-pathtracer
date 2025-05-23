//!#pragma once
//!#include "uniforms.wgsl"
//!#include "shader_resources.wgsl"
//!#include "random.wgsl"
//!#include "helpers.wgsl"
//!#include "brdf.wgsl"
//!#include "bsdf.wgsl"
//!#include "sky.wgsl"
//!#include "restir_di.wgsl"

struct ScatteredRay {
    origin: vec3<f32>,
    direction: vec3<f32>,
    attenuation: vec3<f32>,
    normal: vec3<f32>,
    hit_prediction: bool,
};

struct RayIntersectionCustom {
    ri: RayIntersection,
    //hit_vertex_positions:  array<vec3<f32>, 3>,
    vertices: array<Vertex, 3>,
    material: SampledMaterial,
    hit_light: bool,
    uvw: vec3<f32>,
    //rq: ptr<private, ray_query>
}

struct Fraction {
    numerator: f32,
    denominator: f32
}

fn sample_material(material: Material, uv: vec2<f32>) -> SampledMaterial {
    let rgba = sample_texture_rgba(material.albedo_texture_idx, uv);

    return SampledMaterial (
        rgba.rgb,
        rgba.a,

        sample_texture_float(material.roughness_texture_idx, uv),

        sample_texture_color(material.specular_texture_idx, uv),
        sample_texture_float(material.metallic_texture_idx, uv),

        sample_texture_color(material.emissive_texture_idx, uv),
        sample_texture_float(material.sheen_texture_idx, uv),

        material.clearcoat_thickness,
        material.clearcoat_roughness,
        material.anisotropy,
        material.anisotropy_rotation,

        sample_texture_float(material.transmission_texture_idx, uv),
        sample_texture_float(material.ior_texture_idx, uv)
    );
}

fn trace_ray(ro: vec3<f32>, rd: vec3<f32>, acc_struct: acceleration_structure) -> RayIntersectionCustom {
    var rq: ray_query;

    // Flags: 
    //cull back facing -> 0x10
    rayQueryInitialize(&rq, acc_struct, RayDesc(0u, 0xFFu, 0.001, 10000.0, ro, rd));
    rayQueryProceed(&rq);

    var rq_intersection: RayIntersection = rayQueryGetCommittedIntersection(&rq);

    var verts: array<Vertex, 3>;

    if (rq_intersection.kind != RAY_QUERY_INTERSECTION_NONE) {
       verts[0] = vertices[indices[instance_infos[rq_intersection.instance_index].index_offset + rq_intersection.geometry_index * 3 + 0]];
       verts[1] = vertices[indices[instance_infos[rq_intersection.instance_index].index_offset + rq_intersection.geometry_index * 3 + 1]];
       verts[2] = vertices[indices[instance_infos[rq_intersection.instance_index].index_offset + rq_intersection.geometry_index * 3 + 2]];

    //    if equals(verts[0].normal, vec3<f32>(0.0, 0.0, 0.0)) {
    //         // calcualte normals
    //         // normalize(cross(v1 - v0, v2 - v0))
    //         let normal = normalize(cross(verts[1].position - verts[0].position, verts[2].position - verts[0].position));
    //         verts[0].normal = normal;
    //         verts[1].normal = normal;
    //         verts[2].normal = normal;
    //    }
    }

    let w = 1.0 - rq_intersection.barycentrics.x - rq_intersection.barycentrics.y;

    //let uv = rq_intersection.barycentrics.x * verts[1].uv + rq_intersection.barycentrics.y * verts[2].uv + w * verts[0].uv;

    let bary = vec3<f32>(w, rq_intersection.barycentrics.x, rq_intersection.barycentrics.y);
    let uv = verts[0].uv * bary.x + verts[1].uv * bary.y + verts[2].uv * bary.z;
    //let uv = rq_intersection.barycentrics.x * verts[2].uv + rq_intersection.barycentrics.y * verts[1].uv + w * verts[0].uv;

    let material = sample_material(materials[rq_intersection.instance_custom_data], uv);

    let emissive = material.emissive;
    var hit_light = bool(dot(ceil(emissive), vec3<f32>(1.0)));
    // if emissive.x > 0.0 || emissive.y > 0.0 || emissive.z > 0.0 {
    //     hit_light = true; 
    // }

    var intersection: RayIntersectionCustom = RayIntersectionCustom(
        rq_intersection,
        verts,
        material,
        hit_light,
        vec3<f32>(w, rq_intersection.barycentrics.x, rq_intersection.barycentrics.y),
        //&rq
    );

    return intersection;
}

fn cosine_pdf(ro_i: vec3<f32>, rd_i: vec3<f32>, scattered: ScatteredRay) -> f32 {
    let cos_theta = dot(normalize(scattered.normal), normalize(scattered.direction));

    // if (cos_theta < 0.0) {
    //     return 0.0;
    // }

    return max(0.0, cos_theta);

    // var n = cross(tri.v1 - tri.v0, tri.v2 - tri.v0);

    // var area = length(n) * 0.5;

    // n = faceForward(normalize(n), rd, n);
    
    //var p = fma(rd, vec3<f32>(intersection.ri.t, intersection.ri.t, intersection.ri.t), ro);

    //return length(p - ro) * length(p - ro) / 100000.0;

    // // We hit this triangle
    // let d_squared = length(p - ro) * length(p - ro);//intersection.ri.t * intersection.ri.t * dot((p - ro), (p - ro));
    // let cosine = abs(dot((p - ro), n) / length(p - ro));

    //return cosine;

    //return (cosine * area) / d_squared;
    // return cosine;

    // let intersection: RayIntersectionCustom = trace_ray(ro, rd, acc_struct);

    // if intersection.ri.kind == RAY_QUERY_INTERSECTION_NONE {
    //     return 0.0;
    // }

    // let tri: Triangle = Triangle(intersection.hit_vertex_positions[0], intersection.hit_vertex_positions[1], intersection.hit_vertex_positions[2]);

    // var n = cross(tri.v1 - tri.v0, tri.v2 - tri.v0);

    // var area = length(n) * 0.5;

    // n = faceForward(normalize(n), rd, n);
    
    // var p = fma(rd, vec3<f32>(intersection.ri.t, intersection.ri.t, intersection.ri.t), ro);

    // //return length(p - ro) * length(p - ro) / 100000.0;

    // // We hit this triangle
    // let d_squared = length(p - ro) * length(p - ro);//intersection.ri.t * intersection.ri.t * dot((p - ro), (p - ro));
    // let cosine = abs(dot((p - ro), n) / length(p - ro));

    // //return cosine;

    // //return (cosine * area) / d_squared;
    // return cosine;
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

        //return length(p - ro) * length(p - ro) / 100000.0;

        // We hit this triangle
        let d_squared = length(p - ro) * length(p - ro);//intersection.ri.t * intersection.ri.t * dot((p - ro), (p - ro));
        let cosine = abs(dot((p - ro), n) / length(p - ro));

        //return cosine;

        //return (cosine * area) / d_squared;
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

    //var n = normalize(cross(v1 - v0, v2 - v0));
    
    var n = v0.normal;

    //n = faceForward(n, , );

    var to_p_dir = normalize(to_point - p);

    // if light is behind triangle, then it will not be a hit
    var should_hit = dot(to_p_dir, n) > 0.0;

    let scattered: ScatteredRay = ScatteredRay(
        p,
        to_p_dir,
        intersection.material.albedo,
        //brdf(p, rd, to_p_dir, n, materials[intersection.ri.instance_custom_data]),
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

    //var n = normalize(cross(v1 - v0, v2 - v0));
    
    var n = normalize((v0.normal * intersection.uvw.x + v1.normal * intersection.uvw.y + v2.normal * intersection.uvw.z));
    n = faceForward(n, n, rd);
    // if dot(n, -rd) < 0.0 {
    //     n *= -1.0;
    // }

    // if dot(rd, n) < 0.0 {
    //     n *= -1.0;
    // }
    
    //n = faceForward(n, -rd, n);
    //n.z *= -1.0;

    //let ray_outgoing = normalize(n + rand_in_unit_sphere());
    //let ray_outgoing = rand_in_cosine_weighted_hemisphere(n);
    //let ray_outgoing = reflect(rd, n);

    // let ray_outgoing = mix(
    //     rand_in_cosine_weighted_hemisphere(n), 
    //     reflect(rd, n), 
    //     (ggx_sample(materials[intersection.ri.instance_custom_data].roughness * materials[intersection.ri.instance_custom_data].roughness, n, rd) + (PI / 2.0)) / PI
    // );
    //var ray_outgoing: vec3<f32> = ggx_sample(materials[intersection.ri.instance_custom_data].roughness * materials[intersection.ri.instance_custom_data].roughness, n, rd); 
    // if rand_float() < specular_probability(materials[intersection.ri.instance_custom_data], rd, n) {
    //    ray_outgoing = reflect(rd, n);
    // } else {
    //var ray_outgoing = rand_in_cosine_weighted_hemisphere(n);
    // }

    //ray_outgoing = ggx_sample(materials[intersection.ri.instance_custom_data].roughness * materials[intersection.ri.instance_custom_data].roughness, n, rd);//rand_in_cosine_weighted_hemisphere(n);

    //var ggx = ggx_sample(materials[intersection.ri.instance_custom_data].roughness * materials[intersection.ri.instance_custom_data].roughness, n, rd);
    // var ray_outgoing = mix(
    //     reflect(rd, n), 
    //     rand_in_cosine_weighted_hemisphere(n), 
    //     saturate(ggx)
    // );

    let material = intersection.material;

    //let ray_outgoing = rand_in_cosine_weighted_hemisphere(n);

    //let result = disney_brdf(p, rd, n, material);
    let result = disney_bsdf(p, rd, n, material);

    var scattered: ScatteredRay = ScatteredRay(
        p,
        result.ray_outgoing,//ray_outgoing,//rand_in_cosine_weighted_hemisphere(n)), //rand_in_unit_hemisphere(n),
        result.color / result.pdf,//0.5 * n + 0.5,//brdf(p, rd, ray_outgoing, n, materials[intersection.ri.instance_custom_data]),
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
    let scattered_pdf = 1.0;//dot(scattered.normal, -rd_i);

    let importance = Fraction(
        1.0,
        1.0
        // ggx_sample_pdf(
        //     materials[intersection.ri.instance_custom_data].roughness * materials[intersection.ri.instance_custom_data].roughness,
        //     scattered.normal,
        //     -rd_i + scattered.direction,
        //     rd_i
        // )
        // ggx(
        //     materials[intersection.ri.instance_custom_data].roughness * materials[intersection.ri.instance_custom_data].roughness,
        //     dot(scattered.normal, -rd_i + scattered.direction)
        // ) 
        // / (4.0 * dot(scattered.direction, -rd_i + scattered.direction))
    );
    
    var sr: SentRay = SentRay(
        scattered,
        scattered_pdf, 
        importance
    );

    return sr;

    // scattered = scatter(curr_ro, curr_rd, intersection);

    // scattered_pdf = 1.0;
    // importance = Fraction(1.0, 1.0);//Fraction(1.0, 2.0 * PI);
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

    // scattered = light_scatter(curr_ro, curr_rd, light, intersection);
    
    // importance = triangle_pdf(scattered.origin, scattered.direction, light);
    // scattered_pdf = cosine_pdf(curr_ro, curr_rd, scattered);

    // sent_to_light = true;  
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
                
                // scattered = scatter(curr_ro, curr_rd, intersection);

                // scattered_pdf = 1.0;
                // importance = Fraction(1.0, 1.0);//Fraction(1.0, 2.0 * PI);
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

                // scattered = light_scatter(curr_ro, curr_rd, light, intersection);
                
                // importance = triangle_pdf(scattered.origin, scattered.direction, light);
                // scattered_pdf = cosine_pdf(curr_ro, curr_rd, scattered);

                // sent_to_light = true;  
            }

            curr_ro = scattered.origin;
            curr_rd = scattered.direction;

            let material = intersection.material;

            accumulated_color += material.emissive * color_mask;

            //color_mask *= scattered.attenuation * scattered_pdf * importance.numerator / importance.denominator;
            //color_mask *= scattered.attenuation * scattered_pdf;
            color_mask *= scattered.attenuation;

            if intersection.hit_light == true {
                break;
            }

            //return vec4<f32>(scattered.attenuation, 1.0);
            //return vec4<f32>(curr_rd, 1.0);
            //return vec4<f32>(scattered.normal, 1.0);
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

    var color = vec4<f32>(0.0, 0.0, 0.0, 1000.0);
    
    for (var i = 0u; i < SAMPLES_PER_PIXEL; i++) {
        color += ray_color(ro, rd, acc_struct);
        //color += ray_color_restir_di(ro, rd, acc_struct);
    }

    color /= f32(SAMPLES_PER_PIXEL);

    textureStore(depth_output, xy, vec4<f32>(color.a, 0.0, 0.0, 1.0));

    color.w = 1.0;

    //color = vec4<f32>(sqrt(color.rgb), 1.0);
    //color = vec4<f32>(color.rgb * color.rgb, 1.0);

    //color = vec4<f32>(to_khronos_pbr_neutral(color.rgb), 1.0);
    
    return color;
}