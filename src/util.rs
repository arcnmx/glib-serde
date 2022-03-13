use glib::{VariantTy, BoolError, bool_error, Variant, ToVariant};
use std::{iter::FusedIterator, borrow::Cow};
use crate::traits::GlibVariantWrapper;

#[derive(Copy, Clone)]
pub struct VariantTyTupleIterator<'a> {
    elem: Option<&'a VariantTy>,
}

impl<'a> VariantTyTupleIterator<'a> {
    pub fn new(ty: &'a VariantTy) -> Result<Self, BoolError> {
        if (ty.is_tuple() && ty != VariantTy::TUPLE) || ty.is_dict_entry() {
            Ok(Self {
                elem: ty.first(),
            })
        } else {
            Err(bool_error!("Expected a definite tuple or dictionary entry type"))
        }
    }

    pub fn peek(&self) -> Option<&'a VariantTy> {
        self.elem
    }
}

impl<'a> Iterator for VariantTyTupleIterator<'a> {
    type Item = &'a VariantTy;

    fn next(&mut self) -> Option<Self::Item> {
        let elem = self.elem?;
        self.elem = elem.next();
        Some(elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.len();
        (count, Some(count))
    }
}

impl<'a> ExactSizeIterator for VariantTyTupleIterator<'a> {
    fn len(&self) -> usize {
        self.clone().count()
    }
}

impl<'a> FusedIterator for VariantTyTupleIterator<'a> { }

pub fn variant_list_type<'a, V: GlibVariantWrapper<'a> + 'a, I: IntoIterator<Item=V>>(vars: I) -> Option<Option<Cow<'a, VariantTy>>> {
    vars.into_iter()
        .fold(None, |res, var| match &res {
            None => Some(Some(match var.variant_cow() {
                Cow::Owned(v) => v.type_().to_owned().into(),
                Cow::Borrowed(v) => v.type_().into(),
            })),
            Some(Some(ty)) if *ty == var.variant_ref().type_() => res,
            Some(_) => Some(None),
        })
}

pub struct VariantList<'a, V: 'a> {
    elem_ty: Option<Cow<'a, VariantTy>>,
    variants: V,
}

impl<'a, V: 'a> VariantList<'a, V> {
    pub fn new(elem_ty: Option<Cow<'a, VariantTy>>, variants: V) -> Self {
        Self {
            elem_ty,
            variants,
        }
    }

    pub fn elem_type(&self) -> &VariantTy {
        match &self.elem_ty {
            Some(ty) => ty,
            None => VariantTy::VARIANT,
        }
    }
}

impl<'a, V: GlibVariantWrapper<'a> + 'a, I: Clone + IntoIterator<Item=V> + 'a> VariantList<'a, I> {
    pub fn with_variants(variants: I, empty_fallback: &'a VariantTy) -> Self {
        let ty = match variant_list_type(variants.clone()) {
            None => Some(empty_fallback.into()),
            Some(Some(ty)) => Some(ty),
            Some(None) => None,
        };
        Self::new(ty, variants)
    }

    pub fn is_empty(&self) -> bool {
        self.variants().next().is_none()
    }

    pub fn variants(&self) -> impl Iterator<Item=Cow<'a, Variant>> {
        self.variants.clone().into_iter()
            .map(|v| v.variant_cow())
    }

    pub fn iter(&self) -> impl Iterator<Item=Cow<'a, Variant>> + 'a {
        let elem_ty_boxed = self.elem_ty.is_none();
        self.variants()
            .map(move |v| match elem_ty_boxed {
                true => Cow::Owned(v.to_variant()),
                false => v,
            })
    }

    pub fn to_array(&self) -> Variant {
        Variant::array_from_iter_with_type(self.elem_type(), self.iter())
    }

    pub fn to_tuple_or_array(&self) -> Variant {
        match &self.elem_ty {
            None => Variant::tuple_from_iter(self.variants()),
            Some(..) if self.is_empty() => ().to_variant(),
            Some(ty) => Variant::array_from_iter_with_type(ty, self.variants()),
        }
    }
}
