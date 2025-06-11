//! ScyllaDB Client Implementation
//!
//! This module provides a high-level client for interacting with ScyllaDB databases
//! using the Charybdis ORM and Scylla driver. It offers connection management,
//! CRUD operations, batch processing, streaming, and keyspace management.

use std::{fmt::Debug, path::Path, sync::Arc};

use super::migrate::MigrationBuilder;
use super::model::Model;
use super::operations::{CharybdisModelBatch, Delete, Insert, ModelBatch, Update};
use super::query::{CharybdisQuery, ModelMutation, ModelRow, ModelStream, QueryExecutor};
use super::stream::CharybdisModelStream;
use super::Result;
use super::{ConnectionParams, CrudParams};

use charybdis::query::OptionalModelRow;
use charybdis::scylla::response::query_result::QueryResult;
use charybdis::scylla::serialize::row::SerializeRow;
use futures::StreamExt;
use tracing::debug;

pub use scylla::client::caching_session::*;
pub use scylla::client::session::*;
pub use scylla::client::session_builder::*;
pub use scylla::client::*;

/// High-level ScyllaDB client that provides an abstraction layer over the Scylla driver
///
/// The `Client` struct encapsulates a cached session to ScyllaDB and optional CRUD parameters
/// for customizing query execution. It provides methods for database operations, keyspace
/// management, and connection handling.
///
/// # Examples
///
/// ```rust,no_run
/// use grapple_db::scylla::{Client, ConnectionParams};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let params = ConnectionParams::default();
///     let client = Client::connect(&params).await?;
///     
///     // Do socmething with client
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    /// Thread-safe reference to the cached ScyllaDB session
    session: Arc<CachingSession>,
    /// Optional CRUD parameters for customizing query execution
    crud_params: Option<CrudParams>,
}

// ================================================================================================
// Constructors
// ================================================================================================
impl Client {
    /// Creates a new client with default connection parameters
    ///
    /// This is a convenience method that uses `ConnectionParams::default()` to establish
    /// a connection to ScyllaDB running on localhost with default settings.
    ///
    /// # Returns
    ///
    /// A `Result` containing the connected `Client` or an error if connection fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     // Do something with client
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn default() -> Result<Self> {
        let con_params = ConnectionParams::default();
        Self::connect(&con_params).await
    }

    /// Creates a new client from an existing cached session
    ///
    /// This method allows you to create a client instance from a pre-configured
    /// `CachingSession`, which is useful when you need to share sessions across
    /// multiple client instances or when you have custom session configuration.
    ///
    /// # Arguments
    ///
    /// * `session` - An `Arc<CachingSession>` representing the ScyllaDB session
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Client` instance.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use grapple_db::scylla::{Client,CachingSession};
    ///
    /// fn create_client_from_session(session: Arc<CachingSession>) -> Result<Client, Box<dyn std::error::Error>> {
    ///     Ok(Client::from_session(&session)?)
    /// }
    /// ```
    pub fn from_session(session: &Arc<CachingSession>) -> Result<Self> {
        Ok(Self {
            session: session.clone(),
            crud_params: None,
        })
    }

    /// Establishes a connection to ScyllaDB using the provided connection parameters
    ///
    /// This is the primary method for creating a new client. It handles the complete
    /// connection setup including keyspace management, file execution, and migrations.
    ///
    /// # Arguments
    ///
    /// * `con_params` - Connection parameters specifying how to connect to ScyllaDB
    ///
    /// # Connection Process
    ///
    /// 1. Creates a cached session using the connection parameters
    /// 2. Optionally creates or recreates the specified keyspace
    /// 3. Sets the keyspace as the default for the session
    /// 4. Executes any initialization files specified in the parameters
    /// 5. Runs database migrations if enabled
    ///
    /// # Returns
    ///
    /// A `Result` containing the connected `Client` or an error if any step fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, ConnectionParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let params = ConnectionParams {
    ///         uri: "127.0.0.1:9042".to_string(),
    ///         use_keyspace: Some("my_keyspace".to_string()),
    ///         recreate_keyspace: true,
    ///         migrate: true,
    ///         ..Default::default()
    ///     };
    ///     
    ///     let client = Client::connect(&params).await?;
    ///     
    ///     // Do socmething with client
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(con_params: &ConnectionParams) -> Result<Self> {
        debug!("Connecting to {}", con_params.uri);

        let session = con_params.caching().await?;
        let client = Self {
            session: Arc::new(session),
            crud_params: None,
        };

        // Handle keyspace setup if specified
        if let Some(keyspace) = &con_params.use_keyspace {
            if con_params.recreate_keyspace {
                client.recreate_keyspace(keyspace).await?;
            } else {
                client.create_keyspace(keyspace).await?;
            }

            client.use_keyspace(keyspace).await?;
        }

        // Execute initialization files
        for filename in &con_params.init_files {
            client.execute_file(filename).await?;
        }

        // Run migrations if enabled
        if con_params.migrate {
            Self::migrate(client.session.get_session(), &con_params.use_keyspace).await?;
        }

        Ok(client)
    }
}

// ================================================================================================
// Setters
// ================================================================================================
impl Client {
    /// Sets CRUD parameters for customizing query execution
    ///
    /// CRUD parameters allow you to specify default consistency levels, timeouts,
    /// and timestamps that will be applied to all database operations performed
    /// by this client instance.
    ///
    /// # Arguments
    ///
    /// * `params` - CRUD parameters that implement `Into<CrudParams>`
    ///
    /// # Returns
    ///
    /// The client instance with updated CRUD parameters (builder pattern).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, ConnectionParams, CrudParams};
    /// use grapple_db::scylla::statement::Consistency;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::connect(&ConnectionParams::default())
    ///         .await?
    ///         .with_params(CrudParams {
    ///             consistency: Consistency::Quorum,
    ///             timeout: Some(Duration::from_secs(30)),
    ///             timestamp: None,
    ///         });
    ///
    ///     // Do something with client
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn with_params(mut self, params: impl Into<CrudParams>) -> Self {
        _ = self.crud_params.insert(params.into());
        self
    }
}

// ================================================================================================
// Getters
// ================================================================================================
impl Client {
    /// Returns a reference to the underlying cached session
    ///
    /// This method provides access to the raw ScyllaDB session for advanced
    /// operations that are not covered by the high-level client methods.
    ///
    /// # Returns
    ///
    /// An `Arc<CachingSession>` that can be shared across threads.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     let session = client.session();
    ///     
    ///     // Use session for advanced operations
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn session(&self) -> Arc<CachingSession> {
        self.session.clone()
    }
}

// ================================================================================================
// CRUD Operations
// ================================================================================================
impl Client {
    /// Executes a query to retrieve a single entity from the database
    ///
    /// This method executes a Charybdis query that returns a single model instance.
    /// The query is automatically enhanced with any CRUD parameters configured
    /// for this client instance.
    ///
    /// # Type Parameters
    ///
    /// * `Val` - The type of values being serialized for the query
    /// * `E` - The entity/model type being retrieved
    ///
    /// # Arguments
    ///
    /// * `query` - A Charybdis query configured to return a single row
    ///
    /// # Returns
    ///
    /// A `Result` containing the retrieved entity or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    /// use grapple_db::scylla::types::Uuid;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: Uuid,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     // Get id somehow
    ///     let user_id = Uuid::from_u128(5);
    ///     let user = client.get(User::find_by_id(user_id)).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get<'a, Val, E>(&self, query: CharybdisQuery<'a, Val, E, ModelRow>) -> Result<E>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send,
    {
        debug!("Get query: {}", query.query_string());

        let res = self
            .query_apply_params(query)
            .execute(&self.session)
            .await?;

        Ok(res)
    }

    /// Executes a query to retrieve an optional entity from the database.
    ///
    /// This method executes a Charybdis query that may return a single model instance or no result at all.
    /// The query is automatically enhanced with any CRUD parameters configured for this client instance.
    ///
    /// # Type Parameters
    ///
    /// * `Val` - The type of values being serialized for the query
    /// * `E` - The entity/model type being retrieved
    ///
    /// # Arguments
    ///
    /// * `query` - A Charybdis query configured to return an optional row
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<E>`, where `Some(E)` is the retrieved entity if found, or `None` if no matching record exists.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    /// use grapple_db::scylla::types::Uuid;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: Uuid,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     // Prepare your query
    ///     let user_id = Uuid::from_u128(5);
    ///     let query = User::maybe_find_first_by_id(user_id);
    ///     
    ///     // Retrieve the user, which may or may not exist
    ///     let user: Option<User> = client.get_optional(query).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_optional<'a, Val, E>(
        &self,
        query: CharybdisQuery<'a, Val, E, OptionalModelRow>,
    ) -> Result<Option<E>>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send,
    {
        debug!("Get query: {}", query.query_string());

        let res = self
            .query_apply_params(query)
            .execute(&self.session)
            .await?;

        Ok(res)
    }

    /// Counts the total number of entities that match the given query
    ///
    /// This method executes a streaming query and counts all the results without loading
    /// them into memory. It's an efficient way to get the count of entities that match
    /// specific criteria without the overhead of retrieving and deserializing all the data.
    ///
    /// The method internally uses the streaming functionality to iterate through all
    /// matching records and returns the total count.
    ///
    /// # Type Parameters
    ///
    /// * `Val` - The type of values being serialized for the query
    /// * `E` - The entity/model type being counted
    ///
    /// # Arguments
    ///
    /// * `query` - A Charybdis query configured to return a stream of results
    ///
    /// # Returns
    ///
    /// A `Result` containing the total count of entities matching the query, or an error
    /// if the query execution fails.
    ///
    /// # Performance Notes
    ///
    /// This method streams through all matching records to count them, which means:
    /// - Memory usage is minimal as records are not stored
    /// - For large result sets, this may take time as it processes all records
    /// - Consider using database-native COUNT queries for better performance on large datasets
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #       global_secondary_indexes = [status],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: String,
    /// #     status: String,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     // Count all active users
    ///     let count = client.count(User::find_by_status("active".to_string())).await?;
    ///     println!("Total active users: {}", count);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn count<'a, Val, E>(
        &self,
        query: CharybdisQuery<'a, Val, E, ModelStream>,
    ) -> Result<usize>
    where
        Val: SerializeRow + Sync + Send + Debug,
        E: Model + Sync + Send + 'static,
    {
        Ok(self.stream(query).await?.count().await)
    }

    /// Updates a single entity in the database
    ///
    /// This method takes an entity that implements the `Update` trait and
    /// generates an update query automatically. The entity's `update()` method
    /// is called to create the appropriate Charybdis query.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The entity/model type being updated
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity instance to update
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the update operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: String,
    /// #     name: String,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     /// Get user somehow
    ///     let mut user = User::default();
    ///     user.name = "New Name".to_string();
    ///     client.update(&user).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn update<E>(&self, entity: &E) -> Result<()>
    where
        E: Model + Update + Sync + Send + 'static,
    {
        self.update_query(entity.update()).await?;

        Ok(())
    }

    /// Internal method for executing update queries
    ///
    /// This method handles the actual execution of update queries with proper
    /// parameter application and logging.
    async fn update_query<'a, Val, E>(
        &self,
        query: CharybdisQuery<'a, Val, E, ModelMutation>,
    ) -> Result<()>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send,
    {
        debug!("Update query: {}", query.query_string());

        _ = self
            .query_apply_params(query)
            .execute(&self.session)
            .await?;

        Ok(())
    }

    /// Updates multiple entities in the database using batch operations
    ///
    /// This method efficiently updates a large number of entities by grouping
    /// them into batches of the specified size. This reduces the number of
    /// round trips to the database and improves performance.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The entity/model type being updated
    ///
    /// # Arguments
    ///
    /// * `iter` - A slice of entities to update
    /// * `chunk_size` - The number of entities to include in each batch
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the batch update operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: String,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     let users: Vec<User> = vec![/* ... users to update ... */];
    ///
    ///     client.update_many(&users, 1000).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn update_many<'a, E>(&self, iter: &[E], chunk_size: usize) -> Result<()>
    where
        E: ModelBatch<'a> + Sync + Send + 'a,
    {
        self.batch_apply_params(E::batch())
            .chunked_update(&self.session, iter, chunk_size)
            .await?;

        Ok(())
    }

    /// Inserts a single entity into the database
    ///
    /// This method takes an entity that implements the `Insert` trait and
    /// generates an insert query automatically. The entity's `insert()` method
    /// is called to create the appropriate Charybdis query.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The entity/model type being inserted
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity instance to insert
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the insert operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    /// use grapple_db::scylla::operations::New;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [name],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     name: String,
    /// # }
    /// #
    /// # impl User{
    /// #    pub fn new(name: &str) -> Self {
    /// #        Self { name: name.to_string() }
    /// #    }
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     let user = User::new("John Doe");
    ///     client.insert(&user).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn insert<E>(&self, entity: &E) -> Result<()>
    where
        E: Model + Insert + Sync + Send + 'static,
    {
        self.insert_query(entity.insert()).await?;

        Ok(())
    }

    /// Internal method for executing insert queries
    ///
    /// This method handles the actual execution of insert queries with proper
    /// parameter application and logging.
    async fn insert_query<'a, Val, E>(
        &self,
        query: CharybdisQuery<'a, Val, E, ModelMutation>,
    ) -> Result<()>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send,
    {
        debug!("Insert query: {}", query.query_string());

        _ = self
            .query_apply_params(query)
            .execute(&self.session)
            .await?;

        Ok(())
    }

    /// Inserts multiple entities into the database using batch operations
    ///
    /// This method efficiently inserts a large number of entities by grouping
    /// them into batches of the specified size. This is much more efficient
    /// than inserting entities one by one.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The entity/model type being inserted
    ///
    /// # Arguments
    ///
    /// * `iter` - A slice of entities to insert
    /// * `chunk_size` - The number of entities to include in each batch
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the batch insert operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: String,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     let users: Vec<User> = vec![/* ... users to insert ... */];
    ///     client.insert_many(&users, 1000).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn insert_many<'a, E>(&self, iter: &[E], chunk_size: usize) -> Result<()>
    where
        E: ModelBatch<'a> + Sync + Send + 'a,
    {
        self.batch_apply_params(E::batch())
            .chunked_insert(&self.session, iter, chunk_size)
            .await?;

        Ok(())
    }

    /// Deletes a single entity from the database
    ///
    /// This method takes an entity that implements the `Delete` trait and
    /// generates a delete query automatically. The entity's `delete()` method
    /// is called to create the appropriate Charybdis query.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The entity/model type being deleted
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity instance to delete
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the delete operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// // Assuming you have a User model defined
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: String,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     /// Get user somehow
    ///     let user = User::default();
    ///     client.delete(&user).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete<E>(&self, entity: &E) -> Result<()>
    where
        E: Model + Delete + Sync + Send + 'static,
    {
        self.delete_query(entity.delete()).await?;

        Ok(())
    }

    /// Internal method for executing delete queries
    ///
    /// This method handles the actual execution of delete queries with proper
    /// parameter application and logging.
    async fn delete_query<'a, Val, E>(
        &self,
        query: CharybdisQuery<'a, Val, E, ModelMutation>,
    ) -> Result<()>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send,
    {
        debug!("Delete query: {}", query.query_string());

        _ = self
            .query_apply_params(query)
            .execute(&self.session)
            .await?;

        Ok(())
    }

    /// Deletes multiple entities from the database using batch operations
    ///
    /// This method efficiently deletes a large number of entities by grouping
    /// them into batches of the specified size. This reduces the number of
    /// round trips to the database and improves performance.
    ///
    /// # Type Parameters
    ///
    /// * `E` - The entity/model type being deleted
    ///
    /// # Arguments
    ///
    /// * `iter` - A slice of entities to delete
    /// * `chunk_size` - The number of entities to include in each batch
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the batch delete operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: String,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     let users: Vec<User> = vec![/* ... users to delete ... */];
    ///     client.delete_many(&users, 1000).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete_many<'a, E>(&self, iter: &[E], chunk_size: usize) -> Result<()>
    where
        E: ModelBatch<'a> + Sync + Send + 'a,
    {
        self.batch_apply_params(E::batch())
            .chunked_delete(&self.session, iter, chunk_size)
            .await?;

        Ok(())
    }

    /// Creates a stream for efficiently processing large result sets
    ///
    /// This method executes a query that returns a stream of results, which is
    /// useful for processing large datasets without loading everything into memory
    /// at once. The stream can be used with pagination or consumed incrementally.
    ///
    /// # Type Parameters
    ///
    /// * `Val` - The type of values being serialized for the query
    /// * `E` - The entity/model type being streamed
    ///
    /// # Arguments
    ///
    /// * `query` - A Charybdis query configured to return a stream of results
    ///
    /// # Returns
    ///
    /// A `Result` containing a `CharybdisModelStream` for processing results.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    /// use futures::StreamExt;
    /// use grapple_db::scylla::operations::Find;
    ///
    /// // Assuming you have a `User` model defined with `Charybdis`
    /// # #[grapple_db::scylla::macros::charybdis_model(
    /// #       table_name = users,
    /// #       partition_keys = [id],
    /// #       clustering_keys = [],
    /// #   )]
    /// # #[derive(Debug, Default)]
    /// # struct User {
    /// #     id: String,
    /// # }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     let mut stream = client.stream(User::find_all()).await?;
    ///     while let Some(user) = stream.next().await {
    ///          match user {
    ///              Ok(user) => println!("User: {:?}", user),
    ///              Err(e) => eprintln!("Error: {:?}", e),
    ///          }
    ///      }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn stream<'a, Val, E>(
        &self,
        query: CharybdisQuery<'a, Val, E, ModelStream>,
    ) -> Result<CharybdisModelStream<E>>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send + 'static,
    {
        debug!("Stream query: {}", query.query_string());

        let res = self
            .query_apply_params(query)
            .execute(&self.session)
            .await?;

        Ok(res)
    }
}

// ================================================================================================
// Table Management
// ================================================================================================
impl Client {
    /// Drops a table from the database if it exists
    ///
    /// This method executes a `DROP TABLE IF EXISTS` statement for the specified
    /// table name. It's safe to call even if the table doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the table to drop
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the drop operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     client.drop_table("old_users_table").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn drop_table(&self, name: &str) -> Result<()> {
        let query = format!("DROP TABLE IF EXISTS {name};");

        self.execute(&query, &[]).await?;

        Ok(())
    }
}

// ================================================================================================
// Keyspace Management
// ================================================================================================
impl Client {
    /// Retrieves a list of all keyspaces in the ScyllaDB cluster
    ///
    /// This method queries the system schema to get a list of all available
    /// keyspaces in the connected ScyllaDB cluster.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of keyspace names or an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     let keyspaces = client.keyspaces().await?;
    ///     for keyspace in keyspaces {
    ///         println!("Keyspace: {}", keyspace);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn keyspaces(&self) -> Result<Vec<String>> {
        let query = "SELECT keyspace_name FROM system_schema.keyspaces;";

        let res = self.session.execute_unpaged(query, &[]).await?;

        let keyspaces: Vec<String> = res
            .into_rows_result()?
            .rows::<(String,)>()?
            .filter_map(|s| s.ok()) // Используем filter_map для извлечения значений
            .map(|(keyspace_name,)| keyspace_name) // Извлекаем имя keyspace
            .collect();

        Ok(keyspaces)
    }

    /// Gets the currently active keyspace for this session
    ///
    /// Returns the name of the keyspace that is currently being used by
    /// the session, if any.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the keyspace name, or `None` if no keyspace is set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     if let Some(keyspace) = client.get_keyspace() {
    ///         println!("Current keyspace: {}", keyspace);
    ///     } else {
    ///         println!("No keyspace is currently set");
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn get_keyspace(&self) -> Option<String> {
        let keyspace = self.session.get_session().get_keyspace();

        keyspace.map(|k| k.to_string())
    }

    /// Sets the active keyspace for this session
    ///
    /// Changes the current keyspace context for the session. All subsequent
    /// queries will be executed in the context of this keyspace unless
    /// explicitly qualified with a different keyspace name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the keyspace to use
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     client.use_keyspace("my_application").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn use_keyspace(&self, name: &str) -> Result<()> {
        self.session.get_session().use_keyspace(name, true).await?;

        Ok(())
    }

    // Drops and recreates a keyspace
    ///
    /// This method first drops the specified keyspace (if it exists) and then
    /// creates it again with default replication settings. This is useful for
    /// resetting a keyspace to a clean state.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the keyspace to recreate
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    /// use grapple_db::scylla::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let client = Client::default().await?;
    ///
    ///     client.recreate_keyspace("test_keyspace").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn recreate_keyspace(&self, name: &str) -> Result<()> {
        self.drop_keyspace(name).await?;
        self.create_keyspace(name).await?;

        Ok(())
    }

    /// Recreates a keyspace and returns the client instance (builder pattern)
    ///
    /// This is a convenience method that combines `recreate_keyspace` with the
    /// builder pattern, allowing you to chain method calls during client setup.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the keyspace to recreate
    ///
    /// # Returns
    ///
    /// A `Result` containing the client instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, ConnectionParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::connect(&ConnectionParams::default())
    ///         .await?
    ///         .with_recreate_keyspace("test_keyspace")
    ///         .await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_recreate_keyspace(self, name: &str) -> Result<Self> {
        self.recreate_keyspace(name).await?;

        Ok(self)
    }

    /// Creates a new keyspace if it doesn't already exist
    ///
    /// This method executes a `CREATE KEYSPACE IF NOT EXISTS` statement with
    /// SimpleStrategy replication and a replication factor of 1. This is suitable
    /// for development and testing environments.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the keyspace to create
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     client.create_keyspace("my_application").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_keyspace(&self, name: &str) -> Result<()> {
        let query = format!("CREATE KEYSPACE IF NOT EXISTS {name} WITH REPLICATION = {{ 'class' : 'SimpleStrategy', 'replication_factor' : 1 }};");

        self.execute(&query, &[]).await?;

        Ok(())
    }

    /// Drops a keyspace if it exists
    ///
    /// This method executes a `DROP KEYSPACE IF EXISTS` statement for the
    /// specified keyspace. It's safe to call even if the keyspace doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the keyspace to drop
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     client.drop_keyspace("old_keyspace").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn drop_keyspace(&self, name: &str) -> Result<()> {
        let query = format!("DROP KEYSPACE IF EXISTS {name};");

        self.execute(&query, &[]).await?;

        Ok(())
    }

    /// Creates a keyspace and returns the client instance (builder pattern)
    ///
    /// This is a convenience method that combines `create_keyspace` with the
    /// builder pattern, allowing you to chain method calls during client setup.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the keyspace to create
    ///
    /// # Returns
    ///
    /// A `Result` containing the client instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, ConnectionParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::connect(&ConnectionParams::default())
    ///         .await?
    ///         .with_keyspace("my_application")
    ///         .await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_keyspace(self, name: &str) -> Result<Self> {
        self.create_keyspace(name).await?;

        Ok(self)
    }

    /// Creates multiple keyspaces and returns the client instance (builder pattern)
    ///
    /// This method creates multiple keyspaces in sequence and returns the client
    /// instance for method chaining. Useful when setting up multiple keyspaces
    /// during application initialization.
    ///
    /// # Arguments
    ///
    /// * `names` - A slice of keyspace names to create
    ///
    /// # Returns
    ///
    /// A `Result` containing the client instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, ConnectionParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::connect(&ConnectionParams::default())
    ///         .await?
    ///         .with_keyspaces(&["users", "products", "orders"])
    ///         .await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_keyspaces(self, names: &[&str]) -> Result<Self> {
        for name in names {
            self.create_keyspace(name).await?;
        }

        Ok(self)
    }

    /// Drops a keyspace and returns the client instance (builder pattern)
    ///
    /// This is a convenience method that combines `drop_keyspace` with the
    /// builder pattern, allowing you to chain method calls during client setup.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the keyspace to drop
    ///
    /// # Returns
    ///
    /// A `Result` containing the client instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, ConnectionParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::connect(&ConnectionParams::default())
    ///         .await?
    ///         .without_keyspace("old_keyspace")
    ///         .await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn without_keyspace(self, name: &str) -> Result<Self> {
        self.drop_keyspace(name).await?;

        Ok(self)
    }

    /// Drops multiple keyspaces and returns the client instance (builder pattern)
    ///
    /// This method drops multiple keyspaces in sequence and returns the client
    /// instance for method chaining. Useful when cleaning up multiple keyspaces
    /// during application shutdown or testing.
    ///
    /// # Arguments
    ///
    /// * `names` - A slice of keyspace names to drop
    ///
    /// # Returns
    ///
    /// A `Result` containing the client instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, ConnectionParams};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::connect(&ConnectionParams::default())
    ///         .await?
    ///         .without_keyspaces(&["test_users", "test_products"])
    ///         .await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn without_keyspaces(self, names: &[&str]) -> Result<Self> {
        for name in names {
            self.drop_keyspace(name).await?;
        }

        Ok(self)
    }
}

// ================================================================================================
// Utility methods
// ================================================================================================
impl Client {
    /// Executes a raw CQL query with the provided values
    ///
    /// This method provides direct access to the underlying ScyllaDB session
    /// for executing custom CQL queries that are not covered by the high-level
    /// CRUD operations. Use this for complex queries, DDL statements, or
    /// database administration tasks.
    ///
    /// # Arguments
    ///
    /// * query - The CQL query string to execute
    /// * values - Values to bind to the query parameters
    ///
    /// # Returns
    ///
    /// A Result containing the QueryResult or an error.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///
    ///     let result = client.execute("SELECT COUNT(*) FROM users WHERE active = ?",
    ///         (true,)).await?;  
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute(&self, query: &str, values: impl SerializeRow) -> Result<QueryResult> {
        debug!("Executing query: {}", query);

        let res = self.session.execute_unpaged(query, values).await?;

        Ok(res)
    }

    /// Executes CQL queries from a file
    ///
    /// This method reads a file containing CQL statements separated by semicolons
    /// and executes them sequentially. This is useful for running initialization
    /// scripts, schema migrations, or bulk data operations.
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to the file containing CQL statements
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the file execution.
    ///
    /// # File Format
    ///
    /// The file should contain CQL statements separated by semicolons:
    /// ```sql
    /// CREATE TABLE users (id UUID PRIMARY KEY, name TEXT);
    /// INSERT INTO users (id, name) VALUES (uuid(), 'John Doe');
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = Client::default().await?;
    ///     
    ///     client.execute_file("database/schema.cql").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn execute_file(&self, filename: &str) -> Result<()> {
        debug!("Init file '{}'", filename);

        let current_path = std::env::current_dir().unwrap();
        let file_path = Path::new(filename);
        let full_path = current_path.join(file_path);

        let raw_queries = tokio::fs::read_to_string(full_path)
            .await
            .unwrap_or_else(|_| panic!("Could not read file"));

        let queries = raw_queries
            .split(";")
            .map(|query| query.trim())
            .collect::<Vec<&str>>();

        for query in queries {
            if query.is_empty() {
                continue;
            }

            self.execute(query, &[]).await?;
        }

        Ok(())
    }

    /// Runs database migrations using Charybdis migration builder
    ///
    /// This method executes database schema migrations using the Charybdis
    /// migration framework. It can optionally drop and recreate the keyspace
    /// before running migrations, which is useful for development environments.
    ///
    /// # Arguments
    ///
    /// * `session` - The ScyllaDB session to use for migrations
    /// * `use_keyspace` - Optional keyspace name to target for migrations
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the migration process.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::{Client, client::Session};
    ///
    /// async fn run_migrations(session: &Session) -> Result<(), Box<dyn std::error::Error>> {
    ///     Client::migrate(session, &Some("my_keyspace".to_string())).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn migrate(session: &Session, use_keyspace: &Option<String>) -> Result<()> {
        debug!("Migration started");

        let mut builder = MigrationBuilder::new();

        if let Some(keyspace) = use_keyspace {
            builder = builder.keyspace(keyspace.to_owned());
        }

        let migration = builder.build(session).await;

        migration.run().await;

        Ok(())
    }

    /// Internal method for applying CRUD parameters to batch operations
    ///
    /// This method applies the client's CRUD parameters (consistency, timeout,
    /// timestamp) to a Charybdis model batch if parameters are configured.
    ///
    /// # Type Parameters
    ///
    /// * `Val` - The type of values being serialized for the batch
    /// * `E` - The entity/model type being batched
    ///
    /// # Arguments
    ///
    /// * `batch` - The Charybdis model batch to enhance with parameters
    ///
    /// # Returns
    ///
    /// The batch with applied CRUD parameters, or the original batch if no parameters are set.
    fn batch_apply_params<'a, Val, E>(
        &self,
        batch: CharybdisModelBatch<'a, Val, E>,
    ) -> CharybdisModelBatch<'a, Val, E>
    where
        Val: SerializeRow + Sync + Send,
        E: ModelBatch<'a>,
    {
        if let Some(params) = &self.crud_params {
            params.apply_batch(batch)
        } else {
            batch
        }
    }

    /// Internal method for applying CRUD parameters to queries
    ///
    /// This method applies the client's CRUD parameters (consistency, timeout,
    /// timestamp) to a Charybdis query if parameters are configured.
    ///
    /// # Type Parameters
    ///
    /// * `Val` - The type of values being serialized for the query
    /// * `E` - The entity/model type being queried
    /// * `Qe` - The query executor type
    ///
    /// # Arguments
    ///
    /// * `query` - The Charybdis query to enhance with parameters
    ///
    /// # Returns
    ///
    /// The query with applied CRUD parameters, or the original query if no parameters are set.
    fn query_apply_params<'a, Val, E, Qe>(
        &self,
        query: CharybdisQuery<'a, Val, E, Qe>,
    ) -> CharybdisQuery<'a, Val, E, Qe>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send,
        Qe: QueryExecutor<E>,
    {
        if let Some(params) = &self.crud_params {
            params.apply_query(query)
        } else {
            query
        }
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

    use super::*;

    use crate::scylla::{
        charybdis::{self, macros::charybdis_model, types::Text},
        Client, ConnectionParams,
    };

    #[charybdis_model(
        table_name = users,
        partition_keys = [id],
        clustering_keys = [],

        global_secondary_indexes = [name],
        local_secondary_indexes = [],
    )]
    #[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Tst {
        id: Text,
        name: Option<Text>,
    }

    impl Tst {
        fn with_id(id: &str) -> Self {
            Self {
                id: id.to_string(),
                name: None,
            }
        }

        fn with_name(mut self, name: impl Into<String>) -> Self {
            _ = self.name.insert(name.into());
            self
        }
    }

    async fn get_client() -> Client {
        let params = ConnectionParams {
            migrate: false,
            use_keyspace: Some("test".into()),

            ..Default::default()
        };

        let client = Client::connect(&params).await.unwrap();

        client
            .execute(
                "
        CREATE TABLE IF NOT EXISTS users (
            id text PRIMARY KEY,
            name text
        );
        ",
                &[],
            )
            .await
            .unwrap();

        client
            .execute("CREATE INDEX IF NOT EXISTS ON users (name);", &[])
            .await
            .unwrap();

        client
    }

    #[tokio::test]
    async fn test_scylla_get() -> Result<()> {
        let client = get_client().await;

        let id = "test_scylla_get";

        // Create model
        let model = Tst::with_id(id);
        client.insert(&model).await?;

        // Test
        assert_eq!(model, client.get(Tst::find_by_id(id.into())).await?);

        // Clear

        client.delete(&model).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_get_optional() -> Result<()> {
        let client = get_client().await;

        let id = "test_scylla_maybe_get";

        assert!(client
            .get_optional(Tst::maybe_find_first_by_id(id.into()))
            .await?
            .is_none());

        // Create model
        let model = Tst::with_id(id);
        client.insert(&model).await?;

        assert!(client
            .get_optional(Tst::maybe_find_first_by_id(id.into()))
            .await?
            .is_some());

        // Clear
        client.delete(&model).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_stream() -> Result<()> {
        let client = get_client().await;
        let fx_name = "test_scylla_stream";

        let models = [
            Tst::with_id("test_scylla_stream1").with_name(fx_name),
            Tst::with_id("test_scylla_stream2").with_name(fx_name),
            Tst::with_id("test_scylla_stream3").with_name(fx_name),
            Tst::with_id("test_scylla_stream4").with_name(fx_name),
            Tst::with_id("test_scylla_stream5").with_name(fx_name),
            Tst::with_id("test_scylla_stream6").with_name(fx_name),
        ];

        // Create models
        client.insert_many(&models, 6).await?;

        // Test
        let mut stream = client
            .stream(Tst::find_by_name(fx_name.to_string()))
            .await?;

        let mut got = vec![];

        while let Some(Ok(model)) = stream.next().await {
            got.push(model);
        }

        assert_eq!(6, got.len());

        got.sort();

        assert_eq!(models[0], got[0]);
        assert_eq!(models[1], got[1]);
        assert_eq!(models[2], got[2]);
        assert_eq!(models[3], got[3]);
        assert_eq!(models[4], got[4]);
        assert_eq!(models[5], got[5]);

        // Clear
        client.delete_many(&got, 6).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_insert() -> Result<()> {
        let client = get_client().await;

        let id = "test_scylla_insert";

        // Create model
        let model = Tst::with_id(id);
        client.insert(&model).await?;

        // Test
        assert_eq!(model, client.get(Tst::find_by_id(id.into())).await?);

        // Clear
        client.delete(&model).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_insert_many() -> Result<()> {
        let client = get_client().await;
        let fx_name = "test_scylla_insert_many";

        let models = [
            Tst::with_id("test_scylla_insert_many1").with_name(fx_name),
            Tst::with_id("test_scylla_insert_many2").with_name(fx_name),
            Tst::with_id("test_scylla_insert_many3").with_name(fx_name),
        ];

        // Create models
        client.insert_many(&models, 3).await?;

        // Test
        let mut find = client
            .stream(Tst::find_by_name(fx_name.into()))
            .await?
            .try_collect()
            .await?;
        assert_eq!(3, find.len());

        find.sort();

        assert_eq!(models[0], find[0]);
        assert_eq!(models[1], find[1]);
        assert_eq!(models[2], find[2]);

        // Clear
        client.delete_many(&find, 3).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_update() -> Result<()> {
        let client = get_client().await;

        let id = "test_scylla_update";

        // Create model
        let model = Tst::with_id(id);
        client.insert(&model).await?;

        // Test
        let updated = model.with_name("new name");
        client.update(&updated).await?;

        assert_eq!(updated, client.get(Tst::find_by_id(id.into())).await?);

        // Clear
        client.delete(&updated).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_update_many() -> Result<()> {
        let client = get_client().await;
        let fx_name = "name";
        let fx_new_name = "new_name";

        let models = [
            Tst::with_id("test_scylla_update_many1").with_name(fx_name),
            Tst::with_id("test_scylla_update_many2").with_name(fx_name),
            Tst::with_id("test_scylla_update_many3").with_name(fx_name),
        ];

        // Create models
        client.insert_many(&models, 3).await?;

        let mut find = client
            .stream(Tst::find_by_name(fx_name.into()))
            .await?
            .try_collect()
            .await?;
        assert_eq!(3, find.len());
        find.sort();

        assert_eq!(models[0], find[0]);
        assert_eq!(models[1], find[1]);
        assert_eq!(models[2], find[2]);

        // Test
        let mut updated = vec![];
        for model in models {
            let new_model = model.with_name(fx_new_name);
            updated.push(new_model);
        }

        client.update_many(&updated, 3).await?;

        let mut find = client
            .stream(Tst::find_by_name(fx_new_name.into()))
            .await?
            .try_collect()
            .await?;
        assert_eq!(3, find.len());
        find.sort();

        assert_eq!(updated[0], find[0]);
        assert_eq!(updated[1], find[1]);
        assert_eq!(updated[2], find[2]);

        // Clear
        client.delete_many(&find, 3).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_delete() -> Result<()> {
        let client = get_client().await;

        let id = "test_scylla_delete";

        // Create model
        let model = Tst::with_id(id);
        client.insert(&model).await?;

        // Test
        client.delete(&model).await?;

        assert!(client.get(Tst::find_by_id(id.into())).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_delete_many() -> Result<()> {
        let client = get_client().await;
        let fx_name = "test_scylla_delete_many";

        let models = [
            Tst::with_id("test_scylla_delete_many1").with_name(fx_name),
            Tst::with_id("test_scylla_delete_many2").with_name(fx_name),
            Tst::with_id("test_scylla_delete_many3").with_name(fx_name),
        ];

        // Create models
        client.insert_many(&models, 3).await?;

        // Test
        let count = client.count(Tst::find_by_name(fx_name.into())).await?;
        assert_eq!(3, count);

        client.delete_many(&models, 3).await?;

        let count = client.count(Tst::find_by_name(fx_name.into())).await?;
        assert_eq!(0, count);

        Ok(())
    }

    #[tokio::test]
    async fn test_scylla_count() -> Result<()> {
        let client = get_client().await;
        let fx_name = "test_scylla_count";

        let models = [
            Tst::with_id("test_scylla_count1").with_name(fx_name),
            Tst::with_id("test_scylla_count2").with_name(fx_name),
            Tst::with_id("test_scylla_count3").with_name(fx_name),
        ];

        // Create models
        client.insert_many(&models, 3).await?;

        // Test
        let count = client.count(Tst::find_by_name(fx_name.into())).await?;
        assert_eq!(3, count);

        // Clear
        client.delete_many(&models, 3).await?;

        let count = client.count(Tst::find_by_name(fx_name.into())).await?;
        assert_eq!(0, count);

        Ok(())
    }
}

// endregion: --- Tests
