use crate::subversion::SvnErrnoConstants;
use crate::subversion::ffi;

use super::apr;
use super::subversion;

use snafu::Backtrace;
use snafu::IntoError;
use snafu::Snafu;
use uniffi::UnexpectedUniFFICallbackError;
use std::fmt::Debug;
use strum::EnumDiscriminants;

use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Snafu, Debug, EnumDiscriminants, uniffi::Error)]
#[snafu(module(builder), context(suffix(false)), visibility(pub))]
#[strum_discriminants(name(AliasError))]
#[strum_discriminants(derive(strum::Display))]
#[uniffi(flat_error)]
pub enum Error {
    #[snafu(display("{}", serde_json::to_string(source).unwrap_or_default()))]
    AprError {
        source: apr::AprError,
        backtrace: Backtrace,
    },
    #[snafu(display("{}", serde_json::to_string(source).unwrap_or_default()))]
    SvnError {
        source: subversion::SVNError,
        backtrace: Backtrace,
    },

    #[snafu(display("{}", source))]
    DatabaseError {
        source: Box<dyn std::error::Error + Sync + Send>,
        backtrace: Backtrace,
    },

    #[snafu(display("Invalid argument: {detail} at {location}"))]
    InvalidArgument {
        detail: String,
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("System IO error: {source}"))]
    IOError {
        source: io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Invalid uuid: {source}"))]
    InvalidID {
        source: uuid::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("General error: {detail}"))]
    GeneralError { detail: String },

    #[snafu(display("Rust tokio runtime error: {source}"))]
    RuntimeError { source: tokio::task::JoinError },

    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Sync + Send>, Some)))]
        source: Option<Box<dyn std::error::Error + Sync + Send>>,
    },

    #[snafu(display("Svg decode error: {source}"))]
    SvgError { source: resvg::usvg::Error },

    #[snafu(display("Json error: {source}"))]
    JsonError {
        source: serde_json::Error,
        backtrace: Backtrace,
    },
}


pub fn ok<T>(value: T) -> Result<T, Error> {
    Ok(value)
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        builder::IO {}.into_error(source)
    }
}

impl From<uuid::Error> for Error {
    fn from(source: uuid::Error) -> Self {
        builder::InvalidID {}.into_error(source)
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(value: redb::DatabaseError) -> Self {
        builder::Database {}.into_error(Box::new(value) as _)
    }
}

impl From<redb::TransactionError> for Error {
    fn from(value: redb::TransactionError) -> Self {
        builder::Database {}.into_error(Box::new(value) as _)
    }
}

impl From<redb::StorageError> for Error {
    fn from(value: redb::StorageError) -> Self {
        builder::Database {}.into_error(Box::new(value) as _)
    }
}

impl From<redb::TableError> for Error {
    fn from(value: redb::TableError) -> Self {
        builder::Database {}.into_error(Box::new(value) as _)
    }
}

impl From<redb::CommitError> for Error {
    fn from(value: redb::CommitError) -> Self {
        builder::Database {}.into_error(Box::new(value) as _)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        builder::Json {}.into_error(value)
    }
}

#[derive(Snafu, Debug, uniffi::Error)]
pub enum CSharpError {
    #[snafu(display("{msg}({code})"))]
    SubversionError {
        code: i32,
        msg: String,
    },

    #[snafu(display("{detail}"))]
    UnexpectedError {
        detail: String,
    }
}

impl From<UnexpectedUniFFICallbackError> for CSharpError {
    fn from(value: UnexpectedUniFFICallbackError) -> Self {
        UnexpectedSnafu { detail: value.to_string() }.into_error(snafu::NoneError)
    }
}

impl CSharpError {
    pub fn native_error(&self) -> *mut ffi::svn_error_t {
        let (code, msg) = match self {
            CSharpError::SubversionError { code, msg } => {
                (*code, msg.as_str())
            },
            CSharpError::UnexpectedError { detail } => {
                let constants = SvnErrnoConstants::new();
                (constants.cease_invocation, detail.as_str())
            },
        };
        unsafe  {
            let mut pool = apr::Pool::create();
            let msg = pool.string(msg).unwrap_or_default();

            let error = ffi::svn_error_create(code.try_into().unwrap_or_default(), Default::default(), msg);

            error
        }
    }
}

#[easy_ext::ext(CSharpErrorExtension)]
pub impl<T> Result<T, CSharpError> {
    fn native_error(&self) -> *mut ffi::svn_error_t {
        match self {
            Ok(_) => std::ptr::null_mut(),
            Err(e) => e.native_error(),
        }
    }
}
