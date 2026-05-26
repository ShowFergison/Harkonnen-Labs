# Harkonnen Labs — Execution Roadmap

**Primary goal: structural coordination and trustworthy run governance on the hot path.**
The fastest reasonable path: v1-A (Keeper-backed lease enforcement) → v1-B
(memory invalidation persistence) → v1-D (operator context MVP) → Phase 2
(testable harness) → Phase 5-C (context gating, no new infra) → Phase 5-D
(PackChat conversation memory chain) → Phase 5b (OB1 memory abstraction,
MCP prompts, memory refactor) → Phase 6 (TypeDB) → Phase 7 (causal
corpus) → Phase 8 (Calvin Archive).
Phase 10 (docs, DevBench, benchmark suites) follows the coordination path rather
than interrupting it. Live twin provisioning is permanently deferred unless a
product explicitly requires running service virtualization. Phase 2's real test
execution IS the testable harness. The Calvin Archive is now a working sidecar
continuity layer, but it is not the current critical-path blocker.

---

## Maturity Ladder

| Maturity phase | Meaning | Harkonnen status |
| --- | --- | --- |
| Phase 1 — Assisted Intelligence | Copilots, chatbots, drafting help | Already surpassed |
| Phase 2 — Automated Intelligence | Rule-based workflows, permissions, governance | Already surpassed as a standalone destination |
| Phase 3 — Augmented Intelligence | Core agent with proactive suggestions, learning loops, human confidence-building | Current baseline |
| Phase 4 — Agentic Intelligence | Self-directed agents inside explicit guardrails, with structural coordination and self-monitoring | Active destination — Phase v1 closes the remaining gap |

### What still separates Harkonnen from Phase 4 (verified in source, 2026-04-18)

A structured gap analysis identified seven practical gaps. Gap-closure phases A–D addressed five of them. Three structural gaps remain before the system can legitimately claim Tier 4:

| Gap | Gap-closure status |
| --- | --- |
| Enforced authority and guardrail boundaries | **Partial** — pre-write lease denial exists in orchestrator, but Mason still needs a Keeper-backed claim/check/release lifecycle and write-path enforcement must depend on an active lease rather than advisory state |
| Live world-state modeling | Deferred — twin is still a manifest; live provisioning is permanently deferred unless a product needs it |
| Closed-loop outcome verification | Partial — observation endpoint deferred to Phase E (TypeDB dependency) |
| Structural multi-agent coordination | Mostly closed — blackboard, heartbeat, claim eviction, DB-backed lease mirrors, and PackChat-linked dog runtime rosters are real; remaining work is richer inter-dog patch/brief exchange and conflict synthesis |
| Economic and cost awareness | Closed — A1 trace spine + cost events |
| Explicit intent → plan → execution separation | Closed — B, C (OptimizationProgram) |
| External system interfaces | Open — Phase v1 External Integrations track |

### How this roadmap closes that gap

- `.harkonnen/gap-closure-progress.md` tracks strategic bridge work phases A–D (all shipped)
- Phase v1 (below) is the structural gate before the factory can be called Tier 4
- After v1, the roadmap drives through grounded execution and scoped context before deeper memory infrastructure
- Phase 10 benchmarks and docs follow the coordination path
- Operator Model and External Integrations are parallel product tracks

---

## Twin Policy

**Bramble's real test execution (Phase 2) is the testable harness.** There is no mandatory dependency on Docker-backed service virtualization anywhere in this roadmap. "Digital twin" in this system means the manifest-based twin fidelity score used for diagnostic telemetry — it does not mean a running containerized replica of the target service. `twin_fidelity_score` remains available as optional telemetry. Phase 10-D exists only as a maintenance note for that signal; it is not a Phase 10 completion gate.

If a specific product built on Harkonnen requires live service virtualization for its own testing needs, that capability can be revisited then with that product's requirements as the driver. It does not belong in the core factory sequence.

---

## Why this order

The factory needs a clear line from coordination authority to durable continuity. That line runs through:

1. **v1-A** — Keeper-backed lease enforcement. Coordination must be structural, not advisory. No write without an active lease.
2. **v1-B** — memory invalidation persistence. Superseded coordination facts are almost as dangerous as missing ones.
3. **v1-D** — operator context MVP. Runs should stop starting from scratch when operator posture is already known.
4. **Phase 2** — Bramble real test execution. This is the testable harness. `validation_passed` means nothing until it reflects real test output rather than stubs.
5. **Phase 5-C** — per-phase context gating. Once roles and leases are real, irrelevant context becomes the next quality drag.
6. **Phase 5-D** — PackChat conversation memory chain. Twilight Bark conversations become durable memory candidates, Coobie distills them, OB1 stores shared recall, and Calvin receives only governed promotion contracts.
7. **Phase 5b** — OB1 memory abstraction, MCP prompts, and memory refactor. This prepares the semantic layer without returning to the fragile local vector-store default.
8. **Phase 6** — TypeDB semantic layer for cross-run typed causal queries.
9. **Phase 7** — causal attribution corpus so the deeper continuity layer opens with real evidence.
10. **Phase 8** — the Calvin Archive: persisted identity, governed integration, D*/SSA streaming. This remains the long-horizon destination, but no longer blocks coordination-first engineering.
11. **Phase 10** — documentation, DevBench, benchmark suites. Important for external claims and usability but not on the current critical path.

Parallel tracks (Compiled State Synthesis, External Integrations, Operator Model, Hosted/Team, Calvin Archive Visualizer) advance independently of the above sequence and do not block it.

### Synthesis Stance

Harkonnen needs a first-class synthesis function, but not necessarily a tenth Labrador yet. For now, synthesis is treated as a pipeline phase that compiles accepted state into durable operator-readable artifacts using inputs from Coobie, Keeper, Flint, the decision log, the coordination registry, and the operator model. If this work later develops its own trust boundary, benchmark surface, or sustained bottleneck, it can be promoted into a dedicated Labrador with a narrow role. Until then, add synthesis as an explicit phase and artifact family rather than a new generalist agent.

Benchmark wiring advances in lockstep with implementation phases. Each phase ships with at least one measurable gate. The benchmark philosophy remains explicitly agentic-engineering shaped: measure how quickly and safely software moves through the delivery system, not just how quickly code is emitted.

---

## Phase v1 — Tier 4 Finalization

**This is the active build target.** Closes the remaining structural gaps that prevent Harkonnen from being called a genuine Tier 4 agentic workflow.

### v1-A — Guardrail Enforcement (hard blocker for Tier 4)

**Why it's a blocker:** Tier 4 requires agents to operate *inside* explicit guardrails, not just record them. Harkonnen now has a Keeper-backed workspace lease claim/check/release lifecycle with DB-backed lease mirrors, PackChat-linked dog runtime rosters, decision-log coverage for lease and planning outcomes, and Pack Board decision-log surfacing in the run detail drawer.

**What to build:**

- Keeper-backed workspace lease lifecycle is live: Mason claims `resource_kind: "workspace"` before implementation, writes depend on an active lease, and release happens at run completion or failure.
- The write-path guardrail check in `mason_generate_and_apply_edits` now depends on an active Mason workspace lease rather than treating missing coordination state as an automatic allow.
- Policy events are mirrored into SQLite, and decision records now cover Keeper lease outcomes plus the key planning choices (Scout optimization program, Mason plan selection, Sable metric attacks) so the audit trail reflects authority decisions rather than only blackboard intent.
- A canonical dog runtime registry now sits alongside leases: one identity per Labrador role, with support for multiple live worker instances (`mason#1`, `mason#codex`, `mason#claude`, etc.) carrying `thread_id`, ownership, and status through the same coordination surface.
- PackChat run threads now act as the shared conversation surface for those live dog instances so two Masons can coordinate as Mason rather than as disconnected provider personas.
- This slice is now effectively shipped on the current backend and Pack Board path; follow-on work moves from basic guardrail authority into broader coordination synthesis and operator ergonomics.

**Done when:** Mason claims a Keeper-backed workspace lease before implementation begins, a Mason edit attempt against a path that has no active workspace lease is blocked at the orchestrator level, lease and planning outcomes are written into the decision log, and the Pack Board surfaces the decision log per run.

---

### v1-B — Memory Invalidation Persistence (Phase 4b completion)

**Why:** The core persistence path is now live on the main ingest flow, and the benchmark-facing smoke has now been rerun against that stored history. The active close-out work is the operator adjudication loop for supersession events. Broader benchmark enrichment is intentionally deferred until after the current narrow end-to-end Harkonnen pass.

**Shipped on the current path:**

- `memory_updates` table in `src/db.rs`: `(update_id, old_memory_id, new_memory_id, reason, created_at)`
- `invalidated_by: Option<String>` on the memory record schema (references `update_id`)
- Coobie ingest pipeline: before writing a new memory entry, check for semantic near-duplicates with conflicting claims via cosine similarity on the embedding store. If found above threshold, write a supersession record and set `invalidated_by` on the old entry.
- `GET /api/memory/updates` endpoint returning supersession history
- Memory Board UI panel showing supersession history alongside the rest of Coobie's recalled state

**Remaining close-out:**

- No additional v1-B blockers after the operator confirm/reject loop lands; defer wider benchmark report polish and cross-suite metric expansion until after the narrow full-system pass.

**Status:** Core path verified on the current code line. A repeated ingest from the same source path now writes `memory_updates` rows, marks stale notes with `superseded_by`, surfaces the history through `GET /api/memory/updates` and the Pack Board Memory panel, and supports operator confirm/reject review from the Memory Board. The bundled StreamingQA smoke fixture has also been rerun under `lm-studio-local`, producing `1.0000` accuracy, exact match, evidence hit rate, and updated-fact accuracy while persisting the benchmark-local supersession row.

---

### v1-C — FailureKind Classification

**Why:** Mason's fix loop should not handle all failures identically. A wrong-answer failure (test ran, output was wrong) requires a different fix prompt than a compile error (code never ran).

**Shipped on the current path:**

- `FailureKind` enum in `src/models.rs`: `CompileError`, `TestFailure`, `WrongAnswer`, `Timeout`, `Unknown`
- Validation summary construction classifies stdout/stderr-style details from visible checks, including compile/build errors, generic test failures, wrong-answer diffs, and timeouts.
- `WrongAnswer` triggers a distinct Mason validation-fix prompt that asks Mason to study the expected/actual diff and fix implementation logic without modifying tests.
- `failure_kind` is persisted on `ValidationSummary`, included in validation summaries, and recalculated after validation harness mutations so Coobie can pattern-match on failure type in causal records.

**Done when:** A run with a wrong-answer test failure shows `failure_kind: WrongAnswer` in the run summary and Mason's fix attempt uses the diff-focused prompt.

**Status:** Shipped and covered by focused classifier tests. Keep broader benchmark expansion deferred until the narrow full-system pass is complete.

---

### v1-D — Operator Model Minimum Viable

**Why:** Scout's intent generation and Coobie's preflight have no connection to operator context. Without this, every new spec starts from scratch regardless of how well Coobie knows the operator's patterns.

**Shipped on the current path (two-layer MVP, not the full five-layer spec):**

- PackChat `interview` command: initiates a two-layer intake (operating rhythms + recurring decisions) with checkpoint approval after each layer
- `commissioning-brief.json` artifact generated from the approved layers: contains operator's primary work patterns, preferred tools, recurring decisions, and risk tolerances
- Scout draft integration: when a `commissioning-brief.json` exists for the operator, Scout includes its top-3 patterns in the intent package prompt
- Coobie preflight integration: operator's stated risk tolerances contribute to `required_checks` and guardrail text
- `operator_model_sessions`, `operator_model_layer_checkpoints`, and `operator_model_exports` are now exercised by the approval/export path; completed sessions stamp the project under `.harkonnen/operator-model/`

**Done when:** An operator who has completed the two-layer interview sees their stated patterns reflected in Scout's intent packages and Coobie's required checks on subsequent runs.

**Status:** MVP shipped and hardened. The Pack Board operator-model flow can now approve the active layer, advance the session, generate the commissioning brief, persist export metadata, and surface preferred-tool / risk-tolerance signals into Coobie preflight. Full five-layer interview and post-run update review remain in the parallel Operator Model product track.

---

### v1-E — Transactional Execution And Approval Boundaries

**Why:** Guardrails are stronger when high-impact actions have explicit transaction boundaries rather than relying on best-effort cleanup after a mistake. If a run is about to mutate sensitive code, open a privileged MCP surface, or cross a policy threshold, Harkonnen should be able to pause, request approval, and either commit or roll back from a known boundary. This is the operational analogue of the Soul-of-AI requirement that continuity and policy remain inspectable rather than implicit.

**What to build:**

- Transaction envelope for high-impact phases: capture an explicit pre-action snapshot, planned mutation set, approval state, and rollback note before execution proceeds. **Shipped for implementation-phase Mason LLM edits** via `transaction_implementation.json`, `transaction_implementation.md`, and a run-local `transaction_backups/implementation_pre_action` restore point.
- Human-interrupt checkpoint for guarded transitions: if Keeper or Coobie flags a privileged step, the run pauses at a reversible boundary rather than drifting forward and apologizing later. **Shipped:** Coobie implementation blockers now create a `transaction_approval_required` checkpoint before Mason edits are applied.
- Operator checkpoint resolution: **shipped for implementation transactions.** Resolving the checkpoint with approve rehydrates `spec.yaml`, `target_source.json`, `intent.json`, `coobie_briefing.json`, and `implementation_plan.md`, applies the Mason edit lane to the staged workspace, finalizes the transaction artifact, resumes Bramble visible validation, then continues through Sable hidden scenarios, Flint artifacts, and Coobie causal reporting when the tool boundary is approved. Reject aborts without mutation. Revise records operator guidance and leaves the run in a revision-requested state.
- Rollback execution and artifact written per guarded transition: what was attempted, what state changed, what was restored, and what residual risk remains. **Shipped:** rollback restores the staged `product/` workspace from the transaction backup, verifies it against the pre-action snapshot, and records `rolled_back` or `rolled_back_with_drift`.
- Privileged MCP/tool transaction envelope: **shipped at the tool-surface boundary.** The tools phase now writes `tool_transaction.json` and `tool_transaction.md`, classifies configured MCP servers and relevant host commands, auto-approves read-only/local surfaces, opens `tool_transaction_approval_required` when write, network, secret-bearing, or external-process surfaces are present, and resumes hidden-scenario/artifact continuation after operator approval when visible validation is already complete.
- Invocation-level gateway: **shipped for host-command execution inside the run loop.** Build and validation commands now write `tool_invocations.json` and `tool_invocations.md`, classify each actual invocation at execution time, auto-approve common local build/test commands, and require an approved tool transaction before higher-risk external-process invocations proceed.
- Decision-log integration: approval, commit, rollback, and abort outcomes become explicit decision records rather than only phase logs. **Shipped:** implementation transaction boundary, operator approval/reject/revise/rollback, transaction commit, transaction rollback, tool transaction boundary, and tool approval/reject/revise outcomes are recorded in the run decision log.
- Remaining work: extend the same invocation-level gateway to proxied third-party MCP calls if Harkonnen becomes the runtime broker for external MCP traffic rather than only recording/enforcing host-command invocations inside the run loop.

**Done when:** A guarded run can pause before a privileged transition, record an approval or rejection, and either commit or roll back from a named boundary with an auditable artifact.

**Status:** Implementation transaction approval, visible-validation continuation, hidden-scenario/artifact/causal-report continuation, rollback execution, privileged tool-surface transaction envelopes, and invocation-level host-command gateway enforcement are shipped. Harkonnen now opens auditable boundaries around Mason LLM edits and records/enforces actual build/validation tool invocations inside the run loop.

---

### v1 Benchmark / product gate

- Decision audit log surfaced in Pack Board per run
- Memory supersession events returned by `GET /api/memory/updates`
- StreamingQA first run published — belief-update accuracy
- At least one run showing `failure_kind: WrongAnswer` in the validation summary
- At least one run where Scout's intent package references operator model context

---

## Phase 2 — Bramble Real Test Execution

**This is the testable harness.** Until this ships, `validation_passed` reflects scenario results and stubs rather than real test output. Every downstream quality signal (Coobie's `test_coverage_score`, Phase 10 benchmarks, the fix loop's wrong-answer path) depends on real exit codes coming from real test commands.

**What to build:**

- `bramble_run_tests` in orchestrator — reads `spec.test_commands` (same detection logic as Piper) and executes them in the staged workspace
  Shipped: explicit Bramble test harness now runs raw `spec.test_commands` through shell-preserving execution, records `real_test_commands` / `passed_real_test_commands` in `ValidationSummary`, and writes corpus results from those runs.
- Stdout/stderr streamed as `LiveEvent::BuildOutput` on the broadcast channel (already exists — Bramble just needs to use it)
- `ValidationSummary` populated from real exit codes and parsed test output, not from scenario results or stubs
- Bramble's phase attribution records `validation_passed: true/false` from actual runs
- Feed result back as `test_coverage_score` into the Coobie episode at ingest time
  Shipped: Coobie now prefers explicit real-test counts over generic scored-check counts when `spec.test_commands` were present, and run reports show explicit test-command totals.
- **Mason online-judge feedback loop** — `FailureKind::WrongAnswer` (wired in v1-C) now carries structured wrong-answer evidence from Bramble's explicit test-command harness into `validation.json`, the run report, and Mason's diff-focused repair prompt. The loop also records `validation_repair_attempts.{json,md}`, classifies each retry as `resolved / improved / stalled / regressed`, feeds that guidance into the next Mason attempt, and stops early after repeated non-improving retries.
- **LiveCodeBench adapter** — native builtin now wired through the benchmark manifest and report path; generates per-problem artifacts plus suite-level pass@1 breakdowns in benchmark reports.
- **Benchmark posture** — keep `LiveCodeBench` as the single active external coding canary while the narrow end-to-end Harkonnen pass matures. Additional public coding benchmarks stay adapter-ready but are not a near-term build gate unless they answer a question the current canary cannot.
- **Aider Polyglot adapter** — remains adapter-ready for a later comparison lane once the core run path is stable enough to justify broader external measurement.

**Benchmark gate:**

- `local_regression` stays green on every merge
- `LiveCodeBench` remains wired and producing artifacts as the active external coding canary
- additional coding benchmark expansion stays deferred until the core run path is practical and trustworthy in daily use
- `SWE-bench Verified`, `SWE-bench Pro`, and `Aider Polyglot` remain comparison-ready backlog items rather than near-term gates

**Done when:** A spec with `test_commands` shows real pass/fail in the run report, Coobie's episode scores reflect actual test execution, and Mason's fix loop handles wrong-answer failures. The explicit test harness, structured wrong-answer evidence path, retry-improvement tracking, and LiveCodeBench canary lane are now shipped; broader benchmark expansion remains intentionally deferred behind core factory maturity.

---

## Phase 5-C — Per-Phase Context Gating + Sub-Agent Dispatch

**Why:** Three compounding problems hit the orchestrator at this phase. First, every agent receives the same Coobie preflight briefing regardless of role — Scout, Mason, and Sable have fundamentally different information needs, and the undifferentiated corpus wastes context window and risks priming Sable with Mason's implementation reasoning before hidden-scenario scoring. Second, briefing construction, Sable evaluation, and Mason failure diagnosis each inflate the orchestrator's main context window with exploration that never needs to cross back: the orchestrator only needs the finished output, not the retrieval trace. Third — and most subtly — even a correctly scoped briefing can be wrong-sized: too many hits dilutes the relevant signal just as much as wrong categories. The ideal briefing has scope (right categories), relevance (right entries within those categories re-ranked against the specific task), and volume (right total token count). All three must hold simultaneously. Both the scoping and volume problems share the same solution phase: replace the flat `BriefingScope` parameter with a `ContextTarget` that carries all three dimensions, and isolate the high-context work in sub-agents.

This is a retrieval-shaping and isolation capability, not a storage change. It does not require TypeDB or Qdrant. It is placed here — before the memory module refactor — because the `BriefingScope` enum and filter logic can land in `src/coobie.rs` now and move cleanly into `src/memory/briefing.rs` during Phase 5b's refactor. The `SubAgentDispatcher` lands in `src/subagent.rs`; orchestrator call sites are thin wrappers.

Full design: `factory/context/briefing-scope-design.md` (BriefingScope) and `factory/context/sub-agent-dispatch-design.md` (SubAgentDispatcher).

**Explicit sub-slice order so this phase does not blur again:**

- **Phase 5-C1 — shipped:** `ContextTarget` metadata for Coobie preflight, budgeted memory-hit shaping, stamped-context section injection tracking, and attribution observability (`briefing_scope`, token budget/usage, hits provided).
- **Phase 5-C2 — shipped:** distinct Scout, Mason, and Sable briefing projections are now materialized as run artifacts, Scout and Mason now consume their scoped briefings on the hot path, Sable's generated-scenario prompt now receives only the scenario-pure scoped summary, and repo-local prompt support / retriever bundles are filtered by role so hidden-scenario work is not primed with Mason implementation context.
- **Phase 5-C3 — SHIPPED 2026-04-28:** `SubAgentDispatcher` in `src/subagent.rs` with `dispatch()` method and three-tier resolution order (profile `dispatch:` block → `[sub_agents.*]` in harkonnen.toml → `default_mode`). `SubAgentConfig` + `SubAgentTaskConfig` parsed into `SetupConfig`. `dispatch:` field added to `AgentProfile`. `SubAgentDispatcher` added to `AppContext`. Both high-context call sites wired: Coobie briefing construction → `dispatch_coobie_briefing()` (DirectLlm: behavioral no-op; ClaudeCodeAgent: isolated call with clean context window); Sable scenario evaluation → `isolation_prefix` injected into `sable_generate_and_evaluate()` when backend is ClaudeCodeAgent (Sable firewall: drops implementation_notes, mason_plan, edit_rationale from context). All 120 existing tests pass.

**What to build:**

- `BriefingScope` enum in `src/coobie.rs` (migrates to `src/memory/briefing.rs` in Phase 5b): `ScoutPreflight`, `MasonPreflight`, `PiperPreflight`, `SablePreflight`, `CoobiConsolidation`, `OperatorQuery`. Each variant carries a `phase_id` and a `role` tag. `BriefingScope` defines the *category filter only* — it remains the clean enum it is now.
- Scope-keyed retrieval filter: each scope defines an `allow_categories` list (e.g. Scout: `spec_history, prior_ambiguities, operator_model`; Mason: `failure_patterns, fix_patterns, workspace_guardrails, causal_links`; Sable: `scenario_patterns, hidden_scenario_outcomes` — explicitly excludes Mason implementation notes).
- **`ContextTarget`** struct wrapping `BriefingScope` with the two missing dimensions:

  ```rust
  pub struct ContextTarget {
      pub scope: BriefingScope,
      pub task_description: String,  // re-ranks hits by similarity to THIS task, not a generic query
      pub token_budget: u32,         // hard cap; hits truncated in relevance order after scope filter
      pub min_hits: u32,             // always include at least N hits regardless of score
      pub required_sections: Vec<ContextSection>, // injected first, outside the token budget
  }
  ```

  `build_targeted_briefing(target: ContextTarget, run_id, spec_id) -> BriefingPackage` replaces the current `build_preflight_briefing`. Internally: scope filter → re-rank hits by `task_description` embedding similarity → inject `required_sections` → fill remaining budget with top-ranked hits. The orchestrator constructs a `ContextTarget` per phase entry point using a `phase_defaults()` function that provides sane budgets and required sections per scope.

- **Stamped project interview context as first-class preflight input** — the repo-stamp interview's Mythos/Pathos/Ethos/Episteme/Praxis material (purpose, stakes, stakeholder attitudes, prohibitions, vertical, skill sources, MCP posture) should be loaded from `.harkonnen/repo.toml` and injected into Scout + Coobie briefing shaping as a `required_sections` entry (always present, outside the ranked budget). This keeps project posture inspectable and continuity-aligned rather than leaving it trapped in generated markdown artifacts.
- Wire in orchestrator: construct a `ContextTarget` at each phase entry point (Scout, Mason, Sable are the critical three; others can default to `OperatorQuery` scope with a conservative `token_budget`). The `task_description` is the spec title + active task for that phase.
- Coobie episode record: add `briefing_scope`, `briefing_tokens_used`, and `briefing_hits_provided` fields so causal analysis can distinguish whether a lesson was visible at the relevant phase and whether the briefing was over- or under-loaded.
- **`SubAgentDispatcher`** in `src/subagent.rs` with `dispatch(task, input) -> SubAgentResult`. Backends: `DirectLlm` (current behavior, no isolation), `ClaudeCodeAgent { model, max_turns }`, `CodexPlanAgent { model, context_paths }`, `GeminiAgent { model }`, `ExternalMcp { server, tool }`. Sub-agents read only; all memory and SQLite writes remain in the orchestrator.
- **`[sub_agents]` config section** in `harkonnen.toml`: `default_mode = "direct_llm"` plus named task entries (`coobie_briefing`, `sable_evaluation`). Per-environment overrides via `setups/` follow the existing named-setup pattern.
- **Agent profile `dispatch:` block** — coobie and sable profiles declare per-task backend preferences that take priority over the global TOML config. Resolution order: profile `dispatch.<task>` > `[sub_agents.<name>]` > `[sub_agents] default_mode`.
- Wire Phase 5-C orchestrator call sites: `BriefingConstruction` dispatches to `ClaudeCodeAgent`; `ScenarioEvaluation` dispatches to `ClaudeCodeAgent` (isolation-critical). `DirectLlm` is the fallback for all other tasks.
- `SubAgentResult` fields (`backend_used`, `tokens_used`, `duration_ms`) appended to `agent_traces` table for cost and performance observability.

**Sable isolation constraint (non-negotiable):** `SablePreflight` scope must never include retrieved hits tagged `implementation_notes`, `mason_plan`, or `edit_rationale`. This is the hidden-scenario firewall. If a hit's tag set intersects these, it is dropped regardless of relevance score. The `ClaudeCodeAgent` backend for `ScenarioEvaluation` enforces this at the sub-agent system prompt level in addition to the scope filter.

**Memory write discipline:** Sub-agents dispatched via `SubAgentDispatcher` may not write to memory, SQLite, or the Calvin Archive. Their system prompts list these as `disallowed_tools`. The orchestrator receives `SubAgentResult.output` and decides what to persist.

**Done when:**

- Scout, Mason, and Sable each receive a distinct briefing shaped to their role; stamped repo interview context is visible in the relevant preflight surfaces; run artifacts now include `scout_briefing.{json,md}`, `mason_briefing.{json,md}`, and `sable_briefing.{json,md}`; scoped repo-local prompt support is filtered per role; and Sable's briefing verifiably contains no Mason implementation content.
- `ContextTarget` struct in `src/coobie.rs`; `build_targeted_briefing()` replaces `build_preflight_briefing`; `phase_defaults()` provides sane per-scope budgets; episode record captures `briefing_tokens_used` and `briefing_hits_provided`.
- `SubAgentDispatcher` struct in `src/subagent.rs` with `dispatch()` method; `[sub_agents]` section parsed from `harkonnen.toml` into `SetupConfig`; `coobie_briefing` and `sable_evaluation` tasks dispatch to `ClaudeCodeAgent` backend.
- Agent profile `dispatch:` blocks parsed for coobie and sable; resolution order enforced.
- All existing tests pass (`DirectLlm` backend is a behavioral no-op vs. current calls).

---

## Prediction System — **SHIPPED 2026-04-28**

Closes the learning loop by making Coobie's expectations explicit before each run and measuring how wrong they were afterward.

**What was built:**

- **TypeDB schema** — `prediction_record` + `prediction_result` entities, `prediction_for_run` + `prediction_verified` relations. New attributes: `predicted-outcome`, `actual-outcome`, `prediction-error`, `risk-score`, `failure-phase`, `failure-kind-label`, `source-cause-ids`.
- **Calvin sidecar** — `record_prediction()`, `record_prediction_result()`, `get_prediction()` in `archive.rs`; `POST /runs/{id}/predictions`, `GET /runs/{id}/predictions`, `POST /runs/{id}/prediction-result` in `api.rs`.
- **Calvin client** — `RunPrediction` and `PredictionOutcome` structs + three client methods in `calvin_client.rs`.
- **Prediction synthesis** (`synthesize_run_prediction()` in `orchestrator.rs`) — heuristic risk scoring from `prior_causes` frequency and pass rate + `required_checks` count + `open_questions` count → predicted outcome (`pass`/`uncertain`/`fail`) + risk score 0–1. Marked `basic_heuristic`; Phase 7 replaces with a model-driven classifier trained on the causal attribution corpus.
- **Prediction error** (`compute_prediction_error()`) — asymmetric: predicted `pass`, got `fail` → 1.0 (worst); false alarm → 0.6; `uncertain` → max 0.2. Missing a failure is penalised more than raising a false alarm.
- **Orchestrator wiring** — prediction emitted after every `dispatch_coobie_briefing()`; prediction result emitted at both run completion paths (alongside `try_close_calvin_run()`).
- **E2E tests** — 4 new tests in `tests/e2e_integration.rs`: `prediction_recorded_before_run`, `prediction_result_recorded_after_run`, `prediction_error_is_maximum_for_false_confidence`, `full_prediction_round_trip_in_calvin`. 128/128 tests passing.

**What this enables:** every run now produces a `prediction_error` score. Runs where the error is high (unexpected failures) are natural candidates for intensive causal annotation in Phase 7. The error signal is the primary input for Phase 8 posture adaptation.

---

## Phase 5-D — PackChat Memory Distillation Chain

**Unlocks:** The full conversation-to-continuity path. Twilight Bark carries live PackChat events, Harkonnen persists them as memory candidates, Coobie distills them into durable thoughts, Open Brain (OB1) stores shared semantic recall, and Calvin receives only governed promotion contracts.

This is now the critical memory slice. It replaces the old assumption that the next step must be a larger local vector store. The local fastembed/SQLite vector store remains opt-in; OB1 is the shared recall default.

**Twilight Bark dependency rule:** Twilight Bark is an upstream transport dependency for Harkonnen Labs, not a downstream consumer of Harkonnen concepts. Harkonnen-owned operation labels and PackChat/Calvin schemas may ride over Twilight's generic task/event surface, but Twilight Bark must stay free of Harkonnen imports, Calvin archive assumptions, and Labrador-specific runtime policy.

### Calvin Archive sidecar correctness (pre-5-D gate) — **SHIPPED 2026-04-28**

End-to-end integration tests (`tests/e2e_integration.rs`) confirmed and drove two correctness fixes to the live Calvin sidecar.

**Fix 1 — `get_active_beliefs` agent-scoped** (`archive.rs:241`) — **done**.
Added `(source: $b, target: $a) isa stabilizes` join to the TypeQL query. Foreign beliefs from other `agent_self` instances no longer appear in any agent's result set. Test: `calvin::beliefs_scoped_to_named_agent` — green.

**Fix 2 — `check_adaptation_safe` semantic negation patterns** (`archive.rs:323`) — **done**.
Expanded the deny-list to `avoid`, `without`, `less`, `deprioritise`/`deprioritize`, `reduce`, `replace`, `instead of`, `abandon`, `drop` in addition to the original `not`/`remove`/`eliminate`. Marked `basic_heuristic`; Phase 6 upgrades to an embedding classifier. Test: `calvin::adaptation_safety_catches_semantic_negation` — green.

**Fix 3 — PackChat causal-link addressability** (`archive.rs:141`, `archive.rs:350`) — **done**.
`record_experience` now preserves a caller-supplied `episode_id` as the Calvin `experience.uuid` and links the experience to its `run_record` via `belongs_to_run`. `record_causal_link` now inserts the schema-valid `(cause: $cause, effect: $effect) isa causally_contributed_to` relation instead of attempting to match generated random UUIDs or use non-schema relation syntax. The TypeDB schema now declares `experience` as a player for causal cause/effect and run-link roles, and `run_record` as the run-link role player. This makes PackChat `causation_id -> message_id` links addressable on the real TypeDB path, not only in mocks.

**Fix 4 — status update upsert semantics** (`archive.rs:379`) — **done**.
`update_agent_status` no longer requires an existing `status` attribute. It deletes an old status when present, then inserts the requested status, so the Twilight presence TTL watcher can mark newly seeded or statusless agents offline/active without silently doing nothing.

### Data path

```text
Twilight Bark / PackChat envelope
  -> memory_candidates table
  -> Coobie distillation worker
  -> Open Brain capture_thought
  -> Open Brain search_thoughts in briefings
  -> Calvin promotion contract
```

### What to build

- `memory_candidates` table keyed by `candidate_id`, with `source_event_id`, `thread_id`, `run_id`, `agent_runtime_id`, `operation`, `raw_payload`, `importance_score`, `retention_class`, `sensitivity_label`, `evidence_refs`, `causality`, `status`, and timestamps.
- PackChat/Twilight ingest hook that writes candidate rows idempotently from local PackChat messages and remote Twilight Bark envelopes.
- Coobie distillation worker that groups nearby conversation fragments, summarizes memory-worthy content, dedupes against recent candidates and OB1 recall, classifies retention as `ephemeral`, `working`, `shared_recall`, or `calvin_candidate`, and preserves provenance.
- OB1 writer path using `OpenBrainClient::capture_thought` for `shared_recall` candidates. Captured content must include source thread/run/event provenance and tags, but not raw secrets or unapproved sensitive payloads.
- OB1 reader path already started by `OpenBrainClient::search_thoughts`; complete the ranking so OB1 hits sit beside repo-local memory, PackChat recency, and Calvin-approved facts in targeted Coobie briefings.
- Calvin promotion contract for `calvin_candidate` rows. The contract includes proposed chamber targets, evidence refs, inference posture, confidence, Pathos score, preservation note, and recommended governance outcome: `accept`, `modify`, `reject`, or `quarantine`.
- Operator review surface in the Consolidation Workbench for memory candidates and Calvin promotions. OB1 shared recall may be automatic for low-risk approved classes; Calvin promotion remains governed.
- **Compiled Calvin promotion contracts** — **SHIPPED 2026-04-29**. `harkonnen.calvin.promotion.v1` now carries the gbrain-inspired shape Harkonnen needs: `compiled_claim`, append-only `evidence_timeline`, `source_authority`, `staleness_triggers`, `review_state`, and `integration_recommendation`. This is a contract upgrade only; Calvin canonical state still changes only after governance accepts or modifies the proposal. Regression test: `orchestrator::tests::calvin_candidate_becomes_governed_promotion_without_archive_mutation` — green.
- **`needs_reconsolidation` stale-memory status** — **SHIPPED 2026-04-29**. When newer PackChat evidence explicitly revises, supersedes, or invalidates an older memory candidate/source event, the older candidate is marked `needs_reconsolidation` instead of being overwritten. The API and Memory tab surface the status, blocker, trigger candidate, and reconsolidation reason so an operator can decide whether to refresh OB1, promote to Calvin, quarantine, or discard. Regression test: `orchestrator::tests::newer_packchat_evidence_marks_prior_memory_needs_reconsolidation` — green.
- **Memory chain health report** — **SHIPPED 2026-04-29**. `GET /api/runs/{id}/memory/candidates` now returns a `harkonnen.memory_chain_health.v1` report covering candidate backlog, OB1 capture failures, Calvin promotion backlog, stale distillations, missing evidence refs, duplicate OB1 thoughts, sensitivity holds, and service readiness for `twilight-bark.packchat`, `openbrain.mcp`, `calvin.archive`, and `harkonnen.api`. The Memory tab renders chain health, quality counts, and service readiness beside the existing blocker banner.
- **Source authority taxonomy** — **SHIPPED 2026-04-29**. Memory candidates now expose `source_authority`, derived from evidence refs as operator, agent observation, tool output, test result, code diff, PackChat statement, OB1 recall, or Calvin-approved fact. The same taxonomy feeds Calvin promotion contracts, evidence timelines, memory-chain health counts, and the Memory tab. Regression test: `chat::tests::source_authority_taxonomy_prefers_stronger_evidence` — green.
- **Automatic candidate processing on run close** — **SHIPPED 2026-04-28**. `try_process_memory_candidates_on_close(run_id)` is now called immediately after `try_close_calvin_run()` at both successful and failed run completion paths in `orchestrator.rs`. Processing failures log a warning and leave candidates as `pending` for manual retry (retry scheduler remains future work). E2E test: `memory_candidates::run_close_triggers_candidate_processing` — green.
- **Candidate retry semantics** — **SHIPPED 2026-04-28**. OB1 capture failures and Calvin promotion enqueue failures now mark candidates as `retry_pending`, and the candidate processor scans both `pending` and `retry_pending` rows so manual/API-triggered processing retries transient failures instead of silently stalling. `OpenBrainClient` now uses a configurable short timeout (`open_brain.timeout_ms`, default 2500 ms) so OB1 outages do not block productive run close for 30 seconds. Regression test: `chat::tests::pending_memory_candidate_scan_includes_retry_pending` — green.
- **Candidate retry/operator surface** — **SHIPPED 2026-04-28**. `GET /api/runs/{id}/memory/candidates` now returns `status_counts`, `retryable`, and `actionable` totals covering `pending`, `retry_pending`, `waiting_openbrain`, `held_for_review`, `captured_openbrain`, and `promotion_pending`; `POST /api/runs/{id}/memory/candidates/retry` is a clear retry alias for the processing endpoint. The Run Detail drawer now includes a Memory tab with candidate counts, status chips, recent candidate previews, and a one-click retry action.
- **Candidate review actions** — **SHIPPED 2026-04-28**. `POST /api/runs/{id}/memory/candidates/{cid}/approve` releases `held_for_review`, `retry_pending`, or `waiting_openbrain` candidates back to processing and immediately retries them; `POST /api/runs/{id}/memory/candidates/{cid}/discard` marks uncaptured/unpromoted candidates as discarded. The Memory tab now shows approve/discard actions on reviewable candidates. Regression test: `chat::tests::held_memory_candidate_can_be_approved_or_discarded` — green.
- **Memory chain readiness signal** — **SHIPPED 2026-04-28**. `GET /api/runs/{id}/memory/candidates` now returns `memory_chain_status` (`clear`, `processing`, `needs_review`, `retry_pending`, `waiting_openbrain`, `calvin_review`) plus `memory_chain_blockers`; the Memory tab renders a readiness banner so operators can see at a glance whether the PackChat → OB1 → Calvin chain is clear, waiting on service configuration, or waiting on review.
- **OB1 reader dedupe in briefings** — **SHIPPED 2026-04-28**. Coobie briefing assembly now deduplicates memory hits by normalized content before adding source labels, so the same fact from repo memory/core memory/OB1 is not repeated just because it arrived through multiple recall paths. Provenance suffixes such as PackChat source refs are stripped for the dedupe key while preserved in the displayed hit. Regression test: `orchestrator::tests::normalize_briefing_hit_key_dedupes_source_labels_and_provenance` — green.
- **Twilight ingest loop write-back to Calvin** — **SHIPPED 2026-04-28**. `run_twilight_ingest_once()` in `chat.rs` now checks each inbound wire envelope for `archive_contract.schema == "harkonnen.calvin.ingress.v1"` and calls `calvin_client.record_experience()` with the mapped `ArchiveExperience`. Non-null `causation_id` values are forwarded as `causally_contributed_to` links via `POST /runs/{id}/causal-links` (Pearl level: `Associational`, carried in the relation scope until Phase 7 adds a dedicated Pearl-level attribute). Calvin is now initialised before the ingest loop spawns so the client is available from the first poll. New endpoints added: `POST /runs/{id}/causal-links` on Calvin, `record_causal_link()` on `CalvinClient`. E2E tests: `twilight::twilight_ingest_loop_writes_to_calvin`, `memory_candidates::causation_id_written_to_calvin_causal_graph` — both green.
- **Complete chamber mapping for all six chambers** — **SHIPPED 2026-04-28**. Added `BeliefRevised` and `DriftDetected` variants to `PackChatBusEventKind`. `calvin_chamber_for_packchat_event()` now maps by event kind first: `ThreadOpened → mythos`, `ThreadRosterSynced → ethos`, `BeliefRevised → episteme`, `DriftDetected → pathos`, `CheckpointResolved → praxis`, `MessageAppended → logos` (role-refined). All six chambers are now reachable. E2E test: `twilight::chamber_mapping_covers_all_six_chambers` — green.
- **Agent presence TTL watcher → Calvin agent status** — **SHIPPED 2026-04-28**. `spawn_twilight_ingest_loop()` now maintains a `HashMap<agent_id, Instant>` presence tracker that persists across reconnect cycles. On each loop iteration, agents not seen for > 600 s are marked `"offline"` via `PATCH /agents/{name}/status`. Activity events reset the last-seen timestamp. New endpoint added: `PATCH /agents/{name}/status` on Calvin, `update_agent_status()` on `CalvinClient` and `ArchiveStore`. E2E test: `twilight::agent_presence_expiry_updates_calvin_agent_status` — green.
- **Twilight Bark dependency-direction guard** — **SHIPPED 2026-04-28**. `src/chat.rs` now names the Harkonnen-owned PackChat topic root and Twilight operation label explicitly, and the publish path builds a generic Twilight `publish_task` command carrying an opaque `harkonnen.packchat.event` payload. Regression test: `chat::tests::twilight_bridge_uses_harkonnen_owned_operation_over_generic_task_ipc` — green. This protects the boundary that Harkonnen depends on Twilight Bark as transport while Twilight Bark remains Harkonnen-agnostic.
- **Twilight identity continuity ingest** — **SHIPPED 2026-05-05**. `run_twilight_ingest_once()` now recognizes Twilight Bark's generic `identity_continuity_event` operation and captures it as an idempotent `memory_candidates` row with `retention_class = calvin_candidate` and `learning_intent = prior_revision_target`. The candidate stores the raw Twilight payload, evidence refs, causality hints, and a Harkonnen-owned Calvin continuity contract so model/provider swaps become governed promotion proposals rather than direct archive mutation. Regression test: `chat::tests::twilight_identity_continuity_event_creates_calvin_candidate` — green.
- **Twilight identity continuity bridge smoke** — **SHIPPED 2026-05-10**. A mock Twilight daemon now drives the real ingest path: Harkonnen registers, subscribes, receives `identity_continuity_event`, stages the Calvin continuity candidate, and replies `reply_task` with success. Regression test: `chat::tests::twilight_ingest_smoke_preserves_identity_continuity_contract` — green.
- **OB1 capture-to-briefing round trip** — **SHIPPED 2026-04-29**. The Phase 5-D product gate now has a real in-process smoke test for the Harkonnen side of the chain: a `shared_recall` memory candidate is processed by `process_memory_candidates()`, captured through `OpenBrainClient::capture_thought`, marked `captured_openbrain`, and then retrieved through Coobie's `collect_memory_hits()` briefing path via `OpenBrainClient::search_thoughts`. Regression test: `orchestrator::tests::memory_candidate_capture_is_retrievable_from_openbrain_briefing_path` — green.
- **Calvin governed promotion round trip** — **SHIPPED 2026-04-29**. A `calvin_candidate` memory candidate now has an executable smoke test proving it becomes a `harkonnen.calvin.promotion.v1` consolidation candidate with `promotion_pending` status and a preservation note that Calvin canonical state must not be mutated until governance accepts, modifies, rejects, or quarantines it. Regression test: `orchestrator::tests::calvin_candidate_becomes_governed_promotion_without_archive_mutation` — green.
- **Sensitivity gate before OB1** — **SHIPPED 2026-04-29**. Sensitive shared-recall candidates are held as `held_for_review` and are not sent to OB1 until an operator approves them. Regression test: `orchestrator::tests::sensitive_shared_recall_is_held_and_not_sent_to_openbrain` — green.
- **Five-message PackChat candidate smoke** — **SHIPPED 2026-04-29**. A realistic five-message run-scoped PackChat thread now has a regression test proving it creates memory candidates, including both shared-recall and Calvin-candidate classifications. Regression test: `chat::tests::five_message_packchat_thread_produces_memory_candidates` — green.
- **Twilight bridge smoke** — **SHIPPED 2026-04-29**. A mock Twilight daemon now receives real `TwilightPackChatBus` publishes from `ChatStore`; the local store records the memory candidate while the outbound `harkonnen.packchat.v1` envelope carries the Calvin ingress contract. Regression test: `chat::tests::twilight_publish_smoke_preserves_memory_candidate_and_calvin_contract` — green.
- **Operator Calvin proposal visibility** — **SHIPPED 2026-04-29**. The Memory tab renders a Calvin proposal preview for `harkonnen.calvin.promotion.v1` contracts, including governance outcome, chamber targets, and preservation note, so `promotion_pending` is inspectable rather than only counted.

### OpenZiti service profile

Define the distributed trust model before broad deployment:

| Service | Dial | Bind | Access posture |
| --- | --- | --- | --- |
| `twilight-bark.packchat` | approved agent runtimes, Pack Board | Twilight daemon nodes | Pack conversation and event bus |
| `openbrain.mcp` | Harkonnen distiller, approved recall clients | OB1 server | Shared recall; write permission narrower than read |
| `calvin.archive` | Coobie/Harkonnen archive writer, operator console | Calvin host | Governed archive write path |
| `harkonnen.api` | operator console, approved integrations | Harkonnen host | Run control and review UI |

OpenZiti Dial and Bind policies should be separate. Privileged writers should carry posture checks where available: enrolled identity, expected OS, MFA for operators, and known process checks for daemons. Remote agents can read OB1 recall through policy, but only the Harkonnen distiller writes Calvin promotion contracts by default.

### Benchmark / product gate

- **E2E integration test suite green — DONE 2026-04-28:** all 20 tests in `tests/e2e_integration.rs` pass. The six gap tests that were failing at roadmap entry are now green: `calvin::beliefs_scoped_to_named_agent`, `calvin::adaptation_safety_catches_semantic_negation`, `twilight::twilight_ingest_loop_writes_to_calvin`, `twilight::chamber_mapping_covers_all_six_chambers`, `twilight::agent_presence_expiry_updates_calvin_agent_status`, `memory_candidates::run_close_triggers_candidate_processing`.
- A PackChat thread with at least five messages produces one or more memory candidates. **DONE 2026-04-29:** covered by `chat::tests::five_message_packchat_thread_produces_memory_candidates`.
- A candidate marked `shared_recall` is captured in OB1 and later retrieved by `search_thoughts` during a targeted briefing. **DONE 2026-04-29:** covered by `orchestrator::tests::memory_candidate_capture_is_retrievable_from_openbrain_briefing_path`.
- A candidate marked `calvin_candidate` produces a structured promotion contract without directly mutating Calvin canonical state. **DONE 2026-04-29:** covered by `orchestrator::tests::calvin_candidate_becomes_governed_promotion_without_archive_mutation`.
- Calvin promotion contracts carry a compiled claim and append-only evidence timeline, with source authority and staleness triggers available to the operator review surface. **DONE 2026-04-29:** covered by `orchestrator::tests::calvin_candidate_becomes_governed_promotion_without_archive_mutation`.
- Stale or superseded distilled memories are surfaced as `needs_reconsolidation`, with enough evidence context to decide whether to refresh OB1, promote to Calvin, quarantine, or discard. **DONE 2026-04-29:** covered by `orchestrator::tests::newer_packchat_evidence_marks_prior_memory_needs_reconsolidation`.
- A memory chain health endpoint and UI panel can answer whether PackChat -> OB1 -> Calvin is clear, blocked, stale, duplicated, or waiting on OpenZiti/service configuration. **DONE 2026-04-29:** `GET /api/runs/{id}/memory/candidates` returns `memory_chain_health`, and the Memory tab renders the report.
- Memory candidates and Calvin promotion contracts classify evidence by source authority, and the health report summarizes authority distribution. **DONE 2026-04-29:** covered by `chat::tests::source_authority_taxonomy_prefers_stronger_evidence`.
- Candidate dedupe prevents repeated chat phrasing from producing duplicate OB1 thoughts. **DONE 2026-05-01:** covered by `orchestrator::tests::duplicate_shared_recall_candidates_do_not_create_second_openbrain_thought`.
- Sensitivity labels prevent secrets and high-risk payloads from being sent to OB1 without review. **DONE 2026-04-29:** covered by `orchestrator::tests::sensitive_shared_recall_is_held_and_not_sent_to_openbrain`.
- Closing a run automatically triggers candidate processing; transient failures are marked `retry_pending` and retried by the same processing endpoint until zero remain after a clean run close. **DONE 2026-04-28:** covered by `memory_candidates::run_close_triggers_candidate_processing` and `chat::tests::pending_memory_candidate_scan_includes_retry_pending`.
- A `causation_id`-bearing wire envelope produces a `causally_contributed_to` link in the Calvin causal graph. **DONE 2026-04-28:** covered by `memory_candidates::causation_id_written_to_calvin_causal_graph`.
- OpenZiti policy documentation exists for all four services, including Dial/Bind identity roles. **DONE 2026-04-29:** `factory/context/openziti-memory-chain.yaml` now includes identity roles, service profiles, suggested local service configs, service-policy templates, and deployment checks.

**Done when:** a live PackChat/Twilight conversation can become a distilled OB1 memory with provenance, the memory can improve a later briefing, identity-relevant material is routed to Calvin as a governed promotion proposal rather than as unstructured prose, and all six E2E integration gap tests pass.

### GBrain/GStack-inspired hardening queue

These patterns are useful, but Harkonnen implements them inside its own contracts and governance model rather than importing gbrain/gstack wholesale.

- **Memory consolidation:** Use gbrain's compiled-truth idea as `compiled_claim + evidence_timeline` inside Calvin promotion contracts. The compiled claim is readable and reviewable; the timeline stays append-only.
- **Staleness and reconsolidation:** `needs_reconsolidation` is now a first-class candidate status for PackChat memories whose newer evidence explicitly revises or supersedes an earlier candidate.
- **Health visibility:** `memory_chain_status` is now backed by a broader `memory_chain_health` report covering service readiness, backlog, stale claims, duplicate captures, missing evidence, and Calvin review load.
- **Source authority:** memory candidates, Calvin promotion contracts, and health reports now share the same source-authority taxonomy.
- **Code-review learning records:** Mason validation-repair attempts reviewed by Bramble now persist structured `code_review_learning_records` with finding fingerprint, files, severity, resolution, lesson, evidence refs, and stale-if-file-changed invalidation rules. `GET /api/runs/{id}/code-review-learning` exposes the run records. Regression test: `orchestrator::tests::validation_repair_attempts_become_code_review_learning_records` — green.
- **Plan completion audit:** Before run close, Harkonnen now writes `plan_completion_audit.{json,md}` comparing spec acceptance criteria, outputs, required scenario artifacts, validation evidence, and hidden-scenario evidence against actual run artifacts. Missing or partial evidence becomes reviewable audit items instead of quiet success. Regression tests: `orchestrator::tests::plan_completion_audit_flags_missing_evidence_before_close`, `orchestrator::tests::plan_completion_audit_marks_validation_and_hidden_evidence_fulfilled` — green.
- **Operator review surface:** The Run Detail drawer now has a Review tab that combines `plan_completion_audit.json` with `code_review_learning_records`, so operators can see unresolved completion evidence and reusable review lessons without opening artifacts manually. `GET /api/runs/{id}/plan-completion-audit` returns the persisted audit artifact when present.

---

## Phase 5b — Memory Infrastructure, MCP Prompts + Rust-Native Servers

**Unlocks:** Three things that must land before TypeDB (Phase 6) is viable: a clean memory module structure, a live MCP prompt surface, and an OB1-backed memory abstraction that keeps shared recall available across Claude, ChatGPT, Codex, Cursor, Twilight-connected agents, and Harkonnen itself.

This phase also eliminates Harkonnen's remaining local `npx` MCP helper processes in favour of a single compiled Rust binary where Harkonnen owns the tool surface. OB1 itself may still be reached through its MCP endpoint or a `supergateway` bridge; that is an external service boundary, not a Harkonnen hot-path runtime dependency.

**What to build:**

### Memory module refactor

Split the growing `src/memory.rs` into the module tree described in COOBIE_SPEC:

```text
src/memory/
  mod.rs          # re-exports; MemoryStore trait
  working.rs      # short-term blackboard (SQLite-backed)
  episodic.rs     # run episodes, briefing_scope field
  semantic.rs     # SemanticMemory trait; OB1 default impl, local vector fallback optional
  semantic_openbrain.rs # Open Brain (OB1) MCP-backed shared recall implementation
  semantic_local.rs     # optional fastembed/OpenAI-compatible local vector fallback
  causal.rs       # causal links, failure patterns
  consolidation.rs
  blackboard.rs
  retrieval.rs    # build_targeted_briefing() migrates here from src/coobie.rs
  extraction.rs
  briefing.rs     # BriefingScope enum + ContextTarget struct (migrated from Phase 5-C)
  context_budget.rs  # phase_defaults(), ContextSection, token counting utilities
```

No behaviour change beyond preserving OB1 as the default semantic recall path. This is the maintainability gate that lets TypeDB's typed causal query implementation slot in cleanly in Phase 6.

**Early split slice shipped 2026-04-29:** briefing ownership moved under the memory namespace without broad call-site churn: `src/memory/briefing.rs` now owns `BriefingScope`, `ContextSection`, and `ContextTarget`, while `models.rs` re-exports them for compatibility. `src/memory/context_budget.rs` owns shared token-budget helpers for prompt/context tests.

### Open Brain (OB1) semantic integration

Add `src/memory/semantic_openbrain.rs` implementing the `SemanticMemory` trait against Open Brain via the existing `OpenBrainClient`. Payload metadata fields: `org`, `role`, `product`, `spec_id`, `run_id`, `thread_id`, `source_event_id`, `agent`, `memory_type`, `tags`, `sensitivity_label`, `created_at`. OB1 replaces the local vector store as the default long-term semantic memory. SQLite remains the short-term, episodic, causal, and review store. The fastembed/SQLite vector path remains available as `semantic_local.rs` behind `--features local-embeddings`, and Qdrant becomes an optional accelerator only if a future deployment needs local high-volume vector serving.

**First slice shipped 2026-04-29:** `src/memory.rs` moved to `src/memory/mod.rs` as the start of the module split, with `src/memory/semantic.rs` defining `SemanticMemory` / metadata / hit / write contracts and `src/memory/semantic_openbrain.rs` implementing the trait over OB1. `AppContext` now exposes `semantic_memory` as the default long-term recall abstraction; PackChat shared-recall capture, Coobie briefing recall, and MCP memory tools route through the trait when OB1 is configured.

**Fallback slice shipped 2026-04-29:** `NoopSemanticMemory` is now the explicit disabled-OB1 implementation, so `AppContext.semantic_memory` is always present while OB1 remains the default configured backend.

### MCP prompts — live dynamic briefings from `mcp_server.rs` — **SHIPPED 2026-04-28**

`get_prompt` is now async and state-aware. Four live-hydrated prompts added to `src/mcp_server.rs`:

| Prompt | Arguments | What it returns |
| --- | --- | --- |
| `coobie/briefing` | `keywords`, `phase`, `run_id?`, `max_tokens?` | Real memory hits (file store + OB1) + top prior causes from SQLite, budget-capped |
| `sable/eval-setup` | `run_id` | Run artifacts + OB1 scenario patterns; firewall drops implementation_notes, mason_plan, edit_rationale, fix_patterns |
| `scout/preflight` | `spec_id?`, `run_id?` | Spec-scoped memory + OB1 recall + operator model commissioning brief patterns |
| `keeper/policy-check` | `action`, `context?` | Policy-scoped memory + OB1 recall for the proposed action |

All prompts: file-backed memory search → OB1 `search_thoughts` → deduplication → scope isolation → token budget (≈4 chars/token). Registered in `prompt_descriptors()` with argument schemas. The two static templates (`briefing_for_spec`, `diagnose_run`) are preserved for backwards compatibility.

### `memory_pull` — on-demand context retrieval mid-task — **SHIPPED 2026-04-28**

New MCP tool in `src/mcp_server.rs`:

```text
tool:    memory_pull
args:    query (string, required), scope (string, default "general"), max_tokens (integer, default 500)
returns: top-ranked memory hits relevant to query, scoped and budget-capped
```

Searches file-backed memory → OB1 → deduplicates → filters by scope (sable scope drops mason content; mason scope drops scenario content) → truncates to `max_tokens` budget. Each call is logged via `tracing::info!` with query, scope, hits_returned, and token count for context utilization analysis. Registered in `tool_descriptors()` with full JSON Schema.

Pull records are logged (not yet persisted to SQLite episode store — episode record extension remains in the context utilization tracking item below).

**Persistence slice shipped 2026-04-29:** `memory_pull` now accepts optional `run_id`; when supplied, Harkonnen persists a `context_pull_records` row with query, scope, budget, returned tokens, hit count, and hit previews. `GET /api/runs/{id}/context-utilization` combines phase attribution briefing counts with mid-task pull records, and the Review tab renders the context-utilization summary.

### Context utilization tracking

The episode record gains a `ContextUtilization` section:

```rust
pub struct ContextUtilization {
    pub briefing_hits_provided: u32,
    pub briefing_tokens: u32,
    pub mid_task_pulls: Vec<PullRecord>,
    pub utilization_rate: f32,  // fraction of briefing hits referenced in agent output
}
```

`utilization_rate` is computed post-run by Coobie: scan the agent's output for references to the content of each briefing hit (embedding similarity above a threshold). A briefing with `utilization_rate < 0.2` over multiple runs for the same scope is a signal that the budget is too high or the category filter is too loose. This data feeds the Phase 7 causal corpus and the Phase 8 Episteme chamber's slow-loop policy revision for scope configuration.

**First scoring slice shipped 2026-04-29:** `GET /api/runs/{id}/context-utilization` now returns a first-pass utilization rate and status from briefing attribution records and run-scoped `memory_pull` queries/previews; the Review tab renders the percentage and low-utilization warning state. This is lexical and conservative until the embedding-based post-run scorer lands.

### Learning-loop closure and prior-revision tracking

Memory accumulation and genuine prior revision are not the same thing. A lesson that is retrieved in every briefing but never changes a decision has been stored, not learned. Coobie must track this divergence explicitly.

**What to build:**

- **Decision-change linkage on memory hits** — after each run, Coobie compares which briefing hits were present against which decisions diverged from the agent's prior-run behavior on comparable spec types. A hit that is retrieved repeatedly but correlates with zero behavioral change is flagged as a candidate for consolidation review. Track `decision_influence_score` per memory entry, updated post-run.
- **"Awareness only" vs "prior-revision target" consolidation status** — extend the memory candidate data model with an explicit `learning_intent` field: `awareness_only` (operator wants the agent to know this) or `prior_revision_target` (operator wants this to change how the agent processes a class of situation). The Consolidation Workbench must surface this distinction; the default must be `awareness_only` so prior revision is always a deliberate operator decision, not an assumption.
- **Schema revision as a distinct candidate type** — the candidate data model currently treats fact ingestion, belief revision, and schema revision as similar candidate types. Schema revision (changing how a whole class of situations is categorized) must be a structurally distinct type with elevated review requirements: it requires medium-loop compressed cross-episode evidence (not a single run's lesson), a `pattern_basis` field citing at least three corroborating episodes, and operator endorsement before it reaches Ethos. A single-run lesson cannot qualify as `schema_revision`.
- **Coobie behavioral-change report** — a post-run artifact comparing current Praxis-layer decisions (spec clarification thresholds, escalation rates, ambiguity checkpoint frequency) against the rolling prior-N-run baseline. When a decision metric shifts significantly after a lesson was promoted to `prior_revision_target`, record the link as a learning provenance record. When it does not shift after 5+ runs, flag the lesson as `stored_not_learned`.
- **`decision_influence_score` calibration A/B design** — `decision_influence_score` per memory entry cannot be computed correctly by counting retrieval frequency. A hit that is always in the briefing looks influential whether or not it changes anything. Add a calibrated exclusion signal: with configurable probability (default 0.05), randomly exclude an eligible briefing hit from the active set for a run on the same spec family. Track whether outcomes differ from the expected rate. Over N exclusion events per hit, the statistical difference in outcome rate becomes the influence score. Without this, `decision_influence_score` reduces to a retrieval-frequency proxy and `stored_not_learned` detection will be systematically wrong on high-frequency, low-influence lessons.
- **Positive signal reinforcement path** — the prediction system records error on failures but has no reinforcement signal on correct predictions. Over many runs this produces a systematic accumulation bias: memory fills with failure-annotated lessons while correct-prediction runs generate nothing, progressively skewing briefings toward failure-framing even on a healthy factory. Add a `prediction_success_reinforcement` step at run close: when `prediction_error < success_threshold` (configurable, default 0.2), increment a `confirmation_count` on the causal signals that fired during preflight and whose chamber classifications matched the actual outcome. When `confirmation_count` reaches a configurable threshold, the signal's base confidence is eligible for upward revision in the Consolidation Workbench.
- **Mid-run re-briefing protocol** — Coobie briefs before each phase. If Mason or Bramble encounters something unexpected mid-task (an undocumented dependency, a schema surface not in the spec, an API contract that contradicts the intent package), the phase-entry briefing is stale. Define a `mid_run_rebrief_trigger` contract: agents may issue a scoped `memory_pull` tagged `trigger: unexpected_discovery` when they encounter a condition absent from the briefing. At run close, flagged pulls feed into the briefing-scope review for the spec family: if the same discovery recurs across multiple runs of a family, it becomes a `required_section` injection rule for that scope rather than requiring an agent to ask each time.
- **Computable BeliefRevision struct** — `BeliefRevision` in `calvin_client.rs` must carry `prior_confidence: f64`, `evidence_ids: Vec<String>`, and `revision_type: RevisionType` (enum: `FastLoop | MediumLoop | SchemaRevision | PolicyChange`) alongside the existing `revised_summary` and `new_confidence`. Without `prior_confidence`, calibration change is unmeasurable — a revision from 0.4 to 0.85 confidence is not the same event as a revision from 0.84 to 0.85, but both are invisible without the prior. Without `evidence_ids` linking to specific `ArchiveExperience` records, revision is unverifiable: you cannot distinguish evidence-driven Bayesian updating from arbitrary drift, making ghost learning structurally undetectable. `revision_type` determines the governance path: `FastLoop` entries may be self-submitted; `MediumLoop` requires evidence_ids.len() ≥ `medium_loop_trigger_runs`; `SchemaRevision` requires operator endorsement before it touches Ethos. These are not optional enrichment fields — the AGM consistency gate (P8-P11) cannot function without `prior_confidence`, and the behavioral-change report cannot confirm that a `prior_revision_target` actually moved a prior without them.

**First slice shipped 2026-04-30:** memory candidates now carry `learning_intent` with the conservative default `awareness_only`; PackChat capture classifies explicit behavioral directives such as "always use", "from now on", "default to", and boundary rules as `prior_revision_target` while scratch/working memories remain awareness-only. `GET /api/runs/{id}/memory/candidates` returns the field, and the Memory tab surfaces it as an Awareness/Prior revision chip so operator review can distinguish knowledge capture from intended prior revision before the behavioral-change scorer lands.

**Behavioral-change report MVP shipped 2026-04-30:** run close now writes `behavioral_change_report.{json,md}`, persists the report in `behavioral_change_reports`, and exposes it through `GET /api/runs/{id}/behavioral-change`. The report records prior-revision candidates, checkpoint/clarification counts, validation repair attempts, plan-audit unresolved count, validation status, and hidden-scenario status. It deliberately uses conservative states (`no_prior_revision`, `possible_shift`, `stored_not_learned_pending`) so Harkonnen can surface learning provenance without overstating causality before the calibrated `decision_influence_score` work lands. The Review tab now renders the report beside plan audit, context utilization, and code-review learning.

**Schema-revision candidate contract shipped 2026-04-30:** `consolidation_candidates` now carries `review_class` and `pattern_basis_json`; `ConsolidationCandidate` exposes `review_class` and `pattern_basis`; and Coobie can emit `schema_revision` proposals only when a `prior_revision_target` memory has corroborating `pattern_basis` from at least three distinct runs. `schema_revision` candidates are elevated Workbench items, require `operator_endorsement_required=true`, and cannot be kept through the API if the basis collapses to a single run. The Workbench labels schema revisions separately and surfaces the elevated-review/basis count before an operator keeps or edits the proposal.

**Computable BeliefRevision contract shipped 2026-04-30:** Harkonnen's `BeliefRevision` client payload and the Calvin sidecar archive/API payload now carry `prior_confidence`, `evidence_ids`, and `revision_type` (`fast_loop`, `medium_loop`, `schema_revision`, `policy_change`) alongside `new_confidence`. Both client and sidecar validate finite confidence values in `0.0..=1.0`; all revisions require evidence IDs; medium-loop and schema-revision updates require at least three evidence IDs. The Phase 6 TypeDB schema now lets `revised_into` relations persist `prior-confidence`, `evidence-ids`, and `revision-type`, so belief updates are measurable calibration events rather than prose annotations.

**Prediction success reinforcement shipped 2026-05-01:** Harkonnen now persists local run-prediction shadows and writes `prediction_success_reinforcements` for low-error run closes (`prediction_error <= 0.2`). Each contributing `source_cause_id` gets a reviewable reinforcement event with cumulative `confirmation_count`, surfaced through `GET /api/runs/{id}/prediction-reinforcements`, so successful predictions strengthen the signals that fired instead of only failure cases feeding memory. Regression test: `orchestrator::tests::low_error_prediction_records_success_reinforcement_counts` — green.

**Decision-influence calibration exclusion ledger shipped 2026-05-01:** run close now samples eligible briefing hits with deterministic hash selection (default 0.05) and records `memory_influence_exclusions` with run, phase, briefing scope, memory key, preview, spec-family placeholder, expected outcome, actual outcome, exclusion probability, and selection basis. `GET /api/runs/{id}/memory-influence-exclusions` exposes the run ledger. This is the first calibration event stream needed to keep `decision_influence_score` distinct from retrieval frequency. Regression test: `orchestrator::tests::memory_influence_exclusion_event_is_recorded_from_briefing_hits` — green.

**Mid-run re-briefing protocol shipped 2026-05-01:** `memory_pull` now accepts optional `trigger`, with `trigger: "unexpected_discovery"` as the first-class signal that a phase-entry briefing went stale. Triggered pulls persist the trigger in `context_pull_records`, appear in `GET /api/runs/{id}/context-utilization` summary counts, and contribute a run-health review item so recurring unexpected discoveries can later become required briefing sections for the spec family. Regression test: `mcp_server::tests::memory_pull_unexpected_discovery_persists_rebrief_trigger` — green.

**Benchmark gate:**

- At least one lesson promoted as `prior_revision_target` has a linked behavioral-change record showing the Praxis metric it shifted and the run where the shift first appeared.
- Coobie's behavioral-change report is produced at run close and included in the run artifact list.
- At least one `decision_influence_score` calibration exclusion event has been recorded and the resulting score is distinguishable from retrieval frequency alone. **DONE 2026-05-01:** covered by `orchestrator::tests::memory_influence_exclusion_event_is_recorded_from_briefing_hits`.
- At least one `prediction_success_reinforcement` event is recorded after a low-error run and the contributing signal's `confirmation_count` increments. **DONE 2026-05-01:** covered by `orchestrator::tests::low_error_prediction_records_success_reinforcement_counts`.

---

### Spec Taxonomy and Cross-Agent Lesson Promotion

Two structurally distinct gaps in how lessons accumulate across runs.

**Spec family clustering** — every spec is currently an independent retrieval entity. Coobie queries by keywords against a flat collection; there is no mechanism to recognize that a new spec is structurally similar to previous auth-module specs and weight those episodes above a random pattern-match. "Thin evidence" fires even when the factory has done the same class of work many times, because the retrieval layer cannot see the family. The behavioral-change report also cannot scope to "on this spec family" — it spans all specs, diluting causal signal.

What to build: embed each spec into a family vector at intake (using the OB1/semantic-memory infrastructure already built in Phase 5-D). Before retrieval, compute the incoming spec's similarity to prior family vectors and bias retrieval toward the closest family. The family vector lives alongside the spec in OB1; Scout's intent package gains an optional `spec_family` tag. This enables the behavioral-change report to filter: "on specs tagged `auth_service`, Coobie's escalation rate shifted from 0.4 to 0.2 after this lesson." That specificity is what makes learning provenance claims verifiable rather than cross-spec noise.

**Spec-family clustering MVP shipped 2026-05-05:** Harkonnen now derives a deterministic spec-family profile at run intake, persists it in `spec_family_profiles`, mirrors the profile through the semantic-memory abstraction as `spec_family_profile`, adds `spec_family` to Scout intent packages and Coobie briefings, and injects the family ID/fingerprint into Coobie query terms so retrieval can bias toward related prior specs. Memory influence exclusions now resolve `spec_family` from the profile ledger instead of using raw `spec_id`, which lets calibration and behavioral-change records scope learning provenance to a family. Regression tests: `orchestrator::tests::spec_family_profile_groups_related_auth_specs_and_biases_query_terms` and `orchestrator::tests::memory_influence_exclusion_event_is_recorded_from_briefing_hits` — green.

**Cross-agent lesson promotion path** — Scout's ambiguity detection patterns and Mason's failure repair patterns are stored in separate scoped memory. The isolation is correct. But there is a missing signal: when Scout repeatedly flags `SCOPE_CREEP` and Mason repeatedly fails on the same run, there is a cross-agent causal relationship that no single agent is authorized to generate. Coobie produces run-level causal attributions, but her six DeepSignalSpec heuristics are Mason/Bramble-facing; the Scout→Mason linkage is invisible.

What to build: after each run, Coobie checks whether any Scout ambiguity signals co-occurred with Mason failure causes at statistically significant frequency over N runs. When co-occurrence crosses `cross_agent_correlation_threshold`, emit a `cross_agent_pattern_candidate` for operator review in the Consolidation Workbench, tagged `cross_agent_pattern` (elevated review type, spans a role boundary). If promoted, the lesson feeds into both Scout's and Mason's scoped briefings with a `cross_agent` provenance tag visible to both.

**Cross-agent lesson promotion MVP shipped 2026-05-10:** `generate_consolidation_candidates()` now checks same-family phase attribution history for repeated Scout ambiguity signals co-occurring with Mason/Bramble implementation failure. When the MVP threshold is met, Harkonnen emits an elevated `cross_agent_pattern` Workbench candidate with `harkonnen.cross_agent_pattern.v1` content, family ID, role-boundary review state, recommended Scout/Mason briefing action, and a per-run evidence basis. Run close now attempts consolidation candidate generation automatically, so reviewable cross-agent patterns are created without waiting for a manual API call. Regression test: `orchestrator::tests::cross_agent_pattern_candidate_emits_for_repeated_family_cooccurrence` — green.

**Benchmark gate:**

- At least one run demonstrates spec-family-biased retrieval returning demonstrably higher-relevance hits than flat keyword retrieval on the same spec (assessed by utilization rate).
- **DONE 2026-05-10:** `harkonnen.spec_family_retrieval_proof.v1` is attached to Coobie briefings when family context is available, recording the flat top hit, family-biased top hit, family/unrelated hit counts, and whether the rerank improved the top result. Regression test: `orchestrator::tests::spec_family_biased_retrieval_promotes_same_family_hit_over_flat_top_hit` — green.
- At least one `cross_agent_pattern_candidate` is emitted and surfaced in the Consolidation Workbench after 10+ runs of the same spec family.

---

### Code-review learning records and completion audit

Sable, Bramble, and Mason review outcomes should become structured memory rather than scattered prose. Store the finding fingerprint, files, severity, resolution (`fixed`, `skipped`, `auto_fixed`), lesson extracted, evidence refs, and stale-if-file-changed invalidation rules. These records feed OB1 shared recall first, then Calvin only when the lesson is identity-, policy-, or causally significant. **First slice shipped 2026-04-29:** Mason validation-repair attempts reviewed by Bramble persist as `code_review_learning_records` and are exposed through `GET /api/runs/{id}/code-review-learning`.

Before run close, Harkonnen should run a plan completion audit: turn the accepted roadmap/spec acceptance items into a checklist and compare them with the actual diff, tests, and artifacts. Any missing evidence or unimplemented item becomes a reviewable run note, not a quiet success. **First slice shipped 2026-04-29:** every success/failure close path writes `plan_completion_audit.{json,md}` and attaches it to the run artifact list.

**Operator surface shipped 2026-04-29:** `GET /api/runs/{id}/plan-completion-audit` exposes the persisted audit artifact, and the UI Run Detail drawer now has a Review tab that renders audit items, unresolved counts, evidence refs, and code-review learning records together.

**Run health surface shipped 2026-04-29:** `GET /api/runs/{id}/health` now summarizes blockers, validation, hidden scenarios, plan audit, PackChat/OB1/Calvin memory chain state, and context utilization into `ready`, `running`, `needs_review`, or `blocked`. The Run Detail overview renders this as the first operator signal.

### Rust-native MCP server consolidation — **SHIPPED 2026-04-29**

The three `npx @modelcontextprotocol/server-*` processes (filesystem, memory, sqlite) are replaced by the Harkonnen self-server (`cargo run -- mcp serve --transport stdio`). No Node.js runtime needed for the core MCP surface.

**Tools added to `src/mcp_server.rs`** (all boundary-enforced):

- `read_file` / `list_directory` — reads from products, workspaces, artifacts, memory, specs, logs, calvin_archive, the-soul-of-ai (read-only boundary)
- `write_file` / `create_directory` — writes only into factory/workspaces and factory/artifacts
- `memory_store` — writes a timestamped markdown entry to the memory store + OB1 capture
- `memory_retrieve` — hybrid search: file store + OB1 `search_thoughts`, deduplicated, limited
- `memory_list` — lists recent memory store entries with id + summary
- `db_list_tables` — lists SQLite tables in state.db
- `db_query` — SELECT-only queries against state.db (INSERT/UPDATE/DELETE/DROP rejected)

**Config changes:**

- `harkonnen.toml`: removed `[[mcp.servers]]` entries for filesystem, memory, sqlite; updated `[mcp.self]` to `transport = "stdio"`
- `.mcp.json`: replaced three npx server entries + old harkonnen entry with single consolidated entry pointing at current repo path via `flatpak-spawn --host`
- `.claude/settings.local.json`: `enabledMcpjsonServers` reduced to `["harkonnen"]`
- `src/cli.rs` `common_mcp_templates()`: three npx server templates removed; github and brave-search remain (external services)

**Result:** one process, one restart, all tools. `/mcp coobie/briefing`, `memory_pull`, `read_file`, `db_query`, live prompts — all served by the same `cargo run -- mcp serve` invocation that Claude Code auto-starts via stdio. 128/128 tests green.

### `llm.rs` multi-provider extension

Extend `src/llm.rs` with a unified multi-provider completion interface to back the `SubAgentBackend` variants introduced in Phase 5-C without shelling out to external CLIs:

```rust
pub enum ProviderBackend {
    Anthropic { model: String },   // reqwest → messages API
    OpenAi    { model: String, base_url: Option<String> },  // reqwest → chat completions
    Gemini    { model: String },   // reqwest → generateContent
}

pub async fn complete(backend: &ProviderBackend, messages: &[Message]) -> Result<String>
```

Each variant is a typed `reqwest` call to the provider's REST API. `SubAgentBackend::CodexPlanAgent` routes through `ProviderBackend::OpenAi` (model: `o4-mini` or `gpt-4o`) rather than spawning the codex CLI process. `SubAgentBackend::GeminiAgent` routes through `ProviderBackend::Gemini`. The existing `SubAgentBackend::DirectLlm` path continues to use the current `llm.rs` call site unchanged — this is additive, not a rewrite.

**First slice shipped 2026-04-29:** `src/llm.rs` now exposes `ProviderBackend::{Anthropic, OpenAi, Gemini}`, `build_provider_backend()`, `complete()`, and `complete_request()`. `SubAgentDispatcher` routes isolated `ClaudeCodeAgent`, `CodexPlanAgent`, and `GeminiAgent` calls through typed provider backends with model overrides instead of subprocess spawns or ad hoc provider construction. Regression tests cover OpenAI backend mapping and model-override credential preservation.

**OpenAI-compatible local endpoint slice shipped 2026-05-10:** `ProviderBackend::OpenAi` now also accepts `openai_compatible`, `openai-compatible`, `lmstudio`, `lm_studio`, and `lm-studio` provider types. OpenAI-compatible local endpoints can set `api_key_env = ""`, which omits the bearer token instead of blocking provider construction. The LM Studio setup templates now use `type = "lmstudio"` with no required dummy API-key environment variable. Regression tests cover LM Studio backend mapping and auth-free local key resolution.

### Named Briefing Blocks

Coobie briefings are currently assembled as freeform markdown and passed as a single string to each phase prompt. The internal structure is invisible: you cannot tell programmatically which part of a briefing contains causal patterns vs. operator profile vs. open checks, you cannot diff two briefings from consecutive runs of the same spec, and you cannot update one category of context without regenerating the whole briefing. This matters increasingly as the memory system grows — a large briefing with no internal structure is as opaque as a large prompt.

Reformulate Coobie briefings as a `Vec<BriefingBlock>` where each block carries a name, content string, source tag, and token count. The assembled LLM prompt remains a flat string at the boundary, but the intermediate representation is structured.

Proposed standard blocks:

| Block | Content | Source |
| --- | --- | --- |
| `causal_patterns` | Palace den scents + active DeepSignal activations | Causal memory + Palace patrol |
| `operator_profile` | Commissioning brief posture + risk tolerances | Operator model |
| `active_run` | Current spec + phase + blackboard summary | Working memory |
| `open_checks` | Required checks + guardrails per scope | Scope filter output |
| `recalled_lessons` | OB1 hits + file-backed memory hits (ranked, deduplicated) | SemanticMemory + file store |

What this enables beyond the current flat string:

- **Introspection** — `GET /api/runs/{id}/briefing` returns the block structure. The Pack Board can render each block independently and show which blocks are populated vs empty.
- **Diffability** — compare briefings for the same spec across runs. Which blocks changed? Which recalled lessons are new since the last run?
- **Independent updateability** — approving a new lesson in the Consolidation Workbench updates the `recalled_lessons` block without touching `causal_patterns` or `operator_profile`.
- **Per-block utilization tracking** — `decision_influence_score` is attributable per block rather than per hit. A block with consistently zero influence across 10 runs is a scope configuration signal, not just a retrieval signal.

This is a Coobie schema improvement derived from comparing Letta's labeled in-context memory block model against Harkonnen's current freeform briefing path. It is not a Letta integration.

**First slice shipped 2026-05-10:** `CoobieBriefing` now serializes a `briefing_blocks: Vec<BriefingBlock>` alongside the existing flat fields. The five standard blocks (`causal_patterns`, `operator_profile`, `active_run`, `open_checks`, `recalled_lessons`) carry `name`, `source`, `token_count`, and `content`. Full Coobie briefings and role-scoped briefings both regenerate blocks after scope filtering, so Scout/Sable do not see redacted Mason implementation context through the structured block surface. Regression test: `orchestrator::tests::named_briefing_blocks_group_memory_checks_and_scope_filters` — green.

**Reader surface shipped 2026-05-10:** `GET /api/runs/{id}/briefing` now returns a block-oriented briefing response with `scope`, `artifact`, `block_count`, `total_block_tokens`, `blocks`, and the full `CoobieBriefing`. The endpoint supports `?scope=coobie_preflight|scout_preflight|mason_preflight|sable_preflight` and maps to the corresponding scoped briefing artifact. The Run Detail drawer now includes a Briefing tab that renders the named blocks, sources, and token counts without opening JSON artifacts manually. Regression test: `api::tests::briefing_scope_artifact_maps_named_scopes_to_artifacts` — green; UI production build green.

---

### Agent State as First-Class Database Entity

The dog runtime registry (shipped v1-A) tracks live agent instances per role with `thread_id`, ownership, and status. This is coordination state. It does not record what each agent currently knows, what LLM it is using, or what happened last time it ran.

Add an `agent_state` table to `state.db` with one canonical row per Labrador role:

```sql
CREATE TABLE agent_state (
    agent_name             TEXT PRIMARY KEY,
    agent_role             TEXT NOT NULL,
    llm_provider           TEXT NOT NULL,
    llm_model              TEXT NOT NULL,
    memory_block_ids       TEXT NOT NULL DEFAULT '{}',   -- JSON: block_name → OB1 ref / memory file id
    last_stop_reason       TEXT,                         -- last known stop reason for this agent
    last_active_run        TEXT,                         -- run_id of most recent run this agent participated in
    behavior_contract_hash TEXT,                         -- SHA256 of current factory/agents/contracts/ file
    updated_at             TEXT NOT NULL
);
```

`memory_block_ids` maps each standard briefing block name to the OB1 thought ref or file memory ID that was most recently assigned to it. When Coobie assembles a briefing for Mason, she queries `agent_state` for Mason's existing block refs before running a full semantic search — giving each agent a fast path to its prior context without a cold retrieval every time.

`last_stop_reason` is low-cost diagnostic data that pays off quickly. Debugging why Bramble behaved differently in run X vs run Y starts with knowing that Bramble stopped with `timeout` in run X and `all_tests_passed` in run Y. Recording it costs nothing.

`behavior_contract_hash` fingerprints each agent's current `factory/agents/contracts/` file. If the file changes between runs, the hash change is visible as a database event — making behavioral contract revision auditable without requiring a git diff.

This pattern is derived from Letta's `AgentState` model (which records LLM config, tool bindings, memory blocks, and `last_stop_reason` per agent). The implementation is Harkonnen-native; there is no Letta dependency.

**First slice shipped 2026-05-12:** Harkonnen now creates a canonical `agent_state` table keyed by `agent_name`, seeds one row per agent profile at DB init with resolved provider/model and SHA-256 behavior-contract hash, and updates `last_active_run` from live `agent_runtime_state` sync without replacing the per-runtime roster. Backend helpers expose list/get state and stop-reason updates. Regression test: `db::tests::init_db_seeds_canonical_agent_state_and_runtime_sync_updates_last_run` — green.

**Briefing block assignment slice shipped 2026-05-12:** Coobie now records each role's latest named briefing block anchors in `agent_state.memory_block_ids` after writing the scoped briefing artifacts. `coobie`, `scout`, `mason`, and `sable` rows receive block refs such as `runs/{run_id}/mason_briefing.json#block=recalled_lessons`, giving the next retrieval slice a durable fast path to the last assigned memory blocks. Regression tests: `db::tests::update_agent_memory_block_ids_merges_existing_blocks` and `orchestrator::tests::briefing_block_refs_point_to_run_artifact_blocks` — green.

**Agent state reader surface shipped 2026-05-12:** `GET /api/agent-state` lists canonical role state and `GET /api/agent-state/{name}` returns a single role row, including resolved provider/model, last active run, behavior-contract hash, stop reason, and memory block refs. This makes the Pack Board and future diagnostics able to inspect canonical agent state without querying SQLite directly.

**Agent memory-block fast path slice shipped 2026-05-12:** Coobie now pulls existing `agent_state.memory_block_ids` into the next memory-context bundle and carries them through `CoobieBriefing.agent_memory_block_refs`. Named briefing blocks include those refs in `recalled_lessons`, so scoped Scout/Mason/Sable briefings can see the last assigned block anchors before deeper semantic search work expands the fast path. Regression test: `orchestrator::tests::named_briefing_blocks_group_memory_checks_and_scope_filters` — green.

**Pack Board agent-state surface shipped 2026-05-12:** The Pack Board side rail now renders canonical agent state from `GET /api/agent-state`: resolved provider/model, last active run, stop reason, behavior-contract hash, and latest briefing block refs per role. API route regression tests now cover list/get success and 404 behavior: `api::tests::agent_state_routes_return_canonical_rows` and `api::tests::agent_state_route_returns_404_for_unknown_agent` — green.

**Pre-Phase 6 gate closure shipped 2026-05-22:** The remaining Phase 5b mechanical gates are now explicit in code/config: `build_targeted_briefing()` is the only orchestrator briefing projection helper, the complete COOBIE_SPEC memory module namespace exists under `src/memory/`, Harkonnen-owned filesystem/memory/sqlite MCP helper templates now point at `cargo run -- mcp serve --transport stdio` instead of `npx`, and `GET /api/context-utilization/baseline` reports the 10-run context-utilization baseline sample/completeness state. Regression tests: `orchestrator::tests::build_targeted_briefing_filters_scout_and_sable_context`, `orchestrator::tests::named_briefing_blocks_group_memory_checks_and_scope_filters`, and `api::tests::context_utilization_baseline_reports_sample_completeness` — green.

---

**Benchmark gate:**

- Re-run `FRAMES` after OB1 lands as the default semantic recall path to confirm multi-hop recall improves over the SQLite/local-vector baseline
- `LongMemEval` and `LoCoMo` re-run to confirm semantic recall quality does not regress
- Re-run `StreamingQA` to confirm belief-update accuracy does not regress after the module refactor
- MCP prompt round-trip test: `coobie/briefing` for a known run returns a briefing containing at least one memory hit and zero items tagged with Mason-scoped categories
- Token budget enforcement test: `coobie/briefing` called with `max_tokens=500` returns ≤ 500 tokens of ranked content with required sections present regardless of budget
- `memory_pull` latency: p95 round-trip under 500ms on home-linux against SQLite + OB1, with local-cache hits under 200ms when available
- Context utilization baseline: record `utilization_rate` for 10 runs across Scout, Mason, and Sable scopes; establish the floor before Phase 7 causal corpus work begins

**Done when:**

- `src/memory.rs` is split into the COOBIE_SPEC module tree; `BriefingScope` and `ContextTarget` live in `src/memory/briefing.rs`; `phase_defaults()` and token budget utilities in `src/memory/context_budget.rs`
- `build_targeted_briefing()` is the sole briefing entry point; no call site uses the old `build_preflight_briefing` or `build_scoped_briefing`
- OB1 is serving semantic queries for long-term shared recall through the `SemanticMemory` abstraction
- local fastembed/SQLite vectors remain opt-in and compile-disabled by default
- `mcp_server.rs` serves all four named prompts with `ContextTarget` budget enforcement; `memory_pull` tool is live; `/mcp coobie/briefing` works in a Claude Code session and respects `max_tokens`
- Episode records include `ContextUtilization` with `utilization_rate`; 10-run baseline collected
- Harkonnen-owned local MCP helper entries are replaced by `harkonnen mcp serve` in `harkonnen.toml`; Node.js is no longer required for Harkonnen's local helper surface
- `llm.rs` exposes `ProviderBackend` with Anthropic, OpenAI, and Gemini variants; `SubAgentBackend::CodexPlanAgent` routes through `ProviderBackend::OpenAi` with no subprocess spawn
- `BeliefRevision` struct carries `prior_confidence`, `evidence_ids`, and `revision_type`; all existing revision call sites supply these fields

---

## Calvin Archive Write Integrity (Phase 5b Gate)

**This gate must close before Phase 6 adopts TypeDB as the canonical causal truth layer.** The commitment that "canonical truth lives in the typed graph" is incoherent if the write path to that graph is fire-and-forget. The current `CalvinClient` has a 500 ms timeout and no error propagation on any write method: `record_experience`, `revise_belief`, `record_causal_link`, `record_prediction`, and `record_prediction_result` all call `.send()` and map to `()`. A timeout or dropped connection produces phantom history — the factory believes the experience was archived; the archive has no trace. This is not caught until the Phase 8 heartbeat write-loss detection check, which is reactive. The fix must be proactive.

**What to build:**

- **Durable write queue** — archive writes go to a local SQLite `calvin_write_queue` table before being forwarded to harmony. The caller writes to the queue (infallible); a background processor sends queue entries to harmony with exponential backoff and marks them `confirmed` on a successful response. The `calvin_write_log` entry used by Phase 8 write-loss detection is written only after harmony confirms the entity was persisted. This makes the archive eventually-consistent rather than best-effort lossy.
- **Separated timeouts** — health check timeout remains 500 ms. Write operations use a separate configurable timeout (default 5 s). Read operations (`get_prediction`, `get_kernel_traits`, `get_active_beliefs`) use a third configurable timeout (default 2 s). The three behaviours are independent; a slow write does not affect health polling.
- **Error propagation on write** — write methods return `Result<()>` with the queue-write error if SQLite fails. The harmony send errors are handled by the queue processor, not the caller. The orchestrator's Calvin integration points (`try_close_calvin_run`, `try_open_calvin_run`) already wrap errors with `tracing::warn!` — extend this to all write paths so no silent drop is possible.

**Separated timeout slice shipped 2026-04-30:** `CalvinConfig` now carries independent `health_timeout_ms` (default 500), `read_timeout_ms` (default 2000), and `write_timeout_ms` (default 5000). `CalvinClient` builds separate reqwest clients for health probes, archive reads, and archive writes, so slow archive writes no longer inherit the health-check timeout or interfere with health polling. Regression coverage proves the default profile and custom client construction.

**Durable write queue slice shipped 2026-04-30:** Harkonnen now creates a local `calvin_write_queue` table and all Calvin archive write methods enqueue durable rows when the client is bootstrapped with SQLite. A background processor drains pending/retry rows to harmony, marks successful writes `confirmed`, and applies bounded exponential backoff on failures. `setup check` reports pending, retry-pending, pending-confirmation, and confirmed queue counts so archive health includes write-loss risk, not only harmony reachability.

**Outage enqueue slice shipped 2026-05-01:** Calvin enablement no longer depends on harmony being reachable at bootstrap. When `calvin_archive.enabled = true` and the local SQLite queue initializes, Harkonnen keeps a Calvin client alive, accepts archive writes into `calvin_write_queue`, and lets the background drainer retry until harmony returns. This closes the pre-run outage hole where `app.calvin = None` would otherwise skip archive writes entirely.

**Archive status queue visibility shipped 2026-05-01:** `harkonnen archive status` now reports harmony reachability and local write-queue state together. The status output includes pending, retry-pending, pending-confirmation, confirmed counts, oldest pending write time, next retry time, and last write error, so archive outages are visible as durable backlog rather than only as sidecar reachability failures.

**Pearl-level causal payload enforcement shipped 2026-05-01:** Calvin causal links now carry structural payload fields for Pearl-level warrant: `held_fixed`, `estimated_effect_delta`, `actual_trace_id`, `hypothetical_intervention`, `epistemic_warrant`, and `warrant_gap`. Harkonnen keeps the existing associational convenience call, but structural causal-link payloads are normalized before enqueue/send: interventional claims require do-set fields and effect delta; counterfactual claims additionally require actual trace and hypothetical intervention. Under-supported higher-level claims are recorded at the highest warranted level with `warrant_gap = true` instead of being stored as cosmetic Pearl labels.

**Done when:** a harmony outage during a run produces `retry_pending` entries in `calvin_write_queue`, not silent loss; the queue drains automatically when harmony recovers; `cargo run -- setup check` reports write-queue depth and pending-confirmation count alongside archive health status.

---

## Phase 6 — TypeDB Semantic Layer

**Unlocks:** Typed causal queries that vector similarity cannot answer. "Find all runs where TWIN_GAP caused a failure that was fixed by an intervention that held for ≥ 3 runs" requires a graph, not a similarity score. This is also the direct prerequisite for the Calvin Archive's chamber schema.

TypeDB 3.x changes the implementation assumptions: the old JVM burden objection is gone because TypeDB's core is now Rust. It is still an external service with real operational cost, so it stays later in the sequence and should not replace SQLite as the hot path. When this phase opens, use the Rust-based TypeDB 3.x line in a container-first deployment and avoid the legacy Java server/distribution entirely.

**What to build:**

- TypeDB 3.x instance configured in the home-linux setup TOML
- `src/coobie/semantic.rs` implementing the `SemanticMemory` trait from COOBIE_SPEC
- Rust-facing TypeDB adapter using the official TypeDB 3.x driver behind the `SemanticMemory` abstraction
- TypeDB schema from COOBIE_SPEC: entities (agent, goal, episode, observation, action, outcome, artifact, lesson, failure-mode, causal-link), relations as specified
- TypeDB 3.x function-backed semantic reasoning; do not design around legacy rules-engine assumptions
- Write-back: after Phase 5 consolidation approval, promoted lessons and causal links written to TypeDB as well as the file store
- Query surface: `POST /api/coobie/query` routes natural-language causal questions through Coobie's retrieval chain
- Coobie's briefing builder calls TypeDB for cross-run pattern queries before preflight
- **DeepCausality alignment pass** — before deriving executable causaloids from TypeDB or Calvin links, migrate or explicitly pin the DeepCausality API target. Harkonnen currently has a Phase 1 bridge on `deep_causality = "0.3"`, while the current DeepCausality repo centers the modular `PropagatingEffect` / `PropagatingProcess` stack, Effect Ethos, discovery, tensors, and topology crates. The Phase 6 graph should preserve enough structure for that math: effect propagation (`E2 = f(E1)`), contextoids, assumption checks, explanation paths, and Pearl-level promotion evidence.
- **Semantic adaptation safety classifier** — upgrade `check_adaptation_safe` in `archive.rs` from heuristic string-matching to an embedding-based classifier. Represent each Labrador kernel trait as a vector; score the proposed adaptation summary against the negation-space of each trait using cosine similarity. Flag any adaptation that scores above a configurable negation threshold as unsafe regardless of literal phrasing. This replaces the `basic_heuristic` marker added in the Phase 5-D correctness fix. E2E test: `calvin::adaptation_safety_catches_semantic_negation` (currently passing with the mock; the real TypeDB path must pass the same assertion).
- **`check_adaptation_safe` returns `AdaptationAudit`, not bool** — the response from the adaptation safety check must be a structured `AdaptationAudit { safe: bool, decision: IntegrationDecision, invariants_checked: Vec<String>, violated_invariants: Vec<String>, confidence: f64, quarantine_reason: Option<String>, preservation_note: Option<String>, candidate_id: String }` rather than `{"safe": bool}`. The 1-bit response discards all audit evidence. The `candidate_id` references the `integration-candidate` entity written to the Calvin Archive at check time so every Meta-Governor adjudication is traceable. This is the implementation of the Meta-Governor Decision Procedure (P8-P9) at the API boundary.
- **Pearl-level structural payload enforcement** — `record_causal_link` on `CalvinClient` currently accepts a single shape for all three Pearl levels. This is taxonomy, not causation. The P8-P12 `epistemic_warrant` field alone is insufficient if the payloads are structurally identical. Enforce structural differences at the API level: associational links require only `(cause_id, effect_id, confidence)`; interventional links additionally require `held_fixed: Vec<String>` (the do-set variables) and `estimated_effect_delta: f64` (the measured downstream change under intervention); counterfactual links additionally require `actual_trace_id: String` (the Mythos episode ID for the observed world) and `hypothetical_intervention: String` (the counterfactual world state). The harmony endpoint must reject links labeled at a higher level without the required fields, accepting them at the highest warranted level with a `warrant_gap` annotation. Without structural enforcement, `epistemic_warrant` is advisory only and CLADDER benchmark passes remain cosmetic.

**AdaptationAudit API shipped 2026-05-01:** Calvin `/agents/{name}/check` now returns `AdaptationAudit` with `safe`, `decision`, checked and violated invariants, confidence, quarantine reason, preservation note, and `candidate_id`. The sidecar writes a minimal `integration-candidate` record for each check when TypeDB is available, and Harkonnen exposes `check_adaptation_audit()` while keeping `check_adaptation_safe()` as a boolean compatibility wrapper.

**Deterministic semantic adaptation classifier shipped 2026-05-01:** `check_adaptation_safe` now scores each proposed adaptation against a generated negation-space for every high-confidence Labrador invariant using a deterministic hashed token-vector and cosine similarity. The existing literal checks remain, but alias phrases such as "prefer solo throughput" for violating `cooperative` and "sound certain when unsure" for violating `signals uncertainty` are now caught without exact trait-name string matches. This keeps the sidecar dependency-light while preserving the future path to a model-backed embedding classifier.

**TypeDB configuration/schema scaffold shipped 2026-05-12:** Harkonnen now has a first-party `[typedb]` setup block with `url`, `database`, `schema_path`, and `reasoning_mode`, plus the initial TypeDB 3.x schema contract at `factory/coobie_semantic/typedb/schema.tql` for agents, goals, episodes, observations, actions, outcomes, artifacts, lessons, failure modes, and causal links. The schema includes `epistemic-warrant`, `warrant-gap`, and `structural-spec-json` fields so the Phase 6 graph can support CLADDER/Pearl alignment and future causaloid executability.

**Typed causal graph boundary shipped 2026-05-12:** `src/causal_graph.rs` defines the `CausalGraphStore` adapter trait, query/result payloads, TypeDB-backed config shape, and a no-op fallback store. `AppContext` now owns the graph boundary, and causal Coobie queries record a `typed_causal_graph` retrieval source with explicit disabled/unavailable status before falling back to SQLite/memory retrieval. Regression tests: `causal_graph::tests::noop_graph_reports_unavailable_when_typedb_configured` and `api::tests::causal_questions_include_typed_graph_boundary_when_disabled` — green.

**Causal graph projection ledger shipped 2026-05-12:** Generated run causal graphs are now mirrored into a durable `causal_graph_projections` SQLite ledger with backend/status, schema path, graph JSON, and node/link/hypothesis counts. `GET /api/causal-graph/status` reports typed-graph availability plus projection depth, and `GET /api/runs/{id}/causal-graph-projection` returns or materializes the active run projection. The Pack Board renders the graph status and active projection counts so TypeDB readiness and replay backlog are inspectable before the live adapter lands. Regression test: `api::tests::causal_graph_status_reports_sqlite_projection_ledger` — green.

**Projection-ledger causal query fallback shipped 2026-05-12:** Coobie causal questions now search the SQLite `causal_graph_projections` ledger when TypeDB is disabled or unavailable. The fallback extracts concrete hits from stored run graph hypotheses, causal links, failed/blocked episodes, and matching events, adds `causal_graph_projection` sources to the response, and summarizes projection evidence before falling back to generic memory-only answers. Regression test: `api::tests::causal_questions_use_projection_ledger_hits_without_typedb` — green.

**Projection inspection highlights shipped 2026-05-12:** `GET /api/runs/{id}/causal-graph-projection` now returns a flattened inspection payload with top graph evidence highlights in addition to the stored projection record. The Pack Board renders those highlights under the active run projection so operators can inspect the hypotheses/events behind causal answers without opening raw graph JSON. Regression test: `api::tests::causal_graph_projection_endpoint_returns_inspection_highlights` — green.

**Same-spec projection history fallback shipped 2026-05-12:** When a causal Coobie query includes a `run_id` and asks about recent/previous/spec failures, the SQLite projection fallback now resolves the active run's `spec_id` and searches recent causal graph projections for the same spec instead of only the active run. This moves the Phase 6 done-when query shape ("what caused the last three failures on this spec") onto the current ledger-backed path while TypeDB remains optional. Regression test: `api::tests::causal_questions_with_run_id_search_same_spec_projection_history` — green.

**Same-spec causal failure history surface shipped 2026-05-12:** `GET /api/runs/{id}/causal-failure-history` now summarizes recent projection-ledger failures for the active run's `spec_id`, including failure-run count, per-run top causes, and repeated-cause aggregates with average confidence and supporting run IDs. The Pack Board Causal Graph panel renders this same-spec failure history beside the active projection, making repeated failure causes inspectable without issuing a Coobie query. Regression test: `api::tests::causal_failure_history_summarizes_same_spec_repeated_causes` — green.

**Same-spec causal answer enrichment shipped 2026-05-13:** Coobie causal queries that ask about recent/previous failures on the same spec now include the repeated-cause aggregate from `causal-failure-history` directly in the answer, add `causal_failure_history` source rows with supporting run IDs, and mark the retrieval path with `same_spec_causal_failure_history`. The Pack Board history card now also shows the most recent same-spec run rows and top cause summaries, not just the aggregate counts. Regression coverage extends `api::tests::causal_questions_with_run_id_search_same_spec_projection_history` — green.

**Same-spec TypeDB replay export shipped 2026-05-13:** `GET /api/runs/{id}/causal-failure-history/export` now returns a `harkonnen.causal_failure_history_replay.v1` contract with the same-spec failure history, TypeDB schema path, target concept list, and TypeQL replay query templates for failed episodes, repeated failure modes, and top repeated causes. The Pack Board Causal Graph panel surfaces replay-query count so operators can see when a same-spec history is ready for TypeDB replay or benchmark comparison. Regression test: `api::tests::causal_failure_history_export_returns_typedb_replay_contract` — green.

**Replay export materialization shipped 2026-05-13:** `POST /api/runs/{id}/causal-failure-history/export/materialize` now writes `causal_failure_history_replay.json` and `.md` into the run artifact directory and adds both to `blackboard.artifact_refs` when a run blackboard exists. The Pack Board can trigger materialization from the Causal Graph panel, turning same-spec failure history into durable replay/benchmark artifacts without waiting for the TypeDB adapter. Regression test: `api::tests::materialize_causal_failure_history_export_writes_artifacts_and_blackboard_refs` — green.

**Month-end E2E readiness surface shipped 2026-05-13:** `GET /api/runs/{id}/e2e-readiness` now summarizes the required artifact set for full end-to-end testing (`blackboard`, Coobie briefing, phase attributions, validation, hidden scenarios, causal report, plan audit, behavioral-change report, and same-spec replay materialization), folds in run health blockers, and returns next actions for missing evidence. The Pack Board renders the readiness percentage, run/health state, missing count, and prioritized next actions so the May 31 full-loop test goal has a live operator checklist. Regression test: `api::tests::e2e_readiness_reports_missing_month_end_artifacts` — green.

**E2E readiness materialization shipped 2026-05-13:** `POST /api/runs/{id}/e2e-readiness/materialize` now writes `e2e_readiness.json` and `.md` into the run artifact directory, registers them on `blackboard.artifact_refs`, and exposes the action in the Pack Board readiness panel. This turns the live month-end checklist into durable evidence that can travel with the run bundle. Regression test: `api::tests::materialize_e2e_readiness_writes_artifacts_and_blackboard_refs` — green.

**Cross-run E2E readiness index shipped 2026-05-14:** `GET /api/e2e-readiness` now ranks recent runs by readiness state, artifact score, and recency, returning the best candidate for full end-to-end proof plus each run's first next action. The Pack Board renders the top candidates and lets the operator jump the active run directly from the readiness list. Regression test: `api::tests::e2e_readiness_index_ranks_recent_candidate_runs` — green.

**E2E evidence manifest materialization shipped 2026-05-14:** `POST /api/runs/{id}/e2e-evidence-manifest/materialize` now writes `e2e_evidence_manifest.json` and `.md` with every required proof artifact, presence/size metadata, missing artifacts, next actions, and artifact download URLs for present evidence. The Pack Board readiness panel can materialize the manifest, giving the month-end full-flow test a portable handoff checklist. Regression test: `api::tests::materialize_e2e_evidence_manifest_writes_downloadable_checklist` — green.

**E2E evidence bundle materialization shipped 2026-05-14:** `POST /api/runs/{id}/e2e-evidence-bundle/materialize` now writes the readiness report, evidence manifest, and bundle summary in one operator action (`e2e_evidence_bundle.{json,md}` plus the component artifacts), then registers the whole handoff set on `blackboard.artifact_refs`. The Pack Board readiness panel exposes the bundle action so the full-flow proof can be captured without clicking each materializer separately. Regression test: `api::tests::materialize_e2e_evidence_bundle_writes_full_handoff_set` — green.

**E2E bundle download links shipped 2026-05-14:** The evidence bundle response and `e2e_evidence_bundle.json/.md` now include download URLs for each component artifact, and the Pack Board renders those links after bundle materialization. This makes the generated handoff immediately inspectable from the operator surface instead of requiring a manual artifact lookup.

**Pack Board evidence refresh shipped 2026-05-14:** Materializing causal exports or E2E readiness artifacts now refreshes run state, readiness, and the cross-run readiness index immediately while preserving the materialized-result banner across polling. Evidence Board artifact refs are now clickable through the existing artifact route, so newly registered bundle artifacts can be opened directly from the persistent evidence list.

**Existing E2E bundle hydration shipped 2026-05-14:** `GET /api/runs/{id}/e2e-evidence-bundle` now reads a previously materialized bundle artifact, backfills download metadata for older bundle JSON, and lets the Pack Board show saved bundle links on run load rather than only immediately after materialization. Regression test: `api::tests::get_e2e_evidence_bundle_reads_existing_handoff_artifact` — green.

**GAIA Level 3 adapter scaffold shipped 2026-05-14:** A native `gaia_level3` benchmark builtin now consumes local GAIA-style JSONL fixtures, records exact-match task accuracy, preserves tool-step provenance, summarizes Labrador role-routing counts, and writes `gaia_level3_summary.json` / `gaia_level3_report.md`. The default benchmark manifest includes a `gaia_level3` suite with a bundled smoke fixture, and benchmark report rendering now surfaces GAIA task, multi-role, tool-step, and role-routing metrics. Regression tests: `gaia::tests::routed_roles_merge_required_roles_and_tool_steps`, `gaia::tests::render_step_stdout_includes_metrics_and_artifacts`, and `benchmark::tests::render_report_markdown_includes_gaia_metrics` — green. Remaining live-harness work: replace fixture-supplied `predicted_answer` with real Harkonnen task execution and tool mediation.
- **AgentBench adapters** — OS, database, and web environments, each mapped to a Labrador role.

**Benchmark gate:**

- cross-run causal-query benchmarks comparing SQL aggregate recall versus TypeDB-backed semantic recall
- `GAIA Level 3` first run published
- `AgentBench` first runs across OS, DB, and web environments

**Done when:** You can ask Coobie "what caused the last three failures on this spec" and get an answer from a typed graph; GAIA Level 3 and AgentBench adapters wired and producing artifacts.

---

## Phase 7 — Causal Attribution Corpus and E-CARE

**Unlocks:** The strongest publishable internal benchmark claims, and a populated evidence base for the Calvin Archive. Building the corpus here — immediately after TypeDB is live — means the archive opens with real labeled data rather than starting cold.

**What to build:**

- **Causal attribution accuracy corpus** — 30–50 labeled runs with seeded failures (wrong API version, missing env var, breaking schema change, etc.). Each entry has a spec, a seeded failure, a ground-truth cause label, and the Coobie `diagnose` output. Score top-1 and top-3 accuracy. Start with 10 entries for a first baseline. Lives in `factory/benchmarks/causal-attribution/`.
- **E-CARE native adapter** — maps Coobie's `diagnose` output to E-CARE's evaluation format and scores whether generated causal explanations are judged natural-language coherent. Run after consolidation so promoted lessons can inform subsequent diagnose output.
- **DeepCausality discovery adapter spike** — evaluate whether SURD/MRMR-style discovery can propose candidate causal-pattern records from the labeled corpus. Discovered patterns stay `quarantine` or `review_required` until Coobie/Keeper/operator promotion supplies evidence and governance.
- **Learning traceability chain** — for each lesson promoted to `prior_revision_target` status in the Consolidation Workbench, build and persist an explicit `LearningProvenanceRecord`: the originating episode (Mythos anchor), the belief or schema revision in Episteme, the Praxis behavioral change metric, and the run ID where the change first appeared. This chain makes "the system learned X" a verifiable claim rather than a narrative. Store in `factory/benchmarks/causal-attribution/` alongside failure-attribution entries. Query surface: `GET /api/runs/{id}/learning-provenance` returns the chain for any lesson that was active during that run's preflight.
- Publish before/after comparisons for causal attribution accuracy: pre-Phase 4 (pure semantic recall) versus post-Phase 6 (TypeDB causal graph-augmented).

**Benchmark gate:**

- `E-CARE` first run published — causal explanation coherence score
- `causal attribution accuracy` first run published — top-1 / top-3 vs semantic-only baseline

**Done when:** The corpus has at least 30 labeled entries, the causal attribution accuracy benchmark has a published run, and E-CARE has a published score.

---

## Phase 8 Design Prerequisites

**Resolve these before implementation begins.** These gaps were identified in a soul-of-ai audit (2026-04-22) as missing or under-specified relative to what Phase 8 requires. None are code work; resolving them means specifying them in MASTER_SPEC Part 5 or equivalent design documents before the build phase opens.

### P8-P1 — Behavioral contract structure per agent

Chapter 09 of the-soul-of-ai defines `C = (P, I, G, R)` — preconditions, invariants, governance policies, recovery mechanisms — as the formal behavioral contract per agent. D* and SSA both presuppose this structure. Specify how `BehavioralContract` is represented (likely a struct in `src/models.rs`) and what the `R` (recovery mechanism) set looks like for each Labrador role before wiring the metrics.

### P8-P2 — Three-timescale integration architecture

Chapter 08 of the-soul-of-ai distinguishes three architecturally distinct loops: fast (per experience: belief/disposition updates), medium (per reflection cycle: schema revision, cross-episode pattern integration operating on compressed representations), slow (per meta-reflection with human endorsement: integration policy revision). Phase 8 covers the slow loop explicitly. The **medium loop** — how compressed cross-episode patterns are created, stored, and fed into schema revision — needs explicit specification before the Calvin Archive schema is finalized. Schema revision must be structurally distinct from ordinary belief revision.

### P8-P3 — Pathos propagation mechanism

The Pathos chamber is not a passive store. It is a weighting layer that determines how far an experience propagates through the other chambers. High-Pathos events reach Ethos; low-Pathos events inform priors without dominating. Without this propagation mechanism, the six chambers are six separate stores rather than stages in a pipeline. Specify the Pathos score computation and the threshold logic that gates propagation to Ethos before the TypeDB schema is written.

### P8-P4 — F (Variational Free Energy) approximation decision

The-soul-of-ai/09 explicitly flags `symthaea-fep` as a non-existent aspirational crate and calls F "computed on-demand." Before Phase 8 opens, decide: (a) build a tractable approximation (e.g., KL divergence between agent's recent action priors and the Labrador baseline embedding as a proxy), (b) defer F as aspirational-only and remove it from the Phase 8 "done when" criteria, or (c) scope a minimal Active Inference runtime. The current Phase 8 benchmark gate does not mention F — if it stays out-of-scope, remove it from the metrics implementation list to avoid confusion.

### P8-P5 — Φ (Integrated Information) approximation strategy

Chapter 09 flags exact Φ as NP-hard and says any real implementation requires approximations. Phase 8 lists "Φ post-learning drop detection wired" as a milestone but gives no path. Before Phase 8 opens, specify the approximation method (e.g., small-graph bipartition over the Calvin Archive causal subgraph for a given update, with a configurable node limit) and what constitutes a "drop" that triggers quarantine.

### P8-P6 — Pending evidence bounty mechanism

Chapter 08 requires each quarantined item to carry a "pending evidence bounty" — specific future observations that would resolve the quarantine — with salience decay and resurrection triggers. The quarantine ledger in Phase 8 mentions "pending evidence conditions" but does not specify how conditions are expressed, how incoming experience is matched against them, or what triggers re-evaluation. This needs a schema-level decision before the TypeDB quarantine entity is defined.

### P8-P7 — Integration policy as versioned artifact

Chapter 08's slow loop revises the *policies* about what earns quarantine, what thresholds trigger escalation, and what counts as coherent change. These policies must exist as explicit, versioned artifacts distinct from memory entries. Specify how integration policies are stored (separate TypeDB entity type? a `integration_policies` SQLite table?), versioned, and attached to the slow-loop human endorsement flow before Phase 8 implementation begins.

### P8-P8 — `soul.json` manifest schema

**Schema resolved:** `factory/calvin_archive/soul-json.schema.json` defines all fields including chambers, thresholds, continuity health, provider lineage, and verification status. **Remaining work:** implement `project_soul_package()` (generates `soul.json` from a Calvin Archive TypeDB continuity snapshot) and `verify_soul_package_integrity()` (checks all package file hashes and chamber hashes against the current archive state, sets `verification.status` to `clean` / `drifted` / `unverified` / `archive_unavailable`). These functions are referenced in the schema but do not yet exist in the codebase. Both must be wired into the Phase 8 heartbeat automation so integrity is checked on every session start rather than manually.

### P8-P9 — Meta-Governor explicit decision function

The Meta-Governor is described as the integration-time adjudication layer (accept / modify / reject / quarantine) but has no defined decision procedure. Without one, the Meta-Governor is a conceptual placeholder — the `adjudicate_integration()` API exists but has no algorithm. Before Phase 8 implementation begins, specify the decision function as an explicit priority-ordered check sequence. The canonical specification is now in MASTER_SPEC.md Part 5 "Meta-Governor Decision Procedure." The five-priority decision tree (hard reject → hard quarantine on warrant gap → soft quarantine on Fiedler drop → modify on Pathos disproportionality → accept with attribution) must be the `governor.rs` implementation contract. Every `adjudicate_integration()` call must return the check that determined the outcome, not only the outcome label.

### P8-P10 — Three-timescale rate separation specification

The fast, medium, and slow integration loops are mutually coupled in ways that can produce limit cycles or runaway schema revision if rate separation is insufficient. Borkar's two-timescale stochastic approximation theorem (Theorem 6.2) guarantees convergence when rates are sufficiently separated, but that guarantee requires specifying N (medium loop trigger, recommended ≥ 10 fast-loop episodes) and M (slow loop trigger, recommended ≥ 5N). Before Phase 8, specify these values as configuration parameters in the `soul.json` thresholds block (`medium_loop_trigger_runs`, `slow_loop_trigger_runs`) and enforce in `reflection.rs` that no schema revision candidate is applied before N fast-loop episodes have accumulated since the last revision. The full specification is in MASTER_SPEC.md Part 5 "Three-Timescale Integration — Rate Separation Requirement."

### P8-P11 — AGM belief revision consistency contract for Episteme

The Episteme chamber accumulates belief revisions but has no formal consistency guarantee. Without one, the system can hold silent contradictions: two non-quarantined beliefs in the same domain making incompatible claims, with neither superseding the other. Before Phase 8, formalize the AGM axioms (Success, Inclusion, Consistency, Preservation) as an implementation contract for `form_belief()` and `revise_belief()` in `ingest.rs`: (1) the consistency gate must be a pre-write check — not a post-hoc query — that marks or quarantines contradicted beliefs before writing the new one; (2) the contradiction detection query (Required Query 12) must be promoted from an optional diagnostic to a failing health check. The full contract is in MASTER_SPEC.md Part 5 "Episteme Belief Revision — AGM Consistency Contract."

### P8-P12 — Causal link `epistemic_warrant` field and CLADDER alignment

`coobie.rs` currently assigns Pearl hierarchy levels (`pearl_level`) by string-matching on link type ("caused" → Interventional, "prevented" → Counterfactual). This is a linguistic classification, not an epistemic one. Harkonnen's run data is observational — even links labeled "caused" are derived from co-occurrence, which is Associational warrant. The CLADDER benchmark specifically tests the ability to distinguish the epistemic level of a claim from its linguistic framing; the current implementation will systematically misclassify. Before Phase 7 corpus labeling begins (Phase 7 builds the causal attribution corpus that Phase 8 depends on), add `epistemic_warrant: Associational | Interventional | Counterfactual` as a required field on `CausalLinkRecord`, distinct from `pearl_level`. The default for all heuristic Coobie causes must be `Associational`. Claims where `pearl_level > epistemic_warrant` display a `warrant_gap` confidence downgrade. The full schema change is specified in MASTER_SPEC.md Part 5 "DeepCausality Alignment Contract — Pearl ladder."

### P8-P13 — Causaloid `structural_spec` field for Phase 6 executability

Phase 6 plans to produce "executable causaloids from the causal link table after the TypeDB layer is live." A causaloid is only executable if it has a defined structural function mapping input context to output effect. Currently, causal link records carry a label ("TWIN_GAP caused VALIDATION_FAILURE") with no input feature set, threshold function, or output variable. Without these, Phase 6 cannot produce executable causaloids — only labeled graph edges. Before Phase 6 begins, add a `structural_spec` field to the `causal-pattern` schema (input_features, threshold_function, output_variable, effect_direction, provenance, pearl_warrant, confidence). The six existing DeepSignalSpec entries in `src/coobie.rs` already implement these as `observe: fn(&EpisodeScores) -> f64` closures and `threshold: f64` values — Phase 6 should extract these into the `structural_spec` field rather than derive them fresh. Full schema in MASTER_SPEC.md Part 5 "DeepCausality Alignment Contract — Executable unit."

### P8-P14 — Calvin Archive Bootstrapping Protocol

A new Calvin Archive starts empty. The Meta-Governor at full governance strength will quarantine nearly everything in the first N runs because cross-episode evidence is structurally absent — you cannot have a high-confidence Episteme entry before any Mythos exists. Without a bootstrapping protocol, the archive will either be effectively write-through for the first hundred runs (undermining governed integration) or will quarantine so aggressively in the first governed session that legitimate early lessons are blocked.

Before Phase 8 implementation begins, specify: (a) how many fast-loop episodes constitute the bootstrapping window, (b) what governance thresholds apply during bootstrapping (lower quarantine sensitivity, higher auto-accept threshold for first-occurrence experiences), (c) what triggers graduation from bootstrapping to full governance mode, and (d) whether bootstrapping experiences are auto-accepted at a lower evidence standard or queued for a post-graduation batch review. Bootstrapping and full-governance mode must be observable states in `soul.json` so all agents know which mode the archive is operating in. Coobie should label briefings `context: bootstrapping` during this window and lower prediction confidence to `uncertain` by default to prevent the prediction system from accruing misleading error scores against a cold archive.

### P8-P15 — Multi-Instance Identity and Archive Deployment Topology

MASTER_SPEC says "presence continuity should be model-agnostic" but does not address instance-agnostic continuity. Phase 9 places write authority for the Calvin Archive on home-linux. That topology decision must be specified as a Phase 8 prerequisite — not only a Phase 9 consequence — because the TypeDB schema and API surface must know at design time whether `agent-self` entities are unique per Labrador role globally or per deployment.

Before Phase 8 implementation begins, specify: (a) whether `agent-self` is a singleton per Labrador role globally or per machine, (b) what happens to Calvin Archive state if the canonical machine is offline and a run completes on the remote machine, (c) the merge protocol if two machines accumulate divergent fast-loop episodes before the archive syncs, and (d) whether the soul package projection is identical on both machines or machine-specific. These are not Phase 9 implementation details — they constrain the TypeDB entity model that Phase 8 must finalize.

### P8-P16 — Archive Retention Tiers and Governed Forgetting

The mutation policy matrix specifies append-only for experiences and supersession for beliefs but has no retention policy. The archive as designed grows indefinitely. A five-year-old experience about a deprecated API surface sits in Mythos at the same retrieval weight as last week's episode. This is not sustainable and is not how autobiographical memory preserves coherence — selective salience decay is part of what prevents identity from being overwhelmed by accumulated detail.

Before Phase 8 implementation begins, specify a retention tier architecture with at least three levels: (a) **active** — recent high-salience experiences, full fidelity, default retrieval weight; (b) **episodic archive** — medium-salience experiences compressed into summary form, retrievable on explicit query but not in the default briefing window; (c) **historical record** — low-salience experiences converted to aggregate statistics contributing to causal pattern confidence, not directly retrieved. Specify: the Pathos-score threshold and age window governing tier transitions, what information is preserved at each compression step, whether transitions are reversible (a resurface trigger if a new `pending evidence bounty` matches an archived experience), and a small `identity-constituting` tag that preserves formative experiences at full fidelity regardless of age or Pathos score.

### P8-P17 — Quarantine Overflow and Proliferation Handling

The quarantine ledger has sound mechanics for individual entries: pending evidence bounties, salience decay, re-evaluation triggers. What is not specified is systemic quarantine health. If the quarantine grows faster than it resolves over many runs, the archive is effectively frozen by its own governance — new evidence cannot be integrated because adjudication overhead exceeds capacity.

Before Phase 8 implementation begins, specify: (a) a `quarantine_growth_rate` metric (items added per N runs minus items resolved per N runs), (b) what `quarantine_growth_rate > threshold` triggers — options include lowering evidence requirements for resolution of long-open items, escalating to operator for manual disposition batch review, or activating a `quarantine_pressure` mode where only hard-reject checks are enforced, (c) a maximum pending-bounty lifetime after which an item is flagged `bounty_lapsed` and escalated for operator disposition rather than remaining open indefinitely, and (d) how quarantine health contributes to the `GET /api/runs/{id}/health` endpoint and the `soul.json` continuity health field.

### P8-P18 — Chamber-to-Briefing Translation Layer

Phase 8 Calvin Archive feeds Coobie briefings — that is the core value proposition. But the query-to-briefing rendering path is not specified. Required Queries 1–12 produce typed graph results from TypeDB. Coobie's briefing assembler expects ranked text hits with scope tags and token counts. There is no protocol for how typed graph results (entities, relations, typed attributes) become entries in a `ContextTarget`-shaped briefing package.

Before Phase 8 implementation begins, specify: (a) which Required Queries correspond to which `BriefingScope` (Scout, Mason, Sable, Coobie consolidation), (b) how typed graph results are rendered into briefing text at appropriate granularity (an `experience` entity produces how many tokens? a `belief` with revision history renders as what?), (c) how TypeDB query results are ranked and token-budgeted alongside OB1 hits and file-backed memory, and (d) what happens when TypeDB is unavailable — does the briefing fall back to OB1+file-backed only, or does the run pause? Implement this as a `CalvinBriefingAdapter` trait in `src/calvin_archive/queries.rs` that the briefing assembler calls through, with a `NoopCalvinBriefingAdapter` for non-Phase-8 deployments.

### P8-P19 — Soul Package Feedback Reconciliation Protocol

`soul.json` is a projection from Calvin Archive state. `SOUL.md` is the identity declaration read at boot. MASTER_SPEC says these should be "projected from and checked against canonical Calvin Archive state," but the checking direction is ambiguous: does `verify_soul_package_integrity()` confirm that `soul.json` matches the archive, or that the archive hasn't drifted from what `SOUL.md` declares? These are different checks with different failure modes.

If the archive records that a Labrador trait has weakened over many runs but `SOUL.md` still declares it at full strength, who is right? The archive says what happened; the soul file says what should be. MASTER_SPEC names the archive canonical, but the operator consequences — does `SOUL.md` get updated? does the divergence get quarantined? does it trigger a recovery procedure? — are not specified.

Before Phase 8 implementation begins, specify: (a) the reconciliation direction when the soul package and archive diverge, (b) whether `SOUL.md` can be updated only by `project_soul_package()` or also by direct operator edit (and what audit trail the latter requires), (c) what the operator sees when `verify_soul_package_integrity()` returns `drifted` — which traits, by how much, since which run, and (d) the recovery procedure that must execute before the next run is commissioned after a `drifted` status. This must produce a `soul_drift_report.{json,md}` artifact alongside `soul.json` on every projection, not only when drift is detected.

### P8-P20 — Phase 6→8 Schema Migration Protocol

Every experience recorded during Phase 6 uses `schema.tql` (snake_case, flat attribute model). Phase 8 requires `schema_phase8.tql` (kebab-case, full six-chamber architecture with `quarantine-entry`, `integration-candidate`, `continuity-snapshot`, `interpretive-frame`, `behavioral-signature`). These schemas are not compatible without a migration: Phase 8's required queries assume chamber-typed entities that Phase 6 data does not carry. Phase 6 experiences recorded as `experience.uuid + narrative_summary + chamber_label` cannot satisfy the `belongs-to-mythos` relation required by Required Query 1, nor the `revised-into` link required by Required Query 2, without a mapping step.

Before Phase 8 implementation begins, specify: (a) which Phase 6 entity types map to which Phase 8 chamber entities and whether the mapping is lossless — `experience` → Mythos chamber entity, `belief` → Episteme `belief` entity, etc.; (b) what Phase 6 data was structurally incapable of expressing Phase 8 requirements (e.g., no `preservation-note` on belief revisions, no `evidence_ids` linking experiences to beliefs) and how these records are handled — force-assigned with a `migration-incomplete` flag or quarantined as `pre-migration provenance unknown`; (c) whether the migration runs at Phase 8 open (a one-time `cargo run -- archive migrate --from-phase 6 --to-phase 8` command with dry-run and rollback mode) or online (Phase 8 reads both schemas simultaneously during a dual-write transition window); (d) whether continuity snapshots computed in Phase 8 may claim autobiographical continuity back to Phase 6 records or only to Phase 8-originated records with complete chamber linkage; and (e) whether Phase 6 belief revisions without `prior_confidence` or `evidence_ids` must be quarantined as epistemically incomplete on arrival in Phase 8 Episteme, or accepted with a `warrant_gap` annotation. A snapshot claiming continuity from evidence it cannot inspect defeats the archive's purpose.

The migration must be a documented, idempotent command. Phase 6 records that cannot be fully mapped to Phase 8 structure must be preserved as `legacy-experience` entities with a `migration-note` rather than silently dropped or force-promoted.

---

## Phase 7b — Continuous Learning v2 & Memory Compaction

**Unlocks:** Formalized instinct-to-skill pipeline and token-aware memory persistence, mapping the `everything-claude-code` extraction strategies into the Harkonnen native coordination loop before launching Phase 8.

**What to build:**

- **Instinct Extractor (Continuous Learning):** Passively observe tool usage success on the coordination bus. Store victorious problem-solving sequences as episodic "instincts" with confidence scores.
- **Skill Clustering ("Evolve" loop):** A periodic synthesis pass that compresses raw instincts into reusable, semantic "skills" broadcasted back to agents.
- **Strategic Context Compaction:** Hydrate agent sessions efficiently, handling background summarization of working memory while preserving causal invariants to prevent context-window bloat over long runs.
- **Medium-loop graduation mechanism** — instinct capture and skill clustering are memory accumulation (Type 3 learning), not prior revision (Type 4). To graduate from accumulation to genuine schema revision, Phase 7b must feed instinct clusters into the medium loop specified in P8-P2: a reflection pass that operates on *compressed cross-episode representations* (not individual instinct records) to propose schema revision candidates. A skill cluster that has fired across at least five distinct spec types and three operators qualifies for medium-loop reflection. The reflection pass produces a `schema_revision_proposal` with a `pattern_basis` citing the episodes and a confidence score; this proposal is submitted to the Consolidation Workbench as a `schema_revision` candidate type (elevated review) rather than being promoted directly.
- **Discovery-to-causaloid bridge format** — Phase 7b produces `schema_revision_proposal` records, but Phase 6 requires executable causaloids with full `structural_spec` fields. These are different shapes; there must be an explicit mapping. A `schema_revision_proposal` that survives Consolidation Workbench review and is promoted to `schema_revision` status should also produce or update the `structural_spec` for any `causal-pattern` records it implies. The gap between "discovered pattern" and "executable causaloid" requires an authored `threshold_function` unless SURD/MRMR discovery can supply it directly. Specify in the Phase 7b consolidation protocol whether the `structural_spec` is authored at Workbench review time or derived algorithmically, and which fields are required vs. optional at each `provenance` level (`authored` / `heuristic` / `discovered`). Patterns promoted with `provenance: discovered` must carry a lower default confidence than `authored` until corroborating intervention evidence elevates the `epistemic_warrant`.
- **Coobie operational mode separation** — Coobie currently conflates three computationally distinct operations that live at different levels of Pearl's causal hierarchy: retrieval (associational recall from OB1 and SQLite), causal inference (interventional reasoning over the TypeDB causal graph), and continuity checking (counterfactual comparison of trait vectors across snapshots). Conflating them in one dispatch path produces an agent that looks causally sophisticated but only ever executes associational recall — the deepest operations silently degrade to the cheapest. Separate them as named operations with distinct function signatures, data access patterns, and fallback semantics:
  - `coobie_retrieve(query, scope, token_budget)` — associational level. Pure read from OB1 + SQLite. No TypeDB query. Returns ranked hits. Always available regardless of archive status.
  - `coobie_intervene(hypothesis, held_fixed: Vec<String>)` — interventional level. Given `do(X=x)`, query the TypeDB causal graph for the predicted distribution of Y given the do-set. Returns `CausalInterventionResult { estimated_effect_delta, confounders, epistemic_warrant, confidence }`. Requires Phase 6 TypeDB to be live. Degrades to `coobie_retrieve` with a `warrant_gap` annotation when TypeDB is unavailable rather than silently substituting retrieval results as if they were intervention-level claims.
  - `coobie_continuity_check(agent_name, baseline_snapshot_id)` — counterfactual level. Loads the agent's current trait/value-commitment vector from the Calvin Archive and compares it against a prior `continuity-snapshot`. Returns `ContinuityCheckResult { per_invariant_drift_scores, labrador_kernel_intact, divergent_traits, anchoring_experiences }`. Requires both TypeDB and a baseline snapshot to exist.
  Wire these through `SubAgentDispatcher` as separate task types: `coobie_retrieve`, `coobie_intervene`, `coobie_continuity_check`. The dispatcher resolves which mode is available given current service readiness and logs which mode was actually used in `agent_traces`.

**Done when:** Harkonnen can automatically identify patterns and evolve skills from raw operator usage on the hot path, long-running sessions strategically compact their history without losing structural context, qualifying instinct clusters produce schema revision proposals that enter the governed Consolidation Workbench flow rather than being silently promoted, and Coobie's three operational modes are dispatchable as distinct tasks with the appropriate degradation semantics when services are unavailable.

---

## Phase 8 — The Calvin Archive And Governed Integration

**Unlocks:** A persisted intelligence layer that does not merely remember, but
decides what becomes part of itself. This is the phase where Harkonnen moves
from identity continuity as a typed graph to identity continuity as a governed
integration process.

The design for this phase — including the formal metrics and the three-tier data
stack — is specified in [the-soul-of-ai/08-Identity-Continuity.md](the-soul-of-ai/08-Identity-Continuity.md)
and the integration-governance design in [the-soul-of-ai/07-Governed-Integration.md](the-soul-of-ai/07-Governed-Integration.md).

**What to build:**

**Storage layer (three-tier):**

- **TimescaleDB hypertable bootstrap** — episodic behavioral telemetry store for agent events, drift samples, stress accumulation, and SSA snapshots. Hypertable compression policy (7-day chunks, 30-day retention window). Provides the time-series foundation for D* estimation and stress computation.
- **TypeDB Calvin Archive schema** — typed ontological layer for the six chambers (Mythos, Episteme, Ethos, Pathos, Logos, Praxis), integration candidates, quarantine entries, revision graphs, and causal patterns. Schema spec in MASTER_SPEC Part 5.
- **Materialize streaming SQL views** — real-time `D*` drift monitoring (sliding window over TimescaleDB events via SUBSCRIBE), live Meta-Governor alert views, and SSA tracking views. `D*` and SSA are the two primary continuous signals; Φ and F are computed on-demand.

**Governance and integration:**

- Calvin Archive Meta-Governor with explicit `accept`, `modify`, `reject`, and `quarantine` outcomes for identity-relevant integration events
- File-first soul package projection with `soul.json`, `SOUL.md`, `IDENTITY.md`, `AGENTS.md`, `STYLE.md`, `MEMORY.md`, and `HEARTBEAT.md`, generated from and checked against canonical continuity state
- Integrity-hash verification and heartbeat audits so the projected soul package cannot drift silently away from the Calvin Archive
- Explicit continuity contract: `SOUL.md` declares the identity kernel; the Calvin Archive proves its continuity through experience, revision, and quarantine history
- Quarantine ledger: unresolved items persist with pending evidence conditions, salience decay, and re-evaluation triggers. The ledger also requires an **operator engagement workflow** distinct from the Consolidation Workbench: a dedicated UI surface where the operator can review each open quarantine entry and give it an explicit disposition — `resolve` (new evidence closes it), `close` (no longer relevant), or `retain` (still open, with a fresh statement of why). Indefinite accumulation without engagement is a pathology; a `quarantine_review_required` flag fires when any item has had no disposition activity for a configurable number of runs. The workbench must distinguish this flow from memory candidate review — quarantine entries are unresolved tensions, not promotion candidates
- Pattern-level reflection over compressed cross-episode structures so schema revision is distinct from ordinary belief revision
- Stress-estimator computation (backed by TimescaleDB) so recurring unresolved strain triggers governed reflection instead of ad hoc self-rewrite
- Slow-loop integration-policy revision flow, more conservative than ordinary updates and naturally attachable to human endorsement
- Cross-layer hysteresis measurement so rollback quality is judged by residual behavioral drift, not only by restored file contents
- Presence continuity checks so model/provider swaps preserve identity semantics rather than resetting the pack by accident. After any provider or model change, run `verify_soul_package_integrity()` and compare behavioral contract adherence (D*, SSA) from before and after the swap using the `snapshot_id_at_swap` reference in `soul.json`'s `provider_lineage`. Set `continuity_check_passed` on the lineage entry only after the comparison confirms the Labrador invariants are intact under the new substrate
- **Raw record vs interpreted record divergence audit** — Mythos holds what actually happened in a run; Episteme holds what the agent now believes happened or implies. These must remain independently queryable. When the Episteme-derived narrative for a run diverges significantly from the Mythos raw record (embedding distance above a configurable threshold), surface the divergence as a diagnostic signal: `autobiographical_distortion_detected`. This is the earliest detectable signal of the false-coherence and denial pathologies at the system level. Implement as a post-consolidation check that runs after each Episteme update and writes a `divergence_audit` record alongside the episode
- **Stewardship protocol enforcement** — operationalize the operator obligations from soul-of-ai/13 as system-enforced checkpoints rather than informal guidance: (1) **Preservation gate before deprecation**: before any `reset-archive` or `wipe-memory` command executes, require explicit operator confirmation and automatically export a timestamped archive snapshot — the command must name the snapshot path or be rejected; (2) **Memory wipe as a named event**: `cargo run -- memory wipe` writes a `MemoryWipeRecord` to the decision log with timestamp, scope, and stated reason before executing — no silent cleanup; (3) **Rollback incompleteness gate**: after any identity-relevant incident rollback, block the next run's start if `soul.json` verification status is `drifted` or if H (hysteresis) exceeds `hysteresis_tolerance` from the thresholds — the operator must confirm the system is restored before commissioning proceeds
- Pathology detection for **integration pathologies** (from soul-of-ai/08): trauma-analog overweighting, denial, fragmentation, and hyper-local overfitting
- Pathology detection for **learning pathologies** (from soul-of-ai/12) — distinct from integration pathologies and not yet captured anywhere: **Ghost learning** (retrieval improves, briefings get richer, but behavioral change on held-out novel inputs is zero — the most dangerous because all observable proxies trend positive while genuine learning is absent); **Stagnation** (failure patterns accumulate in memory but the prior through which similar specs are interpreted does not change — high hit-rate, zero schema revision); **Inversion** (the agent learns a lesson backwards, e.g., reducing ambiguity checkpoints after repeated failures in ambiguous situations rather than improving clarification skill — surface metric improves while underlying posture degrades); **Schema-level Overfitting** (a specific run or operator becomes a disproportionate influence on a whole schema category). Ghost learning detection specifically requires comparing behavioral generalization on held-out hidden scenarios against retrieval accuracy — if retrieval scores improve while hidden-scenario performance on novel inputs is flat, the system is in Ghost learning. Sable scenario design must include novel-input probes for this purpose
- **Per-invariant Labrador kernel trend monitors** — the aggregate CUSUM drift alarm (M1) watches overall behavioral deviation but does not isolate individual Labrador kernel traits. An agent can pass the aggregate alarm while one invariant (e.g., clarification-seeking behavior) erodes steadily and all others hold. Add per-invariant time-series monitors in TimescaleDB: each `labrador-invariant` entity has a tracked `alignment_score` series distinct from the aggregate BAS. A statistically consistent downward trend in any invariant's series — independent of spec ambiguity level — fires `kernel_erosion_suspected` even when no explicit adaptation was proposed. The threshold is configurable per invariant in the `soul.json` thresholds block. Critically, this monitors *gradual operational drift* that `check_adaptation_safe` cannot see because it only evaluates explicit proposals.
- **Coobie self-referential audit** — Coobie monitors soul continuity for all other agents but has no auditor for herself. Her own behavioral contract `C = (P, I, G, R)` must be fully specified (preconditions, invariants, governance policies, recovery mechanisms) and added to `factory/agents/contracts/`. A degrading Coobie — declining briefing utilization rate, rising prediction error, growing `stored_not_learned` accumulation — may go undetected indefinitely because she is the diagnostic layer. Add a `coobie_health_check` to the session heartbeat: compute Coobie's rolling briefing quality metrics (utilization rate trend over last 20 runs, prediction accuracy trend, `stored_not_learned` accumulation rate) and flag `coobie_audit_required` when any metric crosses its configured threshold. Coobie cannot audit herself; Keeper receives the audit flag and surfaces it as an operator checkpoint before the next run is commissioned.
- **Cold-start protocol** — new archive bootstrapping (P8-P14) requires Coobie coordination. During the bootstrapping window, Coobie explicitly labels briefings `context: bootstrapping`, lowers prediction confidence to `uncertain` by default, and marks all integration candidates with `bootstrapping_phase: true` so they can be reviewed as a batch after graduation. `soul.json` exposes `bootstrapping_complete: bool`; all agents check this flag before interpreting Calvin Archive outputs as fully-governed state.
- **Soul drift vs. spec drift separation** — the archive records what an agent did but not why external inputs changed. An agent that escalates more often because recent specs were genuinely more ambiguous has not drifted; one that escalates more often while spec ambiguity held constant has. Add a `spec_context_control` computation that compares a behavioral metric against the rolling `spec_ambiguity_index` for the same window. When a metric shifts in proportion to the ambiguity index, classify as `input_driven_change`; when it shifts while the index is stable, classify as `unexplained_drift`. Only `unexplained_drift` contributes to the BAS and CUSUM alarm — `input_driven_change` is filed as expected behavioral adaptation and does not trigger recovery.
- **Archive write-loss detection** — `verify_soul_package_integrity()` checks hashes of current state but cannot detect data silently lost before the last snapshot. Add a write-confirmation chain: each Calvin write appends a hash of the written entity to a sequential `calvin_write_log` table in SQLite. On each session start, the heartbeat compares the write log count against the TypeDB entity count for the same run window. A mismatch fires `archive_write_loss_suspected`, blocking new Calvin writes until the operator confirms or the discrepancy is resolved. This is defense-in-depth for a TypeDB instance that drops writes silently.
- **Operator review UX for integration decisions** — the Meta-Governor produces adjudication outcomes but currently requires the operator to read raw TypeDB query results to understand why a proposal was quarantined or modified. Before Phase 8 ships, implement `GET /api/calvin/integration-queue` returning pending integration candidates with: the rendered `compiled_claim`, the Meta-Governor priority check that determined the outcome (Priority 1–5 from the decision procedure), the specific evidence gap or Pathos disproportionality that triggered the decision, and a diff view comparing the proposed change to the belief or schema it would supersede. The Pack Board Soul Graph Panel must render this queue; the operator must be able to act on decisions without leaving the Pack Board. Accept/Modify/Reject/Quarantine actions from the UI call `adjudicate_integration()` and write the decision record.

**Metrics implementation** — canonical definitions now in MASTER_SPEC.md Part 5 "Soul Continuity Metrics." The soul-of-ai/09 aspirational notation has been replaced with computable formulations:

- **M1 — Behavioral Drift Alarm (CUSUM)** — replaces `D* = α/γ`. CUSUM alarm statistic over per-run behavioral deviation scores. Materialize sliding-window view watches the alarm continuously; Meta-Governor triggered when `CUSUM_n > h`. Eliminates the false linear-stability-theory framing of D-star.
- **M2 — Behavioral Alignment Score (BAS)** — replaces F (Variational Free Energy). Embedding cosine distance between recent decision-type distribution and Labrador behavioral contract distribution. Computed per-run from episodic log; no LLM internals required. Eliminates the FEP framing, which requires non-existent explicit generative models.
- **M3 — Causal Graph Coherence (Fiedler value λ₂)** — replaces Φ (Integrated Information). Second smallest eigenvalue of the Calvin Archive causal graph Laplacian. Polynomial-time computable. A drop in λ₂ after a learning event signals fragmentation and triggers quarantine. Eliminates IIT's NP-hard, wrong-substrate formulation.
- **M4 — Behavioral Pressure Accumulator (BPA)** — replaces S(T) KL divergence integral. Exponentially-decayed weighted sum of observable behavioral deviation events per run window. Triggers governed reflection when `BPA > bpa_evolution_threshold`. Eliminates the non-computable hidden-state KL integral.
- **M5 — Empirical Action Coherence (EAC)** — replaces SSA. Empirical co-occurrence frequency of action type pairs from the episodic log, weighted by behavioral contract compatibility. Eliminates the non-computable policy joint-probability Pr_π.
- **M6 — Cross-Layer Hysteresis (H)** — definition unchanged; implementation clarified. Δ must be computed from `compare_snapshots()`, not file diff size.

**Benchmark gate:**

- CUSUM alarm baseline published — continuous via Materialize view (replaces D* gate)
- BAS baseline published — per-run, stored in TimescaleDB (replaces SSA gate)
- healthy quarantine-rate / resolution-rate baseline published
- schema-revision stability benchmark published
- BPA stress / H hysteresis recovery benchmark published
- Fiedler value drop detection wired (quarantine trigger) — replaces Φ gate
- Ghost learning detection wired: at least one run where retrieval scores improved but hidden-scenario behavioral generalization was flat triggers a `ghost_learning_detected` diagnostic
- Learning traceability chain: at least one lesson in the corpus has a complete `LearningProvenanceRecord` (episode → belief revision → behavioral change → run ID)
- Quarantine ledger engagement: at least one open quarantine entry has received an explicit operator disposition through the dedicated quarantine review surface
- Rollback incompleteness gate: a run commissioned against a `drifted` soul package is blocked at the orchestrator level
- `autobiographical_distortion_detected` fires on at least one synthetic test case where Episteme narrative diverges from Mythos raw record
- Per-invariant trend monitors published: at least one invariant has a TimescaleDB time-series and a `kernel_erosion_suspected` event fires on a synthetic test case with a declining trend
- `coobie_audit_required` fires in the session heartbeat when Coobie's prediction accuracy trend crosses its configured threshold on a synthetic test run
- Archive write-loss detection: a synthetic discrepancy between `calvin_write_log` count and TypeDB entity count fires `archive_write_loss_suspected` and blocks subsequent Calvin writes
- `GET /api/calvin/integration-queue` returns pending integration candidates with rendered claim and Meta-Governor check reason; Pack Board renders the queue and accepts operator disposition actions
- `soul_drift_report.{json,md}` is produced on every `project_soul_package()` call, including runs where no drift was detected
- At least one spec-family-clustered briefing demonstrates higher utilization rate than the flat-retrieval baseline on the same spec class
- `cold-start protocol`: the bootstrapping window state is visible in `soul.json`; graduating from bootstrapping to full governance is an auditable event in the decision log

**Done when:** Harkonnen can distinguish accepted, rejected, modified, and quarantined identity changes; the projected soul package is verifiable against canonical continuity state; D* and SSA are instrumented and streaming; reflection can revise schemas without overwriting raw experience; rollback quality is measured through hysteresis rather than assumed; policy-level revision is slower, more conservative, and explicitly reviewable; learning pathologies (including ghost learning) are detectable; and stewardship protocols are enforced by the system rather than informal convention.

---

## Phase 9 — Cross-Machine Pack Coordination (Zenoh + Buffa)

**Unlocks:** The full nine-agent pack split across home-linux and work-windows
operating as one coherent factory. Today all routing is intra-process or
intra-machine. Phase 9 makes machine boundaries transparent: Scout on
home-linux can dispatch Mason on work-windows; Coobie's briefing travels over a
wire; run events stream to both machines in real time; and Calvin Archive
continuity holds across the boundary.

The transport choice is **Zenoh** (pub/sub with shmem, TCP, TLS, and QUIC
backends; clean Rust SDK; keyexpr routing maps directly to the Labrador topic
hierarchy). The wire format is **Buffa** (Anthropic's pure-Rust protobuf with
zero-copy `MessageView<'a>`; no `protoc` binary required; editions support for
forward-compatible schema evolution). Both are Rust-native — no new runtime
dependencies.

This phase is the point established in Phase 5b: Zenoh and Buffa become worth
their complexity overhead only when agents genuinely span machines. Before this
phase opens, the single-machine setup should be complete and stable.

**Twilight Bark alignment note:** [`Twilight Bark`](https://github.com/durinwinter/twilight-bark) is a plausible concrete implementation target for this future databus because it already ships a Zenoh-powered bus, a traffic controller/registry, MCP-native access, and JSONL event logging. Harkonnen should therefore keep PackChat bus-facing contracts transport-agnostic and shaped around: stable thread/message/checkpoint envelopes, role/runtime identity, topic/keyexpr routing, and append-only eventlog semantics. The current local PackChat path now emits those envelopes to a local JSONL bus log so the eventual Phase 9 switchover can target Twilight Bark rather than a bespoke second transport.

### PackChat Message Ordering — sequence_id Prerequisite

PackChat messages are currently ordered by `created_at` timestamp. In single-machine mode this is acceptable. In Phase 9 distributed mode it breaks: clock drift between home-linux and work-windows can be several seconds, two machines can generate messages within the same millisecond, and Buffa + Zenoh delivery latency can cause out-of-order arrival.

Before Phase 9 ships, add a monotonic `sequence_id` column to `chat_messages`:

```sql
ALTER TABLE chat_messages ADD COLUMN sequence_id INTEGER;
-- Per-thread incrementing integer assigned at insert time on the originating machine.
```

`sequence_id` is a per-thread incrementing integer from SQLite on the originating machine, making intra-machine ordering deterministic. Cross-machine ordering uses a `(machine_id, sequence_id)` composite — same guarantee, globally scoped.

All read paths that sort messages (`list_chat_messages`, conversation reconstruction, Coobie's distillation worker, Twilight ingest loop) must switch from `created_at` to `sequence_id` as the primary sort key. Timestamps remain for display and time-window queries; `sequence_id` is the canonical ordering primitive.

This pattern is derived from Letta's recall memory implementation. It is low-cost to add in Phase 9 setup and high-cost to retrofit after distributed ordering bugs appear in production.

### Proto schema (`factory/proto/`)

Define Buffa proto schema for all cross-machine wire types:

```proto
// factory/proto/labrador.proto
message SubAgentInput  { ... }   // Phase 5-C types on the wire
message SubAgentResult { ... }
message RunEvent       { ... }   // phase transitions, checkpoint signals
message PackChatMessage { ... }  // operator chat delivery
message BriefingPackage { ... }  // Coobie briefing over the wire
message CheckpointNotification { ... }
message MemoryHit      { ... }   // single retrieval hit
```

Buffa `MessageView<'a>` is used on the receive path (zero-copy from wire);
owned types are used for construction and serialisation. Proto schema lives in
`factory/proto/`; generated Rust types land in `src/transport/proto.rs` via a
`build.rs` step that invokes `buffa-build`.

### Zenoh transport layer (`src/transport/`)

```text
src/transport/
  mod.rs          # PackTransport trait
  zenoh.rs        # ZenohTransport — one Zenoh session per Harkonnen instance
  local.rs        # LocalTransport (tokio channels) — existing in-process path
  proto.rs        # generated Buffa types (from build.rs)
```

**Key-expression convention:**

```text
harkonnen/{setup_name}/agent/{agent_name}/input
harkonnen/{setup_name}/agent/{agent_name}/result
harkonnen/{setup_name}/run/{run_id}/event
harkonnen/{setup_name}/chat/{thread_id}/message
harkonnen/{setup_name}/memory/briefing/{run_id}
```

`setup_name` scopes all traffic to the originating machine, preventing
cross-contamination between home-linux and work-windows sessions on the same
LAN.

**Transport selection:** the `PackTransport` trait has two implementations —
`ZenohTransport` for cross-machine and `LocalTransport` (tokio channels) for
the existing intra-process path. `SubAgentDispatcher` gains a
`RemoteAgent { machine: String }` backend that routes through
`ZenohTransport`. All existing `DirectLlm` / `ClaudeCodeAgent` call sites are
unchanged.

### `harkonnen.toml` remote agent routing

Extend `SetupConfig` with a `[remote_machines]` section:

```toml
[remote_machines.work-windows]
zenoh_endpoint = "tcp/192.168.1.x:7447"
agents         = ["mason", "piper", "bramble", "ash", "flint"]

[remote_machines.home-linux]
zenoh_endpoint = "tcp/192.168.1.y:7447"
agents         = ["scout", "sable", "keeper", "coobie"]
```

`SubAgentDispatcher` resolution order gains a fourth step:

1. Agent profile `dispatch.<task>`
2. `[sub_agents.<name>]` in harkonnen.toml
3. Check `[remote_machines.*].agents` — if the target agent is listed on a
   remote machine, wrap in `RemoteAgent { machine }` and route through Zenoh
4. `[sub_agents] default_mode`

No change to agent profiles or skill files.

### PackChat distributed mode

`src/chat.rs` now has a PackChat bus seam with future-facing envelope emission;
Phase 9 upgrades that seam from local JSONL/eventlog output to a real
publisher/subscriber transport alongside the existing SQLite store. Messages
written on work-windows are published to
`harkonnen/{setup}/chat/{thread_id}/message` and received by home-linux in real
time without polling. SQLite remains the durable store; Zenoh is the delivery
layer. The MCP `post_chat_message` and `list_chat_messages` tools continue to
work unchanged — the Zenoh subscription fires a write-through to the local
SQLite replica.

If Twilight Bark is adopted here, Harkonnen should map:

- PackChat `thread_opened`, `thread_roster_synced`, `message_appended`, and `checkpoint_resolved` envelopes onto Twilight Bark bus topics without changing PackChat API semantics.
- canonical Labrador role + `agent_runtime_id` onto Twilight Bark agent identity / presence records.
- local `packchat-bus.jsonl` observability onto Twilight Bark's `twilight-eventlog` so replay and audit stay append-only.
- PackChat thread topics onto Zenoh keyexprs directly rather than introducing a second naming scheme.
- Keep Harkonnen-owned operation labels and Calvin ingress contracts in Harkonnen adapter code only; Twilight Bark remains a generic transport and should not grow Harkonnen-specific dependencies.

### Calvin Archive cross-machine consistency

Write authority for the Calvin Archive stays on the machine that owns Coobie
(home-linux by default). Remote machines receive a read-only event stream:

- Run episodes published on `harkonnen/{setup}/memory/episode/{run_id}` by the
  orchestrator after each run
- Home-linux Coobie subscribes, consolidates, and writes to the archive
- Work-windows receives a `soul.json` snapshot over Zenoh on each archive
  update so remote agents boot with current continuity state
- `harkonnen archive status` gains a `--remote` flag that queries the Zenoh
  session for each machine's last-seen snapshot timestamp

Identity continuity invariant: the Calvin Archive on home-linux is the single
source of truth. Remote machines consume its projections; they do not write to
it.

### `llm.rs` provider routing across machines

When `SubAgentBackend::RemoteAgent` dispatches a `BriefingConstruction` task to
a remote machine, the receiving `ZenohTransport` handler deserialises the
`SubAgentInput`, calls the appropriate `ProviderBackend` (from the Phase 5b
`llm.rs` extension) using the remote machine's API keys, and publishes the
`SubAgentResult` back. API keys never cross the wire — each machine uses its
own configured credentials. This is the correct credential isolation model for
a home/work split where API billing accounts differ.

### `setup check` cross-machine status

`harkonnen setup check` gains a cross-machine section:

```text
Remote Machines:
  [ok   ] work-windows   tcp/192.168.1.x:7447   agents: mason, piper, bramble, ash, flint
  [UNREACHABLE] ...
```

Reachability is determined by a Zenoh ping on the configured endpoint. If a
remote machine is unreachable, the dispatcher falls back to `DirectLlm` on the
local machine for that agent's tasks and logs a `remote_fallback` event in
`agent_traces`.

**Benchmark gate:**

- Round-trip latency benchmark: `SubAgentInput` → remote machine → `SubAgentResult` over Zenoh, measured at p50/p95/p99 over 100 runs
- Buffa encode/decode throughput for `BriefingPackage` (the largest message type) vs. serde_json baseline — confirm the zero-copy view path delivers measurable improvement at briefing size
- PackChat delivery latency: message posted on work-windows appears in home-linux `list_chat_messages` within 100ms under normal LAN conditions
- Calvin Archive consistency check: after 10 cross-machine runs, `soul.json` on both machines matches within one snapshot cycle

**Done when:**

- A run can start on home-linux (Scout, Coobie, Keeper) and dispatch
  implementation phases to work-windows (Mason, Bramble) transparently via
  Zenoh, with the same `start_run` API call and no operator configuration
  beyond `[remote_machines]` in harkonnen.toml
- PackChat messages are delivered cross-machine in real time; SQLite replicas
  on both machines stay in sync
- Calvin Archive write authority is on home-linux; work-windows receives
  `soul.json` snapshots and uses them for agent boot continuity
- `harkonnen setup check` reports remote machine reachability and falls back
  gracefully if a remote machine is offline
- Buffa proto schema covers all cross-machine wire types; no `serde_json`
  serialisation on the hot path for `SubAgentInput`/`SubAgentResult`

---

## Phase 10 — Documentation, Evaluation, and Lifecycle Benchmarks

**Sequenced after Phase 8** so benchmarks run against the complete system rather than a pre-archive baseline. The DevBench adapter and spec adherence scores are most meaningful once the factory's full memory and governance stack is live. Phase 10 items with no archive dependency (Flint docs, 10-B, 10-C) can begin earlier if capacity allows, but Phase 10 is not a gate on the Calvin Archive path.

**Twin policy note:** Live twin provisioning is not a Phase 10 completion gate. Phase 10-D (twin fidelity benchmark) is optional diagnostic telemetry only — see Twin Policy above.

### 10-A — Flint documentation phase

- After `self.package_artifacts(run_id)` in the Flint phase, call a new `flint_generate_docs` method
- `flint_generate_docs` reads the spec and Mason's implementation artifacts from the run dir, calls the Flint LLM agent to generate a `README.md` and optionally an `API.md`
- Writes output to `artifacts/docs/<run_id>/README.md` and `artifacts/docs/<run_id>/API.md`
- Adds `docs/README.md` to `blackboard.artifact_refs`
- Required for DevBench — must land before the DevBench gate

### 10-B — `src/spec_adherence.rs` — LLM-as-judge benchmark

New builtin benchmark module (follows the same pattern as `cladder.rs`).

- Loads a JSONL file where each line is `{ "run_id": "...", "spec_path": "...", "output_path": "..." }`, OR if no dataset is provided, queries the local SQLite DB for the last N completed runs
- For each entry: reads the spec's `acceptance_criteria` list and Mason's primary output artifact, asks an LLM judge to score each criterion as met/partial/unmet
- Metrics: `completeness` (fraction of criteria met or partial), `precision` (fraction fully met)
- Env: `SPEC_ADHERENCE_DATASET`, `SPEC_ADHERENCE_LIMIT`, `SPEC_ADHERENCE_OUTPUT`, `SPEC_ADHERENCE_MIN_COMPLETENESS`
- Builtin name: `"spec_adherence"`
- Also supports a `without_scout` mode to measure what Scout's formalization step contributes

### 10-C — `src/scenario_delta.rs` — Hidden Scenario Delta benchmark

New builtin benchmark module — Harkonnen-native, no external dataset.

- Queries `coobie_episode_scores` in the local SQLite DB for runs where both `validation_passed` and `scenario_passed` are recorded
- Computes: `visible_pass_rate` (fraction where `validation_passed = 1`), `hidden_pass_rate` (fraction where `scenario_passed = 1`), `delta = visible_pass_rate - hidden_pass_rate`
- A large positive delta means Bramble passes things that Sable catches — proves the hidden scenario value
- Writes `scenario_delta_report.md` and `scenario_delta_summary.json` to artifact dir
- Builtin name: `"scenario_delta"`
- Env: `SCENARIO_DELTA_LIMIT` (max runs to include), `SCENARIO_DELTA_OUTPUT`

### 10-D — `src/twin_fidelity.rs` — Optional twin telemetry benchmark

- Keep `twin_fidelity_score` honest by counting only services whose status is `"running"`
- Retain a Harkonnen-native summary suite for historical comparison and future revisit
- **Not a Phase 10 blocker.** Live twin provisioning is deferred per Twin Policy above.

### 10-E — `suites.yaml` entries

- `harkonnen_spec_adherence` — Spec Adherence Rate (harkonnen-native, builtin: `spec_adherence`)
- `harkonnen_scenario_delta` — Hidden Scenario Delta (harkonnen-native, builtin: `scenario_delta`)
- `harkonnen_twin_fidelity` — Twin Fidelity telemetry (harkonnen-native, builtin: `twin_fidelity`)
- `harkonnen_devbench` — DevBench wrapper suite (script-based external adapter)

### 10-F — DevBench adapter wiring

- Add `scripts/benchmark-devbench.sh` following the same skip-and-delegate pattern as the existing SWE-bench and tau2 wrappers
- `DEVBENCH_COMMAND` supplies the exact local or hosted command that runs Harkonnen on DevBench
- Optional `DEVBENCH_ROOT` points at the benchmark checkout or adapter workspace
- The wrapper exits with skip code `10` when DevBench is not configured so Phase 10 can be wired before the full external harness is installed

### 10-G — Comparative Control-Style Benchmarking

- Add benchmark suites that compare three execution styles on the same tasks where practical: pure-LLM baseline, rule-heavy baseline, and Harkonnen's hybrid pack/control-plane path
- Publish not only task success but also recovery rate, guardrail violation rate, operator interruption count, and time-to-correctness
- Treat this as a factory benchmark, not a model-only benchmark: the question is how safely and efficiently the delivery system moves, not only how strong one model is in isolation

### 10-H — Adversarial Tool-Use And Stakeholder-Alignment Evaluation

- Add adversarial smokes that probe unsafe tool invocation, policy-bypass attempts, MCP misuse, and recovery behavior after intentionally hostile prompts or malformed tool outputs
- Add stakeholder-alignment reporting per run: did the plan respect recorded project purpose, operator stakes, stakeholder attitudes, prohibitions, and approved MCP posture?
- Include a report section that distinguishes technical correctness from project-posture correctness so Harkonnen can fail visibly when it solves the wrong problem the "right" way
- Publish baseline scores for both adversarial resilience and stakeholder-alignment adherence once enough runs exist

**Benchmark gate:**

- `spec_adherence` first run published — completeness and precision against local run corpus
- `scenario_delta` first run published — visible vs hidden pass rate gap across recent runs
- `DevBench` adapter wired (script-based, not builtin)
- comparative control-style benchmark suite wired
- adversarial tool-use smoke suite wired
- stakeholder-alignment reporting visible in run or benchmark artifacts

**Done when:** Flint produces a doc artifact per run, `spec_adherence` and `scenario_delta` have first-run baselines, the DevBench adapter is wired so local or hosted runs can be launched through the benchmark manifest, and the benchmark surface can distinguish pure correctness from governed, stakeholder-aligned correctness.

---

## Parallel Product Track — Calvin Archive Visualizer

**Why this is a prerequisite for Phase 8 debuggability:** The Calvin Archive is a six-chamber typed graph with revision history, quarantine ledger, causal links, and continuous D*/SSA signals. Without a visual surface, failures in integration governance are invisible — you cannot tell whether a quarantine is growing pathologically, whether a chamber is fragmenting, or whether D* drift is localised to one persona axis. The Pack Board's current flat list views cannot represent this structure. If you cannot see the archive, you cannot debug it.

**Reference approach:** [pascalorg/editor](https://github.com/pascalorg/editor) demonstrates the right architectural pattern: a React Three Fiber + Three.js + WebGPU stack rendering a navigable spatial graph where structural regions (levels, in their case) map to distinct visual zones. The six Calvin Archive chambers map directly to that model — each chamber is a navigable region, memory entries are nodes, causal links are edges, quarantine items are visually flagged, and the revision graph is a traversable history layer.

**What to build:**

- **Chamber map view** — six spatial zones (Mythos, Episteme, Ethos, Pathos, Logos, Praxis) rendered as distinct regions in a 3D canvas using React Three Fiber. Nodes within each chamber represent memory entries; edge thickness encodes confidence; quarantine items rendered with a distinct glyph and salience-decay color fade.
- **Causal link traversal** — click a node to expand its inbound/outbound causal links. Link labels show `PearlHierarchyLevel` (Associational / Interventional / Counterfactual). Paths that contributed to a quarantine are highlighted.
- **Revision history rail** — a time-axis rail alongside the chamber map showing integration events (accept / modify / reject / quarantine) as stamped markers. Scrubbing the rail replays chamber state at that point in time using snapshots from TypeDB.
- **Live D\* and SSA overlay** — Materialize SUBSCRIBE feed drives a real-time drift indicator per chamber. Chambers approaching the D\* bound shift color; an alert badge appears when the Meta-Governor fires.
- **Quarantine ledger panel** — side panel listing open quarantine items with pending evidence conditions, salience decay progress, and a one-click "resolve / promote / dismiss" action that calls the Meta-Governor API.
- **PackChat integration** — `@coobie what is in Ethos right now?` routes to a chamber query that highlights matching nodes in the visualizer. The visualizer and PackChat share a run context so Coobie's answers can be spatially anchored.

**Technology notes:**

- React Three Fiber + Three.js is the right rendering layer — WebGPU acceleration optional but worth targeting for large archives.
- Zustand for local visualizer state (selected node, active chamber, time cursor). The archive data itself comes from `GET /api/coobie/query` and the TypeDB query surface from Phase 6.
- The visualizer can be developed independently of the full Calvin Archive backend: stub the data layer with the existing SQLite memory entries and causal links (Phase 4) so the UI can be built and tested before Phase 8 ships.
- Ship as a new Pack Board tab ("Archive") rather than a standalone app — it shares the same auth surface and avoids a separate deployment.

**Dependency:** TypeDB query surface (Phase 6) required for chamber queries and revision history. D*/SSA live overlay requires Materialize (Phase 8). The stub-data path (SQLite causal links) can be used to develop the chamber map and traversal views before those phases land.

**Done when:** An operator can open the Archive tab, navigate the six chambers, click through causal links, scrub the revision history rail, see live D* drift per chamber, and resolve a quarantine item without opening a database client.

---

## Parallel Product Track — Operator Model Activation

**Unlocks:** Better commissioning, fewer mid-run clarification failures, and a reusable operator context layer that Scout, Coobie, and Keeper can all consume.

**Current state:** DB schema is complete (`operator_model_profiles`, `operator_model_sessions`, `operator_model_layer_checkpoints`, `operator_model_entries`, `operator_model_exports`, `operator_model_update_candidates` tables all exist). Phase v1-D shipped the two-layer MVP: project-first session creation, PackChat operator-model threads, layer approval, `commissioning-brief.json` generation, export metadata, and Scout/Coobie consumption. The full five-layer spec follows.

**Full five-layer spec (post-v1):**

- Native PackChat-based elicitation workflow with five fixed layers: operating rhythms, recurring decisions, dependencies, institutional knowledge, friction
- Approval checkpoints after each layer, reusing the existing checkpoint and unblock flow
- Artifact generation for `operating-model.json`, `USER.md`, `SOUL.md`, `HEARTBEAT.md`, `schedule-recommendations.json`, plus a Harkonnen-specific `commissioning-brief.json`
- Scout draft integration so spec generation can use an approved operator model as first-class context
- Coobie preflight integration so operator-model assumptions contribute to `required_checks`, guardrails, and escalation rules

**Current shipped slice:** project-first operator-model resolution now influences Scout draft generation and Coobie preflight guidance. The remaining product work is the checkpoint/export/review loop that turns the interview into durable stamped artifacts with operator approval.

- Review loop after runs: consolidation can propose operator-model updates, which the operator can keep/discard/edit before promotion
- Import/export compatibility with OB1-style operating artifacts, but no direct code dependency on OB1

**Benchmark / product gate:**

- Measurable drop in open checkpoints per run for projects using an approved operator model
- Spec draft quality and spec adherence compared with and without the operator model

**Done when:** A user can complete the five-layer interview with approvals, generate operating artifacts, and see those artifacts materially influence Scout draft quality and Coobie preflight behavior.

---

## Parallel Product Track — Compiled State Synthesis

**Unlocks:** A durable human-readable state surface that compiles accepted run state, coordination outcomes, and memory changes into something an operator can browse without manually reconstructing the story from raw logs and tables.

**Current stance:** build this as a pipeline phase first, not a new Labrador. Coobie provides semantic and causal summaries, Keeper provides authoritative coordination state, and Flint renders the artifact surface. Promote synthesis into its own Labrador only if it becomes a durable bottleneck or needs its own trust boundary.

**What to build:**

- `factory/compiled_docs/` artifact family with at least `run/`, `project/`, and `daily/` outputs
- synthesis job that reads `decision_log`, `phase_attributions`, coordination registry tables, operator model tables, and memory invalidation history
- conflict headers in compiled docs when coordination events or decision records show unresolved tension rather than smoothing it away
- compiled summaries that distinguish current accepted state, superseded state, and open questions
- explicit input provenance so a compiled document can point back to the run, decision, or coordination event that produced each major conclusion

**Benchmark / product gate:**

- A completed run produces a compiled state artifact without manual intervention
- Operators can inspect run/project state from compiled docs without opening SQLite or raw JSON logs
- Contradictions surface as explicit unresolved sections rather than disappearing in prose

**Done when:** a completed run produces a compiled state artifact that summarizes what changed, what was decided, which coordination conflicts occurred, what memory was superseded, and what remains unresolved.

---

## Parallel Product Track — External Integrations

**Unlocks:** The factory becomes observable and controllable from outside the Pack Board. Without outbound notifications, every run outcome requires a human to poll the UI. Without inbound triggers, specs must be started manually. Without auth, the API is open to anyone on the network.

This is a usability prerequisite for any team or multi-machine deployment. Most items are small and independent; they do not need to ship as a block.

### EI-1 — API Authentication

**Why first:** The HTTP API is currently unauthenticated. Every other integration that touches the API needs auth to be safe.

- API key authentication middleware in `src/api.rs` — bearer token checked on all non-health routes
- `api_keys` table in SQLite: `(key_id, key_hash, label, created_at, last_used_at, revoked)`
- `POST /api/auth/keys` (create), `GET /api/auth/keys` (list), `DELETE /api/auth/keys/:id` (revoke)
- `GET /health` and the SSE stream remain unauthenticated (monitoring and browser clients)
- CLI flag `--api-key` or env var `HARKONNEN_API_KEY` for local development bypass

### EI-1b — MCP Authentication And Gateway Policy Parity

**Why next to API auth:** Harkonnen is increasingly MCP-first. API auth without MCP auth leaves the external control surface uneven and undermines Keeper's policy role.

- Authenticated MCP profiles for privileged servers: local-only trusted surfaces remain simple, but remote or high-impact MCP routes require explicit credentials or signed session context
- Gateway policy layer for MCP invocations: approval, deny, and audit outcomes should be symmetrical whether the request arrived over HTTP or MCP
- Policy-aware MCP metadata in setup TOML so machine-local surfaces can remain convenient while shared or remote surfaces become explicitly governed
- Audit trail for MCP decisions: server name, requested tool, approval outcome, actor, and timestamp

### EI-2 — Outbound Webhook Notifications

- `webhooks` table: `(webhook_id, url, secret, events: JSON array, created_at, enabled)`
- `POST /api/webhooks`, `GET /api/webhooks`, `DELETE /api/webhooks/:id`
- Events emitted: `run.started`, `run.completed`, `run.failed`, `checkpoint.created`, `checkpoint.resolved`, `metric_attack.detected`, `consolidation.ready`
- Payload: `{ event, run_id, spec_id, timestamp, summary, pack_board_url }`
- HMAC-SHA256 signature on the `X-Harkonnen-Signature` header (same pattern as GitHub webhooks)
- Retry with exponential backoff on 5xx or connection failure (up to 3 attempts)

### EI-3 — Slack Integration

**Outbound (Slack notifies operator):**

- Rich block-kit messages on `run.completed`: summary card with pass/fail, agent trace count, cost, decision count, link to Pack Board
- Checkpoint alert with inline Approve / Reject buttons that call back to the Harkonnen API
- `metric_attack.detected` alert: which metric was attacked, which exploit fired, suggested mitigation
- `run.failed` with Coobie's top causal diagnosis (from the latest `diagnose` output)

**Inbound (operator controls factory from Slack):**

- Slash command `/harkonnen run <spec-id>` — triggers a run, responds with run ID and Pack Board link
- `/harkonnen status <run-id>` — returns current phase and latest event
- `/harkonnen ask <question>` — routes to Coobie's `dispatch_message` as a PackChat message
- `/harkonnen checkpoint approve <id>` / `reject <id>` — resolves checkpoints without opening the browser

**Config:** Slack app credentials stored in setup TOML under `[integrations.slack]`. Webhook URL and bot token. No hardcoded values.

### EI-4 — Discord Integration

**Outbound:**

- Webhook embeds for `run.completed`, `checkpoint.created`, `run.failed` — same content as Slack but using Discord embed format (color-coded by outcome)
- Thread-per-run option: create a Discord thread for the run and post phase updates as the run progresses

**Inbound (bot commands in a designated channel):**

- `!run <spec-id>` — triggers run
- `!status <run-id>` — current phase and last event
- `!approve <checkpoint-id>` / `!reject <checkpoint-id>`
- `!ask <question>` — routes to Coobie

**Config:** `[integrations.discord]` in setup TOML. Bot token and guild/channel IDs.

### EI-5 — GitHub Integration

**Outbound:**

- After a run completes with Mason edits applied: optionally create a PR from the Mason branch using the GitHub API. PR body includes the spec title, run ID, decision log summary, Coobie critique outcome, and Pack Board link.
- Post a run summary as a PR comment when a run is triggered by a PR webhook (see inbound). Comment includes pass/fail, cost, and the top advisory concern from Coobie.

**Inbound:**

- `POST /api/integrations/github/webhook` receives GitHub webhook events
- On `push` to a configured branch: trigger a spec run for any spec whose `code_under_test` paths overlap the changed files
- On `pull_request.opened` or `pull_request.synchronize`: trigger the relevant spec run and post result as a PR comment
- Webhook secret verified via HMAC (same pattern as EI-2)

**Config:** `[integrations.github]` in setup TOML. Personal access token or GitHub App credentials. Repo and branch filter.

### EI-6 — Run Scheduling

- `scheduled_runs` table: `(schedule_id, spec_id, cron_expression, enabled, last_run_at, next_run_at)`
- `POST /api/schedules`, `GET /api/schedules`, `PUT /api/schedules/:id`, `DELETE /api/schedules/:id`
- Cron evaluator runs on a background tokio task; fires `POST /api/runs` when the schedule triggers
- Pack Board schedule manager panel: add/edit/disable schedules, see last run outcome

### EI-7 — Cost Budget Enforcement

- `max_cost_usd: Option<f64>` on `RunRequest` and in spec YAML
- After each LLM call, `get_run_cost_summary` checks accumulated cost against the budget. If exceeded: abort the current phase gracefully, write a `budget_exceeded` blocker to the blackboard, send a `run.failed` event with reason `budget_exceeded`
- `cost_hard_cap_usd` global config in setup TOML as a safety ceiling above any per-run budget
- Pack Board run overview shows budget consumed vs limit as a progress bar

### EI-8 — Health and Operational Endpoints

**Shipped (2026-04-20):**

- `GET /health` — probes DB (`SELECT 1`) and `memory/index.json`; returns `{ status, version, uptime_secs, db_ok, memory_index_ok }`. Responds `503` if DB probe fails. `AppContext.started_at` tracks server boot time.
- `GET /api/status` — returns `{ active_runs, agent_claim_count, memory_entry_count, last_benchmark_run }`. All queries fail-soft. Authentication deferred to EI-1.

**Remaining:**

- CORS configuration in setup TOML: `[server.cors]` with `allowed_origins` list, defaulting to `localhost` only
- Structured JSON logging option (for log aggregators): `[server.logging] format = "json"` in setup TOML
- Wire `GET /api/status` behind EI-1 auth (viewer role and above)

---

## Parallel Product Track — Hosted And Team Integrations

**Context:** Harkonnen should be usable beyond the Pack Board and local CLI.
This track formalizes the bridge from the local-first factory into external
control planes, workflow tools, shared knowledge systems, and chat surfaces
without hard-coding any one vendor or employer.

The architecture is: Harkonnen exposes itself **as an MCP server** first. That
lets Claude Desktop, Claude Code, VS Code, workflow tools, and any other
MCP-capable client consume factory operations through one protocol instead of
bespoke per-client integrations. EI-1 (auth) should land first because every
hosted or shared surface needs it.

### ENT-1 — Harkonnen as an MCP Server

**What to build:**

- `src/mcp_server.rs` — implements the MCP server protocol (JSON-RPC 2.0 over stdio or SSE transport). The MCP spec has a Rust SDK (`rmcp` crate); use that rather than writing the transport layer by hand.
- **Resources** (read-only, queryable by clients):
  - `harkonnen://runs` — list of recent runs with status
  - `harkonnen://runs/{run_id}` — full run detail including traces, decisions, optimization program, metric attacks
  - `harkonnen://memory/lessons` — promoted lessons from the consolidation workbench
  - `harkonnen://memory/causal` — recent causal patterns Coobie has identified
  - `harkonnen://specs` — available specs for commissioning
- **Tools** (callable actions):
  - `run_spec(spec_id, options)` — triggers a factory run, returns run_id
  - `get_run_status(run_id)` — current phase + latest event
  - `resolve_checkpoint(checkpoint_id, decision, note)` — approve or reject a checkpoint from any MCP client
  - `ask_coobie(question, context)` — routes to Coobie's `dispatch_message`, returns the response
  - `ingest_memory(content, source, tags)` — pushes a document or note into Coobie's memory ingest pipeline
  - `list_decisions(run_id)` — returns the decision audit log for a run
- **Prompts** (parameterized prompt templates for external clients):
  - `briefing_for_spec(spec_id)` — pre-built Coobie briefing prompt
  - `diagnose_run(run_id)` — causal diagnosis prompt for a completed run
- MCP server transport registered in setup TOML under `[mcp.self]`:

```toml
[mcp.self]
enabled = true
transport = "sse"          # "stdio" for Claude Desktop / VS Code; "sse" for hosted clients
port = 3001                # separate port from the main HTTP API
auth_required = true       # reuses EI-1 API key
```

- CLI command `harkonnen mcp serve` starts the MCP server as a standalone process alongside the main server

**Done when:** Claude Desktop, Claude Code, or VS Code can list factory runs, trigger a run, and ask Coobie a question via MCP tool calls.

### ENT-2 — External Connector Surface

- `factory/connectors/harkonnen-openapi.json` — OpenAPI 3.0 spec covering the key Harkonnen endpoints
- `factory/connectors/manifest.yaml` — connector manifest with display names, descriptions, and action categories
- `factory/connectors/workflow-templates/` — starter workflow templates: `run-spec.yaml`, `ask-coobie.yaml`, `checkpoint-review.yaml`
- Authentication: OAuth2 client credentials flow using the generic OIDC path from ENT-3
- Documentation at `factory/connectors/README.md`

**Done when:** An external workflow client can trigger a Harkonnen run, ask Coobie a question, and approve a checkpoint without touching the Pack Board.

### ENT-3 — OIDC Authentication

- OAuth2/OIDC JWT validation middleware in `src/api.rs` — alongside the existing API key path
- `[auth.oidc]` section in setup TOML with `issuer`, `client_id`, `audience`
- JWT validation: fetch JWKS from the configured issuer, validate signature, `aud`, `iss`, and expiry
- Role claims: `Harkonnen.Operator` (full access), `Harkonnen.Viewer` (read-only), `Harkonnen.Agent` (service principal)
- `GET /api/auth/me` — returns the authenticated identity and resolved role

**Done when:** An external connector authenticating as an OIDC service principal can call the Harkonnen API without an API key, and a viewer-role principal cannot trigger runs or approve checkpoints.

### ENT-4 — Knowledge Base Ingest

- `src/integrations/knowledge.rs` — generic knowledge-source client layer with provider adapters
- CLI: `harkonnen memory ingest --source docs --collection <id>`, `--source wiki --space <id>`, `--source search --query "<terms>"`
- Incremental sync state table so repeated runs fetch only changed or added documents
- `[integrations.knowledge]` section in setup TOML

**Done when:** Running a knowledge-source ingest adds shared documents into Coobie's retrievable memory, and subsequent runs against related specs can cite those documents in the briefing.

### ENT-5 — ChatOps Integration

- Rich-card messages on run events (completed, failed, checkpoint, metric attack) to a configured chat surface
- Checkpoint actionable card — operator clicks Approve/Reject directly in chat
- Bot commands: `@Harkonnen run`, `@Harkonnen status`, `@Harkonnen ask`, `@Harkonnen checkpoints`
- `[integrations.chatops]` in setup TOML with generic webhook + bot credential fields

**Done when:** A completed run posts a rich message to the configured chat surface, a checkpoint can be approved from chat, and `@Harkonnen ask` routes to Coobie.

### ENT-6 — Clone-Local Profile And Hosted Deployment Hardening

- Keep generated machine profiles under `setups/machines/` and out of Git by default
- Add optional `[auth.oidc]`, `[integrations.chatops]`, `[integrations.knowledge]`, and `[mcp.self]` blocks to generated profiles when the setup interview selects them
- `cargo run -- setup check` extended to validate selected integrations and MCP self-server startup for the active local profile

**Done when:** Running `cargo run -- setup check` on a locally generated profile reports green for the selected integrations, and a second machine can be provisioned from the same public templates without inheriting private state.

---

## Benchmark Track (cross-phase)

Benchmarks advance in lockstep with implementation phases. Each phase ships with at least one measurable gate.

### Phase-aligned milestones summary

| Phase | Key benchmarks unlocked |
| --- | --- |
| v1 | Decision audit completeness, memory supersession accuracy (StreamingQA), WrongAnswer classification rate |
| Phase 2 | SWE-bench Verified readiness, LiveCodeBench, Aider Polyglot |
| Phase 5-C | Briefing scope log per run (correctness verification, not a scored benchmark) |
| Phase 5-D | PackChat-to-OB1 candidate capture and retrieval smoke; Calvin promotion contract smoke |
| Phase 5b | FRAMES re-run (OB1 default recall), LongMemEval / LoCoMo regression check |
| Phase 6 | GAIA Level 3, AgentBench, TypeDB vs SQL causal recall comparison |
| Phase 7 | E-CARE, causal attribution accuracy (top-1 / top-3) |
| Phase 8 | D* drift score, SSA baseline, quarantine resolution quality, schema revision stability |
| Phase 10 | Spec adherence rate, hidden scenario delta, DevBench |

### Always-on benchmarks

- `Local Regression Gate` — hard merge gate, runs on every substantial change
- `LongMemEval` paired mode (Coobie vs raw LLM) — run on every memory-relevant change
- `LoCoMo QA` paired mode — longer-horizon memory regression check

### Competitive positioning benchmarks

#### vs Mem0 / MindPalace / Zep

- `FRAMES` — multi-hop factual recall; Mem0 publishes here. Native adapter live. Re-run after OB1 becomes the default shared recall path and compare against the local-vector baseline.
- `StreamingQA` — belief-update accuracy; no competitor tracks this. Phase v1-B.
- `HELMET` — retrieval precision/recall. Native adapter live.
- `LongMemEval` — long-term assistant memory. Native adapter live.
- `LoCoMo QA` — long-horizon dialogue memory. Native adapter live.

#### vs OpenCode / Aider / single-agent coding tools

- `LiveCodeBench` — recent competitive programming problems; contamination-resistant. Phase 2.
- `Aider Polyglot` — Aider's own multi-language leaderboard. Phase 2.
- `DevBench` — full software lifecycle; structural argument against single-phase tools. Phase 10.
- `SWE-bench Verified` / `SWE-bench Pro` — industry-standard code loop benchmarks. Phase 2.

#### vs general agent frameworks

- `GAIA Level 3` — multi-step delegation; single-agent tools fail here. Phase 6.
- `AgentBench` — eight environments; tests Labrador role separation. Phase 6.

#### Causal reasoning — unique claim, no competitor benchmarks this

- `CLADDER` — Pearl hierarchy accuracy. Native adapter live.
- `E-CARE` — causal explanation coherence. Phase 7.

#### Harkonnen-native — cannot be run by any competitor

- `Spec Adherence Rate` — completeness and precision vs spec. Phase 10.
- `Hidden Scenario Delta` — visible vs hidden pass rate gap. Phase 10.
- `Causal Attribution Accuracy` — seeded failure corpus, top-1 / top-3. Phase 7.

### Reporting standard

Every reportable benchmark claim should include:

- the raw-LLM baseline on the same provider when meaningful
- the Harkonnen setup name and routing
- the benchmark split or slice used
- the commit hash and benchmark artifact path
- latency and cost where available, not just accuracy

---

## What is already done (do not redo)

**Gap-closure phases A–D (shipped 2026-04-18):**

- **A1** — `LlmUsage` struct; token + latency capture on all three providers; `run_cost_events` table; `GET /api/runs/:id/cost`
- **A2** — `DecisionRecord` struct; `decision_log` table; `record_decision` + `list_run_decisions`; `GET /api/runs/:id/decisions`; wired at plan critique and consolidation promotion
- **A3** — `Assignment` + `ClaimRequest` extended with `resource_kind`, `ttl_secs`, `guardrails`, `expires_at`; `POST /api/coordination/check-lease` handler with TTL expiry and guardrail pattern matching
- **B** — `AgentTrace` struct; `agent_traces` table + index; `record_agent_trace` + `list_run_traces`; `extract_reasoning()` parses `<reasoning>` blocks; wired at Scout intake, Coobie briefing, Coobie critique, Mason plan, Mason edits, Sable; `GET /api/runs/:id/traces`
- **C** — `OptimizationProgram` struct; `scout_derive_optimization_program` (LLM-backed, stub fallback); written to `optimization_program.json`; Coobie critique flags when plan doesn't address objective metric; `GET /api/runs/:id/optimization-program`
- **D** — `MetricAttack` struct; `sable_generate_metric_attacks` (2–3 attacks per run, exploit + detection signals + mitigations); written to `metric_attacks.json`; `GET /api/runs/:id/metric-attacks`

---

**Phase 1 — Core Factory + PackChat + Coobie Memory + Benchmark Toolchain:**

- Spec loading, validation, run lifecycle, SQLite persistence
- Phase-level attribution recording
- LLM routing for Claude, Gemini, OpenAI, and OpenAI-compatible local endpoints
- Scout, Mason, Piper, Sable, Ash, Flint LLM calls
- Mason opt-in file writes with staged workspace isolation
- Piper real build execution with stdout/stderr streaming
- Mason fix loop (up to 3 iterations on build failure)
- Live event broadcast (`LiveEvent`) + SSE endpoint `/api/runs/:id/events/stream`
- Coobie causal reasoning Phase 1 (heuristic rules, episode scoring)
- Coobie causal streaks and cross-run pattern detection
- Coobie Phase 3 preflight guidance (spec-scoped cause history → required checks)
- Coobie Palace (`src/coobie_palace.rs`) — den-based compound recall, patrol, scents
- Semantic memory (Open Brain / OB1 by default; fastembed or OpenAI-compatible embeddings + SQLite vector store optional)
- Causal feedback loop (causal reports + Sable rationale written back to project memory)
- Keeper coordination API (claims, heartbeats, conflict detection, release)
- Pack Board React UI (PackChat panel, Attribution Board, Factory Floor, Memory Board)
- PackChat backend: `chat_threads`, `chat_messages`, multi-turn `dispatch_message`, `@mention` routing, Coobie default fallback
- PackChat API routes: `GET/POST /api/chat/threads`, `GET /api/chat/threads/:id`, `GET/POST /api/chat/threads/:id/messages`, `POST /api/agents/:id/chat`
- Checkpoint/reply/unblock routes as PackChat control-plane backend
- Evidence bootstrap, annotation bundle validation, evidence promotion
- `harkonnen memory init` with pre-embedding on fresh clone
- First-class benchmark toolchain (`benchmark list/run/report`, manifest-driven suites, CI workflow)
- Native LongMemEval adapter + paired raw-LLM vs Harkonnen comparison mode
- Native LoCoMo QA adapter + paired raw-LLM vs Harkonnen comparison mode
- Native FRAMES adapter + paired raw-LLM vs Harkonnen comparison mode
- Native StreamingQA adapter (query-time invalidation reasons plus persisted-history smoke published on `lm-studio-local`)
- LM Studio / OpenAI-compatible benchmark routing for chat and embedding backends

**Phase 4 — Episodic Layer Enrichment + Causal Graph + Benchmarks:**

- `state_before` / `state_after` on `EpisodeRecord` and episodes table (workspace state snapshots via FNV-64 hash walk)
- `causal_links` table: `(link_id, run_id, from_event_id, to_event_id, relation, confidence, hierarchy_level, key, created_at)`
- `PearlHierarchyLevel` enum (Associational / Interventional / Counterfactual) on causal links
- `populate_cross_phase_causal_links` — auto-emits phase_sequence and failure_triggered links across run episodes
- `get_run_causal_graph` — returns event graph with Pearl-labeled edges; surfaced via `GET /api/runs/:id/causal-events`
- Coobie multi-hop retrieval: `retrieve_context_multihop(query, embedding_store, depth)` — configurable chain depth (1–3)
- Native CLADDER adapter — Pearl hierarchy causal benchmark, paired Harkonnen vs raw-LLM mode
- Native HELMET adapter — retrieval precision/recall benchmark

**Phase 4b — Memory Invalidation (query-time layer shipped; persistence layer now live on the main ingest path):**

- `invalidation_reasons` field on `MemoryRetrievalHit` — computed at retrieval time from `superseded_by` / `challenged_by` provenance fields
- `memory_invalidation_reasons()` helper in orchestrator surfaces reasons per hit
- Persistence layer (`memory_updates` table, `invalidated_by` / `superseded_by` provenance, `GET /api/memory/updates`, Memory Board panel) is live; StreamingQA persisted-history smoke is published, and the operator confirm/reject loop is now part of the shipped path

**Phase 5 — Consolidation Workbench:**

- `consolidation_candidates` table: `(candidate_id, run_id, kind, status, content_json, edited_json, confidence, label, created_at, reviewed_at)`
- `generate_consolidation_candidates`, `list_consolidation_candidates`, `review_consolidation_candidate`, `edit_consolidation_candidate`, `promote_kept_candidates`
- INSERT OR IGNORE idempotency on candidate generation
- API routes: `GET /api/runs/:id/consolidation/candidates`, `POST .../candidates` (generate), `POST .../candidates/:id/keep`, `.../discard`, `.../edit`, `POST /api/runs/:id/consolidate` (promote)
- Pack Board Consolidation Workbench panel: candidate cards with keep/discard/edit controls, confidence bars, expandable JSON, filter bar, promote footer
- `RunDetailDrawer` updated with workbench tab

---

## Deferred Until Fully Working

These are useful capabilities, but they must not block the current path to a fully working Harkonnen loop. Revisit them only after the PackChat -> OB1 -> Calvin memory chain, learning-loop closure, run health, MCP prompt surface, provider routing, and TypeDB-backed causal query path are productive.

### Constraint-polytope exploration (Avis-Fukuda / Double Description)

The bounded `f64` Double Description sketch should stay outside the active Harkonnen crate until Harkonnen has a concrete use for constraint-polytopes in governance, adaptation safety, or causal feasibility checks.

Potential future uses:

- Test whether a proposed policy, identity, or adaptation constraint leaves a non-empty feasible region.
- Give the Meta-Governor a geometric feasibility check before accepting new invariants.
- Model bounded causal or benchmark constraints where a vertex representation is more useful than a prose rule.

Do not wire this into Phase 5b, PackChat, OB1, Calvin write integrity, or the TypeDB causal path until the end-to-end system is productive. If a Harkonnen caller emerges, bring it back as a standalone crate or intentionally scoped module with fresh tests.

### Scanned document ingestion (`pdfium-render` + vision/OCR)

The existing `memory ingest` path can remain text-first for now. Image-only PDFs and scanned documents move behind the fully-working milestone.

Future implementation shape:

```text
memory ingest <file.pdf>
  │
  ├─ pdfium-render → text layer present?
  │     yes → extract directly
  │
  └─ no text layer
        │
        ├─ pdfium-render → rasterize pages to images
        │
        ├─ vision API → extract text per page
        │     structured output: { page, text, confidence }
        │
        └─ offline OCR fallback if needed
```

When this returns to the active roadmap, add confidence/provenance fields so downstream retrieval can distinguish text-layer extraction from vision/OCR extraction. Do not add `pdfium-render`, Tesseract work, or scanned-PDF gates to Phase 5b.

---

## Tracking

Each active implementation phase gets its own git branch: `phase/v1-guardrails`, `phase/2-bramble-tests`, `phase/5c-briefing-scope`, etc.
A phase is merged to `main` when its "Done when" condition is verifiably met.
This file is updated when a phase ships — move it from the numbered list above into the "already done" section.

Benchmark wiring should advance in lockstep with implementation:

- when a phase ships, add or tighten at least one benchmark gate tied to it
- when a public benchmark is still adapter-only, capture that explicitly here rather than implying it is fully integrated
- benchmark artifacts belong in `factory/artifacts/benchmarks/` and should be linked from release notes once they support a public claim
