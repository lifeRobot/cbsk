use std::ops::Deref;
use std::vec::IntoIter;
use crate::mut_data_obj::MutDataObj;
use crate::mut_data_ref::MutDataRef;

/// ref mut vec
#[derive(Default)]
pub struct MutDataVec<T> {
    /// mut data
    data: MutDataObj<Vec<T>>,
}

/// custom method
impl<T> MutDataVec<T> {
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

crate::impl_sync_send!(T,MutDataVec<T>);
crate::impl_debug!(T,MutDataVec<T>);
crate::impl_as_ref!(T,Vec<T>,MutDataVec<T>);

/// support deref
impl<T> Deref for MutDataVec<T> {
    type Target = MutDataObj<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
