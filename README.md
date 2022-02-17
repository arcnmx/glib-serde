# glib-serde

## Serde support for GLib types in gtk-rs-core

Supports serializing arbitrary types to/from `glib::Variant` using
[serde](https://serde.rs). The main interface is `to_variant` and `from_variant`.

Serializing structs and enums requires an implementation of `VariantType`, which should be
automatically derived:

```rust
#[derive(Debug, PartialEq, Eq)]
#[derive(glib_serde::VariantType, serde::Serialize, serde::Deserialize)]
struct MyStruct {
    id: i32,
    name: String
}

let s = MyStruct {
    id: 1,
    name: String::from("Item")
};
let variant = glib_serde::to_variant(&s).unwrap();
assert_eq!(variant.type_(), "(is)");
assert_eq!(variant.to_string(), "(1, 'Item')");
let value: MyStruct = glib_serde::from_variant(&variant).unwrap();
assert_eq!(s, value);
```

Additional derive macros are provided to serialize/deserialize GLib enum and flag types:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, glib::Enum)]
#[derive(glib_serde::VariantType, glib_serde::EnumSerialize, glib_serde::EnumDeserialize)]
#[enum_type(name = "Direction")]
enum Direction {
    North = 1,
    East = 2,
    South = 3,
    West = 4,
}

let variant = glib_serde::to_variant(&Direction::South).unwrap();
assert_eq!(variant.type_(), "s");
assert_eq!(variant.to_string(), "'south'");
let value: Direction = glib_serde::from_variant(&variant).unwrap();
assert_eq!(value, Direction::South);
```

## Tracking the various forms of serialization

- `to_variant<T: Serialize + VariantType>(T) -> Variant`: requires type info via VariantType
- `from_variant<T: Deserialize>(Variant) -> T`: deserialize any data into serde
- `Serialize for crate::Variant`: as tuple struct (type, data)
- `Deserialize for crate::Variant`: from tuple struct (type, data)

Gaps:
- `to_variant` shouldn't need any type info up-front
- `Serialize for Variant`: in the same fashion as from_variant (from Variant to Serializer)
- `Deserialize for Variant`: like to_variant, but without type info (from Deserializer to Variant)
- `impl IntoDeserializer for Variant`: from_variant basically
- `derive(VariantType)` should impl `ToVariant`/`FromVariant`, no?
