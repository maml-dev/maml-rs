# maml

A Rust implementation of the [MAML](https://maml.dev) data format — a minimal, human-readable alternative to JSON, YAML,
and TOML.

## Installation

```toml
[dependencies]
maml = "0.1"
```

## Usage

### Parsing

```rust
use maml::{parse, Value};

let value = parse(r#"
{
  project: "MAML"
  tags: [
    "minimal"
    "readable"
  ]

  # A nested object
  spec: {
    version: 1
    author: "Anton Medvedev"
  }

  notes: """
This is a raw multiline string.
Keeps formatting as-is.
"""
}
"#).unwrap();

assert_eq!(value["project"], Value::String("MAML".into()));
assert_eq!(value["spec"]["version"], Value::Int(1));
```

### Serializing

```rust
use maml::{stringify, Value};

let value = Value::Object(vec![
    ("name".into(), Value::String("maml".into())),
    ("version".into(), Value::Int(1)),
    ("enabled".into(), Value::Bool(true)),
]);

let output = stringify(&value).unwrap();
// {
//   name: "maml"
//   version: 1
//   enabled: true
// }
```

### Value type

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}
```

`Value` supports indexing with `["key"]` for objects and `[0]` for arrays, and provides accessor methods like
`as_str()`, `as_i64()`, `as_bool()`, `as_array()`, `as_object()`, `get(key)`, and `is_null()`.

## MAML format

```maml
{
  # Comments start with hash
  key: "quoted string"
  identifier_key: 42

  array: [
    "comma or newline separated"
    true
    null
    3.14
  ]

  raw_string: """
No escaping needed here.
Preserves \n and "quotes" as-is.
"""
}
```

See the full [MAML specification](https://maml.dev).

## License

[MIT](LICENSE)
