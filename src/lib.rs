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
//! use glib::ToVariant;
//!
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
//! let variant = s.to_variant();
//! assert_eq!(variant.type_(), "(is)");
//! assert_eq!(variant.to_string(), "(1, 'Item')");
//! let value: MyStruct = glib_serde::from_variant(&variant).unwrap();
//! assert_eq!(s, value);
//! ```

pub use glib;

#[macro_use]
mod macros;

pub mod any_variant;
pub use any_variant::AnyVariant;

pub mod serialized;
pub use serialized::SerializedVariant;

pub mod pretty;
pub use pretty::PrettyVariant;

pub mod variant_type;
pub use variant_type::VariantType;

pub mod variant_dict;
pub use variant_dict::VariantDict;

pub(crate) mod util;

/*mod enums;
pub use enums::*;*/
mod error;
pub use error::*;
/*mod flags;
pub use flags::*;
mod object_path;
pub use object_path::*;
mod signature;
pub use signature::*;
mod variant;
pub use variant::{from_variant, to_variant, Variant};
mod variant_builder;
use variant_builder::*;
*/

/// Extension traits for variants and serializable types.
pub mod traits {
    use std::borrow::Cow;

    pub trait GlibVariantExt {
        fn as_serializable(&self) -> super::AnyVariant;

        fn as_serialized(&self) -> super::SerializedVariant;

        fn into_deserializer(&self) -> super::any_variant::Deserializer;
    }

    impl GlibVariantExt for glib::Variant {
        fn as_serializable(&self) -> super::AnyVariant {
            From::from(self)
        }

        fn as_serialized(&self) -> super::SerializedVariant {
            From::from(self)
        }

        fn into_deserializer(&self) -> super::any_variant::Deserializer {
            From::from(self)
        }
    }

    impl GlibVariantExt for glib::VariantDict {
        fn as_serializable(&self) -> super::AnyVariant {
            From::from(self.end())
        }

        fn as_serialized(&self) -> super::SerializedVariant {
            From::from(self.end())
        }

        fn into_deserializer(&self) -> super::any_variant::Deserializer {
            From::from(self.end())
        }
    }

    pub trait GlibVariantWrapper<'v>: Sized {
        // TODO: constructors!

        fn variant_ref(&self) -> &glib::Variant;
        fn into_variant(self) -> glib::Variant;
        fn variant_cow(self) -> Cow<'v, glib::Variant> {
            Cow::Owned(self.into_variant())
        }
    }

    impl GlibVariantWrapper<'static> for glib::Variant {
        fn variant_ref(&self) -> &glib::Variant {
            self
        }

        fn into_variant(self) -> glib::Variant {
            self
        }
    }

    impl<'a> GlibVariantWrapper<'a> for Cow<'a, glib::Variant> {
        fn variant_cow(self) -> Cow<'a, glib::Variant> {
            self
        }

        fn variant_ref(&self) -> &glib::Variant {
            self
        }

        fn into_variant(self) -> glib::Variant {
            self.into_owned()
        }
    }

    impl<'a, 'v, T: GlibVariantWrapper<'v>> GlibVariantWrapper<'a> for &'a T {
        fn variant_cow(self) -> Cow<'a, glib::Variant> {
            Cow::Borrowed(self.variant_ref())
        }

        fn variant_ref(&self) -> &glib::Variant {
            GlibVariantWrapper::variant_ref(*self)
        }

        fn into_variant(self) -> glib::Variant {
            self.variant_ref().clone()
        }
    }

    /// Alternative to [`ToVariant`](glib::ToVariant) for [`serde::Serialize`] types.
    pub trait ToVariantExt {
        fn serialize_to_variant(&self) -> glib::Variant;
    }

    impl<T: serde::Serialize> ToVariantExt for T {
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

pub mod prelude {
    pub use crate::traits::{
        GlibVariantExt as _,
        ToVariantExt as _,
        FromVariantExt as _,
    };
}

pub fn to_variant<S: serde::Serialize>(value: S) -> Result<glib::Variant, Error> {
    AnyVariant::from_serde(value)
        .map(Into::into)
}

pub fn from_variant<'de, D: serde::Deserialize<'de>>(variant: &'de glib::Variant) -> Result<D, Error> {
    serde::Deserialize::deserialize(any_variant::Deserializer::from(variant))
}
