#[macro_export]
macro_rules! newtype_wrapper {
    (
        $(#[$meta:meta])*
        $(@[
            $(PartialOrd $partialord:ident)?
            $(PartialEq $partialeq:ident)?
            $(Display $display:ident)?
            $(FromStr $fromstr:ident)?
            $(GlibVariantWrapper $glibvariantwrapper:ident)?
            $(StaticVariantType $staticvarianttype:ident)?
            $(ToVariant $tovariant:ident)?
            $(FromVariant $fromvariant:ident)?
        ])*
        $vis:vis $id:ident($ty:ty|$owned:ty) $into_owned:ident
    ) => {
        $(#[$meta])*
        $($(#[derive($partialord)])?)*
        $($(#[derive($partialeq)])?)*
        #[repr(transparent)]
        $vis struct $id<'a>(std::borrow::Cow<'a, $ty>);

        impl<'a> $id<'a> {
            pub fn wrap(v: $owned) -> Self {
                Self(std::borrow::Cow::Owned(v))
            }

            pub fn borrow(v: &'a $ty) -> Self {
                Self(std::borrow::Cow::Borrowed(v))
            }

            pub fn inner<'s>(&'s self) -> &'a $ty where 's: 'a {
                match self.0 {
                    std::borrow::Cow::Borrowed(s) => s,
                    std::borrow::Cow::Owned(ref s) => s,
                }
            }

            pub fn inner_mut(&mut self) -> &mut $owned {
                self.0.to_mut()
            }

            pub fn $into_owned(self) -> $owned {
                self.into_inner().into_owned()
            }

            pub fn into_inner(self) -> std::borrow::Cow<'a, $ty> {
                self.0
            }

            pub fn borrowed<'s>(&'s self) -> Self where 's: 'a {
                Self(std::borrow::Cow::Borrowed(self.inner()))
            }
        }

        impl<'a> From<$owned> for $id<'a> {
            fn from(v: $owned) -> Self {
                Self::wrap(v)
            }
        }

        impl<'a> From<&'a $ty> for $id<'a> {
            fn from(v: &'a $ty) -> Self {
                Self::borrow(v)
            }
        }

        impl<'a> Into<$owned> for $id<'a> {
            fn into(self) -> $owned {
                self.$into_owned()
            }
        }

        impl<'a> std::ops::Deref for $id<'a> {
            type Target = $ty;

            fn deref(&self) -> &Self::Target {
                self.inner()
            }
        }

        impl<'a> AsRef<$ty> for $id<'a> {
            fn as_ref(&self) -> &$ty {
                self.inner()
            }
        }

        $(
            $(
                impl<'a> $partialord<$ty> for $id<'a> {
                    fn partial_cmp(&self, rhs: &$ty) -> Option<std::cmp::Ordering> {
                        $partialord::partial_cmp(self.inner(), rhs)
                    }
                }
            )?
            $(
                impl<'a> $partialeq<$ty> for $id<'a> {
                    fn eq(&self, rhs: &$ty) -> bool {
                        $partialeq::eq(self.inner(), rhs)
                    }
                }
            )?
            $(
                impl<'a> std::fmt::$display for $id<'a> {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        std::fmt::$display::fmt(self.inner(), f)
                    }
                }
            )?
            $(
                impl<'a> std::str::$fromstr for $id<'a> {
                    type Err = <$owned as std::str::$fromstr>::Err;
                    fn from_str(str: &str) -> Result<Self, Self::Err> {
                        <$owned as std::str::$fromstr>::from_str(str)
                            .map(Into::into)
                    }
                }
            )?
            $(
                impl<'a> crate::traits::$glibvariantwrapper<'a> for $id<'a> {
                    fn variant_ref(&self) -> &glib::Variant {
                        self.inner()
                    }

                    fn variant_cow(self) -> std::borrow::Cow<'a, glib::Variant> {
                        self.into_inner()
                    }

                    fn into_variant(self) -> glib::Variant {
                        self.$into_owned()
                    }
                }
            )?

            $(
                impl<'a> glib::$staticvarianttype for $id<'a> {
                    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
                        <$ty as glib::$staticvarianttype>::static_variant_type()
                    }
                }
            )?

            $(
                impl<'a> glib::$tovariant for $id<'a> {
                    fn to_variant(&self) -> glib::Variant {
                        glib::$tovariant::to_variant(self.inner())
                    }
                }
            )?

            $(
                impl<'a> glib::$fromvariant for $id<'a> {
                    fn from_variant(variant: &glib::Variant) -> Option<Self> {
                        glib::$fromvariant::from_variant(variant).map(Self::wrap)
                    }
                }
            )?
        )*
    };
}
