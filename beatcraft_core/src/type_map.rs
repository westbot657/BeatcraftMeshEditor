use std::any::Any;
use jni::JValue;
use jni::sys::jlong;
use slotmap::{new_key_type, KeyData, SlotMap};

new_key_type! { pub struct DataKey; }

impl From<u64> for DataKey {
    fn from(value: u64) -> Self {
        Self::from(KeyData::from_ffi(value))
    }
}

impl From<i64> for DataKey {
    fn from(value: i64) -> Self {
        Self::from(KeyData::from_ffi(value as u64))
    }
}

impl From<DataKey> for JValue<'_> {
    fn from(value: DataKey) -> Self {
        (value.0.as_ffi() as jlong).into()
    }
}

#[derive(Default)]
pub struct TypeMap {
    map: SlotMap<DataKey, Box<dyn Any>>,
}

impl TypeMap {
    pub fn insert<T: Any>(&mut self, value: T) -> DataKey {
        self.map.insert(Box::new(value))
    }

    pub fn get<T: Any>(&self, key: DataKey) -> Option<&T> {
        self.map
            .get(key)?
            .downcast_ref()
    }

    pub fn get_mut<T: Any>(&mut self, key: DataKey) -> Option<&mut T> {
        self.map
            .get_mut(key)?
            .downcast_mut()
    }

    pub fn remove<T: Any>(&mut self, key: DataKey) -> Option<T> {
        Some(*self.map
            .remove(key)?
            .downcast().ok()?)
    }
}




