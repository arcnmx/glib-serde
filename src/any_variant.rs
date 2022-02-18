use serde::{
    Deserialize, Deserializer,
    Serialize, Serializer,
    de::{self, Visitor, Error as _, IntoDeserializer},
    ser::{SerializeSeq, Error as _, SerializeMap, SerializeTuple, SerializeTupleStruct},
};
use glib::{Variant, ToVariant, VariantClass, VariantTy, VariantType};
use std::fmt;
use std::borrow::Cow;

newtype_wrapper! {
    pub AnyVariant(glib::Variant) into_variant
}

impl<'a> AnyVariant<'a> {
    pub fn from_serde<S: Serialize>(value: &S) -> Result<Self, crate::Error> {
        let serializer: VariantSerializer = todo!();

        //value.serialize(serializer)
    }
}

impl<'a> Into<crate::Variant> for AnyVariant<'a> { // TODO: From instead!
    fn into(self) -> crate::Variant {
        self.into_variant().into()
    }
}

impl<'a> glib::StaticVariantType for AnyVariant<'a> {
    fn static_variant_type() -> Cow<'static, VariantTy> {
        <glib::Variant as glib::StaticVariantType>::static_variant_type()
    }
}
impl<'a> crate::VariantType for AnyVariant<'a> { }

impl<'a, 'de> IntoDeserializer<'de, crate::Error> for &'de AnyVariant<'a> where
    'de: 'a
{
    type Deserializer = AnyVariant<'a>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.borrowed()
    }
}

impl<'a> Serialize for AnyVariant<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct VariantVisitor;

        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = glib::Variant;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("glib::Variant")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where
                D: Deserializer<'de>,
            {
                todo!()
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
                Ok(glib::Variant::from_none(VariantTy::VARIANT))
            }

            fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
                deserialize(deserializer).map(|v| glib::Variant::from_some(&v))
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
                    values.push(value.into());
                }
                Ok(match list_type(values.iter()) {
                    Some(Some(ty)) => {
                        let ty = ty.to_owned();
                        Variant::array_from_iter_with_type(&ty, values)
                    },
                    None | Some(None) => Variant::tuple_from_iter(values),
                })
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
                /*while let Some((key, value)) = map.next_entry::<AnyVariant, AnyVariant>()? {
                    entries.push((key.to_variant(), value.to_variant()));
                }*/
                while let Some((key, value)) = map.next_entry::<AnyVariant, AnyVariant>()? {
                    // TODO: use dynamic type variant above if all key+value types are consistent
                    entries.push((key, value));
                }
                let key_type = list_type(entries.iter().map(|&(ref k, _)| k.inner()));
                let value_type = list_type(entries.iter().map(|&(_, ref v)| v.inner()));

                let key_type = match key_type {
                    Some(Some(ty)) => Ok(ty),
                    None => Ok(VariantTy::STRING),
                    Some(None) => Err(A::Error::custom("inconsistent map key type")),
                }?;

                Ok(match value_type {
                    Some(Some(value_type)) => {
                        let ty = VariantType::new_dict_entry(&key_type, value_type);
                        Variant::array_from_iter_with_type(&ty, entries.iter().map(|(k, v)|
                            Variant::from_dict_entry(k, v)
                        ))
                    },
                    None | Some(None) => {
                        let ty = VariantType::new_dict_entry(&key_type, VariantTy::VARIANT);
                        Variant::array_from_iter_with_type(&ty, entries.iter().map(|(k, v)|
                            Variant::from_dict_entry(k, &Variant::from_variant(v))
                        ))
                    },
                })
            }
        }

        deserializer.deserialize_any(VariantVisitor)
            .map(Self::from)
    }
}

impl<'a, 'de> Deserializer<'de> for AnyVariant<'a> {
    type Error = crate::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        todo!()
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}

pub fn deserialize<'a, T: From<glib::Variant>, D: Deserializer<'a>>(deserializer: D) -> Result<T, D::Error> {
    AnyVariant::deserialize(deserializer).map(|v| v.into_variant().into())
}

pub fn serialize<'a, T: Into<&'a glib::Variant>, S: Serializer>(var: T, serializer: S) -> Result<S::Ok, S::Error> {
    AnyVariant::from(var.into()).serialize(serializer)
}

fn list_type<'a, I: IntoIterator<Item=&'a Variant>>(vars: I) -> Option<Option<&'a VariantTy>> {
    vars.into_iter()
        .fold(None, |ty, var| match ty {
            None => Some(Some(var.type_())),
            res @ Some(Some(ty)) if ty == var.type_() => res,
            Some(_) => Some(None),
        })
}

pub struct VariantSerializer {
}
