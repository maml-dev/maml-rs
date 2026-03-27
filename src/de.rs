use crate::error::Error;
use crate::value::Value;

/// Deserializes a MAML string into any type that implements [`Deserialize`](serde::de::Deserialize).
///
/// This parses the input into a [`Value`] and then deserializes into `T`.
///
/// # Errors
///
/// Returns an [`Error`] on invalid MAML syntax or if the value cannot be
/// deserialized into `T`.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Config {
///     name: String,
///     port: u16,
/// }
///
/// let config: Config = maml::from_str(r#"{name: "app", port: 8080}"#).unwrap();
/// assert_eq!(config, Config { name: "app".into(), port: 8080 });
/// ```
pub fn from_str<'de, T>(s: &str) -> Result<T, Error>
where
    T: serde::de::Deserialize<'de>,
{
    let value = crate::parse::parse(s)?;
    T::deserialize(value)
}

/// Deserializes a [`Value`] into any type that implements [`Deserialize`](serde::de::Deserialize).
///
/// # Errors
///
/// Returns an [`Error`] if the value cannot be deserialized into `T`.
///
/// # Examples
///
/// ```
/// let value = maml::parse("{x: 10, y: 20}").unwrap();
/// let point: std::collections::HashMap<String, i64> = maml::from_value(value).unwrap();
/// assert_eq!(point["x"], 10);
/// ```
pub fn from_value<'de, T>(value: Value) -> Result<T, Error>
where
    T: serde::de::Deserialize<'de>,
{
    T::deserialize(value)
}
