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

impl Default for ConnectionParams {
    fn default() -> Self {
        Self {
            uri: "127.0.0.1:9042".to_string(),
            caching_capacity: 1000,
            connection_timeout: Duration::from_secs(3),
            compression: None,
            fetch_keyspaces: vec![],
            keyspace_case_sensitive: false,
            use_keyspace: None,
            migrate: false,
            recreate_keyspace: false,
            init_file: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub uri: String,
    pub connection_timeout: Duration,
    pub caching_capacity: usize,
    pub compression: Option<Compression>,
    pub fetch_keyspaces: Vec<String>,
    pub use_keyspace: Option<String>,
    pub keyspace_case_sensitive: bool,
    pub migrate: bool,
    pub recreate_keyspace: bool,
    pub init_file: Option<String>,
}

impl ConnectionParams {
    pub async fn build(&self) -> Result<Session> {
        let builder = SessionBuilder::new()
            .known_node(&self.uri)
            .connection_timeout(self.connection_timeout)
            .keyspaces_to_fetch(&self.fetch_keyspaces)
            .compression(self.compression);

        Ok(builder.build().await?)
    }

    pub async fn caching(&self) -> Result<CachingSession> {
        let session = self.build().await?;

        let caching = CachingSessionBuilder::new(session)
            .max_capacity(self.caching_capacity)
            .build();

        Ok(caching)
    }
}
