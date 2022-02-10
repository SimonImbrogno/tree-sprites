use std::mem::size_of;

use super::super::buffer_usages::BufferUsages;
use super::super::utils::gpu::create_buffer;
use super::super::vertex::Vertex;
use super::{Buffer, Index, WriteGeometryBuffer, DrawGeometryBuffer};

pub struct GeometryBuffer<V: Vertex, I: Index> {
    pending_writes: bool,

    pub vertex_capacity: usize,
    vertex_cache: Vec<V>,
    pub vertex_buffer: wgpu::Buffer,

    pub index_capacity: usize,
    index_cache: Vec<I>,
    pub index_buffer: wgpu::Buffer,
}

impl<V: Vertex, I: Index> Buffer<V, I> for GeometryBuffer<V, I> { }

impl<V: Vertex, I: Index> GeometryBuffer<V, I> {
    pub fn new(device: &wgpu::Device, label: &'static str, vertex_capacity: usize, index_capacity: usize) -> Self {
        debug_assert!(vertex_capacity > 0);
        debug_assert!(index_capacity > 0);

        let vertex_buffer = create_buffer(device, &format!("{}.vertex_buffer", label), vertex_capacity * size_of::<V>(), BufferUsages::VertexCopyDst.into());
        let index_buffer  = create_buffer(device, &format!("{}.index_buffer", label),  index_capacity * size_of::<I>(), BufferUsages::IndexCopyDst.into());

        let vertex_cache = Vec::with_capacity(vertex_capacity);
        let index_cache = Vec::with_capacity(index_capacity);

        Self {
            vertex_capacity,
            index_capacity,
            pending_writes: false,

            vertex_cache,
            vertex_buffer,

            index_cache,
            index_buffer,
        }
    }

    pub fn reset(&mut self) {
        self.index_cache.clear();
        self.vertex_cache.clear();
    }

    fn debug_assert_capacity(&self, new_vertices: &[V], new_indices: &[I]) {
        #[cfg(debug_assertions)] {
            let new_vertex_count = self.vertex_cache.len() + new_vertices.len();
            let new_index_count  = self.index_cache.len()  + new_indices.len();

            let has_vert_cap = new_vertex_count <= self.vertex_capacity;
            let has_index_cap = new_index_count <= self.index_capacity;

            debug_assert!(
                has_vert_cap,
                "Attempting to write vertices to {}B, but capacity only {}B",
                new_vertex_count * size_of::<V>(),
                self.vertex_capacity * size_of::<V>()
            );

            debug_assert!(
                has_index_cap,
                "Attempting to write indices to {}B, but capacity only {}B",
                new_index_count * size_of::<I>(),
                self.index_capacity * size_of::<I>()
            );
        }
    }

    pub fn has_capacity(&self, new_vertices: &[V], new_indices: &[I]) -> bool {
        let new_vertex_count = self.vertex_cache.len() + new_vertices.len();
        let new_index_count  = self.index_cache.len()  + new_indices.len();

        let has_vert_cap = new_vertex_count <= self.vertex_capacity;
        let has_index_cap = new_index_count <= self.index_capacity;

        (has_vert_cap && has_index_cap)
    }

    pub fn push_geometry(&mut self, new_vertices: &[V], new_indices: &[I]) {
        self.pending_writes = true;
        self.debug_assert_capacity(new_vertices, new_indices);

        self.vertex_cache.extend(new_vertices);
        self.index_cache.extend(new_indices);
    }
}

impl<V, I> WriteGeometryBuffer<GeometryBuffer<V, I>, V, I> for wgpu::Queue
where
    V: Vertex,
    I: Index
{
    fn write_geometry_buffer(&self, buffer: &mut GeometryBuffer<V, I>) {
        buffer.pending_writes = false;
        self.write_buffer(&buffer.vertex_buffer, 0, bytemuck::cast_slice(&buffer.vertex_cache));
        self.write_buffer(&buffer.index_buffer, 0, bytemuck::cast_slice(&buffer.index_cache));
    }
}

impl<'b, 'r, V, I> DrawGeometryBuffer<'b, GeometryBuffer<V, I>, V, I> for wgpu::RenderPass<'r>
where
    'b: 'r,
    V: Vertex,
    I: Index
{
    fn draw_geometry_buffer(&mut self, buffer: &'b GeometryBuffer<V, I>) {
        self.set_vertex_buffer(0, buffer.vertex_buffer.slice(..));
        self.set_index_buffer(buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..buffer.index_cache.len() as u32, 0, 0..1);
    }
}
