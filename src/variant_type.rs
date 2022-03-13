use serde::{
    Deserialize, Deserializer,
    Serialize, Serializer,
    de,
};
use std::borrow::Cow;
use std::fmt;

pub const NEWTYPE_NAME: &'static str = "VariantType";

newtype_wrapper! {
    #[derive(Debug, Clone, Hash)]
    @[PartialEq PartialEq]
    @[Display Display] @[FromStr FromStr]
    pub VariantType(glib::VariantTy | glib::VariantType) into_type
}

pub fn deserialize<'a, T: From<Cow<'a, glib::VariantTy>>, D: Deserializer<'a>>(deserializer: D) -> Result<T, D::Error> {
    VariantType::deserialize(deserializer).map(|v| v.into_inner().into())
}

pub fn serialize<T: AsRef<glib::VariantTy>, S: Serializer>(var: T, serializer: S) -> Result<S::Ok, S::Error> {
    VariantType::from(var.as_ref()).serialize(serializer)
}

impl<'a> Serialize for VariantType<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct(NEWTYPE_NAME, self.as_str())
    }
}

impl<'de> Deserialize<'de> for VariantType<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VariantTypeVisitor;
        impl<'de> de::Visitor<'de> for VariantTypeVisitor {
            type Value = VariantType<'de>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a GVariant VariantType string")
            }

            fn visit_string<E: de::Error>(self, ty: String) -> Result<Self::Value, E> {
                glib::VariantType::from_string(ty).map(Into::into).map_err(de::Error::custom)
            }

            fn visit_str<E: de::Error>(self, ty: &str) -> Result<Self::Value, E> {
                glib::VariantType::new(ty).map(Into::into).map_err(de::Error::custom)
            }

            fn visit_borrowed_str<E: de::Error>(self, ty: &'de str) -> Result<Self::Value, E> {
                glib::VariantTy::new(ty).map(Into::into).map_err(de::Error::custom)
            }
        }
        deserializer.deserialize_newtype_struct(NEWTYPE_NAME, VariantTypeVisitor)
    }
}
