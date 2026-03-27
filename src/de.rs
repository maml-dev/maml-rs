use crate::error::Error;
use crate::value::Value;

pub fn from_str<'de, T>(s: &str) -> Result<T, Error>
where
    T: serde::de::Deserialize<'de>,
{
    let value = crate::parse::parse(s)?;
    T::deserialize(value)
}

pub fn from_value<'de, T>(value: Value) -> Result<T, Error>
where
    T: serde::de::Deserialize<'de>,
{
    T::deserialize(value)
}
