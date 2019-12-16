use glium::uniforms::{Uniforms, UniformValue};

/// Nests a set of uniforms under a prefix followed by '.'
///
/// Used to create normals with names like `light.color`.
pub struct NestedUniforms<T: Uniforms> {
    value: T,
}

impl<T: Uniforms> NestedUniforms<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
        }
    }

    pub fn visit_nested<'a, F: FnMut(&str, UniformValue<'a>)>(
        &'a self,
        prefix: &str,
        mut visit: F,
    ) {
        let Self {value} = self;
        value.visit_values(|name, value| {
            let name = format!("{}.{}", prefix, name);
            visit(&name, value);
        });
    }
}
