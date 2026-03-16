//! A module for collecting Redis models into key-value pairs.

use crate::redis::RedisModel;

/// Простой трейт для конвертации в пары ссылок
pub trait AsRedisPairs<M: RedisModel> {
    fn as_pairs(&self) -> Vec<(&M::Key, &M::Value)>;
}

// Реализация для среза ссылок на модели
impl<'a, M> AsRedisPairs<M> for &'a [&'a M]
where
    M: RedisModel,
{
    fn as_pairs(&self) -> Vec<(&M::Key, &M::Value)> {
        self.iter().map(|m| (m.key_ref(), m.value_ref())).collect()
    }
}

// Реализация для массива ссылок фиксированной длины
impl<'a, M, const N: usize> AsRedisPairs<M> for [&'a M; N]
where
    M: RedisModel,
{
    fn as_pairs(&self) -> Vec<(&M::Key, &M::Value)> {
        self.iter().map(|m| (m.key_ref(), m.value_ref())).collect()
    }
}

// Реализация для одного элемента (удобно для set)
impl<'a, M> AsRedisPairs<M> for &'a M
where
    M: RedisModel,
{
    fn as_pairs(&self) -> Vec<(&M::Key, &M::Value)> {
        vec![(self.key_ref(), self.value_ref())]
    }
}

// Реализация для Vec ссылок
impl<'a, M> AsRedisPairs<M> for Vec<&'a M>
where
    M: RedisModel,
{
    fn as_pairs(&self) -> Vec<(&M::Key, &M::Value)> {
        self.iter().map(|m| (m.key_ref(), m.value_ref())).collect()
    }
}
