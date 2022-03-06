use super::geometry_buffer::GeometryBuffer;
use super::super::vertex::Vertex;
use super::{Buffer, Index, WriteGeometryBuffer, DrawGeometryBuffer};

pub struct QuadBuffer<V, I>
where
    V: Vertex,
    I: Index
{
    pub buffer: GeometryBuffer<V, I>,
    quad_index: usize,
}

impl<V, I> Buffer<V, I> for QuadBuffer<V, I>
where
    V: Vertex,
    I: Index
{ }

impl<V, I> QuadBuffer<V, I>
where
    V: Vertex,
    I: Index
{
    pub fn new(device: &wgpu::Device, label: &'static str, capacity: usize) -> Self {
        let vertex_capacity = capacity * 4;
        let index_capacity = capacity * 6;

        Self {
            buffer: GeometryBuffer::new(device, label, vertex_capacity, index_capacity),
            quad_index: 0,
        }
    }

    pub fn push_quad<Q>(&mut self, quad: Q)
    where
        Q: Into<[V; 4]>
    {
        let new_vertices: &[V; 4] = &quad.into();

        let base = self.quad_index * 4;
        let new_indices = &[
            I::from_usize(base + 0), I::from_usize(base + 1), I::from_usize(base + 2),
            I::from_usize(base + 2), I::from_usize(base + 3), I::from_usize(base + 0)
        ];

        self.buffer.push_geometry(new_vertices, new_indices);
        self.quad_index += 1;
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
        self.quad_index = 0;
    }

    pub fn quad_index(&self) -> usize {
        self.quad_index
    }

    pub fn set_quad_index(&mut self, new: usize) {
        self.quad_index = new;
    }
}

impl<V, I> WriteGeometryBuffer<QuadBuffer<V, I>, V, I> for wgpu::Queue
where
    V: Vertex,
    I: Index
{
    fn write_geometry_buffer(&self, buffer: &mut QuadBuffer<V, I>) {
        self.write_geometry_buffer(&mut buffer.buffer);
    }
}

impl<'b, 'r, V, I> DrawGeometryBuffer<'b, QuadBuffer<V, I>, V, I> for wgpu::RenderPass<'r>
where
    'b: 'r,
    V: Vertex,
    I: Index
{
    fn draw_geometry_buffer(&mut self, buffer: &'b QuadBuffer<V, I>) {
        self.draw_geometry_buffer(&buffer.buffer);
    }
}
