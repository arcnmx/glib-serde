// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use glib::{StaticVariantType, VariantTy};
use glib_serde::{from_variant, prelude::*, to_variant, Variant};
use std::{collections::HashMap, ops::Deref};

#[derive(Debug, PartialEq, Eq, glib::Variant, serde::Serialize, serde::Deserialize)]
struct MyNewtypeStruct(i32);

#[derive(Debug, PartialEq, Eq, glib::Variant, serde::Serialize, serde::Deserialize)]
struct MyTupleStruct(u64, String, Option<String>);

#[derive(Debug, PartialEq, glib::Variant, serde::Serialize, serde::Deserialize)]
struct MyStruct {
    id: u32,
    position: f64,
    my_tuple: MyTupleStruct,
}

#[derive(glib::Variant, serde::Serialize, serde::Deserialize)]
struct MyWrapperStruct {
    ftype: glib_serde::EnumValue<gio::FileType>,
    ftype_num: glib_serde::EnumReprValue<gio::FileType>,
    cond: glib_serde::FlagsValue<glib::IOCondition>,
    cond_num: glib_serde::FlagsReprValue<glib::IOCondition>,
    path: glib_serde::ObjectPath,
    sig: glib_serde::Signature,
    #[serde(with = "glib_serde::variant")]
    var: glib::Variant,
    #[serde(with = "glib_serde::variant_dict")]
    dict: glib::VariantDict,
}

#[test]
fn serialize_structs() {
    assert_eq!(*MyNewtypeStruct::static_variant_type(), "i");
    assert_eq!(*MyTupleStruct::static_variant_type(), "(tsms)");
    assert_eq!(*MyStruct::static_variant_type(), "(ud(tsms)(sv)m(sv))");
    assert_eq!(*MyWrapperStruct::static_variant_type(), "(sisuogva{sv})");

    let variant = to_variant(&MyNewtypeStruct(52)).unwrap();
    assert_eq!(variant.type_(), "i");
    assert_eq!(variant.to_string(), "52");

    let variant = to_variant(&MyTupleStruct(3, "hello".into(), Some("world".into()))).unwrap();
    assert_eq!(variant.type_(), "(tsms)");
    assert_eq!(variant.to_string(), "(3, 'hello', 'world')");

    let variant = to_variant(&MyStruct {
        id: 3050,
        position: -182.5,
        my_tuple: MyTupleStruct(99, "Foo".into(), None),
    })
    .unwrap();
    assert_eq!(variant.type_(), "(ud(tsms)(sv)m(sv))");
    assert_eq!(
        variant.to_string(),
        "(\
            3050, \
            -182.5, \
            (99, 'Foo', nothing), \
            ('StructVariant', <([int16 7, 6, 5], {int64 -100: 'Goodbye'})>), \
            ('UnitVariant', <()>)\
        )"
    );

    let variant = to_variant(&MyWrapperStruct {
        ftype: gio::FileType::Special.into(),
        ftype_num: gio::FileType::Special.into(),
        cond: glib::IOCondition::IN.into(),
        cond_num: glib::IOCondition::IN.into(),
        path: "/org/glib_serde/test".parse().unwrap(),
        sig: MyNewtypeStruct::static_variant_type()
            .deref()
            .try_into()
            .unwrap(),
        var: (0i32, 1i32).serialize_to_variant().into(),
        dict: {
            let dict = glib_serde::VariantDict::new(None);
            dict.insert("hello", &"world");
            dict
        },
    })
    .unwrap();
    assert_eq!(variant.type_(), "(sisuogva{sv})");
    assert_eq!(
        variant.to_string(),
        "(\
        'special', \
        4, \
        'in', \
        1, \
        '/org/glib_serde/test', \
        'i', \
        <(0, 1)>, \
        {'hello': <'world'>}\
    )"
    );
}

#[test]
fn deserialize_structs() {
    let s = "52";
    let value: MyNewtypeStruct = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyNewtypeStruct(52));

    let s = "(uint64 3, 'hello', just 'world')";
    let value: MyTupleStruct = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(
        value,
        MyTupleStruct(3, "hello".into(), Some("world".into()))
    );

    let s = "(\
        uint32 3050, \
        -182.5, \
        (uint64 99, 'Foo', @ms nothing), \
        ('StructVariant', <([int16 7, 6, 5], {int64 -100: 'Goodbye'})>), \
        just ('UnitVariant', <()>)\
    )";
    let value: MyStruct = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(
        value,
        MyStruct {
            id: 3050,
            position: -182.5,
            my_tuple: MyTupleStruct(99, "Foo".into(), None),
        }
    );

    let s = "(\
        'special', \
        4, \
        'in', \
        uint32 1, \
        objectpath '/org/glib_serde/test', \
        signature '(istxa{ys}as)', \
        <(0, 1)>, \
        {'hello': <'world'>}\
    )";
    let value: MyWrapperStruct = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.ftype.value(), gio::FileType::Special);
    assert_eq!(value.ftype_num.value(), gio::FileType::Special);
    assert_eq!(value.cond.value(), glib::IOCondition::IN);
    assert_eq!(value.cond_num.value(), glib::IOCondition::IN);
    assert_eq!(value.path.as_str(), "/org/glib_serde/test");
    assert_eq!(value.sig.as_str(), "(istxa{ys}as)");
    assert_eq!(value.var.type_(), "(ii)");
    assert_eq!(value.var.to_string(), "(0, 1)");
    assert_eq!(value.dict.lookup_value("hello", None).unwrap().type_(), "s");
    assert_eq!(
        value
            .dict
            .lookup_value("hello", None)
            .unwrap()
            .str()
            .unwrap(),
        "world"
    );
}
