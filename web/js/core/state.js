/* The single client state object and lookups derived from it. */

import {$} from './dom.js';

export const svg = $('#board');

export const G = {
  levels: [], idx: 0, circuit: {parts: [], wires: []}, inputs: [],
  pins: {}, outputs: [], short: false, unstable: false, simError: null,
  sel: null, pending: null, mouse: {x: 0, y: 0},
  vb: {x: 0, y: 0, w: 1200, h: 800}, drag: null, pan: null, clock: null, last: {},
  // Waveform probes and samples, timing mode, and the program debugger.
  probes: [], wave: [], timed: false, timing: null, breakpoints: new Set(), internals: [], pc: null,
};

/* Which board this is, for keeping probes and breakpoints per board. */
export const boardKey = () => G.free ? 'free:' + G.free.name : (G.levels[G.idx] || {}).id;

export const lvl = () => G.free ? freeLevel() : G.levels[G.idx];

export const chipDef = id => (G.components || []).find(c => c.id === id) || (G.custom || []).find(c => c.id === id) || G.levels.find(l => l.id === id);

/* Store what /api/state returns. */
export function applyState(st) {
  G.levels = st.levels; G.components = st.components; G.custom = st.custom || []; G.unlocks = st.unlocks || {};
}
/* Free build: the circuit itself defines its pins, every part is available. */

export function freePins(kind, prefix) {
  const ps = G.circuit.parts.filter(p => p.kind === kind);
  const n = ps.reduce((m, p) => Math.max(m, p.pin + 1), 0);
  return Array.from({length: n}, (_, k) => { const p = ps.find(q => q.pin === k); return (p && p.label) || prefix + k; });
}

export function freeLevel() {
  // Experiment mode offers only what Missions have unlocked, plus your own
  // components. The server enforces the same rule.
  const u = G.unlocks || {};
  const pal = [{kind: 'in', label: 'INPUT', cost: 0}, {kind: 'out', label: 'OUTPUT', cost: 0},
    {kind: 'vcc', label: 'VCC', cost: 0}, {kind: 'gnd', label: 'GND', cost: 0},
    ...(u.transistors ? [{kind: 'nmos', label: 'NMOS', cost: 1}, {kind: 'pmos', label: 'PMOS', cost: 1}] : []),
    ...(u.rom ? [{kind: 'rom', label: 'ROM', cost: 128}, {kind: 'display', label: 'DISPLAY', cost: 0}] : []),
    ...(G.components || []).filter(c => c.unlocked).map(c => ({kind: 'chip', chip: c.id, label: c.label, cost: c.cost})),
    ...(G.custom || []).map(c => ({kind: 'chip', chip: c.id, label: c.label, cost: c.cost, custom: true}))];
  return {id: 'free', name: G.free.name, chapter: 'Experiment', label: 'EXP', inputs: freePins('in', 'IN'), outputs: freePins('out', 'OUT'),
    chip: false, fets: true, rom: true, par: null, best: null, mode: 'free', table: null, sequential: false, palette: pal, intro: ''};
}

export const mkNo = id => G.levels.findIndex(l => l.id === id) + 1;

export const clkIdx = () => lvl().inputs.indexOf('CLK');

export const doneCount = () => G.levels.filter(l => l.status === 'done').length;
