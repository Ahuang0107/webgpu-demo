use crate::{Rect, Transform};
use glam::{Mat4, Vec2, Vec4};

pub struct Camera2D {
    /// Specifies the origin of the viewport as a normalized position from 0 to 1, where (0, 0) is the bottom left
    /// and (1, 1) is the top right. This determines where the camera's position sits inside the viewport.
    ///
    /// When the projection scales due to viewport resizing, the position of the camera, and thereby `viewport_origin`,
    /// remains at the same relative point.
    ///
    /// Consequently, this is pivot point when scaling. With a bottom left pivot, the projection will expand
    /// upwards and to the right. With a top right pivot, the projection will expand downwards and to the left.
    /// Values in between will caused the projection to scale proportionally on each axis.
    ///
    /// Defaults to `(0.5, 0.5)`, which makes scaling affect opposite sides equally, keeping the center
    /// point of the viewport centered.
    ///
    /// 来自 bevy 的 OrthographicProjection 中的 viewport_origin
    pub viewport_origin: Vec2,
    pub viewport_size: Vec2,
    pub transform: Transform,
    /// sprite 的 z 越大，表示约 near（靠近镜头）
    pub near: f32,
    pub far: f32,
}

impl Camera2D {
    #[inline(always)]
    pub fn new(viewport_size: Vec2) -> Camera2D {
        Camera2D {
            viewport_origin: Vec2::splat(0.5),
            viewport_size,
            transform: Transform::IDENTITY,
            near: -1000.0,
            far: 1.0,
        }
    }

    #[inline(always)]
    fn area(&self) -> Rect {
        let origin_x = self.viewport_size.x * self.viewport_origin.x;
        let origin_y = self.viewport_size.y * self.viewport_origin.y;

        Rect::new(
            -origin_x,
            -origin_y,
            self.viewport_size.x - origin_x,
            self.viewport_size.y - origin_y,
        )
    }

    #[inline(always)]
    fn get_clip_from_view(&self) -> Mat4 {
        let area = self.area();
        Mat4::orthographic_rh(
            area.min.x, area.max.x, area.min.y, area.max.y, self.near, self.far,
        )
    }

    #[inline(always)]
    fn get_world_from_view(&self) -> Mat4 {
        self.transform.compute_matrix()
    }

    #[inline(always)]
    fn get_view_from_world(&self) -> Mat4 {
        self.get_world_from_view().inverse()
    }

    #[inline(always)]
    fn get_clip_from_world(&self) -> Mat4 {
        self.get_clip_from_view() * self.get_view_from_world()
    }

    #[inline(always)]
    pub fn get_view_uniform(&self) -> ViewUniform {
        ViewUniform {
            clip_from_world: self.get_clip_from_world(),
            viewport: Vec4::new(
                self.viewport_origin.x,
                self.viewport_origin.y,
                self.viewport_size.x,
                self.viewport_size.y,
            ),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniform {
    clip_from_world: Mat4,
    /// viewport_origin(default = `[0.5, 0.5]`) + viewport_size(default = window_size)
    viewport: Vec4,
}
