//! A module for paginating Charybdis model streams.
//!
//! This module provides the `PagableCharybdisStream` struct, which allows for
//! efficient pagination of items retrieved from a Charybdis model stream. It
//! encapsulates the logic for managing the current state of the stream, including
//! fetching the next set of items and skipping pages as needed.
//!
//! The `Pagable` trait is implemented for `PagableCharybdisStream`, enabling
//! asynchronous operations to retrieve and manage paginated data. This module
//! is particularly useful for applications that require handling large datasets
//! in a memory-efficient manner, allowing users to process items in manageable
//! chunks.
//!
//! # Examples
//!
//! ```rust,no_run
//! use charybdis::{model::Model, stream::CharybdisModelStream};
//! use crate::PagableCharybdisStream;
//!
//! // Initialize a Charybdis model stream
//! let stream = CharybdisModelStream::new(...); // Assume this initializes a stream
//!
//! // Create a paginated stream with 10 items per page
//! let mut paginated_stream = PagableCharybdisStream::new(stream, 10);
//!
//! // Fetch and process items page by page
//! while let Some(items) = paginated_stream.next_page().await {
//!     for item in items {
//!         // Process each item
//!     }
//! }
//! ```

use async_trait::async_trait;
use charybdis::{model::Model, stream::CharybdisModelStream};
use futures::StreamExt;

use crate::Pagable;

/// A paginated stream for Charybdis models.
///
/// This struct provides a way to paginate through a stream of Charybdis models,
/// allowing users to retrieve a specified number of items per page. It manages
/// the internal state of the stream and provides methods to fetch the next page
/// or skip pages as needed.
///
/// # Fields
///
/// - `stream`: The underlying Charybdis model stream from which items are fetched.
/// - `per_page`: The number of items to retrieve per page.
/// - `page_items`: A vector that holds the items of the current page.
///
/// # Examples
///
/// ```rust,no_run
/// use charybdis::{model::Model, stream::CharybdisModelStream};
/// use crate::PagableCharybdisStream;
///
/// // Creating a new paginated stream with a specified number of items per page
/// let stream = CharybdisModelStream::new(...); // Assume this initializes a stream
/// let mut paginated_stream = PagableCharybdisStream::new(stream, 10);
///
/// // Fetching the next page of items
/// if let Some(items) = paginated_stream.next_page().await {
///     for item in items {
///         // Process each item
///     }
/// }
/// ```
pub struct PagableCharybdisStream<E>
where
    E: Model + 'static,
{
    stream: CharybdisModelStream<E>,
    per_page: i32,
    page_items: Vec<E>,
}

impl<E> PagableCharybdisStream<E>
where
    E: Model + 'static,
{
    /// Creates a new instance of `PagableCharybdisStream`.
    ///
    /// This function initializes the stream with the specified number of items per page,
    /// preparing it for pagination.
    ///
    /// # Parameters
    ///
    /// - `stream`: The Charybdis model stream to paginate.
    /// - `per_page`: The number of items to retrieve per page.
    ///
    /// # Returns
    ///
    /// A new instance of `PagableCharybdisStream`.
    pub fn new(stream: CharybdisModelStream<E>, per_page: i32) -> Self {
        Self {
            stream,
            per_page,
            page_items: Vec::with_capacity(per_page as usize),
        }
    }
}

#[async_trait]
impl<E> Pagable<E> for PagableCharybdisStream<E>
where
    E: Model + 'static + Send + Sync,
{
    async fn next_page(&mut self) -> Option<&[E]> {
        self.page_items.clear();
        let mut available = 0;

        for _ in 0..self.per_page {
            match self.stream.next().await {
                Some(Ok(item)) => {
                    self.page_items.push(item);
                    available += 1;
                }
                _ => break,
            }
        }

        if available == 0 {
            None
        } else {
            Some(self.page_items())
        }
    }

    async fn skip_page(&mut self) {
        self.page_items.clear();

        for _ in 0..self.per_page {
            if self.stream.next().await.is_none() {
                break;
            }
        }
    }

    fn page_items(&self) -> &[E] {
        &self.page_items
    }
}
