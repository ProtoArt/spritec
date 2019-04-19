//! Re-exports all of rayon::prelude and provides replacements (i.e. "polyfills") that dispatch to
//! normal Iterators when rayon isn't supported.

#[cfg(not(target_arch = "wasm32"))]
pub use rayon::prelude::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

/// Polyfill for rayon traits to add support for wasm32-unknown-unknown
///
/// Basically implements rayon traits to use regular Iterators instead of parallel iterators.
#[cfg(target_arch = "wasm32")]
mod wasm {
    pub trait IntoParallelIterator {
        type Item;
        type Iter: Iterator<Item = Self::Item>;

        fn into_par_iter(self) -> Self::Iter;
    }

    impl<T: IntoIterator> IntoParallelIterator for T {
        type Item = <T as IntoIterator>::Item;
        type Iter = <T as IntoIterator>::IntoIter;

        fn into_par_iter(self) -> Self::Iter {
            self.into_iter()
        }
    }
}
