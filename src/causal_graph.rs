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

/// Bounds the *unary* RPCs the typedb-driver issues while connecting
/// (connection open, `databases_contains`, `databases_create`, transaction
/// open). Per the driver's own doc comment on `DriverOptions::request_timeout`,
/// this does NOT bound operations inside an open transaction (queries,
/// commits) — that's why `build_store()` additionally wraps the whole
/// `connect()` call in an outer `tokio::time::timeout` using this same
/// duration as its budget.
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Cheap read query used to detect whether a database that already exists
/// actually has the Coobie semantic schema deployed to it. If `connect()`
/// created the database but a subsequent failure (e.g. a timeout) killed the
/// process before the schema transaction committed, the database would be
/// left permanently schemaless — every future `connect()` would see
/// `dbs.contains() == true` and skip deployment forever. This query detects
/// that condition so deployment can be retried.
const SCHEMA_PRESENCE_CHECK_TQL: &str = "match $e sub episode; select $e; limit 1;";

impl TypeDbCausalGraphStore {
    pub async fn connect(config: CausalGraphConfig) -> Result<Self> {
        let credentials = Credentials::new("admin", "password");
        let options = DriverOptions::new(DriverTlsConfig::disabled()).request_timeout(CONNECT_TIMEOUT);
        let addresses = Addresses::try_from_address_str(&config.url)
            .with_context(|| format!("parsing TypeDB address '{}'", config.url))?;
        let driver = TypeDBDriver::new(addresses, credentials, options)
            .await
            .with_context(|| format!("connecting to TypeDB at {}", config.url))?;

        let dbs = driver.databases();
        let already_existed = dbs
            .contains(&config.database)
            .await
            .with_context(|| format!("checking TypeDB database '{}'", config.database))?;

        if !already_existed {
            dbs.create(&config.database)
                .await
                .with_context(|| format!("creating TypeDB database '{}'", config.database))?;
            tracing::info!("Created TypeDB database '{}'", config.database);
        }

        // Deploy the schema if the database is new, OR if it already existed
        // but is missing the schema (self-repair for a prior partial failure,
        // e.g. create() succeeded but the schema transaction was interrupted
        // by a timeout before it could commit).
        let needs_schema = !already_existed || !Self::schema_is_present(&driver, &config.database).await?;

        if needs_schema {
            if already_existed {
                tracing::warn!(
                    "TypeDB database '{}' exists but is missing the Coobie semantic schema; repairing",
                    config.database
                );
            }

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

    /// Runs a cheap read query for a known schema type and returns whether it
    /// resolved. Used both for the initial "was schema deployment interrupted"
    /// check and would be reused by any future health-check tooling.
    ///
    /// Empirically verified against live TypeDB 3.12.1: querying a `sub`
    /// relationship against a type that doesn't exist (i.e. schema not
    /// deployed) does NOT return an empty answer — it errors at query
    /// analysis time (`[INF2] Type label 'episode' not found`). So a query
    /// error here is treated as "schema absent" (returns `Ok(false)`) rather
    /// than propagated, which is what drives the self-repair path in
    /// `connect()`. If the error instead reflects a genuine connectivity
    /// problem, the subsequent schema (re)deploy attempt will fail loudly and
    /// `connect()` will return that error, which is the correct fallback
    /// behavior either way.
    async fn schema_is_present(driver: &TypeDBDriver, database: &str) -> Result<bool> {
        let tx = driver
            .transaction(database, TransactionType::Read)
            .await
            .context("opening TypeDB read transaction for schema presence check")?;
        let answer = match tx.query(SCHEMA_PRESENCE_CHECK_TQL).await {
            Ok(answer) => answer,
            Err(err) => {
                tracing::debug!(
                    "TypeDB schema presence check query failed on database '{database}' \
                     (treating as schema absent): {err:#}"
                );
                return Ok(false);
            }
        };
        let mut rows = answer.into_rows();
        let mut found = false;
        while let Some(row_result) = rows.next().await {
            row_result.context("reading schema presence check row")?;
            found = true;
        }
        Ok(found)
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
            note: Some(
                "TypeDB connected; typed query path not yet implemented (Task 3 placeholder)".to_string(),
            ),
        })
    }
}

pub async fn build_store(config: CausalGraphConfig) -> std::sync::Arc<dyn CausalGraphStore> {
    if !config.enabled {
        return std::sync::Arc::new(NoopCausalGraphStore::new(config));
    }
    // `TypeDbCausalGraphStore::connect()` sets `DriverOptions::request_timeout`,
    // but per the driver's own docs that only bounds unary RPCs (connection
    // open, database checks, transaction open) — NOT operations inside an
    // open transaction (schema query, commit), which simply `.await` the next
    // stream item with no timeout at all. A blackholed host (packets dropped,
    // no RST) can therefore hang `connect()` indefinitely even with
    // `request_timeout` set. Wrap the whole call in an outer timeout so
    // `build_store()` keeps its "never fail or hang startup" guarantee.
    match tokio::time::timeout(CONNECT_TIMEOUT, TypeDbCausalGraphStore::connect(config.clone())).await {
        Ok(Ok(store)) => std::sync::Arc::new(store),
        Ok(Err(err)) => {
            tracing::warn!(
                "TypeDB causal graph unavailable ({err:#}); falling back to SQLite/memory retrieval"
            );
            std::sync::Arc::new(NoopCausalGraphStore::new(config))
        }
        Err(_) => {
            tracing::warn!(
                "TypeDB causal graph connect timed out after {}s; falling back to SQLite/memory retrieval",
                CONNECT_TIMEOUT.as_secs()
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

    /// CRITICAL regression test: `build_store()` must never hang forever when
    /// the configured TypeDB host is unreachable in a way that produces no
    /// response at all (as opposed to an immediate connection refusal).
    /// `10.255.255.1` is an RFC1918-adjacent non-routable address that
    /// reliably blackholes traffic (packets dropped, no RST, no ICMP
    /// unreachable) rather than refusing the connection outright — this is
    /// exactly the failure mode `request_timeout` alone cannot bound, since
    /// per the driver's docs that timeout doesn't cover in-transaction
    /// operations. Does not require a live TypeDB instance; runs in the
    /// normal suite.
    #[tokio::test]
    async fn build_store_falls_back_to_noop_on_blackholed_host() {
        let config = CausalGraphConfig {
            backend: CausalGraphBackend::TypeDb3,
            enabled: true,
            url: "10.255.255.1:1729".to_string(),
            database: "harkonnen_semantic_blackhole_test".to_string(),
            schema_path: "factory/coobie_semantic/typedb/schema.tql".to_string(),
            reasoning_mode: "function_backed".to_string(),
        };

        let started = std::time::Instant::now();
        let store = build_store(config).await;
        let elapsed = started.elapsed();

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
        assert!(
            elapsed < std::time::Duration::from_secs(30),
            "build_store() against a blackholed host took {elapsed:?}, expected well under 30s"
        );
    }

    /// IMPORTANT-1 regression test: a database that exists but is missing the
    /// Coobie semantic schema (simulating a prior partial failure — e.g.
    /// `dbs.create()` succeeded but the schema transaction never committed)
    /// must be self-repaired by `connect()`, not silently accepted as
    /// "already provisioned". Constructs that exact state directly via the
    /// raw driver (create the database, deliberately skip schema deployment),
    /// then calls `TypeDbCausalGraphStore::connect()` against it and confirms
    /// the schema is present afterward.
    #[tokio::test]
    #[ignore = "requires a live TypeDB instance: docker compose -f docker-compose.calvin.yml up -d typedb"]
    async fn connect_repairs_database_left_without_schema() {
        let db_name = "harkonnen_semantic_repair_check";

        // Set up the broken state directly against the raw driver, bypassing
        // TypeDbCausalGraphStore::connect() entirely so no schema is deployed.
        {
            let credentials = Credentials::new("admin", "password");
            let options = DriverOptions::new(DriverTlsConfig::disabled());
            let addresses = Addresses::try_from_address_str("localhost:1729").expect("parse address");
            let driver = TypeDBDriver::new(addresses, credentials, options)
                .await
                .expect("connect to local TypeDB");
            let dbs = driver.databases();
            if dbs.contains(db_name).await.expect("check exists") {
                dbs.get(db_name).await.expect("get").delete().await.expect("delete stale test database");
            }
            dbs.create(db_name).await.expect("create database without schema");

            // Confirm the broken state is real before testing the repair.
            let present = TypeDbCausalGraphStore::schema_is_present(&driver, db_name)
                .await
                .expect("schema presence check");
            assert!(!present, "test setup invariant: database should have no schema yet");
        }

        let config = CausalGraphConfig {
            backend: CausalGraphBackend::TypeDb3,
            enabled: true,
            url: "localhost:1729".to_string(),
            database: db_name.to_string(),
            schema_path: "factory/coobie_semantic/typedb/schema.tql".to_string(),
            reasoning_mode: "function_backed".to_string(),
        };

        let store = TypeDbCausalGraphStore::connect(config)
            .await
            .expect("connect() should repair the missing schema, not error");

        let repaired = TypeDbCausalGraphStore::schema_is_present(&store.driver, db_name)
            .await
            .expect("schema presence check after repair");
        assert!(repaired, "connect() should have deployed the schema to the pre-existing, schemaless database");

        // Clean up the test database.
        store
            .driver
            .databases()
            .get(db_name)
            .await
            .expect("get for cleanup")
            .delete()
            .await
            .expect("cleanup");
    }
}
