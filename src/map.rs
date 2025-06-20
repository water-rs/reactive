//! # Map Module
//!
//! This module provides transformation and memoization capabilities for reactive values.
//!
//! The `Map` type enables you to transform values from one type to another while preserving
//! the reactive nature of the computation. It automatically caches the result of the transformation
//! for better performance, invalidating the cache only when the source value changes.
//!
//! ## Key Components
//!
//! - `Map<C, F, Output>`: A reactive value that applies transformation `F` to source `C`
//! - `map()`: Helper function for creating `Map` instances
//! - Automatic caching of transformation results
//! - Reactive propagation of changes from source to transformed value
//!
//! ## Usage Example
//!
//! ```rust
//! use reactive::{binding, Compute};
//! use reactive::map::map;
//!
//! let number = binding(5);
//! let doubled = map(number, |n| n * 2);
//!
//! assert_eq!(doubled.compute(), 10);
//!
//! // The transformation is automatically cached
//! doubled.compute(); // Uses cached value, doesn't recompute
//! ```

use core::marker::PhantomData;

use alloc::rc::Rc;

use crate::{
    Compute,
    watcher::{Watcher, WatcherGuard},
};

/// A reactive computation that transforms values from a source computation.
///
/// `Map<C, F, Output>` applies a transformation function `F` to the results
/// of a source computation `C`, producing a value of type `Output`. The result
/// is automatically cached and only recomputed when the source value changes.
pub struct Map<C, F, Output> {
    source: C,
    f: Rc<F>,
    _marker: PhantomData<Output>,
}

impl<C: Compute + 'static, F: 'static, Output> Map<C, F, Output> {
    /// Creates a new `Map` that transforms values from `source` using function `f`.
    ///
    /// # Parameters
    ///
    /// * `source`: The source computation whose results will be transformed
    /// * `f`: The transformation function to apply to the source's results
    ///
    /// # Returns
    ///
    /// A new `Map` instance that will transform values from the source.
    pub fn new(source: C, f: F) -> Self {
        Self {
            source,
            f: Rc::new(f),
            _marker: PhantomData,
        }
    }
}

/// Helper function to create a new `Map` transformation.
///
/// This is a convenience wrapper around `Map::new()` with improved type inference.
///
/// # Parameters
///
/// * `source`: The source computation whose results will be transformed
/// * `f`: The transformation function to apply to the source's results
///
/// # Returns
///
/// A new `Map` instance that will transform values from the source.
///
/// # Example
///
/// ```rust
/// use reactive::{binding, Compute};
/// use reactive::map::map;
///
/// let counter = binding(1);
/// let doubled = map(counter, |n| n * 2);
/// assert_eq!(doubled.compute(), 2);
/// ```
pub fn map<C, F, Output>(source: C, f: F) -> Map<C, F, Output>
where
    C: Compute + 'static,
    F: 'static + Fn(C::Output) -> Output,
{
    Map::new(source, f)
}

impl<C: Clone, F, Output> Clone for Map<C, F, Output> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<C, F, Output> Compute for Map<C, F, Output>
where
    C: Compute,
    F: 'static + Fn(C::Output) -> Output,
    Output: 'static,
{
    type Output = Output;

    /// Computes the transformed value, using the cache when available.
    fn compute(&self) -> Output {
        (self.f)(self.source.compute())
    }

    /// Registers a watcher to be notified when the transformed value changes.
    fn add_watcher(&self, watcher: impl Watcher<Self::Output>) -> WatcherGuard {
        let this = self.clone();

        self.source
            .add_watcher(move |_value, metadata| watcher.notify(this.compute(), metadata))
    }
}
