// region:    --- Modules

use async_trait::async_trait;

// -- Modules

#[cfg(feature = "scylla")]
pub mod scylla;

// endregion: --- Modules

#[async_trait]
pub trait Pagable<E>
where
    E: Send + Sync,
{
    async fn next_page(&mut self) -> Option<&[E]>;
    async fn skip_page(&mut self);
    async fn skip_pages(&mut self, page_count: i32) {
        for _ in 0..page_count {
            self.skip_page().await;
        }
    }
    fn page_num(&self) -> i32;
    fn page_items(&self) -> &[E];
}

pub trait WithId<I>: Default {
    fn id(&self) -> I;
    fn with_id(id: I) -> Self;
}

pub trait ValueBy<I>: WithId<I> {}
