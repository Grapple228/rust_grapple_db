//! A module for collecting Redis models into key-value pairs.
//!
//! This module defines the `RedisModelCollector` trait, which is intended for types that can
//! aggregate multiple instances of a model implementing the `RedisModel` trait into a collection
//! of key-value pairs. This functionality is particularly useful for operations that involve
//! setting multiple values in Redis at once.
//!
//! # Overview
//!
//! The `RedisModelCollector` trait provides a method for converting a collection of models into
//! a vector of tuples, where each tuple contains a key and a value as converted to `ToRedisArgs`. This is essential
//! for preparing data to be stored in Redis efficiently.
//!
//! # Associated Types
//!
//! * `M` - The type of the model that implements the `RedisModel` trait.
//!
//! # Methods
//!
//! ## collect
//!
//! Converts the implementing type into a vector of tuples, where each tuple contains a key
//! and a value as converted to `ToRedisArgs`. This method is crucial for batch operations in Redis.
//!
//! # Example
//!
//! ```rust,no_run
//! use grapple_db::redis;
//! use grapple_db::redis::ToRedisArgs;
//! use grapple_db::redis::RedisModel;
//! use grapple_db::redis::RedisModelCollector;
//! use grapple_db::redis::macros::FromRedisValue;
//!
//! #[derive(Debug, serde::Serialize, serde::Deserialize, FromRedisValue)]
//! struct MyModel {
//!     key: String,
//!     value: String,
//! }
//!
//! impl RedisModel for MyModel {
//!     fn key(&self) -> Result<String, redis::Error> {
//!         Ok(self.key.clone())
//!     }
//! }
//!
//! struct MyModelCollector {
//!     models: Vec<MyModel>,
//! }
//!
//! impl RedisModelCollector<MyModel> for MyModelCollector {
//!     fn collect(&self) -> Vec<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
//!         self.models.iter()
//!             .map(|model| (model.key().unwrap().to_redis_args(), model.value().unwrap().to_redis_args()))
//!             .collect()
//!     }
//! }
//! ```

use deadpool_redis::redis::ToRedisArgs;

use crate::redis::RedisModel;

/// A trait for collecting Redis models into a vector of key-value pairs.
///
/// The `RedisModelCollector` trait is designed for types that can aggregate multiple instances
/// of a model implementing the `RedisModel` trait into a collection of key-value pairs. This
/// is particularly useful for operations that involve setting multiple values in Redis at once.
///
/// # Associated Types
///
/// * `M` - The type of the model that implements the `RedisModel` trait.
///
/// # Methods
///
/// ## collect
///
/// Converts the implementing type into a vector of tuples, where each tuple contains a key
/// and a value as converted to `ToRedisArgs`. This method is crucial for batch operations in Redis.
///
/// # Example
///
/// ```rust,no_run
/// use grapple_db::redis;
/// use grapple_db::redis::ToRedisArgs;
/// use grapple_db::redis::RedisModel;
/// use grapple_db::redis::RedisModelCollector;
/// use grapple_db::redis::macros::FromRedisValue;
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize, FromRedisValue)]
/// struct MyModel {
///     key: String,
///     value: String,
/// }
///
/// impl RedisModel for MyModel {
///     fn key(&self) -> Result<String, redis::Error> {
///         Ok(self.key.to_string())
///     }
/// }
///
/// struct MyModelCollector {
///     models: Vec<MyModel>,
/// }
///
/// impl RedisModelCollector<MyModel> for MyModelCollector {
///     fn collect(&self) -> Vec<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
///         self.models.iter()
///             .map(|model| (model.key().unwrap().to_redis_args(), model.value().unwrap().to_redis_args()))
///             .collect()
///     }
/// }
/// ```
pub trait RedisModelCollector<M>
where
    M: RedisModel,
{
    fn collect(&self) -> Vec<(Vec<Vec<u8>>, Vec<Vec<u8>>)>;
}

impl<'a, I, M> RedisModelCollector<M> for I
where
    M: RedisModel + 'a,
    I: AsRef<[&'a M]>,
{
    fn collect(&self) -> Vec<(Vec<Vec<u8>>, Vec<Vec<u8>>)> {
        self.as_ref()
            .iter()
            .filter_map(|m| match (m.key(), m.value()) {
                (Ok(key), Ok(value)) => Some((key.to_redis_args(), value.to_redis_args())),
                _ => None,
            })
            .collect()
    }
}
