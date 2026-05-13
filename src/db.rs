use anyhow::Result;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::collections::BTreeMap;

use crate::config::Paths;

pub async fn init_db(paths: &Paths) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(&paths.db_file)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS runs (
            run_id TEXT PRIMARY KEY,
            spec_id TEXT NOT NULL,
            product TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS calvin_write_queue (
            queue_id TEXT PRIMARY KEY,
            run_id TEXT,
            operation TEXT NOT NULL,
            method TEXT NOT NULL,
            path TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            status TEXT NOT NULL,
            attempts INTEGER NOT NULL DEFAULT 0,
            next_attempt_at TEXT NOT NULL,
            last_error TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            confirmed_at TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_calvin_write_queue_status_next_attempt
        ON calvin_write_queue (status, next_attempt_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS run_predictions (
            prediction_id      TEXT PRIMARY KEY,
            run_id             TEXT NOT NULL UNIQUE,
            spec_id            TEXT NOT NULL,
            predicted_outcome  TEXT NOT NULL,
            risk_score         REAL NOT NULL,
            confidence         REAL NOT NULL,
            failure_phase      TEXT,
            failure_kind       TEXT,
            source_cause_ids   TEXT NOT NULL DEFAULT '',
            narrative_summary  TEXT NOT NULL,
            created_at         TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_run_predictions_spec_created
        ON run_predictions (spec_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS spec_family_profiles (
            family_id        TEXT PRIMARY KEY,
            label            TEXT NOT NULL,
            fingerprint_json TEXT NOT NULL DEFAULT '[]',
            spec_ids_json    TEXT NOT NULL DEFAULT '[]',
            created_at       TEXT NOT NULL,
            updated_at       TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_spec_family_profiles_updated
        ON spec_family_profiles (updated_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS prediction_success_reinforcements (
            reinforcement_id   TEXT PRIMARY KEY,
            run_id             TEXT NOT NULL,
            prediction_id      TEXT NOT NULL,
            source_cause_id    TEXT NOT NULL,
            prediction_error   REAL NOT NULL,
            actual_outcome     TEXT NOT NULL,
            confirmation_count INTEGER NOT NULL,
            status             TEXT NOT NULL DEFAULT 'pending_workbench_review',
            created_at         TEXT NOT NULL,
            UNIQUE(run_id, source_cause_id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_prediction_success_reinforcements_cause
        ON prediction_success_reinforcements (source_cause_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS memory_influence_exclusions (
            exclusion_id          TEXT PRIMARY KEY,
            run_id                TEXT NOT NULL,
            phase                 TEXT NOT NULL,
            briefing_scope        TEXT,
            memory_key            TEXT NOT NULL,
            memory_preview        TEXT NOT NULL,
            spec_family           TEXT NOT NULL DEFAULT 'unknown',
            expected_outcome      TEXT NOT NULL,
            actual_outcome        TEXT NOT NULL,
            exclusion_probability REAL NOT NULL,
            selection_basis       TEXT NOT NULL,
            created_at            TEXT NOT NULL,
            UNIQUE(run_id, phase, memory_key)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_influence_exclusions_run
        ON memory_influence_exclusions (run_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_influence_exclusions_key
        ON memory_influence_exclusions (memory_key, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS run_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id TEXT NOT NULL,
            phase TEXT NOT NULL,
            episode_id TEXT,
            agent TEXT NOT NULL,
            status TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "run_events",
        "episode_id",
        "ALTER TABLE run_events ADD COLUMN episode_id TEXT",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS episodes (
            episode_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            phase TEXT NOT NULL,
            goal TEXT NOT NULL,
            outcome TEXT,
            confidence REAL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            state_before TEXT,
            state_after TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "episodes",
        "state_before",
        "ALTER TABLE episodes ADD COLUMN state_before TEXT",
    )
    .await?;

    ensure_column(
        &pool,
        "episodes",
        "state_after",
        "ALTER TABLE episodes ADD COLUMN state_after TEXT",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS phase_attributions (
            attribution_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            episode_id TEXT NOT NULL UNIQUE,
            phase TEXT NOT NULL,
            agent_name TEXT NOT NULL,
            outcome TEXT NOT NULL,
            confidence REAL,
            prompt_bundle_fingerprint TEXT,
            prompt_bundle_provider TEXT,
            prompt_bundle_artifact TEXT,
            pinned_skill_ids TEXT NOT NULL DEFAULT '[]',
            memory_hits TEXT NOT NULL DEFAULT '[]',
            core_memory_ids TEXT NOT NULL DEFAULT '[]',
            project_memory_ids TEXT NOT NULL DEFAULT '[]',
            relevant_lesson_ids TEXT NOT NULL DEFAULT '[]',
            required_checks TEXT NOT NULL DEFAULT '[]',
            guardrails TEXT NOT NULL DEFAULT '[]',
            query_terms TEXT NOT NULL DEFAULT '[]',
            briefing_scope TEXT,
            briefing_token_budget INTEGER NOT NULL DEFAULT 0,
            briefing_tokens_used INTEGER NOT NULL DEFAULT 0,
            briefing_hits_provided INTEGER NOT NULL DEFAULT 0,
            stakeholder_alignment_json TEXT NOT NULL DEFAULT 'null',
            created_at TEXT NOT NULL,
            FOREIGN KEY (episode_id) REFERENCES episodes(episode_id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "phase_attributions",
        "briefing_scope",
        "ALTER TABLE phase_attributions ADD COLUMN briefing_scope TEXT",
    )
    .await?;

    ensure_column(
        &pool,
        "phase_attributions",
        "briefing_token_budget",
        "ALTER TABLE phase_attributions ADD COLUMN briefing_token_budget INTEGER NOT NULL DEFAULT 0",
    )
    .await?;

    ensure_column(
        &pool,
        "phase_attributions",
        "briefing_tokens_used",
        "ALTER TABLE phase_attributions ADD COLUMN briefing_tokens_used INTEGER NOT NULL DEFAULT 0",
    )
    .await?;

    ensure_column(
        &pool,
        "phase_attributions",
        "briefing_hits_provided",
        "ALTER TABLE phase_attributions ADD COLUMN briefing_hits_provided INTEGER NOT NULL DEFAULT 0",
    )
    .await?;

    ensure_column(
        &pool,
        "phase_attributions",
        "stakeholder_alignment_json",
        "ALTER TABLE phase_attributions ADD COLUMN stakeholder_alignment_json TEXT NOT NULL DEFAULT 'null'",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_phase_attributions_run_id_phase
        ON phase_attributions (run_id, phase)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS context_pull_records (
            pull_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            query TEXT NOT NULL,
            scope TEXT NOT NULL,
            max_tokens INTEGER NOT NULL,
            tokens_returned INTEGER NOT NULL,
            hits_returned INTEGER NOT NULL,
            hit_previews TEXT NOT NULL DEFAULT '[]',
            trigger TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (run_id) REFERENCES runs(run_id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "context_pull_records",
        "trigger",
        "ALTER TABLE context_pull_records ADD COLUMN trigger TEXT",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_context_pull_records_run_created
        ON context_pull_records (run_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS run_checkpoints (
            checkpoint_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            phase TEXT,
            agent TEXT,
            checkpoint_type TEXT NOT NULL,
            status TEXT NOT NULL,
            prompt TEXT NOT NULL,
            context_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            resolved_at TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_run_checkpoints_run_status
        ON run_checkpoints (run_id, status, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS checkpoint_answers (
            answer_id TEXT PRIMARY KEY,
            checkpoint_id TEXT NOT NULL,
            answered_by TEXT NOT NULL,
            answer_text TEXT NOT NULL,
            decision_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            FOREIGN KEY (checkpoint_id) REFERENCES run_checkpoints(checkpoint_id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_checkpoint_answers_checkpoint
        ON checkpoint_answers (checkpoint_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS causal_links (
            link_id TEXT PRIMARY KEY,
            from_event INTEGER NOT NULL REFERENCES run_events(event_id),
            to_event INTEGER NOT NULL REFERENCES run_events(event_id),
            link_type TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.5,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS lessons (
            lesson_id TEXT PRIMARY KEY,
            source_episode TEXT REFERENCES episodes(episode_id),
            pattern TEXT NOT NULL,
            intervention TEXT,
            tags TEXT NOT NULL,
            strength REAL NOT NULL DEFAULT 1.0,
            recall_count INTEGER NOT NULL DEFAULT 0,
            last_recalled TEXT,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_run_events_run_id_event_id
        ON run_events (run_id, event_id)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_causal_links_from
        ON causal_links (from_event)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_causal_links_to
        ON causal_links (to_event)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_lessons_tags
        ON lessons (tags)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Coobie causal reasoning tables ────────────────────────────────────────

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS coobie_episode_scores (
            run_id                  TEXT PRIMARY KEY,
            spec_clarity_score      REAL NOT NULL DEFAULT 0.5,
            change_scope_score      REAL NOT NULL DEFAULT 0.5,
            twin_fidelity_score     REAL NOT NULL DEFAULT 0.5,
            test_coverage_score     REAL NOT NULL DEFAULT 0.0,
            memory_retrieval_score  REAL NOT NULL DEFAULT 0.0,
            phase_success_score     REAL NOT NULL DEFAULT 1.0,
            scenario_passed         INTEGER NOT NULL DEFAULT 0,
            validation_passed       INTEGER NOT NULL DEFAULT 0,
            human_accepted          INTEGER,
            scored_at               TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "coobie_episode_scores",
        "phase_success_score",
        "ALTER TABLE coobie_episode_scores ADD COLUMN phase_success_score REAL NOT NULL DEFAULT 1.0",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS causal_hypotheses (
            hypothesis_id   TEXT PRIMARY KEY,
            run_id          TEXT NOT NULL,
            cause_id        TEXT NOT NULL,
            description     TEXT NOT NULL,
            confidence      REAL NOT NULL DEFAULT 0.5,
            hierarchy_level TEXT NOT NULL DEFAULT 'associational',
            supporting_runs TEXT NOT NULL DEFAULT '[]',
            evidence        TEXT NOT NULL DEFAULT '[]',
            counterfactuals TEXT NOT NULL DEFAULT '[]',
            created_at      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "causal_hypotheses",
        "hierarchy_level",
        "ALTER TABLE causal_hypotheses ADD COLUMN hierarchy_level TEXT NOT NULL DEFAULT 'associational'",
    )
    .await?;

    ensure_column(
        &pool,
        "causal_hypotheses",
        "evidence",
        "ALTER TABLE causal_hypotheses ADD COLUMN evidence TEXT NOT NULL DEFAULT '[]'",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS interventions (
            intervention_id TEXT PRIMARY KEY,
            run_id          TEXT NOT NULL,
            target          TEXT NOT NULL,
            action          TEXT NOT NULL,
            expected_impact TEXT NOT NULL,
            applied         INTEGER NOT NULL DEFAULT 0,
            actual_outcome  TEXT,
            created_at      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS heuristics (
            heuristic_id    TEXT PRIMARY KEY,
            cause_pattern   TEXT NOT NULL,
            effect_pattern  TEXT NOT NULL,
            intervention    TEXT NOT NULL,
            hit_count       INTEGER NOT NULL DEFAULT 0,
            success_count   INTEGER NOT NULL DEFAULT 0,
            strength        REAL NOT NULL DEFAULT 1.0,
            accepted        INTEGER NOT NULL DEFAULT 1,
            created_at      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_causal_hypotheses_run_id
        ON causal_hypotheses (run_id)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_coobie_scores_scenario
        ON coobie_episode_scores (scenario_passed, validation_passed)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS memory_embeddings (
            entry_id    TEXT NOT NULL,
            memory_root TEXT NOT NULL,
            backend_id  TEXT NOT NULL DEFAULT '',
            model_id    TEXT NOT NULL DEFAULT '',
            embedding   BLOB NOT NULL,
            embedded_at TEXT NOT NULL,
            PRIMARY KEY (entry_id, memory_root)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "memory_embeddings",
        "backend_id",
        "ALTER TABLE memory_embeddings ADD COLUMN backend_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;

    ensure_column(
        &pool,
        "memory_embeddings",
        "model_id",
        "ALTER TABLE memory_embeddings ADD COLUMN model_id TEXT NOT NULL DEFAULT ''",
    )
    .await?;

    // ── PackChat tables ───────────────────────────────────────────────────────

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat_threads (
            thread_id      TEXT PRIMARY KEY,
            run_id         TEXT,
            spec_id        TEXT,
            title          TEXT NOT NULL DEFAULT '',
            status         TEXT NOT NULL DEFAULT 'open',
            thread_kind    TEXT NOT NULL DEFAULT 'general',
            metadata_json  TEXT NOT NULL DEFAULT '{}',
            created_at     TEXT NOT NULL,
            updated_at     TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "chat_threads",
        "thread_kind",
        "ALTER TABLE chat_threads ADD COLUMN thread_kind TEXT NOT NULL DEFAULT 'general'",
    )
    .await?;

    ensure_column(
        &pool,
        "chat_threads",
        "metadata_json",
        "ALTER TABLE chat_threads ADD COLUMN metadata_json TEXT NOT NULL DEFAULT '{}'",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_chat_threads_run_id
        ON chat_threads (run_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat_messages (
            message_id      TEXT PRIMARY KEY,
            thread_id       TEXT NOT NULL REFERENCES chat_threads(thread_id),
            role            TEXT NOT NULL,  -- 'operator' | 'agent' | 'system'
            agent           TEXT,           -- which agent sent/received this
            agent_runtime_id TEXT,          -- stable dog instance identifier when present
            content         TEXT NOT NULL,
            checkpoint_id   TEXT,           -- non-null when this msg resolves a checkpoint
            created_at      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "chat_messages",
        "agent_runtime_id",
        "ALTER TABLE chat_messages ADD COLUMN agent_runtime_id TEXT",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_chat_messages_thread_id
        ON chat_messages (thread_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS memory_candidates (
            candidate_id      TEXT PRIMARY KEY,
            source_event_id   TEXT NOT NULL UNIQUE,
            thread_id         TEXT,
            run_id            TEXT,
            spec_id           TEXT,
            message_id        TEXT,
            agent_runtime_id  TEXT,
            agent             TEXT,
            role              TEXT NOT NULL DEFAULT 'system',
            operation         TEXT NOT NULL,
            raw_payload       TEXT NOT NULL DEFAULT '{}',
            distilled_content TEXT,
            dedupe_key        TEXT,
            importance_score  REAL NOT NULL DEFAULT 0.0,
            retention_class   TEXT NOT NULL DEFAULT 'working',
            learning_intent   TEXT NOT NULL DEFAULT 'awareness_only',
            sensitivity_label TEXT NOT NULL DEFAULT 'normal',
            evidence_refs     TEXT NOT NULL DEFAULT '[]',
            causality_json    TEXT NOT NULL DEFAULT '{}',
            status            TEXT NOT NULL DEFAULT 'pending',
            openbrain_ref     TEXT,
            calvin_contract_json TEXT,
            created_at        TEXT NOT NULL,
            processed_at      TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "memory_candidates",
        "dedupe_key",
        "ALTER TABLE memory_candidates ADD COLUMN dedupe_key TEXT",
    )
    .await?;

    ensure_column(
        &pool,
        "memory_candidates",
        "learning_intent",
        "ALTER TABLE memory_candidates ADD COLUMN learning_intent TEXT NOT NULL DEFAULT 'awareness_only'",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_candidates_run_status
        ON memory_candidates (run_id, status, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_candidates_thread_status
        ON memory_candidates (thread_id, status, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_candidates_dedupe_status
        ON memory_candidates (dedupe_key, status)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Operator Model Activation tables ──────────────────────────────────────

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operator_model_profiles (
            profile_id       TEXT PRIMARY KEY,
            scope            TEXT NOT NULL,
            project_root     TEXT,
            display_name     TEXT NOT NULL DEFAULT '',
            status           TEXT NOT NULL DEFAULT 'active',
            current_version  INTEGER NOT NULL DEFAULT 0,
            created_at       TEXT NOT NULL,
            updated_at       TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_operator_model_profiles_scope_project
        ON operator_model_profiles (scope, project_root)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operator_model_sessions (
            session_id      TEXT PRIMARY KEY,
            profile_id      TEXT NOT NULL REFERENCES operator_model_profiles(profile_id),
            thread_id       TEXT,
            status          TEXT NOT NULL,
            pending_layer   TEXT,
            started_by      TEXT,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            completed_at    TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_operator_model_sessions_profile_status
        ON operator_model_sessions (profile_id, status, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operator_model_layer_checkpoints (
            checkpoint_id    TEXT PRIMARY KEY,
            session_id       TEXT NOT NULL REFERENCES operator_model_sessions(session_id),
            profile_id       TEXT NOT NULL REFERENCES operator_model_profiles(profile_id),
            version          INTEGER NOT NULL,
            layer            TEXT NOT NULL,
            status           TEXT NOT NULL,
            summary_md       TEXT NOT NULL,
            raw_notes_json   TEXT NOT NULL DEFAULT '{}',
            approved_by      TEXT,
            created_at       TEXT NOT NULL,
            approved_at      TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_operator_model_checkpoints_profile_layer
        ON operator_model_layer_checkpoints (profile_id, version, layer, status)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operator_model_entries (
            entry_id              TEXT PRIMARY KEY,
            profile_id            TEXT NOT NULL REFERENCES operator_model_profiles(profile_id),
            version               INTEGER NOT NULL,
            layer                 TEXT NOT NULL,
            entry_type            TEXT NOT NULL,
            title                 TEXT NOT NULL,
            content               TEXT NOT NULL,
            details_json          TEXT NOT NULL DEFAULT '{}',
            source_checkpoint_id  TEXT NOT NULL REFERENCES operator_model_layer_checkpoints(checkpoint_id),
            status                TEXT NOT NULL DEFAULT 'current',
            superseded_by         TEXT,
            created_at            TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_operator_model_entries_profile_layer
        ON operator_model_entries (profile_id, version, layer, status)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operator_model_exports (
            export_id       TEXT PRIMARY KEY,
            profile_id      TEXT NOT NULL REFERENCES operator_model_profiles(profile_id),
            version         INTEGER NOT NULL,
            artifact_name   TEXT NOT NULL,
            content         TEXT NOT NULL,
            content_type    TEXT NOT NULL,
            created_at      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_operator_model_exports_profile_version
        ON operator_model_exports (profile_id, version, artifact_name)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operator_model_update_candidates (
            candidate_id    TEXT PRIMARY KEY,
            profile_id      TEXT NOT NULL REFERENCES operator_model_profiles(profile_id),
            run_id          TEXT,
            entry_id        TEXT,
            proposal_kind   TEXT NOT NULL,
            summary         TEXT NOT NULL,
            proposal_json   TEXT NOT NULL DEFAULT '{}',
            status          TEXT NOT NULL DEFAULT 'open',
            confidence      REAL NOT NULL DEFAULT 0.0,
            created_at      TEXT NOT NULL,
            reviewed_at     TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_operator_model_update_candidates_profile_status
        ON operator_model_update_candidates (profile_id, status, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Phase 5 — Consolidation Workbench ─────────────────────────────────────
    // Candidates are generated by Coobie at the end of a run and surface what
    // it *proposes* to promote.  The operator reviews and keeps/discards/edits
    // each one before the final consolidate action writes anything durable.

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS consolidation_candidates (
            candidate_id    TEXT PRIMARY KEY,
            run_id          TEXT NOT NULL,
            kind            TEXT NOT NULL,   -- 'lesson' | 'causal_link' | 'pattern'
            status          TEXT NOT NULL DEFAULT 'pending',  -- 'pending' | 'kept' | 'discarded'
            content_json    TEXT NOT NULL DEFAULT '{}',
            edited_json     TEXT,            -- operator-edited content, NULL = use content_json
            review_class    TEXT NOT NULL DEFAULT 'standard',
            pattern_basis_json TEXT NOT NULL DEFAULT '[]',
            confidence      REAL NOT NULL DEFAULT 0.5,
            label           TEXT NOT NULL DEFAULT '',
            created_at      TEXT NOT NULL,
            reviewed_at     TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "consolidation_candidates",
        "review_class",
        "ALTER TABLE consolidation_candidates ADD COLUMN review_class TEXT NOT NULL DEFAULT 'standard'",
    )
    .await?;

    ensure_column(
        &pool,
        "consolidation_candidates",
        "pattern_basis_json",
        "ALTER TABLE consolidation_candidates ADD COLUMN pattern_basis_json TEXT NOT NULL DEFAULT '[]'",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_consolidation_candidates_run_status
        ON consolidation_candidates (run_id, status, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── A1: LLM cost events ───────────────────────────────────────────────────
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS run_cost_events (
            event_id    TEXT PRIMARY KEY,
            run_id      TEXT NOT NULL,
            agent       TEXT NOT NULL DEFAULT '',
            phase       TEXT NOT NULL DEFAULT '',
            provider    TEXT NOT NULL DEFAULT '',
            model       TEXT NOT NULL DEFAULT '',
            input_tokens  INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            latency_ms    INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_run_cost_events_run_id
        ON run_cost_events (run_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── A2: Decision log ──────────────────────────────────────────────────────
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS decision_log (
            decision_id     TEXT PRIMARY KEY,
            run_id          TEXT NOT NULL,
            agent           TEXT NOT NULL DEFAULT '',
            phase           TEXT NOT NULL DEFAULT '',
            decision_kind   TEXT NOT NULL DEFAULT '',
            chose           TEXT NOT NULL DEFAULT '',
            alternatives_json TEXT NOT NULL DEFAULT '[]',
            justification   TEXT NOT NULL DEFAULT '',
            approved_by     TEXT,
            created_at      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_decision_log_run_id
        ON decision_log (run_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── A3: Coordination registry / ActionLease mirror ───────────────────────
    // The JSON assignments file remains the hot-path interchange surface for
    // now, but the active leases and Keeper policy events are mirrored into
    // SQLite so coordination state survives restarts and can be queried
    // alongside the rest of the run metadata.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS coordination_leases (
            agent             TEXT PRIMARY KEY,
            task              TEXT NOT NULL DEFAULT '',
            files_json        TEXT NOT NULL DEFAULT '[]',
            claimed_at        TEXT NOT NULL,
            last_heartbeat_at TEXT NOT NULL DEFAULT '',
            status            TEXT NOT NULL DEFAULT 'active',
            resource_kind     TEXT NOT NULL DEFAULT 'file',
            ttl_secs          INTEGER NOT NULL DEFAULT 0,
            guardrails_json   TEXT NOT NULL DEFAULT '[]',
            expires_at        TEXT NOT NULL DEFAULT '',
            updated_at        TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_coordination_leases_status_updated
        ON coordination_leases (status, updated_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS coordination_policy_events_db (
            event_id            TEXT PRIMARY KEY,
            managed_by          TEXT NOT NULL DEFAULT 'keeper',
            event_type          TEXT NOT NULL DEFAULT '',
            status              TEXT NOT NULL DEFAULT '',
            agent               TEXT,
            conflicting_agent   TEXT,
            files_json          TEXT NOT NULL DEFAULT '[]',
            message             TEXT NOT NULL DEFAULT '',
            created_at          TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_coordination_policy_events_created_at
        ON coordination_policy_events_db (created_at DESC)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_runtime_state (
            runtime_id         TEXT PRIMARY KEY,
            run_id             TEXT NOT NULL,
            thread_id          TEXT,
            canonical_role     TEXT NOT NULL DEFAULT '',
            display_name       TEXT NOT NULL DEFAULT '',
            ownership          TEXT NOT NULL DEFAULT '',
            status             TEXT NOT NULL DEFAULT 'active',
            provider           TEXT,
            surface            TEXT,
            source             TEXT NOT NULL DEFAULT '',
            last_heartbeat_at  TEXT NOT NULL DEFAULT '',
            started_at         TEXT NOT NULL,
            updated_at         TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_state (
            agent_name             TEXT PRIMARY KEY,
            agent_role             TEXT NOT NULL,
            llm_provider           TEXT NOT NULL,
            llm_model              TEXT NOT NULL,
            memory_block_ids       TEXT NOT NULL DEFAULT '{}',
            last_stop_reason       TEXT,
            last_active_run        TEXT,
            behavior_contract_hash TEXT,
            updated_at             TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_agent_state_updated_at
        ON agent_state (updated_at)
        "#,
    )
    .execute(&pool)
    .await?;

    seed_agent_state_from_profiles(&pool, paths).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS causal_graph_projections (
            run_id           TEXT PRIMARY KEY,
            backend          TEXT NOT NULL,
            status           TEXT NOT NULL,
            database_name    TEXT NOT NULL,
            schema_path      TEXT NOT NULL,
            graph_json       TEXT NOT NULL,
            episode_count    INTEGER NOT NULL DEFAULT 0,
            event_count      INTEGER NOT NULL DEFAULT 0,
            link_count       INTEGER NOT NULL DEFAULT 0,
            hypothesis_count INTEGER NOT NULL DEFAULT 0,
            projected_at     TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_causal_graph_projections_projected_at
        ON causal_graph_projections (projected_at DESC)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_agent_runtime_state_run
        ON agent_runtime_state (run_id, canonical_role, updated_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_agent_runtime_state_thread
        ON agent_runtime_state (thread_id, updated_at)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── v1-B: Memory supersession / invalidation persistence ─────────────────
    // Tracks when a new memory entry supersedes an older one. The old entry's
    // provenance.superseded_by field points at new_memory_id via this table.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS memory_updates (
            update_id       TEXT PRIMARY KEY,
            old_memory_id   TEXT NOT NULL,
            new_memory_id   TEXT NOT NULL,
            memory_root     TEXT NOT NULL DEFAULT '',
            reason          TEXT NOT NULL DEFAULT '',
            review_status   TEXT NOT NULL DEFAULT 'pending',
            reviewed_by     TEXT,
            review_note     TEXT,
            reviewed_at     TEXT,
            created_at      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    ensure_column(
        &pool,
        "memory_updates",
        "memory_root",
        "ALTER TABLE memory_updates ADD COLUMN memory_root TEXT NOT NULL DEFAULT ''",
    )
    .await?;

    ensure_column(
        &pool,
        "memory_updates",
        "review_status",
        "ALTER TABLE memory_updates ADD COLUMN review_status TEXT NOT NULL DEFAULT 'pending'",
    )
    .await?;

    ensure_column(
        &pool,
        "memory_updates",
        "reviewed_by",
        "ALTER TABLE memory_updates ADD COLUMN reviewed_by TEXT",
    )
    .await?;

    ensure_column(
        &pool,
        "memory_updates",
        "review_note",
        "ALTER TABLE memory_updates ADD COLUMN review_note TEXT",
    )
    .await?;

    ensure_column(
        &pool,
        "memory_updates",
        "reviewed_at",
        "ALTER TABLE memory_updates ADD COLUMN reviewed_at TEXT",
    )
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_updates_old_memory_id
        ON memory_updates (old_memory_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_memory_updates_created_at
        ON memory_updates (created_at DESC)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Phase 5b: Code-review learning records ───────────────────────────────
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS code_review_learning_records (
            record_id                  TEXT PRIMARY KEY,
            run_id                     TEXT NOT NULL,
            source_agent               TEXT NOT NULL,
            reviewer_agent             TEXT NOT NULL,
            finding_fingerprint        TEXT NOT NULL,
            files_json                 TEXT NOT NULL DEFAULT '[]',
            severity                   TEXT NOT NULL,
            resolution                 TEXT NOT NULL,
            lesson                     TEXT NOT NULL,
            evidence_refs_json         TEXT NOT NULL DEFAULT '[]',
            stale_if_file_changed_json TEXT NOT NULL DEFAULT '[]',
            status                     TEXT NOT NULL DEFAULT 'active',
            created_at                 TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_code_review_learning_records_run
        ON code_review_learning_records (run_id, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_code_review_learning_records_fingerprint
        ON code_review_learning_records (run_id, finding_fingerprint)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Phase 5b: Behavioral-change reports ─────────────────────────────────
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS behavioral_change_reports (
            report_id                       TEXT PRIMARY KEY,
            run_id                          TEXT NOT NULL UNIQUE,
            spec_id                         TEXT NOT NULL,
            status                          TEXT NOT NULL,
            summary                         TEXT NOT NULL,
            metrics_json                    TEXT NOT NULL DEFAULT '{}',
            prior_revision_candidates_json  TEXT NOT NULL DEFAULT '[]',
            artifact_json                   TEXT NOT NULL DEFAULT '{}',
            created_at                      TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_behavioral_change_reports_status
        ON behavioral_change_reports (status, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    // ── Phase B: Agent Trace Spine ────────────────────────────────────────────
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_traces (
            trace_id          TEXT PRIMARY KEY,
            run_id            TEXT NOT NULL,
            agent             TEXT NOT NULL DEFAULT '',
            phase             TEXT NOT NULL DEFAULT '',
            input_summary     TEXT NOT NULL DEFAULT '',
            reasoning_steps   TEXT NOT NULL DEFAULT '[]',
            actions_taken     TEXT NOT NULL DEFAULT '[]',
            outcome           TEXT NOT NULL DEFAULT '',
            input_tokens      INTEGER NOT NULL DEFAULT 0,
            output_tokens     INTEGER NOT NULL DEFAULT 0,
            latency_ms        INTEGER NOT NULL DEFAULT 0,
            created_at        TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_agent_traces_run_agent
        ON agent_traces (run_id, agent, created_at)
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn sync_coordination_leases(
    pool: &SqlitePool,
    state: &crate::api::AssignmentsState,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM coordination_leases")
        .execute(&mut *tx)
        .await?;

    for assignment in state.active.values() {
        sqlx::query(
            r#"
            INSERT INTO coordination_leases (
                agent,
                task,
                files_json,
                claimed_at,
                last_heartbeat_at,
                status,
                resource_kind,
                ttl_secs,
                guardrails_json,
                expires_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&assignment.agent)
        .bind(&assignment.task)
        .bind(serde_json::to_string(&assignment.files)?)
        .bind(&assignment.claimed_at)
        .bind(&assignment.last_heartbeat_at)
        .bind(&assignment.status)
        .bind(&assignment.resource_kind)
        .bind(assignment.ttl_secs)
        .bind(serde_json::to_string(&assignment.guardrails)?)
        .bind(&assignment.expires_at)
        .bind(&state.updated_at)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn seed_agent_state_from_profiles(pool: &SqlitePool, paths: &Paths) -> Result<()> {
    let profiles_dir = paths.factory.join("agents").join("profiles");
    if !profiles_dir.exists() {
        return Ok(());
    }

    let profiles = crate::agents::load_profiles(&profiles_dir)?;
    let now = chrono::Utc::now().to_rfc3339();
    for profile in profiles.values() {
        let provider_name = paths
            .setup
            .resolve_agent_provider_name(&profile.name, &profile.provider);
        let provider = paths
            .setup
            .resolve_agent_provider(&profile.name, &profile.provider);
        let model = profile
            .model_override
            .clone()
            .or_else(|| provider.map(|provider| provider.model.clone()))
            .unwrap_or_else(|| "unresolved".to_string());
        let contract_hash = agent_contract_hash(paths, &profile.name)?;

        sqlx::query(
            r#"
            INSERT INTO agent_state (
                agent_name,
                agent_role,
                llm_provider,
                llm_model,
                memory_block_ids,
                behavior_contract_hash,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, '{}', ?5, ?6)
            ON CONFLICT(agent_name) DO UPDATE SET
                agent_role = excluded.agent_role,
                llm_provider = excluded.llm_provider,
                llm_model = excluded.llm_model,
                behavior_contract_hash = excluded.behavior_contract_hash,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&profile.name)
        .bind(&profile.role)
        .bind(&provider_name)
        .bind(&model)
        .bind(contract_hash)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn upsert_agent_state_from_runtime(
    pool: &SqlitePool,
    run_id: &str,
    runtime: &crate::models::AgentRuntimeState,
) -> Result<()> {
    let agent_name = runtime.canonical_role.trim();
    if agent_name.is_empty() {
        return Ok(());
    }
    let updated_at = if runtime.last_heartbeat_at.trim().is_empty() {
        chrono::Utc::now().to_rfc3339()
    } else {
        runtime.last_heartbeat_at.clone()
    };
    let provider = runtime
        .provider
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown");

    sqlx::query(
        r#"
        INSERT INTO agent_state (
            agent_name,
            agent_role,
            llm_provider,
            llm_model,
            memory_block_ids,
            last_active_run,
            updated_at
        )
        VALUES (?1, ?2, ?3, '', '{}', ?4, ?5)
        ON CONFLICT(agent_name) DO UPDATE SET
            llm_provider = CASE
                WHEN excluded.llm_provider = 'unknown' THEN agent_state.llm_provider
                ELSE excluded.llm_provider
            END,
            last_active_run = excluded.last_active_run,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(agent_name)
    .bind(agent_name)
    .bind(provider)
    .bind(run_id)
    .bind(&updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn record_agent_stop_reason(
    pool: &SqlitePool,
    agent_name: &str,
    stop_reason: &str,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE agent_state
        SET last_stop_reason = ?2,
            updated_at = ?3
        WHERE agent_name = ?1
        "#,
    )
    .bind(agent_name)
    .bind(stop_reason)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_agent_memory_block_ids(
    pool: &SqlitePool,
    agent_name: &str,
    block_ids: BTreeMap<String, String>,
) -> Result<()> {
    if block_ids.is_empty() {
        return Ok(());
    }

    let existing = get_agent_state(pool, agent_name).await?;
    let mut merged = existing
        .as_ref()
        .map(|state| state.memory_block_ids.clone())
        .unwrap_or_default();
    merged.extend(block_ids);

    let now = chrono::Utc::now().to_rfc3339();
    let payload = serde_json::to_string(&merged)?;
    sqlx::query(
        r#"
        INSERT INTO agent_state (
            agent_name,
            agent_role,
            llm_provider,
            llm_model,
            memory_block_ids,
            updated_at
        )
        VALUES (?1, ?1, 'unknown', 'unknown', ?2, ?3)
        ON CONFLICT(agent_name) DO UPDATE SET
            memory_block_ids = excluded.memory_block_ids,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(agent_name)
    .bind(payload)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_agent_state(pool: &SqlitePool) -> Result<Vec<crate::models::AgentState>> {
    let rows = sqlx::query(
        r#"
        SELECT agent_name, agent_role, llm_provider, llm_model, memory_block_ids,
               last_stop_reason, last_active_run, behavior_contract_hash, updated_at
        FROM agent_state
        ORDER BY agent_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(agent_state_from_row).collect()
}

pub async fn get_agent_state(
    pool: &SqlitePool,
    agent_name: &str,
) -> Result<Option<crate::models::AgentState>> {
    let row = sqlx::query(
        r#"
        SELECT agent_name, agent_role, llm_provider, llm_model, memory_block_ids,
               last_stop_reason, last_active_run, behavior_contract_hash, updated_at
        FROM agent_state
        WHERE agent_name = ?1
        "#,
    )
    .bind(agent_name)
    .fetch_optional(pool)
    .await?;

    row.map(agent_state_from_row).transpose()
}

fn agent_state_from_row(row: sqlx::sqlite::SqliteRow) -> Result<crate::models::AgentState> {
    let memory_block_ids_json: String = row.get("memory_block_ids");
    let memory_block_ids = serde_json::from_str::<BTreeMap<String, String>>(&memory_block_ids_json)
        .unwrap_or_default();

    Ok(crate::models::AgentState {
        agent_name: row.get("agent_name"),
        agent_role: row.get("agent_role"),
        llm_provider: row.get("llm_provider"),
        llm_model: row.get("llm_model"),
        memory_block_ids,
        last_stop_reason: row.get("last_stop_reason"),
        last_active_run: row.get("last_active_run"),
        behavior_contract_hash: row.get("behavior_contract_hash"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn upsert_causal_graph_projection(
    pool: &SqlitePool,
    graph: &crate::models::RunCausalGraph,
    config: &crate::causal_graph::CausalGraphConfig,
) -> Result<crate::causal_graph::CausalGraphProjectionRecord> {
    let projected_at = Utc::now();
    let record = crate::causal_graph::CausalGraphProjectionRecord {
        run_id: graph.run_id.clone(),
        backend: config.backend.clone(),
        status: "sqlite_projection".to_string(),
        database: config.database.clone(),
        schema_path: config.schema_path.clone(),
        graph_json: serde_json::to_value(graph)?,
        episode_count: graph.episodes.len() as u64,
        event_count: graph.events.len() as u64,
        link_count: graph.links.len() as u64,
        hypothesis_count: graph.hypotheses.len() as u64,
        projected_at,
    };

    sqlx::query(
        r#"
        INSERT INTO causal_graph_projections (
            run_id, backend, status, database_name, schema_path, graph_json,
            episode_count, event_count, link_count, hypothesis_count, projected_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(run_id) DO UPDATE SET
            backend = excluded.backend,
            status = excluded.status,
            database_name = excluded.database_name,
            schema_path = excluded.schema_path,
            graph_json = excluded.graph_json,
            episode_count = excluded.episode_count,
            event_count = excluded.event_count,
            link_count = excluded.link_count,
            hypothesis_count = excluded.hypothesis_count,
            projected_at = excluded.projected_at
        "#,
    )
    .bind(&record.run_id)
    .bind(causal_graph_backend_label(&record.backend))
    .bind(&record.status)
    .bind(&record.database)
    .bind(&record.schema_path)
    .bind(serde_json::to_string(&record.graph_json)?)
    .bind(record.episode_count as i64)
    .bind(record.event_count as i64)
    .bind(record.link_count as i64)
    .bind(record.hypothesis_count as i64)
    .bind(record.projected_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(record)
}

pub async fn get_causal_graph_projection(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<crate::causal_graph::CausalGraphProjectionRecord>> {
    let row = sqlx::query(
        r#"
        SELECT run_id, backend, status, database_name, schema_path, graph_json,
               episode_count, event_count, link_count, hypothesis_count, projected_at
        FROM causal_graph_projections
        WHERE run_id = ?1
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?;

    row.map(causal_graph_projection_from_row).transpose()
}

pub async fn list_causal_graph_projection_summaries(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<crate::causal_graph::CausalGraphProjectionSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT run_id, backend, status, database_name, schema_path, graph_json,
               episode_count, event_count, link_count, hypothesis_count, projected_at
        FROM causal_graph_projections
        ORDER BY projected_at DESC
        LIMIT ?1
        "#,
    )
    .bind(limit.max(1))
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(causal_graph_projection_from_row)
        .map(|record| {
            record.map(|record| crate::causal_graph::CausalGraphProjectionSummary::from(&record))
        })
        .collect()
}

pub async fn list_recent_causal_graph_projections(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<crate::causal_graph::CausalGraphProjectionRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT run_id, backend, status, database_name, schema_path, graph_json,
               episode_count, event_count, link_count, hypothesis_count, projected_at
        FROM causal_graph_projections
        ORDER BY projected_at DESC
        LIMIT ?1
        "#,
    )
    .bind(limit.max(1))
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(causal_graph_projection_from_row)
        .collect()
}

pub async fn list_causal_graph_projections_for_spec(
    pool: &SqlitePool,
    spec_id: &str,
    limit: i64,
) -> Result<Vec<crate::causal_graph::CausalGraphProjectionRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT p.run_id, p.backend, p.status, p.database_name, p.schema_path, p.graph_json,
               p.episode_count, p.event_count, p.link_count, p.hypothesis_count, p.projected_at
        FROM causal_graph_projections p
        JOIN runs r ON r.run_id = p.run_id
        WHERE r.spec_id = ?1
        ORDER BY p.projected_at DESC
        LIMIT ?2
        "#,
    )
    .bind(spec_id)
    .bind(limit.max(1))
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(causal_graph_projection_from_row)
        .collect()
}

pub async fn count_causal_graph_projections(pool: &SqlitePool) -> Result<u64> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM causal_graph_projections")
        .fetch_one(pool)
        .await?;
    Ok(count.max(0) as u64)
}

fn causal_graph_projection_from_row(
    row: sqlx::sqlite::SqliteRow,
) -> Result<crate::causal_graph::CausalGraphProjectionRecord> {
    let backend = match row.get::<String, _>("backend").as_str() {
        "type_db3" | "typedb3" => crate::causal_graph::CausalGraphBackend::TypeDb3,
        _ => crate::causal_graph::CausalGraphBackend::Disabled,
    };
    let projected_at = DateTime::parse_from_rfc3339(row.get::<String, _>("projected_at").as_str())?
        .with_timezone(&Utc);

    Ok(crate::causal_graph::CausalGraphProjectionRecord {
        run_id: row.get::<String, _>("run_id"),
        backend,
        status: row.get::<String, _>("status"),
        database: row.get::<String, _>("database_name"),
        schema_path: row.get::<String, _>("schema_path"),
        graph_json: serde_json::from_str(row.get::<String, _>("graph_json").as_str())?,
        episode_count: row.get::<i64, _>("episode_count").max(0) as u64,
        event_count: row.get::<i64, _>("event_count").max(0) as u64,
        link_count: row.get::<i64, _>("link_count").max(0) as u64,
        hypothesis_count: row.get::<i64, _>("hypothesis_count").max(0) as u64,
        projected_at,
    })
}

fn causal_graph_backend_label(backend: &crate::causal_graph::CausalGraphBackend) -> &'static str {
    match backend {
        crate::causal_graph::CausalGraphBackend::TypeDb3 => "typedb3",
        crate::causal_graph::CausalGraphBackend::Disabled => "disabled",
    }
}

fn agent_contract_hash(paths: &Paths, agent_name: &str) -> Result<Option<String>> {
    let path = paths
        .factory
        .join("agents")
        .join("contracts")
        .join(format!("{agent_name}.yaml"));
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)?;
    let digest = Sha256::digest(bytes);
    Ok(Some(format!("{digest:x}")))
}

pub async fn sync_agent_runtime_state(
    pool: &SqlitePool,
    run_id: &str,
    runtimes: &[crate::models::AgentRuntimeState],
) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM agent_runtime_state WHERE run_id = ?1")
        .bind(run_id)
        .execute(&mut *tx)
        .await?;

    for runtime in runtimes {
        let started_at = if runtime.last_heartbeat_at.trim().is_empty() {
            chrono::Utc::now().to_rfc3339()
        } else {
            runtime.last_heartbeat_at.clone()
        };
        sqlx::query(
            r#"
            INSERT INTO agent_runtime_state (
                runtime_id,
                run_id,
                thread_id,
                canonical_role,
                display_name,
                ownership,
                status,
                provider,
                surface,
                source,
                last_heartbeat_at,
                started_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(&runtime.runtime_id)
        .bind(run_id)
        .bind(&runtime.thread_id)
        .bind(&runtime.canonical_role)
        .bind(&runtime.display_name)
        .bind(&runtime.ownership)
        .bind(&runtime.status)
        .bind(&runtime.provider)
        .bind(&runtime.surface)
        .bind(&runtime.source)
        .bind(&runtime.last_heartbeat_at)
        .bind(&started_at)
        .bind(&runtime.last_heartbeat_at)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    for runtime in runtimes {
        upsert_agent_state_from_runtime(pool, run_id, runtime).await?;
    }
    Ok(())
}

pub async fn insert_coordination_policy_event(
    pool: &SqlitePool,
    event: &crate::api::CoordinationPolicyEvent,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO coordination_policy_events_db (
            event_id,
            managed_by,
            event_type,
            status,
            agent,
            conflicting_agent,
            files_json,
            message,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(&event.event_id)
    .bind(&event.managed_by)
    .bind(&event.event_type)
    .bind(&event.status)
    .bind(&event.agent)
    .bind(&event.conflicting_agent)
    .bind(serde_json::to_string(&event.files)?)
    .bind(&event.message)
    .bind(&event.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::{
        CalvinConfig, OpenBrainConfig, ProvidersConfig, SetupConfig, SetupMeta, SubAgentConfig,
        TwilightBarkConfig,
    };
    use std::collections::HashMap;

    fn test_paths(root: &std::path::Path) -> Paths {
        let factory = root.join("factory");
        Paths {
            root: root.to_path_buf(),
            factory: factory.clone(),
            specs: factory.join("specs"),
            scenarios: factory.join("scenarios"),
            artifacts: factory.join("artifacts"),
            logs: factory.join("logs"),
            workspaces: factory.join("workspaces"),
            memory: factory.join("memory"),
            db_file: factory.join("state.db"),
            products: root.join("products"),
            setup: SetupConfig {
                setup: SetupMeta {
                    name: "test".to_string(),
                    template: None,
                    role: None,
                    organization: None,
                    platform: "test".to_string(),
                    anythingllm: Some(false),
                    openclaw: Some(false),
                },
                machine: None,
                providers: ProvidersConfig {
                    default: "claude".to_string(),
                    claude: Some(crate::setup::ProviderConfig {
                        provider_type: "anthropic".to_string(),
                        model: "claude-test-model".to_string(),
                        api_key_env: "ANTHROPIC_API_KEY".to_string(),
                        enabled: true,
                        credential_kind: None,
                        usage_rights: None,
                        surface: None,
                        base_url: None,
                    }),
                    gemini: None,
                    codex: None,
                    extras: HashMap::new(),
                },
                routing: None,
                mcp: None,
                calvin_archive: CalvinConfig::default(),
                twilight_bark: TwilightBarkConfig::default(),
                open_brain: OpenBrainConfig::default(),
                sub_agents: SubAgentConfig::default(),
                typedb: Default::default(),
            },
        }
    }

    #[tokio::test]
    async fn init_db_seeds_canonical_agent_state_and_runtime_sync_updates_last_run() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let paths = test_paths(dir.path());
        let profiles_dir = paths.factory.join("agents").join("profiles");
        let contracts_dir = paths.factory.join("agents").join("contracts");
        std::fs::create_dir_all(&profiles_dir).expect("profiles dir");
        std::fs::create_dir_all(&contracts_dir).expect("contracts dir");
        std::fs::write(
            profiles_dir.join("mason.yaml"),
            r#"
name: mason
display_name: Mason
role: build_retriever
provider: claude
model_override: ~
personality_file: ../personality/labrador.md
"#,
        )
        .expect("profile");
        std::fs::write(
            contracts_dir.join("mason.yaml"),
            "invariants:\n  - persists uncertainty\n",
        )
        .expect("contract");

        let pool = init_db(&paths).await.expect("init db");
        let mason = get_agent_state(&pool, "mason")
            .await
            .expect("get state")
            .expect("mason state");
        assert_eq!(mason.agent_role, "build_retriever");
        assert_eq!(mason.llm_provider, "claude");
        assert_eq!(mason.llm_model, "claude-test-model");
        assert_eq!(
            mason.behavior_contract_hash.as_deref().map(str::len),
            Some(64)
        );
        assert!(mason.last_active_run.is_none());

        sync_agent_runtime_state(
            &pool,
            "run-agent-state",
            &[crate::models::AgentRuntimeState {
                runtime_id: "mason#claude".to_string(),
                canonical_role: "mason".to_string(),
                display_name: "Mason".to_string(),
                ownership: "implementation".to_string(),
                status: "active".to_string(),
                provider: Some("claude".to_string()),
                surface: Some("local".to_string()),
                thread_id: Some("thread-agent-state".to_string()),
                source: "test".to_string(),
                last_heartbeat_at: "2026-05-12T12:00:00Z".to_string(),
            }],
        )
        .await
        .expect("sync runtime");

        let mason = get_agent_state(&pool, "mason")
            .await
            .expect("get state")
            .expect("mason state");
        assert_eq!(mason.last_active_run.as_deref(), Some("run-agent-state"));
        assert_eq!(mason.updated_at, "2026-05-12T12:00:00Z");
    }

    #[tokio::test]
    async fn update_agent_memory_block_ids_merges_existing_blocks() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let paths = test_paths(dir.path());
        std::fs::create_dir_all(paths.factory.join("agents").join("profiles"))
            .expect("profiles dir");
        let pool = init_db(&paths).await.expect("init db");

        update_agent_memory_block_ids(
            &pool,
            "mason",
            BTreeMap::from([(
                "recalled_lessons".to_string(),
                "runs/run-1/mason_briefing.json#block=recalled_lessons".to_string(),
            )]),
        )
        .await
        .expect("first block update");
        update_agent_memory_block_ids(
            &pool,
            "mason",
            BTreeMap::from([(
                "open_checks".to_string(),
                "runs/run-2/mason_briefing.json#block=open_checks".to_string(),
            )]),
        )
        .await
        .expect("second block update");

        let mason = get_agent_state(&pool, "mason")
            .await
            .expect("get state")
            .expect("mason state");
        assert_eq!(mason.memory_block_ids.len(), 2);
        assert_eq!(
            mason
                .memory_block_ids
                .get("recalled_lessons")
                .map(String::as_str),
            Some("runs/run-1/mason_briefing.json#block=recalled_lessons")
        );
        assert_eq!(
            mason
                .memory_block_ids
                .get("open_checks")
                .map(String::as_str),
            Some("runs/run-2/mason_briefing.json#block=open_checks")
        );
    }
}

async fn ensure_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    alter_sql: &str,
) -> Result<()> {
    let pragma = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&pragma).fetch_all(pool).await?;
    let exists = rows
        .iter()
        .any(|row| row.get::<String, _>("name") == column);
    if !exists {
        sqlx::query(alter_sql).execute(pool).await?;
    }
    Ok(())
}
