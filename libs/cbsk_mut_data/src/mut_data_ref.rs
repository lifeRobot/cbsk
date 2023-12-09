use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// support ref to mut data<br />
/// more see [NonNull]
pub struct MutDataRef<T> {
    /// mut data
    data: NonNull<T>,
}

/// custom method
impl<T> MutDataRef<T> {
    /// create MutDataRef by *mut data<br />
    /// more see [NonNull::new_unchecked]
    pub fn new(data: *mut T) -> Self {
        Self {
            data: unsafe { NonNull::new_unchecked(data) }
        }
    }

    /// create MutDataRef by &mut data
    pub fn new_ref(data: &mut T) -> Self {
        Self::new(data)
    }
}

/// deconstruct multiple data
impl<Q, T: DerefMut<Target=Q>> MutDataRef<T> {
    /// deconstruct multiple data<br />
    /// if your data is [`MutDataRef<MutDataRef<T>>`], you can call this method to convert it to [`MutDataRef<T>`]
    pub fn to_short(&mut self) -> MutDataRef<Q> {
        MutDataRef::new(self.deref_mut().deref_mut())
    }
}

crate::impl_sync_send!(T,MutDataRef<T>);
crate::impl_debug_display!(T,MutDataRef<T>);
crate::impl_as_ref!(T,T,MutDataRef<T>);

/// support clone
impl<T: Clone> Clone for MutDataRef<T> {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}

/// support deref
impl<T> Deref for MutDataRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref() }
    }
}

/// support derefmut
impl<T> DerefMut for MutDataRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.data.as_mut() }
    }
}
