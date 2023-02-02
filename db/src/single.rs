use crate::errors::DbResult;
use crate::logger::Logger;
use crate::table::Table;
use crate::TableId;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Single<V>
where
    V: Serialize + DeserializeOwned,
{
    table_id: TableId,
    inner: Option<V>,
    pub logger: Logger,
}

impl<V> Table for Single<V>
where
    V: Serialize + DeserializeOwned,
{
    fn init(table_id: TableId, logger: Logger) -> Self {
        Self { table_id, inner: None, logger }
    }

    fn handle_event(&mut self, bytes: &[u8]) -> DbResult<()> {
        self.inner = bincode::deserialize(bytes)?;

        Ok(())
    }

    fn compact_repr(&self) -> DbResult<Vec<u8>> {
        if let Some(v) = &self.inner {
            let data = bincode::serialize(&Some(v))?;
            let data = Logger::log_entry(self.table_id, data);
            Ok(data)
        } else {
            Ok(vec![])
        }
    }
}

impl<V> Single<V>
where
    V: Serialize + DeserializeOwned,
{
    pub fn insert(&mut self, value: V) -> DbResult<Option<V>> {
        let log_entry = Some(&value);
        let data = bincode::serialize(&log_entry)?;

        let ret = self.inner.replace(value);

        self.logger.write(self.table_id, data)?;

        Ok(ret)
    }

    pub fn get(&self) -> Option<&V> {
        self.inner.as_ref()
    }

    pub fn clear(&mut self) -> DbResult<Option<V>> {
        let log_entry = Option::<V>::None;
        let data = bincode::serialize(&log_entry)?;
        let ret = self.inner.take();
        self.logger.write(self.table_id, data)?;
        Ok(ret)
    }
}
