use std::mem::size_of;
use std::slice::SliceIndex;

use super::super::buffer_usages::BufferUsages;
use super::super::utils::gpu::create_buffer;
use super::super::vertex::Vertex;
use super::{Buffer, Index, WriteGeometryBuffer, DrawGeometryBuffer};

pub struct GeometryBuffer<V: Vertex, I: Index> {
    pending_writes: bool,
    over_capacity: bool,

    pub vertex_capacity: usize,
    vertex_cache: Vec<V>,
    pub vertex_buffer: wgpu::Buffer,

    pub index_capacity: usize,
    index_cache: Vec<I>,
    pub index_buffer: wgpu::Buffer,
}

impl<V: Vertex, I: Index> GeometryBuffer<V, I> {
    pub fn new(device: &wgpu::Device, label: &'static str, vertex_capacity: usize, index_capacity: usize) -> Self {
        debug_assert!(vertex_capacity > 0);
        debug_assert!(index_capacity > 0);

        let vertex_buffer = create_buffer(device, &format!("{}.vertex_buffer", label), vertex_capacity * size_of::<V>(), BufferUsages::VertexCopyDst.into());
        let index_buffer  = create_buffer(device, &format!("{}.index_buffer", label),  index_capacity * size_of::<I>(), BufferUsages::IndexCopyDst.into());

        let vertex_cache = Vec::with_capacity(vertex_capacity);
        let index_cache = Vec::with_capacity(index_capacity);

        Self {
            pending_writes: false,
            over_capacity: false,

            vertex_capacity,
            index_capacity,

            vertex_cache,
            vertex_buffer,

            index_cache,
            index_buffer,
        }
    }

    pub fn new_with_quad_capacity(device: &wgpu::Device, label: &'static str, quad_capacity: usize) -> Self {
        let vertex_capacity = quad_capacity * 4;
        let index_capacity = quad_capacity * 6;

        Self::new(device, label, vertex_capacity, index_capacity)
    }

    pub fn has_capacity(&self, new_vertices: &[V], new_indices: &[I]) -> bool {
        let has_vert_cap = new_vertices.len() <= self.remaining_vertex_capacity();
        let has_index_cap = new_indices.len() <= self.remaining_index_capacity();

        (has_vert_cap && has_index_cap)
    }

    pub fn push_geometry(&mut self, new_vertices: &[V], new_indices: &[I]) {
        self.pending_writes = true;
        if self.has_capacity(new_vertices, new_indices) {
            self.vertex_cache.extend(new_vertices);
            self.index_cache.extend(new_indices);
        } else {
            self.over_capacity = true;
        }
    }

    pub fn push_quad<Q>(&mut self, quad: Q)
    where
        Q: Into<[V; 4]>
    {
        let new_vertices: &[V; 4] = &quad.into();
        let new_indices = self.generate_quad_vertices();

        self.push_geometry(new_vertices, &new_indices);
    }

    // pub fn push_quad_vertices<Q>(&mut self, new_vertices: &[V; 4]) {
    //     let new_indices = self.generate_quad_vertices();

    //     self.push_geometry(new_vertices, &new_indices);
    // }

    fn generate_quad_vertices(&self) -> [I; 6] {
        let v_base = self.vertex_cache.len();

        [
            I::from_usize(v_base + 0), I::from_usize(v_base + 1), I::from_usize(v_base + 2),
            I::from_usize(v_base + 2), I::from_usize(v_base + 3), I::from_usize(v_base + 0)
        ]
    }

    pub fn get<S>(&self, index: S) -> Option<&<S as SliceIndex<[V]>>::Output>
    where
        S: SliceIndex<[V]>
    {
        self.vertex_cache.get(index)
    }

    pub fn get_mut<S>(&mut self, index: S) -> Option<&mut <S as SliceIndex<[V]>>::Output>
    where
        S: SliceIndex<[V]>
    {
        self.vertex_cache.get_mut(index)
    }
}

impl<V: Vertex, I: Index> Buffer for GeometryBuffer<V, I> {
    fn reset(&mut self) {
         #[cfg(debug_assertions)] {
            if self.pending_writes {
                log::warn!("Clearing buffer with pending writes.");
            }
        }

        self.index_cache.clear();
        self.vertex_cache.clear();

        self.over_capacity = false;
        self.pending_writes = false;
    }

    fn vertex_count(&self) -> usize {
        self.vertex_cache.len()
    }

    fn vertex_capacity(&self) -> usize {
        self.vertex_capacity
    }

    fn remaining_vertex_capacity(&self) -> usize {
        self.vertex_capacity() - self.vertex_count()
    }

    fn index_count(&self) -> usize {
        self.index_cache.len()
    }

    fn index_capacity(&self) -> usize {
        self.index_capacity
    }

    fn remaining_index_capacity(&self) -> usize {
        self.index_capacity() - self.index_count()
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
        #[cfg(debug_assertions)] {
            if buffer.pending_writes {
                log::warn!("Drawing buffer with pending writes.");
            }
        }

        self.set_vertex_buffer(0, buffer.vertex_buffer.slice(..));
        self.set_index_buffer(buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..buffer.index_cache.len() as u32, 0, 0..1);
    }
}
