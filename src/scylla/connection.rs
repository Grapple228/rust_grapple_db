//! ScyllaDB Connection Parameters
//!
//! This module provides configuration structures and utilities for establishing
//! connections to ScyllaDB clusters. It supports both regular sessions and
//! cached sessions with customizable connection parameters.

use super::Result;
use scylla::{
    client::{
        caching_session::{CachingSession, CachingSessionBuilder},
        session::Session,
        session_builder::SessionBuilder,
    },
    frame::Compression,
};
use std::time::Duration;

/// Default implementation for ConnectionParams
///
/// Provides sensible defaults for connecting to a local ScyllaDB instance
/// running on the standard port with common configuration settings.
impl Default for ConnectionParams {
    /// Creates a new ConnectionParams instance with default values
    ///
    /// # Default Values
    ///
    /// - `uri`: "127.0.0.1:9042" (local ScyllaDB instance)
    /// - `caching_capacity`: 1000 prepared statements
    /// - `connection_timeout`: 3 seconds
    /// - `compression`: None (no compression)
    /// - `fetch_keyspaces`: Empty vector (no keyspaces pre-fetched)
    /// - `keyspace_case_sensitive`: true
    /// - `use_keyspace`: None (no default keyspace)
    /// - `migrate`: true (run migrations by default)
    /// - `recreate_keyspace`: false (don't recreate keyspace by default)
    /// - `init_files`: Empty vector (no initialization files)
    ///
    /// # Returns
    ///
    /// A `ConnectionParams` instance with default configuration suitable for
    /// local development and testing.
    fn default() -> Self {
        Self {
            uri: "127.0.0.1:9042".to_string(),
            caching_capacity: 1000,
            connection_timeout: Duration::from_secs(3),
            compression: None,
            fetch_keyspaces: vec![],
            keyspace_case_sensitive: true,
            use_keyspace: None,
            migrate: true,
            recreate_keyspace: false,
            init_files: vec![],
        }
    }
}

/// Configuration parameters for establishing connections to ScyllaDB
///
/// This struct encapsulates all the necessary configuration options for connecting
/// to a ScyllaDB cluster, including connection settings, caching options, keyspace
/// management, and initialization parameters.
///
/// # Examples
///
/// ```rust,no_run
/// use grapple_db::scylla::ConnectionParams;
/// use std::time::Duration;
///
/// // Using default configuration
/// let params = ConnectionParams::default();
///
/// // Custom configuration
/// let params = ConnectionParams {
///     uri: "192.168.1.100:9042".to_string(),
///     connection_timeout: Duration::from_secs(10),
///     caching_capacity: 2000,
///     use_keyspace: Some("my_app".to_string()),
///     migrate: true,
///     recreate_keyspace: false,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    /// The URI of the ScyllaDB node to connect to
    ///
    /// Format: "host:port" (e.g., "127.0.0.1:9042" or "scylla.example.com:9042")
    /// This is the initial contact point for the cluster.
    pub uri: String,

    /// Maximum time to wait for a connection to be established
    ///
    /// If the connection cannot be established within this timeout,
    /// the connection attempt will fail with a timeout error.
    pub connection_timeout: Duration,

    /// Maximum number of prepared statements to cache
    ///
    /// The caching session will store up to this many prepared statements
    /// in memory to avoid re-preparing frequently used queries. Higher values
    /// improve performance but use more memory.
    pub caching_capacity: usize,

    /// Optional compression algorithm to use for network communication
    ///
    /// Compression can reduce network bandwidth usage at the cost of CPU overhead.
    /// Common options include LZ4 and Snappy. None means no compression.
    pub compression: Option<Compression>,

    /// List of keyspaces to fetch metadata for during connection
    ///
    /// Pre-fetching keyspace metadata can improve query performance by avoiding
    /// metadata lookups during query execution. Leave empty to fetch all keyspaces
    /// or specify only the keyspaces your application uses.
    pub fetch_keyspaces: Vec<String>,

    /// The keyspace to use as the default for this session
    ///
    /// If specified, all queries will be executed in this keyspace context
    /// unless explicitly qualified with a different keyspace name.
    pub use_keyspace: Option<String>,

    /// Whether keyspace names should be treated as case-sensitive
    ///
    /// When true, keyspace names must match exactly including case.
    /// When false, keyspace names are case-insensitive.
    pub keyspace_case_sensitive: bool,

    /// Whether to run database migrations after connecting
    ///
    /// When true, the Charybdis migration system will be executed to ensure
    /// the database schema is up to date with the application models.
    pub migrate: bool,

    /// Whether to drop and recreate the keyspace before connecting
    ///
    /// When true, the specified keyspace will be dropped (if it exists) and
    /// recreated. This is useful for testing and development environments
    /// but should never be used in production.
    pub recreate_keyspace: bool,

    /// List of CQL files to execute during initialization
    ///
    /// These files will be executed in order after the connection is established
    /// and before migrations (if enabled). Useful for setting up initial data,
    /// creating custom types, or running setup scripts.
    pub init_files: Vec<String>,
}

impl ConnectionParams {
    /// Creates a regular ScyllaDB session using these connection parameters
    ///
    /// This method establishes a connection to the ScyllaDB cluster and returns
    /// a regular session that can be used for executing queries. The session
    /// will be configured with the specified connection timeout, compression,
    /// and keyspace settings.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Session` instance or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The ScyllaDB node is unreachable
    /// - Authentication fails
    /// - The connection timeout is exceeded
    /// - Network issues prevent connection establishment
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::ConnectionParams;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let params = ConnectionParams::default();
    ///     let session = params.build().await?;
    ///     
    ///     // Use the session for database operations
    ///     Ok(())
    /// }
    /// ```
    pub async fn build(&self) -> Result<Session> {
        let builder = SessionBuilder::new()
            .known_node(&self.uri)
            .connection_timeout(self.connection_timeout)
            .keyspaces_to_fetch(&self.fetch_keyspaces)
            .compression(self.compression);

        Ok(builder.build().await?)
    }

    /// Creates a caching ScyllaDB session using these connection parameters
    ///
    /// This method first creates a regular session using `build()` and then wraps
    /// it in a caching layer. The caching session automatically caches prepared
    /// statements to improve performance for frequently executed queries.
    ///
    /// The caching session is particularly beneficial for applications that:
    /// - Execute the same queries repeatedly with different parameters
    /// - Have a limited set of query patterns
    /// - Want to minimize query preparation overhead
    ///
    /// # Returns
    ///
    /// A `Result` containing a `CachingSession` instance or an error if the
    /// connection fails or the caching session cannot be created.
    ///
    /// # Performance Benefits
    ///
    /// - Prepared statements are cached and reused automatically
    /// - Reduces query preparation overhead for repeated queries
    /// - Improves overall application performance for query-heavy workloads
    ///
    /// # Memory Usage
    ///
    /// The caching session will use memory proportional to the `caching_capacity`
    /// setting. Each cached prepared statement consumes a small amount of memory.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use grapple_db::scylla::ConnectionParams;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let params = ConnectionParams {
    ///         caching_capacity: 2000,
    ///         ..Default::default()
    ///     };
    ///     
    ///     let caching_session = params.caching().await?;
    ///     
    ///     // Use the caching session for improved performance
    ///     Ok(())
    /// }
    /// ```
    pub async fn caching(&self) -> Result<CachingSession> {
        let session = self.build().await?;

        let caching = CachingSessionBuilder::new(session)
            .max_capacity(self.caching_capacity)
            .build();

        Ok(caching)
    }
}
