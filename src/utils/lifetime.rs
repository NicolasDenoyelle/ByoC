use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Structure to Attach the appropriate lifetime to objects returned by
/// [`Get`](../../trait.Get.html) and [`GetMut`](../../trait.GetMut.html) traits.
///
/// [`LifeTimeGuard`] can be dereferenced through the pointer it wraps to
/// return a reference to the pointed value. This allow containers implementation
/// to return their own smart pointer with [`Get`](../../trait.Get.html) trait
/// with a mandatory lifetime attached while allowing the user to access the
/// pointed value with only one call to `deref()`/`deref_mut()`.
/// See [`Get`](../../trait.Get.html) and [`GetMut`](../../trait.GetMut.html)
/// traits documentation to see how it is used.
#[derive(Debug)]
pub struct LifeTimeGuard<'a, T> {
    value: T,
    lifetime: PhantomData<&'a T>,
}

impl<'a, T> LifeTimeGuard<'a, T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            lifetime: PhantomData,
        }
    }

    pub fn map<O, F: FnOnce(T) -> O>(self, f: F) -> LifeTimeGuard<'a, O> {
        LifeTimeGuard {
            value: f(self.value),
            lifetime: PhantomData,
        }
    }

    pub(crate) fn unwrap(self) -> T {
        self.value
    }
}

impl<'a, U, T: Deref<Target = U>> Deref for LifeTimeGuard<'a, T> {
    type Target = U;
    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<'a, U, T: Deref<Target = U> + DerefMut> DerefMut
    for LifeTimeGuard<'a, T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}
