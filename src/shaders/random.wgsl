//!#pragma once

//!#include "helpers.wgsl"

var<private> rng_state: u32;

fn jenkins_hash(input: u32) -> u32 {
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}

fn pcg_hash() -> u32 {
    rng_state = rng_state * 747796405u + 2891336453u;
    var state = rng_state;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_init(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) {
    // Adapted from https://github.com/boksajak/referencePT
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ jenkins_hash(frame);
    rng_state = jenkins_hash(seed);
}

fn rand_int() -> u32 {
    return pcg_hash();
}

fn rand_float() -> f32 {
    return f32(rand_int()) / f32(0xffffffffu);
}

fn rand_in_unit_sphere() -> vec3<f32> {
    //return normalize(vec3<f32>(rand_float() - 0.5, rand_float() - 0.5, rand_float() - 0.5));

    // let r = pow(rand_float(), 0.33333f);
    // let theta = PI * rand_float();
    // let phi = 2.0 * PI * rand_float();

    // let x = r * sin(theta) * cos(phi);
    // let y = r * sin(theta) * sin(phi);
    // let z = r * cos(theta);

    // return vec3(x, y, z);

    let r1 = rand_float();
    let r2 = rand_float();

    let x = cos(2.0 * PI * r1) * 2.0 * sqrt(r2 * (1.0 - r2));
    let y = sin(2.0 * PI * r1) * 2.0 * sqrt(r2 * (1.0 - r2));
    let z = (1.0 - 2.0 * r2);

    return vec3<f32>(x, y, z);
}

fn rand_in_unit_hemisphere(normal: vec3<f32>) -> vec3<f32> {
    let in_unit_sphere = rand_in_unit_sphere();
    if dot(in_unit_sphere, normal) > 0.0 {
        return in_unit_sphere;
    } else {
        return -in_unit_sphere;
    }
}

fn rand_in_cosine_weighted_hemisphere(normal: vec3<f32>) -> vec3<f32> {
    let u = rand_float();
    let v = rand_float();

    let radial = sqrt(u);
    let theta = 2.0 * PI * v;

    let x = cos(theta) * radial;
    let y = sin(theta) * radial;
    let z = sqrt(1.0 - u);

    let sphere = normalize(vec3<f32>(x, y, z));

    if dot(sphere, normal) > 0.0 {
        return sphere;
    } else {
        return -sphere;
    }
}