
use std::{any::Any, sync::Arc};

use sea_orm::ActiveValue;
use snafu::{OptionExt, ResultExt};

use crate::{AnyValue, error::Error};

// #[easy_ext::ext(JsonExtension)]
// pub impl<T> T {
//     fn as_json(&self) -> error::Result<String>
//     where
//         Self: Serialize,
//     {
//         let v = serde_json::to_string(self)?;
//         Ok(v)
//     }
//     fn from_json<'de>(json: &'de str) -> error::Result<Self>
//     where
//         Self: Deserialize<'de>,
//     {
//         let v = serde_json::from_str(json)?;
//         Ok(v)
//     }
// }

#[easy_ext::ext(ActiveValueExtension)]
pub impl<T: Into<sea_query::Value>> ActiveValue<T> {
    fn set_value(&mut self, value: T) {
        *self = Self::Set(value);
    }
}

#[easy_ext::ext(DefaultExtension)]
pub impl<T: Default> T {
    fn default_value() -> Self {
        Default::default()
    }
}

// pub enum ValueOrRef<'a, T> {
//     Ref(&'a T),
//     Value(T),
// }

// impl<'a, T> AsRef<T> for ValueOrRef<'a, T> {
//     fn as_ref(&self) -> &T {
//         match *self {
//             ValueOrRef::Ref(v) => v,
//             ValueOrRef::Value(ref v) => v,
//         }
//     }
// }

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

    fn inner_into<V: From<T>>(self) -> Option<V> {
        self.map(|v| V::from(v))
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
    fn if_some<V>(self, option: Option<V>, f: impl FnOnce(Self, V) -> Self) -> Self {
        if let Some(v) = option { f(self, v) } else { self }
    }

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
