//! A module for managing CRUD parameters in Charybdis operations.
//!
//! This module provides the `CrudParams` struct, which encapsulates the
//! configuration options for performing Create, Read, Update, and Delete
//! (CRUD) operations with Charybdis. It allows users to specify consistency
//! levels, timeouts, and timestamps for their operations, ensuring that
//! these settings are consistently applied across different database
//! interactions.
//!
//! The module includes methods to apply these parameters to both batch
//! operations and individual queries, facilitating a streamlined approach
//! to managing database interactions in a scalable manner.
//!
//! # Examples
//!
//! ```rust,no_run
//! use std::time::Duration;
//! use charybdis::{batch::CharybdisModelBatch, query::CharybdisQuery};
//! use scylla::statement::Consistency;
//!
//! // Creating CRUD parameters with specific settings
//! let params = CrudParams {
//!     consistency: Consistency::Quorum,
//!     timeout: Some(Duration::from_secs(5)),
//!     timestamp: Some(1625078400),
//! };
//!
//! // Applying parameters to a batch operation
//! let batch = CharybdisModelBatch::new(...); // Assume this initializes a batch
//! let configured_batch = params.apply_batch(batch);
//!
//! // Applying parameters to a query
//! let query = CharybdisQuery::new(...); // Assume this initializes a query
//! let configured_query = params.apply_query(query);
//! ```

use charybdis::{
    batch::{CharybdisModelBatch, ModelBatch},
    model::Model,
    query::{CharybdisQuery, QueryExecutor},
};
use scylla::{serialize::row::SerializeRow, statement::Consistency};
use std::time::Duration;

/// Parameters for CRUD operations in Charybdis.
///
/// This struct encapsulates the configuration options for performing CRUD
/// operations with Charybdis, including consistency levels, timeouts, and
/// timestamps. It provides methods to apply these parameters to batch
/// operations and queries, ensuring that the desired settings are used
/// consistently across different operations.
///
/// # Examples
///
/// ```rust,no_run
/// use std::time::Duration;
/// use charybdis::{batch::CharybdisModelBatch, query::CharybdisQuery};
/// use scylla::statement::Consistency;
///
/// // Creating CRUD parameters with specific settings
/// let params = CrudParams {
///     consistency: Consistency::Quorum,
///     timeout: Some(Duration::from_secs(5)),
///     timestamp: Some(1625078400),
/// };
///
/// // Applying parameters to a batch operation
/// let batch = CharybdisModelBatch::new(...); // Assume this initializes a batch
/// let configured_batch = params.apply_batch(batch);
///
/// // Applying parameters to a query
/// let query = CharybdisQuery::new(...); // Assume this initializes a query
/// let configured_query = params.apply_query(query);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CrudParams {
    pub consistency: Consistency,
    pub timeout: Option<Duration>,
    pub timestamp: Option<i64>,
}

impl CrudParams {
    /// Applies the CRUD parameters to a Charybdis model batch.
    ///
    /// This method configures the provided batch with the consistency level
    /// and timestamp specified in the `CrudParams`. It returns the modified
    /// batch with the applied settings.
    ///
    /// # Parameters
    ///
    /// - `batch`: The Charybdis model batch to configure.
    ///
    /// # Returns
    ///
    /// Modified `CharybdisModelBatch` with the applied parameters.
    pub fn apply_batch<'a, Val, E>(
        &self,
        batch: CharybdisModelBatch<'a, Val, E>,
    ) -> CharybdisModelBatch<'a, Val, E>
    where
        Val: SerializeRow + Sync + Send,
        E: ModelBatch<'a>,
    {
        batch
            .consistency(self.consistency)
            .timestamp(self.timestamp)
    }

    /// Applies the CRUD parameters to a Charybdis query.
    ///
    /// This method configures the provided query with the consistency level,
    /// timeout, and timestamp specified in the `CrudParams`. It returns the
    /// modified query with the applied settings.
    ///
    /// # Parameters
    ///
    /// - `query`: The Charybdis query to configure.
    ///
    /// # Returns
    ///
    /// Modified `CharybdisQuery` with the applied parameters.
    pub fn apply_query<
        'a,
        Val: SerializeRow + Send + Sync,
        E: Model + Send + Sync,
        Qe: QueryExecutor<E>,
    >(
        &self,
        query: CharybdisQuery<'a, Val, E, Qe>,
    ) -> CharybdisQuery<'a, Val, E, Qe> {
        query
            .consistency(self.consistency)
            .timeout(self.timeout)
            .timestamp(self.timestamp)
    }
}

/// Converts a reference to `CrudParams` into an owned `CrudParams`.
///
/// This implementation allows for easy conversion from a reference to an
/// owned instance, enabling flexibility in how parameters are passed
/// around in the code.
impl From<&CrudParams> for CrudParams {
    fn from(value: &CrudParams) -> Self {
        value.clone()
    }
}
