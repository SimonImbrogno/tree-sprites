use std::slice::SliceIndex;

use super::geometry_buffer::GeometryBuffer;
use super::super::vertex::Vertex;
use super::{Buffer, Index, WriteGeometryBuffer, DrawGeometryBuffer, ViewableBuffer};

pub struct QuadBuffer<V, I>
where
    V: Vertex,
    I: Index
{
    pub buffer: GeometryBuffer<V, I>,
    next_quad_index: usize,
}

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
            next_quad_index: 0,
        }
    }

    pub fn push_quad<Q>(&mut self, quad: Q)
    where
        Q: Into<[V; 4]>
    {
        let new_vertices: &[V; 4] = &quad.into();

        let base = self.next_quad_index * 4;
        let new_indices = &[
            I::from_usize(base + 0), I::from_usize(base + 1), I::from_usize(base + 2),
            I::from_usize(base + 2), I::from_usize(base + 3), I::from_usize(base + 0)
        ];

        self.buffer.push_geometry(new_vertices, new_indices);
        self.next_quad_index += 1;
    }
}

impl<V, I> Buffer for QuadBuffer<V, I>
where
    V: Vertex,
    I: Index
{
    fn reset(&mut self) {
        self.buffer.reset();
        self.next_quad_index = 0;
    }

    fn vertex_count(&self) -> usize {
        self.buffer.vertex_count()
    }

    fn vertex_capacity(&self) -> usize {
        self.buffer.vertex_capacity()
    }

    fn remaining_vertex_capacity(&self) -> usize {
        self.buffer.remaining_vertex_capacity()
    }

    fn index_count(&self) -> usize {
        self.buffer.index_count()
    }

    fn index_capacity(&self) -> usize {
        self.buffer.index_capacity()
    }

    fn remaining_index_capacity(&self) -> usize {
        self.buffer.remaining_index_capacity()
    }
}

impl<V, I>  ViewableBuffer<V, I> for QuadBuffer<V, I>
where
    V: Vertex,
    I: Index
{
    // TODO: is there some way to abstract over SliceIndex implementors to correctly adjust indices? :/
    fn get(&self, index: usize) -> Option<&[V]>
    {
        let begin = index * 4;
        let end = begin + 4;

        self.buffer.get(begin..end)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut [V]>
    {
        let begin = index * 4;
        let end = begin + 4;

        self.buffer.get_mut(begin..end)
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
