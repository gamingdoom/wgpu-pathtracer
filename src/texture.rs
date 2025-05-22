use image::{EncodableLayout, GenericImageView, Luma, Rgb, Rgba};

#[derive(Debug, Clone)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub texture_size: (u32, u32),
    pub color: Option<[f32; 4]>,
    pub scalar: Option<f32>
}

// pub struct TextureUniform {
//     pub texture_view: wgpu::TextureView,
//     pub sampler: wgpu::Sampler,
// }

// impl TextureUniform {
//     pub fn new(texture: Texture) -> Self {
//         Self {
//             texture_view: texture.view,
//             sampler: texture.sampler,
//         }
//     }
// }

impl Texture {
    pub fn from_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: image::Rgba<f32>,
        label: Option<&str>
    ) -> Result<Self, image::ImageError> {
        let img = image::DynamicImage::ImageRgba32F(image::ImageBuffer::from_pixel(1, 1, color));
        let tex = Self::from_image_rgba(device, queue, &img, label);

        if tex.is_ok() {
            let mut tex = tex.unwrap();
            tex.color = Some([color.0[0], color.0[1], color.0[2], color.0[3]]);
            Ok(tex)
        } else {
            tex
        }
        
    }

    pub fn from_scalar(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scalar: Luma<f32>,
        label: Option<&str>
    ) -> Result<Self, image::ImageError> {
        let tex = Self::from_color(device, queue, image::Rgba([scalar.0[0], 0.0, 0.0, 1.0]), label);

        if tex.is_ok() {
            let mut tex = tex.unwrap();
            tex.scalar = Some(scalar.0[0]);
            Ok(tex)
        } else {
            tex
        }
    }
    

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8], 
        label: &str
    ) -> Result<Self, image::ImageError> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image_rgba(device, queue, &img, Some(label))
    }

    pub fn from_image_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>
    ) -> Result<Self, image::ImageError> {
        let rgba = img.to_rgba32f();
        
        let is_compressed = img.dimensions().0 % 4 == 0 && img.dimensions().1 % 4 == 0;

        let mut format = wgpu::TextureFormat::Rgba32Float;
        let mut data;
        let mut bytes_per_row;
        if is_compressed {
            let compressed = image_dds::SurfaceRgba32Float::from_image(&rgba).encode(
                image_dds::ImageFormat::BC7RgbaUnorm,
                image_dds::Quality::Fast,
                image_dds::Mipmaps::Disabled
            );
            format = wgpu::TextureFormat::Bc7RgbaUnorm;
            data = compressed.unwrap().data;

            bytes_per_row = Some(16 * (img.dimensions().0 / 4) as u32);

            println!("{}", bytes_per_row.unwrap());
        } else {
            data = rgba.as_bytes().to_vec();
            bytes_per_row = Some(4 * img.dimensions().0 * std::mem::size_of::<f32>() as u32);
        }

        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                //format: wgpu::TextureFormat::Rgba32Float,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            //&rgba.as_bytes(),
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: bytes_per_row,//Some(4 * dimensions.0 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );
        
        Ok(Self { texture, view, sampler , texture_size: (dimensions.0, dimensions.1), color: None, scalar: None })
    }

    pub fn from_image_scalar(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::ImageBuffer<Luma<f32>, Vec<f32>>,
        label: Option<&str>
    ) -> Result<Self, image::ImageError> {
        let scalar = img;
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &scalar.as_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(dimensions.0 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );
        
        Ok(Self { texture, view, sampler , texture_size: (dimensions.0, dimensions.1), color: None, scalar: None })
    }

    pub fn from_file_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
        label: Option<&str>
    ) -> Result<Self, image::ImageError> {
        let img = image::open(path).unwrap();

        Self::from_image_rgba(device, queue, &img, label)
    }

    pub fn from_file_scalar(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
        label: Option<&str>
    ) -> Result<Self, image::ImageError> {
            let img = image::open(path).unwrap();

            Self::from_image_rgba(device, queue, &img, label)
            //Self::from_image_scalar(device, queue, &img.to_luma32f(), label)
    }

}
