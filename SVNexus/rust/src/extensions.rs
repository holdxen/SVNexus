
use std::{any::Any, sync::Arc};

use snafu::{OptionExt, ResultExt};

use crate::{AnyValue, error::Error};

#[easy_ext::ext(OptionExtension)]
pub impl<T> Option<T> {
    #[track_caller]
    fn any_context<S>(self, context: S) -> Result<T, Error>
    where
        S: Into<String>,
    {
        self.whatever_context::<_, Error>(context)
    }

    #[track_caller]
    fn with_any_context<F, S>(self, context: F) -> Result<T, Error>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.with_whatever_context::<_, _, Error>(context)
    }
}

#[easy_ext::ext(ResultExtension)]
pub impl<T, E: std::error::Error + Send + Sync + 'static> Result<T, E> {
    #[track_caller]
    fn any_context<S>(self, context: S) -> Result<T, Error>
    where
        S: Into<String>,
    {
        self.whatever_context::<_, Error>(context)
    }

    #[track_caller]
    fn with_any_context<F, S>(self, context: F) -> Result<T, Error>
    where
        F: FnOnce(&mut E) -> S,
        S: Into<String>,
    {
        self.with_whatever_context::<_, _, Error>(context)
    }

}

#[easy_ext::ext(CommonExtension)]
pub impl<T: Sized> T {
    fn if_or<R>(
        self,
        value: bool,
        i: impl FnOnce(Self) -> R,
        o: impl FnOnce(Self) -> R,
    ) -> R {
        if value { i(self) } else { o(self) }
    }

    fn so_if_or<R>(
        self,
        b: impl FnOnce(&Self) -> bool,
        i: impl FnOnce(Self) -> R,
        o: impl FnOnce(Self) -> R,
    ) -> R {
        if b(&self) { i(self) } else { o(self) }
    }

    fn if_or_ref<R>(
        &self,
        value: bool,
        i: impl FnOnce(&Self) -> R,
        o: impl FnOnce(&Self) -> R,
    ) -> R {
        if value { i(self) } else { o(self) }
    }

    fn also_build(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }

    fn into_option_some(self) -> Option<Self> {
        Some(self)
    }

    fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    fn into_any_value(self) -> AnyValue
    where
        Self: Any + Send + Sync + 'static,
    {
        AnyValue::new(self)
    }
}
