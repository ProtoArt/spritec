//! The `UniformsStorage` struct provided by glium is difficult to use in an extensible way because
//! it requires all of the types of every uniform to be encoded in its type. This makes a sort of
//! "linked list" of types, which is difficult to use in an actual program because to name the type
//! you essentially need to write out the entire list. It's also limited to storing string slices
//! (&str), so it's hard to compute uniform names at runtime.
//!
//! This module provides `UniformMap` which is a simple map from String -> UniformValue. This is
//! sufficient to implement the `Uniforms` trait and does not suffer from the issues we were facing
//! with `UniformsStorage`. This type also supports doing things like merging/nesting maps.

use std::collections::HashMap;

use glium::uniforms::{Uniforms, AsUniformValue, UniformValue};

#[derive(Default, Clone)]
pub struct UniformMap {
    // This field was made 'static because we currently don't need any other lifetime
    uniforms: HashMap<String, UniformValue<'static>>,
}

impl Uniforms for UniformMap {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, visit: F) {
        for (name, value) in &self.uniforms {
            visit(name, *value);
        }
    }
}

impl UniformMap {
    /// Inserts a new uniform value into this map, overwriting any previous value for the same name
    pub fn insert<S, U>(&mut self, name: S, value: U)
        where S: Into<String>,
              U: AsUniformValue + 'static,
    {
        self.uniforms.insert(name.into(), value.as_uniform_value());
    }

    /// Inserts another uniform map into this map, nesting all of the names under the given prefix
    ///
    /// Example: if prefix is `foo` and `other` has a property `bar`, `foo.bar` will be inserted
    /// into this map.
    pub fn insert_nested(&mut self, prefix: &str, other: UniformMap) {
        let prefix = format!("{}.", prefix);
        for (name, value) in other.uniforms.into_iter() {
            name.insert_str(0, &prefix);
            self.uniforms.insert(name, value);
        }
    }
}
