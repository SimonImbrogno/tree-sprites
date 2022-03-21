use crate::game::game_state::GameCamera;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub aspect_ratio: f32,
    pub position: cgmath::Point3<f32>,
    pub y_axis_dim: f32,
}

impl Camera {
    pub fn update(&mut self, src: &GameCamera, physical_size: winit::dpi::PhysicalSize<u32>) {
        self.aspect_ratio = physical_size.width as f32 / physical_size.height as f32;
        self.position = src.position;
        self.y_axis_dim = src.zoom_level;
    }
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub position: [f32; 4],
    pub view_proj: [[f32; 4]; 4]
}

impl From<Camera> for CameraUniform {
    fn from(src: Camera) -> Self {
        let y_radius = src.y_axis_dim / 2.0;

        let vp_matrix = {
            let left   = -y_radius * src.aspect_ratio + src.position.x;
            let right  =  y_radius * src.aspect_ratio + src.position.x;
            let bottom = -y_radius + src.position.y;
            let top    =  y_radius + src.position.y;
            let near   =  0.0;
            let far    =  1.0;

            ortho_matrix(left, right, bottom, top, near, far)
        };

        Self {
            position: src.position.to_homogeneous().into(),
            view_proj: vp_matrix.into(),
        }
    }
}

impl CameraUniform {
    pub fn simple_canvas_ortho(x: u32, y: u32) -> Self {
        let vp_matrix = {
            let left   = 0.0;
            let right  = x as f32;
            let bottom = 0.0;
            let top    = y as f32;
            let near   = 0.0;
            let far    = 1.0;

            ortho_matrix(left, right, bottom, top, near, far)
        };

        let position = cgmath::Point3::new(0.0, 0.0, 0.0).to_homogeneous();

        Self {
            position: position.into(),
            view_proj: vp_matrix.into(),
        }
    }

    pub fn simple_top_down_canvas_ortho(x: u32, y: u32) -> Self {
        let vp_matrix = {
            let left   = 0.0;
            let right  = x as f32;
            let bottom = y as f32;
            let top    = 0.0;
            let near   = 0.0;
            let far    = 1.0;

            ortho_matrix(left, right, bottom, top, near, far)
        };

        let position = cgmath::Point3::new(0.0, 0.0, 0.0).to_homogeneous();

        Self {
            position: position.into(),
            view_proj: vp_matrix.into(),
        }
    }
}

fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> cgmath::Matrix4<f32> {
    // _LEFT HANDED_ ortho matrix: (2.0 / (far - near) has been negated vs right handed ortho.
    let ortho = cgmath::Matrix4::new(
        2.0 / (right - left),             0.0,                              0.0,                          0.0,
        0.0,                              2.0 / (top - bottom),             0.0,                          0.0,
        0.0,                              0.0,                              2.0 / (far - near),           0.0,
        -(right + left) / (right - left), -(top + bottom) / (top - bottom), -(far + near) / (far - near), 1.0,
    );

    OPENGL_TO_WGPU_MATRIX * ortho
}
