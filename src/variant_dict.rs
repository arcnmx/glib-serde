// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use serde::{
    Deserialize, Deserializer,
    Serialize, Serializer,
    de, ser::SerializeMap,
};
use crate::AnyVariant;

newtype_wrapper! {
    /// Wrapper type for [`glib::VariantDict`].
    #[derive(Clone, Default)]
    @[StaticVariantType StaticVariantType] @[ToVariant ToVariant] @[FromVariant FromVariant]
    pub VariantDict(glib::VariantDict | glib::VariantDict) into_variant_dict
}

impl<'a> VariantDict<'a> {
    pub fn new(from_asv: Option<&glib::Variant>) -> Self {
        glib::VariantDict::new(from_asv).into()
    }
}

impl<'a, 'v, T: crate::traits::GlibVariantWrapper<'v>> From<T> for VariantDict<'a> {
    fn from(other: T) -> Self {
        Self::new(Some(other.variant_ref()))
    }
}

impl<'a> Serialize for VariantDict<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = self.end();
        let count = v.n_children();
        let mut seq = serializer.serialize_map(Some(count))?;
        for i in 0..count {
            let entry = v.child_value(i);
            let key = entry.child_value(0);
            let key = key.str().unwrap();
            let value = entry.child_value(1).as_variant().unwrap();
            seq.serialize_entry(&key, &AnyVariant::from(value))?;
        }
        seq.end()
    }
}

impl<'a, 'de> Deserialize<'de> for VariantDict<'a> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor;

        impl<'de> de::Visitor<'de> for MapVisitor {
            type Value = glib::VariantDict;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VariantDict map")
            }

            fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let dict = glib::VariantDict::new(None);

                while let Some((key, value)) = map.next_entry::<String, AnyVariant>()? {
                    dict.insert_value(&key, &value);
                }

                Ok(dict)
            }
        }

        deserializer.deserialize_map(MapVisitor).map(Into::into)
    }
}

pub fn deserialize<'a, T: From<glib::VariantDict>, D: Deserializer<'a>>(deserializer: D) -> Result<T, D::Error> {
    VariantDict::deserialize(deserializer).map(|v| v.into_variant_dict().into())
}

pub fn serialize<T: AsRef<glib::VariantDict>, S: Serializer>(var: T, serializer: S) -> Result<S::Ok, S::Error> {
    VariantDict::from(var.as_ref()).serialize(serializer)
}

impl<'a, 'de> de::IntoDeserializer<'de, crate::Error> for VariantDict<'a> {
    type Deserializer = crate::any_variant::Deserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.end().into()
    }
}

impl<'a, 'r, 'de> de::IntoDeserializer<'de, crate::Error> for &'r VariantDict<'a> {
    type Deserializer = crate::any_variant::Deserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.end().into()
    }
}
