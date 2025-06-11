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
    redis::{AsyncCommands, Expiry, FromRedisValue},
    Config, Connection, Pool,
};
use futures::future::join_all;
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
    /// Asynchronously retrieves a value from Redis using the provided key.
    ///
    /// This method fetches the value associated with the specified key from Redis. If the key exists, it returns
    /// the value deserialized into the type `V`. The type `V` must implement the `FromRedisValue` trait.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to a string slice that represents the key for which the value is to be retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<V>`, where `Some(value)` is the deserialized value if the key exists,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a type defined with trait `FromRedisValue` implemented
    /// # #[derive(Debug,Serialize, Deserialize, FromRedisValue)]
    /// # struct MyValue {
    /// #     a: u64,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let key = "some_key";
    ///     let result: Option<MyValue> = client.get(key).await?;
    ///
    ///     if let Some(value) = result {
    ///         println!("Retrieved value: {:?}", value);
    ///     } else {
    ///         println!("No value found for key: {}", key);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get<V>(&self, key: impl AsRef<str>) -> Result<Option<V>>
    where
        V: FromRedisValue,
    {
        let mut connection = self.connection().await?;
        Ok(connection.get(key.as_ref()).await?)
    }

    /// Asynchronously retrieves multiple values from Redis using the provided keys.
    ///
    /// This method fetches the values associated with the specified keys from Redis. It returns a vector of `Option<V>`,
    /// where each `Option` contains the deserialized value if the corresponding key exists, or `None` if it does not.
    /// The type `V` must implement the `FromRedisValue` trait.
    ///
    /// # Arguments
    ///
    /// * `keys` - An iterable collection of string slices representing the keys for which the values are to be retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Option<V>>`, where each element corresponds to a key in the input collection,
    /// with `Some(value)` for existing keys and `None` for non-existing keys.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a type defined with trait `FromRedisValue` implemented
    /// # #[derive(Debug,Serialize, Deserialize, FromRedisValue)]
    /// # struct MyValue {
    /// #     a: u64,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let keys = vec!["key1", "key2", "key3"];
    ///     let results: Vec<Option<MyValue>> = client.mget(&keys).await?;
    ///
    ///     for (key, value) in keys.iter().zip(results) {
    ///         match value {
    ///             Some(v) => println!("Retrieved value for {}: {:?}", key, v),
    ///             None => println!("No value found for key: {}", key),
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn mget<'a, K, T, V>(&self, keys: K) -> Result<Vec<Option<V>>>
    where
        V: FromRedisValue,
        K: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut connection = self.connection().await?;
        Ok(connection.mget(Self::map_keys(keys)).await?)
    }

    /// Asynchronously retrieves a value from Redis using the provided key and sets an expiration time.
    ///
    /// This method fetches the value associated with the specified key from Redis and sets an expiration time for that key.
    /// If the key exists, it returns the value deserialized into the type `V`. The type `V` must implement the `FromRedisValue` trait.
    /// The `expire_at` parameter specifies when the key should expire.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to a string slice that represents the key for which the value is to be retrieved.
    /// * `expire_at` - An `Expiry` value indicating when the key should expire.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<V>`, where `Some(value)` is the deserialized value if the key exists,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::Expiry;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a type defined with trait `FromRedisValue` implemented
    /// # #[derive(Debug,Serialize, Deserialize, FromRedisValue)]
    /// # struct MyValue {
    /// #     a: u64,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let key = "some_key";
    ///     let expire_at = Expiry::EX(60); // Set expiration to 60 seconds
    ///     let result: Option<MyValue> = client.get_ex(key, expire_at).await?;
    ///
    ///     if let Some(value) = result {
    ///         println!("Retrieved value: {:?}", value);
    ///     } else {
    ///         println!("No value found for key: {}", key);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_ex<V>(&self, key: impl AsRef<str>, expire_at: Expiry) -> Result<Option<V>>
    where
        V: FromRedisValue,
    {
        let mut connection = self.connection().await?;
        Ok(connection.get_ex(key.as_ref(), expire_at).await?)
    }

    /// Asynchronously retrieves a value from Redis using the provided key and deletes the key.
    ///
    /// This method fetches the value associated with the specified key from Redis and deletes the key in the process.
    /// If the key exists, it returns the value deserialized into the type `V`. The type `V` must implement the `FromRedisValue` trait.
    ///
    /// # Arguments
    ///
    /// * `key` - A reference to a string slice that represents the key for which the value is to be retrieved and deleted.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<V>`, where `Some(value)` is the deserialized value if the key exists,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a type defined with trait `FromRedisValue` implemented
    /// # #[derive(Debug,Serialize, Deserialize, FromRedisValue)]
    /// # struct MyValue {
    /// #     a: u64,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let key = "some_key";
    ///     let result: Option<MyValue> = client.get_del(key).await?;
    ///
    ///     if let Some(value) = result {
    ///         println!("Retrieved and deleted value: {:?}", value);
    ///     } else {
    ///         println!("No value found for key: {}", key);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_del<V>(&self, key: impl AsRef<str>) -> Result<Option<V>>
    where
        V: FromRedisValue,
    {
        let mut connection = self.connection().await?;
        Ok(connection.get_del(key.as_ref()).await?)
    }

    /// Asynchronously retrieves a value from Redis using the key from the provided model and sets a new value.
    ///
    /// This method fetches the value associated with the key derived from the provided model and replaces it with a new value
    /// obtained from the model. If the key exists, it returns the old value deserialized into the type `V`. The type `V` must
    /// implement the `FromRedisValue` trait. The type `M` must implement the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to a model that contains the key and the new value to be set in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<V>`, where `Some(value)` is the deserialized old value if the key exists,
    /// or `None` if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use grapple_db::redis::RedisModel;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Debug,Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> Result<String, redis::Error> {
    /// #         Ok(self.a.to_string())
    /// #     }
    /// #     fn value(&self) -> Result<String, redis::Error> {
    /// #         Ok((self.a + 1).to_string()) // Example of a new value
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let model = MyModel { a: 42 };
    ///     let old_value: Option<MyModel> = client.getset(&model).await?;
    ///
    ///     if let Some(value) = old_value {
    ///         println!("Retrieved and replaced old value: {:?}", value);
    ///     } else {
    ///         println!("No value found for key: {}", model.key()?);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn getset<M, V>(&self, model: &M) -> Result<Option<V>>
    where
        M: RedisModel,
        V: FromRedisValue,
    {
        let mut connection = self.connection().await?;
        Ok(connection.getset(model.key()?, model.value()?).await?)
    }
}

// Set
impl Client {
    /// Asynchronously sets a value in Redis using the key and value from the provided model.
    ///
    /// This method stores the key-value pair in Redis, where the key is derived from the provided model and the
    /// value is also obtained from the model. If the operation is successful, it returns a confirmation message.
    /// The type `M` must implement the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to a model that contains the key and value to be stored in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` confirmation message indicating the success of the operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use grapple_db::redis::RedisModel;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> Result<String, redis::Error> {
    /// #         Ok(self.a.to_string())
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
        Ok(connection.set(model.key()?, model.value()?).await?)
    }

    /// Asynchronously sets multiple values in Redis using the keys and values from the provided models.
    ///
    /// This method takes a collection of models that implement the `RedisModel` trait and stores their key-value
    /// pairs in Redis. If the operation is successful, it returns a confirmation message. The type `M` must
    /// implement the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `models` - A collection of models to be stored in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` confirmation message indicating the success of the operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::RedisModelCollector;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> Result<String, redis::Error> {
    /// #         Ok(self.a.to_string())
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

    /// Asynchronously sets multiple values in Redis using the keys and values from the provided models, only if the keys do not already exist.
    ///
    /// This method takes a collection of models that implement the `RedisModel` trait and stores their key-value
    /// pairs in Redis only if the keys are not already present. If the operation is successful and no keys were
    /// overwritten, it returns `true`. If any of the keys already exist, it returns `false`. The type `M` must
    /// implement the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `models` - A collection of models to be stored in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, where `true` indicates that the values were set successfully without
    /// overwriting existing keys, and `false` indicates that at least one key already existed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use grapple_db::redis::RedisModel;
    /// # use grapple_db::redis::RedisModelCollector;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> Result<String, redis::Error> {
    /// #         Ok(self.a.to_string())
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let models = vec![&MyModel { a: 1 }, &MyModel { a: 2 }];
    ///     let result: bool = client.mset_nx(&models).await?;
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

    /// Asynchronously sets a value in Redis using the key and value from the provided model, only if the key does not already exist.
    ///
    /// This method stores the key-value pair in Redis, where the key is derived from the provided model and the
    /// value is also obtained from the model. If the key already exists, it does not overwrite the existing value
    /// and returns `false`. If the operation is successful and the key was set, it returns `true`. The type `M`
    /// must implement the `RedisModel` trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to a model that contains the key and value to be stored in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, where `true` indicates that the value was set successfully, and `false`
    /// indicates that the key already existed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use grapple_db::redis::RedisModel;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> Result<String, redis::Error> {
    /// #         Ok(self.a.to_string())
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let model = MyModel { a: 42 };
    ///     let result: bool = client.set_nx(&model).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_nx<M>(&self, model: &M) -> Result<bool>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection.set_nx(model.key()?, model.value()?).await?)
    }

    /// Asynchronously sets a value in Redis using the key and value from the provided model, with an expiration time.
    ///
    /// This method stores the key-value pair in Redis, where the key is derived from the provided model and the
    /// value is also obtained from the model. The key will expire after the specified number of seconds. If the
    /// operation is successful, it returns a confirmation message. The type `M` must implement the `RedisModel`
    /// trait.
    ///
    /// # Arguments
    ///
    /// * `model` - A reference to a model that contains the key and value to be stored in Redis.
    /// * `secs` - The number of seconds after which the key should expire.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` confirmation message indicating the success of the operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    /// # use grapple_db::redis::macros::FromRedisValue;
    /// # use grapple_db::redis::RedisModel;
    /// # use serde::{Serialize, Deserialize};
    ///
    /// // Assuming you have a model defined with trait `RedisModel` implemented
    /// # #[derive(Serialize, Deserialize, FromRedisValue)]
    /// # struct MyModel {
    /// #     a: u64,
    /// # }
    /// #
    /// # impl RedisModel for MyModel {
    /// #     fn key(&self) -> Result<String, redis::Error> {
    /// #         Ok(self.a.to_string())
    /// #     }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let model = MyModel { a: 42 };
    ///     let result: String = client.set_ex(&model, 60).await?; // Set with 60 seconds expiration
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn set_ex<M>(&self, model: &M, secs: u64) -> Result<String>
    where
        M: RedisModel,
    {
        let mut connection = self.connection().await?;
        Ok(connection
            .set_ex(model.key()?, model.value()?, secs)
            .await?)
    }
}

// Del
impl Client {
    /// Asynchronously deletes a key from Redis.
    ///
    /// This method removes the specified key from Redis. If the key exists and is successfully deleted, it returns
    /// the `true`, if the key does not exist, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to be deleted from Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, which indicates if the entity was removed. This will be `true` if
    /// the key was successfully deleted, or `false` if the key did not exist.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let result: bool = client.del("my_key").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn del(&self, key: impl AsRef<str>) -> Result<bool> {
        let mut connection = self.connection().await?;
        Ok(connection.del(key.as_ref()).await?)
    }

    /// Asynchronously deletes multiple keys from Redis.
    ///
    /// This method removes the specified keys from Redis. It takes an iterable collection of keys and attempts to delete
    /// each one. The method returns the total number of keys that were successfully removed. If a key does not exist, it
    /// is simply ignored in the count.
    ///
    /// # Arguments
    ///
    /// * `keys` - An iterable collection of keys to be deleted from Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `usize`, which indicates the number of keys that were successfully removed. This count
    /// reflects only the keys that existed and were deleted.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let deleted_count: usize = client.mdel(vec!["key1", "key2", "key3"]).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn mdel<'a, K, T>(&self, keys: K) -> Result<usize>
    where
        K: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut futures = vec![];

        for key in Self::map_keys(keys) {
            futures.push(self.del(key));
        }

        // Wait to all operations complete
        let results = join_all(futures).await;

        // Return count of successfull operations, that returned true
        Ok(results
            .iter()
            .filter(|result| matches!(result, Ok(true)))
            .count())
    }
}

// Other
impl Client {
    /// Converts an iterable collection of keys into a vector of strings.
    ///
    /// This function takes an iterable collection of keys and maps each key to a `String`. It is useful for ensuring
    /// that the keys are in the correct format for further processing, such as deletion from Redis.
    ///
    /// # Arguments
    ///
    /// * `keys` - An iterable collection of keys, where each key can be referenced as a string.
    ///
    /// # Returns
    ///
    /// A `Vec<String>` containing the keys converted to `String` format.
    #[inline]
    fn map_keys<K, T>(keys: K) -> Vec<String>
    where
        K: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        keys.into_iter().map(|k| k.as_ref().to_string()).collect()
    }

    /// Asynchronously checks if a key exists in Redis.
    ///
    /// This method checks whether the specified key is present in Redis. If the key exists, it returns `true`;
    /// otherwise, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for existence in Redis.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, where `true` indicates that the key exists, and `false` indicates that it does not.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::redis::Client;
    /// # use grapple_db::redis;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let exists: bool = client.exists("my_key").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn exists(&self, key: impl AsRef<str>) -> Result<bool> {
        let mut connection = self.connection().await?;
        Ok(connection.exists(key.as_ref()).await?)
    }

    /// Asynchronously sends a ping command to Redis to check the connection.
    ///
    /// This method sends a ping command to the Redis server. If the server is reachable and responsive, it returns
    /// a confirmation message (usually "PONG"). If there is an issue with the connection, an error will be returned.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String`, which is the response from the Redis server, typically "PONG".
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
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn ping(&self) -> Result<String> {
        let mut connection = self.connection().await?;
        Ok(connection.ping().await?)
    }

    /// Asynchronously renames a key in Redis.
    ///
    /// This method renames the specified key to a new key. If the operation is successful, it returns a confirmation
    /// message. If the new key already exists, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `key` - The current key to be renamed.
    /// * `new_key` - The new key name to assign.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `String` confirmation message indicating the success of the operation.
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
    ///     let response: String = client.rename("old_key", "new_key").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn rename(&self, key: impl AsRef<str>, new_key: impl AsRef<str>) -> Result<String> {
        let mut connection = self.connection().await?;
        Ok(connection.rename(key.as_ref(), new_key.as_ref()).await?)
    }

    /// Asynchronously renames a key in Redis only if the new key does not already exist.
    ///
    /// This method attempts to rename the specified key to a new key name, but only if the new key does not already
    /// exist in Redis. If the operation is successful and the new key was created, it returns `true`. If the new
    /// key already exists, it does not perform the rename and returns `false`.
    ///
    /// # Arguments
    ///
    /// * `key` - The current key to be renamed.
    /// * `new_key` - The new key name to assign.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `bool`, where `true` indicates that the rename was successful, and `false` indicates
    /// that the new key already existed.
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
    pub async fn rename_nx(&self, key: impl AsRef<str>, new_key: impl AsRef<str>) -> Result<bool> {
        let mut connection = self.connection().await?;
        Ok(connection.rename_nx(key.as_ref(), new_key.as_ref()).await?)
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T> = super::Result<T>; // For tests.

    use std::time::Duration;

    use crate::redis;
    use crate::redis::macros::FromRedisValue;
    use crate::redis::RedisModel;
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
        fn key(&self) -> redis::Result<String> {
            Ok(self.key.clone())
        }
    }

    impl Tst {
        pub fn inc(mut self, value: u64) -> Self {
            self.a += value;
            self.b += value;

            self
        }

        pub fn default(key: impl AsRef<str>) -> Self {
            Self {
                key: key.as_ref().to_string(),
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
            client.mget(&[key1, key2]).await?
        );

        // Clear
        client.mdel(&[key1, key2]).await?;

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
        client.mdel(&[key1, key2]).await?;

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
            client.mget(&[key1, key2]).await?
        );

        assert!(!client.mset_nx(&[&model1_after, &model2_after]).await?);
        assert_eq!(
            vec![Some(model1_before.clone()), Some(model2_before.clone())],
            client.mget(&[key1, key2]).await?
        );

        // Clear
        client.mdel(&[key1, key2]).await?;

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

        assert!(client.del(key).await?);

        assert_eq!(None::<Tst>, client.get(key).await?);

        assert!(!client.del(key).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_redis_mdel() -> Result<()> {
        let client = get_client().await;

        let key1 = "test_redis_mdel1";
        let key2 = "test_redis_mdel2";

        // Create model
        let model1 = Tst::default(key1);
        let model2 = Tst::default(key2);

        client.mset(&[&model1, &model2]).await?;

        assert_eq!(Some(model1), client.get(key1).await?);
        assert_eq!(Some(model2), client.get(key2).await?);

        // Test
        assert_eq!(2, client.mdel(&[key1, key2]).await?);

        println!("{:?}", client.get::<String>(key1).await?);

        assert_eq!(None::<Tst>, client.get(key1).await?);
        assert_eq!(None::<Tst>, client.get(key2).await?);

        assert_eq!(0, client.mdel(&[key1, key2]).await?);

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

        assert_eq!(None, client.get::<Tst>(key).await?);
        assert_eq!(Some(fx_key_new_model), client.get(new_key).await?);

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

        assert_eq!(None, client.get::<Tst>(key).await?);
        assert_eq!(Some(fx_key_model.clone()), client.get(new_key).await?);

        let fx_key_new_model = Tst::default(key);
        client.set(&fx_key_new_model).await?;

        assert!(!client.rename_nx(new_key, key).await?);

        assert_eq!(Some(fx_key_model), client.get::<Tst>(key).await?);
        assert_eq!(Some(fx_key_new_model), client.get(new_key).await?);

        // Clear
        client.mdel(&[key, new_key]).await?;

        Ok(())
    }

    // endregion: --- OTHER TESTS
}

// endregion: --- Tests
