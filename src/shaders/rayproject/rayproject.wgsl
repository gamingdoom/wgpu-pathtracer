//!#define pub
//!#include "../shader_definitions.rs"

struct Camera {
    position: vec3<f32>,
    lookat: vec3<f32>,

    frame: u32,

    pixel_space_x: vec3<f32>,
    pixel_space_y: vec3<f32>,
    first_pixel_pos: vec3<f32>,

    width: u32,
    height: u32,

    fov: f32,
  
    samples_per_pixel: u32,
    max_bounces: u32,
}

struct RayprojectUniforms {
    camera: Camera,
    prev_camera: Camera
}

@group(0)
@binding(0)
var out_tex: texture_storage_2d<rgba32float, write>;

@group(0)
@binding(1)
var last_real_frame_tex: texture_storage_2d<rgba32float, read>;

@group(0)
@binding(2)
var depth: texture_storage_2d<r32float, read>;

@group(0)
@binding(3)
var<uniform> uniforms: RayprojectUniforms;


@compute @workgroup_size(WORKGROUP_DIM, WORKGROUP_DIM)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let target_size = textureDimensions(out_tex);

    let x = global_id.x;
    let y = global_id.y;

    var last_frame_color = textureLoad(last_real_frame_tex, vec2<u32>(x, y));
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    let prev_ray_origin_at_xy = uniforms.prev_camera.first_pixel_pos + f32(x) * uniforms.prev_camera.pixel_space_x + f32(y) * uniforms.prev_camera.pixel_space_y;
    let prev_ray_dir_at_xy = normalize(prev_ray_origin_at_xy - uniforms.prev_camera.position);

    let first_t = textureLoad(depth, vec2<u32>(x, y)).x;

    let new_center_to_old_pix = (prev_ray_origin_at_xy + prev_ray_dir_at_xy * first_t) - uniforms.camera.position;

    let reprojected_pix_pos = new_center_to_old_pix + uniforms.camera.position;

    let reprojected_pix_pos_on_vp_plane = new_center_to_old_pix / dot(new_center_to_old_pix, uniforms.camera.lookat - uniforms.camera.position);

    let reprojected_pix_x = (dot(normalize(uniforms.camera.pixel_space_x), reprojected_pix_pos_on_vp_plane) / length(uniforms.camera.pixel_space_x)) 
                            + f32(uniforms.camera.width) * 0.5
                            + 0.5;
    
    let reprojected_pix_y = (dot(normalize(uniforms.camera.pixel_space_y), reprojected_pix_pos_on_vp_plane) / length(uniforms.camera.pixel_space_y)) 
                            + f32(uniforms.camera.height) * 0.5
                            + 0.5;

    textureStore(out_tex, vec2<u32>(u32(round(reprojected_pix_x)), u32(round(reprojected_pix_y))), last_frame_color);
}