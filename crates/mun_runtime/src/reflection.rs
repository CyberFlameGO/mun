use crate::{marshal::Marshal, Runtime, StructRef};
use abi::HasStaticTypeInfo;
use once_cell::sync::OnceCell;

/// Returns whether the specified argument type matches the `type_info`.
pub fn equals_argument_type<'e, 'f, T: ArgumentReflection>(
    runtime: &'f Runtime,
    type_info: &'e abi::TypeInfo,
    arg: &'f T,
) -> Result<(), (&'e str, &'f str)> {
    if type_info.guid != arg.type_guid(runtime) {
        Err((type_info.name(), arg.type_name(runtime)))
    } else {
        Ok(())
    }
}

/// Returns whether the specified return type matches the `type_info`.
pub fn equals_return_type<T: ReturnTypeReflection>(
    type_info: &abi::TypeInfo,
) -> Result<(), (&str, &str)> {
    match type_info.group {
        abi::TypeGroup::FundamentalTypes => {
            if type_info.guid != T::type_guid() {
                return Err((type_info.name(), T::type_name()));
            }
        }
        abi::TypeGroup::StructTypes => {
            if <StructRef as ReturnTypeReflection>::type_guid() != T::type_guid() {
                return Err(("struct", T::type_name()));
            }
        }
    }
    Ok(())
}

/// A type to emulate dynamic typing across compilation units for static types.
pub trait ReturnTypeReflection: Sized {
    /// The resulting type after marshaling.
    type Marshalled: Marshal<Self>;

    /// Retrieves the type's `Guid`.
    fn type_guid() -> abi::Guid;

    /// Retrieves the type's name.
    fn type_name() -> &'static str;
}

/// A type to emulate dynamic typing across compilation units for statically typed values.
pub trait ArgumentReflection: Sized {
    /// The resulting type after dereferencing.
    type Marshalled: Marshal<Self>;

    /// Retrieves the `Guid` of the value's type.
    fn type_guid(&self, runtime: &Runtime) -> abi::Guid;

    /// Retrieves the name of the value's type.
    fn type_name<'r>(&'r self, runtime: &'r Runtime) -> &'r str;

    /// Marshals the value.
    fn marshal(self) -> Self::Marshalled;
}

macro_rules! impl_primitive_type {
    ($($ty:ty),+) => {
        $(
            impl ArgumentReflection for $ty {
                type Marshalled = Self;

                fn type_guid(&self, _runtime: &Runtime) -> abi::Guid {
                    Self::type_info().guid
                }

                fn type_name(&self, _runtime: &Runtime) -> &str {
                    Self::type_info().name()
                }

                fn marshal(self) -> Self::Marshalled {
                    self
                }
            }

            impl ReturnTypeReflection for $ty {
                type Marshalled = Self;

                fn type_guid() -> abi::Guid {
                    Self::type_info().guid
                }

                fn type_name() -> &'static str {
                    Self::type_info().name()
                }
            }
        )+
    }
}

impl_primitive_type!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool
);

impl ReturnTypeReflection for () {
    type Marshalled = ();

    fn type_name() -> &'static str {
        "core::empty"
    }

    fn type_guid() -> abi::Guid {
        // TODO: Once `const_fn` lands, replace this with a const md5 hash
        static GUID: OnceCell<abi::Guid> = OnceCell::new();
        *GUID.get_or_init(|| abi::Guid {
            b: md5::compute(Self::type_name()).0,
        })
    }
}

impl<T> ArgumentReflection for *const T
where
    *const T: HasStaticTypeInfo,
{
    type Marshalled = Self;

    fn type_guid(&self, _runtime: &Runtime) -> abi::Guid {
        Self::type_info().guid
    }

    fn type_name(&self, _runtime: &Runtime) -> &str {
        Self::type_info().name()
    }

    fn marshal(self) -> Self::Marshalled {
        self
    }
}

impl<T> ReturnTypeReflection for *const T
where
    *const T: HasStaticTypeInfo,
{
    type Marshalled = Self;

    fn type_guid() -> abi::Guid {
        Self::type_info().guid
    }

    fn type_name() -> &'static str {
        Self::type_info().name()
    }
}

impl<T> ArgumentReflection for *mut T
where
    *mut T: HasStaticTypeInfo,
{
    type Marshalled = Self;

    fn type_guid(&self, _runtime: &Runtime) -> abi::Guid {
        Self::type_info().guid
    }

    fn type_name(&self, _runtime: &Runtime) -> &str {
        Self::type_info().name()
    }

    fn marshal(self) -> Self::Marshalled {
        self
    }
}

impl<T> ReturnTypeReflection for *mut T
where
    *mut T: HasStaticTypeInfo,
{
    type Marshalled = Self;

    fn type_guid() -> abi::Guid {
        Self::type_info().guid
    }

    fn type_name() -> &'static str {
        Self::type_info().name()
    }
}
