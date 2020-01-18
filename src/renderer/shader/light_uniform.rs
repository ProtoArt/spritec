use glium::uniforms::{Uniforms, UniformValue};

use crate::math::{Mat4, Vec3, Decompose, Transforms};
use crate::renderer::Light;

/// This struct must match the `Light` struct in our shaders
pub struct LightUniform {
    position: UniformValue<'static>,
    color: UniformValue<'static>,
    range: UniformValue<'static>,
    cone_direction: UniformValue<'static>,
    light_angle_scale: UniformValue<'static>,
    light_angle_offset: UniformValue<'static>,
}

impl Uniforms for LightUniform {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        let &Self {
            position,
            color,
            range,
            cone_direction,
            light_angle_scale,
            light_angle_offset,
        } = self;

        visit("position", position);
        visit("color", color);
        visit("range", range);
        visit("cone_direction", cone_direction);
        visit("light_angle_scale", light_angle_scale);
        visit("light_angle_offset", light_angle_offset);
    }
}

impl LightUniform {
    pub fn new(light: &Light, world_transform: Mat4) -> Self {
        // scale is ignored by all the different light types
        let Transforms {scale: _, rotation, translation: pos} = world_transform.decompose();
        let direction = rotation * Vec3 {x: 0.0, y: 0.0, z: -1.0};

        use Light::*;
        match light {
            Point {color, intensity, range} => Self {
                position: UniformValue::Vec4([pos.x, pos.y, pos.z, 1.0]),
                color: UniformValue::Vec3((color * intensity).into_array()),
                range: UniformValue::Float(range.unwrap_or(0.0)),
                cone_direction: UniformValue::Vec3([0.0; 3]),
                light_angle_scale: UniformValue::Float(0.0),
                light_angle_offset: UniformValue::Float(0.0),
            },

            Directional {color, intensity} => Self {
                position: UniformValue::Vec4([direction.x, direction.y, direction.z, 0.0]),
                color: UniformValue::Vec3((color * intensity).into_array()),
                range: UniformValue::Float(0.0),
                cone_direction: UniformValue::Vec3([0.0; 3]),
                light_angle_scale: UniformValue::Float(0.0),
                light_angle_offset: UniformValue::Float(0.0),
            },

            Spot {color, intensity, range, inner_cone_angle, outer_cone_angle} => {
                // cos() expects a value in radians
                let inner_cone_angle = inner_cone_angle.get_radians();
                let outer_cone_angle = outer_cone_angle.get_radians();

                let light_angle_scale = 1.0 / 0.001f32.max(inner_cone_angle.cos() - outer_cone_angle.cos());
                let light_angle_offset = -outer_cone_angle.cos() * light_angle_scale;

                Self {
                    position: UniformValue::Vec4([pos.x, pos.y, pos.z, 1.0]),
                    color: UniformValue::Vec3((color * intensity).into_array()),
                    range: UniformValue::Float(range.unwrap_or(0.0)),
                    cone_direction: UniformValue::Vec3([direction.x, direction.y, direction.z]),
                    light_angle_scale: UniformValue::Float(light_angle_scale),
                    light_angle_offset: UniformValue::Float(light_angle_offset),
                }
            },
        }
    }
}
