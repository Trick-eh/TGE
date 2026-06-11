pub use glam::*;

pub struct Transform2D {
    pub position: Vec2,
    pub rotation_in_radians: f32,
    pub scale: Vec2,
}

pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
}

impl Transform2D {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position.extend(0.0))
            * Mat4::from_rotation_z(self.rotation_in_radians)
            * Mat4::from_scale(self.scale.extend(1.0))
    }
}

impl Camera2D {
    pub fn projection_matrix(&self, width: f32, height: f32) -> Mat4 {
        let half_w = (width / 2.0) / self.zoom;
        let half_h = (height / 2.0) / self.zoom;
        let projection = Mat4::orthographic_rh_gl(-half_w, half_w, -half_h, half_h, -1.0, 1.0);
        let view = Mat4::from_translation((-self.position).extend(0.0));

        projection * view
    }
}
