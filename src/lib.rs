// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

//! # Serde support for GLib types in gtk-rs-core
//!
//! Supports serializing arbitrary types to/from [`glib::Variant`](struct@glib::Variant) using
//! [serde](https://serde.rs). The main interface is [`to_variant`] and [`from_variant`].
//!
//! Serializing structs and enums requires an implementation of `VariantType`, which should be
//! automatically derived:
//!
//! ```
//! #[derive(Debug, PartialEq, Eq)]
//! #[derive(glib::Variant, serde::Serialize, serde::Deserialize)]
//! struct MyStruct {
//!     id: i32,
//!     name: String
//! }
//!
//! let s = MyStruct {
//!     id: 1,
//!     name: String::from("Item")
//! };
//! let variant = glib_serde::to_variant(&s).unwrap();
//! assert_eq!(variant.type_(), "(is)");
//! assert_eq!(variant.to_string(), "(1, 'Item')");
//! let value: MyStruct = glib_serde::from_variant(&variant).unwrap();
//! assert_eq!(s, value);
//! ```

pub use glib;

mod enums;
pub use enums::*;
mod error;
pub use error::*;
mod flags;
pub use flags::*;
mod object_path;
pub use object_path::*;
mod signature;
pub use signature::*;
mod variant;
pub use variant::{from_variant, to_variant, Variant};
mod variant_builder;
use variant_builder::*;
mod variant_dict;
pub use variant_dict::*;
mod variant_type;
pub use variant_type::*;

/// Extension traits for variants and serializable types.
pub mod prelude {
    pub use super::variant::GlibVariantExt;

    /// Alternative to [`ToVariant`](glib::ToVariant) for [`serde::Serialize`] types.
    pub trait ToVariantExt {
        fn serialize_to_variant(&self) -> glib::Variant;
    }

    impl<T: serde::Serialize + super::VariantType> ToVariantExt for T {
        fn serialize_to_variant(&self) -> glib::Variant {
            super::to_variant(self).unwrap()
        }
    }

    /// Alternative to [`FromVariant`](glib::FromVariant) for [`serde::Deserialize`] types.
    pub trait FromVariantExt<'t, T> {
        fn deserialize_from_variant(variant: &'t glib::Variant) -> Option<T>;
    }

    impl<'de, T: serde::Deserialize<'de>> FromVariantExt<'de, T> for T {
        fn deserialize_from_variant(variant: &'de glib::Variant) -> Option<T> {
            super::from_variant(variant).ok()
        }
    }
}
