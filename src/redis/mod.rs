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

pub mod macros {
    pub use grapple_redis_macros::*;
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
/// use grapple_db::redis;
/// use grapple_db::redis::ToRedisArgs;
/// use grapple_db::redis::Result;
/// use grapple_db::redis::RedisModel;
/// use grapple_db::redis::macros::FromRedisValue;
///
/// #[derive(Debug, Deserialize, Serialize, FromRedisValue)]
/// struct MyModel {
///     id: String,
///     data: String,
/// }
///
/// impl RedisModel for MyModel {
///     fn key(&self) -> Result<String> {
///         Ok(self.id.to_string())
///     }
/// }
///
/// fn example(model: &MyModel) -> Result<Vec<Vec<u8>>> {
///     Ok(model.value()?.to_redis_args())
/// }
/// ```
pub trait RedisModel: FromRedisValue + Serialize + DeserializeOwned {
    fn key(&self) -> Result<String>;
    #[inline]
    fn value(&self) -> Result<impl ToRedisArgs + Send + Sync> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl<V> RedisModel for (String, V)
where
    V: Serialize + DeserializeOwned + FromRedisValue + Clone,
{
    #[inline]
    fn key(&self) -> Result<String> {
        Ok(self.0.clone())
    }

    #[inline]
    fn value(&self) -> Result<impl ToRedisArgs + Send + Sync> {
        Ok(serde_json::to_string(&self.1)?)
    }
}
