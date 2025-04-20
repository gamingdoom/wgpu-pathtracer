//!#include "uniforms.wgsl"

struct BRDFData {
    p: vec3<f32>, 
    ray_incoming: vec3<f32>, 
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

}

// fn diffuse(
//     p: vec3<f32>, 
//     ray_incoming: vec3<f32>, 
//     ray_outgoing: vec3<f32>, 
//     normal: vec3<f32>, 
//     material: SampledMaterial
// ) -> f32 {
//     let h = normalize(ray_incoming + ray_outgoing);
//     let theta_v = dot(normal, ray_incoming);
//     let theta_l = dot(normal, ray_outgoing);
//     let theta_d = dot(ray_outgoing, h);

//     let f_d90 = 0.5 + 2.0 * material.roughness * (theta_d * theta_d);

//     return mix(1.0, f_d90, pow(1.0 - theta_l, 5.0)) * mix(1, f_d90, pow(1.0 - theta_v, 5.0));

//     // // 0.5 + 2(roughness * cos^2(incoming dot halfway))
//     // let f90 = 0.5 + 2.0 * (material.roughness * (dot(ray_outgoing, h) * dot(ray_outgoing, h)));
//     // let f0 = 1.0;

//     // let f_sheen = pow(1.0 - dot(ray_outgoing, h), 5.0);

//     // // f_diffuse = lerp(f0, f90, normal dot incoming) * lerp(f0, f90, outgoing dot incoming) + f_sheen
//     // let f_diffuse = 
//     //     mix(f0, f90, dot(normal, ray_outgoing)) 
//     //     * mix(f0, f90, dot(ray_outgoing, ray_incoming)) 
//     //     + (f_sheen * material.sheen);

//     // return f_diffuse * material.albedo * (1.0 - material.metallic);
// }

fn diffuse(
    data: BRDFData
) -> f32 {
    // let h = normalize(ray_incoming + ray_outgoing);
    // let theta_v = dot(normal, ray_incoming);
    // let theta_l = dot(normal, ray_outgoing);
    // let theta_d = dot(ray_outgoing, h);

    let f_d90 = 0.5 + 2.0 * data.material.roughness * (data.ldoth * data.ldoth);

    return mix(1.0, f_d90, pow(1.0 - data.ndotl, 5.0)) * mix(1, f_d90, pow(1.0 - data.ndotv, 5.0));

    // // 0.5 + 2(roughness * cos^2(incoming dot halfway))
    // let f90 = 0.5 + 2.0 * (material.roughness * (dot(ray_outgoing, h) * dot(ray_outgoing, h)));
    // let f0 = 1.0;

    // let f_sheen = pow(1.0 - dot(ray_outgoing, h), 5.0);

    // // f_diffuse = lerp(f0, f90, normal dot incoming) * lerp(f0, f90, outgoing dot incoming) + f_sheen
    // let f_diffuse = 
    //     mix(f0, f90, dot(normal, ray_outgoing)) 
    //     * mix(f0, f90, dot(ray_outgoing, ray_incoming)) 
    //     + (f_sheen * material.sheen);

    // return f_diffuse * material.albedo * (1.0 - material.metallic);
}


// fn ggx(
//     alpha: f32,
//     cos_normal_halfway: f32,
// ) -> f32 {
//     // alpha^2 / (pi * ((alpha^2 - 1) * dot(normal, halfway)^2 + 1)^2)
//     let b = (alpha - 1.0) * (cos_normal_halfway * cos_normal_halfway) + 1.0;
//     return (alpha) / (PI * b * b);
// }

fn ggx(
    data: BRDFData
) -> f32 {
    // alpha^2 / (pi * ((alpha^2 - 1) * dot(normal, halfway)^2 + 1)^2)
    let b = (data.alpha_x - 1.0) * (data.ndoth * data.ndoth) + 1.0;
    return (data.alpha_x) / (PI * b * b);
}

// fn ggx_sample_microfacet_normal(
//     alpha: f32,
//     normal: vec3<f32>,
//     v: vec3<f32>
// ) -> vec3<f32> {
//     // https://agraphicsguynotes.com/posts/sample_microfacet_brdf/
//     // theta = arctan(alpha * sqrt((epsilon)/(1 - epsilon)))
//     // phi can be randomly sampled

//     let tangent: vec3<f32> = normalize(cross(normal, v));
//     let bitangent = -normalize(cross(normal, tangent));

//     let epsilon = rand_float();

//     let angle_from_normal = (PI / 2.0) - atan(alpha * sqrt(epsilon / (1.0 - epsilon)));
//     let theta = rand_float() * 2.0 * PI;

//     let local_x = cos(theta) * cos(angle_from_normal);
//     let local_y = sin(angle_from_normal);
//     let local_z = sin(theta) * cos(angle_from_normal);

//     let local_microfacet_normal = vec3<f32>(local_x, local_y, local_z);

//     // bitangent: x-axis
//     // normal: y-axis
//     // tangent: z-axis

//     // Rotate local_rd_o to world space
//     let X = vec3<f32>(1.0, 0.0, 0.0);
//     let Y = vec3<f32>(0.0, 1.0, 0.0);
//     let Z = vec3<f32>(0.0, 0.0, 1.0);
//     let world_microfacet_normal = dot(local_microfacet_normal, X) * bitangent + dot(local_microfacet_normal, Y) * normal + dot(local_microfacet_normal, Z) * tangent;

//     return world_microfacet_normal;

//     //return reflect(v, world_microfacet_normal);
// }

fn ggx_sample_microfacet_normal(
    data: BRDFData
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

fn lambda_ggx(
    a: f32
) -> f32 {
    return (-1.0 + sqrt(1.0 + 1.0 / (a * a))) / 2.0;
}

// fn smith_ggx(
//     h: vec3<f32>,
//     s: vec3<f32>,
//     alpha: f32
// ) -> f32 {
//     return 1.0 / (1 + lambda_ggx((dot(h, s))/(alpha * sqrt(1.0 - dot(h, s) * dot(h, s)))));
// }

fn smith_ggx(
    data: BRDFData,
    hdots: f32
) -> f32 {
    return 1.0 / (1.0 + lambda_ggx((hdots)/(data.alpha_x * sqrt(1.0 - hdots * hdots))));
}

// fn ggx_sample_pdf(
//     alpha: f32,
//     n: vec3<f32>,
//     h: vec3<f32>,
//     v: vec3<f32>
// ) -> f32 {
//     return (ggx(alpha, dot(h, n)) * smith_ggx(h, v, alpha)) / (4.0 * dot(n, v));
// }

fn ggx_sample_pdf(
    data: BRDFData
) -> f32 {
    return (ggx(data) * smith_ggx(data, data.vdoth)) / (4.0 * data.ndotv);
}




// fn ggx_aniso(
//     alpha_x: f32,

// )

// fn specular_probability(
//     f_schlick: vec3<f32>,
//     material: SampledMaterial
// ) -> f32 {
//     // let f0 = mix(material.specular * vec3<f32>(0.08, 0.08, 0.08), material.albedo, 1.0 - material.metallic);
//     // let f_schlick = luminance(mix(f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - dot(ray_incoming, normal), 5.0)));

//     let diffuse_reflectance = saturate(luminance(material.albedo * (1.0 - material.metallic)));

//     let fresnel = luminance(f_schlick);

//     return fresnel;
//     //return f_schlick / (f_schlick + (diffuse_reflectance * (1.0 - f_schlick)));
// }

fn specular_probability(
    data: BRDFData
) -> f32 {
    // let f0 = mix(material.specular * vec3<f32>(0.08, 0.08, 0.08), material.albedo, 1.0 - material.metallic);
    // let f_schlick = luminance(mix(f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - dot(ray_incoming, normal), 5.0)));

    let diffuse_reflectance = saturate(luminance(data.diffuse_reflectance));

    let fresnel = luminance(data.f_schlick);

    // TODO ???
    //return fresnel;
    return clamp(fresnel / (fresnel + (diffuse_reflectance * (1.0 - fresnel))), 0.1, 0.9);
    //return f_schlick / (f_schlick + (diffuse_reflectance * (1.0 - f_schlick)));
}


// fn specular(
//     ray_incoming: vec3<f32>, 
//     ray_outgoing: vec3<f32>, 
//     f_schlick: vec3<f32>,
//     normal: vec3<f32>, 
//     h: vec3<f32>,
//     material: SampledMaterial
// ) -> vec3<f32> {
//     // let h = normalize(ray_incoming + ray_outgoing);

//     let alpha_x = material.roughness * material.roughness;
//     let alpha_y = alpha_x;


//     // Todo anisotropy
//     // var tangent = vec3<f32>(0.0, 1.0, 0.0);

//     // // rotate tangent so its perpendicular to normal
//     // let theta = (PI * 0.5) - acos(dot(tangent, normal));

//     // if (material.anisotropy > 0.0) {
//     //     let aspect = sqrt(1.0 - saturate(material.anisotropy) * 0.9);
//     //     alpha_x /= aspect;
//     //     alpha_y *= aspect;

//     //     // TODO aniso rotation
//     // }

//     let distribution = ggx(alpha_x, dot(normal, h));
    
//     // let f0 = mix(material.albedo, material.specular * vec3<f32>(0.08, 0.08, 0.08), material.metallic);
//     // let f_schlick = mix(f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - dot(ray_incoming, h), 5.0));
//     //let f_schlick = vec3<f32>(1.0, 1.0, 1.0);

//     let geometric_attenuation = smith_ggx(h, ray_outgoing, alpha_x) * smith_ggx(h, ray_incoming, alpha_x);

//     return (f_schlick * distribution * geometric_attenuation) / (4.0 * dot(normal, ray_outgoing) * dot(normal, ray_incoming));
// }

fn specular(
    data: BRDFData
) -> vec3<f32> {
    // Todo anisotropy
    // var tangent = vec3<f32>(0.0, 1.0, 0.0);

    // // rotate tangent so its perpendicular to normal
    // let theta = (PI * 0.5) - acos(dot(tangent, normal));

    // if (material.anisotropy > 0.0) {
    //     let aspect = sqrt(1.0 - saturate(material.anisotropy) * 0.9);
    //     alpha_x /= aspect;
    //     alpha_y *= aspect;

    //     // TODO aniso rotation
    // }

    let distribution = ggx(data);
    
    // let f0 = mix(material.albedo, material.specular * vec3<f32>(0.08, 0.08, 0.08), material.metallic);
    // let f_schlick = mix(f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - dot(ray_incoming, h), 5.0));
    //let f_schlick = vec3<f32>(1.0, 1.0, 1.0);

    let geometric_attenuation = smith_ggx(data, data.ldoth) * smith_ggx(data, data.vdoth);

    return (data.f_schlick * distribution * geometric_attenuation) / (4.0 * data.ndotl * data.ndotv);
}

struct BRDFResult {
    color: vec3<f32>,
    pdf: f32,
    ray_outgoing: vec3<f32>
}

fn prep_data(
    p: vec3<f32>, 
    ray_incoming: vec3<f32>, 
    normal: vec3<f32>, 
    material: SampledMaterial
) -> BRDFData {
    var data = BRDFData();

    data.p = p;
    data.ray_incoming = ray_incoming;
    data.v = -ray_incoming;
    data.normal = normal;

    data.material = material;
    data.alpha_x = material.roughness * material.roughness;
    data.alpha_y = data.alpha_x;
    
    data.diffuse_reflectance = material.albedo * (1.0 - material.metallic);

    data.ndotv = dot(data.normal, data.v);

    data.f0 = mix(material.specular * vec3<f32>(0.08, 0.08, 0.08), material.albedo, material.metallic);
    data.f_schlick = mix(data.f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - data.ndotv, 5.0));

    return data;
}

fn disney_brdf(
    p: vec3<f32>, 
    ray_incoming: vec3<f32>, 
    normal: vec3<f32>, 
    material: SampledMaterial
) -> BRDFResult {
    var data = prep_data(p, ray_incoming, normal, material);

    // let v = normalize(-ray_incoming);

    let u = rand_float();

    // let f0 = mix(material.specular * vec3<f32>(0.08, 0.08, 0.08), material.albedo, material.metallic);
    // //let f_schlick = mix(f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - dot(v, normal), 5.0));
    // let f_schlick = mix(f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - dot(v, normal), 5.0));

    // let p_specular = specular_probability(f_schlick, material);
    let p_specular = specular_probability(data);

    //let diffuse_reflectance = material.albedo * (1.0 - material.metallic); 

    var color: vec3<f32>;
    var pdf: f32;
    if u > p_specular {
        data.ray_outgoing = rand_in_cosine_weighted_hemisphere(data.normal);

        data.h = data.ray_outgoing + data.v;

        data.ndotl = dot(data.normal, data.ray_outgoing);
        data.ldoth = dot(data.ray_outgoing, data.h);
        data.ndoth = dot(data.normal, data.h);    
        data.vdoth = dot(data.v, data.h);

        //color = diffuse_reflectance * diffuse(p, v, ray_outgoing, normal, material);
        color = data.diffuse_reflectance * diffuse(data) * (1.0 - data.f_schlick);

        pdf = (1.0 - p_specular);
    } else {
        //data.h = ggx_sample_microfacet_normal(data.alpha_x, data.normal, data.v);
        data.h = ggx_sample_microfacet_normal(data);
        
        data.ray_outgoing = -reflect(data.v, data.h);

        data.ndotl = dot(data.normal, data.ray_outgoing);
        data.ldoth = dot(data.ray_outgoing, data.h);
        data.ndoth = dot(data.normal, data.h);    
        data.vdoth = dot(data.v, data.h);

        //color = specular(v, ray_outgoing, sqrt(1.0 - f_schlick), normal, h, material);
        color = specular(data);

        //color *= ggx_sample_pdf(material.roughness * material.roughness, normal, v + ray_outgoing, v);

        //pdf = ggx_sample_pdf(material.roughness * material.roughness, normal, h, v);
        pdf = ggx_sample_pdf(data);
    }
    
    //color *= dot(v, normal);
    color *= data.ndotv;

    //color = vec3<f32>(p_specular);

    let rtn: BRDFResult = BRDFResult(
        color,
        pdf,
        data.ray_outgoing  
    );

    return rtn;
}

// fn brdf(
//     p: vec3<f32>, 
//     // ray from camera
//     ray_incoming: vec3<f32>, 
//     // ray to somewhere else
//     ray_outgoing: vec3<f32>, 
//     normal: vec3<f32>, 
//     material: Material
// ) -> vec3<f32> {
//     //return specular(p, ray_incoming, ray_outgoing, normal, material);

//     let h = normalize(-ray_incoming + ray_outgoing);

//     // let specular_sample = ggx_sample(material.roughness * material.roughness, normal, ray_incoming);
//     // let specular_h = normalize(-ray_incoming + specular_sample);

//     let f0 = mix(material.specular * vec3<f32>(0.08, 0.08, 0.08), material.albedo, material.metallic);
//     let f_schlick = mix(f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - dot(-ray_incoming, normal), 5.0));

//     let diffuse_reflectance = material.albedo * (1.0 - material.metallic); 
//     var weight = diffuse_reflectance * diffuse(p, -ray_incoming, ray_outgoing, normal, material) * (1.0 - f_schlick);

//     let specular = specular(p, ray_incoming, ray_outgoing, normal, material);

//     weight += specular;
//     return weight;
// }