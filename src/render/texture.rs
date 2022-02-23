use std::path::Path;

use anyhow::*;
use log::debug;
use image::GenericImageView;

pub struct Texture {
    pub device_texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

#[derive(Debug, Clone)]
pub enum TextureCreationError {
    Error(&'static str),
    ArrayTextureTooManyLayers,
    ArrayTextureSizeMismatch,
}

impl std::fmt::Display for TextureCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(s)                  => write!(f, "{}", s),
            Self::ArrayTextureTooManyLayers => write!(f, "array texture data contains too many layers"),
            Self::ArrayTextureSizeMismatch  => write!(f, "array texture data size mismatch"),
        }
    }
}

impl std::error::Error for TextureCreationError { }

impl Texture {
    pub fn try_from_path<P: AsRef<Path>>(path: P, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let pbuf  = path.as_ref().to_path_buf();
        let label = pbuf.to_str();
        let image = image::open(path)?;

        Self::try_from_image(label, &image, device, queue)
    }

    pub fn try_from_bytes(label: Option<&str>, bytes: &[u8], device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let image = image::load_from_memory(bytes)?;
        Self::try_from_image(label, &image, device, queue)
    }

    pub fn try_from_image(label: Option<&str>, image: &image::DynamicImage, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        let image_rgba = image.to_rgba8();
        let (x_dim, y_dim) = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: x_dim,
            height: y_dim,
            depth_or_array_layers: 1,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let device_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &device_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * x_dim),
                rows_per_image: std::num::NonZeroU32::new(y_dim),
            },
            texture_size
        );

        let view = device_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        Ok(Self { device_texture, view, sampler, format })
    }

    pub fn try_create_array_texture_from_images(images: &[image::DynamicImage], device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self> {
        use TextureCreationError::*;

        debug!("loading array texture");

        let wgpu_limits = wgpu::Limits::downlevel_defaults();
        let max_layers = wgpu_limits.max_texture_array_layers as usize;

        if images.len() > max_layers { return Err(ArrayTextureTooManyLayers.into()); }
        if images.len() <= 1         { return Err(Error("missing initialization data").into()); }

        let (width, height) = images[0].dimensions();

        if !images.iter().all(|i| {
            let (x_dim, y_dim) = i.dimensions();
            (x_dim == width) && (y_dim == height)
        }) {
            return Err(ArrayTextureSizeMismatch.into());
        }

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: images.len() as u32,
        };

        debug!("Determined size: {:?}", texture_size);

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let device_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: None,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            }
        );
        debug!("beginning write...");

        for (index, image) in images.iter().enumerate() {
            let rgba = image.to_rgba8();

            let mut write_extent = texture_size.clone();
            write_extent.depth_or_array_layers = 1;

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &device_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: index as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * width),
                    rows_per_image: std::num::NonZeroU32::new(height),
                },
                write_extent
            );
        }

        debug!("Creating view");

        let view = device_texture.create_view(
            &wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            }
        );
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }
        );

        Ok(Self { device_texture, view, sampler, format })
    }

    // pub fn create_surface_sized_depth_texture(label: &str, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
    //     let format = wgpu::TextureFormat::Depth32Float;
    //     let size = wgpu::Extent3d {
    //         width: config.width,
    //         height: config.height,
    //         depth_or_array_layers: 1,
    //     };

    //     let device_texture = device.create_texture(
    //         &wgpu::TextureDescriptor {
    //             label: Some(label),
    //             size,
    //             mip_level_count: 1,
    //             sample_count: 1,
    //             dimension: wgpu::TextureDimension::D2,
    //             format,
    //             usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    //         }
    //     );

    //     let view = device_texture.create_view(&wgpu::TextureViewDescriptor::default());

    //     let sampler = device.create_sampler(
    //         &wgpu::SamplerDescriptor {
    //             address_mode_u: wgpu::AddressMode::ClampToEdge,
    //             address_mode_v: wgpu::AddressMode::ClampToEdge,
    //             address_mode_w: wgpu::AddressMode::ClampToEdge,
    //             mag_filter: wgpu::FilterMode::Linear,
    //             min_filter: wgpu::FilterMode::Linear,
    //             mipmap_filter: wgpu::FilterMode::Nearest,
    //             lod_min_clamp: -100.0,
    //             lod_max_clamp: 100.0,
    //             compare: Some(wgpu::CompareFunction::LessEqual),
    //             ..Default::default()
    //         }
    //     );

    //     Self { device_texture, view, sampler, format }
    // }
}
