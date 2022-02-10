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
            let near   = -1.0;
            let far    =  1.0;

            cgmath::ortho(left, right, bottom, top, near, far)
        };

        Self {
            position: src.position.to_homogeneous().into(),
            view_proj: vp_matrix.into(),
        }
    }
}
