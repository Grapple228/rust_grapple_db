//! `grapple_db` is a library designed to simplify interactions with various databases.
//! It offers a flexible architecture that allows you to select the database client you need and utilize it according to your requirements.
//! The library can be easily integrated into your project using feature flags.

// region:    --- Modules

use async_trait::async_trait;

// -- Modules

#[cfg(feature = "scylla")]
pub mod scylla;

// endregion: --- Modules

/// Trait for paginating through stream models.
///
/// This trait provides methods to fetch the next page of items and skip
/// pages in the stream. It requires the model type to be `Send` and `Sync` for
/// safe concurrent access.
///
/// # Methods
///
/// - `next_page`: Fetches the next page of items from the stream.
/// - `skip_page`: Skips the current page in the stream without retrieving items.
/// - `skip_pages`: Skips `page_count` pages in the stream without retrieving items.
/// - `page_items`: Returns the items of the current page.
#[async_trait]
pub trait Pagable<E>
where
    E: Send + Sync,
{
    /// Fetches the next page of items from the stream.
    ///
    /// This method clears the current page items and attempts to fill the vector
    /// with the next set of items from the stream. If no items are available,
    /// it returns `None`.
    ///
    /// # Returns
    ///
    /// An optional slice of items for the current page.
    async fn next_page(&mut self) -> Option<&[E]>;

    /// Skips the current page in the stream.
    ///
    /// This method clears the current page items and advances the stream by the
    /// number of items specified by `per_page`, without storing them.
    async fn skip_page(&mut self);

    /// Skips `page_count`  of pages in the stream.
    ///
    /// This method clears the current page items and advances the stream by the
    /// number of items specified by `per_page` multiplyed by `page_count`, without storing them.
    async fn skip_pages(&mut self, page_count: i32) {
        for _ in 0..page_count {
            self.skip_page().await;
        }
    }

    /// Returns the items of the current page.
    ///
    /// # Returns
    ///
    /// A slice of the items currently stored in the page.
    fn page_items(&self) -> &[E];
}
