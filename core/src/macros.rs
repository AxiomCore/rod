#[macro_export]
macro_rules! rod_obj {
    ( $( $key:ident : $schema:expr ),* $(,)? ) => {
        {
            let mut shape: std::collections::HashMap<String, $crate::types::node::RodNode> = std::collections::HashMap::new();
            $(
                shape.insert(
                    stringify!($key).to_string(),
                    $crate::types::node::IntoRodNode::into_node($schema)
                );
            )*
            $crate::types::object::RodObject::new(shape)
        }
    };
}
