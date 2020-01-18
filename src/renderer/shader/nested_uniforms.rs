use glium::uniforms::{Uniforms, UniformValue};

/// Nests a set of uniforms under a prefix followed by '.'
///
/// Used to create normals with names like `light.color` or `lights[1].color`
pub trait NestedUniforms: Uniforms {
    fn visit_nested<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, prefix: &str, visit: F);
    fn visit_nested_index<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, prefix: &str, index: usize, visit: F);
}

impl<T: Uniforms> NestedUniforms for T {
    fn visit_nested<'a, F: FnMut(&str, UniformValue<'a>)>(
        &'a self,
        prefix: &str,
        mut visit: F,
    ) {
        self.visit_values(|name, value| {
            let name = format!("{}.{}", prefix, name);
            visit(&name, value);
        });
    }

    fn visit_nested_index<'a, F: FnMut(&str, UniformValue<'a>)>(
        &'a self,
        prefix: &str,
        index: usize,
        visit: F,
    ) {
        let prefix_index = format!("{}[{}]", prefix, index);
        self.visit_nested(&prefix_index, visit);
    }
}
