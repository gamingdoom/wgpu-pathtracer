use std::{default, mem};

use glam::{Mat4, Vec3};
use wgpu::hal::vulkan::Adapter;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgc::api::Vulkan;
use wgpu::{AccelerationStructureFlags, AccelerationStructureUpdateMode, BlasBuildEntry, BlasGeometries, BlasGeometrySizeDescriptors, BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor, BufferUsages, CreateBlasDescriptor, CreateTlasDescriptor, IndexFormat, TlasInstance, TlasPackage};

use ash::vk;

// use winit::window::Window;
// use winit::event::{WindowEvent};

use crate::wgpu_buffer::{UniformBuffer, BufferType};
use crate::window;

pub struct WGPUState <'a> {
    pub instance: ash::Instance,
    pub adapter: wgpu::Adapter,
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub pp_queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    // pub render_pipeline: wgpu::ComputePipeline,
    // pub render_bind_group: wgpu::BindGroup,
    // pub blit_pipeline: wgpu::RenderPipeline,
    // pub blit_bind_group: wgpu::BindGroup,
    pub blit_storage_texture: wgpu::Texture,
    pub depth_texture: wgpu::Texture,
    pub depth_texture_rt_view: wgpu::TextureView,
    // pub tlas_package: TlasPackage,
    pub size: (u32, u32),
    pub window: &'a sdl3::video::Window,
    pub sdl_context: &'a sdl3::Sdl,
    pub rt_device: wgpu::Device,
    pub rt_queue: wgpu::Queue,
    pub window_cursor_grabbed: bool,
    pub refresh_rate: f32,

    pub latest_real_frame_rt: Option<wgpu::Texture>,

    pub oidn_device: oidn_wgpu_interop::Device,
}

impl<'a> WGPUState<'a>{
    pub fn new(sdl_context: &'a sdl3::Sdl, window: &'a sdl3::video::Window) -> WGPUState<'a> {
        let size = window.size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN, 
            flags: wgpu::InstanceFlags::empty(),
            //flags: wgpu::InstanceFlags::VALIDATION | wgpu::InstanceFlags::GPU_BASED_VALIDATION,
            ..Default::default()
        });

        let surface = crate::window::create_surface::create_surface(&instance, &window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).unwrap();


        // this is so jank
        // Essentially, we tell wgpu that there are 2 vk devices, even though there is only 1. This is so that we can have 2 queues.
        let vk_instance = unsafe {instance.as_hal::<Vulkan>().unwrap()}.shared_instance().raw_instance();
        
        let vk_physdev = unsafe { adapter.as_hal::<Vulkan, _, _>(|adapter| {return adapter.unwrap().raw_physical_device()})};

        let features = 
            wgpu::Features::EXPERIMENTAL_RAY_TRACING_ACCELERATION_STRUCTURE
            | wgpu::Features::EXPERIMENTAL_RAY_QUERY
            | wgpu::Features::EXPERIMENTAL_RAY_HIT_VERTEX_RETURN
            | wgpu::Features::TEXTURE_BINDING_ARRAY
            | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | wgpu::Features::FLOAT32_FILTERABLE
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | wgpu::Features::CLEAR_TEXTURE
            | wgpu::Features::SPIRV_SHADER_PASSTHROUGH
            | wgpu::Features::TEXTURE_COMPRESSION_BC;

        let mut extensions = unsafe { adapter.as_hal::<Vulkan, _, _>(|adapter| {return adapter.unwrap().required_device_extensions(
            features
        )})};

        // extensions.push(c"VK_EXT_acquire_drm_display");

        #[cfg(target_os = "linux")]
            extensions.push(c"VK_KHR_external_memory_fd");
            extensions.push(c"VK_EXT_external_memory_dma_buf");
        
        #[cfg(target_os = "windows")]
            extensions.push(c"VK_KHR_external_memory_win32");

        // Get queue families
        let queue_fam_props = unsafe { vk_instance.get_physical_device_queue_family_properties(vk_physdev) };

        let mut family_idx: usize = 0;
        for (i, queue_family) in queue_fam_props.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE | vk::QueueFlags::GRAPHICS) {
                println!("Found queue family at index {}", i);
                family_idx = i;
            }
        }

        let mut queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(family_idx as u32)
            .queue_priorities(&[1.0f32, 0.0f32]);
        queue_create_info.queue_count = 2;

        let ext_ptr = &extensions.iter()
        .map(
            |ext| ext.as_ptr()
        )
        .collect::<Vec<_>>();

        let queue_create_infos = [queue_create_info];

        let dev_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&ext_ptr);

        let mut physdev_feats = unsafe { adapter.as_hal::<Vulkan, _, _>(|adapter| {
            return adapter.unwrap().physical_device_features(&extensions, features)
        })};

        let dev_create_info = physdev_feats.add_to_device_create(dev_create_info);

        let dev = unsafe { vk_instance.create_device(vk_physdev, &dev_create_info, None) }.unwrap();

        // let raytrace_queue = unsafe { dev.get_device_queue(family_idx as u32, 0) };
        // let postprocessing_queue = unsafe { dev.get_device_queue(family_idx as u32, 1) };

        let hal_device = unsafe {
            adapter.as_hal::<Vulkan, _, _>(|adapter| {
                return adapter.unwrap().device_from_raw(
                    dev.clone(),
                    None,
                    &extensions,
                    features,
                    &wgpu::MemoryHints::Performance,
                    family_idx as u32,
                    1
                )
            })
        }.unwrap();

        let hal_device_2 = unsafe {
            adapter.as_hal::<Vulkan, _, _>(|adapter| {
                return adapter.unwrap().device_from_raw(
                    dev.clone(),
                    None,
                    &extensions,
                    features,
                    &wgpu::MemoryHints::Performance,
                    family_idx as u32,
                    0
                )
            })
        }.unwrap();

        let (device, queue) = unsafe { adapter.create_device_from_hal(hal_device, &wgpu::DeviceDescriptor {
            label: Some("pp device"),
            required_features: features,
            required_limits: wgpu::Limits {
                max_binding_array_elements_per_shader_stage: 500000,
                max_binding_array_sampler_elements_per_shader_stage: 1000,
                max_buffer_size: 1024 * 1024 * 1024,
                max_storage_buffer_binding_size: 1024 * 1024 * 1024,
                max_compute_invocations_per_workgroup: 4096,
                ..Default::default()
            },
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })}.unwrap();
        
        let (device_2, queue_2) = unsafe { adapter.create_device_from_hal(hal_device_2, &wgpu::DeviceDescriptor {
            label: Some("rt device"),
            required_features: features,
            required_limits: wgpu::Limits {
                max_binding_array_elements_per_shader_stage: 500000,
                max_binding_array_sampler_elements_per_shader_stage: 1000,
                max_buffer_size: 1024 * 1024 * 1024,
                max_storage_buffer_binding_size: 1024 * 1024 * 1024,
                max_compute_invocations_per_workgroup: 4096,
                ..Default::default()
            },
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })}.unwrap();

        let (oidn_device, queue_2) = pollster::block_on(oidn_wgpu_interop::Device::new_from_dev(&adapter, device_2, queue_2, None)).unwrap();

        let device_2 = oidn_device.wgpu_device().to_owned();

        unsafe {
            //device.start_graphics_debugger_capture();
            device_2.start_graphics_debugger_capture();
        }

        return Self::resize(device, device_2, queue, queue_2, vk_instance, adapter, oidn_device, surface, window, sdl_context, size);

        // let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        //         required_features: 
        //             wgpu::Features::EXPERIMENTAL_RAY_TRACING_ACCELERATION_STRUCTURE
        //             | wgpu::Features::EXPERIMENTAL_RAY_QUERY
        //             | wgpu::Features::EXPERIMENTAL_RAY_HIT_VERTEX_RETURN
        //             | wgpu::Features::TEXTURE_BINDING_ARRAY
        //             | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        //             | wgpu::Features::FLOAT32_FILTERABLE
        //             | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        //             | wgpu::Features::CLEAR_TEXTURE,
        //         required_limits: wgpu::Limits {
        //             max_binding_array_elements_per_shader_stage: 500000,
        //             max_binding_array_sampler_elements_per_shader_stage: 1000,
        //             max_buffer_size: 1024 * 1024 * 1024,
        //             max_storage_buffer_binding_size: 1024 * 1024 * 1024,
        //             ..Default::default()
        //         },
        //         label: None,
        //         ..Default::default()
        //     }
        // ).await.unwrap();


    }

    pub fn resize(device: wgpu::Device, device_2: wgpu::Device, pp_queue: wgpu::Queue, rt_queue: wgpu::Queue, instance: &ash::Instance, adapter: wgpu::Adapter, oidn_device: oidn_wgpu_interop::Device, surface: wgpu::Surface<'a>, window: &'a sdl3::video::Window, sdl_context: &'a sdl3::Sdl, size: (u32, u32)) -> WGPUState<'a> {
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            //present_mode: surface_caps.present_modes[0],
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let storage_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let depth_tex_desc = &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let depth_tex: wgpu::Texture = device.create_texture(depth_tex_desc);

        let raw_tex = unsafe { depth_tex.as_hal::<Vulkan, _, _>(|tex| {
            tex.unwrap().raw_handle()
        }) };

        let depth_tex_rt = unsafe { device_2.create_texture_from_hal::<Vulkan>(
            device_2.as_hal::<Vulkan, _, _>(|dev| wgpu::hal::vulkan::Device::texture_from_raw(
                raw_tex, 
                &wgpu::hal::TextureDescriptor {
                    label: Some("prev_frame"),
                    size: wgpu::Extent3d {
                        width: config.width,
                        height: config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R32Float,
                    view_formats: (&[]).to_vec(),
                    usage: wgpu::TextureUses::STORAGE_WRITE_ONLY,
                    memory_flags: wgpu::hal::MemoryFlags::empty()
                }, 
                Some(Box::new(|| {
                    println!("asked to drop?");
                }))
            )),
            depth_tex_desc
        ) };

        let refresh_rate = 1.0 / ((window.get_display().unwrap().get_mode().unwrap().refresh_rate));
                
        Self {
            instance: instance.clone(),
            adapter,
            surface,
            device,
            pp_queue: pp_queue,
            config,
            blit_storage_texture: storage_tex,
            depth_texture: depth_tex,
            depth_texture_rt_view: depth_tex_rt.create_view(&wgpu::TextureViewDescriptor::default()),
            size,
            window,
            sdl_context,
            rt_device: device_2,
            rt_queue: rt_queue,
            window_cursor_grabbed: false,
            refresh_rate,
            latest_real_frame_rt: None,
            oidn_device
        }
    }

    pub fn resize_2(&mut self) {
        // let surface_caps = self.surface.get_capabilities(&self.adapter);
        // let surface_format = surface_caps.formats.iter()
        //     .find(|f| f.is_srgb())
        //     .copied()
        //     .unwrap_or(surface_caps.formats[0]);

        // self.config = wgpu::SurfaceConfiguration {
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        //     format: surface_format,
        //     width: self.size.width,
        //     height: self.size.height,
        //     present_mode: surface_caps.present_modes[0],
        //     alpha_mode: surface_caps.alpha_modes[0],
        //     view_formats: vec![],
        //     desired_maximum_frame_latency: 2,
        // };

        // self.surface.configure(&self.device, &self.config);

        // // let storage_tex = self.device.create_texture(&wgpu::TextureDescriptor {
        // //     label: None,
        // //     size: wgpu::Extent3d {
        // //         width: self.config.width,
        // //         height: self.config.height,
        // //         depth_or_array_layers: 1,
        // //     },
        // //     mip_level_count: 1,
        // //     sample_count: 1,
        // //     dimension: wgpu::TextureDimension::D2,
        // //     format: wgpu::TextureFormat::Rgba32Float,
        // //     usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        // //     view_formats: &[],
        // // });

        // // self.blit_storage_texture = storage_tex;

        // let depth_tex_desc = &wgpu::TextureDescriptor {
        //     label: Some("depth_tex"),
        //     size: wgpu::Extent3d {
        //         width: self.config.width,
        //         height: self.config.height,
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::R32Float,
        //     usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        //     view_formats: &[],
        // };

        // let mut depth_texture = self.device.create_texture(depth_tex_desc);
        // //std::mem::forget(depth_texture);
        
        // let mut depth_texture_rt = unsafe { self.rt_device.create_texture_from_hal::<Vulkan>(
        //     self.rt_device.as_hal::<Vulkan, _, _>(|dev| wgpu::hal::vulkan::Device::texture_from_raw(
        //         self.depth_texture.as_hal::<Vulkan, _, _>(|tex| {
        //             let handle = tex.unwrap().raw_handle();
        //             handle
        //         }), 
        //         &wgpu::hal::TextureDescriptor {
        //             label: Some("depth_tex"),
        //             size: wgpu::Extent3d {
        //                 width: self.config.width,
        //                 height: self.config.height,
        //                 depth_or_array_layers: 1,
        //             },
        //             mip_level_count: 1,
        //             sample_count: 1,
        //             dimension: wgpu::TextureDimension::D2,
        //             format: wgpu::TextureFormat::R32Float,
        //             view_formats: (&[]).to_vec(),
        //             usage: wgpu::TextureUses::STORAGE_WRITE_ONLY,
        //             memory_flags: wgpu::hal::MemoryFlags::empty()
        //         }, 
        //         Some(Box::new(|| {
        //             println!("asked to drop resized?");
        //         }))
        //     )),
        //     depth_tex_desc
        // ) };
        
        // let view = depth_texture_rt.create_view(&wgpu::TextureViewDescriptor::default());

        // self.depth_texture = depth_texture;
        // self.depth_texture_rt_view = view;

        // println!("resized");
        let surface_caps = self.surface.get_capabilities(&self.adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.size.0,
            height: self.size.1,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            //present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        self.surface.configure(&self.device, &config);

        let storage_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let depth_tex_desc = &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let depth_tex: wgpu::Texture = self.device.create_texture(depth_tex_desc);

        let raw_tex = unsafe { depth_tex.as_hal::<Vulkan, _, _>(|tex| {
            tex.unwrap().raw_handle()
        }) };

        let depth_tex_rt = unsafe { self.rt_device.create_texture_from_hal::<Vulkan>(
            self.rt_device.as_hal::<Vulkan, _, _>(|dev| wgpu::hal::vulkan::Device::texture_from_raw(
                raw_tex, 
                &wgpu::hal::TextureDescriptor {
                    label: Some("prev_frame"),
                    size: wgpu::Extent3d {
                        width: config.width,
                        height: config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R32Float,
                    view_formats: (&[]).to_vec(),
                    usage: wgpu::TextureUses::STORAGE_WRITE_ONLY,
                    memory_flags: wgpu::hal::MemoryFlags::empty()
                }, 
                Some(Box::new(|| {
                    println!("asked to drop?");
                }))
            )),
            depth_tex_desc
        ) };

        self.blit_storage_texture = storage_tex;
        self.depth_texture = depth_tex;
        self.depth_texture_rt_view = depth_tex_rt.create_view(&wgpu::TextureViewDescriptor::default());

        self.refresh_rate = 1.0 / ((self.window.get_display().unwrap().get_mode().unwrap().refresh_rate));

    }

    pub fn window(&self) -> &sdl3::video::Window {
        &self.window
    }
}

pub fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts(
            (p as *const T) as *const u8,
            ::core::mem::size_of::<T>(),
        )
    }
}