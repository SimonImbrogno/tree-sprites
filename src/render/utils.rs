pub mod gpu {
    use wgpu::util::DeviceExt;
    use log::debug;

    pub fn create_shader_module(device: &wgpu::Device, label: &str, src: &str) -> wgpu::ShaderModule {
        device.create_shader_module(
            &wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            }
        )
    }

    pub fn create_buffer(device: &wgpu::Device, label: &str, len_bytes: usize, usage: wgpu::BufferUsages) -> wgpu::Buffer {
        debug!("Allocating {}B buffer: {}", len_bytes, label);

        // NOTE: This is somewhat cannibalized from the wgpu implementation of `create_buffer_init`
        //       Buffers (might) need a specific alignment depending on backend, so we align them always.
        const ALIGN_MASK: wgpu::BufferAddress = wgpu::COPY_BUFFER_ALIGNMENT - 1;

        let unpadded_size = len_bytes as wgpu::BufferAddress;
        let padded_size = ((unpadded_size + ALIGN_MASK) & !ALIGN_MASK).max(wgpu::COPY_BUFFER_ALIGNMENT);

        device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some(label),
                size: padded_size,
                usage,
                mapped_at_creation: false,
            }
        )
    }

    pub fn create_buffer_init<T>(device: &wgpu::Device, label: &str, data: &[T], usage: wgpu::BufferUsages) -> wgpu::Buffer
    where
        T: bytemuck::Pod
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(data),
                usage,
            }
        )
    }

    pub fn create_render_pipeline(
        device: &wgpu::Device,
        label: &str,
        layout: &wgpu::PipelineLayout,
        buffer_layouts: &[wgpu::VertexBufferLayout],
        fragment_color_format: wgpu::TextureFormat,
        shader_module: &wgpu::ShaderModule
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader_module,
                    entry_point: "main",
                    buffers: &buffer_layouts,
                },
                fragment: Some(
                    wgpu::FragmentState {
                        module: shader_module,
                        entry_point: "main",
                        targets: &[
                            wgpu::ColorTargetState {
                                format: fragment_color_format,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            }
                        ],
                    }
                ),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    clamp_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                // depth_stencil: Some(wgpu::DepthStencilState {
                //     format: todo!(),
                //     depth_write_enabled: todo!(),
                //     depth_compare: todo!(),
                //     stencil: todo!(),
                //     bias: todo!(),
                // })),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            }
        )
    }
}
