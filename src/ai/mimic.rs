use lancedb::connection::Connection;
use lancedb::table::Table;
use lancedb::connect;
use arrow_array::{RecordBatch, StringArray, FixedSizeListArray, Float32Array, RecordBatchIterator, ArrayRef, Array};
use arrow_schema::{Schema, Field, DataType, ArrowError};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::path::PathBuf;
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};

pub struct MimicManager {
    uri: String,
    table_name: String,
}

impl MimicManager {
    pub fn new() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".chev");
        path.push("mimic_db");
        let uri = path.to_string_lossy().to_string();
        
        // Ensure directory exists
        let _ = std::fs::create_dir_all(&path);

        Self {
            uri,
            table_name: "history".to_string(),
        }
    }

    async fn get_connection(&self) -> Result<Connection> {
        connect(&self.uri).execute().await.map_err(|e| anyhow!("LanceDB connect error: {}", e))
    }

    pub async fn add_command(&self, command: &str, vector: Vec<f32>) -> Result<()> {
        let conn = self.get_connection().await?;
        let table = match conn.open_table(&self.table_name).execute().await {
            Ok(t) => t,
            Err(_) => self.create_table(&conn, vector.len()).await?,
        };

        let schema = table.schema().await.map_err(|e| anyhow!("Schema error: {}", e))?;
        
        let cmd_array = Arc::new(StringArray::from(vec![command])) as ArrayRef;
        
        let vector_data = Arc::new(Float32Array::from(vector.clone())) as ArrayRef;
        let vector_field = Arc::new(Field::new("item", DataType::Float32, true));
        let vector_array = Arc::new(FixedSizeListArray::try_new(
            vector_field,
            vector.len() as i32,
            vector_data,
            None
        ).map_err(|e| anyhow!("FixedSizeListArray error: {}", e))?) as ArrayRef;
        
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![cmd_array, vector_array],
        ).map_err(|e| anyhow!("RecordBatch error: {}", e))?;

        let batches = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());
        table.add(batches).execute().await
            .map_err(|e| anyhow!("LanceDB add error: {}", e))?;

        Ok(())
    }

    async fn create_table(&self, conn: &Connection, dim: usize) -> Result<Table> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("command", DataType::Utf8, false),
            Field::new("vector", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dim as i32), false),
        ]));

        let batches = RecordBatchIterator::new(Vec::<std::result::Result<RecordBatch, ArrowError>>::new(), schema.clone());
        conn.create_table(&self.table_name, batches).execute().await
            .map_err(|e| anyhow!("LanceDB create table error: {}", e))
    }

    pub async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<String>> {
        let conn = self.get_connection().await?;
        let table = conn.open_table(&self.table_name).execute().await
            .map_err(|e| anyhow!("LanceDB open table error: {}", e))?;

        let stream = table
            .query()
            .limit(limit)
            .nearest_to(vector).map_err(|e| anyhow!("Nearest to error: {}", e))?
            .execute()
            .await.map_err(|e| anyhow!("Execute error: {}", e))?;

        let results: Vec<RecordBatch> = stream
            .try_collect::<Vec<_>>()
            .await.map_err(|e| anyhow!("Collect error: {}", e))?;

        let mut commands = Vec::new();
        for batch in results {
            let cmd_col = batch.column(0).as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| anyhow!("Failed to downcast command column"))?;
            for j in 0..cmd_col.len() {
                commands.push(cmd_col.value(j).to_string());
            }
        }

        Ok(commands)
    }
}
