#[macro_export]
macro_rules! newtype_wrapper {
    ($vis:vis $id:ident($ty:ty) $owned:ident) => {
        #[derive(Debug, Clone, PartialOrd, PartialEq, Hash)]
        $vis struct $id<'a>(std::borrow::Cow<'a, $ty>);

        impl<'a> $id<'a> {
            pub fn inner<'s>(&'s self) -> &'a $ty where 's: 'a {
                match self.0 {
                    std::borrow::Cow::Borrowed(s) => s,
                    std::borrow::Cow::Owned(ref s) => s,
                }
            }

            pub fn inner_mut(&mut self) -> Option<&mut $ty> {
                match self.0 {
                    std::borrow::Cow::Borrowed(_) => None,
                    std::borrow::Cow::Owned(ref mut s) => Some(s),
                }
            }

            pub fn $owned(self) -> $ty {
                self.into_inner().into_owned()
            }

            pub fn into_inner(self) -> std::borrow::Cow<'a, $ty> {
                self.0
            }

            pub fn borrowed<'s>(&'s self) -> Self where 's: 'a {
                Self(std::borrow::Cow::Borrowed(self.inner()))
            }
        }

        impl<'a> From<$ty> for $id<'a> {
            fn from(v: $ty) -> Self {
                Self(std::borrow::Cow::Owned(v))
            }
        }

        impl<'a> From<&'a $ty> for $id<'a> {
            fn from(v: &'a $ty) -> Self {
                Self(std::borrow::Cow::Borrowed(v))
            }
        }

        impl<'a> Into<$ty> for $id<'a> {
            fn into(self) -> $ty {
                self.into_variant()
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
    };
}
