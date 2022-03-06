use bytemuck::{ Pod, Zeroable };

pub trait Vertex: Pod + Zeroable {
    fn describe_buffer<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UvVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex for UvVertex {
    fn describe_buffer<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                },
            ]
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tex_index: i32,
}

impl Vertex for TexturedVertex {
    fn describe_buffer<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                },
                wgpu::VertexAttribute {
                    shader_location: 2,
                    format: wgpu::VertexFormat::Sint32,
                    offset: size_of::<[f32; 5]>() as wgpu::BufferAddress,
                }
            ],
        }
    }
}
