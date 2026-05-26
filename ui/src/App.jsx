import React, { useEffect, useState } from 'react';
import AgentCard from './components/AgentCard';
import CoobieSignalPanel from './components/CoobieSignalPanel';
import CausalTesseract from './visualization/tesseract/CausalTesseract';
import CausalWorkbench from './components/CausalWorkbench';
import FactoryFloor from './components/FactoryFloor';
import AttributionPanel from './components/AttributionPanel';
import OperationsDeck from './components/OperationsDeck';
import PackChat from './components/PackChat';

const API_BASE = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:3057/api';
const API_ORIGIN = API_BASE.replace(/\/api\/?$/, '');

// Agent roster — source of truth, kept in sync with AGENTS.md
// pinned: 'claude' means trust-critical, always routed to Claude
const AGENT_DEFS = [
  {
    id: 'scout', name: 'Scout', role: 'Spec Retriever', group: 'planning', accentColor: '#c4922a',
    pinned: 'claude',
    desc: 'Parse specs, flag ambiguity, produce intent package',
  },
  {
    id: 'keeper', name: 'Keeper', role: 'Boundary Retriever', group: 'planning', accentColor: '#8a7a3a',
    pinned: 'claude',
    desc: 'Enforce policy, guard boundaries, and manage file-claim coordination',
  },
  {
    id: 'mason', name: 'Mason', role: 'Build Retriever', group: 'action', accentColor: '#c4662a',
    pinned: null,
    desc: 'Generate and modify code, multi-file changes',
  },
  {
    id: 'piper', name: 'Piper', role: 'Tool Retriever', group: 'action', accentColor: '#5a7a5a',
    pinned: null,
    desc: 'Run build tools, fetch docs, execute helpers',
  },
  {
    id: 'ash', name: 'Ash', role: 'Twin Retriever', group: 'action', accentColor: '#2a7a7a',
    pinned: null,
    desc: 'Provision digital twins, mock dependencies',
  },
  {
    id: 'bramble', name: 'Bramble', role: 'Test Retriever', group: 'verification', accentColor: '#a89a2a',
    pinned: null,
    desc: 'Generate tests, run lint/build/visible tests',
  },
  {
    id: 'sable', name: 'Sable', role: 'Scenario Retriever', group: 'verification', accentColor: '#3a4a5a',
    pinned: 'claude',
    desc: 'Execute hidden scenarios, produce eval reports',
  },
  {
    id: 'flint', name: 'Flint', role: 'Artifact Retriever', group: 'verification', accentColor: '#8a6a3a',
    pinned: null,
    desc: 'Collect outputs, package artifact bundles',
  },
  {
    id: 'coobie', name: 'Coobie', role: 'Memory Retriever', group: 'memory', accentColor: '#7a2a3a',
    pinned: null,
    desc: 'Coordinate pack memory: working context, episodic capture, causal graph, consolidation, and cross-agent blackboard health',
  },
];

function titleCase(value) {
  if (!value) {
    return 'idle';
  }
  return value
    .replaceAll('_', ' ')
    .split(' ')
    .filter(Boolean)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(' ');
}

function normalizeStatus(status, ownership) {
  const normalized = (status || '').toLowerCase();
  if (normalized === 'warning' || normalized === 'failed') {
    return 'blocked';
  }
  if (normalized === 'complete') {
    return 'complete';
  }
  if (normalized === 'running') {
    return 'running';
  }
  if (ownership) {
    return 'running';
  }
  return 'idle';
}

function shortHash(value) {
  if (!value) {
    return 'untracked';
  }
  return value.length > 12 ? value.slice(0, 12) : value;
}

function memoryBlockEntries(agentState) {
  return Object.entries(agentState?.memory_block_ids || {}).sort(([left], [right]) => left.localeCompare(right));
}

function graphCount(value) {
  return Number.isFinite(Number(value)) ? Number(value) : 0;
}

function deriveAgents(events, blackboard, executions) {
  const claims = blackboard?.agent_claims || {};
  const executionMap = Object.fromEntries(
    (executions || []).map((execution) => [execution.agent_name.toLowerCase(), execution]),
  );

  return AGENT_DEFS.map((definition) => {
    const agentEvents = (events || []).filter(
      (event) => event.agent.toLowerCase() === definition.id.toLowerCase(),
    );
    const latest = agentEvents.length > 0 ? agentEvents[agentEvents.length - 1] : null;
    const execution = executionMap[definition.id.toLowerCase()];
    const ownership = claims[definition.id] || '';

    return {
      ...definition,
      status: normalizeStatus(latest?.status, ownership),
      task: ownership || latest?.message || `Awaiting ${definition.role.toLowerCase()}`,
      latestLog: latest
        ? `${titleCase(latest.phase)} · ${latest.message}`
        : 'Ready for the next run.',
      latestPhase: latest ? titleCase(latest.phase) : 'Awaiting signal',
      ownership,
      engine: execution ? `${execution.provider}/${execution.model}` : 'unassigned',
      bundleProvider: execution?.prompt_bundle_provider || null,
      bundleFingerprint: execution?.prompt_bundle_fingerprint || null,
      pinnedSkillIds: execution?.pinned_skill_ids || [],
      desc: definition.desc,
      pinned: definition.pinned,
    };
  });
}

async function fetchJson(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText}`);
  }
  return response.json();
}

async function postJson(url, body) {
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `${response.status} ${response.statusText}`);
  }
  return response.json();
}

function Panel({ title, children, compact = false }) {
  return (
    <section className={`ops-panel ${compact ? 'compact' : ''}`}>
      <div className="panel-title-row">
        <h3>{title}</h3>
        <div className="panel-line"></div>
      </div>
      {children}
    </section>
  );
}

function App() {
  const [runs, setRuns] = useState([]);
  const [activeRunId, setActiveRunId] = useState('');
  const [runState, setRunState] = useState(null);
  const [memoryUpdates, setMemoryUpdates] = useState([]);
  const [agentState, setAgentState] = useState([]);
  const [causalGraphStatus, setCausalGraphStatus] = useState(null);
  const [causalGraphProjection, setCausalGraphProjection] = useState(null);
  const [causalFailureHistory, setCausalFailureHistory] = useState(null);
  const [causalFailureExport, setCausalFailureExport] = useState(null);
  const [e2eReadinessIndex, setE2eReadinessIndex] = useState(null);
  const [e2eReadiness, setE2eReadiness] = useState(null);
  const [materializedE2eReadiness, setMaterializedE2eReadiness] = useState(null);
  const [materializingE2eReadiness, setMaterializingE2eReadiness] = useState(false);
  const [materializedE2eManifest, setMaterializedE2eManifest] = useState(null);
  const [materializingE2eManifest, setMaterializingE2eManifest] = useState(false);
  const [materializedE2eBundle, setMaterializedE2eBundle] = useState(null);
  const [materializingE2eBundle, setMaterializingE2eBundle] = useState(false);
  const [materializedCausalExport, setMaterializedCausalExport] = useState(null);
  const [materializingCausalExport, setMaterializingCausalExport] = useState(false);
  const [reviewingMemoryUpdateId, setReviewingMemoryUpdateId] = useState('');
  const [selectedRole, setSelectedRole] = useState('mason');
  const [roleBoard, setRoleBoard] = useState(null);
  const [coordination, setCoordination] = useState(null);
  const [policyEvents, setPolicyEvents] = useState([]);
  const [capacity, setCapacity] = useState(null);
  const [showTesseract, setShowTesseract] = useState(false);
  const [showWorkbench, setShowWorkbench] = useState(false);
  const [showSystem, setShowSystem] = useState(false);
  const [causalReport, setCausalReport] = useState(null);
  const [error, setError] = useState('');

  const loadMemoryUpdates = async () => {
    try {
      const data = await fetchJson(`${API_BASE}/memory/updates`);
      setMemoryUpdates(Array.isArray(data) ? data : []);
    } catch {
      setMemoryUpdates([]);
    }
  };

  const reviewMemoryUpdate = async (updateId, status) => {
    setReviewingMemoryUpdateId(updateId);
    try {
      await postJson(`${API_BASE}/memory/updates/${updateId}/review`, {
        status,
        reviewed_by: 'operator',
      });
      await loadMemoryUpdates();
      setError('');
    } catch (reviewError) {
      setError(reviewError.message);
    } finally {
      setReviewingMemoryUpdateId('');
    }
  };

  useEffect(() => {
    let cancelled = false;

    const loadRuns = async () => {
      try {
        const data = await fetchJson(`${API_BASE}/runs`);
        if (cancelled) {
          return;
        }
        setRuns(data);
        setActiveRunId((current) => {
          if (!data.length) {
            return '';
          }
          if (current && data.some((run) => run.run_id === current)) {
            return current;
          }
          return data[0].run_id;
        });
        setError('');
      } catch (fetchError) {
        if (!cancelled) {
          setError(fetchError.message);
        }
      }
    };

    loadRuns();
    const interval = setInterval(loadRuns, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  useEffect(() => {
    if (!activeRunId) {
      setRunState(null);
      return undefined;
    }

    let cancelled = false;

    const loadState = async () => {
      try {
        const data = await fetchJson(`${API_BASE}/runs/${activeRunId}/state`);
        if (!cancelled) {
          setRunState(data);
          setError('');
        }
      } catch (fetchError) {
        if (!cancelled) {
          setError(fetchError.message);
        }
      }
    };

    loadState();
    const interval = setInterval(loadState, 1500);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [activeRunId]);

  useEffect(() => {
    if (!activeRunId || !selectedRole) {
      setRoleBoard(null);
      return undefined;
    }

    let cancelled = false;

    const loadRoleBoard = async () => {
      try {
        const data = await fetchJson(`${API_BASE}/runs/${activeRunId}/blackboard/${selectedRole}`);
        if (!cancelled) {
          setRoleBoard(data);
        }
      } catch (fetchError) {
        if (!cancelled) {
          setRoleBoard(null);
        }
      }
    };

    loadRoleBoard();
    const interval = setInterval(loadRoleBoard, 1500);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [activeRunId, selectedRole]);

  useEffect(() => {
    let cancelled = false;

    const loadCoordination = async () => {
      try {
        const [assignments, events] = await Promise.all([
          fetchJson(`${API_BASE}/coordination/assignments`),
          fetchJson(`${API_BASE}/coordination/policy-events`),
        ]);
        if (!cancelled) {
          setCoordination(assignments);
          setPolicyEvents(Array.isArray(events) ? events : []);
        }
      } catch (fetchError) {
        if (!cancelled) {
          setCoordination(null);
          setPolicyEvents([]);
        }
      }
    };

    loadCoordination();
    const interval = setInterval(loadCoordination, 2000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    const loadAgentState = async () => {
      try {
        const data = await fetchJson(`${API_BASE}/agent-state`);
        if (!cancelled) {
          setAgentState(Array.isArray(data) ? data : []);
        }
      } catch {
        if (!cancelled) {
          setAgentState([]);
        }
      }
    };

    loadAgentState();
    const interval = setInterval(loadAgentState, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    const loadGraphStatus = async () => {
      try {
        const [graphStatus, readinessIndex] = await Promise.allSettled([
          fetchJson(`${API_BASE}/causal-graph/status`),
          fetchJson(`${API_BASE}/e2e-readiness`),
        ]);
        if (!cancelled) {
          setCausalGraphStatus(graphStatus.status === 'fulfilled' ? graphStatus.value : null);
          setE2eReadinessIndex(readinessIndex.status === 'fulfilled' ? readinessIndex.value : null);
        }
      } catch {
        if (!cancelled) {
          setCausalGraphStatus(null);
          setE2eReadinessIndex(null);
        }
      }
    };

    loadGraphStatus();
    const interval = setInterval(loadGraphStatus, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  useEffect(() => {
    if (!activeRunId) {
      setCausalGraphProjection(null);
      setCausalFailureHistory(null);
      setCausalFailureExport(null);
      setE2eReadiness(null);
      return undefined;
    }

    let cancelled = false;
    const loadCausalGraphRunData = async () => {
      try {
        const [projection, history, replayExport, readiness, evidenceBundle] = await Promise.allSettled([
          fetchJson(`${API_BASE}/runs/${activeRunId}/causal-graph-projection`),
          fetchJson(`${API_BASE}/runs/${activeRunId}/causal-failure-history`),
          fetchJson(`${API_BASE}/runs/${activeRunId}/causal-failure-history/export`),
          fetchJson(`${API_BASE}/runs/${activeRunId}/e2e-readiness`),
          fetchJson(`${API_BASE}/runs/${activeRunId}/e2e-evidence-bundle`),
        ]);
        if (!cancelled) {
          setCausalGraphProjection(projection.status === 'fulfilled' ? projection.value : null);
          setCausalFailureHistory(history.status === 'fulfilled' ? history.value : null);
          setCausalFailureExport(replayExport.status === 'fulfilled' ? replayExport.value : null);
          setE2eReadiness(readiness.status === 'fulfilled' ? readiness.value : null);
          setMaterializedE2eBundle(evidenceBundle.status === 'fulfilled' ? evidenceBundle.value : null);
        }
      } catch {
        if (!cancelled) {
          setCausalGraphProjection(null);
          setCausalFailureHistory(null);
          setCausalFailureExport(null);
          setE2eReadiness(null);
          setMaterializedE2eBundle(null);
        }
      }
    };

    loadCausalGraphRunData();
    const interval = setInterval(loadCausalGraphRunData, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [activeRunId]);

  useEffect(() => {
    setMaterializedE2eReadiness(null);
    setMaterializedE2eManifest(null);
    setMaterializedE2eBundle(null);
    setMaterializedCausalExport(null);
  }, [activeRunId]);

  const refreshEvidenceSurfaces = async () => {
    if (!activeRunId) {
      return;
    }
    const [state, readiness, readinessIndex] = await Promise.allSettled([
      fetchJson(`${API_BASE}/runs/${activeRunId}/state`),
      fetchJson(`${API_BASE}/runs/${activeRunId}/e2e-readiness`),
      fetchJson(`${API_BASE}/e2e-readiness`),
    ]);
    if (state.status === 'fulfilled') {
      setRunState(state.value);
    }
    if (readiness.status === 'fulfilled') {
      setE2eReadiness(readiness.value);
    }
    if (readinessIndex.status === 'fulfilled') {
      setE2eReadinessIndex(readinessIndex.value);
    }
  };

  const materializeCausalExport = async () => {
    if (!activeRunId) {
      return;
    }
    setMaterializingCausalExport(true);
    try {
      const response = await postJson(`${API_BASE}/runs/${activeRunId}/causal-failure-history/export/materialize`, {});
      setMaterializedCausalExport(response);
      await refreshEvidenceSurfaces();
      setError('');
    } catch (materializeError) {
      setError(materializeError.message);
    } finally {
      setMaterializingCausalExport(false);
    }
  };

  const materializeE2eReadiness = async () => {
    if (!activeRunId) {
      return;
    }
    setMaterializingE2eReadiness(true);
    try {
      const response = await postJson(`${API_BASE}/runs/${activeRunId}/e2e-readiness/materialize`, {});
      setMaterializedE2eReadiness(response);
      await refreshEvidenceSurfaces();
      setError('');
    } catch (materializeError) {
      setError(materializeError.message);
    } finally {
      setMaterializingE2eReadiness(false);
    }
  };

  const materializeE2eManifest = async () => {
    if (!activeRunId) {
      return;
    }
    setMaterializingE2eManifest(true);
    try {
      const response = await postJson(`${API_BASE}/runs/${activeRunId}/e2e-evidence-manifest/materialize`, {});
      setMaterializedE2eManifest(response);
      await refreshEvidenceSurfaces();
      setError('');
    } catch (materializeError) {
      setError(materializeError.message);
    } finally {
      setMaterializingE2eManifest(false);
    }
  };

  const materializeE2eBundle = async () => {
    if (!activeRunId) {
      return;
    }
    setMaterializingE2eBundle(true);
    try {
      const response = await postJson(`${API_BASE}/runs/${activeRunId}/e2e-evidence-bundle/materialize`, {});
      setMaterializedE2eBundle(response);
      await refreshEvidenceSurfaces();
      setError('');
    } catch (materializeError) {
      setError(materializeError.message);
    } finally {
      setMaterializingE2eBundle(false);
    }
  };

  useEffect(() => {
    let cancelled = false;

    const loadCapacity = async () => {
      try {
        const data = await fetchJson(`${API_BASE}/capacity`);
        if (!cancelled) setCapacity(data);
      } catch { /* capacity.json optional */ }
    };

    loadCapacity();
    const interval = setInterval(loadCapacity, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    const loadMemoryUpdates = async () => {
      try {
        const data = await fetchJson(`${API_BASE}/memory/updates`);
        if (!cancelled) {
          setMemoryUpdates(Array.isArray(data) ? data : []);
        }
      } catch {
        if (!cancelled) {
          setMemoryUpdates([]);
        }
      }
    };

    loadMemoryUpdates();
    const interval = setInterval(loadMemoryUpdates, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  // Fetch causal report for active run (used by workbench)
  useEffect(() => {
    if (!activeRunId) { setCausalReport(null); return undefined; }
    let cancelled = false;
    const load = async () => {
      try {
        const data = await fetchJson(`${API_BASE}/runs/${activeRunId}/causal-report`);
        if (!cancelled) setCausalReport(data);
      } catch { if (!cancelled) setCausalReport(null); }
    };
    load();
    return () => { cancelled = true; };
  }, [activeRunId]);

  const run = runState?.run || null;
  const events = runState?.events || [];
  const blackboard = runState?.blackboard || null;
  const lessons = runState?.lessons || [];
  const agentExecutions = runState?.agent_executions || [];
  const phaseAttributions = runState?.phase_attributions || [];
  const coobieTranslations = runState?.coobie_translations || [];
  const agents = deriveAgents(events, blackboard, agentExecutions);
  const agentStateByName = Object.fromEntries(
    (agentState || []).map((state) => [state.agent_name?.toLowerCase(), state]),
  );
  const canonicalAgentState = AGENT_DEFS
    .map((definition) => ({
      ...definition,
      state: agentStateByName[definition.id],
    }))
    .filter((entry) => entry.state);
  const planningAgents = agents.filter((agent) => agent.group === 'planning');
  const actionAgents = agents.filter((agent) => agent.group === 'action');
  const verificationAgents = agents.filter((agent) => agent.group === 'verification');
  const memoryAgent = agents.find((agent) => agent.group === 'memory');
  const recentEvents = [...events].slice(-14).reverse();
  const activeThreads = agents.filter((agent) => agent.status === 'running');
  const roleClaims = Object.entries(roleBoard?.agent_claims || {});
  const coordinationClaims = Object.entries(coordination?.active || {});
  const staleClaims = coordinationClaims.filter(([, claim]) => claim.status === 'stale');
  const healthyClaims = coordinationClaims.filter(([, claim]) => claim.status !== 'stale');
  const recentPolicyEvents = [...policyEvents].slice(-6).reverse();
  const recentMemoryUpdates = [...memoryUpdates].slice(0, 6);

  return (
    <div className="pack-board-shell">
      <header className="run-header glass-panel">
        <div>
          <div className="eyebrow">Harkonnen Labs · Pack Board</div>
          <h1>{run ? `${run.product} · ${run.spec_id}` : 'The pack is ready.'}</h1>
          <div className="header-meta">
            {run && <span>Run: {run.run_id.slice(0, 8)}</span>}
            {run && <span>Phase: {titleCase(blackboard?.current_phase || run.status || 'idle')}</span>}
            <span className={`status-pill status-${run?.status || 'idle'}`}>{run?.status || 'idle'}</span>
            <div className="pack-pins">
              {AGENT_DEFS.filter(a => a.pinned).map(a => (
                <span key={a.id} className="pack-pin" title={`${a.name} — pinned to Claude`}>{a.name}</span>
              ))}
              <span className="pack-pin-label">pinned to Claude</span>
            </div>
          </div>
        </div>

        <div className="header-controls">
          <label className="run-selector-label">
            Active run
            <select
              className="run-selector"
              value={activeRunId}
              onChange={(event) => setActiveRunId(event.target.value)}
            >
              {runs.length === 0 ? <option value="">No runs yet</option> : null}
              {runs.map((candidate) => (
                <option key={candidate.run_id} value={candidate.run_id}>
                  {candidate.run_id.slice(0, 8)} · {candidate.product} · {candidate.status}
                </option>
              ))}
            </select>
          </label>
          <div className="header-btn-row">
            <button
              className="tesseract-toggle"
              onClick={() => setShowTesseract(true)}
              title="Open Coobie Causal Tesseract"
            >
              Tesseract
            </button>
            <button
              className="tesseract-toggle workbench-btn"
              onClick={() => setShowWorkbench(true)}
              title="Open Causal Workbench — annotate run timeline for Coobie"
            >
              Workbench
            </button>
          </div>
        </div>
      </header>

      {showTesseract && <CausalTesseract onClose={() => setShowTesseract(false)} />}
      {showWorkbench && (
        <CausalWorkbench
          runId={activeRunId}
          events={events}
          causalReport={causalReport}
          onClose={() => setShowWorkbench(false)}
        />
      )}

      {error ? <div className="error-banner">API error: {error}</div> : null}

      <main className="dashboard-grid">
        <section className="main-column">
          <Panel title="Pack Chat">
            <PackChat
              activeRunId={activeRunId}
              agents={agents}
              onRunStarted={(runId) => setActiveRunId(runId)}
            />
          </Panel>

          <Panel title="Action Board">
            <FactoryFloor
              agents={agents}
              onOpenWorkbench={() => setShowWorkbench(true)}
            />
          </Panel>

          <Panel title="Attribution Board">
            <AttributionPanel phaseAttributions={phaseAttributions} />
          </Panel>

          <Panel title="Run Timeline">
            <div className="timeline-list">
              {recentEvents.length === 0 ? (
                <div className="empty-state">No events recorded yet.</div>
              ) : (
                recentEvents.map((event) => (
                  <div key={event.event_id} className="timeline-item">
                    <div className="timeline-meta">
                      <span>{titleCase(event.phase)}</span>
                      <span>{event.agent}</span>
                      <span>{event.status}</span>
                    </div>
                    <div className="timeline-message">{event.message}</div>
                    <div className="timeline-time">{new Date(event.created_at).toLocaleString()}</div>
                  </div>
                ))
              )}
            </div>
          </Panel>
        </section>

        <aside className="side-column">
          <Panel title="Memory Board" compact>
            {memoryAgent ? <AgentCard agent={memoryAgent} isSingleton /> : null}
            <div className="info-stack top-gap">
              <div className="info-row"><span>Lesson refs</span><strong>{blackboard?.lesson_refs?.length || 0}</strong></div>
              <div className="info-row"><span>Promoted lessons</span><strong>{lessons.length}</strong></div>
              <div className="info-row"><span>Recent recalls</span><strong>{agentExecutions.length}</strong></div>
              <div className="info-row"><span>Live pidgin signals</span><strong>{coobieTranslations.reduce((sum, item) => sum + (item.signals?.length || 0), 0)}</strong></div>
              <div className="info-row"><span>Supersessions</span><strong>{memoryUpdates.length}</strong></div>
            </div>
            <div className="top-gap">
              <CoobieSignalPanel translations={coobieTranslations} compact />
            </div>
            <div className="list-block top-gap">
              <div className="list-section-title">Memory updates</div>
              {recentMemoryUpdates.length === 0 ? (
                <div className="empty-state">No supersessions recorded yet.</div>
              ) : (
                recentMemoryUpdates.map((update) => (
                  <div key={update.update_id} className="list-item">
                    <div className="memory-update-header">
                      <div className="memory-update-status-row">
                        <span className={`status-pill status-${(update.review_status || 'pending').toLowerCase()}`}>
                          {titleCase(update.review_status || 'pending')}
                        </span>
                      </div>
                      {(update.reviewed_at || update.reviewed_by) ? (
                        <div className="list-item-subtle">
                          {(update.reviewed_by || 'operator')} · {update.reviewed_at ? new Date(update.reviewed_at).toLocaleString() : 'reviewed'}
                        </div>
                      ) : null}
                    </div>
                    <div className="list-item-title">
                      {update.old_memory_id} → {update.new_memory_id}
                    </div>
                    <div className="list-item-subtle">{update.reason}</div>
                    {update.review_note ? (
                      <div className="list-item-subtle">note: {update.review_note}</div>
                    ) : null}
                    <div className="list-item-subtle">
                      {(update.memory_root || 'memory root unavailable')} · {new Date(update.created_at).toLocaleString()}
                    </div>
                    {(update.review_status || 'pending').toLowerCase() === 'pending' ? (
                      <div className="memory-update-actions">
                        <button
                          type="button"
                          className="memory-update-btn confirm"
                          disabled={reviewingMemoryUpdateId === update.update_id}
                          onClick={() => reviewMemoryUpdate(update.update_id, 'confirmed')}
                        >
                          {reviewingMemoryUpdateId === update.update_id ? 'Working...' : 'Confirm'}
                        </button>
                        <button
                          type="button"
                          className="memory-update-btn reject"
                          disabled={reviewingMemoryUpdateId === update.update_id}
                          onClick={() => reviewMemoryUpdate(update.update_id, 'rejected')}
                        >
                          Reject
                        </button>
                      </div>
                    ) : null}
                  </div>
                ))
              )}
            </div>
            <div className="list-block top-gap">
              {(lessons || []).length === 0 ? (
                <div className="empty-state">No lessons promoted for this run yet.</div>
              ) : (
                lessons.map((lesson) => (
                  <div key={lesson.lesson_id} className="list-item">
                    <div className="list-item-title">{lesson.pattern}</div>
                    <div className="list-item-subtle">
                      intervention: {lesson.intervention || 'none recorded'}
                    </div>
                  </div>
                ))
              )}
            </div>
          </Panel>

          <Panel title="Mission Board" compact>
            <div className="info-stack">
              <div className="info-row"><span>Current phase</span><strong>{titleCase(blackboard?.current_phase || 'idle')}</strong></div>
              <div className="info-row"><span>Active goal</span><strong>{blackboard?.active_goal || 'Awaiting a run.'}</strong></div>
              <div className="info-row"><span>Resolved items</span><strong>{blackboard?.resolved_items?.length || 0}</strong></div>
              <div className="info-row"><span>Artifacts tracked</span><strong>{blackboard?.artifact_refs?.length || 0}</strong></div>
            </div>
            <div className="chip-row">
              {(blackboard?.open_blockers || []).length === 0 ? (
                <span className="soft-chip ok">No blockers</span>
              ) : (
                blackboard.open_blockers.map((blocker) => (
                  <span key={blocker} className="soft-chip danger">{blocker}</span>
                ))
              )}
            </div>
            <div className="role-lens top-gap">
              <div className="role-lens-header">
                <span>Role lens</span>
                <select value={selectedRole} onChange={(event) => setSelectedRole(event.target.value)}>
                  {AGENT_DEFS.map((agent) => (
                    <option key={agent.id} value={agent.id}>{agent.name}</option>
                  ))}
                </select>
              </div>
              <div className="info-stack">
                <div className="info-row"><span>Lens phase</span><strong>{titleCase(roleBoard?.current_phase || 'idle')}</strong></div>
                <div className="info-row"><span>Visible lessons</span><strong>{roleBoard?.lesson_refs?.length || 0}</strong></div>
                <div className="info-row"><span>Visible artifacts</span><strong>{roleBoard?.artifact_refs?.length || 0}</strong></div>
                <div className="info-row"><span>Visible claims</span><strong>{roleClaims.length}</strong></div>
              </div>
            </div>
          </Panel>

          <Panel title="Agent State" compact>
            <div className="agent-state-list">
              {canonicalAgentState.length === 0 ? (
                <div className="empty-state">Canonical agent state has not been seeded yet.</div>
              ) : (
                canonicalAgentState.map(({ id, name, role, state }) => {
                  const blocks = memoryBlockEntries(state);
                  return (
                    <div key={id} className="agent-state-card">
                      <div className="agent-state-head">
                        <div>
                          <div className="agent-state-name">{name}</div>
                          <div className="agent-state-role">{state.agent_role || role}</div>
                        </div>
                        <span className="agent-state-provider">
                          {state.llm_provider || 'unknown'} / {state.llm_model || 'unknown'}
                        </span>
                      </div>
                      <div className="agent-state-grid">
                        <div>
                          <span>Last run</span>
                          <strong>{state.last_active_run ? state.last_active_run.slice(0, 8) : 'none'}</strong>
                        </div>
                        <div>
                          <span>Contract</span>
                          <strong className="mono">{shortHash(state.behavior_contract_hash)}</strong>
                        </div>
                        <div>
                          <span>Stop reason</span>
                          <strong>{state.last_stop_reason || 'none'}</strong>
                        </div>
                        <div>
                          <span>Blocks</span>
                          <strong>{blocks.length}</strong>
                        </div>
                      </div>
                      {blocks.length > 0 ? (
                        <div className="agent-state-blocks">
                          {blocks.slice(0, 4).map(([block, ref]) => (
                            <div key={block} className="agent-state-block">
                              <span>{block.replaceAll('_', ' ')}</span>
                              <code title={ref}>{ref}</code>
                            </div>
                          ))}
                        </div>
                      ) : null}
                    </div>
                  );
                })
              )}
            </div>
          </Panel>

          <Panel title="E2E Candidates" compact>
            {e2eReadinessIndex && (e2eReadinessIndex.entries || []).length > 0 ? (
              <div className="candidate-stack">
                <div className="info-row">
                  <span>Best run</span>
                  <strong>{e2eReadinessIndex.best_run_id ? e2eReadinessIndex.best_run_id.slice(0, 10) : 'none'}</strong>
                </div>
                {(e2eReadinessIndex.entries || []).slice(0, 4).map((entry) => (
                  <button
                    key={entry.run_id}
                    type="button"
                    className={`candidate-row ${entry.run_id === activeRunId ? 'active' : ''}`}
                    onClick={() => setActiveRunId(entry.run_id)}
                  >
                    <div>
                      <strong>{entry.run_id.slice(0, 10)}</strong>
                      <span>{entry.product} · {entry.spec_id}</span>
                      {entry.next_action ? <p>{entry.next_action}</p> : null}
                    </div>
                    <div className="candidate-score">
                      <strong>{Math.round(Number(entry.artifact_score || 0) * 100)}%</strong>
                      <span>{titleCase(entry.status)}</span>
                    </div>
                  </button>
                ))}
              </div>
            ) : (
              <div className="empty-state">No recent readiness candidates yet.</div>
            )}
          </Panel>

          <Panel title="E2E Readiness" compact>
            {e2eReadiness ? (
              <div className="readiness-stack">
                <div className={`readiness-status ${e2eReadiness.status || 'unknown'}`}>
                  <div>
                    <span>Month-end goal</span>
                    <strong>{titleCase(e2eReadiness.status || 'unknown')}</strong>
                  </div>
                  <strong>{Math.round(Number(e2eReadiness.artifact_score || 0) * 100)}%</strong>
                </div>
                <div className="agent-state-grid">
                  <div><span>Artifacts</span><strong>{graphCount(e2eReadiness.ready_artifact_count)} / {graphCount(e2eReadiness.required_artifact_count)}</strong></div>
                  <div><span>Missing</span><strong>{graphCount(e2eReadiness.missing_artifacts?.length || 0)}</strong></div>
                  <div><span>Run</span><strong>{titleCase(e2eReadiness.run_status || 'unknown')}</strong></div>
                  <div><span>Health</span><strong>{titleCase(e2eReadiness.health_status || 'unknown')}</strong></div>
                </div>
                {(e2eReadiness.next_actions || []).length > 0 ? (
                  <div className="readiness-actions">
                    {e2eReadiness.next_actions.slice(0, 4).map((action) => (
                      <div key={action} className="readiness-action">{action}</div>
                    ))}
                  </div>
                ) : null}
                <div className="graph-export-actions">
                  <button
                    type="button"
                    className="memory-update-btn"
                    onClick={materializeE2eReadiness}
                    disabled={materializingE2eReadiness}
                  >
                    {materializingE2eReadiness ? 'Writing report' : 'Materialize readiness'}
                  </button>
                  {materializedE2eReadiness ? (
                    <span>{materializedE2eReadiness.json_artifact}</span>
                  ) : null}
                </div>
                <div className="graph-export-actions">
                  <button
                    type="button"
                    className="memory-update-btn"
                    onClick={materializeE2eManifest}
                    disabled={materializingE2eManifest}
                  >
                    {materializingE2eManifest ? 'Writing manifest' : 'Materialize manifest'}
                  </button>
                  {materializedE2eManifest ? (
                    <span>{materializedE2eManifest.json_artifact}</span>
                  ) : null}
                </div>
                <div className="graph-export-actions">
                  <button
                    type="button"
                    className="memory-update-btn"
                    onClick={materializeE2eBundle}
                    disabled={materializingE2eBundle}
                  >
                    {materializingE2eBundle ? 'Writing bundle' : 'Materialize bundle'}
                  </button>
                  {materializedE2eBundle ? (
                    <span>{materializedE2eBundle.json_artifact || 'e2e_evidence_bundle.json'}</span>
                  ) : null}
                </div>
                {materializedE2eBundle?.bundle_artifacts?.length ? (
                  <div className="bundle-artifact-links">
                    {materializedE2eBundle.bundle_artifacts.map((artifact, index) => (
                      <a
                        key={artifact}
                        href={`${API_ORIGIN}${materializedE2eBundle.bundle_artifact_urls?.[index] || `/api/runs/${activeRunId}/artifacts/${artifact}`}`}
                        target="_blank"
                        rel="noreferrer"
                      >
                        {artifact}
                      </a>
                    ))}
                  </div>
                ) : null}
              </div>
            ) : (
              <div className="empty-state">Readiness has not been computed for this run yet.</div>
            )}
          </Panel>

          <Panel title="Causal Graph" compact>
            {causalGraphStatus ? (
              <div className="graph-ledger">
                <div className="info-stack">
                  <div className="info-row"><span>Typed graph</span><strong>{titleCase(causalGraphStatus.status)}</strong></div>
                  <div className="info-row"><span>Backend</span><strong>{titleCase(causalGraphStatus.backend)}</strong></div>
                  <div className="info-row"><span>Database</span><strong>{causalGraphStatus.database || 'unconfigured'}</strong></div>
                  <div className="info-row"><span>Projection rows</span><strong>{graphCount(causalGraphStatus.projection_count)}</strong></div>
                </div>
                {causalGraphProjection ? (
                  <div className="agent-state-card graph-projection-card">
                    <div className="agent-state-head">
                      <div>
                        <div className="agent-state-name">Active run projection</div>
                        <div className="agent-state-role">{causalGraphProjection.status || 'sqlite_projection'}</div>
                      </div>
                      <span className="agent-state-provider">
                        {titleCase(causalGraphProjection.backend)}
                      </span>
                    </div>
                    <div className="agent-state-grid">
                      <div><span>Episodes</span><strong>{graphCount(causalGraphProjection.episode_count)}</strong></div>
                      <div><span>Events</span><strong>{graphCount(causalGraphProjection.event_count)}</strong></div>
                      <div><span>Links</span><strong>{graphCount(causalGraphProjection.link_count)}</strong></div>
                      <div><span>Hypotheses</span><strong>{graphCount(causalGraphProjection.hypothesis_count)}</strong></div>
                    </div>
                    {(causalGraphProjection.highlights || []).length > 0 ? (
                      <div className="graph-highlights">
                        {causalGraphProjection.highlights.slice(0, 3).map((hit) => (
                          <div key={hit.label} className="graph-highlight">
                            <div>
                              <span>{hit.label}</span>
                              <p>{hit.summary}</p>
                            </div>
                            <strong>{Number(hit.confidence || 0).toFixed(2)}</strong>
                          </div>
                        ))}
                      </div>
                    ) : null}
                  </div>
                ) : (
                  <div className="empty-state">No projection for the active run yet.</div>
                )}
                {causalFailureHistory ? (
                  <div className="agent-state-card graph-history-card">
                    <div className="agent-state-head">
                      <div>
                        <div className="agent-state-name">Same spec failure history</div>
                        <div className="agent-state-role">{causalFailureHistory.spec_id}</div>
                      </div>
                      <span className="agent-state-provider">
                        {graphCount(causalFailureHistory.failure_run_count)} / {graphCount(causalFailureHistory.projection_count)}
                      </span>
                    </div>
                    {(causalFailureHistory.repeated_causes || []).length > 0 ? (
                      <div className="graph-highlights">
                        {causalFailureHistory.repeated_causes.slice(0, 3).map((cause) => (
                          <div key={cause.cause_id} className="graph-highlight">
                            <div>
                              <span>{cause.cause_id}</span>
                              <p>{cause.count} run(s), avg confidence {Number(cause.average_confidence || 0).toFixed(2)}</p>
                            </div>
                            <strong>{cause.run_ids?.length || 0}</strong>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div className="empty-state">No repeated causes for this spec yet.</div>
                    )}
                    {(causalFailureHistory.runs || []).length > 0 ? (
                      <div className="graph-run-history">
                        {causalFailureHistory.runs.slice(0, 3).map((entry) => {
                          const topCause = entry.top_causes?.[0];
                          return (
                            <div key={entry.run_id} className="graph-run-row">
                              <span>{entry.run_id.slice(0, 10)}</span>
                              <p>{topCause ? topCause.summary : `${graphCount(entry.failed_episode_count)} failed episode(s)`}</p>
                            </div>
                          );
                        })}
                      </div>
                    ) : null}
                    {causalFailureExport ? (
                      <div className="graph-export-row">
                        <span>Replay contract</span>
                        <strong>{causalFailureExport.replay_queries?.length || 0} TypeQL queries</strong>
                      </div>
                    ) : null}
                    {causalFailureExport ? (
                      <div className="graph-export-actions">
                        <button
                          type="button"
                          className="memory-update-btn"
                          onClick={materializeCausalExport}
                          disabled={materializingCausalExport}
                        >
                          {materializingCausalExport ? 'Writing artifacts' : 'Materialize replay'}
                        </button>
                        {materializedCausalExport ? (
                          <span>{materializedCausalExport.json_artifact}</span>
                        ) : null}
                      </div>
                    ) : null}
                  </div>
                ) : null}
                {causalGraphStatus.note ? (
                  <div className="graph-note">{causalGraphStatus.note}</div>
                ) : null}
              </div>
            ) : (
              <div className="empty-state">Causal graph status is unavailable.</div>
            )}
          </Panel>

          <Panel title="Evidence Board" compact>
            <div className="list-block compact-list">
              {(blackboard?.artifact_refs || []).length === 0 ? (
                <div className="empty-state">No artifact refs yet.</div>
              ) : (
                blackboard.artifact_refs.map((artifact) => (
                  <a
                    key={artifact}
                    className="list-item tight artifact-link"
                    href={`${API_ORIGIN}/api/runs/${activeRunId}/artifacts/${artifact}`}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {artifact}
                  </a>
                ))
              )}
            </div>
          </Panel>

          <div className="system-section">
            <button
              className="system-toggle"
              onClick={() => setShowSystem(s => !s)}
            >
              <span>⚙ System</span>
              <span className="system-toggle-chevron">{showSystem ? '▲' : '▼'}</span>
            </button>
            {showSystem && <>
          <Panel title="Provider Capacity" compact>
            {!capacity ? (
              <div className="empty-state">capacity.json not loaded</div>
            ) : (
              <>
                <div className="capacity-chain">
                  {(capacity.fallback_chain || []).map((name, idx) => {
                    const p = capacity.providers?.[name] || {};
                    const statusColor = p.available === false ? '#c7684c' : p.status === 'near_limit' ? '#c4922a' : '#8fae7c';
                    return (
                      <div key={name} className="capacity-row">
                        <div className="capacity-rank">#{idx + 1}</div>
                        <div className="capacity-name">{name}</div>
                        <div className="capacity-chip" style={{ color: statusColor, borderColor: `${statusColor}55` }}>
                          <span className="status-dot" style={{ backgroundColor: statusColor }}></span>
                          {p.status || 'ok'}
                        </div>
                        {p.note && <div className="capacity-note">{p.note}</div>}
                      </div>
                    );
                  })}
                </div>
                <div className="capacity-updated">
                  updated {capacity.updated_at ? new Date(capacity.updated_at).toLocaleString() : '—'}
                </div>
              </>
            )}
          </Panel>

          <Panel title="Operations Deck" compact>
                <OperationsDeck activeRunId={activeRunId} />
              </Panel>

              <Panel title="Keeper Policy Board" compact>
            <div className="info-stack">
              <div className="info-row"><span>Managed by</span><strong>{coordination?.managed_by || 'keeper'}</strong></div>
              <div className="info-row"><span>Policy mode</span><strong>{coordination?.policy_mode || 'exclusive_file_claims'}</strong></div>
              <div className="info-row"><span>Heartbeat timeout</span><strong>{coordination?.stale_after_seconds || 600}s</strong></div>
              <div className="info-row"><span>Healthy claims</span><strong>{healthyClaims.length}</strong></div>
              <div className="info-row"><span>Stale claims</span><strong>{staleClaims.length}</strong></div>
              <div className="info-row"><span>Policy events</span><strong>{policyEvents.length}</strong></div>
            </div>

            <div className="list-block">
              <div className="list-item">
                <div className="list-item-title">{selectedRole} role view</div>
                <div className="list-item-subtle">{roleBoard?.active_goal || 'No role-scoped board loaded.'}</div>
              </div>
            </div>

            <div className="list-block compact-list top-gap">
              {coordinationClaims.length === 0 ? (
                <div className="empty-state">No live Keeper claims.</div>
              ) : (
                coordinationClaims.map(([agent, claim]) => (
                  <div key={agent} className={`list-item claim-item ${claim.status === 'stale' ? 'stale-claim' : ''}`}>
                    <div className="list-item-title">{agent}</div>
                    <div className="list-item-subtle">{claim.task}</div>
                    <div className="list-item-subtle">status: {claim.status || 'active'}</div>
                    <div className="list-item-subtle">last heartbeat: {claim.last_heartbeat_at ? new Date(claim.last_heartbeat_at).toLocaleString() : 'none recorded'}</div>
                    <div className="list-item-subtle mono top-gap-small">
                      {(claim.files || []).join(', ') || 'no files declared'}
                    </div>
                  </div>
                ))
              )}
            </div>

            <div className="list-block compact-list top-gap">
              {recentPolicyEvents.length === 0 ? (
                <div className="empty-state">No Keeper policy events yet.</div>
              ) : (
                recentPolicyEvents.map((event) => (
                  <div key={event.event_id} className={`list-item policy-event ${event.status}`}>
                    <div className="list-item-title">{event.event_type.replaceAll('_', ' ')}</div>
                    <div className="list-item-subtle">{event.message}</div>
                    <div className="list-item-subtle mono top-gap-small">
                      {new Date(event.created_at).toLocaleString()}
                    </div>
                  </div>
                ))
              )}
            </div>

            <div className="role-lens top-gap">
              <div className="list-block compact-list">
                {roleClaims.length === 0 ? (
                  <div className="empty-state">No claims visible to this role.</div>
                ) : (
                  roleClaims.map(([agent, claim]) => (
                    <div key={agent} className="list-item">
                      <div className="list-item-title">{agent}</div>
                      <div className="list-item-subtle">{claim}</div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </Panel>
            </>}
          </div>
        </aside>
      </main>

      <footer className="footer-bar glass-panel">
        <span>Active threads: {activeThreads.length ? activeThreads.map((agent) => agent.name).join(', ') : 'none'}</span>
        <span>Events: {events.length}</span>
        <span>Lessons: {lessons.length}</span>
      </footer>

      <style jsx>{`
        .pack-board-shell {
          min-height: 100vh;
          background:
            radial-gradient(circle at top left, rgba(194, 163, 114, 0.12), transparent 28%),
            radial-gradient(circle at top right, rgba(94, 125, 113, 0.14), transparent 32%),
            linear-gradient(180deg, #171a1c 0%, #121416 100%);
          color: var(--text-primary);
          padding: 1.5rem;
          display: flex;
          flex-direction: column;
          gap: 1.25rem;
        }

        .glass-panel {
          background: rgba(27, 30, 32, 0.84);
          border: 1px solid var(--border-glass);
          box-shadow: 0 18px 40px rgba(0, 0, 0, 0.28);
          backdrop-filter: blur(14px);
        }

        .run-header,
        .footer-bar {
          border-radius: 18px;
          padding: 1.1rem 1.3rem;
        }

        .run-header {
          display: flex;
          justify-content: space-between;
          gap: 1rem;
          align-items: start;
        }

        .eyebrow {
          text-transform: uppercase;
          letter-spacing: 0.16em;
          font-size: 0.72rem;
          color: var(--accent-gold);
          margin-bottom: 0.45rem;
          font-weight: 800;
        }

        h1 {
          font-size: clamp(1.7rem, 3vw, 2.5rem);
          margin-bottom: 0.55rem;
          font-family: 'IBM Plex Sans', 'Segoe UI', sans-serif;
        }

        .header-meta {
          display: flex;
          flex-wrap: wrap;
          gap: 0.65rem;
          color: var(--text-secondary);
          font-size: 0.88rem;
        }

        .header-meta span,
        .status-pill,
        .soft-chip {
          border-radius: 999px;
          padding: 0.28rem 0.65rem;
          border: 1px solid rgba(255, 255, 255, 0.08);
          background: rgba(255, 255, 255, 0.03);
        }

        .pack-pins {
          display: flex;
          align-items: center;
          gap: 0.35rem;
          flex-wrap: wrap;
        }

        .pack-pin {
          font-size: 0.62rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          color: #c4922a;
          background: rgba(196, 146, 42, 0.12);
          border: 1px solid rgba(196, 146, 42, 0.28);
          border-radius: 999px;
          padding: 0.15rem 0.5rem;
        }

        .pack-pin-label {
          font-size: 0.58rem;
          color: rgba(255, 255, 255, 0.22);
          letter-spacing: 0.06em;
          text-transform: uppercase;
          background: none !important;
          border: none !important;
          padding: 0 !important;
        }

        .header-controls {
          display: flex;
          flex-direction: column;
          align-items: end;
          gap: 0.75rem;
        }

        .run-selector-label {
          display: flex;
          flex-direction: column;
          gap: 0.3rem;
          font-size: 0.72rem;
          text-transform: uppercase;
          letter-spacing: 0.12em;
          color: var(--text-secondary);
          font-weight: 700;
        }

        .run-selector {
          min-width: 320px;
          max-width: 100%;
          border-radius: 12px;
          border: 1px solid var(--border-glass);
          background: #15181a;
          color: var(--text-primary);
          padding: 0.72rem 0.85rem;
          font: inherit;
        }

        .status-pill {
          text-transform: uppercase;
          letter-spacing: 0.12em;
          font-weight: 800;
          color: var(--accent-gold);
        }

        .status-pending {
          color: var(--accent-gold);
        }

        .status-confirmed {
          color: #8fae7c;
          border-color: rgba(143, 174, 124, 0.32);
          background: rgba(143, 174, 124, 0.12);
        }

        .status-rejected {
          color: #d8876e;
          border-color: rgba(216, 135, 110, 0.35);
          background: rgba(120, 39, 30, 0.22);
        }

        .dashboard-grid {
          display: grid;
          grid-template-columns: minmax(0, 1.9fr) minmax(320px, 0.95fr);
          gap: 1.2rem;
          align-items: start;
        }

        .main-column,
        .side-column {
          display: flex;
          flex-direction: column;
          gap: 1.1rem;
        }

        .ops-panel {
          background: rgba(22, 24, 26, 0.88);
          border: 1px solid var(--border-glass);
          border-radius: 18px;
          padding: 1rem 1rem 1.05rem;
          box-shadow: 0 18px 36px rgba(0, 0, 0, 0.24);
        }

        .panel-title-row {
          display: flex;
          align-items: center;
          gap: 0.8rem;
          margin-bottom: 0.95rem;
        }

        .panel-title-row h3 {
          white-space: nowrap;
          text-transform: uppercase;
          letter-spacing: 0.12em;
          font-size: 0.82rem;
          color: var(--accent-gold);
        }

        .panel-line {
          height: 1px;
          flex: 1;
          background: linear-gradient(90deg, rgba(194, 163, 114, 0.55), transparent);
        }

        .agent-grid {
          display: grid;
          gap: 0.95rem;
        }

        .two-up {
          grid-template-columns: repeat(2, minmax(0, 1fr));
        }

        .three-up {
          grid-template-columns: repeat(3, minmax(0, 1fr));
        }

        .info-stack {
          display: flex;
          flex-direction: column;
          gap: 0.6rem;
        }

        .top-gap {
          margin-top: 0.9rem;
        }

        .top-gap-small {
          margin-top: 0.35rem;
        }

        .role-lens-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 0.75rem;
          margin-bottom: 0.75rem;
          color: var(--text-secondary);
          text-transform: uppercase;
          letter-spacing: 0.1em;
          font-size: 0.72rem;
          font-weight: 700;
        }

        .role-lens-header select {
          min-width: 150px;
          border-radius: 10px;
          border: 1px solid var(--border-glass);
          background: rgba(255, 255, 255, 0.04);
          color: var(--text-primary);
          padding: 0.45rem 0.65rem;
          font: inherit;
        }

        .info-row {
          display: flex;
          justify-content: space-between;
          gap: 0.8rem;
          border: 1px solid rgba(255, 255, 255, 0.05);
          background: rgba(255, 255, 255, 0.03);
          padding: 0.7rem 0.8rem;
          border-radius: 12px;
        }

        .info-row span {
          color: var(--text-secondary);
          font-size: 0.78rem;
          text-transform: uppercase;
          letter-spacing: 0.08em;
        }

        .info-row strong {
          font-size: 0.86rem;
          text-align: right;
        }

        .chip-row {
          display: flex;
          flex-wrap: wrap;
          gap: 0.5rem;
          margin-top: 0.85rem;
        }

        .soft-chip.ok {
          color: #8fae7c;
        }

        .soft-chip.danger {
          color: #d8876e;
        }

        .list-block {
          display: flex;
          flex-direction: column;
          gap: 0.55rem;
          margin-top: 0.9rem;
        }

        .agent-state-list {
          display: flex;
          flex-direction: column;
          gap: 0.65rem;
        }

        .agent-state-card {
          border: 1px solid rgba(255, 255, 255, 0.06);
          background: rgba(255, 255, 255, 0.03);
          border-radius: 12px;
          padding: 0.72rem 0.8rem;
        }

        .graph-ledger {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .graph-projection-card {
          border-color: rgba(103, 148, 131, 0.32);
        }

        .graph-history-card {
          border-color: rgba(194, 163, 114, 0.3);
        }

        .candidate-stack {
          display: flex;
          flex-direction: column;
          gap: 0.55rem;
        }

        .candidate-row {
          display: grid;
          grid-template-columns: minmax(0, 1fr) auto;
          gap: 0.75rem;
          align-items: start;
          width: 100%;
          border: 1px solid rgba(255, 255, 255, 0.06);
          background: rgba(255, 255, 255, 0.03);
          color: var(--text-primary);
          border-radius: 10px;
          padding: 0.65rem 0.72rem;
          text-align: left;
          cursor: pointer;
        }

        .candidate-row.active,
        .candidate-row:hover {
          border-color: rgba(194, 163, 114, 0.34);
          background: rgba(194, 163, 114, 0.08);
        }

        .candidate-row strong {
          display: block;
          font-size: 0.82rem;
        }

        .candidate-row span,
        .candidate-row p,
        .candidate-score span {
          color: var(--text-secondary);
          font-size: 0.72rem;
          line-height: 1.35;
        }

        .candidate-row p {
          margin: 0.25rem 0 0;
        }

        .candidate-score {
          text-align: right;
          white-space: nowrap;
        }

        .readiness-stack {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .readiness-status {
          display: grid;
          grid-template-columns: minmax(0, 1fr) auto;
          gap: 0.8rem;
          align-items: center;
          border: 1px solid rgba(194, 163, 114, 0.25);
          background: rgba(194, 163, 114, 0.07);
          border-radius: 12px;
          padding: 0.72rem 0.8rem;
        }

        .readiness-status.ready {
          border-color: rgba(143, 174, 124, 0.38);
          background: rgba(143, 174, 124, 0.08);
        }

        .readiness-status.blocked {
          border-color: rgba(216, 135, 110, 0.38);
          background: rgba(216, 135, 110, 0.08);
        }

        .readiness-status span {
          display: block;
          color: var(--text-secondary);
          font-size: 0.62rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          margin-bottom: 0.18rem;
        }

        .readiness-status strong {
          font-size: 0.92rem;
        }

        .readiness-actions {
          display: flex;
          flex-direction: column;
          gap: 0.45rem;
        }

        .readiness-action {
          border-left: 2px solid rgba(194, 163, 114, 0.42);
          color: var(--text-secondary);
          font-size: 0.8rem;
          line-height: 1.35;
          padding-left: 0.6rem;
        }

        .graph-note {
          color: var(--text-secondary);
          font-size: 0.82rem;
          line-height: 1.45;
          border-left: 2px solid rgba(194, 163, 114, 0.45);
          padding-left: 0.7rem;
        }

        .graph-highlights {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
          margin-top: 0.7rem;
        }

        .graph-highlight {
          display: grid;
          grid-template-columns: minmax(0, 1fr) auto;
          gap: 0.65rem;
          align-items: start;
          border-top: 1px solid rgba(255, 255, 255, 0.06);
          padding-top: 0.55rem;
        }

        .graph-highlight span {
          color: var(--accent-gold);
          font-size: 0.68rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.08em;
        }

        .graph-highlight p {
          margin: 0.18rem 0 0;
          color: var(--text-secondary);
          font-size: 0.8rem;
          line-height: 1.35;
        }

        .graph-highlight strong {
          color: var(--text-primary);
          font-size: 0.78rem;
        }

        .graph-run-history {
          display: flex;
          flex-direction: column;
          gap: 0.42rem;
          margin-top: 0.65rem;
          border-top: 1px solid rgba(255, 255, 255, 0.06);
          padding-top: 0.6rem;
        }

        .graph-run-row {
          display: grid;
          grid-template-columns: 72px minmax(0, 1fr);
          gap: 0.55rem;
          align-items: start;
        }

        .graph-run-row span {
          color: var(--text-secondary);
          font-size: 0.72rem;
          font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        }

        .graph-run-row p {
          margin: 0;
          color: var(--text-secondary);
          font-size: 0.78rem;
          line-height: 1.35;
        }

        .graph-export-row {
          display: flex;
          justify-content: space-between;
          gap: 0.65rem;
          margin-top: 0.65rem;
          border-top: 1px solid rgba(255, 255, 255, 0.06);
          padding-top: 0.6rem;
          color: var(--text-secondary);
          font-size: 0.78rem;
        }

        .graph-export-row span {
          text-transform: uppercase;
          letter-spacing: 0.08em;
          font-weight: 800;
        }

        .graph-export-row strong {
          color: var(--text-primary);
          font-size: 0.8rem;
          text-align: right;
        }

        .graph-export-actions {
          display: flex;
          align-items: center;
          gap: 0.55rem;
          margin-top: 0.65rem;
          min-width: 0;
        }

        .graph-export-actions span {
          min-width: 0;
          color: var(--text-secondary);
          font-size: 0.74rem;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .bundle-artifact-links {
          display: flex;
          flex-wrap: wrap;
          gap: 0.4rem;
          margin-top: 0.55rem;
        }

        .bundle-artifact-links a {
          max-width: 100%;
          border: 1px solid rgba(255, 255, 255, 0.08);
          border-radius: 6px;
          padding: 0.28rem 0.45rem;
          color: var(--text-secondary);
          background: rgba(255, 255, 255, 0.035);
          font-size: 0.7rem;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
          text-decoration: none;
        }

        .bundle-artifact-links a:hover {
          border-color: rgba(194, 163, 114, 0.34);
          color: var(--text-primary);
        }

        .agent-state-head {
          display: flex;
          justify-content: space-between;
          align-items: start;
          gap: 0.7rem;
          margin-bottom: 0.65rem;
        }

        .agent-state-name {
          font-size: 0.9rem;
          font-weight: 800;
        }

        .agent-state-role {
          margin-top: 0.12rem;
          color: var(--text-secondary);
          font-size: 0.72rem;
          text-transform: uppercase;
          letter-spacing: 0.08em;
        }

        .agent-state-provider {
          max-width: 48%;
          border: 1px solid rgba(194, 163, 114, 0.25);
          background: rgba(194, 163, 114, 0.08);
          color: var(--accent-gold);
          border-radius: 999px;
          padding: 0.22rem 0.5rem;
          font-size: 0.64rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.06em;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .agent-state-grid {
          display: grid;
          grid-template-columns: repeat(2, minmax(0, 1fr));
          gap: 0.45rem;
        }

        .agent-state-grid div {
          min-width: 0;
          border: 1px solid rgba(255, 255, 255, 0.05);
          border-radius: 9px;
          padding: 0.48rem 0.55rem;
          background: rgba(0, 0, 0, 0.12);
        }

        .agent-state-grid span,
        .agent-state-block span {
          display: block;
          color: var(--text-secondary);
          font-size: 0.62rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          margin-bottom: 0.18rem;
        }

        .agent-state-grid strong {
          display: block;
          min-width: 0;
          font-size: 0.76rem;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .agent-state-blocks {
          display: flex;
          flex-direction: column;
          gap: 0.45rem;
          margin-top: 0.65rem;
        }

        .agent-state-block {
          min-width: 0;
        }

        .agent-state-block code {
          display: block;
          width: 100%;
          color: #c9d4d8;
          font-family: var(--font-mono);
          font-size: 0.68rem;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .compact-list {
          margin-top: 0;
        }

        .list-item {
          border: 1px solid rgba(255, 255, 255, 0.05);
          background: rgba(255, 255, 255, 0.03);
          border-radius: 12px;
          padding: 0.72rem 0.8rem;
        }

        .list-item.tight {
          padding: 0.6rem 0.75rem;
          font-family: var(--font-mono);
          font-size: 0.8rem;
        }

        .artifact-link {
          display: block;
          color: var(--text-secondary);
          overflow: hidden;
          text-decoration: none;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .artifact-link:hover {
          border-color: rgba(194, 163, 114, 0.34);
          color: var(--text-primary);
        }

        .list-item-title {
          font-size: 0.86rem;
          font-weight: 700;
          margin-bottom: 0.25rem;
        }

        .list-item-subtle {
          color: var(--text-secondary);
          font-size: 0.76rem;
          line-height: 1.45;
        }

        .memory-update-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 0.6rem;
          margin-bottom: 0.45rem;
          flex-wrap: wrap;
        }

        .memory-update-status-row {
          display: flex;
          align-items: center;
          gap: 0.45rem;
        }

        .memory-update-actions {
          display: flex;
          gap: 0.55rem;
          margin-top: 0.65rem;
        }

        .memory-update-btn {
          border-radius: 999px;
          border: 1px solid rgba(255, 255, 255, 0.12);
          background: rgba(255, 255, 255, 0.04);
          color: var(--text-primary);
          padding: 0.38rem 0.8rem;
          font: inherit;
          font-size: 0.74rem;
          font-weight: 700;
          cursor: pointer;
        }

        .memory-update-btn:disabled {
          opacity: 0.6;
          cursor: wait;
        }

        .memory-update-btn.confirm {
          border-color: rgba(143, 174, 124, 0.4);
          background: rgba(143, 174, 124, 0.12);
          color: #cfe2bf;
        }

        .memory-update-btn.reject {
          border-color: rgba(216, 135, 110, 0.38);
          background: rgba(120, 39, 30, 0.22);
          color: #f0c7bc;
        }

        .policy-event.blocked,
        .policy-event.stale {
          border-color: rgba(199, 104, 76, 0.45);
          background: rgba(120, 39, 30, 0.18);
        }

        .policy-event.granted,
        .policy-event.released,
        .policy-event.revived {
          border-color: rgba(143, 174, 124, 0.32);
        }

        .claim-item.stale-claim {
          border-color: rgba(199, 104, 76, 0.45);
          background: rgba(120, 39, 30, 0.14);
        }

        .mono {
          font-family: var(--font-mono);
        }

        .status-dot {
          width: 0.45rem;
          height: 0.45rem;
          border-radius: 999px;
          flex-shrink: 0;
          display: inline-block;
        }

        .capacity-chain {
          display: flex;
          flex-direction: column;
          gap: 0.45rem;
        }

        .capacity-row {
          display: grid;
          grid-template-columns: 1.5rem 1fr auto;
          grid-template-rows: auto auto;
          gap: 0.2rem 0.6rem;
          align-items: center;
          border: 1px solid rgba(255, 255, 255, 0.05);
          background: rgba(255, 255, 255, 0.03);
          border-radius: 10px;
          padding: 0.55rem 0.7rem;
        }

        .capacity-rank {
          font-size: 0.65rem;
          font-weight: 800;
          color: var(--text-secondary);
          font-family: var(--font-mono);
        }

        .capacity-name {
          font-size: 0.84rem;
          font-weight: 700;
          text-transform: capitalize;
        }

        .capacity-chip {
          display: inline-flex;
          align-items: center;
          gap: 0.35rem;
          border: 1px solid;
          border-radius: 999px;
          padding: 0.18rem 0.5rem;
          font-size: 0.66rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          white-space: nowrap;
        }

        .capacity-note {
          grid-column: 2 / -1;
          font-size: 0.72rem;
          color: var(--text-secondary);
          line-height: 1.4;
        }

        .capacity-updated {
          margin-top: 0.55rem;
          font-size: 0.68rem;
          color: var(--text-secondary);
          font-family: var(--font-mono);
        }

        .tesseract-toggle {
          padding: 0.4rem 0.9rem;
          background: rgba(194, 163, 114, 0.08);
          border: 1px solid rgba(194, 163, 114, 0.35);
          border-radius: 999px;
          color: var(--accent-gold);
          font-size: 0.72rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          cursor: pointer;
          transition: background 0.15s, border-color 0.15s;
        }

        .tesseract-toggle:hover {
          background: rgba(194, 163, 114, 0.15);
          border-color: rgba(194, 163, 114, 0.6);
        }

        .header-btn-row {
          display: flex;
          gap: 0.5rem;
        }

        .workbench-btn {
          background: rgba(90, 138, 204, 0.08);
          border-color: rgba(90, 138, 204, 0.35);
          color: #5a8acc;
        }

        .workbench-btn:hover:not(:disabled) {
          background: rgba(90, 138, 204, 0.15);
          border-color: rgba(90, 138, 204, 0.6);
        }

        .workbench-btn:disabled {
          opacity: 0.35;
          cursor: default;
        }

        .system-section {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .system-toggle {
          display: flex;
          justify-content: space-between;
          align-items: center;
          width: 100%;
          background: rgba(255, 255, 255, 0.03);
          border: 1px solid rgba(255, 255, 255, 0.07);
          border-radius: 10px;
          color: rgba(255, 255, 255, 0.35);
          font-size: 0.72rem;
          font-weight: 800;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          padding: 0.55rem 0.8rem;
          cursor: pointer;
          transition: background 0.12s, color 0.12s;
        }

        .system-toggle:hover {
          background: rgba(255, 255, 255, 0.06);
          color: rgba(255, 255, 255, 0.55);
        }

        .system-toggle-chevron {
          font-size: 0.6rem;
          opacity: 0.6;
        }

        .timeline-list {
          display: flex;
          flex-direction: column;
          gap: 0.7rem;
        }

        .timeline-item {
          border-left: 2px solid rgba(194, 163, 114, 0.55);
          padding: 0.3rem 0 0.3rem 0.85rem;
        }

        .timeline-meta {
          display: flex;
          flex-wrap: wrap;
          gap: 0.5rem;
          text-transform: uppercase;
          letter-spacing: 0.08em;
          font-size: 0.68rem;
          color: var(--accent-gold);
          margin-bottom: 0.25rem;
        }

        .timeline-message {
          font-size: 0.9rem;
          line-height: 1.45;
        }

        .timeline-time {
          margin-top: 0.25rem;
          color: var(--text-secondary);
          font-size: 0.75rem;
          font-family: var(--font-mono);
        }

        .empty-state {
          color: var(--text-secondary);
          font-size: 0.82rem;
        }

        .error-banner {
          border: 1px solid rgba(199, 104, 76, 0.5);
          background: rgba(120, 39, 30, 0.35);
          color: #f0c7bc;
          border-radius: 14px;
          padding: 0.8rem 1rem;
          font-size: 0.88rem;
        }

        .footer-bar {
          display: flex;
          flex-wrap: wrap;
          gap: 0.9rem;
          justify-content: space-between;
          color: var(--text-secondary);
          font-size: 0.82rem;
        }

        @media (max-width: 1280px) {
          .dashboard-grid {
            grid-template-columns: 1fr;
          }
        }

        @media (max-width: 980px) {
          .two-up,
          .three-up {
            grid-template-columns: 1fr;
          }

          .run-header {
            flex-direction: column;
          }

          .header-controls {
            align-items: stretch;
            width: 100%;
          }

          .run-selector {
            min-width: 0;
            width: 100%;
          }
        }
      `}</style>
    </div>
  );
}

export default App;
