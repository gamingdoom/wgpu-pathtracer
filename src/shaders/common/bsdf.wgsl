// https://sayan1an.github.io/pdfs/references/disneyBrdf.pdf

struct BSDFData {
    p: vec3<f32>, 
    v: vec3<f32>,
    ray_outgoing: vec3<f32>, 
    normal: vec3<f32>, 
    raw_normal: vec3<f32>,
    h: vec3<f32>,
    refracted: bool,

    material: SampledMaterial,
    alpha_x: f32,
    alpha_y: f32,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,

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

struct BSDFResult {
    color: vec3<f32>,
    pdf: f32,
    ray_outgoing: vec3<f32>,
    p: vec3<f32>,
}

fn bsdf_prep_data(
    p: vec3<f32>, 
    ray_incoming: vec3<f32>, 
    normal: vec3<f32>, 
    material: SampledMaterial
) -> BSDFData {
    var data = BSDFData();

    data.p = p;
    data.v = -ray_incoming;
    data.raw_normal = normal;
    data.normal = faceForward(data.raw_normal, ray_incoming, data.raw_normal);

    // var t = cross(data.raw_normal, vec3<f32>(0.0, 1.0, 0.0));
    // t += cross(data.raw_normal, vec3<f32>(0.0, 0.0, 1.0)) * step(dot(t, t), 0.0);

    // data.tangent = normalize(t);
    // data.bitangent = normalize(cross(data.raw_normal, data.tangent));
    data.tangent = normalize(cross(data.normal, data.v));
    data.bitangent = normalize(cross(data.normal, data.tangent));

    // Normal Mapping
    data.normal = normalize(
        data.tangent * material.tangent_space_normal.x 
        + data.bitangent * material.tangent_space_normal.y 
        + data.normal * material.tangent_space_normal.z
    );

    data.material = material;

    let aspect = sqrt(1.0 - 0.9 * data.material.anisotropy);

    // data.alpha_x = max(0.0001, material.roughness * material.roughness / aspect);
    data.alpha_x = max(0.0001, material.roughness / aspect);

    //data.alpha_x = material.roughness;
    data.alpha_y = data.alpha_x;
    
    data.diffuse_color = material.albedo * (1.0 - material.metallic);
    data.specular_color = mix(vec3<f32>(1.0, 1.0, 1.0), material.albedo, material.metallic);

    data.ndotv = dot(data.normal, data.v);

    data.f0 = mix(vec3<f32>(F90), material.albedo, material.metallic);
    data.f_schlick = mix(data.f0, vec3<f32>(1.0, 1.0, 1.0), pow(1.0 - data.ndotv, 5.0));


    return data;
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

fn get_real_fresnel(
    eta: f32,
    h: vec3<f32>,
    o_in: vec3<f32>,
    o_out: vec3<f32>
) -> f32 {
    let rs = (dot(h, o_in) - eta * dot(h, o_out)) / (dot(h, o_in) + eta * dot(h, o_out));
    let rp = (eta * dot(h, o_in) - dot(h, o_out)) / (eta * dot(h, o_in) + dot(h, o_out));

    return 0.5 * (rs * rs + rp * rp);
}

struct OutgoingRaySample {
    ray_outgoing: vec3<f32>,
    refracted: bool,
}

fn bsdf_ray_outgoing_from_h(
    h: vec3<f32>,
    data: BSDFData
) -> OutgoingRaySample {    
    let d = dot(data.v, data.raw_normal);

    let eta = data.material.ior * f32(d <= 0.0) + (1.0 / data.material.ior) * f32(d > 0.0);
    let r = refract(-data.v, h, eta);
    
    let r0 = (1.0 - eta) / (1.0 + eta);
    let r0_sq = r0 * r0;
    let reflectance_fresnel = r0_sq + (1.0 - r0_sq) * (get_schlick_weight(dot(h, data.v)));

    let u = rand_float();

    let transmission_factor = data.material.transmission_weight * (1.0 - reflectance_fresnel);

    let refracted = step(u, transmission_factor) * abs(sign(dot(r, vec3<f32>(1.0))));

    return OutgoingRaySample(
        r * refracted
        + 
        reflect(-data.v, h) * (1.0 - refracted),

        bool(refracted)
    );

}

struct BSDFSample {
    h: vec3<f32>,
    outgoing: OutgoingRaySample,
    pdf: f32,
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

    let local_x = sin_up_down * cos_left_right;
    let local_y = cos_up_down;
    let local_z = sin_up_down * sin_left_right;

    let world_microfacet_normal = normalize(local_x * data.bitangent + local_y * data.normal + local_z * data.tangent);

    let outgoing = bsdf_ray_outgoing_from_h(world_microfacet_normal, data);

    // let t = dot(world_microfacet_normal, data.normal);
    // let denom = (a_sq - 1.0) * t * t + 1.0;
    //let pdf = ((a_sq * t) / (PI * denom * denom));
    let pdf = 1.0;

    return BSDFSample(
        world_microfacet_normal,
        outgoing,
        pdf
    );
}

fn bsdf_specular(
    data: BSDFData
) -> vec3<f32> {
    return data.specular_color * get_schlick_fresnel(data.ndoth) * f32(!data.refracted);   
}

fn disney_bsdf(
    p: vec3<f32>, 
    ray_incoming: vec3<f32>, 
    normal: vec3<f32>, 
    material: SampledMaterial
) -> BSDFResult {
    var data = bsdf_prep_data(p, ray_incoming, normal, material);

    let u = rand_float();

    var color: vec3<f32>;
    var pdf: f32;

    let sample = bsdf_sample(data);
    data.h = sample.h;
    data.ray_outgoing = sample.outgoing.ray_outgoing;
    data.refracted = sample.outgoing.refracted;
    pdf = sample.pdf;

    data.ndotl = dot(data.normal, data.ray_outgoing);
    data.ldoth = dot(data.ray_outgoing, data.h);
    data.ndoth = dot(data.normal, data.h);    
    data.vdoth = dot(data.v, data.h);

    color = data.diffuse_color;

    pdf = mix(pdf, 1.0, f32(data.refracted));

    let rtn: BSDFResult = BSDFResult(
        color,
        pdf,
        data.ray_outgoing,
        p + (data.normal * mix(0.01, 0.0, f32(data.refracted)))
    );

    return rtn;
}