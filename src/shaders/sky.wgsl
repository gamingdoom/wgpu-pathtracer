fn sky_color(rd: vec3<f32>) -> vec3<f32> {
    var unit_direction = normalize(rd);
    var t = 0.5 * (unit_direction.y + 1.0);

    return vec3<f32>(1.0, 1.0, 1.0) * (1.0 - t) + vec3<f32>(0.5, 0.7, 1.0) * t;

    //return vec3<f32>(0.0517339484814);
}