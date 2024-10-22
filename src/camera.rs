pub struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    width: u32,
    height: u32,
    pub use_grab: bool,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            eye: (0.0, 0.0, 2.0).into(),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            width,
            height,
            use_grab: false,
        }
    }
    pub fn build_view_projection_matrix(&self) -> CameraUniform {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj =
            glam::Mat4::perspective_rh(45.0, self.width as f32 / self.height as f32, 0.1, 100.0);

        return CameraUniform::new(proj * view, self.width, self.height, self.use_grab);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    size: [f32; 4],
}

impl CameraUniform {
    pub fn new(v: glam::Mat4, width: u32, height: u32, use_grab: bool) -> Self {
        Self {
            view_proj: v.to_cols_array_2d(),
            size: [
                width as f32,
                height as f32,
                if use_grab { 1.0 } else { 0.0 },
                0.0,
            ],
        }
    }
}
