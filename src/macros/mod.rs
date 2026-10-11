#[macro_export]
macro_rules! optional_struct {
    (
        $(#[$struct_meta:meta])*
        $struct_name:ident {
            $($(#[$field_meta:meta])* $field_name:ident : $field_ty:ty),* $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        pub struct $struct_name {
            $($(#[$field_meta])* pub $field_name: Option<$field_ty>,)*
        }

        impl $struct_name {
            /// Take a member out, leaving None behind
            pub fn take_field<T>(&mut self, field: &mut Option<T>) -> T {
                field.take().expect("field already taken")
            }
        }
    };
}

#[macro_export]
macro_rules! optional_struct_init {
    (
        $struct_name:ident {
            $($field_name:ident : $expr:expr),* $(,)?
        }
    ) => {
        $struct_name {
            $($field_name: Some($expr),)*
        }
    };
}
