use std::{fmt::Debug, path::Path, sync::Arc};

use charybdis::{
    batch::{CharybdisModelBatch, ModelBatch},
    migrate::MigrationBuilder,
    model::Model,
    query::{CharybdisQuery, ModelMutation, ModelRow, ModelStream, QueryExecutor},
    stream::CharybdisModelStream,
};
use scylla::{
    client::{caching_session::CachingSession, session::Session},
    response::query_result::QueryResult,
    serialize::row::SerializeRow,
};
use tracing::debug;

use crate::scylla::{ConnectionParams, CrudParams};

use super::Result;

#[derive(Debug, Clone)]
pub struct Client {
    session: Arc<CachingSession>,
    crud_params: Option<CrudParams>,
}

impl Client {
    pub fn from_session(session: Arc<CachingSession>) -> Result<Self> {
        Ok(Self {
            session: session.clone(),
            crud_params: None,
        })
    }

    pub async fn connect(con_params: &ConnectionParams) -> Result<Self> {
        debug!("Connecting to {}", con_params.uri);

        let session = con_params.caching().await?;
        let client = Self {
            session: Arc::new(session),
            crud_params: None,
        };

        if let Some(keyspace) = &con_params.use_keyspace {
            if con_params.recreate_keyspace {
                client.recreate_keyspace(&keyspace).await?;
            } else {
                client.create_keyspace(&keyspace).await?;
            }

            client.use_keyspace(&keyspace).await?;
        }

        if let Some(filename) = &con_params.init_file {
            client.init_from_file(filename).await?;
        }

        if con_params.migrate {
            Self::migrate(&client.session.get_session(), &con_params.use_keyspace).await?;
        }

        Ok(client)
    }

    pub async fn execute(&self, query: &str, values: impl SerializeRow) -> Result<QueryResult> {
        debug!("Executing query: {}", query);

        let res = self.session.execute_unpaged(query, values).await?;

        Ok(res)
    }

    pub async fn migrate(session: &Session, use_keyspace: &Option<String>) -> Result<()> {
        debug!("Migration started");

        let mut builder = MigrationBuilder::new().drop_and_replace(true);

        if let Some(keyspace) = use_keyspace {
            builder = builder.keyspace(keyspace.to_owned());
        }

        let migration = builder.build(&session).await;

        migration.run().await;

        Ok(())
    }

    pub fn with_params(mut self, params: impl Into<CrudParams>) -> Self {
        _ = self.crud_params.insert(params.into());
        self
    }

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

    pub async fn update<'a, Val, E>(
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

    pub async fn update_many<'a, E>(&self, iter: &[E], chunk_size: usize) -> Result<()>
    where
        E: ModelBatch<'a> + Sync + Send + 'a,
    {
        self.batch_apply_params(E::batch())
            .chunked_update(&self.session, iter, chunk_size)
            .await?;

        Ok(())
    }

    pub async fn create<'a, Val, E>(
        &self,
        query: CharybdisQuery<'a, Val, E, ModelMutation>,
    ) -> Result<()>
    where
        Val: SerializeRow + Sync + Send,
        E: Model + Sync + Send,
    {
        debug!("Create query: {}", query.query_string());

        _ = self
            .query_apply_params(query)
            .execute(&self.session)
            .await?;

        Ok(())
    }

    pub async fn create_many<'a, E>(&self, iter: &[E], chunk_size: usize) -> Result<()>
    where
        E: ModelBatch<'a> + Sync + Send + 'a,
    {
        self.batch_apply_params(E::batch())
            .chunked_insert(&self.session, iter, chunk_size)
            .await?;

        Ok(())
    }

    pub async fn delete<'a, Val, E>(
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

    pub async fn delete_many<'a, E>(&self, iter: &[E], chunk_size: usize) -> Result<()>
    where
        E: ModelBatch<'a> + Sync + Send + 'a,
    {
        self.batch_apply_params(E::batch())
            .chunked_delete(&self.session, iter, chunk_size)
            .await?;

        Ok(())
    }

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

    pub async fn init_from_file(&self, filename: &str) -> Result<()> {
        debug!("Init from file '{}'", filename);

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

    pub fn session(&self) -> Arc<CachingSession> {
        self.session.clone()
    }
}

// Tables
impl Client {
    pub async fn drop_table(&self, name: &str) -> Result<()> {
        let query = format!("DROP TABLE IF EXISTS {name};");

        self.execute(&query, &[]).await?;

        Ok(())
    }
}

// Keyspaces
impl Client {
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

    pub fn get_keyspace(&self) -> Option<String> {
        let keyspace = self.session.get_session().get_keyspace();

        keyspace.map(|k| k.to_string())
    }

    pub async fn use_keyspace(&self, name: &str) -> Result<()> {
        self.session.get_session().use_keyspace(name, true).await?;

        Ok(())
    }

    pub async fn recreate_keyspace(&self, name: &str) -> Result<()> {
        self.drop_keyspace(name).await?;
        self.create_keyspace(name).await?;

        Ok(())
    }

    pub async fn with_recreate_keyspace(self, name: &str) -> Result<Self> {
        self.recreate_keyspace(name).await?;

        Ok(self)
    }

    pub async fn create_keyspace(&self, name: &str) -> Result<()> {
        let query = format!("CREATE KEYSPACE IF NOT EXISTS {name} WITH REPLICATION = {{ 'class' : 'SimpleStrategy', 'replication_factor' : 1 }};");

        self.execute(&query, &[]).await?;

        Ok(())
    }

    pub async fn drop_keyspace(&self, name: &str) -> Result<()> {
        let query = format!("DROP KEYSPACE IF EXISTS {name};");

        self.execute(&query, &[]).await?;

        Ok(())
    }

    pub async fn with_keyspace(self, name: &str) -> Result<Self> {
        self.create_keyspace(name).await?;

        Ok(self)
    }

    pub async fn with_keyspaces(self, names: &[&str]) -> Result<Self> {
        for name in names.as_ref() {
            self.create_keyspace(name).await?;
        }

        Ok(self)
    }

    pub async fn without_keyspace(self, name: &str) -> Result<Self> {
        self.drop_keyspace(name).await?;

        Ok(self)
    }

    pub async fn without_keyspaces(self, names: &[&str]) -> Result<Self> {
        for name in names.as_ref() {
            self.drop_keyspace(name).await?;
        }

        Ok(self)
    }
}
