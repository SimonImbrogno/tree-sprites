use super::super::buffer::Buffer;

pub struct BufferUsageMeter {
    pub vertex_capacity: usize,
    pub vertex_usage: usize,
    pub index_capacity: usize,
    pub index_usage: usize,
}

impl<T> From<&T> for BufferUsageMeter
where
    T: Buffer
{
    fn from(src: &T) -> Self {
        Self {
            vertex_capacity: src.vertex_capacity(),
            vertex_usage: src.vertex_count(),
            index_capacity: src.index_capacity(),
            index_usage: src.index_count(),
        }
    }
}
