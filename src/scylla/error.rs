use derive_more::derive::From;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

/// An enumeration representing various errors that can occur in the application
///
/// This enum derives the `Debug` and `From` traits, allowing for easy debugging and
/// automatic conversion from specific error types into the `Error` type. Each variant
/// corresponds to a specific error that may arise during operations involving ScyllaDB
/// or Charybdis.
///
/// # Variants
///
/// - `NewSession` - Represents an error that occurs when creating a new session.
/// - `Prepare` - Represents an error that occurs during the preparation of a query.
/// - `Execution` - Represents an error that occurs during the execution of a query.
/// - `IntoRows` - Represents an error that occurs when converting results into rows.
/// - `Rows` - Represents an error related to row operations.
/// - `Deserialization` - Represents an error that occurs during deserialization of data.
/// - `UseKeyspace` - Represents an error that occurs when using a specific keyspace.
/// - `Charybdis` - Represents an error from the Charybdis library.
#[derive(Debug, From)]
pub enum Error {
    // TBC
    #[from]
    NewSession(charybdis::scylla::errors::NewSessionError),
    #[from]
    Prepare(charybdis::scylla::errors::PrepareError),
    #[from]
    Execution(charybdis::scylla::errors::ExecutionError),
    #[from]
    IntoRows(charybdis::scylla::errors::IntoRowsResultError),
    #[from]
    Rows(charybdis::scylla::errors::RowsError),
    #[from]
    Deserialization(charybdis::scylla::errors::DeserializationError),
    #[from]
    UseKeyspace(charybdis::scylla::errors::UseKeyspaceError),
    #[from]
    Charybdis(charybdis::errors::CharybdisError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the error based on its variant
        match self {
            Error::NewSession(new_session_error) => {
                // Serialize the NewSession error as a string
                serializer.serialize_str(&new_session_error.to_string())
            }
            Error::Prepare(prepare_error) => {
                // Serialize the Prepare error as a string
                serializer.serialize_str(&prepare_error.to_string())
            }
            Error::Execution(execution_error) => {
                // Serialize the Execution error as a string
                serializer.serialize_str(&execution_error.to_string())
            }
            Error::IntoRows(into_rows_error) => {
                // Serialize the IntoRows error as a string
                serializer.serialize_str(&into_rows_error.to_string())
            }
            Error::Rows(rows_error) => {
                // Serialize the Rows error as a string
                serializer.serialize_str(&rows_error.to_string())
            }
            Error::Deserialization(deserialization_error) => {
                // Serialize the Deserialization error as a string
                serializer.serialize_str(&deserialization_error.to_string())
            }
            Error::UseKeyspace(use_keyspace_error) => {
                // Serialize the UseKeyspace error as a string
                serializer.serialize_str(&use_keyspace_error.to_string())
            }
            Error::Charybdis(charybdis_error) => {
                // Serialize the Charybdis error as a string
                serializer.serialize_str(&charybdis_error.to_string())
            }
        }
    }
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
