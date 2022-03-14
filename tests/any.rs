use glib_serde::{AnyVariant, any_variant};
use glib::{ToVariant, StaticVariantType};
use serde::{Serialize, Deserialize, de::IntoDeserializer};

#[derive(Debug, glib::Variant, Serialize, Deserialize, PartialOrd, PartialEq)]
struct TestDataVariant {
    numeric: u8,
    array: Vec<(String, u64)>,
    string: String,
    #[serde(with = "glib_serde::any_variant")]
    explicit_var: glib::Variant,
}

impl TestDataVariant {
    fn new() -> Self {
        Self {
            numeric: 1,
            array: vec![
                ("hi".into(), 2),
                ("hey".into(), 3),
            ],
            string: "hello".into(),
            explicit_var: 4u64.to_variant().into(), // TODO: u32 should work, but `glib_serde::Variant as Deserialize` doesn't preserve types properly
        }
    }

    fn variant(&self) -> (&u8, &[(String, u64)], &str, &glib::Variant) {
        (&self.numeric, &self.array, &self.string, &self.explicit_var)
    }

    fn vardict(&self) -> glib::Variant {
        let dict = glib::VariantDict::new(None);
        let explicit_var: &glib::Variant = &self.explicit_var;
        dict.insert("numeric", &(self.numeric as u64));
        dict.insert("array", &self.array);
        dict.insert("string", &self.string);
        if false {
            dict.insert("explicit_var", explicit_var); // can't distinguish boxed variants properly .-.
        } else {
            dict.insert_value("explicit_var", explicit_var);
        }
        dict.end()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
struct TestData {
    data: TestDataVariant,
    #[serde(with = "glib_serde::any_variant")]
    transparent_var: glib::Variant,
    transparent_any: AnyVariant<'static>,
}

impl TestData {
    fn new() -> Self {
        Self {
            data: TestDataVariant::new(),
            transparent_var: 5u64.to_variant(),
            transparent_any: "test".to_variant().into(),
        }
    }

    fn variant(&self) -> ((&u8, &[(String, u64)], &str, &glib::Variant), &glib::Variant, &glib::Variant) {
        let data = self.data.variant();
        (data, &self.transparent_var, &self.transparent_any)
    }

    fn vardict(&self) -> glib::Variant {
        let dict = glib::VariantDict::new(None);
        let transparent_any: &glib::Variant = &self.transparent_any;
        dict.insert_value("data", &self.data.vardict());
        dict.insert_value("transparent_var", &self.transparent_var);
        dict.insert_value("transparent_any", transparent_any);
        dict.end()
    }
}

#[test]
fn serde_any() {
    let data = TestData::new();
    println!("{:?}", data);
    let json = serde_json::to_string(&data).unwrap();
    println!("{}", json);

    let fromjson: TestData = serde_json::from_str(&json).unwrap();
    println!("fromjson: {:?}", fromjson);
    assert_eq!(data, fromjson);

    let variant = data.data.serialize(any_variant::Serializer::new(Some(&TestDataVariant::static_variant_type()))).unwrap();
    println!("tovariant: {:?}", variant);
    assert_eq!(variant, data.data.variant().to_variant());

    let fromvariant: TestDataVariant = AnyVariant::from(variant).to_serde().unwrap();
    println!("fromvariant: {:?}", fromvariant);
    assert_eq!(data.data, fromvariant);

    let fromvariant: TestData = AnyVariant::from(data.variant().to_variant()).to_serde().unwrap();
    println!("fromvariant: {:?}", fromvariant);
    assert_eq!(data, fromvariant);

    let anyfromjson: AnyVariant = serde_json::from_str(&json).unwrap();
    println!("anyfromjson: {:?}", anyfromjson);
    assert_eq!(anyfromjson.inner(), &data.vardict());

    let fromany = TestData::deserialize(anyfromjson.into_deserializer()).unwrap();
    assert_eq!(fromany, data);

    let anytojson = serde_json::to_string(&anyfromjson).unwrap();
    assert_eq!(anytojson, json);
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
enum SimpleEnum {
    A,
    B,
}

#[test]
fn serde_enum() {
    let a: SimpleEnum = AnyVariant::from("A".to_variant()).to_serde().unwrap();
    assert_eq!(SimpleEnum::A, a);
}
