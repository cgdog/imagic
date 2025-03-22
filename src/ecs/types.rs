use std::any::TypeId;

use imagic_macros::batch_process_tuples;

pub trait TupleTypesInfo: 'static {
    fn type_ids() -> Vec<TypeId>;
    fn type_names() -> Vec<&'static str>;
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