use derive_more::derive::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    // TBC
    #[from]
    NewSession(scylla::errors::NewSessionError),
    #[from]
    Prepare(scylla::errors::PrepareError),
    #[from]
    Execution(scylla::errors::ExecutionError),
    #[from]
    IntoRows(scylla::errors::IntoRowsResultError),
    #[from]
    Rows(scylla::errors::RowsError),
    #[from]
    Deserialization(scylla::errors::DeserializationError),
    #[from]
    UseKeyspace(scylla::errors::UseKeyspaceError),
    #[from]
    Charybdis(charybdis::errors::CharybdisError),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
