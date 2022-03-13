use serde::{
    Deserialize, Deserializer,
    Serialize, Serializer,
    de,
    ser::SerializeTupleStruct,
};
use glib::{Variant, Bytes};
use std::borrow::Cow;
use std::fmt;

newtype_wrapper! {
    #[derive(Debug, Clone, Hash)]
    @[PartialOrd PartialOrd] @[PartialEq PartialEq]
    @[Display Display] @[FromStr FromStr]
    @[GlibVariantWrapper GlibVariantWrapper]
    @[StaticVariantType StaticVariantType] @[ToVariant ToVariant] @[FromVariant FromVariant]
    pub SerializedVariant(Variant | Variant) into_variant
}

pub type SerializedData<'a> = (crate::VariantType<'a>, Cow<'a, [u8]>);

pub const NEWTYPE_NAME: &'static str = "SerializedVariant";

pub fn deserialize<'a, T: From<Variant>, D: Deserializer<'a>>(deserializer: D) -> Result<T, D::Error> {
    SerializedVariant::deserialize(deserializer).map(|v| v.into_variant().into())
}

pub fn serialize<T: AsRef<Variant>, S: Serializer>(var: T, serializer: S) -> Result<S::Ok, S::Error> {
    SerializedVariant::from(var.as_ref()).serialize(serializer)
}

impl<'a> Serialize for SerializedVariant<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser = serializer.serialize_tuple_struct(NEWTYPE_NAME, 2)?;
        ser.serialize_field(&crate::VariantType::from(self.type_()))?;
        let v = maybe_byteswap(Cow::Borrowed(self));
        ser.serialize_field(v.data())?;
        ser.end()
    }
}

impl<'de> Deserialize<'de> for SerializedVariant<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SerializedDataVisitor;
        impl<'de> de::Visitor<'de> for SerializedDataVisitor {
            type Value = SerializedVariant<'de>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({}, Bytes)", NEWTYPE_NAME, crate::variant_type::NEWTYPE_NAME)
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let ty: crate::VariantType<'de> = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let bytes: Cow<'de, [u8]> = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                let bytes = Bytes::from_owned(bytes.into_owned());
                // TODO: it's not obvious how to actually validate the data..? .normal_form()?
                let untrusted = Variant::from_bytes_with_type(&bytes, &ty);
                let variant = maybe_byteswap(Cow::Owned(untrusted))
                    .into_owned();
                Ok(variant.into())
            }
        }
        deserializer.deserialize_newtype_struct(NEWTYPE_NAME, SerializedDataVisitor)
    }
}

fn maybe_byteswap<'a>(variant: Cow<'a, Variant>) -> Cow<'a, Variant> {
    match () {
        #[cfg(target_endian = "little")]
        () => variant,
        #[cfg(target_endian = "big")]
        () => Cow::Owned(v.byteswap()),
    }
}
