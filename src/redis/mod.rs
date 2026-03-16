//! Redis client implementation
//!
//! This module provides a comprehensive implementation of a Redis client
//! utilizing the `deadpool-redis` library for efficient data management and interaction.

mod client;
pub mod collector;
mod error;

pub mod pool {
    pub use deadpool_redis::*;
}

pub mod macros {
    pub use grapple_redis_macros::*;
}

pub use client::Client;
pub use deadpool_redis::redis::FromRedisValue;
pub use deadpool_redis::redis::*;
pub use error::{Error, Result};

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

// Базовый трейт для моделей, которые можно сохранять
pub trait RedisModel: Serialize {
    type Key: ToRedisArgs + Send + Sync;
    type Value: ToRedisArgs + Send + Sync;

    fn key(&self) -> Result<Self::Key>;
    fn key_ref(&self) -> &Self::Key;
    fn value(&self) -> Result<impl ToRedisArgs + Send + Sync> {
        Ok(serde_json::to_string(&self)?)
    }
    fn value_ref(&self) -> &Self::Value;
}

// Трейт для типов, которые можно читать из Redis
pub trait RedisRead: FromRedisValue + DeserializeOwned {}

// Автоматическая реализация для всех, кто имеет FromRedisValue + DeserializeOwned
impl<T> RedisRead for T where T: FromRedisValue + DeserializeOwned {}

// ЕДИНСТВЕННАЯ ОБЩАЯ РЕАЛИЗАЦИЯ ДЛЯ КОРТЕЖЕЙ
impl<K, V> RedisModel for (K, V)
where
    K: ToRedisArgs + Send + Sync + Clone + Serialize,
    V: ToRedisArgs + Send + Sync + Serialize,
    for<'a> &'a V: ToRedisArgs, // Позволяет работать с &[u8; N] как с &[u8]
{
    type Key = K;
    type Value = V;

    fn key(&self) -> Result<Self::Key> {
        Ok(self.0.clone())
    }

    fn key_ref(&self) -> &Self::Key {
        &self.0
    }

    fn value(&self) -> Result<impl ToRedisArgs + Send + Sync> {
        // Всегда возвращаем ссылку на значение
        Ok(&self.1)
    }

    fn value_ref(&self) -> &Self::Value {
        &self.1
    }
}

/// Обертка для пары ссылок (key, value) - если нужны явные ссылки
#[derive(Debug, Clone, Copy, Serialize)]
pub struct RedisPairRef<'a, K, V> {
    pub key: &'a K,
    pub value: &'a V,
}

impl<'a, K, V> RedisPairRef<'a, K, V> {
    pub fn new(key: &'a K, value: &'a V) -> Self {
        Self { key, value }
    }
}

// Реализация для RedisPairRef
impl<'a, K, V> RedisModel for RedisPairRef<'a, K, V>
where
    K: ToRedisArgs + Send + Sync + Serialize + Clone,
    V: ToRedisArgs + Send + Sync + Serialize,
    for<'b> &'b V: ToRedisArgs,
{
    type Key = K;
    type Value = V;

    fn key(&self) -> Result<Self::Key> {
        Ok(self.key.clone())
    }

    fn key_ref(&self) -> &Self::Key {
        self.key
    }

    fn value(&self) -> Result<impl ToRedisArgs + Send + Sync> {
        Ok(self.value)
    }

    fn value_ref(&self) -> &Self::Value {
        self.value
    }
}
