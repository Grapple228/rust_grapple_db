//! Scylla client implementation based on `Charybdis` ORM
//!
//! This module provides a comprehensive implementation of a ScyllaDB client
//! utilizing the `Charybdis` ORM for efficient data management and interaction.
//! It includes various components necessary for establishing connections,
//! performing CRUD operations, handling errors, and managing data streams.
//!
//! # Modules
//!
//! - `client`: Contains the implementation of the Scylla client for interacting
//!   with the database.
//! - `connection`: Defines parameters and methods for establishing and managing
//!   connections to the ScyllaDB cluster.
//! - `crud`: Provides the `CrudParams` struct for configuring CRUD operations,
//!   including consistency levels and timeouts.
//! - `error`: Defines custom error types and result types for handling errors
//!   throughout the client.
//! - `stream`: Implements the `PagableCharybdisStream` for paginated access
//!   to data streams from the database.
//!
//! This module facilitates modular development and simplifies the maintenance
//! of the Scylla client, allowing each component to be developed and tested
//! in isolation.

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
