# _DYN SERDE_

**Dynamic serialization and deserialization based on serde.**

TODO: better benchmarks

TODO: no-alloc support

## Performance

Some simple benchmarks could prove that `serde_dyn` is much faster than
`erased_serde`. Because the later does a lot of memory allocations and manually
manages a virtual table for an arbitrary object type.

Instead, `serde_dyn` uses in-place serialization and deserialization, allowing
to get rid of _unsafe_ codes and eliminate most of the memory allocations. The
only required allocation happens in the error handling, which makes it possible
to support "no-alloc" feature.

## No-std support

To opt off "std", add the following line in the `Cargo.toml`:

```toml
serde_dyn = { version = "*", default-features = false }
```

Note that, without "std", `serde_dyn` will use "alloc" library to support error
handling.

## No-alloc support

Planned but currently not supported.

Without memory allocation, error messages need to be discarded, but the
serialization and deserialization procedure won't be affected.

## Examples

### Use `Serialize`

```Rust
use serde_dyn::Serialize;

let heterogeneous = [
    &[true, false] as &dyn Serialize,
    &100_u8 as &dyn Serialize,
    &"Hello, world" as &dyn Serialize,
    &3.14_f32 as &dyn Serialize,
];

let json = serde_json::to_string(&heterogeneous).unwrap();
assert_eq!(json, r#"[[true,false],100,"Hello, world",3.14]"#);
```

### Use `Serializer`

```Rust
use serde_dyn::Serializer;

let mut json = Vec::new();
let writer = std::io::Cursor::new(&mut json);
let mut serializer = serde_json::Serializer::new(writer);
let mut serializer = <dyn Serializer>::new(&mut serializer);
let serializer: &mut dyn Serializer = &mut serializer;

let data = ["Hello", "World"];
serde::Serialize::serialize(&data, serializer).unwrap();
assert_eq!(json, b"[\"Hello\",\"World\"]");
```

### Use `Deserializer`

```Rust
use serde_dyn::Deserializer;

let mut deserializer = serde_json::Deserializer::from_str("[\"Hello\",\"World\"]");
let mut deserializer = <dyn Deserializer>::new(&mut deserializer);
let deserializer: &mut dyn Deserializer = &mut deserializer;

let value: Vec<String> = serde::Deserialize::deserialize(deserializer).unwrap();
assert_eq!(value, vec!["Hello", "World"]);
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
