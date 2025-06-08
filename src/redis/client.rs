//! A module for interacting with Redis using a connection pool.
//!
//! This module provides a `Client` struct that allows for asynchronous operations with a Redis
//! database. It utilizes a connection pool to manage connections efficiently, enabling multiple
//! concurrent operations without the overhead of establishing new connections for each request.
//!
//! # Overview
//!
//! The `Client` struct serves as the primary interface for interacting with Redis. It supports
//! various methods for connecting to Redis, retrieving connections, and performing operations
//! such as setting and getting values. The module is designed to work with the `deadpool-redis`
//! crate for connection pooling and management.
//!
//! # Features
//!
//! - **Connection Pooling**: Efficiently manage multiple connections to Redis, reducing latency
//!   and resource usage.
//! - **Asynchronous Operations**: Utilize Rust's async/await syntax for non-blocking I/O,
//!   allowing for high-performance applications.
//! - **Flexible Configuration**: Create clients from default settings, specific URLs, or existing
//!   connection pools.
//!
//! # Usage
//!
//! To use this module, you typically start by creating a `Client` instance, either using the
//! `default` method or by specifying a Redis URL. Once you have a `Client`, you can retrieve
//! connections and perform various Redis operations.
//!
//! # Example
//!
//! ```rust,no_run
//! use grapple_db::redis::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new Redis client with default settings
//!     let client = Client::default().await?;
//!
//!     // Use the client to perform Redis operations...
//!
//!     Ok(())
//! }
//! ```

use super::Result;
use crate::redis::{RedisModel, RedisModelCollector};
use deadpool_redis::{
    redis::{AsyncCommands, Expiry, ToRedisArgs},
    Config, Connection, Pool,
};
use std::fmt::Debug;

/// A Redis client for managing connections to a Redis database.
///
/// The `Client` struct provides an interface for interacting with a Redis database using a
/// connection pool. It allows for efficient management of multiple connections, enabling
/// asynchronous operations without the overhead of creating new connections for each request.
///
/// # Fields
///
/// * `pool` - A connection pool that manages the Redis connections. This pool allows for
///   concurrent access to the Redis database, improving performance and resource utilization.
///
/// # Implementations
///
/// The `Client` struct includes methods for creating instances from default settings, specific
/// URLs, or existing connection pools. It also provides methods for retrieving connections and
/// performing various Redis operations.
///
/// # Example
///
/// ```rust,no_run
/// use grapple_db::redis::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a new Redis client with default settings
///     let client = Client::default().await?;
///
///     // Use the client to perform Redis operations...
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pool: Pool,
}

// Constructors
impl Client {
    /// Creates a new `Client` instance with default settings, connecting to Redis at the default address.
    ///
    /// This asynchronous method initializes a `Client` by connecting to Redis at the specified
    /// default URL (`redis://127.0.0.1:6379`). It returns a `Result` containing the `Client` instance
    /// or an error if the connection fails.
    ///
    /// # Returns
    ///
    /// A `Result<Self>` where `Self` is the `Client` instance.
    pub async fn default() -> Result<Self> {
        Self::from_url("redis://127.0.0.1:6379").await
    }

    /// Creates a new `Client` instance from an existing connection pool.
    ///
    /// This method initializes a `Client` using the provided `Pool`. It is a synchronous method
    /// and does not perform any network operations.
    ///
    /// # Arguments
    ///
    /// * `pool` - The connection pool to use for Redis connections.
    ///
    /// # Returns
    ///
    /// A `Client` instance initialized with the provided pool.
    pub fn from_pool(pool: Pool) -> Self {
        Self { pool }
    }

    /// Creates a new `Client` instance by connecting to Redis at the specified URL.
    ///
    /// This asynchronous method initializes a `Client` by parsing the provided URL and creating
    /// a connection to Redis. It returns a `Result` containing the `Client` instance or an error
    /// if the connection fails.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the Redis server to connect to.
    ///
    /// # Returns
    ///
    /// A `Result<Self>` where `Self` is the `Client` instance.
    pub async fn from_url(url: &str) -> Result<Self> {
        let config = Config::from_url(url);
        Self::connect(&config).await
    }

    /// Establishes a connection to Redis using the provided configuration.
    ///
    /// This asynchronous method creates a connection pool based on the provided `Config` and
    /// returns a `Client` instance initialized with that pool. It returns a `Result` containing
    /// the `Client` instance or an error if the connection fails.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for connecting to Redis.
    ///
    /// # Returns
    ///
    /// A `Result<Self>` where `Self` is the `Client` instance.
    pub async fn connect(config: &Config) -> Result<Self> {
        let pool = config.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

        Ok(Self { pool })
    }

    /// Retrieves a connection from the connection pool.
    ///
    /// This asynchronous method fetches a connection from the pool associated with the `Client`.
    /// It returns a `Result` containing the `Connection` or an error if the retrieval fails.
    ///
    /// # Returns
    ///
    /// A `Result<Connection>` where `Connection` is the retrieved connection from the pool.
    pub async fn connection(&self) -> Result<Connection> {
        Ok(self.pool.get().await?)
    }
}

// Get
impl Client {
    /// Retrieves a value from Redis associated with the given key.
    ///
    /// This asynchronous method fetches the value of type `M` from Redis using the provided key.
    /// If the key does not exist, it returns `None`. The method requires that `M` implements the
    /// `RedisModel` trait, and `K` must implement `ToRedisArgs`, as well as be `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<M>`, where `Some(M)` is the value retrieved from Redis,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented`
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let value: Option<MyModel> = client.get("my_key").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get<K, M>(&self, key: K) -> Result<Option<M>>
    where
        M: RedisModel,
        K: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.get::<K, Option<M>>(key).await?)
    }

    /// Retrieves multiple values from Redis associated with the given keys.
    ///
    /// This asynchronous method fetches values of type `M` from Redis using the provided keys.
    /// If any of the keys do not exist, their corresponding entries in the result will be `None`.
    /// The method requires that `M` implements the `RedisModel` trait, and `K` must implement
    /// `ToRedisArgs`, as well as be `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `keys` - A reference to a slice of keys to look up in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Option<M>>`, where each entry is the value retrieved from Redis.
    /// If a key does not exist, its corresponding entry in the vector will be `None`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let keys = vec!["key1", "key2", "key3"];
    ///     let values: Vec<Option<MyModel>> = client.mget(keys).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn mget<K, M>(&self, keys: impl AsRef<[K]>) -> Result<Vec<Option<M>>>
    where
        M: RedisModel,
        K: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.mget(keys.as_ref()).await?)
    }

    /// Retrieves a value from Redis associated with the given key and sets an expiration time.
    ///
    /// This asynchronous method fetches the value of type `M` from Redis using the provided key
    /// and sets an expiration time for that key. If the key does not exist, it returns `None`.
    /// The method requires that `M` implements the `RedisModel` trait, and `K` must implement
    /// `ToRedisArgs`, as well as be `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up in Redis.
    /// * `expire_at` - An `Expiry` instance that specifies the expiration time for the key.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<M>`, where `Some(M)` is the value retrieved from Redis,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    /// # use grapple_db::redis::Expiry;
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let expire_at = Expiry::EX(60); // Expires in 60 seconds
    ///     let value: Option<MyModel> = client.get_ex("my_key", expire_at).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_ex<K, M>(&self, key: K, expire_at: Expiry) -> Result<Option<M>>
    where
        M: RedisModel,
        K: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.get_ex(key, expire_at).await?)
    }

    /// Retrieves a value from Redis associated with the given key and deletes the key.
    ///
    /// This asynchronous method fetches the value of type `M` from Redis using the provided key
    /// and deletes the key from Redis after retrieval. If the key does not exist, it returns `None`.
    /// The method requires that `M` implements the `RedisModel` trait, and `K` must implement
    /// `ToRedisArgs`, as well as be `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<M>`, where `Some(M)` is the value retrieved from Redis,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let value: Option<MyModel> = client.get_del("my_key").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_del<K, M>(&self, key: K) -> Result<Option<M>>
    where
        M: RedisModel,
        K: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.get_del(key).await?)
    }

    /// Retrieves the current value associated with the key of the given model and sets a new value.
    ///
    /// This asynchronous method fetches the current value of type `M` from Redis using the key
    /// defined in the provided model, and then updates the key with the new value from the model.
    /// If the key does not exist, it returns `None`. The method requires that `M` implements the
    /// `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the model instance containing the key and the new value to set.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<M>`, where `Some(M)` is the current value retrieved from Redis,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let model = MyModel { a: 42 };
    ///     let current_value: Option<MyModel> = client.getset(&model).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn getset<M>(&self, model: &M) -> Result<Option<M>>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection.getset(model.key(), model.value()?).await?)
    }
}

// Set
impl Client {
    /// Sets multiple values in Redis associated with the keys of the given models.
    ///
    /// This asynchronous method takes a collection of models and sets their values in Redis using
    /// the keys defined in each model. If any of the models do not have a valid key or value, the
    /// operation may fail. The method requires that `M` implements the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `models` - A collection of model instances to be set in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` indicating the status of the operation. On success, it
    /// returns `OK` from Redis.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let models = vec![&MyModel { a: 1 }, &MyModel { a: 2 }];
    ///     let result: String = client.mset(&models).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn mset<M>(&self, models: impl RedisModelCollector<M>) -> Result<String>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection.mset(&models.collect()).await?)
    }

    /// Sets a value in Redis associated with the key of the given model.
    ///
    /// This asynchronous method takes a model and sets its value in Redis using the key defined
    /// in the model. If the operation is successful, it returns an `OK` from Redis.
    /// The method requires that `M` implements the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the model instance containing the key and the value to set.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` indicating the status of the operation. On success, it
    /// returns `OK` from Redis.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let model = MyModel { a: 42 };
    ///     let result: String = client.set(&model).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set<M>(&self, model: &M) -> Result<String>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection.set(model.key(), model.value()?).await?)
    }

    /// Sets multiple values in Redis associated with the keys of the given models only if the keys do not already exist.
    ///
    /// This asynchronous method takes a collection of models and sets their values in Redis using
    /// the keys defined in each model, but only if those keys are not already present in Redis.
    /// If any of the models have a key that already exists, the operation will not set any values.
    /// The method requires that `M` implements the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `models` - A collection of model instances to be set in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, which indicates whether the operation was successful. It returns
    /// `true` if all values were set successfully, or `false` if any key already existed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let models = vec![&MyModel { a: 1 }, &MyModel { a: 2 }];
    ///     let success: bool = client.mset_nx(&models).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn mset_nx<M>(&self, models: impl RedisModelCollector<M>) -> Result<bool>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection.mset_nx(&models.collect()).await?)
    }

    /// Sets a value in Redis associated with the key of the given model only if the key does not already exist.
    ///
    /// This asynchronous method takes a model and sets its value in Redis using the key defined
    /// in the model, but only if that key is not already present in Redis. If the operation is
    /// successful, it returns `true` from Redis, `false` in other cases. The method requires that `M` implements
    /// the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the model instance containing the key and the value to set.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, which indicates whether the operation was successful. It returns
    /// `true` if all values were set successfully, or `false` if any key already existed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let model = MyModel { a: 42 };
    ///     let success: bool = client.set_nx(&model).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_nx<M>(&self, model: &M) -> Result<bool>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection.set_nx(model.key(), model.value()?).await?)
    }

    /// Sets a value in Redis associated with the key of the given model and sets an expiration time.
    ///
    /// This asynchronous method takes a model and sets its value in Redis using the key defined
    /// in the model, along with an expiration time specified in seconds. If the operation is
    /// successful, it returns `OK` from Redis. The method requires that `M` implements
    /// the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to the model instance containing the key and the value to set.
    /// * `secs` - The expiration time in seconds for the key.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` indicating the status of the operation. On success, it
    /// returns `OK` from Redis.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::FromRedisValue;
    /// # use grapple_redis_macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> String {
    /// #         self.a.to_string()
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let model = MyModel { a: 42 };
    ///     let result: String = client.set_ex(&model, 60).await?; // Expires in 60 seconds
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_ex<M>(&self, model: &M, secs: u64) -> Result<String>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection.set_ex(model.key(), model.value()?, secs).await?)
    }
}

// Del
impl Client {
    /// Deletes the specified key from Redis.
    ///
    /// This asynchronous method removes the value associated with the given key from Redis. If the
    /// key exists and is successfully deleted, it returns the number of keys that were removed (1).
    /// If the key does not exist, it returns 0. The method requires that `K` implements the
    /// `ToRedisArgs` trait, as well as being `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to be deleted from Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `usize`, which indicates the number of keys that were removed. This
    /// will be 1 if the key was successfully deleted, or 0 if the key did not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let deleted_count: usize = client.del("my_key").await?;
    ///
    ///     println!("Number of keys deleted: {}", deleted_count);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn del<K>(&self, key: K) -> Result<usize>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.del(key).await?)
    }
}

// Other
impl Client {
    /// Checks if the specified key exists in Redis.
    ///
    /// This asynchronous method verifies whether the given key is present in Redis. It returns
    /// `true` if the key exists and `false` if it does not. The method requires that `K` implements
    /// the `ToRedisArgs` trait, as well as being `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for existence in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, which indicates whether the key exists (`true`) or not (`false`).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let key_exists: bool = client.exists("my_key").await?;
    ///
    ///     if key_exists {
    ///         println!("The key exists in Redis.");
    ///     } else {
    ///         println!("The key does not exist in Redis.");
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn exists<K>(&self, key: K) -> Result<bool>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.exists(key).await?)
    }

    /// Sends a PING command to the Redis server to check connectivity.
    ///
    /// This asynchronous method sends a PING command to the Redis server and expects a PONG response.
    /// It is useful for verifying that the connection to the Redis server is active and functioning.
    /// The method returns a `Result` containing a `String` with the response from the server.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String`, which will be "PONG" if the server is reachable and responsive.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let response: String = client.ping().await?;
    ///     println!("Response from Redis: {}", response);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn ping(&self) -> Result<String> {
        let mut connection = self.connection().await?;
        Ok(connection.ping().await?)
    }

    /// Renames a key in Redis.
    ///
    /// This asynchronous method renames the specified key to a new key. If the operation is successful,
    /// it returns `OK` from Redis. If the new key already exists, the operation will fail.
    /// The method requires that both `K` and `N` implement the `ToRedisArgs` trait, as well as being
    /// `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `key` - The current key to be renamed.
    /// * `new_key` - The new key name to assign.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String`, which indicates the status of the operation. On success,
    /// it returns `OK` from Redis.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let result: String = client.rename("old_key", "new_key").await?;
    ///     println!("Rename result: {}", result);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn rename<K, N>(&self, key: K, new_key: N) -> Result<String>
    where
        K: ToRedisArgs + Send + Sync,
        N: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.rename(key, new_key).await?)
    }

    /// Renames a key in Redis only if the new key does not already exist.
    ///
    /// This asynchronous method renames the specified key to a new key, but only if the new key
    /// does not already exist in Redis. If the operation is successful, it returns `true`.
    /// If the new key already exists, the operation will not rename the key
    /// and will return `false`. The method requires that both `K` and `N` implement the `ToRedisArgs` trait,
    /// as well as being `Send` and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `key` - The current key to be renamed.
    /// * `new_key` - The new key name to assign.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, which indicates if operation was successfull. This
    /// will be `true` if the key was successfully renamed, or `false` if the new key already exists.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let success: bool = client.rename_nx("old_key", "new_key").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn rename_nx<K, N>(&self, key: K, new_key: N) -> Result<bool>
    where
        N: ToRedisArgs + Send + Sync,
        K: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connection().await?;
        Ok(connection.rename_nx(key, new_key).await?)
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T> = super::Result<T>; // For tests.

    use std::time::Duration;

    use crate::redis::RedisModel;
    use grapple_redis_macros::FromRedisValue;
    use serde::{Deserialize, Serialize};

    use super::*;

    // region:    --- Tst Struct

    #[derive(Debug, Clone, Serialize, Deserialize, FromRedisValue, PartialEq)]
    struct Tst {
        key: String,
        a: u64,
        b: u64,
    }

    impl RedisModel for Tst {
        fn key(&self) -> String {
            self.key.clone()
        }
    }

    impl Tst {
        pub fn inc(mut self, value: u64) -> Self {
            self.a += value;
            self.b += value;

            self
        }

        pub fn default(key: &str) -> Self {
            Self {
                key: key.to_string(),
                a: 3,
                b: 4,
            }
        }
    }

    // endregion: --- Tst Struct

    async fn get_client() -> Client {
        Client::default().await.unwrap()
    }

    // region:    --- GET TESTS

    #[tokio::test]
    async fn test_redis_get() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_get";

        // Create model
        let fx_model = Tst::default(key);
        client.set(&fx_model).await?;

        // Test
        assert_eq!(Some(fx_model), client.get(key).await?);

        // Clear
        client.del(key).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_mget() -> Result<()> {
        let client = get_client().await;

        let key1 = "test_redis_mget1";
        let key2 = "test_redis_mget2";

        // Create model
        let model1 = Tst::default(key1);
        let model2 = Tst::default(key2);
        client.mset(&[&model1, &model2]).await?;

        // Test
        assert_eq!(
            vec![Some(model1.clone()), Some(model2.clone())],
            client.mget::<_, Tst>(&[key1, key2]).await?
        );

        // Clear
        client.del(key1).await?;
        client.del(key2).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_get_ex() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_get_ex";

        // Create model
        let fx_model = Tst::default(key);
        client.set(&fx_model).await?;

        // Test
        assert_eq!(Some(fx_model), client.get_ex(key, Expiry::EX(2)).await?);

        tokio::time::sleep(Duration::from_secs(3)).await;

        assert_eq!(None::<Tst>, client.get(key).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_get_del() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_get_del";

        // Create model
        let fx_model = Tst::default(key);
        client.set(&fx_model).await?;

        // Test
        assert_eq!(Some(fx_model), client.get_del(key).await?);

        assert_eq!(None::<Tst>, client.get(key).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_getset() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_getset";

        // Create model
        let to_get = Tst::default(key);
        let to_set = Tst::default(key).inc(5);

        client.set(&to_get).await?;

        // Test
        assert_eq!(Some(to_get), client.getset(&to_set).await?);
        assert_eq!(Some(to_set), client.get(key).await?);

        // Clear
        client.del(key).await?;

        Ok(())
    }

    // endregion: --- GET TESTS

    // region:    --- SET TESTS

    #[tokio::test]
    async fn test_redis_set() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_set";

        // Create model
        let fx_model = Tst::default(key);

        // Test
        assert_eq!("OK", client.set(&fx_model).await?);
        assert_eq!(Some(fx_model), client.get(key).await?);

        // Clear
        client.del(key).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_mset() -> Result<()> {
        let client = get_client().await;

        let key1 = "test_redis_mset1";
        let key2 = "test_redis_mset2";

        // Create model
        let model1 = Tst::default(key1);
        let model2 = Tst::default(key2);

        // Test
        assert_eq!("OK", client.mset(&[&model1, &model2]).await?);

        assert_eq!(Some(model1), client.get(key1).await?);
        assert_eq!(Some(model2), client.get(key2).await?);

        // Clear
        client.del(key1).await?;
        client.del(key2).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_set_ex() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_set_ex";

        // Create model
        let fx_model = Tst::default(key);

        // Test
        assert_eq!("OK", client.set_ex(&fx_model, 2).await?);

        assert_eq!(Some(fx_model), client.get(key).await?);
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert_eq!(None::<Tst>, client.get(key).await?);

        // Clear
        client.del(key).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_set_nx() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_set_nx1";

        // Create model
        let model1 = Tst::default(key);
        let model2 = Tst::default(key).inc(5);

        // Test
        assert!(client.set_nx(&model1).await?);

        assert_eq!(Some(model1.clone()), client.get(key).await?);

        assert!(!client.set_nx(&model2).await?);

        assert_eq!(Some(model1), client.get(key).await?);

        // Clear
        client.del(key).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_mset_nx() -> Result<()> {
        let client = get_client().await;

        let key1 = "test_redis_mset_nx1";
        let key2 = "test_redis_mset_nx2";

        // Create model

        let model1_before = Tst::default(key1);
        let model2_before = Tst::default(key2);

        let model1_after = Tst::default(key1).inc(3);
        let model2_after = Tst::default(key2).inc(3);

        // Test
        assert!(client.mset_nx(&[&model1_before, &model2_before]).await?);
        assert_eq!(
            vec![Some(model1_before.clone()), Some(model2_before.clone())],
            client.mget::<_, Tst>(&[key1, key2]).await?
        );

        assert!(!client.mset_nx(&[&model1_after, &model2_after]).await?);
        assert_eq!(
            vec![Some(model1_before.clone()), Some(model2_before.clone())],
            client.mget::<_, Tst>(&[key1, key2]).await?
        );

        // Clear
        client.del(key1).await?;
        client.del(key2).await?;

        Ok(())
    }

    // endregion: --- SET TESTS

    // region:    --- DEL TESTS

    #[tokio::test]
    async fn test_redis_del() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_del";

        // Create model
        let fx_model = Tst::default(key);
        client.set(&fx_model).await?;

        // Test
        assert_eq!(Some(fx_model), client.get(key).await?);

        assert_eq!(1, client.del(key).await?);

        assert_eq!(None::<Tst>, client.get(key).await?);

        Ok(())
    }

    // endregion: --- DEL TESTS

    // region:    --- OTHER TESTS

    #[tokio::test]
    async fn test_redis_exists() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_exists";

        assert!(!client.exists(key).await?);

        // Create model
        let fx_model = Tst::default(key);
        client.set(&fx_model).await?;

        assert!(client.exists(key).await?);

        // Clear
        client.del(key).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_ping() -> Result<()> {
        let client = get_client().await;

        assert_eq!("PONG", client.ping().await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_rename() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_rename";
        let new_key = "test_redis_rename_new";

        // Create model
        let fx_key_model = Tst::default(key);
        let fx_key_new_model = Tst::default(key);

        client.set(&fx_key_model).await?;
        client.set(&fx_key_new_model).await?;

        assert_eq!("OK", client.rename(key, new_key).await?);

        // Clear
        client.del(key).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_rename_nx() -> Result<()> {
        let client = get_client().await;

        let key = "test_redis_rename_nx";
        let new_key = "test_redis_rename_nx_new";

        // Create model
        let fx_key_model = Tst::default(key);
        client.set(&fx_key_model).await?;

        // Test
        assert!(client.rename_nx(key, new_key).await?);

        let fx_key_new_model = Tst::default(key);
        client.set(&fx_key_new_model).await?;

        assert!(!client.rename_nx(new_key, key).await?);

        // Clear
        client.del(key).await?;
        client.del(new_key).await?;

        Ok(())
    }

    // endregion: --- OTHER TESTS
}

// endregion: --- Tests
