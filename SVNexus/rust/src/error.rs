use super::apr;
use super::subversion;

use snafu::Backtrace;
use snafu::IntoError;
use snafu::Snafu;
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

impl From<apache_avro::Error> for Error {
    fn from(value: apache_avro::Error) -> Self {
        todo!()
    }
}
