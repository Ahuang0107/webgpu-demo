use crate::{EasingAnimator, Rect, Transform};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::time::Duration;

#[derive(Debug, Default)]
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
    /// 目前来看应该始终保持 [0.5, 0.5] 不会有其他情况
    pub viewport_origin: Vec2,
    pub viewport_size: Vec2,
    pub transform: Transform,
    /// sprite 的 z 越大，表示约 near（靠近镜头）
    pub near: f32,
    pub far: f32,
    /// 因为镜头缩放动画需要完美像素需要的缩放必须是整数，但是缩放过程中一定是有小数的
    /// 所有缩放需要 pixel_zoom 和 scale_animator 和 transform.scale 共同作用
    ///
    /// 限制放大倍率只有 x1 x2 x4 x8
    pub pixel_zoom: u8,
    pub scale_animator: EasingAnimator,
}

impl Camera2D {
    #[inline(always)]
    pub fn new(viewport_size: Vec2) -> Camera2D {
        Camera2D {
            viewport_origin: Vec2::splat(0.5),
            viewport_size,
            transform: Transform::IDENTITY,
            near: -2000.0,
            far: 1.0,
            pixel_zoom: 1,
            scale_animator: EasingAnimator::default(),
        }
    }

    pub fn zoom_in(&mut self) {
        if self.pixel_zoom < 8 {
            if self.scale_animator.if_finished() {
                // NOTE 确保在可以执行动画的情况下再修改 pixel_zoom
                self.pixel_zoom *= 2;
                self.scale_animator = EasingAnimator::new(
                    self.transform.scale.x,
                    1.0 / self.pixel_zoom as f32,
                    Duration::from_millis(500),
                );
            }
        }
    }

    pub fn zoom_out(&mut self) {
        if self.pixel_zoom > 1 {
            if self.scale_animator.if_finished() {
                self.pixel_zoom /= 2;
                self.scale_animator = EasingAnimator::new(
                    self.transform.scale.x,
                    1.0 / self.pixel_zoom as f32,
                    Duration::from_millis(500),
                );
            }
        }
    }

    pub fn set_zoom(&mut self, zoom: u8) {
        self.pixel_zoom = zoom;
        self.transform.scale.x = 1.0 / self.pixel_zoom as f32;
        self.transform.scale.y = 1.0 / self.pixel_zoom as f32;
    }

    pub fn update_anima(&mut self, delta: Duration) {
        if !self.scale_animator.if_finished() {
            let result = self.scale_animator.update(delta);
            // NOTE 这里不能修改 z 的 scale 因为这会影响到 near 和 far
            //  这在 3D 游戏中是需要逻辑但是 2D 不需要
            self.transform.scale.x = result;
            self.transform.scale.y = result;
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
    pub fn get_viewport(&self) -> Vec4 {
        Vec4::new(
            self.viewport_origin.x,
            self.viewport_origin.y,
            self.viewport_size.x,
            self.viewport_size.y,
        )
    }

    #[inline(always)]
    pub fn get_view_uniform(&self) -> ViewUniform {
        ViewUniform {
            clip_from_world: self.get_clip_from_world(),
            viewport: self.get_viewport(),
        }
    }

    pub fn viewport_to_world(&self, mut viewport_position: Vec2) -> Vec3 {
        let target_size = self.viewport_size;
        // Flip the Y co-ordinate origin from the top to the bottom.
        viewport_position.y = target_size.y - viewport_position.y;
        let ndc = viewport_position * 2. / target_size - Vec2::ONE;

        let ndc_to_world = self.transform.compute_matrix() * self.get_clip_from_view().inverse();
        ndc_to_world.project_point3(ndc.extend(1.))
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewUniform {
    clip_from_world: Mat4,
    /// viewport_origin(default = `[0.5, 0.5]`) + viewport_size(default = window_size)
    viewport: Vec4,
}
