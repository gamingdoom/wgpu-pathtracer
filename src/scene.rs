use std::{any, num::NonZeroU32};

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
    pub textures: Vec<texture::Texture>,
}

impl Scene {
    pub fn new(state: &wgpu_util::WGPUState, camera: Camera) -> Self {
        let mut materials = Vec::new();

        let mut textures = Vec::new();
        textures.push(texture::Texture::from_color(&state.device, &state.queue, image::Rgba([1.0, 1.0, 1.0, 1.0]), Some("albedo default")).unwrap());
        textures.push(texture::Texture::from_scalar(&state.device, &state.queue, image::Luma([1.0]), Some("roughness default")).unwrap());
        textures.push(texture::Texture::from_color(&state.device, &state.queue, image::Rgba([0.5, 0.5, 0.5, 1.0]), Some("specular default")).unwrap());
        textures.push(texture::Texture::from_scalar(&state.device, &state.queue, image::Luma([0.0]), Some("metallic default")).unwrap());
        // Emissive's index (4) is hardcoded and used elsewhere!
        textures.push(texture::Texture::from_color(&state.device, &state.queue, image::Rgba([0.0, 0.0, 0.0, 1.0]), Some("emissive default")).unwrap());
        textures.push(texture::Texture::from_scalar(&state.device, &state.queue, image::Luma([0.0]), Some("sheen default")).unwrap());
        textures.push(texture::Texture::from_color(&state.device, &state.queue, image::Rgba([0.0, 0.0, 0.0, 1.0]), Some("normal default")).unwrap());

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
            anisotropy_rotation: 0.0
        });

        Self {
            meshes: Vec::new(),
            materials,
            blases: Vec::new(),
            camera,
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
                anisotropy_rotation: Scene::get_material_scalar_prop(material, "Pnr").unwrap_or(0.0)
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
                    uv = [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]];
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

            // // Compute normals if not present
            // if mesh.normals.len() == 0 {
            //     for indices in mesh.indices.chunks(3) {
            //         // normalize(cross(v1 - v0, v2 - v0))
            //         let v0 = Vec3::from_slice(&self.vertices[indices[0] as usize].position);
            //         let v1 = Vec3::from_slice(&self.vertices[indices[1] as usize].position);
            //         let v2 = Vec3::from_slice(&self.vertices[indices[2] as usize].position);

            //         let normal = (v1 - v0).cross(v2 - v0).normalize().into();

            //         self.vertices[indices[0] as usize].normal = normal;
            //         self.vertices[indices[1] as usize].normal = normal;
            //         self.vertices[indices[2] as usize].normal = normal;
            //     }
            // }

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
                tex = Some(texture::Texture::from_file_scalar(&state.device, &state.queue, tex_path, Some(tex_path)).unwrap());
            } else {
                tex = Some(texture::Texture::from_file_rgba(&state.device, &state.queue, tex_path, Some(tex_path)).unwrap());
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
                        if self.textures[i].scalar.is_some() && self.textures[i].scalar.unwrap() == val {
                            return Some(i as u32);
                        }
                    }

                    tex = Some(texture::Texture::from_scalar(&state.device, &state.queue, image::Luma([val]), Some(prop_val)).unwrap());
                } else {
                    let val = Scene::get_material_vec_prop(material, prop).unwrap();

                    if prop == "Ke" && val[0] == 0.0 && val[1] == 0.0 && val[2] == 0.0 {
                        return None
                    }

                    // Check if texture with this value already exists
                    for i in 0..self.textures.len() {
                        if self.textures[i].color.is_some() && self.textures[i].color.unwrap() == [val[0], val[1], val[2], 1.0] {
                            return Some(i as u32);
                        }
                    }

                    tex = Some(texture::Texture::from_color(&state.device, &state.queue, image::Rgba([val[0], val[1], val[2], 1.0]), Some(prop_val)).unwrap());
                }
            }
        }

        if tex.is_none() {
            return None
        }
        
        self.textures.push(tex?);
        Some((self.textures.len() - 1) as u32)
    }

    fn get_material_texture_known_scalar(&mut self, state: &wgpu_util::WGPUState, tex_prop: Option<String>, prop: Option<f32>, obj_path: &str) -> Option<u32> {
        // Check if tex_prop exists
        let mut tex: Option<texture::Texture> = None;
        if tex_prop.is_some() {
            let tex_path_path = std::path::Path::new(obj_path).parent().unwrap().join(tex_prop.unwrap());
            let tex_path = tex_path_path.to_str().unwrap();
            tex = Some(texture::Texture::from_file_scalar(&state.device, &state.queue, &tex_path, Some(&tex_path)).unwrap());

        } else {
            // Check if prop exists
            if prop.is_some() {
                let prop_val = prop.unwrap();

                // Check if texture with this value already exists
                for i in 0..self.textures.len() {
                    if self.textures[i].scalar.is_some() && self.textures[i].scalar.unwrap() == prop_val {
                        return Some(i as u32);
                    }
                }

                tex = Some(texture::Texture::from_scalar(&state.device, &state.queue, image::Luma([prop_val]), None).unwrap());

            }
        }

        if tex.is_none() {
            return None
        }
        
        self.textures.push(tex?);
        Some((self.textures.len() - 1) as u32)
    }

    fn get_material_texture_known_vector(&mut self, state: &wgpu_util::WGPUState, tex_prop: Option<String>, prop: Option<[f32; 3]>, obj_path: &str) -> Option<u32> {
        // Check if tex_prop exists
        let mut tex: Option<texture::Texture> = None;
        if tex_prop.is_some() {
            let tex_path_path = std::path::Path::new(obj_path).parent().unwrap().join(tex_prop.unwrap());
            let tex_path = tex_path_path.to_str().unwrap();
            tex = Some(texture::Texture::from_file_rgba(&state.device, &state.queue, &tex_path, Some(&tex_path)).unwrap());

        } else {
            // Check if prop exists
            if prop.is_some() {
                let prop_val = prop.unwrap();

                // Check if texture with this value already exists
                for i in 0..self.textures.len() {
                    if self.textures[i].color.is_some() && self.textures[i].color.unwrap() == [prop_val[0], prop_val[1], prop_val[2], 1.0] {
                        return Some(i as u32);
                    }
                }

                tex = Some(texture::Texture::from_color(&state.device, &state.queue, image::Rgba([prop_val[0], prop_val[1], prop_val[2], 1.0]), None).unwrap());
            }
        }

        if tex.is_none() {
            return None;
        }
        
        self.textures.push(tex?);
        Some((self.textures.len() - 1) as u32)
    }

    fn get_rt_bgl(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl for shader.wgsl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::AccelerationStructure {
                        vertex_return: true
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // wgpu::BindGroupLayoutEntry {
                //     binding: 8,
                //     visibility: wgpu::ShaderStages::COMPUTE,
                //     ty: wgpu::BindingType::Texture {
                //         sample_type: wgpu::TextureSampleType::Float { filterable: true },
                //         view_dimension: wgpu::TextureViewDimension::D2,
                //         multisampled: false,
                //     },
                //     count: NonZeroU32::new(self.textures.len() as u32),
                // },
                // wgpu::BindGroupLayoutEntry {
                //     binding: 9,
                //     visibility: wgpu::ShaderStages::COMPUTE,
                //     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    
                //     // Should be same as number of texture groups (diffuse + normal + ...)
                //     count: NonZeroU32::new(self.textures.len() as u32),
                // },
            ],
        });

        bgl
    }

    fn get_texture_bgl(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl for shader.wgsl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(self.textures.len() as u32),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    
                    // Should be same as number of texture groups (diffuse + normal + ...)
                    count: NonZeroU32::new(self.textures.len() as u32),
                },
            ],
        });

        bgl
    }

    pub fn create_resources(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::ComputePipeline, Vec<wgpu::Blas>) {
        let rt_bgl = self.get_rt_bgl(device);
        let texture_bgl = self.get_texture_bgl(device);

        let mut blases = Vec::new();
        //let mut blas_sizes = Vec::new();
        //let mut blas_build_entries = Vec::new();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage:  wgpu::BufferUsages::BLAS_INPUT | 
                    wgpu::BufferUsages::STORAGE | 
                    wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::BLAS_INPUT,
        });

        let mut mesh_size_descs = Vec::new();

        for (i, mesh) in self.meshes.iter().enumerate() {
            let mut tri_size_descs = Vec::new();
            for _ in (0..mesh.index_count).step_by(3) {
                let size_desc = wgpu::BlasTriangleGeometrySizeDescriptor {
                    vertex_format: wgpu::VertexFormat::Float32x3,
                    vertex_count: 0 as u32,
                    index_format: Some(wgpu::IndexFormat::Uint32),
                    index_count: Some(3),
                    flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
                };

                tri_size_descs.push(size_desc);
            }
            // let size_desc = wgpu::BlasTriangleGeometrySizeDescriptor {
            //     vertex_format: wgpu::VertexFormat::Float32x3,
            //     vertex_count: (mesh.vertex_count) as u32,
            //     index_format: Some(wgpu::IndexFormat::Uint32),
            //     index_count: Some(mesh.index_count),
            //     flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
            // };

            mesh_size_descs.push(tri_size_descs);
        }

        for (i, mesh) in self.meshes.iter().enumerate() {
            let blas = device.create_blas(
                &wgpu::CreateBlasDescriptor {
                    label: Some(format!("BLAS {}", i).as_str()),
                    flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE 
                        | wgpu::AccelerationStructureFlags::ALLOW_RAY_HIT_VERTEX_RETURN,
                    update_mode: wgpu::AccelerationStructureUpdateMode::Build,
                },
                wgpu::BlasGeometrySizeDescriptors::Triangles {
                    descriptors: mesh_size_descs[i].clone(),
                }
            );

            blases.push(blas);
        }

        let mut blas_build_entries = Vec::new();
        for (i, mesh) in self.meshes.iter().enumerate() {
            let mut tri_geoms = Vec::new();
            for j in (0..mesh.index_count).step_by(3) {
                // let tri_geom = wgpu::BlasTriangleGeometry {
                //     size: &size_descs[i],
                //     vertex_buffer: &vertex_buffer,
                //     first_vertex: mesh.vertex_offset,
                //     vertex_stride: std::mem::size_of::<uniforms::Vertex>() as u64,
                //     index_buffer: Some(&index_buffer),
                //     first_index: Some(0),
                //     transform_buffer: None,
                //     transform_buffer_offset: None,
                // };

                let tri_geom = wgpu::BlasTriangleGeometry {
                    size: &mesh_size_descs[i][(j / 3) as usize],
                    vertex_buffer: &vertex_buffer,
                    first_vertex: 0 as u32,
                    vertex_stride: std::mem::size_of::<uniforms::Vertex>() as u64,
                    index_buffer: Some(&index_buffer),
                    first_index: Some((mesh.index_offset + j) as u32),
                    transform_buffer: None,
                    transform_buffer_offset: None,
                };

                tri_geoms.push(tri_geom);
            }

            blas_build_entries.push(wgpu::BlasBuildEntry {
                blas: &blases[i],
                geometry: wgpu::BlasGeometries::TriangleGeometries(tri_geoms),
            })    
        }

        // for (i, mesh) in self.meshes.iter().enumerate() {
            // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //     label: Some(format!("vertex buffer {}", i).as_str()),
            //     contents: bytemuck::cast_slice(&mesh.vertices),
            //     usage: wgpu::BufferUsages::BLAS_INPUT,
            // });

            // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //     label: Some(format!("index buffer {}", i).as_str()),
            //     contents: bytemuck::cast_slice(&mesh.indices),
            //     usage: wgpu::BufferUsages::BLAS_INPUT,
            // });

            // let blas_size_desc = wgpu::BlasTriangleGeometrySizeDescriptor {
            //     vertex_format: wgpu::VertexFormat::Float32x3,
            //     // 3 coordinates per vertex
            //     vertex_count: (mesh.vertices.len() / 3) as u32,
            //     index_format: Some(wgpu::IndexFormat::Uint32),
            //     index_count: Some(mesh.indices.len() as u32),
            //     flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
            // };

            // let blas = device.create_blas(
            //     &wgpu::CreateBlasDescriptor {
            //         label: None,
            //         flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
            //         update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            //     },
            //     wgpu::BlasGeometrySizeDescriptors::Triangles {
            //         descriptors: vec![blas_size_desc.clone()],
            //     },
            // );

            // blases.push(blas.clone());
            // blas_sizes.push(blas_size_desc);
            
            // let triangle_geometry = wgpu::BlasTriangleGeometry {
            //     size: &blas_size_desc,
            //     vertex_buffer: &vertex_buffer,
            //     first_vertex: 0,
            //     vertex_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            //     index_buffer: Some(&index_buffer),
            //     first_index: Some(0),
            //     transform_buffer: None,
            //     transform_buffer_offset: None
            // };
            
            // blas_build_entries.push(
            //     wgpu::BlasBuildEntry {
            //         blas: &blas.clone(),
            //         geometry: wgpu::BlasGeometries::TriangleGeometries(vec![triangle_geometry]),
            //     }
            // );

            // encoder.build_acceleration_structures(
            //     Some(&wgpu::BlasBuildEntry {
            //         blas: &blas,
            //         geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
            //             size: &blas_size_desc,
            //             vertex_buffer: &vertex_buffer,
            //             first_vertex: 0,
            //             vertex_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            //             // in this case since one triangle gets no compression from an index buffer `index_buffer` and `first_index` could be `None`.
            //             index_buffer: Some(&index_buffer),
            //             first_index: Some(0),
            //             transform_buffer: None,
            //             transform_buffer_offset: None,
            //         }]),
            //     }),
            //     std::iter::empty(),
            // );

            // blases.push(blas);
            // blas_sizes.push(blas_size_desc);
        // }    

        // tlas_package[i] = Some(wgpu::TlasInstance::new(
        //     &blas,
        //     Mat4::from_translation(Vec3 {
        //         x: 0.0,
        //         y: 0.0,
        //         z: 0.0,
        //     })
        //     .transpose()
        //     .to_cols_array()[..12]
        //         .try_into()
        //         .unwrap(),
        //     0,
        //     0xff,
        // ));


        // encoder.build_acceleration_structures(
        //     Some(&wgpu::BlasBuildEntry {
        //         blas: &blas,
        //         geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
        //             size: &blas_size_desc,
        //             vertex_buffer: &vertex_buffer,
        //             first_vertex: 0,
        //             vertex_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        //             // in this case since one triangle gets no compression from an index buffer `index_buffer` and `first_index` could be `None`.
        //             index_buffer: Some(&index_buffer),
        //             first_index: Some(0),
        //             transform_buffer: None,
        //             transform_buffer_offset: None,
        //         }]),
        //     }),
        //     Some(&tlas_package),
        // );

        encoder.build_acceleration_structures(
            blas_build_entries.iter(),
            std::iter::empty()
        );

        queue.submit(Some(encoder.finish()));
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor::preprocess_wgsl!("shaders/shader_main.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&rt_bgl, &texture_bgl],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pipeline for shader.wgsl"),
            layout: Some(&render_pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        (render_pipeline, blases)
    }

    pub fn update_resources(&mut self, wgpu_state: &wgpu_util::WGPUState) -> (wgpu::BindGroup, wgpu::BindGroup) {
        //let uniform_buffer: UniformBuffer = UniformBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&[uniforms::Uniforms::new(&self.camera)]));

        let mut uniforms = uniforms::Uniforms::new(&self.camera);

        let materials_uniform_vec = self.materials.iter().map(|m| uniforms::Material::new(m.clone())).collect::<Vec<uniforms::Material>>();

        let materials_buffer = StorageBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&materials_uniform_vec), Some("Materials"));
        //let materials_buffer = StorageBuffer::new(&wgpu_state.device, wgpu_util::any_as_u8_slice(&materials_uniform_vec));

        // let vertices_vec = self.meshes
        //     .iter()
        //     .map(|m|
        //         m.vertices.clone()
        //     )
        //     .collect::<Vec<Vec<[f32; 3]>>>()
        //     .into_iter()
        //     .flatten()
        //     .map(|v| uniforms::Vertex::new(v))
        //     .collect::<Vec<uniforms::Vertex>>();

        // let vertices_buffer = StorageBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&vertices_vec));
        let vertices_buffer = StorageBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&self.vertices), Some("Vertices"));
        let indices_buffer = StorageBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&self.indices), Some("Indices"));

        // InstanceInfo
        let mut instance_infos = Vec::new();
        for mesh in &mut self.meshes {
            instance_infos.push(uniforms::InstanceInfo::new(mesh.index_offset));
        }

        let instance_info_buffer = StorageBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&instance_infos), Some("InstanceInfos"));
            
        // let mut vertices_info_vec: Vec<uniforms::VertexOffset> = Vec::new();
        // let mut preceeding_vertex_count = 0;

        // for mesh in &self.meshes {
        //     vertices_info_vec.push(uniforms::VertexOffset::new(preceeding_vertex_count));
        //     preceeding_vertex_count += mesh.vertices.len() as u32;
        // }
            
        //let vertices_info_buffer = StorageBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&vertices_info_vec));

        let mut light_triangles = Vec::new();
        for (i, indices) in self.indices.chunks(3).enumerate() {
            // Get the material index from self.meshes's offsets
            let mut mat_idx = 0;
            for mesh in &self.meshes {
                if ((i * 3) as u32) >= mesh.index_offset && ((i * 3) as u32) < mesh.index_offset + mesh.index_count {
                    mat_idx = mesh.material_index;
                    break;
                }
            }

            // let emissive = self.materials[mat_idx as usize].emissive;

            // if emissive[0] == 0.0 && emissive[1] == 0.0 && emissive[2] == 0.0 {
            //     continue;
            // }

            let emissive = self.materials[mat_idx as usize].emissive_texture_idx != 4;
            if !emissive {
                continue;
            }

            light_triangles.push(
                uniforms::Triangle::new(
                    self.vertices[indices[0].index as usize].position,
                    self.vertices[indices[1].index as usize].position,
                    self.vertices[indices[2].index as usize].position,
                    self.vertices[indices[0].index as usize].normal,
                )
            );
        }

        if light_triangles.len() == 0 {
            light_triangles = vec![uniforms::Triangle::new(
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
            )];
        }

        let light_triangles_buffer = StorageBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&light_triangles), Some("LightTriangles"));

        uniforms.num_lights = light_triangles.len() as u32;

        let uniform_buffer: UniformBuffer = UniformBuffer::new(&wgpu_state.device, bytemuck::cast_slice(&[uniforms]), Some("Uniforms"));

        let rt_bgl = self.get_rt_bgl(&wgpu_state.device);

        let rt_bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RT Bind Group"),
            layout: &rt_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &wgpu_state.blit_storage_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu_state.tlas_package.as_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: materials_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: vertices_buffer.buffer().buffer.as_entire_binding(),
                },
                // wgpu::BindGroupEntry {
                //     binding: 5,
                //     resource: vertices_info_buffer.buffer().buffer.as_entire_binding(),
                // },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: indices_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: instance_info_buffer.buffer().buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: light_triangles_buffer.buffer().buffer.as_entire_binding(),
                },
                // wgpu::BindGroupEntry {
                //     binding: 8,
                //     resource: wgpu::BindingResource::TextureViewArray(
                //         &self.textures.iter().map(|texture| &texture.view).collect::<Vec<_>>()
                //     ),
                // },
                // wgpu::BindGroupEntry {
                //     binding: 9,
                //     resource: wgpu::BindingResource::SamplerArray(
                //         &self.textures.iter().map(|texture| &texture.sampler).collect::<Vec<_>>()
                //     ),
                // },
            ],
        });

        let texture_bgl = self.get_texture_bgl(&wgpu_state.device);

        let texture_bind_group = wgpu_state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &self.textures.iter().map(|texture| &texture.view).collect::<Vec<_>>()
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(
                        &self.textures.iter().map(|texture| &texture.sampler).collect::<Vec<_>>()
                    ),
                },
            ],
        });

        (rt_bind_group, texture_bind_group)
    }
}
