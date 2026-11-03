/* Placing, wiring and deleting parts. */

import {box, geom, partById} from './geometry.js';
import {render} from './render.js';
import {safe} from './view.js';
import {snap} from '../core/dom.js';
import {changed, runSim, saveDraft} from '../core/session.js';
import {G, lvl} from '../core/state.js';
import {assemble, STARTER} from '../isa/assembler.js';
import {renderProgram} from '../isa/editor.js';
import {renderMission} from '../panels/mission.js';
import {dialog} from '../ui/dialog.js';
import {say} from '../ui/log.js';

export let uid = Date.now() % 100000;

export function freeSpot(p) {
  const g = geom(p), m = 24, s = safe();
  const hits = (x, y) => G.circuit.parts.some(q => { const h = box(q); return x < h.x + h.w + m && x + g.w + m > h.x && y < h.y + h.h + m && y + g.h + m > h.y; });
  const k = G.vb.w / innerWidth;
  const cx = G.vb.x + (s.l + (innerWidth - s.l - s.r) / 2) * k - g.w / 2, cy = G.vb.y + (s.t + (innerHeight - s.t - s.b) / 2) * k - g.h / 2;
  for (let r = 0; r < 40; r++) for (let a = 0, n = Math.max(1, r * 6); a < n; a++) {
    const t = a / n * Math.PI * 2, x = snap(cx + Math.cos(t) * r * 40), y = snap(cy + Math.sin(t) * r * 30);
    if (!hits(x, y)) return {x, y};
  }
  return {x: snap(cx), y: snap(cy)};
}

export function addPart(e) {
  const p = {id: 'p' + (uid++).toString(36), kind: e.kind, x: 0, y: 0};
  if (e.kind === 'chip') p.chip = e.chip;
  if (e.kind === 'in' || e.kind === 'out') {
    const names = e.kind === 'in' ? lvl().inputs : lvl().outputs;
    p.pin = names.length; p.label = (e.kind === 'in' ? 'IN' : 'OUT') + names.length;
    if (e.kind === 'in') G.inputs[p.pin] = false;
  }
  Object.assign(p, freeSpot(p));
  if (e.kind === 'rom') { p.src = STARTER; p.data = assemble(STARTER).bytes; }
  G.circuit.parts.push(p);
  if (e.kind === 'in' || e.kind === 'out') renderMission();
  G.sel = {type: 'part', id: p.id};
  if (e.kind === 'rom') renderProgram();
  changed();
}

export function deleteSel() {
  const s = G.sel; if (!s) return;
  setTimeout(() => renderProgram(), 0);
  if (s.type === 'wire') G.circuit.wires.splice(s.i, 1);
  else {
    const p = partById(s.id);
    if (!p || ((p.kind === 'in' || p.kind === 'out') && !G.free)) return;
    G.circuit.parts = G.circuit.parts.filter(x => x.id !== s.id);
    G.circuit.wires = G.circuit.wires.filter(w => w.from.part !== s.id && w.to.part !== s.id);
    if (p.kind === 'in' || p.kind === 'out') {
      // Keep pins numbered 0, 1, 2 ... with no gaps.
      const rest = G.circuit.parts.filter(x => x.kind === p.kind).sort((a, b) => a.pin - b.pin);
      const vals = rest.map(x => G.inputs[x.pin]);
      rest.forEach((x, k) => { x.pin = k; });
      if (p.kind === 'in') G.inputs = vals.map(Boolean);
      renderMission();
    }
  }
  G.sel = null; changed();
}

export let firstWire = true;

export function clickPin(id, k) {
  const pd = G.pending;
  if (!pd) { G.pending = {part: id, pin: k}; render(); return; }
  G.pending = null;
  if (pd.part === id && pd.pin === k) { render(); return; }
  const dup = G.circuit.wires.some(w =>
    (w.from.part === pd.part && w.from.pin === pd.pin && w.to.part === id && w.to.pin === k) ||
    (w.to.part === pd.part && w.to.pin === pd.pin && w.from.part === id && w.from.pin === k));
  if (!dup) {
    G.circuit.wires.push({from: {part: pd.part, pin: pd.pin}, to: {part: id, pin: k}});
    if (firstWire) { firstWire = false; say('Trace laid. Gold lines carry charge, steel lines are at 0.'); }
  }
  changed();
}

export function toggleInput(k) { G.inputs[k] = !G.inputs[k]; render(); runSim(); }

/* Turn the selected part a quarter turn about its centre: clockwise, or
   anticlockwise when `back` is set. Wiring is unchanged, so the simulation
   does not need rebuilding. */
export function rotateSel(back = false) {
  const p = G.sel && G.sel.type === 'part' ? partById(G.sel.id) : null;
  if (!p) { say('Select a part to rotate it.', 'warn'); return; }
  p.rot = ((p.rot || 0) + (back ? 3 : 1)) % 4;
  if (!p.rot) delete p.rot;
  saveDraft(); render();
}

/* Remove every part except the input and output pins, after a warning. */
export async function clearBoard() {
  const n = G.circuit.parts.filter(p => p.kind !== 'in' && p.kind !== 'out').length, w = G.circuit.wires.length;
  if (!n && !w) { say('The board is already clear.'); return; }
  const ok = await dialog({title: 'Clear the board?', danger: true, ok: 'Clear board',
    body: `This removes <b>${n} part${n === 1 ? '' : 's'}</b> and <b>${w} wire${w === 1 ? '' : 's'}</b>. Input and output pins stay. This cannot be undone.`});
  if (!ok) return;
  G.circuit.parts = G.circuit.parts.filter(p => p.kind === 'in' || p.kind === 'out');
  G.circuit.wires = []; G.sel = G.pending = null; changed();
  say('Board cleared.', 'warn');
}
