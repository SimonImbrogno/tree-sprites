use std::ops::Add;

mod geometry_buffer;
mod quad_buffer;

pub use geometry_buffer::GeometryBuffer;
pub use quad_buffer::QuadBuffer;
use super::vertex::Vertex;

pub trait Buffer {
    fn reset(&mut self);

    fn vertex_count(&self) -> usize;
    fn vertex_capacity(&self) -> usize;
    fn remaining_vertex_capacity(&self) -> usize;

    fn index_count(&self) -> usize;
    fn index_capacity(&self) -> usize;
    fn remaining_index_capacity(&self) -> usize;
}

// TODO:
//  is there some way to abstract over SliceIndex implementors to correctly adjust stride? :/
//  i.e: A quad buffer indexes elements in 4 vertex chunks but the underlying buffer must be indexed by Vertex.
pub trait ViewableBuffer<V: Vertex, I: Index> {
    fn get(&self, index: usize) -> Option<&[V]>;
    fn get_mut(&mut self, index: usize) -> Option<&mut [V]>;
}

pub trait WriteGeometryBuffer<B, V, I>
where
    B: Buffer,
    V: Vertex,
    I: Index
{
    fn write_geometry_buffer(&self, buffer: &mut B);
}

pub trait DrawGeometryBuffer<'b, B, V, I>
where
    B: Buffer,
    V: Vertex,
    I: Index
{
    fn draw_geometry_buffer(&mut self, buffer: &'b B);
}

pub trait Index: bytemuck::Pod + Add {
    fn from_usize(src: usize) -> Self;
    fn as_usize(self) -> usize;
}

impl Index for u16 {
    fn from_usize(src: usize) -> Self { src as Self }
    fn as_usize(self) -> usize { self as usize }
}

impl Index for u32 {
    fn from_usize(src: usize) -> Self { src as Self }
    fn as_usize(self) -> usize { self as usize }
}
