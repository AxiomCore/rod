use crate::core::input::RodInput;
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssue, ValidationContext};
use std::future::Future;
use std::pin::Pin;

// FIX: Removed the `Send` bound for single-threaded WASM compatibility.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait AsyncRodValidator: 'static {
    fn validate_async<'a>(
        &'a self,
        input: &'a dyn RodInput<'a>,
    ) -> BoxFuture<'a, Result<RodValue<'a>, Vec<RodIssue>>>;
}

impl<T: RodValidator + ?Sized + 'static> AsyncRodValidator for T {
    fn validate_async<'a>(
        &'a self,
        input: &'a dyn RodInput<'a>,
    ) -> BoxFuture<'a, Result<RodValue<'a>, Vec<RodIssue>>> {
        let mut ctx = ValidationContext::new();
        let result = self.validate_with_context(&mut ctx, input);

        let issues = ctx.issues;
        Box::pin(async move {
            match result {
                Ok(v) if issues.is_empty() => Ok(v),
                _ => Err(issues),
            }
        })
    }
}

pub struct AsyncRefine<F> {
    schema: Box<dyn AsyncRodValidator>,
    // FIX: Removed Send + Sync from the closure's future for single-threaded environment
    check: F,
}

impl<F> AsyncRefine<F>
where
    F: for<'v, 'd> Fn(&'v RodValue<'d>) -> BoxFuture<'v, Result<(), String>> + 'static,
{
    pub fn new<V>(schema: V, check: F) -> Self
    where
        V: AsyncRodValidator + 'static,
    {
        Self {
            schema: Box::new(schema),
            check,
        }
    }
}

impl<F> AsyncRodValidator for AsyncRefine<F>
where
    F: for<'v, 'd> Fn(&'v RodValue<'d>) -> BoxFuture<'v, Result<(), String>> + 'static,
{
    fn validate_async<'a>(
        &'a self,
        input: &'a dyn RodInput<'a>,
    ) -> BoxFuture<'a, Result<RodValue<'a>, Vec<RodIssue>>> {
        Box::pin(async move {
            let val = match self.schema.validate_async(input).await {
                Ok(v) => v,
                Err(issues) => return Err(issues),
            };

            if let Err(msg) = (self.check)(&val).await {
                return Err(vec![RodIssue {
                    details: crate::error::RodIssueCode::Custom {
                        message: msg.clone(),
                        params: None,
                    },
                    path: vec![],
                    message: msg,
                }]);
            }

            Ok(val)
        })
    }
}
