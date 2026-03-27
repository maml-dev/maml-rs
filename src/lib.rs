//! A parser and serializer for [MAML](https://maml.dev) (Minimal Abstract Markup Language) — a
//! minimal, human-readable data format.
//!
//! # Quick start
//!
//! ```
//! use maml::{parse, stringify, Value};
//!
//! // Parse a MAML string into a Value
//! let value = parse(r#"{name: "maml", version: 1}"#).unwrap();
//! assert_eq!(value["name"], Value::String("maml".into()));
//!
//! // Serialize a Value back to a MAML string
//! let output = stringify(&value).unwrap();
//! assert_eq!(output, "{\n  name: \"maml\"\n  version: 1\n}");
//! ```
//!
//! # Serde support
//!
//! With the `serde` feature (enabled by default), you can serialize and deserialize
//! any type that implements `Serialize`/`Deserialize`:
//!
//! ```
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct Config {
//!     name: String,
//!     port: u16,
//!     debug: bool,
//! }
//!
//! let config = Config { name: "app".into(), port: 8080, debug: false };
//!
//! // Serialize to MAML
//! let s = maml::to_string(&config).unwrap();
//!
//! // Deserialize from MAML
//! let back: Config = maml::from_str(&s).unwrap();
//! assert_eq!(config, back);
//! ```
//!
//! To use this crate without serde, disable default features:
//!
//! ```toml
//! [dependencies]
//! maml = { version = "0.1", default-features = false }
//! ```
//!
//! # MAML format
//!
//! ```text
//! {
//!   # Comments start with hash
//!   key: "quoted string"
//!   identifier_key: 42
//!
//!   array: [
//!     "comma or newline separated"
//!     true
//!     null
//!     3.14
//!   ]
//!
//!   raw_string: """
//! No escaping needed here.
//! Preserves \n and "quotes" as-is.
//! """
//! }
//! ```
//!
//! See the full specification at <https://maml.dev>.

mod error;
mod parse;
mod stringify;
mod value;

#[cfg(feature = "serde")]
mod de;
#[cfg(feature = "serde")]
mod ser;

pub use error::Error;
pub use parse::parse;
pub use stringify::stringify;
pub use value::Value;

#[cfg(feature = "serde")]
pub use de::{from_str, from_value};
#[cfg(feature = "serde")]
pub use ser::to_string;
