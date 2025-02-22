//! Serde support for UniMRCP vendor specific parameters.

use crate::ffi;
use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    Deserialize, Deserializer,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("serde: {0}")]
    Serde(String),
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(t: T) -> Self {
        Error::Serde(t.to_string())
    }
}

/// Deserialize a struct from an array of vendor specific parameters.
pub unsafe fn from_header_array<'a, T>(header: *mut ffi::apt_pair_arr_t) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = HeaderDeserializer { header, index: 0 };
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

struct HeaderDeserializer {
    header: *mut ffi::apt_pair_arr_t,
    index: i32,
}

impl<'de, 'a> Deserializer<'de> for &'a mut HeaderDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = unsafe { ffi::apt_pair_array_get(self.header, self.index) };
        let s = unsafe { (*pair).value.as_str() };
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(KeyValueList::new(&mut self))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = unsafe { ffi::apt_pair_array_get(self.header, self.index) };
        let s = unsafe { (*pair).value.as_str() };
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = unsafe { ffi::apt_pair_array_get(self.header, self.index) };
        let s = unsafe { (*pair).value.as_str() };
        visitor.visit_string(s.to_string())
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = unsafe { ffi::apt_pair_array_get(self.header, self.index) };
        let s = unsafe { (*pair).value.as_str() };
        let value = s.parse().map_err(serde::de::Error::custom)?;
        visitor.visit_bool(value)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = unsafe { ffi::apt_pair_array_get(self.header, self.index) };
        match unsafe { (*pair).value }.length {
            0 => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let pair = unsafe { ffi::apt_pair_array_get(self.header, self.index) };
        let s = unsafe { (*pair).name.as_str() };
        visitor.visit_borrowed_str(s)
    }

    serde::forward_to_deserialize_any! {
            i8 i16 i32 i64 i128
            u8 u16 u32 u64 u128
            f32 f64
            char bytes byte_buf
            unit unit_struct newtype_struct seq tuple
            tuple_struct enum ignored_any
    }
}

struct KeyValueList<'a> {
    de: &'a mut HeaderDeserializer,
}

impl<'a> KeyValueList<'a> {
    fn new(de: &'a mut HeaderDeserializer) -> Self {
        KeyValueList { de }
    }
}

impl<'de, 'a> MapAccess<'de> for KeyValueList<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let size = unsafe { ffi::apt_pair_array_size_get(self.de.header) };
        if self.de.index == size {
            Ok(None)
        } else {
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let result = seed.deserialize(&mut *self.de);
        self.de.index += 1;
        result
    }
}
