/* Simulation requests, draft saving and opening Marks. */

import {render} from '../board/render.js';
import {fitView} from '../board/view.js';
import {api} from './api.js';
import {$, snap} from './dom.js';
import {applyState, G, lvl} from './state.js';
import {renderListing} from '../isa/editor.js';
import {buildComputerTemplate} from '../isa/template.js';
import {lessonSeen, openLesson} from '../lesson/player.js';
import {stopClock} from '../machine/clock.js';
import {renderDock} from '../panels/dock.js';
import {loadBreakpoints, renderDebugger} from '../panels/debugger.js';
import {renderFlags} from '../panels/flags.js';
import {renderMarks} from '../panels/marks.js';
import {benchChanged} from '../modes/tools.js';
import {renderMission} from '../panels/mission.js';
import {renderSpec} from '../panels/spec.js';
import {loadProbes, recordTrace, renderWave} from '../panels/wave.js';
import {say} from '../ui/log.js';
import {renderReactor} from '../ui/reactor.js';

/* Requests run one at a time, in order. Calls made while one is waiting
   share it, so every state the clock passes through gets simulated. */

export let simChain = Promise.resolve(), simQueued = null, simReset = false;

export function applySim(r) {
  if (r.error) { G.simError = r.error; G.pins = {}; G.outputs = []; G.short = G.unstable = G.degraded = false; G.internals = []; G.pc = null; return; }
  G.simError = null; G.pins = r.pins; G.outputs = r.outputs; G.short = r.short; G.unstable = r.unstable; G.degraded = !!r.degraded;
  // Keep the state from before the last change, so the debugger can mark what changed.
  const now = r.internals || [];
  if (JSON.stringify(now) !== JSON.stringify(G.internals)) G.prevInternals = G.internals;
  G.internals = now; G.pc = r.pc ?? null;
  if (r.timing) G.timing = r.timing;
  recordTrace(r);
}

export function runSim(reset = false, cycles = null) {
  simReset = simReset || reset;
  if (simQueued && cycles == null) return simQueued;
  const job = simChain.then(async () => {
    if (simQueued === job) simQueued = null;
    const r0 = simReset; simReset = false;
    let r = null;
    try {
      r = await api('POST', '/api/sim', {level: lvl().id, circuit: G.circuit, inputs: G.inputs, reset: r0, cycles,
        probes: G.probes.map(p => ({part: p.part, pin: p.pin})), timed: G.timed, inspect: hasRom(),
        breakpoints: [...G.breakpoints]});
      applySim(r);
    } catch (e) { G.simError = 'Server unreachable: ' + (e.message || e); }
    render(); renderFlags(); renderSpec(); renderListing(); renderWave(); renderDebugger();
    return r;
  });
  if (cycles == null) simQueued = job;
  simChain = job;
  return job;
}

export let saveT = null;

export function saveDraft() {
  clearTimeout(saveT);
  const url = G.free ? `/api/designs/${encodeURIComponent(G.free.name)}` : `/api/levels/${lvl().id}/circuit`;
  const c = JSON.parse(JSON.stringify(G.circuit));
  saveT = setTimeout(() => api('PUT', url, c).catch(() => {}), 350);
}

export const hasRom = () => G.circuit.parts.some(p => p.kind === 'rom');

/* After every edit: save, resimulate, and rerun the test bench. */
export function changed() { G.ftable = null; saveDraft(); runSim(); benchChanged(); }

export function ensureIO() {
  const l = lvl(), c = G.circuit;
  c.parts = c.parts || []; c.wires = c.wires || [];
  const xo = Math.max(l.inputs.length, l.outputs.length) > 8 ? 1100 : 640;
  l.inputs.forEach((_, i) => { if (!c.parts.some(p => p.kind === 'in' && p.pin === i)) c.parts.push({id: 'in' + i, kind: 'in', pin: i, x: 0, y: i * 40}); });
  l.outputs.forEach((_, i) => { if (!c.parts.some(p => p.kind === 'out' && p.pin === i)) c.parts.push({id: 'out' + i, kind: 'out', pin: i, x: xo, y: i * 40}); });
  // Older saves predate the 20-unit grid.
  for (const p of c.parts) { p.x = snap(p.x || 0); p.y = snap(p.y || 0); }
}

export async function openLevel(i) {
  stopClock();
  G.free = null;
  G.idx = i;
  const l = lvl();
  G.circuit = await api('GET', `/api/levels/${l.id}/circuit`);
  ensureIO();
  if (l.mode !== 'circuit' && !G.circuit.parts.some(p => p.kind !== 'in' && p.kind !== 'out')) await buildComputerTemplate();
  G.inputs = l.inputs.map(() => false);
  G.sel = G.pending = null; G.pins = {}; G.outputs = []; G.last = {}; G.romTarget = null;
  loadProbes(); loadBreakpoints();
  $('#result').innerHTML = ''; $('#result').className = '';
  try { localStorage.setItem('lf-level', l.id); } catch (e) {}
  renderMarks(); renderMission(); renderDock(); fitView(); render();
  say(`Mk ${i + 1} loaded: ${l.name}. ${l.mode === 'program' ? 'Write the program in the ROM, then run diagnostics.' : l.mode === 'sandbox' ? 'Free build.' : l.chip ? `Pass diagnostics to fabricate ${l.label}.` : 'Wire it up, then run diagnostics.'}`);
  if (!lessonSeen(l.id)) openLesson();
  await runSim(true);
}

export async function refreshState() {
  applyState(await api('GET', '/api/state'));
  renderReactor(); renderMarks(); renderMission(); renderDock(); render();
}
