use std::ops::Index;

/// Represents a MAML value.
///
/// This is the core type for working with MAML data. It supports all seven
/// MAML types: null, booleans, integers, floats, strings, arrays, and objects.
///
/// Objects preserve insertion order using `Vec<(String, Value)>`.
///
/// # Indexing
///
/// `Value` supports indexing with `&str` for objects and `usize` for arrays:
///
/// ```
/// use maml::{parse, Value};
///
/// let v = parse(r#"{items: [1, 2, 3]}"#).unwrap();
/// assert_eq!(v["items"][0], Value::Int(1));
/// ```
///
/// # Conversions
///
/// Common types can be converted into `Value` using `From`:
///
/// ```
/// use maml::Value;
///
/// let v: Value = "hello".into();
/// assert_eq!(v.as_str(), Some("hello"));
///
/// let v: Value = 42i64.into();
/// assert_eq!(v.as_i64(), Some(42));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Represents a MAML `null` value.
    Null,
    /// Represents a MAML boolean (`true` or `false`).
    Bool(bool),
    /// Represents a MAML integer (64-bit signed).
    Int(i64),
    /// Represents a MAML float (64-bit IEEE 754).
    Float(f64),
    /// Represents a MAML string (quoted or raw).
    String(String),
    /// Represents a MAML array.
    Array(Vec<Value>),
    /// Represents a MAML object with ordered key-value pairs.
    Object(Vec<(String, Value)>),
}

// ---------------------------------------------------------------------------
// Value accessors
// ---------------------------------------------------------------------------

impl Value {
    /// Returns `true` if the value is `Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// If the value is a `Bool`, returns the inner value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// If the value is an `Int`, returns the inner value.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// If the value is a `Float`, returns the inner value.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(n) => Some(*n),
            _ => None,
        }
    }

    /// If the value is a `String`, returns a reference to the inner string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the value is an `Array`, returns a reference to the inner slice.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// If the value is an `Object`, returns a reference to the key-value pairs.
    pub fn as_object(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Looks up a value by key in an `Object`. Returns `None` if the value is
    /// not an object or the key is missing.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Index impls
// ---------------------------------------------------------------------------

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, key: &str) -> &Value {
        self.get(key).expect("key not found")
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Value {
        match self {
            Value::Array(a) => &a[index],
            _ => panic!("not an array"),
        }
    }
}

// ---------------------------------------------------------------------------
// From impls
// ---------------------------------------------------------------------------

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Float(n)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<Vec<Value>> for Value {
    fn from(a: Vec<Value>) -> Self {
        Value::Array(a)
    }
}

// ---------------------------------------------------------------------------
// Serde impls (requires "serde" feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "serde")]
mod serde_impl {
    use std::fmt;

    use serde::de::{
        self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
    };
    use serde::forward_to_deserialize_any;

    use crate::error::Error;

    use super::Value;

    // -----------------------------------------------------------------------
    // Serialize for Value
    // -----------------------------------------------------------------------

    impl serde::Serialize for Value {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Value::Null => serializer.serialize_unit(),
                Value::Bool(b) => serializer.serialize_bool(*b),
                Value::Int(n) => serializer.serialize_i64(*n),
                Value::Float(n) => serializer.serialize_f64(*n),
                Value::String(s) => serializer.serialize_str(s),
                Value::Array(a) => serializer.collect_seq(a),
                Value::Object(pairs) => serializer.collect_map(pairs.iter().map(|(k, v)| (k, v))),
            }
        }
    }

    // -----------------------------------------------------------------------
    // Deserialize for Value
    // -----------------------------------------------------------------------

    impl<'de> serde::Deserialize<'de> for Value {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Value, D::Error> {
            struct ValueVisitor;

            impl<'de> Visitor<'de> for ValueVisitor {
                type Value = Value;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("any valid MAML value")
                }

                fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
                    Ok(Value::Bool(v))
                }

                fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
                    Ok(Value::Int(v))
                }

                fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
                    i64::try_from(v).map(Value::Int).map_err(de::Error::custom)
                }

                fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
                    Ok(Value::Float(v))
                }

                fn visit_str<E>(self, v: &str) -> Result<Value, E> {
                    Ok(Value::String(v.to_owned()))
                }

                fn visit_string<E>(self, v: String) -> Result<Value, E> {
                    Ok(Value::String(v))
                }

                fn visit_none<E>(self) -> Result<Value, E> {
                    Ok(Value::Null)
                }

                fn visit_unit<E>(self) -> Result<Value, E> {
                    Ok(Value::Null)
                }

                fn visit_some<D: serde::Deserializer<'de>>(
                    self,
                    deserializer: D,
                ) -> Result<Value, D::Error> {
                    serde::Deserialize::deserialize(deserializer)
                }

                fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
                    let mut vec = Vec::new();
                    while let Some(elem) = seq.next_element()? {
                        vec.push(elem);
                    }
                    Ok(Value::Array(vec))
                }

                fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Value, M::Error> {
                    let mut pairs = Vec::new();
                    while let Some((key, value)) = map.next_entry()? {
                        pairs.push((key, value));
                    }
                    Ok(Value::Object(pairs))
                }
            }

            deserializer.deserialize_any(ValueVisitor)
        }
    }

    // -----------------------------------------------------------------------
    // Deserializer for Value (Value -> T)
    // -----------------------------------------------------------------------

    impl<'de> serde::Deserializer<'de> for Value {
        type Error = Error;

        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self {
                Value::Null => visitor.visit_unit(),
                Value::Bool(b) => visitor.visit_bool(b),
                Value::Int(n) => visitor.visit_i64(n),
                Value::Float(n) => visitor.visit_f64(n),
                Value::String(s) => visitor.visit_string(s),
                Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a)),
                Value::Object(pairs) => visitor.visit_map(MapDeserializer::new(pairs)),
            }
        }

        fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self {
                Value::Null => visitor.visit_none(),
                other => visitor.visit_some(other),
            }
        }

        fn deserialize_enum<V: Visitor<'de>>(
            self,
            _name: &'static str,
            _variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error> {
            match self {
                Value::String(s) => visitor.visit_enum(EnumDeserializer {
                    variant: s,
                    value: None,
                }),
                Value::Object(mut pairs) => {
                    if pairs.len() != 1 {
                        return Err(de::Error::custom(
                            "expected an object with exactly one key for enum",
                        ));
                    }
                    let (variant, value) = pairs.remove(0);
                    visitor.visit_enum(EnumDeserializer {
                        variant,
                        value: Some(value),
                    })
                }
                _ => Err(de::Error::custom("expected a string or object for enum")),
            }
        }

        fn deserialize_newtype_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Error> {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error> {
            match self {
                Value::Object(_) => self.deserialize_any(visitor),
                _ => Err(de::Error::custom("expected an object")),
            }
        }

        forward_to_deserialize_any! {
            bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string
            bytes byte_buf unit seq map unit_struct tuple_struct
            tuple ignored_any identifier
        }
    }

    // -----------------------------------------------------------------------
    // SeqDeserializer
    // -----------------------------------------------------------------------

    struct SeqDeserializer {
        iter: std::vec::IntoIter<Value>,
        len: usize,
    }

    impl SeqDeserializer {
        fn new(vec: Vec<Value>) -> Self {
            let len = vec.len();
            Self {
                iter: vec.into_iter(),
                len,
            }
        }
    }

    impl<'de> SeqAccess<'de> for SeqDeserializer {
        type Error = Error;

        fn next_element_seed<T: DeserializeSeed<'de>>(
            &mut self,
            seed: T,
        ) -> Result<Option<T::Value>, Error> {
            match self.iter.next() {
                Some(value) => {
                    self.len -= 1;
                    seed.deserialize(value).map(Some)
                }
                None => Ok(None),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.len)
        }
    }

    // -----------------------------------------------------------------------
    // MapDeserializer
    // -----------------------------------------------------------------------

    struct MapDeserializer {
        iter: std::vec::IntoIter<(String, Value)>,
        value: Option<Value>,
        len: usize,
    }

    impl MapDeserializer {
        fn new(pairs: Vec<(String, Value)>) -> Self {
            let len = pairs.len();
            Self {
                iter: pairs.into_iter(),
                value: None,
                len,
            }
        }
    }

    impl<'de> MapAccess<'de> for MapDeserializer {
        type Error = Error;

        fn next_key_seed<K: DeserializeSeed<'de>>(
            &mut self,
            seed: K,
        ) -> Result<Option<K::Value>, Error> {
            match self.iter.next() {
                Some((key, value)) => {
                    self.len -= 1;
                    self.value = Some(value);
                    seed.deserialize(Value::String(key)).map(Some)
                }
                None => Ok(None),
            }
        }

        fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
            // unwrap: serde guarantees next_key_seed is called before next_value_seed
            seed.deserialize(self.value.take().unwrap())
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.len)
        }
    }

    // -----------------------------------------------------------------------
    // EnumDeserializer
    // -----------------------------------------------------------------------

    struct EnumDeserializer {
        variant: String,
        value: Option<Value>,
    }

    impl<'de> EnumAccess<'de> for EnumDeserializer {
        type Error = Error;
        type Variant = VariantDeserializer;

        fn variant_seed<V: DeserializeSeed<'de>>(
            self,
            seed: V,
        ) -> Result<(V::Value, Self::Variant), Error> {
            let variant = seed.deserialize(Value::String(self.variant))?;
            Ok((variant, VariantDeserializer { value: self.value }))
        }
    }

    // -----------------------------------------------------------------------
    // VariantDeserializer
    // -----------------------------------------------------------------------

    struct VariantDeserializer {
        value: Option<Value>,
    }

    impl<'de> VariantAccess<'de> for VariantDeserializer {
        type Error = Error;

        fn unit_variant(self) -> Result<(), Error> {
            match self.value {
                None => Ok(()),
                Some(_) => Err(de::Error::custom("expected unit variant, found value")),
            }
        }

        fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
            match self.value {
                Some(value) => seed.deserialize(value),
                None => Err(de::Error::custom("expected newtype variant")),
            }
        }

        fn tuple_variant<V: Visitor<'de>>(
            self,
            _len: usize,
            visitor: V,
        ) -> Result<V::Value, Error> {
            match self.value {
                Some(Value::Array(a)) => visitor.visit_seq(SeqDeserializer::new(a)),
                Some(_) => Err(de::Error::custom("expected array for tuple variant")),
                None => Err(de::Error::custom("expected tuple variant")),
            }
        }

        fn struct_variant<V: Visitor<'de>>(
            self,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error> {
            match self.value {
                Some(Value::Object(pairs)) => visitor.visit_map(MapDeserializer::new(pairs)),
                Some(_) => Err(de::Error::custom("expected object for struct variant")),
                None => Err(de::Error::custom("expected struct variant")),
            }
        }
    }
}
