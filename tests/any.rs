use glib_serde::{AnyVariant, VariantType, from_variant, to_variant};
use glib::ToVariant;
use serde::{Serialize, Deserialize};

#[derive(Debug, VariantType, Serialize, Deserialize, PartialOrd, PartialEq)]
struct TestDataVariant {
    numeric: u8,
    array: Vec<(String, u64)>,
    string: String,
    explicit_var: glib_serde::Variant,
}

impl ToVariant for TestDataVariant {
    fn to_variant(&self) -> glib::Variant {
        to_variant(self).unwrap()
    }
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
        dict.insert("explicit_var", explicit_var);
        dict.end()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
struct TestData {
    data: TestDataVariant,
    #[serde(with = "glib_serde::any")]
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

    let variant = to_variant(&data.data).unwrap();
    println!("tovariant: {:?}", variant);
    assert_eq!(variant, data.data.variant().to_variant());

    let fromvariant: TestDataVariant = from_variant(&variant).unwrap();
    println!("fromvariant: {:?}", fromvariant);
    assert_eq!(data.data, fromvariant);

    let fromvariant: TestData = from_variant(&data.variant().to_variant()).unwrap();
    println!("fromvariant: {:?}", fromvariant);
    assert_eq!(data, fromvariant);

    let anyfromjson: AnyVariant = serde_json::from_str(&json).unwrap();
    println!("anyfromjson: {:?}", anyfromjson);
    // TODO: assert_eq!(anyfromjson.inner(), &data.vardict());
    // - must support deserializing as arrays rather than tuples
    // - doesn't support deserializing variant tuples to boxed tuples
    // - normal_form, vardict key sorting, etc

    /*let fromany = TestData::deserialize(anyfromjson.into_deserializer()).unwrap();
    assert_eq!(fromany, data); // TODO: this requires a deserializer with `is_human_readable` set to be equivalent to JSON
    */

    let anytojson = serde_json::to_string(&anyfromjson).unwrap();
    assert_eq!(anytojson, json);
}
