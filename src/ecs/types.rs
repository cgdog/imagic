use std::{any::{type_name, TypeId}, marker::PhantomData};

use imagic_macros::batch_process_tuples;

pub struct ImagicTypeInfo {
    pub is_ref: bool,
    pub is_mut_ref: bool,
    pub type_id: TypeId,
    pub type_name: &'static str,
}
pub trait TupleTypesInfo: 'static {
    fn type_ids() -> Vec<TypeId>;
    fn type_names() -> Vec<&'static str>;
}

trait RefCheck<T> {
    fn is_ref() -> bool;
    fn is_mut_ref() -> bool;
    fn target_type_id() -> TypeId;
    fn target_type_name() -> &'static str;
}

// 基础类型实现
struct BaseType<T>(PhantomData<T>);
impl<T: 'static> RefCheck<T> for BaseType<T> {
    fn is_ref() -> bool { false }
    fn is_mut_ref() -> bool { false }
    fn target_type_id() -> TypeId { TypeId::of::<T>() }
    fn target_type_name() -> &'static str { type_name::<T>() }
}

// 不可变引用实现
struct ImmutRef<T>(PhantomData<T>);
impl<T: 'static> RefCheck<T> for ImmutRef<&T> {
    fn is_ref() -> bool { true }
    fn is_mut_ref() -> bool { false }
    fn target_type_id() -> TypeId { TypeId::of::<T>() }
    fn target_type_name() -> &'static str { type_name::<T>() }
}

// 可变引用实现
struct MutRef<T>(PhantomData<T>);
impl<T: 'static> RefCheck<T> for MutRef<&mut T> {
    fn is_ref() -> bool { true }
    fn is_mut_ref() -> bool { true }
    fn target_type_id() -> TypeId { TypeId::of::<T>() }
    fn target_type_name() -> &'static str { type_name::<T>() }
}

macro_rules! impl_tuple_types {
    ($($T:ident),*) => {
        impl<$($T: 'static),*> TupleTypesInfo for ($($T,)*) {
            fn type_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$T>()),*]
            }
            fn type_names() -> Vec<&'static str> {
                vec![$(std::any::type_name::<$T>()),*]
            }
        }
    };
}

batch_process_tuples!(impl_tuple_types, 0, 32, T);