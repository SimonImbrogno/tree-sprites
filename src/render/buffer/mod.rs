use std::ops::Add;

mod geometry_buffer;
mod quad_buffer;

pub use geometry_buffer::GeometryBuffer;
pub use quad_buffer::QuadBuffer;
use super::vertex::Vertex;

pub trait Buffer<V: Vertex, I: Index> { }

pub trait WriteGeometryBuffer<B, V, I>
where
    B: Buffer<V, I>,
    V: Vertex,
    I: Index
{
    fn write_geometry_buffer(&self, buffer: &mut B);
}

pub trait DrawGeometryBuffer<'b, B, V, I>
where
    B: Buffer<V, I>,
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
