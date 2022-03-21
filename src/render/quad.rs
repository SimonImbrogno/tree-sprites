use super::vertex::{TexturedVertex, UvVertex, ColoredVertex};

pub struct TexturedQuad {
    pub pos: (f32, f32),
    pub dim: (f32, f32),
    pub tex_index: i32,
}

impl From<TexturedQuad> for [TexturedVertex; 4] {
    fn from(src: TexturedQuad) -> Self {
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

pub struct TexturedUvQuad {
    pub pos: (f32, f32),
    pub dim: (f32, f32),
    pub uv_min: (f32, f32),
    pub uv_max: (f32, f32),
    pub tex_index: i32,
}

impl From<TexturedUvQuad> for [TexturedVertex; 4] {
    fn from(src: TexturedUvQuad) -> Self {
        let x_min = src.pos.0;
        let y_min = src.pos.1;
        let x_max = src.pos.0 + src.dim.0;
        let y_max = src.pos.1 + src.dim.1;

        [
            TexturedVertex { position: [x_max, y_max, 0.0], tex_coords: [src.uv_max.0, src.uv_min.1], tex_index: src.tex_index },
            TexturedVertex { position: [x_min, y_max, 0.0], tex_coords: [src.uv_min.0, src.uv_min.1], tex_index: src.tex_index },
            TexturedVertex { position: [x_min, y_min, 0.0], tex_coords: [src.uv_min.0, src.uv_max.1], tex_index: src.tex_index },
            TexturedVertex { position: [x_max, y_min, 0.0], tex_coords: [src.uv_max.0, src.uv_max.1], tex_index: src.tex_index },
        ]
    }
}

pub struct UntexturedQuad {
    pub pos: (f32, f32),
    pub dim: (f32, f32),
}

impl From<UntexturedQuad> for [UvVertex; 4] {
    fn from(src: UntexturedQuad) -> Self {
        let x_min = src.pos.0;
        let y_min = src.pos.1;
        let x_max = src.pos.0 + src.dim.0;
        let y_max = src.pos.1 + src.dim.1;

        [
            UvVertex { position: [x_max, y_max, 0.0], uv: [1.0, 0.0] },
            UvVertex { position: [x_min, y_max, 0.0], uv: [0.0, 0.0] },
            UvVertex { position: [x_min, y_min, 0.0], uv: [0.0, 1.0] },
            UvVertex { position: [x_max, y_min, 0.0], uv: [1.0, 1.0] },
        ]
    }
}

pub struct ColoredQuad {
    pub pos: (f32, f32),
    pub dim: (f32, f32),
    pub color: (f32, f32, f32, f32),
}

impl From<ColoredQuad> for [ColoredVertex; 4] {
    fn from(src: ColoredQuad) -> Self {
        let x_min = src.pos.0;
        let y_min = src.pos.1;
        let x_max = src.pos.0 + src.dim.0;
        let y_max = src.pos.1 + src.dim.1;

        [
            ColoredVertex { position: [x_max, y_max, 0.0], color: [src.color.0, src.color.1, src.color.2, src.color.3] },
            ColoredVertex { position: [x_min, y_max, 0.0], color: [src.color.0, src.color.1, src.color.2, src.color.3] },
            ColoredVertex { position: [x_min, y_min, 0.0], color: [src.color.0, src.color.1, src.color.2, src.color.3] },
            ColoredVertex { position: [x_max, y_min, 0.0], color: [src.color.0, src.color.1, src.color.2, src.color.3] },
        ]
    }
}
