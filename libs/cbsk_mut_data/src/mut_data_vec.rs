use std::ops::Deref;
use std::vec::IntoIter;
use crate::mut_data_obj::MutDataObj;
use crate::mut_data_ref::MutDataRef;

/// ref mut vec
pub struct MutDataVec<T> {
    /// mut data
    data: MutDataObj<Vec<T>>,
}

/// custom method
impl<T> MutDataVec<T> {
    /// new mut data vec, more see [Vec::new]
    pub fn new() -> Self {
        Self::default()
    }

    /// always use the capacity method after construction, more see [Vec::with_capacity]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { data: MutDataObj::new(Vec::with_capacity(capacity)) }
    }

    /// push data
    pub fn push(&self, t: T) {
        self.as_mut().push(t);
    }

    /// append data
    pub fn append(&self, other: &mut Vec<T>) {
        self.as_mut().append(other)
    }

    /// get mu data
    pub fn get_mut(&self, index: usize) -> Option<MutDataRef<T>> {
        self.as_mut().get_mut(index).map(MutDataRef::new_ref)
    }

    /// get last mut data
    pub fn last_mut(&self) -> Option<MutDataRef<T>> {
        self.as_mut().last_mut().map(MutDataRef::new_ref)
    }

    /// remove a data<br />
    /// more see [Vec::remove]
    pub fn remove(&self, index: usize) -> T {
        self.as_mut().remove(index)
    }

    /// clear data<br />
    /// more see [Vec::clear]
    pub fn clear(&self) {
        self.as_mut().clear();
    }

    /// pop data<br />
    /// more see [Vec::pop]
    pub fn pop(&self) -> Option<T> {
        self.as_mut().pop()
    }

    /// iter mut data<br />
    /// currently do not recommend using this method<br />
    /// alternative methods：
    /// ```
    /// use cbsk_mut_data::mut_data_ref::MutDataRef;
    /// use cbsk_mut_data::mut_data_vec::MutDataVec;
    ///
    /// let list = MutDataVec::<i32>::default();
    /// let iter = list.as_mut().iter_mut()
    ///                 .map(MutDataRef::new_ref);
    ///
    /// iter.for_each(|l|{
    ///     println!("l is {l}");
    /// });
    /// ```
    pub fn iter_mut(&self) -> IntoIter<MutDataRef<T>> {
        self.as_mut().iter_mut().map(MutDataRef::new_ref).collect::<Vec<MutDataRef<T>>>().into_iter()
    }
}

/// support Vec into MutDataVec
impl<T> From<Vec<T>> for MutDataVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self { data: MutDataObj::new(value) }
    }
}

crate::impl_sync_send!(T,MutDataVec<T>);
crate::impl_debug!(T,MutDataVec<T>);
crate::impl_as_ref!(T,Vec<T>,MutDataVec<T>);
crate::impl_default!(T,MutDataVec<T>,Self { data: MutDataObj::default() });

/// support deref
impl<T> Deref for MutDataVec<T> {
    type Target = MutDataObj<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
