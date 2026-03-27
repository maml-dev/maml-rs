use serde::ser::{self, Serialize};

use crate::error::Error;
use crate::stringify::{get_indent, quote_string, stringify_key};

/// Serializes any type that implements [`Serialize`] into a MAML string.
///
/// The output uses 2-space indentation and newline-separated entries.
///
/// # Errors
///
/// Returns an [`Error`] if the value contains non-finite floats or
/// unsupported map key types.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     port: u16,
/// }
///
/// let config = Config { name: "app".into(), port: 8080 };
/// let output = maml::to_string(&config).unwrap();
/// assert_eq!(output, "{\n  name: \"app\"\n  port: 8080\n}");
/// ```
pub fn to_string<T: Serialize>(value: &T) -> Result<String, Error> {
    let mut output = String::new();
    let serializer = Serializer {
        output: &mut output,
        level: 0,
    };
    value.serialize(serializer)?;
    Ok(output)
}

// ---------------------------------------------------------------------------
// Serializer
// ---------------------------------------------------------------------------

struct Serializer<'o> {
    output: &'o mut String,
    level: usize,
}

impl<'o> ser::Serializer for Serializer<'o> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeSeq<'o>;
    type SerializeTuple = SerializeSeq<'o>;
    type SerializeTupleStruct = SerializeSeq<'o>;
    type SerializeTupleVariant = SerializeSeq<'o>;
    type SerializeMap = SerializeMap<'o>;
    type SerializeStruct = SerializeMap<'o>;
    type SerializeStructVariant = SerializeMap<'o>;

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output.push_str(if v { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        if v > i64::MAX as u64 {
            return Err(ser::Error::custom("u64 value exceeds MAML integer range"));
        }
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        if !v.is_finite() {
            return Err(ser::Error::custom("cannot serialize non-finite float"));
        }
        if v == 0.0 && v.is_sign_negative() {
            self.output.push_str("-0");
        } else {
            let s = v.to_string();
            if s.contains('.') || s.contains('e') || s.contains('E') {
                self.output.push_str(&s);
            } else {
                self.output.push_str(&s);
                self.output.push_str(".0");
            }
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.output.push_str(&quote_string(v));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        use ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for b in v {
            seq.serialize_element(b)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<(), Error> {
        self.output.push_str("null");
        Ok(())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Error> {
        self.output.push_str("null");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        let level = self.level;
        let output = self.output;
        output.push_str("{\n");
        let child_indent = get_indent(level + 1);
        output.push_str(&child_indent);
        output.push_str(&stringify_key(variant));
        output.push_str(": ");
        value.serialize(Serializer {
            output,
            level: level + 1,
        })?;
        output.push('\n');
        output.push_str(&get_indent(level));
        output.push('}');
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SerializeSeq<'o>, Error> {
        if len == Some(0) {
            self.output.push_str("[]");
            return Ok(SerializeSeq {
                output: self.output,
                level: self.level,
                first: false,
                empty: true,
            });
        }
        self.output.push('[');
        Ok(SerializeSeq {
            output: self.output,
            level: self.level,
            first: true,
            empty: false,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeSeq<'o>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SerializeSeq<'o>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeSeq<'o>, Error> {
        let level = self.level;
        let output = self.output;
        output.push_str("{\n");
        let child_indent = get_indent(level + 1);
        output.push_str(&child_indent);
        output.push_str(&stringify_key(variant));
        output.push_str(": ");
        // The inner array
        if len == 0 {
            output.push_str("[]");
            Ok(SerializeSeq {
                output,
                level: level + 1,
                first: false,
                empty: true,
            })
        } else {
            output.push('[');
            Ok(SerializeSeq {
                output,
                level: level + 1,
                first: true,
                empty: false,
            })
        }
    }

    fn serialize_map(self, len: Option<usize>) -> Result<SerializeMap<'o>, Error> {
        if len == Some(0) {
            self.output.push_str("{}");
            return Ok(SerializeMap {
                output: self.output,
                level: self.level,
                first: false,
                empty: true,
                variant_wrapper: None,
            });
        }
        self.output.push('{');
        Ok(SerializeMap {
            output: self.output,
            level: self.level,
            first: true,
            empty: false,
            variant_wrapper: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<SerializeMap<'o>, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SerializeMap<'o>, Error> {
        let level = self.level;
        let output = self.output;
        output.push_str("{\n");
        let child_indent = get_indent(level + 1);
        output.push_str(&child_indent);
        output.push_str(&stringify_key(variant));
        output.push_str(": ");
        if len == 0 {
            output.push_str("{}");
            Ok(SerializeMap {
                output,
                level: level + 1,
                first: false,
                empty: true,
                variant_wrapper: Some(level),
            })
        } else {
            output.push('{');
            Ok(SerializeMap {
                output,
                level: level + 1,
                first: true,
                empty: false,
                variant_wrapper: Some(level),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// SerializeSeq (also used for Tuple, TupleStruct, TupleVariant)
// ---------------------------------------------------------------------------

pub struct SerializeSeq<'o> {
    output: &'o mut String,
    level: usize,
    first: bool,
    empty: bool,
}

impl<'o> ser::SerializeSeq for SerializeSeq<'o> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        if !self.first {
            // newline between elements
        }
        self.first = false;
        self.output.push('\n');
        self.output.push_str(&get_indent(self.level + 1));
        value.serialize(Serializer {
            output: self.output,
            level: self.level + 1,
        })
    }

    fn end(self) -> Result<(), Error> {
        if !self.empty {
            self.output.push('\n');
            self.output.push_str(&get_indent(self.level));
            self.output.push(']');
        }
        Ok(())
    }
}

impl<'o> ser::SerializeTuple for SerializeSeq<'o> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'o> ser::SerializeTupleStruct for SerializeSeq<'o> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'o> ser::SerializeTupleVariant for SerializeSeq<'o> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), Error> {
        if !self.empty {
            self.output.push('\n');
            self.output.push_str(&get_indent(self.level));
            self.output.push(']');
        }
        // Close the outer variant wrapper object
        self.output.push('\n');
        self.output.push_str(&get_indent(self.level - 1));
        self.output.push('}');
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SerializeMap (also used for Struct, StructVariant)
// ---------------------------------------------------------------------------

pub struct SerializeMap<'o> {
    output: &'o mut String,
    level: usize,
    first: bool,
    empty: bool,
    variant_wrapper: Option<usize>,
}

impl<'o> ser::SerializeMap for SerializeMap<'o> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        if !self.first {
            // newline between entries
        }
        self.first = false;
        self.output.push('\n');
        self.output.push_str(&get_indent(self.level + 1));
        let mut key_str = String::new();
        key.serialize(MapKeySerializer {
            output: &mut key_str,
        })?;
        self.output.push_str(&stringify_key(&key_str));
        self.output.push_str(": ");
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(Serializer {
            output: self.output,
            level: self.level + 1,
        })
    }

    fn end(self) -> Result<(), Error> {
        if !self.empty {
            self.output.push('\n');
            self.output.push_str(&get_indent(self.level));
            self.output.push('}');
        }
        Ok(())
    }
}

impl<'o> ser::SerializeStruct for SerializeMap<'o> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<(), Error> {
        ser::SerializeMap::end(self)
    }
}

impl<'o> ser::SerializeStructVariant for SerializeMap<'o> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<(), Error> {
        if !self.empty {
            self.output.push('\n');
            self.output.push_str(&get_indent(self.level));
            self.output.push('}');
        }
        // Close outer variant wrapper object
        if let Some(outer_level) = self.variant_wrapper {
            self.output.push('\n');
            self.output.push_str(&get_indent(outer_level));
            self.output.push('}');
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MapKeySerializer — extracts a string from the key
// ---------------------------------------------------------------------------

struct MapKeySerializer<'o> {
    output: &'o mut String,
}

impl<'o> ser::Serializer for MapKeySerializer<'o> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.output.push_str(v);
        Ok(())
    }

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        if v > i64::MAX as u64 {
            return Err(ser::Error::custom("u64 value exceeds MAML integer range"));
        }
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<(), Error> {
        Err(ser::Error::custom("float keys are not supported"))
    }

    fn serialize_f64(self, _v: f64) -> Result<(), Error> {
        Err(ser::Error::custom("float keys are not supported"))
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.output.push(v);
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<(), Error> {
        Err(ser::Error::custom("bytes keys are not supported"))
    }

    fn serialize_none(self) -> Result<(), Error> {
        Err(ser::Error::custom("null keys are not supported"))
    }

    fn serialize_some<T: Serialize + ?Sized>(self, _value: &T) -> Result<(), Error> {
        Err(ser::Error::custom("optional keys are not supported"))
    }

    fn serialize_unit(self) -> Result<(), Error> {
        Err(ser::Error::custom("unit keys are not supported"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        Err(ser::Error::custom("unit struct keys are not supported"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.output.push_str(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        Err(ser::Error::custom("newtype struct keys are not supported"))
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        Err(ser::Error::custom("newtype variant keys are not supported"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(ser::Error::custom("sequence keys are not supported"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(ser::Error::custom("tuple keys are not supported"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(ser::Error::custom("tuple struct keys are not supported"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(ser::Error::custom("tuple variant keys are not supported"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(ser::Error::custom("map keys are not supported"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Err(ser::Error::custom("struct keys are not supported"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(ser::Error::custom("struct variant keys are not supported"))
    }
}
