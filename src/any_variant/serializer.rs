use serde::{ser, Serialize};
use glib::{Variant, ToVariant, VariantTy, VariantType, variant::VariantTypeMismatchError};
use crate::{Error, util};

#[derive(Debug, Copy, Clone)]
pub struct Serializer<'t> {
    ty: Option<&'t VariantTy>,
    humanize: bool,
}

impl<'t> Default for Serializer<'t> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<'t> Serializer<'t> {
    pub fn with_type(ty: &'t VariantTy) -> Self {
        Self::new(Some(ty))
    }

    pub fn new(ty: Option<&'t VariantTy>) -> Self {
        Self {
            ty,
            humanize: true,
        }
    }

    pub fn inhuman(self) -> Self {
        Self {
            ty: self.ty,
            humanize: false,
        }
    }

    pub fn change_type(&self, ty: Option<&'t VariantTy>) -> Self {
        Self {
            ty,
            humanize: self.humanize,
        }
    }

    pub fn child_type(&self) -> Result<Option<&'t VariantTy>, Error> {
        match self.ty {
            None => Ok(None),
            Some(ty) if ty == VariantTy::VARIANT => Ok(None),
            Some(ty) => match ty.is_array() || ty.is_maybe() {
                true => Ok(Some(ty.element())),
                false => Err(Error::UnsupportedType(ty.to_owned())),
            },
        }
    }

    pub fn child_type_or_default(&self) -> Result<&'t VariantTy, Error> {
        Ok(match self.child_type()? {
            Some(ty) => ty,
            None => VariantTy::VARIANT,
        })
    }

    pub fn child_serializer(&self) -> Result<Self, Error> {
        self.child_type().map(|ty| self.change_type(ty))
    }

    pub fn check_type<F: FnOnce(&VariantTy) -> bool>(&self, f: F, ty: &VariantTy) -> Result<(), Error> {
        match self.ty {
            Some(expected) if expected == VariantTy::VARIANT => Ok(()),
            Some(expected) if f(expected) => Ok(()),
            Some(expected) => Err(Error::Mismatch(VariantTypeMismatchError::new(ty.to_owned(), expected.to_owned()))),
            None => Ok(()),
        }
    }

    pub fn check_type_is(&self, ty: &VariantTy) -> Result<(), Error> {
        self.check_type(|expected| expected.is_subtype_of(ty), ty)
    }

    pub fn box_if_needed<V: AsRef<Variant> + Into<Variant>>(&self, v: V) -> Result<Variant, Error> {
        Ok(match self.ty {
            Some(ty) if ty == VariantTy::VARIANT => v.as_ref().to_variant(),
            #[cfg(debug_assertions)]
            Some(ty) if !v.as_ref().type_().is_subtype_of(ty) => return Err(Error::Mismatch(
                VariantTypeMismatchError::new(v.as_ref().type_().to_owned(), ty.to_owned())
            )),
            _ => v.into(),
        })
    }
}

impl<'t> ser::Serializer for Serializer<'t> {
    type Ok = Variant;
    type Error = Error;
    type SerializeSeq = BoxedSerializer<'t, SeqSerializer<'t>>;
    //type SerializeTuple = TupleSerializer<'t>;
    type SerializeTuple = BoxedSerializer<'t, SeqSerializer<'t>>;
    type SerializeTupleStruct = Self::SerializeTuple;
    type SerializeTupleVariant = VariantSerializer<'t, Self::SerializeTuple>;
    type SerializeMap = BoxedSerializer<'t, MapSerializer<'t>>;
    type SerializeStruct = BoxedSerializer<'t, StructSerializer<'t>>;
    type SerializeStructVariant = VariantSerializer<'t, Self::SerializeStruct>;

    fn is_human_readable(&self) -> bool {
        self.humanize
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::BOOLEAN)?;
        self.box_if_needed(v.to_variant())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i16(v as i16)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::INT16)?;
        self.box_if_needed(v.to_variant())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::INT32)?;
        self.box_if_needed(v.to_variant())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::INT64)?;
        self.box_if_needed(v.to_variant())
    }

    /*serde::serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            todo!()
            /*let v = v as u128;
            let buf = [(v >> 64) as i64, v as i64];
            Ok(Variant::array_from_fixed_array(&buf))*/
        }
    }*/

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::BYTE)?;
        self.box_if_needed(v.to_variant())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::UINT16)?;
        self.box_if_needed(v.to_variant())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::UINT32)?;
        self.box_if_needed(v.to_variant())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::UINT64)?;
        self.box_if_needed(v.to_variant())
    }

    /*serde::serde_if_integer128! {
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            todo!()
            /*let buf = [(v >> 64) as u64, v as u64];
            Ok(Variant::array_from_fixed_array(&buf))*/
        }
    }*/

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::DOUBLE)?;
        self.box_if_needed(v.to_variant())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::STRING)?;
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.box_if_needed(match self.ty {
            Some(ty) if ty == VariantTy::OBJECT_PATH || ty == VariantTy::SIGNATURE => todo!(),
            None => v.to_variant(),
            Some(ty) if ty == VariantTy::STRING => v.to_variant(),
            Some(ty) if ty == VariantTy::VARIANT => v.to_variant(),
            Some(ty) => return Err(Error::Mismatch(
                VariantTypeMismatchError::new(VariantTy::STRING.to_owned(), ty.to_owned())
            )),
        })
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::BYTE_STRING)?;
        self.box_if_needed(Variant::array_from_fixed_array(v))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::MAYBE)?;
        let ty = self.child_type_or_default()?;
        self.box_if_needed(Variant::from_none(ty))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.check_type_is(VariantTy::MAYBE)?;
        let value = value.serialize(self.child_serializer()?)?;
        self.box_if_needed(Variant::from_some(&value))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::UNIT)?;
        self.box_if_needed(().to_variant())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.check_type_is(VariantTy::UNIT)?;
        self.box_if_needed(().to_variant())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        if (self.is_human_readable() && self.ty != Some(VariantTy::UINT32)) || self.ty == Some(VariantTy::STRING) {
            self.serialize_str(variant)
        } else {
            self.serialize_u32(variant_index)
        }
        /*let (tag, value_ty) = self.variant_tag(variant_index, variant)?;
        if value_ty.is_some() {
            Ok((tag, ().to_variant()).to_variant())
        } else {
            Ok(tag.to_variant())
        }*/
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        /*static OBJECT_PATH_NODE: VariantTypeNode<'static> =
            VariantTypeNode::new_static(VariantTy::OBJECT_PATH);
        static SIGNATURE_NODE: VariantTypeNode<'static> =
            VariantTypeNode::new_static(VariantTy::SIGNATURE);
        match name {
            object_path::STRUCT_NAME => value.serialize(Serializer::new(&OBJECT_PATH_NODE)),
            signature::STRUCT_NAME => value.serialize(Serializer::new(&SIGNATURE_NODE)),
            _ => value.serialize(self),
        }*/
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        VariantSerializer::new(self, value, name, variant_index, variant)
            .and_then(|s| s.serialize())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.check_type(|exp| exp.is_array() || exp.is_tuple() || exp.is_dict_entry(), VariantTy::ARRAY)?;
        //let len = len.unwrap_or_default();
        /*let ty = self.node.type_();
        if ty.is_array() {
            match ty.element().as_str() {
                "y" => Ok(SeqSerializer::new_u8(len)),
                "q" => Ok(SeqSerializer::new_u16(len)),
                "u" => Ok(SeqSerializer::new_u32(len)),
                "t" => Ok(SeqSerializer::new_u64(len)),
                "n" => Ok(SeqSerializer::new_i16(len)),
                "i" => Ok(SeqSerializer::new_i32(len)),
                "x" => Ok(SeqSerializer::new_i64(len)),
                "d" => Ok(SeqSerializer::new_f64(len)),
                "b" => Ok(SeqSerializer::new_bool(len)),
                _ => Ok(SeqSerializer::new(self.node, len)),
            }
        }*/
        SeqSerializer::for_seq(self, len)
            .map(|seq| BoxedSerializer::new(self, seq))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.check_type(|exp| exp.is_tuple() || exp.is_dict_entry() || exp.is_array(), VariantTy::TUPLE)?;

        SeqSerializer::for_tuple(self, Some(len))
            .map(|tup| BoxedSerializer::new(self, tup))
        //TupleSerializer::new(self.ty, None, len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        if name == crate::serialized::NEWTYPE_NAME {
            todo!()
        }
        SeqSerializer::for_tuple(self, Some(len))
            .map(|s| BoxedSerializer::new(self, s))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_tuple(len)
            .and_then(|ser|
                VariantSerializer::new(self, ser, name, variant_index, variant)
            )
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let (k, v) = match self.child_type()? {
            None => (None, None),
            Some(ty) if ty == VariantTy::DICT_ENTRY => (None, None),
            Some(ty) if ty.is_dict_entry() => (Some(ty.key()), Some(ty.value())),
            Some(ty) => return Err(Error::Mismatch(
                VariantTypeMismatchError::new(VariantTy::DICTIONARY.to_owned(), ty.to_owned())
            )),
        };
        Ok(BoxedSerializer::new(self, MapSerializer::new(self.change_type(k), self.change_type(v), len)))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let is_map = self.ty.map(|t| t.is_array() && t.element().is_dict_entry()).unwrap_or_default();
        let is_tuple = self.ty.map(|t| t.is_tuple()).unwrap_or_default();
        if (self.is_human_readable() && !is_tuple) || is_map {
            Ok(StructSerializer::Map(
                // TODO: `name` unused
                // TODO: require vardict as value type? use `self.ty` to influence type selection?
                MapSerializer::new(self.change_type(Some(VariantTy::STRING)), self.change_type(None), Some(len))
            ))
        } else {
            SeqSerializer::for_tuple(self, Some(len))
                .map(StructSerializer::Tuple)
        }.map(|s| BoxedSerializer::new(self, s))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_struct(name, len)
            .and_then(|ser|
                VariantSerializer::new(self, ser, name, variant_index, variant)
            )
    }
}

enum SeqElement<'t> {
    Tuple {
        iter: util::VariantTyTupleIterator<'t>,
        n_items: usize,
        size: Option<usize>,
    },
    Generic {
        elem: Option<&'t VariantTy>,
        size: Option<usize>,
    },
}

impl<'t> SeqElement<'t> {
    fn new(ty: Option<&'t VariantTy>, size: Option<usize>) -> Result<Self, Error> {
        Ok(match ty {
            None =>
                SeqElement::Generic {
                    elem: None,
                    size,
                },
            Some(ty) if ty == VariantTy::TUPLE || ty == VariantTy::ARRAY || ty == VariantTy::VARIANT =>
                SeqElement::Generic {
                    elem: None,
                    size,
                },
            Some(ty) if ty.is_tuple() || ty.is_dict_entry() =>
                SeqElement::Tuple {
                    n_items: match (size, ty.n_items()) {
                        (Some(size), n_items) if size != n_items => return Err(Error::LengthMismatch {
                            actual: size,
                            expected: n_items,
                        }),
                        (_, n_items) => n_items,
                    },
                    size,
                    iter: util::VariantTyTupleIterator::new(ty)?,
                },
            Some(ty) if ty.is_array() =>
                SeqElement::Generic {
                    elem: Some(ty.element()),
                    size,
                },
            Some(ty) => return Err(Error::Mismatch(
                VariantTypeMismatchError::new(ty.to_owned(), VariantTy::TUPLE.to_owned())
            )),
        })
    }

    fn type_(&mut self) -> Result<Option<&'t VariantTy>, Error> {
        match self {
            SeqElement::Tuple { iter, n_items, size } => iter.next()
                .ok_or_else(|| Error::LengthMismatch {
                    actual: size.unwrap_or_default().max(*n_items + 1),
                    expected: *n_items,
                }).map(Some),
            SeqElement::Generic { elem, .. } => Ok(elem.clone()),
        }
    }

    fn end(self, len: usize) -> Result<(), Error> {
        match self {
            SeqElement::Tuple { iter, n_items, size } => match iter.len() {
                0 => Ok(()),
                left => Err(Error::LengthMismatch {
                    actual: n_items - left,
                    expected: n_items,
                }),
            },
            SeqElement::Generic { size, .. } => match size {
                Some(size) if size != len => Err(Error::LengthMismatch {
                    actual: len,
                    expected: size,
                }),
                _ => Ok(()),
            },
        }
    }
}

pub struct SeqSerializer<'t> {
    ser: Serializer<'t>,
    elem: SeqElement<'t>,
    variants: Vec<Variant>,
    /*U8 {
        values: Vec<u8>,
    },
    U16 {
        values: Vec<u16>,
    },
    U32 {
        values: Vec<u32>,
    },
    U64 {
        values: Vec<u64>,
    },
    I16 {
        values: Vec<i16>,
    },
    I32 {
        values: Vec<i32>,
    },
    I64 {
        values: Vec<i64>,
    },
    F64 {
        values: Vec<f64>,
    },
    Bool {
        values: Vec<bool>,
    },*/
}

impl<'t> SeqSerializer<'t> {
    fn new(ser: Serializer<'t>, size: Option<usize>) -> Result<Self, Error> {
        Ok(Self {
            elem: SeqElement::new(ser.ty, size)?,
            variants: Vec::with_capacity(size.unwrap_or_default()),
            ser,
        })
    }

    pub fn for_tuple(ser: Serializer<'t>, size: Option<usize>) -> Result<Self, Error> {
        let ser = match ser.ty {
            None => ser.change_type(Some(VariantTy::TUPLE)),
            Some(ty) if ty == VariantTy::VARIANT => ser.change_type(Some(VariantTy::TUPLE)),
            Some(..) => ser,
        };
        Self::new(ser, size)
    }

    pub fn for_seq(ser: Serializer<'t>, size: Option<usize>) -> Result<Self, Error> {
        let ser = match ser.ty {
            None => ser.change_type(Some(VariantTy::ARRAY)),
            Some(ty) if ty == VariantTy::VARIANT => ser.change_type(Some(VariantTy::ARRAY)),
            Some(..) => ser,
        };
        Self::new(ser, size)
    }
}

impl<'t> ser::SerializeSeq for SeqSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_element<S: ?Sized>(&mut self, value: &S) -> Result<(), Self::Error>
    where
        S: Serialize,
    {
        let ser = self.ser.change_type(self.elem.type_()?);
        self.variants.push(value.serialize(ser)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.elem.end(self.variants.len())?;
        Ok(match self.ser.ty {
            None =>
                util::VariantList::with_variants(&self.variants, VariantTy::VARIANT).to_array(),
            Some(ty) if ty == VariantTy::ARRAY || ty == VariantTy::VARIANT =>
                util::VariantList::with_variants(&self.variants, VariantTy::VARIANT).to_array(),
            Some(ty) if ty.is_array() =>
                Variant::array_from_iter_with_type(ty.element(), self.variants),
            Some(ty) if ty.is_dict_entry() => match &self.variants[..] {
                [key, value] => Variant::from_dict_entry(key, value),
                _ => unreachable!(), // we already length-checked it in end(), what else could go wrong here?
            },
            Some(ty) if ty.is_tuple() =>
                Variant::tuple_from_iter(self.variants),
            Some(_) => unreachable!(),
        })
    }
}

impl<'t> ser::SerializeTuple for SeqSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'t> ser::SerializeTupleStruct for SeqSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeTuple::end(self)
    }
}

impl<'t> ser::SerializeStruct for SeqSerializer<'t> {
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeTupleStruct::serialize_field(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeTupleStruct::end(self)
    }
}

/*enum VariantTag {
    Str(String),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl ToVariant for VariantTag {
    fn to_variant(&self) -> glib::Variant {
        match self {
            Self::Str(s) => s.to_variant(),
            Self::I16(i) => i.to_variant(),
            Self::I32(i) => i.to_variant(),
            Self::I64(i) => i.to_variant(),
            Self::U8(u) => u.to_variant(),
            Self::U16(u) => u.to_variant(),
            Self::U32(u) => u.to_variant(),
            Self::U64(u) => u.to_variant(),
        }
    }
}*/

/*pub struct TupleVariantSerializer<'t> {
    //tag: VariantTag,
    inner: TupleSerializer<'t>,
}

impl<'t> TupleVariantSerializer<'t> {
    fn new(
        //tag: VariantTag,
        node: Option<&'t VariantTy>,
        name: &'static str,
        size: usize,
    ) -> Self {
        Self {
            //tag,
            inner: TupleSerializer::new(node, Some(name), size),
        }
    }
}

impl<'t> ser::SerializeTupleVariant for TupleVariantSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
        //Ok((self.tag, SerializeTuple::end(self.inner)?).to_variant())
    }
}

impl<'t> ser::SerializeStructVariant for TupleVariantSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
        //Ok((self.tag, SerializeTuple::end(self.inner)?).to_variant())
    }
}*/

pub struct MapSerializer<'t> {
    ser_key: Serializer<'t>,
    ser_value: Serializer<'t>,
    keys: Vec<Variant>,
    values: Vec<Variant>,
    size: Option<usize>,
}

impl<'t> MapSerializer<'t> {
    fn new(ser_key: Serializer<'t>, ser_value: Serializer<'t>, size: Option<usize>) -> Self {
        Self {
            ser_key,
            ser_value,
            keys: Vec::with_capacity(size.unwrap_or_default()),
            values: Vec::with_capacity(size.unwrap_or_default()),
            size,
        }
    }
}

impl<'t> ser::SerializeMap for MapSerializer<'t> {
    type Ok = Variant;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.keys.push(key.serialize(self.ser_key)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.values.push(value.serialize(self.ser_value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.size {
            Some(expected) if expected != self.values.len() => return Err(Error::LengthMismatch {
                actual: self.values.len(),
                expected,
            }),
            _ => (),
        }
        let keys = util::VariantList::with_variants(&self.keys, self.ser_key.ty.unwrap_or(VariantTy::STRING));
        let values = util::VariantList::with_variants(&self.values, self.ser_value.ty.unwrap_or(VariantTy::VARIANT));
        let elem_ty = VariantType::new_dict_entry(keys.elem_type(), values.elem_type());
        let entries = keys.iter()
            .zip(values.iter())
            .map(|(k, v)| Variant::from_dict_entry(&k, &v));
        Ok(Variant::array_from_iter_with_type(&elem_ty, entries))
    }
}

impl<'t> ser::SerializeStruct for MapSerializer<'t> {
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}

pub enum StructSerializer<'t> {
    Map(MapSerializer<'t>),
    Tuple(SeqSerializer<'t>),
}

impl<'t> ser::SerializeStruct for StructSerializer<'t> {
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self {
            StructSerializer::Map(s) => ser::SerializeStruct::serialize_field(s, key, value),
            StructSerializer::Tuple(s) => ser::SerializeStruct::serialize_field(s, key, value),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            StructSerializer::Map(s) => ser::SerializeStruct::end(s),
            StructSerializer::Tuple(s) => ser::SerializeStruct::end(s),
        }
    }
}

impl<'t> ser::SerializeTupleStruct for StructSerializer<'t> {
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        match self {
            StructSerializer::Map(s) => unimplemented!(),
            StructSerializer::Tuple(s) => ser::SerializeTupleStruct::serialize_field(s, value),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            StructSerializer::Map(s) => ser::SerializeStruct::end(s),
            StructSerializer::Tuple(s) => ser::SerializeStruct::end(s),
        }
    }
}

pub struct BoxedSerializer<'t, T> {
    inner: T,
    ser: Serializer<'t>,
}

impl<'t, S> BoxedSerializer<'t, S> {
    pub fn new(ser: Serializer<'t>, inner: S) -> Self {
        Self {
            inner,
            ser,
        }
    }
}

impl<'t, S: ser::SerializeTuple> ser::SerializeTuple for BoxedSerializer<'t, S> where
    S::Ok: AsRef<Variant> + Into<Variant>,
    S::Error: Into<Error>,
{
    type Ok = Variant;
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_element(value)
            .map_err(Into::into)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
            .map_err(Into::into)
            .and_then(|ok| self.ser.box_if_needed(ok))
    }
}

impl<'t, S: ser::SerializeTupleStruct> ser::SerializeTupleStruct for BoxedSerializer<'t, S> where
    S::Ok: AsRef<Variant> + Into<Variant>,
    S::Error: Into<Error>,
{
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_field(value)
            .map_err(Into::into)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
            .map_err(Into::into)
            .and_then(|ok| self.ser.box_if_needed(ok))
    }
}

impl<'t, S: ser::SerializeStruct> ser::SerializeStruct for BoxedSerializer<'t, S> where
    S::Ok: AsRef<Variant> + Into<Variant>,
    S::Error: Into<Error>,
{
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_field(key, value)
            .map_err(Into::into)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
            .map_err(Into::into)
            .and_then(|ok| self.ser.box_if_needed(ok))
    }
}

impl<'t, S: ser::SerializeSeq> ser::SerializeSeq for BoxedSerializer<'t, S> where
    S::Ok: AsRef<Variant> + Into<Variant>,
    S::Error: Into<Error>,
{
    type Ok = Variant;
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_element(value)
            .map_err(Into::into)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
            .map_err(Into::into)
            .and_then(|ok| self.ser.box_if_needed(ok))
    }
}

impl<'t, S: ser::SerializeMap> ser::SerializeMap for BoxedSerializer<'t, S> where
    S::Ok: AsRef<Variant> + Into<Variant>,
    S::Error: Into<Error>,
{
    type Ok = Variant;
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.inner.serialize_key(key)
            .map_err(Into::into)
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_value(value)
            .map_err(Into::into)
    }

    fn serialize_entry<K: Serialize + ?Sized, V: Serialize + ?Sized>(&mut self, key: &K, value: &V) -> Result<(), Self::Error> {
        self.inner.serialize_entry(key, value)
            .map_err(Into::into)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
            .map_err(Into::into)
            .and_then(|ok| self.ser.box_if_needed(ok))
    }
}

struct VariantSerializerState<'t> {
    ser: Serializer<'t>,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
}

impl<'t> VariantSerializerState<'t> {
    pub fn tag<V: AsRef<Variant> + Into<Variant>>(self, variant: V) -> Result<Variant, Error> {
        todo!()
    }
}

pub struct VariantSerializer<'t, T> {
    inner: T,
    state: VariantSerializerState<'t>,
}

impl<'t, S> VariantSerializer<'t, S> {
    pub fn new(ser: Serializer<'t>, inner: S, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self, Error> {
        Ok(Self {
            inner,
            state: VariantSerializerState {
                ser,
                name,
                variant_index,
                variant,
            },
        })
    }

    pub fn serialize(self) -> Result<Variant, Error> where S: Serialize {
        self.inner.serialize(self.state.ser)
            .and_then(|v| self.state.tag(v))
    }
}

impl<'t, S: ser::SerializeTuple> ser::SerializeTupleVariant for VariantSerializer<'t, S> where
    S::Ok: AsRef<Variant> + Into<Variant>,
    S::Error: Into<Error>,
{
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_element(value)
            .map_err(Into::into)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
            .map_err(Into::into)
            .and_then(|ok| self.state.tag(ok))
    }
}

impl<'t, S: ser::SerializeStruct> ser::SerializeStructVariant for VariantSerializer<'t, S> where
    S::Ok: AsRef<Variant> + Into<Variant>,
    S::Error: Into<Error>,
{
    type Ok = Variant;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
        self.inner.serialize_field(key, value)
            .map_err(Into::into)
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        self.inner.skip_field(key)
            .map_err(Into::into)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
            .map_err(Into::into)
            .and_then(|ok| self.state.tag(ok))
    }
}
