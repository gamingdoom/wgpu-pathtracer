use core::num;
use std::{any, num::NonZeroU32, sync::{Arc, Mutex}};

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::{camera::Camera, scene, texture, uniforms, wgpu_buffer::{BufferType, StorageBuffer, UniformBuffer}, wgpu_util};

// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct Material {
//     pub albedo: [f32; 3],
//     pub roughness: f32,

//     pub specular: [f32; 3],
//     pub metallic: f32,

//     pub emissive: [f32; 3],
//     pub sheen: f32,
 
//     pub clearcoat_thickness: f32,
//     pub clearcoat_roughness: f32,
//     pub anisotropy: f32,
//     pub anisotropy_rotation: f32
// }

#[derive(Debug, Clone)]
pub struct Material {
    pub albedo_texture_idx: u32,
    pub roughness_texture_idx: u32,
    pub metallic_texture_idx: u32,
    pub emissive_texture_idx: u32,
 
    pub specular_texture_idx: u32,
    pub sheen_texture_idx: u32,
    pub clearcoat_thickness: f32,
    pub clearcoat_roughness: f32,
 
    pub anisotropy: f32,
    pub anisotropy_rotation: f32,
    pub normal_texture_idx: u32,

    pub transmission_texture_idx: u32,
    pub ior_texture_idx: u32,
}



// struct Triangle {
//     v: [[f32; 3]; 3], 
//     material: Material
// }

pub struct Mesh {
    // tris: Vec<Triangle>,
    // pub vertices: Vec<[f32; 3]>,
    // pub indices: Vec<u32>,
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub material_index: u32
}

pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub vertices: Vec<uniforms::Vertex>,
    pub indices: Vec<uniforms::Index>,
    pub materials: Vec<Material>,
    pub blases: Vec<wgpu::Blas>,
    pub camera: Camera,
    pub prev_camera: Camera,
    pub textures: Vec<Arc<Mutex<texture::Texture>>>,
}

impl Scene {
    pub fn new(state: &wgpu_util::WGPUState, camera: Camera) -> Self {
        let mut materials = Vec::new();

        let mut textures = Vec::new();
        textures.push(Arc::new(Mutex::new(texture::Texture::from_color(&state.rt_device, &state.rt_queue, image::Rgba([1.0, 1.0, 1.0, 1.0]), Some("albedo default")).unwrap())));
        textures.push(Arc::new(Mutex::new(texture::Texture::from_scalar(&state.rt_device, &state.rt_queue, image::Luma([0.99]), Some("roughness default")).unwrap())));
        textures.push(Arc::new(Mutex::new(texture::Texture::from_color(&state.rt_device, &state.rt_queue, image::Rgba([0.5, 0.5, 0.5, 1.0]), Some("specular default")).unwrap())));
        textures.push(Arc::new(Mutex::new(texture::Texture::from_scalar(&state.rt_device, &state.rt_queue, image::Luma([0.0]), Some("metallic default")).unwrap())));
        // Emissive's index (4)Mutex::new( is hardcoded and used elsewhere!
        textures.push(Arc::new(Mutex::new(texture::Texture::from_color(&state.rt_device, &state.rt_queue, image::Rgba([0.0, 0.0, 0.0, 1.0]), Some("emissive default")).unwrap())));
        textures.push(Arc::new(Mutex::new(texture::Texture::from_scalar(&state.rt_device, &state.rt_queue, image::Luma([0.0]), Some("sheen default")).unwrap())));
        textures.push(Arc::new(Mutex::new(texture::Texture::from_color(&state.rt_device, &state.rt_queue, image::Rgba([0.5, 0.5, 1.0, 1.0]), Some("normal default")).unwrap())));

        textures.push(Arc::new(Mutex::new(texture::Texture::from_scalar(&state.rt_device, &state.rt_queue, image::Luma([0.0]), Some("transmission weight default")).unwrap())));
        textures.push(Arc::new(Mutex::new(texture::Texture::from_scalar(&state.rt_device, &state.rt_queue, image::Luma([1.0]), Some("IOR default")).unwrap())));

        textures.push(Arc::new(Mutex::new(texture::Texture::from_file_rgba(&state.rt_device, &state.rt_queue, "res/knob/envmap.exr", Some("envmap")).unwrap())));

        // Default material
        materials.push(Material { 
            albedo_texture_idx: 0,
            roughness_texture_idx: 1,
            specular_texture_idx: 2,
            metallic_texture_idx: 3,
            emissive_texture_idx: 4,
            sheen_texture_idx: 5,
            normal_texture_idx: 6,
            clearcoat_thickness: 0.0,
            clearcoat_roughness: 0.0,
            anisotropy: 0.0,
            anisotropy_rotation: 0.0,
            transmission_texture_idx: 7,
            ior_texture_idx: 8,
        });

        Self {
            meshes: Vec::new(),
            materials,
            blases: Vec::new(),
            camera,
            prev_camera: camera.clone(),
            vertices: Vec::new(),
            indices: Vec::new(),
            textures: textures,
        }
    }

    pub fn load_obj(&mut self, state: &wgpu_util::WGPUState, path: &str) {
        let model = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS);
        let (models, materials) = model.expect(format!("Failed to load OBJ {}", path).as_str());

        let materials = materials.expect(format!("Failed to load MTL for {}", path).as_str());

        //let mut tris = Vec::new();

        let prev_num_materials = self.materials.len();

        for material in materials.iter() {     
            let albedo_texture_idx = self.get_material_texture_known_vector(state, material.diffuse_texture.clone(), material.diffuse, path).unwrap_or(0);
            let roughness_texture_idx = self.get_material_texture_prop(state, material, "map_Pr", "Pr", true, path).unwrap_or(1);
             
            let specular_texture_idx = self.get_material_texture_known_vector(state, material.specular_texture.clone(), material.specular, path).unwrap_or(2);
            let metallic_texture_idx = self.get_material_texture_prop(state, material, "map_Pm", "Pm", true, path).unwrap_or(3);
             
            let emissive_texture_idx = self.get_material_texture_prop(state, material, "map_Ke", "Ke", false, path).unwrap_or(4);
            let sheen_texture_idx = self.get_material_texture_prop(state, material, "map_Ps", "Ps", true, path).unwrap_or(5);
 
            let normal_texture_idx = self.get_material_texture_known_vector(state, material.normal_texture.clone(), None, path).unwrap_or(6);

            let transmission_texture_idx = self.get_material_texture_prop(state, material, "map_Tf", "Tf", false, path).unwrap_or(7);
            let ior_texture_idx = self.get_material_texture_known_scalar(state, None, material.optical_density, path).unwrap_or(8);
            

            self.materials.push(Material { 
                albedo_texture_idx,
                roughness_texture_idx,
                specular_texture_idx,
                metallic_texture_idx,
                emissive_texture_idx,
                sheen_texture_idx,
                normal_texture_idx,

                clearcoat_thickness: Scene::get_material_scalar_prop(material, "Pc").unwrap_or(0.0),
                clearcoat_roughness: Scene::get_material_scalar_prop(material, "Pcr").unwrap_or(0.0),
                anisotropy: Scene::get_material_scalar_prop(material, "Pn").unwrap_or(0.0),
                anisotropy_rotation: Scene::get_material_scalar_prop(material, "Pnr").unwrap_or(0.0),

                transmission_texture_idx,
                ior_texture_idx,
             });
        }

        for (i, m) in models.iter().enumerate() {
            let mesh = &m.mesh;
            
            let prev_num_vertices = self.vertices.len();

            for (i, pos) in mesh.positions.chunks(3).enumerate() {
                let mut normal = [0.0, 0.0, 0.0];
                if mesh.normals.len() != 0 {
                    normal = [mesh.normals[i * 3 + 0], mesh.normals[i * 3 + 1], mesh.normals[i * 3 + 2]];
                }

                let mut uv = [0.0, 0.0];
                if mesh.texcoords.len() != 0 {
                    uv = [mesh.texcoords[i * 2], 1.0 - mesh.texcoords[i * 2 + 1]];
                }

                self.vertices.push(
                    uniforms::Vertex::new([pos[0], pos[1], pos[2]], 
                        normal,
                        uv
                    )
                );
            }

            let prev_num_indices = self.indices.len();

            for (i, index) in mesh.indices.iter().enumerate() {
                self.indices.push(
                    uniforms::Index::new(*index as u32 + prev_num_vertices as u32)
                );
            }

            // Compute normals if not present
            if mesh.normals.len() == 0 {
                for indices in mesh.indices.chunks(3) {
                    // normalize(cross(v1 - v0, v2 - v0))
                    let v0 = Vec3::from_slice(&self.vertices[indices[0] as usize].position);
                    let v1 = Vec3::from_slice(&self.vertices[indices[1] as usize].position);
                    let v2 = Vec3::from_slice(&self.vertices[indices[2] as usize].position);

                    let normal = (v1 - v0).cross(v2 - v0).normalize().into();

                    self.vertices[indices[0] as usize].normal = normal;
                    self.vertices[indices[1] as usize].normal = normal;
                    self.vertices[indices[2] as usize].normal = normal;
                }
            }

            let mut scene_mesh = Mesh {
                //vertices: Vec::new(),
                //indices: Vec::new(),
                vertex_offset: prev_num_vertices as u32,
                vertex_count: (mesh.positions.len() / 3) as u32,
                index_offset: prev_num_indices as u32,
                index_count: mesh.indices.len() as u32,
                material_index: (mesh.material_id.unwrap_or(0) + prev_num_materials) as u32,
            };
    
            //scene_mesh.indices = mesh.indices.clone();
            
            // for pos in mesh.positions.chunks(3) {
            //     scene_mesh.vertices.push([pos[0], pos[1], pos[2]]);
            // }

            self.meshes.push(scene_mesh);
        }

        // Multithreaded load of textures
        println!("Loading textures...");

        for tex_chunk in self.textures.chunks(32) {
            let mut handles = Vec::new();
            for tex in tex_chunk {
                let dev = state.rt_device.clone();
                let queue = state.rt_queue.clone();
                let texture = Arc::clone(&tex);
                
                handles.push(std::thread::spawn(move || {
                    texture.lock().unwrap().load(&dev, &queue, None).unwrap();
                }))
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }


    }


    fn get_material_scalar_prop(material: &tobj::Material, prop: &str) -> Option<f32> {
        material.unknown_param.get(prop)?.parse::<f32>().ok()
    }

    fn get_material_vec_prop(material: &tobj::Material, prop: &str) -> Option<[f32; 3]> {
        material.unknown_param.get(prop)?
            .split(" ")
            .map(
                |s| s.parse::<f32>().unwrap_or(0.0)
            )
            .collect::<Vec<f32>>()
            .try_into()
            .ok()
    }

    fn get_material_texture_prop(&mut self, state: &wgpu_util::WGPUState, material: &tobj::Material, tex_prop: &str, prop: &str, scalar: bool, obj_path: &str) -> Option<u32> {
        // Check if tex_prop exists
        let tex_prop_res = material.unknown_param.get(tex_prop);
        let mut tex: Option<texture::Texture> = None;
        if tex_prop_res.is_some() {
            let tex_path_path = std::path::Path::new(obj_path).parent().unwrap().join(tex_prop_res.unwrap());
            let tex_path = tex_path_path.to_str().unwrap();

            if scalar {
                tex = Some(texture::Texture::from_file_scalar(&state.rt_device, &state.rt_queue, tex_path, Some(tex_path)).unwrap());
            } else {
                tex = Some(texture::Texture::from_file_rgba(&state.rt_device, &state.rt_queue, tex_path, Some(tex_path)).unwrap());
            }
        } else {
            // Check if prop exists
            let prop_res = material.unknown_param.get(prop);
            if prop_res.is_some() {
                let prop_val = prop_res.unwrap();
                if scalar {
                    let val = Scene::get_material_scalar_prop(material, prop)?;

                    // Check if texture with this value already exists
                    for i in 0..self.textures.len() {
                        if self.textures[i].lock().unwrap().scalar.is_some() && self.textures[i].lock().unwrap().scalar.unwrap() == val {
                            return Some(i as u32);
                        }
                    }

                    tex = Some(texture::Texture::from_scalar(&state.rt_device, &state.rt_queue, image::Luma([val]), Some(prop_val)).unwrap());
                } else {
                    let mut val = Scene::get_material_vec_prop(material, prop).unwrap();

                    if prop == "Ke" && val[0] == 0.0 && val[1] == 0.0 && val[2] == 0.0 {
                        return None
                    }

                    // Check if texture with this value already exists
                    for i in 0..self.textures.len() {
                        if self.textures[i].lock().unwrap().color.is_some() && self.textures[i].lock().unwrap().color.unwrap() == [val[0], val[1], val[2], 1.0] {
                            return Some(i as u32);
                        }
                    }

                    tex = Some(texture::Texture::from_color(&state.rt_device, &state.rt_queue, image::Rgba([val[0], val[1], val[2], 1.0]), Some(prop_val)).unwrap());
                }
            }
        }

        if tex.is_none() {
            return None
        }
        
        self.textures.push(Arc::new(Mutex::new(tex.unwrap())));
        Some((self.textures.len() - 1) as u32)
    }

    fn get_material_texture_known_scalar(&mut self, state: &wgpu_util::WGPUState, tex_prop: Option<String>, prop: Option<f32>, obj_path: &str) -> Option<u32> {
        // Check if tex_prop exists
        let mut tex: Option<texture::Texture> = None;
        if tex_prop.is_some() {
            let tex_path_path = std::path::Path::new(obj_path).parent().unwrap().join(tex_prop.unwrap());
            let tex_path = tex_path_path.to_str().unwrap();
            tex = Some(texture::Texture::from_file_scalar(&state.rt_device, &state.rt_queue, &tex_path, Some(&tex_path)).unwrap());

        } else {
            // Check if prop exists
            if prop.is_some() {
                let prop_val = prop.unwrap();

                // Check if texture with this value already exists
                for i in 0..self.textures.len() {
                    if self.textures[i].lock().unwrap().scalar.is_some() && self.textures[i].lock().unwrap().scalar.unwrap() == prop_val {
                        return Some(i as u32);
                    }
                }

                tex = Some(texture::Texture::from_scalar(&state.rt_device, &state.rt_queue, image::Luma([prop_val]), None).unwrap());

            }
        }

        if tex.is_none() {
            return None
        }
        
        self.textures.push(Arc::new(Mutex::new(tex.unwrap())));
        Some((self.textures.len() - 1) as u32)
    }

    fn get_material_texture_known_vector(&mut self, state: &wgpu_util::WGPUState, tex_prop: Option<String>, prop: Option<[f32; 3]>, obj_path: &str) -> Option<u32> {
        // Check if tex_prop exists
        let mut tex: Option<texture::Texture> = None;
        if tex_prop.is_some() {
            let tex_path_path = std::path::Path::new(obj_path).parent().unwrap().join(tex_prop.unwrap());
            let tex_path = tex_path_path.to_str().unwrap();
            tex = Some(texture::Texture::from_file_rgba(&state.rt_device, &state.rt_queue, &tex_path, Some(&tex_path)).unwrap());

        } else {
            // Check if prop exists
            if prop.is_some() {
                let prop_val = prop.unwrap();

                // Check if texture with this value already exists
                for i in 0..self.textures.len() {
                    if self.textures[i].lock().unwrap().color.is_some() && self.textures[i].lock().unwrap().color.unwrap() == [prop_val[0], prop_val[1], prop_val[2], 1.0] {
                        return Some(i as u32);
                    }
                }

                tex = Some(texture::Texture::from_color(&state.rt_device, &state.rt_queue, image::Rgba([prop_val[0], prop_val[1], prop_val[2], 1.0]), None).unwrap());
            }
        }

        if tex.is_none() {
            return None;
        }
        
        self.textures.push(Arc::new(Mutex::new(tex.unwrap())));
        Some((self.textures.len() - 1) as u32)
    }
}
