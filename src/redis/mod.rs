//! Redis client implementation
//!
//! This module provides a comprehensive implementation of a Redis client
//! utilizing the `deadpool-redis` library for efficient data management and interaction.
//! It includes various components necessary for establishing connections,
//! performing operations on Redis models, handling errors, and managing data retrieval.
//!
//! # Modules
//!
//! - `client`: Contains the implementation of the Redis client for interacting
//!   with the Redis database.
//! - `collector`: Provides the `RedisModelCollector` for collecting and managing
//!   Redis models for batch operations.
//! - `error`: Defines custom error types and result types for handling errors
//!   throughout the client.
//!
//! This module facilitates modular development and simplifies the maintenance
//! of the Redis client, allowing each component to be developed and tested
//! in isolation.

mod client;
mod collector;
mod error;

pub mod pool {
    pub use deadpool_redis::*;
}

pub use client::Client;
pub use collector::RedisModelCollector;
pub use deadpool_redis::redis::FromRedisValue;
pub use deadpool_redis::redis::*;
pub use error::{Error, Result};

use serde::{de::DeserializeOwned, Serialize};

/// A trait representing a Redis model
///
/// This trait extends the functionality of Redis models by requiring them to implement
/// the `FromRedisValue`, `Serialize` and `Deserialize` traits. It provides a method to retrieve
/// the key associated with the model and a default implementation for serializing the
/// model into a JSON string.
///
/// # Methods
///
/// * `key` - Returns a `String` representing the key for the Redis model.
/// * `value` - Serializes the model into a JSON string and returns it as a `Result`.
///
/// # Examples
///
/// ```rust,no_run
/// use serde::{Deserialize, Serialize};
/// use grapple_db::redis::FromRedisValue;
/// use grapple_db::redis::Result;
/// use grapple_db::redis::RedisModel;
/// use grapple_redis_macros::FromRedisValue;
///
/// #[derive(Debug, Deserialize, Serialize, FromRedisValue)]
/// struct MyModel {
///     id: String,
///     data: String,
/// }
///
/// impl RedisModel for MyModel {
///     fn key(&self) -> String {
///         self.id.clone()
///     }
/// }
///
/// fn example(model: &MyModel) -> Result<String> {
///     let json_value = model.value()?;
///     Ok(json_value)
/// }
/// ```
pub trait RedisModel: FromRedisValue + Serialize + DeserializeOwned {
    /// Returns a `String` representing the key for the Redis model.
    /// This key will be used for insertions into the Redis database.
    fn key(&self) -> String;
    /// Serializes the model into a JSON string and returns it as a `Result`.
    /// The serialized JSON will be inserted under the corresponding key in Redis.
    /// If serialization fails, it returns an error.
    fn value(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}
