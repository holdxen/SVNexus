use crate::subversion::ffi;
use crate::subversion::SubversionErrorCode;

use super::apr;
use super::subversion;

use serde::ser::SerializeStruct;
use serde::Deserialize;
use serde::Serialize;
use snafu::Backtrace;
use snafu::IntoError;
use snafu::Snafu;
use std::fmt::Debug;
use strum::EnumDiscriminants;

use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Snafu, Debug, EnumDiscriminants, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[snafu(module(builder), context(suffix(false)), visibility(pub))]
// #[strum_discriminants(name(AliasError))]
// #[strum_discriminants(derive(strum::Display, serde::Serialize))]
pub enum Error {
    #[snafu(display("APR error: {source}"))]
    AprError {
        source: apr::AprError,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },
    #[snafu(display("Subversion error: {source}"))]
    SubversionError {
        source: subversion::SubversionError,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    // #[snafu(display("{}", source))]
    // DatabaseError {
    //     source: Box<dyn std::error::Error + Sync + Send>,
    //     backtrace: Backtrace,
    // },
    #[snafu(display("Invalid argument: {detail} at {location}"))]
    InvalidArgument {
        detail: String,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("System IO error: {source}"))]
    IOError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: io::Error,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    // #[snafu(display("Invalid uuid: {source}"))]
    // InvalidID {
    //     #[serde(serialize_with = "serialize_with_display")]
    //     #[ts(type = "string")]
    //     source: uuid::Error,
    //     #[serde(skip)]
    //     backtrace: Backtrace,
    // },
    #[snafu(display("General error: {detail}"))]
    GeneralError {
        detail: String,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Rust tokio runtime error: {source}"))]
    RuntimeError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: tokio::task::JoinError,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Sync + Send>, Some)))]
        #[serde(skip)]
        source: Option<Box<dyn std::error::Error + Sync + Send>>,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Json error: {source}"))]
    JsonError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: serde_json::Error,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Failed to parse enum: {source}, {detail}"))]
    EnumParseError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: strum::ParseError,
        detail: String,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Tauri exception: {source}"))]
    TauriError {
        source: FrontendError,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Repository cache broken: {uuid}"))]
    CacheBrokenError {
        uuid: String,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Failed to query database: {source}"))]
    DatabaseError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: surrealdb::Error,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Failed to find executable: {source}"))]
    WhichError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: which::Error,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Invalid glob: {source}"))]
    GlobError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: ignore::Error,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("Unexpected message: {detail}"))]
    UnexpectedMessage {
        detail: String,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("MessagePack error: {source}"))]
    MessagePackError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: Box<dyn std::error::Error + Sync + Send>,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },

    #[snafu(display("SSH error: {source}"))]
    FlatlineError {
        #[serde(serialize_with = "serialize_with_display")]
        #[ts(type = "string")]
        source: flatline::error::Error,
        #[serde(skip)]
        backtrace: Backtrace,

        #[serde(serialize_with = "serialize_location")]
        #[ts(type = "import('./Location').Location")]
        #[snafu(implicit)]
        location: snafu::Location,
    },
}

#[cfg(test)]
#[derive(Serialize, Debug, ts_rs::TS)]
#[ts(export)]
#[ts(rename = "Location")]
#[serde(rename_all = "camelCase")]
struct LocationMirror {
    file: &'static str,
    line: u32,
    column: u32,
}

fn serialize_location<S: serde::Serializer>(
    value: &snafu::Location,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let mut location = serializer.serialize_struct("Location", 3usize)?;
    location.serialize_field("file", value.file())?;
    location.serialize_field("line", &value.line())?;
    location.serialize_field("column", &value.column())?;
    location.end()
}

fn serialize_with_display<S: serde::Serializer>(
    value: &impl std::fmt::Display,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

pub fn ok<T>(value: T) -> Result<T, Error> {
    Ok(value)
}

impl From<io::Error> for Error {
    #[track_caller]
    fn from(source: io::Error) -> Self {
        builder::IO {}.into_error(source)
    }
}

// impl From<uuid::Error> for Error {
//     fn from(source: uuid::Error) -> Self {
//         builder::InvalidID {}.into_error(source)
//     }
// }

impl From<flatline::error::Error> for Error {
    #[track_caller]
    fn from(value: flatline::error::Error) -> Self {
        builder::Flatline {}.into_error(value)
    }
}

impl From<which::Error> for Error {
    #[track_caller]
    fn from(value: which::Error) -> Self {
        builder::Which {}.into_error(value)
    }
}

impl From<serde_json::Error> for Error {
    #[track_caller]
    fn from(value: serde_json::Error) -> Self {
        builder::Json {}.into_error(value)
    }
}

impl From<strum::ParseError> for Error {
    #[track_caller]
    fn from(value: strum::ParseError) -> Self {
        builder::EnumParse { detail: "" }.into_error(value)
    }
}

impl From<FrontendError> for Error {
    #[track_caller]
    fn from(value: FrontendError) -> Self {
        builder::Tauri {}.into_error(value)
    }
}

impl From<rmp_serde::decode::Error> for Error {
    #[track_caller]
    fn from(value: rmp_serde::decode::Error) -> Self {
        builder::MessagePack {}.into_error(Box::new(value))
    }
}

impl From<rmp_serde::encode::Error> for Error {
    #[track_caller]
    fn from(value: rmp_serde::encode::Error) -> Self {
        builder::MessagePack {}.into_error(Box::new(value))
    }
}
impl From<surrealdb::Error> for Error {
    #[track_caller]
    fn from(value: surrealdb::Error) -> Self {
        builder::Database {}.into_error(value)
    }
}

#[derive(Snafu, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[snafu(visibility(pub))]
pub enum FrontendError {
    #[snafu(display("{msg}({code})"))]
    SubversionError { code: i32, msg: String },

    #[snafu(display("{detail}"))]
    UnexpectedError { detail: String },

    #[snafu(display("{detail}"))]
    TauriError { detail: String },

    #[snafu(display("{detail}"))]
    FlatlineError { detail: String },
}

impl From<Error> for FrontendError {
    #[track_caller]
    fn from(value: Error) -> Self {
        UnexpectedSnafu {
            detail: value.to_string(),
        }
        .build()
    }
}

impl From<flatline::error::Error> for FrontendError {
    fn from(value: flatline::error::Error) -> Self {
        Self::FlatlineError {
            detail: value.to_string(),
        }
    }
}

impl From<tauri::Error> for FrontendError {
    fn from(value: tauri::Error) -> Self {
        TauriSnafu {
            detail: value.to_string(),
        }
        .build()
    }
}

impl FrontendError {
    pub fn native_error(&self) -> *mut ffi::svn_error_t {
        let (code, msg) = match self {
            FrontendError::SubversionError { code, msg } => (*code, msg.as_str()),
            FrontendError::UnexpectedError { detail } => (
                SubversionErrorCode::CeaseInvocation.to_i32(),
                detail.as_str(),
            ),
            FrontendError::TauriError { detail } => (
                SubversionErrorCode::CeaseInvocation.to_i32(),
                detail.as_str(),
            ),
            FrontendError::FlatlineError { detail } => {
                (SubversionErrorCode::RaSvnIoError.to_i32(), detail.as_str())
            }
        };
        unsafe {
            let mut pool = apr::Pool::create();
            let msg = pool.string(msg).unwrap_or_default();

            let error =
                ffi::svn_error_create(code.try_into().unwrap_or_default(), Default::default(), msg);

            error
        }
    }
}

#[easy_ext::ext(FrontendErrorExtension)]
pub impl<T> Result<T, FrontendError> {
    fn native_error(&self) -> *mut ffi::svn_error_t {
        match self {
            Ok(_) => std::ptr::null_mut(),
            Err(e) => e.native_error(),
        }
    }
}
