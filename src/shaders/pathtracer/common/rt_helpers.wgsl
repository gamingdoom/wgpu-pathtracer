struct ScatteredRay {
    origin: vec3<f32>,
    direction: vec3<f32>,
    attenuation: vec3<f32>,
    normal: vec3<f32>,
    hit_prediction: bool,
};

struct RayIntersectionCustom {
    ri: RayIntersection,
    vertices: array<Vertex, 3>,
    material: SampledMaterial,
    hit_light: bool,
    uvw: vec3<f32>,
}

fn sample_material(material: Material, uv: vec2<f32>) -> SampledMaterial {
    let rgba = sample_texture_rgba(material.albedo_texture_idx, uv);

    return SampledMaterial (
        rgba.rgb,
        rgba.a,

        sample_texture_color(material.normal_texture_idx, uv) * 2.0 - 1.0,

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
    rayQueryInitialize(&rq, acc_struct, RayDesc(0u, 0xFFu, 0.01, 10000.0, ro, rd));
    rayQueryProceed(&rq);

    var rq_intersection: RayIntersection = rayQueryGetCommittedIntersection(&rq);

    var verts: array<Vertex, 3>;

    if (rq_intersection.kind != RAY_QUERY_INTERSECTION_NONE) {
       verts[0] = vertices[indices[instance_infos[rq_intersection.instance_index].index_offset + rq_intersection.geometry_index * 3 + 0]];
       verts[1] = vertices[indices[instance_infos[rq_intersection.instance_index].index_offset + rq_intersection.geometry_index * 3 + 1]];
       verts[2] = vertices[indices[instance_infos[rq_intersection.instance_index].index_offset + rq_intersection.geometry_index * 3 + 2]];
    }

    let w = 1.0 - rq_intersection.barycentrics.x - rq_intersection.barycentrics.y;

    let bary = vec3<f32>(w, rq_intersection.barycentrics.x, rq_intersection.barycentrics.y);
    let uv = verts[0].uv * bary.x + verts[1].uv * bary.y + verts[2].uv * bary.z;

    let material = sample_material(materials[rq_intersection.instance_custom_data], uv);

    let emissive = material.emissive;
    var hit_light = dot(emissive, vec3<f32>(1.0)) > 0.0;

    var intersection: RayIntersectionCustom = RayIntersectionCustom(
        rq_intersection,
        verts,
        material,
        hit_light,
        vec3<f32>(w, rq_intersection.barycentrics.x, rq_intersection.barycentrics.y),
    );

    return intersection;
}