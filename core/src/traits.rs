use crate::core::validator::RodValidator;
use crate::{any, boolean, number, string};

pub trait RodSchema {
    fn schema() -> Box<dyn RodValidator>;
}

impl RodSchema for String {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(string())
    }
}
impl RodSchema for &str {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(string())
    }
}
impl RodSchema for bool {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(boolean())
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl RodSchema for $t {
                fn schema() -> Box<dyn RodValidator> { Box::new(number().int()) }
            }
        )*
    };
}
impl_int!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl RodSchema for f32 {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(number())
    }
}
impl RodSchema for f64 {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(number())
    }
}

impl RodSchema for serde_json::Value {
    fn schema() -> Box<dyn RodValidator> {
        Box::new(any())
    }
}
