use std::time::Duration;

use charybdis::{
    batch::{CharybdisModelBatch, ModelBatch},
    model::Model,
    query::{CharybdisQuery, QueryExecutor},
};
use scylla::{serialize::row::SerializeRow, statement::Consistency};

#[derive(Debug, Clone, Default)]
pub struct CrudParams {
    pub consistency: Consistency,
    pub timeout: Option<Duration>,
    pub timestamp: Option<i64>,
}

impl CrudParams {
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

impl From<&CrudParams> for CrudParams {
    fn from(value: &CrudParams) -> Self {
        value.clone()
    }
}
