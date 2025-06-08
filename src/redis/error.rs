use derive_more::derive::From;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

/// An enumeration representing various errors that can occur in the Redis client
///
/// This enum derives the `Debug` and `From` traits, allowing for easy debugging and
/// automatic conversion from specific error types into the `Error` type. Each variant
/// corresponds to a specific error that may arise during operations involving Redis.
///
/// # Variants
///
/// - `CreatePoolError` - Represents an error that occurs when creating a connection pool.
/// - `PoolError` - Represents an error that occurs while interacting with the connection pool.
/// - `Redis` - Represents an error that originates from the Redis library during operations.
/// - `Serde` - Represents an error that occurs during serialization or deserialization of data
///   using the Serde library.
#[derive(Debug, From)]
pub enum Error {
    #[from]
    CreatePoolError(deadpool_redis::CreatePoolError),

    #[from]
    PoolError(deadpool_redis::PoolError),

    #[from]
    Redis(super::RedisError),

    #[from]
    Serde(serde_json::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Error::CreatePoolError(create_pool_error) => {
                // Serialize the CreatePoolError as a string
                serializer.serialize_str(&create_pool_error.to_string())
            }
            Error::PoolError(pool_error) => {
                // Serialize the PoolError as a string
                serializer.serialize_str(&pool_error.to_string())
            }
            Error::Redis(redis_error) => {
                // Serialize the Redis error as a string
                serializer.serialize_str(&redis_error.to_string())
            }
            Error::Serde(serde_error) => {
                // Serialize the Serde error as a string
                serializer.serialize_str(&serde_error.to_string())
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
