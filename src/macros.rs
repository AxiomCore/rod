// src/macros.rs

#[macro_export]
macro_rules! rod_obj {
    ( $( $key:ident : $schema:expr ),* $(,)? ) => {
        {
            let mut shape: std::collections::HashMap<String, Box<dyn $crate::core::validator::RodValidator>> = std::collections::HashMap::new();
            $(
                shape.insert(stringify!($key).to_string(), Box::new($schema));
            )*
            $crate::types::object::object(shape)
        }
    };
}
