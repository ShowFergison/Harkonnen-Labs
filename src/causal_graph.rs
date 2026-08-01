use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
// Not yet used in this task; Task 4's query() implementation will iterate
// TypeDB's answer stream with it.
#[allow(unused_imports)]
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use typedb_driver::{Addresses, Credentials, DriverOptions, DriverTlsConfig, TransactionType, TypeDBDriver};

use crate::setup::TypeDbConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CausalGraphBackend {
    TypeDb3,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CausalGraphStatus {
    Ready,
    Unavailable,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphConfig {
    pub backend: CausalGraphBackend,
    pub enabled: bool,
    pub url: String,
    pub database: String,
    pub schema_path: String,
    pub reasoning_mode: String,
}

impl From<&TypeDbConfig> for CausalGraphConfig {
    fn from(config: &TypeDbConfig) -> Self {
        Self {
            backend: if config.enabled {
                CausalGraphBackend::TypeDb3
            } else {
                CausalGraphBackend::Disabled
            },
            enabled: config.enabled,
            url: config.url.clone(),
            database: config.database.clone(),
            schema_path: config.schema_path.clone(),
            reasoning_mode: config.reasoning_mode.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphQuery {
    pub question: String,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub spec_id: Option<String>,
    #[serde(default)]
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphHit {
    pub label: String,
    pub summary: String,
    pub evidence_refs: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphQueryResult {
    pub status: CausalGraphStatus,
    pub backend: CausalGraphBackend,
    pub database: String,
    pub query: String,
    pub hits: Vec<CausalGraphHit>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphStatusResponse {
    pub status: CausalGraphStatus,
    pub backend: CausalGraphBackend,
    pub enabled: bool,
    pub url: String,
    pub database: String,
    pub schema_path: String,
    pub reasoning_mode: String,
    pub projection_count: u64,
    pub latest_projection: Option<CausalGraphProjectionSummary>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphProjectionRecord {
    pub run_id: String,
    pub backend: CausalGraphBackend,
    pub status: String,
    pub database: String,
    pub schema_path: String,
    pub graph_json: serde_json::Value,
    pub episode_count: u64,
    pub event_count: u64,
    pub link_count: u64,
    pub hypothesis_count: u64,
    pub projected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphProjectionSummary {
    pub run_id: String,
    pub backend: CausalGraphBackend,
    pub status: String,
    pub database: String,
    pub schema_path: String,
    pub episode_count: u64,
    pub event_count: u64,
    pub link_count: u64,
    pub hypothesis_count: u64,
    pub projected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraphProjectionInspection {
    #[serde(flatten)]
    pub record: CausalGraphProjectionRecord,
    pub highlights: Vec<CausalGraphHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalSpecFailureHistory {
    pub anchor_run_id: String,
    pub spec_id: String,
    pub projection_count: u64,
    pub failure_run_count: u64,
    pub repeated_causes: Vec<CausalRepeatedCause>,
    pub runs: Vec<CausalFailureRunSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalRepeatedCause {
    pub cause_id: String,
    pub count: u64,
    pub average_confidence: f64,
    pub run_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalFailureRunSummary {
    pub run_id: String,
    pub projected_at: DateTime<Utc>,
    pub failed_episode_count: u64,
    pub hypothesis_count: u64,
    pub top_causes: Vec<CausalGraphHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalFailureHistoryReplayExport {
    pub schema: String,
    pub generated_at: DateTime<Utc>,
    pub anchor_run_id: String,
    pub spec_id: String,
    pub projection_source: String,
    pub typedb_schema_path: String,
    pub history: CausalSpecFailureHistory,
    pub typedb_targets: Vec<String>,
    pub replay_queries: Vec<CausalReplayQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReplayQuery {
    pub label: String,
    pub purpose: String,
    pub typeql: String,
}

impl From<&CausalGraphProjectionRecord> for CausalGraphProjectionSummary {
    fn from(record: &CausalGraphProjectionRecord) -> Self {
        Self {
            run_id: record.run_id.clone(),
            backend: record.backend.clone(),
            status: record.status.clone(),
            database: record.database.clone(),
            schema_path: record.schema_path.clone(),
            episode_count: record.episode_count,
            event_count: record.event_count,
            link_count: record.link_count,
            hypothesis_count: record.hypothesis_count,
            projected_at: record.projected_at,
        }
    }
}

#[async_trait]
pub trait CausalGraphStore: Send + Sync + std::fmt::Debug {
    fn config(&self) -> &CausalGraphConfig;
    async fn query(&self, query: CausalGraphQuery) -> Result<CausalGraphQueryResult>;
}

#[derive(Debug, Clone)]
pub struct NoopCausalGraphStore {
    config: CausalGraphConfig,
}

impl NoopCausalGraphStore {
    pub fn new(config: CausalGraphConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CausalGraphStore for NoopCausalGraphStore {
    fn config(&self) -> &CausalGraphConfig {
        &self.config
    }

    async fn query(&self, query: CausalGraphQuery) -> Result<CausalGraphQueryResult> {
        let status = if self.config.enabled {
            CausalGraphStatus::Unavailable
        } else {
            CausalGraphStatus::Disabled
        };
        let note = if self.config.enabled {
            Some(
                "TypeDB 3.x semantic graph is configured, but the live driver adapter is not wired in this build.".to_string(),
            )
        } else {
            Some("TypeDB semantic graph is disabled; SQLite and memory retrieval remain authoritative.".to_string())
        };

        Ok(CausalGraphQueryResult {
            status,
            backend: self.config.backend.clone(),
            database: self.config.database.clone(),
            query: query.question,
            hits: Vec::new(),
            note,
        })
    }
}

const SCHEMA_TQL: &str = include_str!("../factory/coobie_semantic/typedb/schema.tql");

#[derive(Debug)]
pub struct TypeDbCausalGraphStore {
    // Not yet read; Task 4's real query() implementation uses it to open
    // read transactions. The placeholder query() in this task doesn't.
    #[allow(dead_code)]
    driver: TypeDBDriver,
    config: CausalGraphConfig,
}

impl TypeDbCausalGraphStore {
    pub async fn connect(config: CausalGraphConfig) -> Result<Self> {
        let credentials = Credentials::new("admin", "password");
        let options = DriverOptions::new(DriverTlsConfig::disabled());
        let addresses = Addresses::try_from_address_str(&config.url)
            .with_context(|| format!("parsing TypeDB address '{}'", config.url))?;
        let driver = TypeDBDriver::new(addresses, credentials, options)
            .await
            .with_context(|| format!("connecting to TypeDB at {}", config.url))?;

        let dbs = driver.databases();
        if !dbs
            .contains(&config.database)
            .await
            .with_context(|| format!("checking TypeDB database '{}'", config.database))?
        {
            dbs.create(&config.database)
                .await
                .with_context(|| format!("creating TypeDB database '{}'", config.database))?;
            tracing::info!("Created TypeDB database '{}'", config.database);

            let tx = driver
                .transaction(&config.database, TransactionType::Schema)
                .await
                .context("opening TypeDB schema transaction")?;
            tx.query(SCHEMA_TQL).await.context("deploying TypeDB schema")?;
            tx.commit().await.context("committing TypeDB schema")?;
            tracing::info!("Deployed Coobie semantic schema to database '{}'", config.database);
        }

        Ok(Self { driver, config })
    }
}

#[async_trait]
impl CausalGraphStore for TypeDbCausalGraphStore {
    fn config(&self) -> &CausalGraphConfig {
        &self.config
    }

    // Placeholder implementation. Task 4 replaces this with the real TypeQL
    // query path (fetch/match against the Coobie semantic schema). This
    // exists only so `TypeDbCausalGraphStore` satisfies `CausalGraphStore`
    // and can be coerced to `Arc<dyn CausalGraphStore>` by `build_store()`.
    async fn query(&self, query: CausalGraphQuery) -> Result<CausalGraphQueryResult> {
        Ok(CausalGraphQueryResult {
            status: CausalGraphStatus::Ready,
            backend: self.config.backend.clone(),
            database: self.config.database.clone(),
            query: query.question,
            hits: Vec::new(),
            note: None,
        })
    }
}

pub async fn build_store(config: CausalGraphConfig) -> std::sync::Arc<dyn CausalGraphStore> {
    if !config.enabled {
        return std::sync::Arc::new(NoopCausalGraphStore::new(config));
    }
    match TypeDbCausalGraphStore::connect(config.clone()).await {
        Ok(store) => std::sync::Arc::new(store),
        Err(err) => {
            tracing::warn!(
                "TypeDB causal graph unavailable ({err:#}); falling back to SQLite/memory retrieval"
            );
            std::sync::Arc::new(NoopCausalGraphStore::new(config))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build_store_returns_noop_when_disabled() {
        let config = CausalGraphConfig {
            backend: CausalGraphBackend::Disabled,
            enabled: false,
            url: "localhost:1729".to_string(),
            database: "harkonnen_semantic".to_string(),
            schema_path: "factory/coobie_semantic/typedb/schema.tql".to_string(),
            reasoning_mode: "function_backed".to_string(),
        };

        let store = build_store(config).await;
        let result = store
            .query(CausalGraphQuery {
                question: "what caused recent failures?".to_string(),
                run_id: None,
                spec_id: None,
                limit: 5,
            })
            .await
            .expect("query");

        assert_eq!(result.status, CausalGraphStatus::Disabled);
    }

    #[tokio::test]
    async fn noop_graph_reports_unavailable_when_typedb_configured() {
        let store = NoopCausalGraphStore::new(CausalGraphConfig {
            backend: CausalGraphBackend::TypeDb3,
            enabled: true,
            url: "localhost:1729".to_string(),
            database: "harkonnen_semantic".to_string(),
            schema_path: "factory/coobie_semantic/typedb/schema.tql".to_string(),
            reasoning_mode: "function_backed".to_string(),
        });

        let result = store
            .query(CausalGraphQuery {
                question: "what caused recent failures?".to_string(),
                run_id: None,
                spec_id: None,
                limit: 5,
            })
            .await
            .expect("query");

        assert_eq!(result.status, CausalGraphStatus::Unavailable);
        assert_eq!(result.backend, CausalGraphBackend::TypeDb3);
        assert!(result.hits.is_empty());
    }
}
