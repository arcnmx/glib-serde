use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
    ser::{self, SerializeSeq, Error as _, SerializeMap, SerializeTuple, SerializeTupleStruct},
};
use glib::{Variant, ToVariant, VariantClass, VariantTy, VariantType};
use std::fmt;
use crate::util;

pub mod deserializer;
pub use deserializer::Deserializer;

pub mod serializer;
pub use serializer::Serializer;

newtype_wrapper! {
    #[derive(Debug, Clone, Hash)]
    @[PartialOrd PartialOrd] @[PartialEq PartialEq]
    @[Display Display] @[FromStr FromStr]
    @[GlibVariantWrapper GlibVariantWrapper] @[FromGlibVariantWrapper(Variant, SerializedVariant, PrettyVariant)]
    @[StaticVariantType StaticVariantType] @[ToVariant ToVariant] @[FromVariant FromVariant]
    pub AnyVariant(Variant | Variant) into_variant
}

pub fn deserialize<'a, T: From<Variant>, D: de::Deserializer<'a>>(deserializer: D) -> Result<T, D::Error> {
    AnyVariant::deserialize(deserializer).map(|v| v.into_variant().into())
}

pub fn serialize<T: AsRef<Variant>, S: ser::Serializer>(var: T, serializer: S) -> Result<S::Ok, S::Error> {
    AnyVariant::from(var.as_ref()).serialize(serializer)
}

impl<'a> AnyVariant<'a> {
    pub fn from_serde<S: Serialize>(value: S) -> Result<Self, crate::Error> {
        let serializer = Serializer::default();

        value.serialize(serializer)
            .map(From::from)
    }

    pub fn to_serde<'de, D: Deserialize<'de>>(&'de self) -> Result<D, crate::Error> {
        let deserializer = Deserializer::from(self);

        D::deserialize(deserializer)
    }
}

impl<'a, 'de> de::IntoDeserializer<'de, crate::Error> for &'de AnyVariant<'a> where
    'de: 'a
{
    type Deserializer = Deserializer<'a>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.into()
    }
}

impl<'a> Serialize for AnyVariant<'a> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.classify() {
            VariantClass::Boolean => self.get::<bool>().unwrap().serialize(serializer),
            VariantClass::Byte => self.get::<u8>().unwrap().serialize(serializer),
            VariantClass::Int16 => self.get::<i16>().unwrap().serialize(serializer),
            VariantClass::Uint16 => self.get::<u16>().unwrap().serialize(serializer),
            VariantClass::Int32 => self.get::<i32>().unwrap().serialize(serializer),
            VariantClass::Uint32 => self.get::<u32>().unwrap().serialize(serializer),
            VariantClass::Int64 => self.get::<i64>().unwrap().serialize(serializer),
            VariantClass::Uint64 => self.get::<u64>().unwrap().serialize(serializer),
            VariantClass::Double => self.get::<f64>().unwrap().serialize(serializer),
            VariantClass::String | VariantClass::ObjectPath | VariantClass::Signature =>
                self.get::<String>().unwrap().serialize(serializer),
            VariantClass::Variant => serializer.serialize_newtype_struct("Variant", &AnyVariant::from(&self.as_variant().unwrap())),
            VariantClass::Maybe if self.n_children() == 1 =>
                serializer.serialize_some(&AnyVariant::from(&self.child_value(0))),
            VariantClass::Maybe =>
                serializer.serialize_none(),
            VariantClass::Array if self.type_().element().is_dict_entry() => {
                let is_vardict = self.type_() == VariantTy::VARDICT;
                let len = self.n_children();
                let mut state = serializer.serialize_map(Some(len))?;
                for kv in self.iter() {
                    debug_assert!(kv.type_().is_dict_entry());
                    let (key, value) = (kv.child_value(0), kv.child_value(1));
                    state.serialize_entry(&AnyVariant::from(&key),
                        &if is_vardict { AnyVariant::from(value.as_variant().unwrap()) } else { AnyVariant::from(&value) }
                    )?;
                }
                state.end()
            },
            VariantClass::Array if self.type_().element() == VariantTy::BYTE =>
                serializer.serialize_bytes(&self.data_as_bytes()),
            VariantClass::Array => {
                let len = self.n_children();
                let mut state = serializer.serialize_seq(Some(len))?;
                for i in 0..len {
                    let element = self.child_value(i);
                    state.serialize_element(&AnyVariant::from(&element))?;
                }
                state.end()
            },
            VariantClass::Tuple => {
                let len = self.n_children();
                let mut state = serializer.serialize_tuple(len)?;
                for i in 0..len {
                    let element = self.child_value(i);
                    state.serialize_element(&AnyVariant::from(&element))?;
                }
                state.end()
            },
            VariantClass::DictEntry => {
                let name = VariantTy::DICT_ENTRY.as_str(); // TODO
                let mut state = serializer.serialize_tuple_struct(name, 2)?;
                state.serialize_field(&AnyVariant::from(&self.child_value(0)))?;
                state.serialize_field(&AnyVariant::from(&self.child_value(1)))?;
                state.end()
            },
            _ => Err(S::Error::custom("expected Variant")),
        }
    }
}

impl<'a, 'de> Deserialize<'de> for AnyVariant<'a> {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VariantVisitor;

        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = Variant;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("glib::Variant")
            }

            fn visit_newtype_struct<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
                // TODO: is this remotely correct?
                deserializer.deserialize_any(Self)
            }

            fn visit_i8<E: de::Error>(self, v: i8) -> Result<Self::Value, E> {
                Ok((v as i16).to_variant())
            }

            fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_i16<E: de::Error>(self, v: i16) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_u16<E: de::Error>(self, v: u16) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_i32<E: de::Error>(self, v: i32) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_f32<E: de::Error>(self, v: f32) -> Result<Self::Value, E> {
                Ok((v as f64).to_variant())
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(Variant::from_none(VariantTy::VARIANT))
            }

            fn visit_some<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
                deserialize(deserializer).map(|v| Variant::from_some(&v))
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                Ok(v.to_variant())
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(value) = seq.next_element::<AnyVariant<'de>>()? {
                    values.push(value);
                }
                Ok(util::VariantList::with_variants(&values, VariantTy::ANY)
                    .to_tuple_or_array())
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((key, value)) = map.next_entry::<AnyVariant, AnyVariant>()? {
                    entries.push((key, value));
                }

                let keys = entries.iter().map(|&(ref k, _)| k.inner());
                let keys = util::VariantList::with_variants(keys, VariantTy::STRING);

                let values = entries.iter().map(|&(_, ref v)| v.inner());
                let values = util::VariantList::with_variants(values, VariantTy::VARIANT);

                let ty = VariantType::new_dict_entry(keys.elem_type(), values.elem_type());
                let entries = keys.iter()
                    .zip(values.iter())
                    .map(|(ref k, ref v)| Variant::from_dict_entry(k, v));
                Ok(Variant::array_from_iter_with_type(&ty, entries))
            }
        }

        deserializer.deserialize_any(VariantVisitor)
            .map(Self::from)
    }
}
