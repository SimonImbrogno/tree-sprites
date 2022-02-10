use super::vertex::TexturedVertex;

pub struct Quad {
    pub pos: (f32, f32),
    pub dim: (f32, f32),
    pub tex_index: i32,
}

impl From<Quad> for [TexturedVertex; 4] {
    fn from(src: Quad) -> Self {
        let x_min = src.pos.0;
        let y_min = src.pos.1;
        let x_max = src.pos.0 + src.dim.0;
        let y_max = src.pos.1 + src.dim.1;

        [
            TexturedVertex { position: [x_max, y_max, 0.0], tex_coords: [1.0, 0.0], tex_index: src.tex_index },
            TexturedVertex { position: [x_min, y_max, 0.0], tex_coords: [0.0, 0.0], tex_index: src.tex_index },
            TexturedVertex { position: [x_min, y_min, 0.0], tex_coords: [0.0, 1.0], tex_index: src.tex_index },
            TexturedVertex { position: [x_max, y_min, 0.0], tex_coords: [1.0, 1.0], tex_index: src.tex_index },
        ]
    }
}
