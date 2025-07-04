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
    transform: Transform,
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

    pub fn get_scale(&self) -> Vec2 {
        self.transform.scale.truncate()
    }

    pub fn get_translation(&self) -> Vec2 {
        self.transform.translation.truncate()
    }

    pub fn set_translation(&mut self, translation: Vec2) {
        self.transform.translation.x = translation.x;
        self.transform.translation.y = translation.y;
    }

    pub fn add_translation(&mut self, translation: Vec2) {
        self.transform.translation.x += translation.x;
        self.transform.translation.y += translation.y;
    }

    pub fn if_can_zoom_in(&self) -> bool {
        self.pixel_zoom < 8
    }

    pub fn zoom_in(&mut self) -> bool {
        if self.pixel_zoom < 8 {
            if self.scale_animator.if_finished() {
                // NOTE 确保在可以执行动画的情况下再修改 pixel_zoom
                self.pixel_zoom *= 2;
                self.scale_animator = EasingAnimator::new(
                    self.transform.scale.x,
                    1.0 / self.pixel_zoom as f32,
                    Duration::from_millis(500),
                );
                return true;
            }
        }
        false
    }

    pub fn if_can_zoom_out(&self) -> bool {
        self.pixel_zoom > 1
    }

    pub fn zoom_out(&mut self) -> bool {
        if self.pixel_zoom > 1 {
            if self.scale_animator.if_finished() {
                self.pixel_zoom /= 2;
                self.scale_animator = EasingAnimator::new(
                    self.transform.scale.x,
                    1.0 / self.pixel_zoom as f32,
                    Duration::from_millis(500),
                );
                return true;
            }
        }
        false
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

    /// 限制镜头展示的世界空间范围，需要配合 transform.scale 来判断镜头实际上可以显示的范围，
    /// 并根据 viewport_size 来判断镜头的 transform.translation 范围
    pub fn update_word_boundary(&mut self, word_boundary: Option<Rect>) {
        if let Some(boundary) = word_boundary {
            let size = self.viewport_size * self.transform.scale.truncate();
            let viewport = Rect::from_center_size(self.transform.translation.truncate(), size);

            // 只有镜头尺寸比边界限制尺寸比要小的情况下，才需要保持镜头在边界尺寸范围内
            if viewport.size().x < boundary.size().x {
                if viewport.left() < boundary.left() {
                    self.transform.translation.x += boundary.left() - viewport.left();
                } else if viewport.right() > boundary.right() {
                    self.transform.translation.x -= viewport.right() - boundary.right();
                }
            } else {
                self.transform.translation.x = boundary.center().x;
            }

            if viewport.size().y < boundary.size().y {
                if viewport.bottom() < boundary.bottom() {
                    self.transform.translation.y += boundary.bottom() - viewport.bottom();
                } else if viewport.top() > boundary.top() {
                    self.transform.translation.y -= viewport.top() - boundary.top();
                }
            } else {
                self.transform.translation.y = boundary.center().y;
            }
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
