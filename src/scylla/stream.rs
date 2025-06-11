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
//! use grapple_db::scylla::{model::Model, stream::CharybdisModelStream};
//! use grapple_db::scylla::stream::PagableCharybdisStream;
//! use grapple_db::Pagable;
//! use grapple_db::scylla::Client;
//! use grapple_db::scylla::operations::Find;
//!
//! // Assuming you have a `User` model defined with `Charybdis`
//! # #[grapple_db::scylla::macros::charybdis_model(
//! #       table_name = users,
//! #       partition_keys = [id],
//! #       clustering_keys = [],
//! #   )]
//! # #[derive(Debug, Default)]
//! # struct User {
//! #     id: String,
//! # }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::default().await?;
//!
//! // Initialize a Charybdis model stream
//! let stream = client.stream(User::find_all()).await?;
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
//!
//! Ok(())
//! # }
//! ```

use super::model::Model;
use async_trait::async_trait;
use futures::StreamExt;

#[allow(unused)]
pub use charybdis::stream::*;

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
/// use grapple_db::scylla::{model::Model, stream::CharybdisModelStream};
/// use grapple_db::scylla::stream::PagableCharybdisStream;
/// use grapple_db::Pagable;
/// # use grapple_db::scylla::Client;
/// # use grapple_db::scylla::macros::charybdis_model;
/// # use grapple_db::scylla::operations::Find;
///
/// // Assuming you have a User model defined
/// # #[charybdis_model(
/// #     table_name = users,
/// #     partition_keys = [id],
/// #     clustering_keys = [],
/// # )]
/// # #[derive(Debug, Default)]
/// # struct User {
/// #     id: String
/// # }
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let client = Client::default().await?;
///
/// // Creating a new paginated stream with a specified number of items per page
/// // Initialize stream from client
/// let stream = client.stream(User::find_all()).await?;
/// let mut paginated_stream = PagableCharybdisStream::new(stream, 10);
///
/// // Fetching the next page of items
/// if let Some(items) = paginated_stream.next_page().await {
///     for item in items {
///         // Process each item
///     }
/// }
///
/// # Ok(())
///
/// # }
/// ```
pub struct PagableCharybdisStream<E>
where
    E: Model + 'static,
{
    stream: CharybdisModelStream<E>,
    per_page: usize,
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
    pub fn new(stream: CharybdisModelStream<E>, per_page: usize) -> Self {
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

    #[inline]
    fn page_items(&self) -> &[E] {
        &self.page_items
    }
}
