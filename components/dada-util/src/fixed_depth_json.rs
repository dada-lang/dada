use serde::ser::{
    self, Error, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

/// Create a JSON value for `value` but only up to a limited depth.
/// Values past that depth are represented as a "..." string.
pub fn to_json_value_max_depth(
    value: &dyn erased_serde::Serialize,
    max_depth: usize,
) -> serde_json::Value {
    let serializer = MaxDepthSerializer::new(max_depth);
    value
        .serialize(serializer)
        .unwrap_or(serde_json::Value::Null)
}
struct MaxDepthSerializer {
    current_depth: usize,
    max_depth: usize,
}

impl MaxDepthSerializer {
    fn new(max_depth: usize) -> Self {
        Self {
            current_depth: 0,
            max_depth,
        }
    }

    fn should_truncate(&self) -> bool {
        self.current_depth >= self.max_depth
    }
}

impl ser::Serializer for MaxDepthSerializer {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;
    type SerializeSeq = MaxDepthSeq;
    type SerializeTuple = MaxDepthSeq;
    type SerializeTupleStruct = MaxDepthSeq;
    type SerializeTupleVariant = MaxDepthSeq;
    type SerializeMap = MaxDepthMap;
    type SerializeStruct = MaxDepthMap;
    type SerializeStructVariant = MaxDepthMap;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(v.into()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(v as f64)
                .ok_or(serde_json::Error::custom("float conversion failed"))?,
        ))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(v)
                .ok_or(serde_json::Error::custom("float conversion failed"))?,
        ))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Array(
            v.iter()
                .map(|&b| serde_json::Value::Number(b.into()))
                .collect(),
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Null)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(serde_json::Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut map = serde_json::Map::new();
        map.insert(variant.to_owned(), value.serialize(self)?);
        Ok(serde_json::Value::Object(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if self.should_truncate() {
            Ok(MaxDepthSeq::Truncated)
        } else {
            Ok(MaxDepthSeq::Active {
                serializer: MaxDepthSerializer {
                    current_depth: self.current_depth + 1,
                    max_depth: self.max_depth,
                },
                vec: Vec::with_capacity(len.unwrap_or(0)),
            })
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        if self.should_truncate() {
            Ok(MaxDepthSeq::Truncated)
        } else {
            Ok(MaxDepthSeq::VariantActive {
                serializer: MaxDepthSerializer {
                    current_depth: self.current_depth + 1,
                    max_depth: self.max_depth,
                },
                variant: variant.to_owned(),
                vec: Vec::with_capacity(len),
            })
        }
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if self.should_truncate() {
            Ok(MaxDepthMap::Truncated)
        } else {
            Ok(MaxDepthMap::Active {
                serializer: MaxDepthSerializer {
                    current_depth: self.current_depth + 1,
                    max_depth: self.max_depth,
                },
                map: serde_json::Map::with_capacity(len.unwrap_or(0)),
                next_key: None,
            })
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        if self.should_truncate() {
            Ok(MaxDepthMap::Truncated)
        } else {
            Ok(MaxDepthMap::VariantActive {
                serializer: MaxDepthSerializer {
                    current_depth: self.current_depth + 1,
                    max_depth: self.max_depth,
                },
                variant: variant.to_owned(),
                map: serde_json::Map::with_capacity(len),
                next_key: None,
            })
        }
    }
}

enum MaxDepthSeq {
    Active {
        serializer: MaxDepthSerializer,
        vec: Vec<serde_json::Value>,
    },
    VariantActive {
        serializer: MaxDepthSerializer,
        variant: String,
        vec: Vec<serde_json::Value>,
    },
    Truncated,
}

impl SerializeSeq for MaxDepthSeq {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            MaxDepthSeq::Active { serializer, vec } => {
                vec.push(value.serialize(serializer.clone())?)
            }
            MaxDepthSeq::VariantActive {
                serializer, vec, ..
            } => vec.push(value.serialize(serializer.clone())?),
            MaxDepthSeq::Truncated => (),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            MaxDepthSeq::Active { vec, .. } => Ok(serde_json::Value::Array(vec)),
            MaxDepthSeq::VariantActive { variant, vec, .. } => {
                let mut map = serde_json::Map::new();
                map.insert(variant, serde_json::Value::Array(vec));
                Ok(serde_json::Value::Object(map))
            }
            MaxDepthSeq::Truncated => Ok(serde_json::Value::String("...".to_owned())),
        }
    }
}

impl SerializeTuple for MaxDepthSeq {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            MaxDepthSeq::Active { serializer, vec } => {
                vec.push(value.serialize(serializer.clone())?)
            }
            MaxDepthSeq::VariantActive {
                serializer, vec, ..
            } => vec.push(value.serialize(serializer.clone())?),
            MaxDepthSeq::Truncated => (),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            MaxDepthSeq::Active { vec, .. } => Ok(serde_json::Value::Array(vec)),
            MaxDepthSeq::VariantActive { variant, vec, .. } => {
                let mut map = serde_json::Map::new();
                map.insert(variant, serde_json::Value::Array(vec));
                Ok(serde_json::Value::Object(map))
            }
            MaxDepthSeq::Truncated => Ok(serde_json::Value::String("...".to_owned())),
        }
    }
}

impl SerializeTupleStruct for MaxDepthSeq {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            MaxDepthSeq::Active { serializer, vec } => {
                vec.push(value.serialize(serializer.clone())?)
            }
            MaxDepthSeq::VariantActive {
                serializer, vec, ..
            } => vec.push(value.serialize(serializer.clone())?),
            MaxDepthSeq::Truncated => (),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            MaxDepthSeq::Active { vec, .. } => Ok(serde_json::Value::Array(vec)),
            MaxDepthSeq::VariantActive { variant, vec, .. } => {
                let mut map = serde_json::Map::new();
                map.insert(variant, serde_json::Value::Array(vec));
                Ok(serde_json::Value::Object(map))
            }
            MaxDepthSeq::Truncated => Ok(serde_json::Value::String("...".to_owned())),
        }
    }
}

impl SerializeTupleVariant for MaxDepthSeq {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            MaxDepthSeq::Active { serializer, vec } => {
                vec.push(value.serialize(serializer.clone())?)
            }
            MaxDepthSeq::VariantActive {
                serializer, vec, ..
            } => vec.push(value.serialize(serializer.clone())?),
            MaxDepthSeq::Truncated => (),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            MaxDepthSeq::Active { vec, .. } => Ok(serde_json::Value::Array(vec)),
            MaxDepthSeq::VariantActive { variant, vec, .. } => {
                let mut map = serde_json::Map::new();
                map.insert(variant, serde_json::Value::Array(vec));
                Ok(serde_json::Value::Object(map))
            }
            MaxDepthSeq::Truncated => Ok(serde_json::Value::String("...".to_owned())),
        }
    }
}

enum MaxDepthMap {
    Active {
        serializer: MaxDepthSerializer,
        map: serde_json::Map<String, serde_json::Value>,
        next_key: Option<String>,
    },
    VariantActive {
        serializer: MaxDepthSerializer,
        variant: String,
        map: serde_json::Map<String, serde_json::Value>,
        next_key: Option<String>,
    },
    Truncated,
}

impl SerializeMap for MaxDepthMap {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            MaxDepthMap::Active { next_key, .. } | MaxDepthMap::VariantActive { next_key, .. } => {
                *next_key = Some(
                    key.serialize(MaxDepthSerializer::new(usize::MAX))
                        .and_then(|v| match v {
                            serde_json::Value::String(s) => Ok(s),
                            _ => Err(ser::Error::custom("map key must be a string")),
                        })?,
                );
            }
            MaxDepthMap::Truncated => (),
        }
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            MaxDepthMap::Active {
                serializer,
                map,
                next_key,
            }
            | MaxDepthMap::VariantActive {
                serializer,
                map,
                next_key,
                ..
            } => {
                let key = next_key.take().ok_or_else(|| {
                    ser::Error::custom("serialize_value called before serialize_key")
                })?;
                map.insert(key, value.serialize(serializer.clone())?);
            }
            MaxDepthMap::Truncated => (),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            MaxDepthMap::Active { map, .. } => Ok(serde_json::Value::Object(map)),
            MaxDepthMap::VariantActive { variant, map, .. } => {
                let mut outer_map = serde_json::Map::new();
                outer_map.insert(variant, serde_json::Value::Object(map));
                Ok(serde_json::Value::Object(outer_map))
            }
            MaxDepthMap::Truncated => Ok(serde_json::Value::String("...".to_owned())),
        }
    }
}

impl SerializeStruct for MaxDepthMap {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeMap::serialize_key(self, key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

impl SerializeStructVariant for MaxDepthMap {
    type Ok = serde_json::Value;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            MaxDepthMap::Active {
                serializer, map, ..
            }
            | MaxDepthMap::VariantActive {
                serializer, map, ..
            } => {
                map.insert(key.to_owned(), value.serialize(serializer.clone())?);
            }
            MaxDepthMap::Truncated => (),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            MaxDepthMap::Active { map, .. } => Ok(serde_json::Value::Object(map)),
            MaxDepthMap::VariantActive { variant, map, .. } => {
                let mut outer_map = serde_json::Map::new();
                outer_map.insert(variant, serde_json::Value::Object(map));
                Ok(serde_json::Value::Object(outer_map))
            }
            MaxDepthMap::Truncated => Ok(serde_json::Value::String("...".to_owned())),
        }
    }
}

impl Clone for MaxDepthSerializer {
    fn clone(&self) -> Self {
        Self {
            current_depth: self.current_depth,
            max_depth: self.max_depth,
        }
    }
}
