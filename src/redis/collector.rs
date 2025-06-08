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
//! a vector of tuples, where each tuple contains a key and a value as strings. This is essential
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
//! and a value as strings. This method is crucial for batch operations in Redis.
//!
//! # Example
//!
//! ```rust,no_run
//! use grapple_db::redis::RedisModel;
//! use grapple_db::redis::RedisModelCollector;
//! use grapple_db::redis::FromRedisValue;
//! use grapple_redis_macros::FromRedisValue;
//!
//! #[derive(Debug, serde::Serialize, serde::Deserialize, FromRedisValue)]
//! struct MyModel {
//!     key: String,
//!     value: String,
//! }
//!
//! impl RedisModel for MyModel {
//!     fn key(&self) -> String {
//!         self.key.clone()
//!     }
//! }
//!
//! struct MyModelCollector {
//!     models: Vec<MyModel>,
//! }
//!
//! impl RedisModelCollector<MyModel> for MyModelCollector {
//!     fn collect(&self) -> Vec<(String, String)> {
//!         self.models.iter()
//!             .map(|model| (model.key(), model.value().unwrap()))
//!             .collect()
//!     }
//! }
//! ```

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
/// and a value as strings. This method is essential for preparing data to be stored in Redis.
///
/// # Example
///
/// ```rust,no_run
/// use grapple_db::redis::RedisModel;
/// use grapple_db::redis::RedisModelCollector;
/// use grapple_db::redis::FromRedisValue;
/// use grapple_redis_macros::FromRedisValue;
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize, FromRedisValue)]
/// struct MyModel {
///     key: String,
///     value: String,
/// }
///
/// impl RedisModel for MyModel {
///     fn key(&self) -> String {
///         self.key.clone()
///     }
/// }
///
/// struct MyModelCollector {
///     models: Vec<MyModel>,
/// }
///
/// impl RedisModelCollector<MyModel> for MyModelCollector {
///     fn collect(&self) -> Vec<(String, String)> {
///         self.models.iter()
///             .map(|model| (model.key(), model.value().unwrap()))
///             .collect()
///     }
/// }
/// ```
pub trait RedisModelCollector<M>
where
    M: RedisModel,
{
    fn collect(&self) -> Vec<(String, String)>;
}

impl<'a, I, M> RedisModelCollector<M> for I
where
    M: RedisModel + 'a,
    I: AsRef<[&'a M]>,
{
    fn collect(&self) -> Vec<(String, String)> {
        self.as_ref()
            .iter()
            .filter_map(|m| {
                if let Ok(value) = m.value() {
                    Some((m.key().clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}
