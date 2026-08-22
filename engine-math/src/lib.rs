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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
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

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> AABB {
        AABB { min, max }
    }

    pub fn from_center_size(center: Vec2, size: Vec2) -> AABB {
        let half_size = size * 0.5;
        AABB {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn half_size(&self) -> Vec2 {
        self.size() * 0.5
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn transform(&self, transform: &Transform2D) -> AABB {
        let corners = [
            Vec2::new(self.min.x, self.min.y),
            Vec2::new(self.max.x, self.min.y),
            Vec2::new(self.max.x, self.max.y),
            Vec2::new(self.min.x, self.max.y),
        ];

        let cos = transform.rotation_in_radians.cos();
        let sin = transform.rotation_in_radians.sin();

        let mut world_min = Vec2::splat(f32::INFINITY);
        let mut world_max = Vec2::splat(f32::NEG_INFINITY);

        for corner in corners {
            let scaled = corner * transform.scale;

            let rotated = Vec2::new(
                scaled.x * cos - scaled.y * sin,
                scaled.x * sin + scaled.y * cos,
            );
            let world_pos = rotated + transform.position;

            world_min = world_min.min(world_pos);
            world_max = world_max.max(world_pos);
        }

        AABB {
            min: world_min,
            max: world_max,
        }
    }
}
