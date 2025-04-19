use crate::{camera, scene};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub position: [f32; 3],
    _pad0: u32,
    pub lookat: [f32; 3],
    
    pub frame: u32,

    pub pixel_space_x: [f32; 3],
    _pad1: u32,
    pub pixel_space_y: [f32; 3],
    _pad2: u32,
    pub first_pixel_pos: [f32; 3],

    pub width: u32,
    pub height: u32,

    pub fov: f32,

    pub samples_per_pixel: u32,
    pub max_bounces: u32,

    //_pad: [u32; 3]
}

impl Camera {
    pub fn new(cam: &camera::Camera) -> Self {
        Self {
            width: cam.width,
            height: cam.height,
            fov: cam.fov,
            position: [cam.position.x, cam.position.y, cam.position.z],
            lookat: [cam.lookat.x, cam.lookat.y, cam.lookat.z],
            samples_per_pixel: cam.samples_per_pixel,
            max_bounces: cam.max_bounces,
            pixel_space_x: [cam.pixel_space_x.x, cam.pixel_space_x.y, cam.pixel_space_x.z],
            pixel_space_y: [cam.pixel_space_y.x, cam.pixel_space_y.y, cam.pixel_space_y.z],
            first_pixel_pos: [cam.first_pixel_pos.x, cam.first_pixel_pos.y, cam.first_pixel_pos.z],
            frame: cam.frame,

            _pad0: 0,
            _pad1: 0,
            _pad2: 0,

            //_pad: [0, 0, 0]
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub camera: Camera,

    pub num_lights: u32,
    _pad: [u32; 3]
}

impl Uniforms {
    pub fn new(cam: &camera::Camera) -> Self {
        Self {
            camera: Camera::new(&cam),
            num_lights: 0,
            _pad: [0, 0, 0]
        }
    }
}

// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct Material {
//     albedo: [f32; 3],
//     roughness: f32,

//     specular: [f32; 3],
//     metallic: f32,

//     emissive: [f32; 3],
//     sheen: f32,

//     clearcoat_thickness: f32,
//     clearcoat_roughness: f32,
//     anisotropy: f32,
//     anisotropy_rotation: f32
// }

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    albedo_texture_idx: u32,
    roughness_texture_idx: u32,
    metallic_texture_idx: u32,
    emissive_texture_idx: u32,

    specular_texture_idx: u32,
    sheen_texture_idx: u32,
    clearcoat_thickness: f32,
    clearcoat_roughness: f32,

    anisotropy: f32,
    anisotropy_rotation: f32,
    normal_texture_idx: u32,
    //_pad: u32
}

impl Material {
    pub fn new(mat: scene::Material) -> Self {
        Self {
            albedo_texture_idx: mat.albedo_texture_idx,
            roughness_texture_idx: mat.roughness_texture_idx,
            metallic_texture_idx: mat.metallic_texture_idx,
            emissive_texture_idx: mat.emissive_texture_idx,

            specular_texture_idx: mat.specular_texture_idx,
            sheen_texture_idx: mat.sheen_texture_idx,
            clearcoat_thickness: mat.clearcoat_thickness,
            clearcoat_roughness: mat.clearcoat_roughness,
            anisotropy: mat.anisotropy,
            anisotropy_rotation: mat.anisotropy_rotation,
            normal_texture_idx: mat.normal_texture_idx,
            //_pad: 0
        }
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    _pad: u32,
    pub normal: [f32; 3],
    _pad2: u32,
    pub uv: [f32; 2],
    _pad3: [u32; 2],
}

impl Vertex {
    pub fn new(pos: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position: pos,
            _pad: 0,
            normal,
            _pad2: 0,
            uv,
            _pad3: [0, 0]
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Index {
    pub index: u32
}

impl Index {
    pub fn new(index: u32) -> Self {
        Self {
            index
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexOffset {
    pub offset: u32
}

impl VertexOffset {
    pub fn new(offset: u32) -> Self {
        Self {
            offset
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceInfo {
    pub index_offset: u32,
}

impl InstanceInfo {
    pub fn new(index_offset: u32) -> Self {
        Self {
            index_offset
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    pub v0: [f32; 3],
    _pad0: u32,
    pub v1: [f32; 3],
    _pad1: u32,
    pub v2: [f32; 3],
    _pad2: u32,
    pub normal: [f32; 3],
    _pad3: u32
}

impl Triangle {
    pub fn new(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], normal: [f32; 3]) -> Self {
        Self {
            v0,
            _pad0: 0,
            v1,
            _pad1: 0,
            v2,
            _pad2: 0,
            normal,
            _pad3: 0
        }
    }
}