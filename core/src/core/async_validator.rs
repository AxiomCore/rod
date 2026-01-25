use crate::core::input::{RodInput, RodThreadBounds};
use crate::core::validator::RodValidator;
use crate::core::value::RodValue;
use crate::error::{RodIssue, ValidationContext};
use std::future::Future;
use std::pin::Pin;

// Define BoxFuture with conditional Send bound based on the feature flag
#[cfg(not(feature = "non-thread-safe"))]
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(feature = "non-thread-safe")]
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Async validation trait.
/// Inherits RodThreadBounds to ensure Send/Sync in native Rust environments (Actix/Axum),
/// but allows !Send in WASM via the feature flag.
pub trait AsyncRodValidator: 'static + RodThreadBounds {
    fn validate_async<'a>(
        &'a self,
        input: &'a dyn RodInput<'a>,
    ) -> BoxFuture<'a, Result<RodValue<'a>, Vec<RodIssue>>>;
}

// Blanket implementation for Sync validators
impl<T: RodValidator + ?Sized + 'static> AsyncRodValidator for T {
    fn validate_async<'a>(
        &'a self,
        input: &'a dyn RodInput<'a>,
    ) -> BoxFuture<'a, Result<RodValue<'a>, Vec<RodIssue>>> {
        // Since T is RodValidator, it is also RodThreadBounds, satisfying the trait constraint.
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
    check: F,
}

impl<F> AsyncRefine<F>
where
    // Ensure the check closure also respects threading bounds
    F: for<'v, 'd> Fn(&'v RodValue<'d>) -> BoxFuture<'v, Result<(), String>>
        + 'static
        + RodThreadBounds,
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
    F: for<'v, 'd> Fn(&'v RodValue<'d>) -> BoxFuture<'v, Result<(), String>>
        + 'static
        + RodThreadBounds,
{
    fn validate_async<'a>(
        &'a self,
        input: &'a dyn RodInput<'a>,
    ) -> BoxFuture<'a, Result<RodValue<'a>, Vec<RodIssue>>> {
        Box::pin(async move {
            // 1. Await inner validation
            let val = match self.schema.validate_async(input).await {
                Ok(v) => v,
                Err(issues) => return Err(issues),
            };

            // 2. Await the async refinement check
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
