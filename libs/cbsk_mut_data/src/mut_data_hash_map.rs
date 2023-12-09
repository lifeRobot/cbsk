use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::vec::IntoIter;
use crate::mut_data_obj::MutDataObj;
use crate::mut_data_ref::MutDataRef;

/// ref mut hash map
pub struct MutDataHashMap<K, V> {
    data: MutDataObj<HashMap<K, V>>,
}

/// custom method
impl<K: Eq + Hash, V> MutDataHashMap<K, V> {
    /// insert data<br />
    /// more see [HashMap::insert]
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.as_mut().insert(key, value)
    }

    /// get mut data<br />
    /// more see [HashMap::get_mut]
    pub fn get_mut(&self, key: &K) -> Option<MutDataRef<V>> {
        self.as_mut().get_mut(key).map(MutDataRef::new_ref)
    }

    /// entry data<br />
    /// more see [HashMap::entry]
    pub fn entry(&self, key: K) -> Entry<K, V> {
        self.as_raw_mut().entry(key)
    }

    /// remove data<br />
    /// more see [HashMap::remove]
    pub fn remove(&self, key: &K) -> Option<V> {
        self.as_mut().remove(key)
    }

    /// iter mut data<br />
    /// currently do not recommend using this method<br />
    /// alternative methods：
    /// ```
    /// use cbsk_mut_data::mut_data_hash_map::MutDataHashMap;
    /// use cbsk_mut_data::mut_data_ref::MutDataRef;
    ///
    /// let map = MutDataHashMap::default();
    /// map.insert(1,1);
    /// map.as_mut().iter_mut().map(|(k,v)|{(k,MutDataRef::new_ref(v))}).for_each(|map|{
    ///     println!("map is {map:?}")
    /// });
    /// ```
    pub fn iter_mut(&self) -> std::collections::hash_map::IntoIter<&K, MutDataRef<V>> {
        self.as_raw_mut().iter_mut().map(|(k, v)| { (k, MutDataRef::new_ref(v)) }).collect::<HashMap<&K, MutDataRef<V>>>().into_iter()
    }
}

/// custom method
impl<K, V> MutDataHashMap<K, V> {
    /// clear data<br />
    /// more see [HashMap::clear]
    pub fn clear(&self) {
        self.as_mut().clear();
    }

    /// get mut values<br />
    /// currently do not recommend using this method<br />
    /// alternative methods：
    /// ```
    /// use cbsk_mut_data::mut_data_hash_map::MutDataHashMap;
    /// use cbsk_mut_data::mut_data_ref::MutDataRef;
    ///
    /// let map = MutDataHashMap::default();
    /// map.insert(1,1);
    /// let iter = map.as_mut().values_mut().map(MutDataRef::new_ref);
    /// iter.for_each(|v|{
    ///     println!("v is {v}");
    /// });
    /// ```
    pub fn values_mut(&self) -> IntoIter<MutDataRef<V>> {
        self.as_mut().values_mut().map(MutDataRef::new_ref).collect::<Vec<MutDataRef<V>>>().into_iter()
    }
}

crate::impl_sync_send!([K,V],MutDataHashMap<K,V>);
crate::impl_debug!([K,V],MutDataHashMap<K,V>);
crate::impl_as_ref!([K,V],HashMap<K,V>,MutDataHashMap<K,V>);
crate::impl_default!([K,V],MutDataHashMap<K,V>,Self { data: MutDataObj::default() });

/// support deref
impl<K, V> Deref for MutDataHashMap<K, V> {
    type Target = MutDataObj<HashMap<K, V>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
