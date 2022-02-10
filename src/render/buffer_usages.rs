pub enum BufferUsages {
    IndexCopyDst,
    UniformCopyDst,
    VertexCopyDst,
}

impl From<BufferUsages> for wgpu::BufferUsages {
    fn from(src: BufferUsages) -> Self {
        match src {
            BufferUsages::IndexCopyDst   => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
            BufferUsages::UniformCopyDst => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            BufferUsages::VertexCopyDst  => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        }
    }
}
