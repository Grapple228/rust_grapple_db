// region:    --- Modules

mod client;
mod connection;
mod crud;
mod error;
mod stream;

pub use client::Client;
pub use connection::ConnectionParams;
pub use crud::CrudParams;
pub use error::{Error, Result};
pub use stream::PagableCharybdisStream;

// endregion: --- Modules
