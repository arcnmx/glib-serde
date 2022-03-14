use serde::{
    Deserialize, Deserializer,
    Serialize, Serializer,
    de,
};
use glib::Variant;
use std::fmt;

newtype_wrapper! {
    #[derive(Debug, Clone, Hash)]
    @[PartialOrd PartialOrd] @[PartialEq PartialEq]
    @[Display Display] @[FromStr FromStr]
    @[GlibVariantWrapper GlibVariantWrapper] @[FromGlibVariantWrapper(Variant, AnyVariant, SerializedVariant)]
    @[StaticVariantType StaticVariantType] @[ToVariant ToVariant] @[FromVariant FromVariant]
    pub PrettyVariant(Variant | Variant) into_variant
}

pub const NEWTYPE_NAME: &'static str = "glib_serde::PrettyVariant";

pub fn deserialize<'a, T: From<Variant>, D: Deserializer<'a>>(deserializer: D) -> Result<T, D::Error> {
    PrettyVariant::deserialize(deserializer).map(|v| v.into_variant().into())
}

pub fn serialize<T: AsRef<Variant>, S: Serializer>(var: T, serializer: S) -> Result<S::Ok, S::Error> {
    PrettyVariant::from(var.as_ref()).serialize(serializer)
}

impl<'a> Serialize for PrettyVariant<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let pretty = self.print(true);
        serializer.serialize_newtype_struct(NEWTYPE_NAME, pretty.as_str())
    }
}

impl<'de> Deserialize<'de> for PrettyVariant<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PrettyDataVisitor;
        impl<'de> de::Visitor<'de> for PrettyDataVisitor {
            type Value = Variant;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Variant text representation")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse()
                    .map_err(de::Error::custom)
            }
        }
        deserializer.deserialize_newtype_struct(NEWTYPE_NAME, PrettyDataVisitor)
            .map(Self::from)
    }
}
