// https://sayan1an.github.io/pdfs/references/disneyBrdf.pdf

struct BSDFData {
    p: vec3<f32>, 
    // ray_incoming: vec3<f32>, 
    v: vec3<f32>,
    ray_outgoing: vec3<f32>, 
    normal: vec3<f32>, 
    h: vec3<f32>,

    material: SampledMaterial,
    alpha_x: f32,
    alpha_y: f32,
    diffuse_reflectance: vec3<f32>,

    f0: vec3<f32>,
    f_schlick: vec3<f32>,

    ndotv: f32,
    ndotl: f32,
    ldoth: f32,
    vdoth: f32,
    ndoth: f32,

    tangent: vec3<f32>,
    bitangent: vec3<f32>,

}

fn bsdf_prep_data(
    p: vec3<f32>, 
    ray_incoming: vec3<f32>, 
    normal: vec3<f32>, 
    material: SampledMaterial
) -> BSDFData {
    var data = BSDFData();

    data.p = p;
    // data.ray_incoming = ray_incoming;
    data.v = -ray_incoming;
    data.normal = normal;

    data.material = material;
    data.alpha_x = material.roughness * material.roughness;
    //data.alpha_x = material.roughness;
    data.alpha_y = data.alpha_x;
    
    data.diffuse_reflectance = material.albedo * (1.0 - material.metallic);

    data.ndotv = dot(data.normal, data.v);

    data.f0 = mix(vec3<f32>(F90), material.albedo, material.metallic);
    data.f_schlick = mix(data.f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - data.ndotv, 5.0));

    data.tangent = normalize(cross(data.normal, data.v));
    data.bitangent = -normalize(cross(data.normal, data.tangent));

    return data;
}

fn bsdf_ggx_sample_microfacet_normal(
    data: BSDFData
) -> vec3<f32> {
    // https://agraphicsguynotes.com/posts/sample_microfacet_brdf/
    // theta = arctan(alpha * sqrt((epsilon)/(1 - epsilon)))
    // phi can be randomly sampled

    let tangent: vec3<f32> = normalize(cross(data.normal, data.v));
    let bitangent = -normalize(cross(data.normal, tangent));

    let epsilon = rand_float();

    let angle_from_normal = (PI / 2.0) - atan(data.alpha_x * sqrt(epsilon / (1.0 - epsilon)));
    let theta = rand_float() * 2.0 * PI;

    let local_x = cos(theta) * cos(angle_from_normal);
    let local_y = sin(angle_from_normal);
    let local_z = sin(theta) * cos(angle_from_normal);

    // let local_microfacet_normal = vec3<f32>(local_x, local_y, local_z);

    // // bitangent: x-axis
    // // normal: y-axis
    // // tangent: z-axis

    // // Rotate local_rd_o to world space
    // let X = vec3<f32>(1.0, 0.0, 0.0);
    // let Y = vec3<f32>(0.0, 1.0, 0.0);
    // let Z = vec3<f32>(0.0, 0.0, 1.0);
    
    //let world_microfacet_normal = dot(local_microfacet_normal, X) * bitangent + dot(local_microfacet_normal, Y) * normal + dot(local_microfacet_normal, Z) * tangent;
    let world_microfacet_normal = local_x * bitangent + local_y * data.normal + local_z * tangent;

    return world_microfacet_normal;

    //return reflect(v, world_microfacet_normal);
}

fn calculate_tint(
    color: vec3<f32>,
) -> vec3<f32> {
    return color / luminance(color);
}

fn get_schlick_weight(
    adotb: f32
) -> f32 {
    let one_minus_adotb = 1.0 - adotb;

    return one_minus_adotb * one_minus_adotb * one_minus_adotb * one_minus_adotb * one_minus_adotb;
}

fn get_schlick_fresnel(
    adotb: f32
) -> f32 {
    return F90 + (1.0 - F90) * get_schlick_weight(adotb);
}

fn to_local_space(
    input: vec3<f32>,
    data: BSDFData
) -> vec3<f32> {
    return vec3<f32>(
        dot(input, data.bitangent),
        dot(input, data.normal),
        dot(input, data.tangent)
    );
}

fn bsdf_sheen(
    data: BSDFData
) -> vec3<f32> {

    let tint = calculate_tint(data.diffuse_reflectance);

    // TODO: sheen tint instead of 0.5
    return data.material.sheen * mix(vec3<f32>(1.0), tint, 0.5) * get_schlick_weight(data.ldoth);
}

fn gtr_1(
    adotb: f32,
    a: f32
) -> f32 {
    let a_sq = a * a;

    return (a_sq - 1.0) / (PI * log2(a_sq) * (1.0 + (a_sq - 1.0) * adotb * adotb));
}

fn separable_smith_GGX_G1(
    adotb: f32,
    a: f32
) -> f32 {
    let a_sq = a * a;
    
    return 2.0 / (1.0 + sqrt(a_sq + (1.0 - a_sq) * adotb * adotb));
}

fn bsdf_clearcoat(
    data: BSDFData
) -> f32 {
    let d = gtr_1(data.ndoth, data.material.clearcoat_roughness * data.material.clearcoat_roughness);
    let f = get_schlick_fresnel(data.ldoth);
    let gv = separable_smith_GGX_G1(data.ndotv, 0.25);
    let gl = separable_smith_GGX_G1(data.ndotl, 0.25);

    return 0.25 * data.material.clearcoat_thickness * d * f * gv * gl;
}

fn bsdf_diffuse(
    data: BSDFData
) -> vec3<f32> {
    let f_d90 = 0.5 + 2.0 * data.material.roughness * (data.ldoth * data.ldoth);

    let one_minus_ndotv = 1.0 - data.ndotv;
    let f_d_in = 1.0 + (f_d90 - 1.0) * one_minus_ndotv * one_minus_ndotv * one_minus_ndotv * one_minus_ndotv * one_minus_ndotv;

    let one_minus_ndotl = 1.0 - data.ndotl;
    let f_d_out = 1.0 + (f_d90 - 1.0) * one_minus_ndotl * one_minus_ndotl * one_minus_ndotl * one_minus_ndotl * one_minus_ndotl;

    let base_diffuse = data.diffuse_reflectance * f_d_in * f_d_out * data.ndotl; 

    // TODO subsurface

    return base_diffuse;
}

fn bsdf_smith_ggx(
    data: BSDFData,
    hdots: f32
) -> f32 {
    return 1.0 / (1.0 + lambda_ggx(
        (hdots)
        /
        (data.alpha_x * sqrt(1.0 - hdots * hdots)))
    );
}

// fn bsdf_ggx(
//     data: BSDFData
// ) -> f32 {
//     // alpha^2 / (pi * ((alpha^2 - 1) * dot(normal, halfway)^2 + 1)^2)
//     let b = (data.alpha_x * data.alpha_x - 1.0) * (data.ndoth * data.ndoth) + 1.0;
//     return (data.alpha_x * data.alpha_x) / (PI * b * b);
// }

// fn bsdf_ggx_sample_pdf(
//     data: BSDFData
// ) -> f32 {
//     return (bsdf_ggx(data) * bsdf_smith_ggx(data, data.vdoth)) / (4.0 * data.ndotv);
// }

fn bsdf_ggx_sample_vndf_microfacet_normal(
    data: BSDFData
) -> vec3<f32> {
    let u1 = rand_float();
    let u2 = rand_float();

    let ve = to_local_space(data.v, data);

    let vh = normalize(vec3<f32>(data.alpha_x * ve.x, data.alpha_y * ve.y, ve.z));

    let lensq = vh.x * vh.x + vh.y * vh.y;
    let T1 = vec3<f32>(-vh.y, vh.x, 0.0) * inverseSqrt(lensq);
    let T2 = cross(vh, T1);

    let r = sqrt(u1);
    let phi = 2.0 * PI * u2;
    let t1 = r * cos(phi);
    var t2 = r * sin(phi);
    let s = 0.5 * (1.0 + vh.z);
    t2 = (1.0 - s) * sqrt(1.0 - t1 * t1) + s * t2;

    let nh = t1 * T1 + t2 * T2 + sqrt(1.0 - t1 * t1 - t2 * t2) * vh;

    let ne = normalize(vec3<f32>(data.alpha_x * nh.x, data.alpha_y * nh.y, nh.z));

    return ne;

    // let world_microfacet_normal = ne.x * data.bitangent + ne.y * data.normal + ne.z * data.tangent;

    // return world_microfacet_normal;
}

fn bsdf_ggx_distribution(
    h: vec3<f32>,
    data: BSDFData
) -> f32 {
    // //let local_h = to_local_space(h, data);
    // let local_h = h;

    // let subfunc = (local_h.x * local_h.x) / (data.alpha_x * data.alpha_x) + (local_h.y * local_h.y) / (data.alpha_y * data.alpha_y) + (local_h.z * local_h.z);
    // let d_m = 1.0 / (PI * data.alpha_x * data.alpha_y * subfunc * subfunc);

    // return d_m;

    let a_sq = data.alpha_x * data.alpha_y;
    let pre_sq = (a_sq - 1.0) * dot(data.normal, h) * dot(data.normal, h) + 1.0;
    return (a_sq * dot(data.normal, h)) / (PI * pre_sq * pre_sq);
}


fn bsdf_vndf_pdf(
    ne: vec3<f32>,
    data: BSDFData
) -> f32 {
    let local_ne = data.h;//to_local_space(ne, data);
    let ve = data.v;//to_local_space(data.v, data);

    return bsdf_smith_ggx(data, dot(ne, ve)) * max(0.0, dot(ve, ne)) * gtr_1(dot(local_ne, ve), data.alpha_x); // ve.z;
}



fn bsdf_metal(
    data: BSDFData
) -> vec3<f32> {
    let f_m = data.diffuse_reflectance + (1.0 - data.diffuse_reflectance) * get_schlick_weight(data.ndotv);

    // let subfunc = (data.h.x * data.h.x) / (data.alpha_x * data.alpha_x) + (data.h.y * data.h.y) / (data.alpha_y * data.alpha_y) + (data.h.z * data.h.z);
    // let d_m = 1.0 / (PI * data.alpha_x * data.alpha_y * subfunc * subfunc);

    let d_m = bsdf_ggx_distribution(data.h, data);

    let g_m = bsdf_smith_ggx(data, data.ldoth) * bsdf_smith_ggx(data, data.vdoth);
    //let g_m = bsdf_smith_ggx_masking_shadowing(data);
    //let g_m = separable_smith_GGX_G1(data.ldoth, data.alpha_x) * separable_smith_GGX_G1(data.vdoth, data.alpha_x);

    // return (f_m * d_m * g_m) / (4.0 * data.ndotv * data.ndotl);

    return (f_m * g_m * 4.0);// / ((dot(data.v, data.normal) * dot(data.h, data.normal)) * 4.0);
}

struct BSDFSample {
    h: vec3<f32>,
    outgoing: vec3<f32>,
    pdf: f32,
}

fn bsdf_ray_outgoing_from_h(
    h: vec3<f32>,
    data: BSDFData
) -> vec3<f32> {
    return reflect(-data.v, h);
}


fn bsdf_sample(
    data: BSDFData
) -> BSDFSample {
    let u1 = rand_float();
    let a_sq = data.alpha_x * data.alpha_y;
    
    let up_down_angle = atan(data.alpha_x * sqrt(u1 / (1.0 - u1)));
    let left_right_angle = rand_float() * 2.0 * PI;

    let cos_up_down = cos(up_down_angle);
    let sin_up_down = sin(up_down_angle);
    let cos_left_right = cos(left_right_angle);
    let sin_left_right = sin(left_right_angle);

    // let local_x = cos_left_right * cos_up_down;
    // let local_y = sin_up_down;
    // let local_z = sin_left_right * cos_up_down;

    let local_x = sin_up_down * cos_left_right;
    let local_y = cos_up_down;
    let local_z = sin_up_down * sin_left_right;

    let world_microfacet_normal = normalize(local_x * data.bitangent + local_y * data.normal + local_z * data.tangent);

    // let pre_sq = (a_sq - 1.0) * cos_up_down * cos_up_down + 1.0;
    // let h_pdf = (a_sq * cos_up_down * sin_up_down) / (PI * pre_sq * pre_sq);
    // //let pdf = h_pdf * 4.0 / ( dot(data.v, world_microfacet_normal));

    // let pdf = h_pdf / dot(data.normal, world_microfacet_normal);

    //let pdf = bsdf_smith_ggx(data, dot(world_microfacet_normal, data.v)) / h_pdf * 4.0 * dot(data.v, world_microfacet_normal);
    //let pdf = h_pdf * 4.0 * dot(data.v, world_microfacet_normal);
    let outgoing = bsdf_ray_outgoing_from_h(world_microfacet_normal, data);

    // let pdf = 
    //     (
    //         dot(data.v, world_microfacet_normal) 
    //         * bsdf_smith_ggx(data, dot(world_microfacet_normal, data.v)) 
    //         * bsdf_smith_ggx(data, dot(world_microfacet_normal, outgoing))
    //     )
    //     /
    //     (
    //         dot(data.v, data.normal) 
    //         * dot(world_microfacet_normal, data.normal)
    //     );

    // let pdf = 1.0 / (dot(data.v, data.normal) * dot(world_microfacet_normal, data.normal));

    // let pdf = (dot(outgoing, data.normal) * dot(world_microfacet_normal, data.normal)) / (dot(outgoing, world_microfacet_normal));

    // let cos_theta = dot(data.normal, world_microfacet_normal);
    // let exp = (a_sq - 1.0) * cos_theta * cos_theta + 1.0;
    // let D = a_sq / (PI * exp * exp);
    // let pdf = (D * dot(data.normal, world_microfacet_normal)) / (2.0 * dot(data.normal, outgoing));

    //let pdf = bsdf_ggx_distribution(world_microfacet_normal, data) / (4.0 * dot(data.v, world_microfacet_normal) * dot(data.normal, world_microfacet_normal));

    //let pdf = (dot(data.v, data.normal) * dot(world_microfacet_normal, data.normal)) * 4.0;
    let pdf = 2.0;

    //return vec4<f32>(world_microfacet_normal, pdf);
    return BSDFSample(
        world_microfacet_normal,
        outgoing,
        pdf
    );

    // let theta = atan(data.alpha_x * sqrt(u1 / (1.0 - u1)));

    // let local_x = r * cos(theta);
    // let local_z = r * sin(theta);
    // let local_y = sqrt(1.0 - local_x * local_x - local_z * local_z);

    // let world_microfacet_normal = normalize(local_x * data.bitangent + local_y * data.normal + local_z * data.tangent);

    // let pre_sq = (a_sq - 1.0) * cos(theta) * cos(theta) + 1.0;
    // let h_pdf = (a_sq * cos(theta) * sin(theta)) / (PI * pre_sq * pre_sq);
    // let pdf = h_pdf / (4.0 * dot(data.v, world_microfacet_normal));

    // return vec4<f32>(world_microfacet_normal, pdf);
}

fn bsdf_smith_ggx_masking_shadowing(
    data: BSDFData
) -> f32 {
    let a_sq = data.alpha_x * data.alpha_y;

    let denomA = dot(data.normal, data.v) * sqrt(a_sq + (1.0 - a_sq) * dot(data.normal, data.ray_outgoing) * dot(data.normal, data.ray_outgoing));
    let denomB = dot(data.normal, data.ray_outgoing) * sqrt(a_sq + (1.0 - a_sq) * dot(data.normal, data.v) * dot(data.normal, data.v));

    return 2.0 * dot(data.normal, data.ray_outgoing) * dot(data.normal, data.v) / (denomA + denomB);
}

fn disney_bsdf(
    p: vec3<f32>, 
    ray_incoming: vec3<f32>, 
    normal: vec3<f32>, 
    material: SampledMaterial
) -> BRDFResult {
    var data = bsdf_prep_data(p, ray_incoming, normal, material);

    let u = rand_float();

    var color: vec3<f32>;
    var pdf: f32;

    // let diffuse_weight = (1.0 - data.material.metallic) * (1.0 - data.material.transmission_weight);
    // let metal_weight = (1.0 - data.material.transmission_weight * data.material.metallic);

    // let p_d = diffuse_weight / (diffuse_weight + metal_weight);

    // if u < p_d {
    //     data.ray_outgoing = rand_in_cosine_weighted_hemisphere(data.normal);

    //     data.h = normalize(data.ray_outgoing + data.v);

    //     pdf = dot(data.normal, data.ray_outgoing);
    // } else {
    //     data.h = bsdf_ggx_sample_vndf_microfacet_normal(data);

    //     data.ray_outgoing = -reflect(data.v, data.h);

    //     pdf = 1.0;// / bsdf_vndf_pdf(data.h, data);
    // }

    //data.ray_outgoing = rand_in_cosine_weighted_hemisphere(data.normal);

    //data.h = normalize(data.ray_outgoing + data.v);

    let sample = bsdf_sample(data);
    data.h = sample.h;
    data.ray_outgoing = sample.outgoing;
    pdf = sample.pdf;

    //data.h = bsdf_ggx_sample_vndf_microfacet_normal(data);

    //data.ray_outgoing = -reflect(data.v, data.h);

    //pdf = 1.0;// / bsdf_vndf_pdf(data.h, data);
    //pdf = dot(data.normal, data.ray_outgoing);
    //pdf = 1.0 / ( data.ndotv);
    
    data.ndotl = dot(data.normal, data.ray_outgoing);
    data.ldoth = dot(data.ray_outgoing, data.h);
    data.ndoth = dot(data.normal, data.h);    
    data.vdoth = dot(data.v, data.h);

    color = 
        0.0 
        + (1.0 - data.material.transmission_weight) * (1.0 - data.material.metallic) * bsdf_diffuse(data)
    //    + (1.0 - data.material.metallic) * data.material.sheen * bsdf_sheen(data)
        + (1.0 - data.material.transmission_weight * data.material.metallic) * bsdf_metal(data)
    //    + 0.25 * data.material.clearcoat_thickness * bsdf_clearcoat(data)
        ;

    //color = data.h;
    //pdf = (4.0 * data.vdoth);
    // pdf = dot(data.normal, data.v);

    //pdf = dot(data.normal, data.ray_outgoing);

    // color = vec3<f32>(pdf);
    //pdf = 1.0;// / dot(data.normal, data.ray_outgoing);

    //color = bsdf_reflectance(data);
    //pdf = 1.0 / bsdf_reflectance(data);

    let rtn: BRDFResult = BRDFResult(
        color,
        pdf,
        data.ray_outgoing  
    );

    return rtn;
}