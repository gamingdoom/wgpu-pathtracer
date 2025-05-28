
struct PathtraceResult {
    color: f32,
    hit_light: bool,    
}

fn pathtrace(ro: vec3<f32>, rd: vec3<f32>, acc_struct: acceleration_structure) -> PathtraceResult {

}


fn pixel_color(xy: vec2<u32>, acc_struct: acceleration_structure) -> vec4<f32> {
    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    
    for (var i = 0u; i < BDPT_SAMPLES_PER_PIXEL; i++) {
        // Get ray origin and direction
        let cam_ro = uniforms.camera.position;
        let cam_rd = normalize(
            (
                uniforms.camera.first_pixel_pos 
                + uniforms.camera.pixel_space_x * (f32(xy.x) + (0.5 * rand_float()))
                + uniforms.camera.pixel_space_y * (f32(xy.y) + (0.5 * rand_float()))
            ) 
            - ro
        );

        let res = pathtrace(cam_ro, cam_rd, acc_struct);

        color += res.color;

        if !res.hit_light {
            // Trace Light Ray
            let light = lights[rand_int() % uniforms.num_lights];
            let ro = random_point_on_light(light);
            // todo create light ray path
            let rd = random_in_h
        }
    }

    color /= f32(SAMPLES_PER_PIXEL);

    textureStore(depth_output, xy, vec4<f32>(color.a, 0.0, 0.0, 1.0));

    color.w = 1.0;
    
    return color;
}