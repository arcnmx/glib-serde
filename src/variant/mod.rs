use serde::{
    Serialize, Deserialize,
    ser, de
};
use crate::{PrettyVariant, SerializedVariant};

newtype_wrapper! {
    #[derive(Debug, Clone, Hash)]
    @[PartialOrd PartialOrd] @[PartialEq PartialEq]
    @[Display Display] @[FromStr FromStr]
    @[GlibVariantWrapper GlibVariantWrapper] @[FromGlibVariantWrapper(AnyVariant, SerializedVariant, PrettyVariant)]
    @[StaticVariantType StaticVariantType] @[ToVariant ToVariant] @[FromVariant FromVariant]
    pub Variant(glib::Variant | glib::Variant) into_variant
}

pub fn deserialize<'a, T: From<glib::Variant>, D: de::Deserializer<'a>>(deserializer: D) -> Result<T, D::Error> {
    Variant::deserialize(deserializer).map(|v| v.into_variant().into())
}

pub fn serialize<T: AsRef<glib::Variant>, S: ser::Serializer>(var: T, serializer: S) -> Result<S::Ok, S::Error> {
    Variant::from(var.as_ref()).serialize(serializer)
}

impl<'a> Serialize for Variant<'a> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            PrettyVariant::from(self).serialize(serializer)
        } else {
            SerializedVariant::from(self).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Variant<'de> {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            PrettyVariant::deserialize(deserializer).map(Into::into)
        } else {
            SerializedVariant::deserialize(deserializer).map(Into::into)
        }
    }
}
