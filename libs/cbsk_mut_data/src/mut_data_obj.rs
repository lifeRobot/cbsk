use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use crate::mut_data_ref::MutDataRef;

/// ref mut object
pub struct MutDataObj<T> {
    /// mut data
    data: UnsafeCell<T>,
}

/// custom method
impl<T> MutDataObj<T> {
    /// create MutDataObj
    pub fn new(data: T) -> Self {
        Self { data: UnsafeCell::new(data) }
    }

    /// to mut data<br />
    /// after calling this method, you can assign values to properties in ref mode
    pub fn as_mut(&self) -> MutDataRef<T> {
        MutDataRef::new(self.data.get())
    }

    /// to raw mut<br />
    /// after calling this method, you can assign values to properties in ref mode
    pub fn as_raw_mut(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    /// get original value
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// set new values in ref mode<br />
    /// # Example:
    /// ```
    /// use cbsk_mut_data::mut_data_obj::MutDataObj;
    ///
    /// let b = MutDataObj::new(false);
    /// assert!(!*b);
    /// b.set(true);
    /// assert!(*b);
    /// ```
    pub fn set(&self, t: T) {
        *self.as_mut() = t;
    }
}

/// bool method
impl MutDataObj<bool> {
    /// get bool value
    pub fn get(&self) -> bool {
        **self
    }

    /// set data true
    pub fn set_true(&self) {
        self.set(true)
    }

    /// set data false
    pub fn set_false(&self) {
        self.set(false)
    }

    /// change data bool once
    pub fn trigger(&self) {
        self.set(!self.get())
    }
}

crate::impl_sync_send!(T,MutDataObj<T>);
crate::impl_debug_display!(T,MutDataObj<T>);
crate::impl_as_ref!(T,T,MutDataObj<T>);

/// support default
impl<T: Default> Default for MutDataObj<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// support deref
impl<T> Deref for MutDataObj<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.get() }
    }
}

/// support derefmut
impl<T> DerefMut for MutDataObj<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_raw_mut()
    }
}
