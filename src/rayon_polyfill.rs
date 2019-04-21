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

    pub struct IterBridge<T>(T);

    impl<T: Iterator> Iterator for IterBridge<T> {
        type Item = <T as Iterator>::Item;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }
    }

    pub trait ParallelBridge: Sized {
        fn par_bridge(self) -> IterBridge<Self>;
    }

    impl<T: Iterator + Send> ParallelBridge for T {
        fn par_bridge(self) -> IterBridge<Self> {
            IterBridge(self)
        }
    }

    pub trait ParallelExtend<T> {
        fn par_extend<I: IntoParallelIterator<Item = T>>(&mut self, par_iter: I);
    }

    impl<T> ParallelExtend<T> for Vec<T> {
        fn par_extend<I: IntoParallelIterator<Item = T>>(&mut self, par_iter: I) {
            let par_iter = par_iter.into_par_iter();
            self.extend(par_iter);
        }
    }
}
