/* Hover scanner: explains whatever is under the pointer. */

import {geom, partById, pv} from '../board/geometry.js';
import {CHIP_FN, PART_INFO, SIG_INFO} from '../content/parts.js';
import {$, esc} from '../core/dom.js';
import {chipDef, G, lvl, mkNo} from '../core/state.js';

export function scanFor(t) {
  if (t.classList && t.classList.contains('pin')) {
    const p = partById(t.dataset.part), k = +t.dataset.pin, v = pv(p.id, k), q = geom(p).pins[k];
    let name = q && q.n ? q.n : p.kind === 'in' ? lvl().inputs[p.pin] : p.kind === 'out' ? lvl().outputs[p.pin] : 'Pin';
    if (p.kind === 'vcc' || p.kind === 'gnd') name = p.kind.toUpperCase();
    if ((p.kind === 'nmos' || p.kind === 'pmos') && q) name = {G: 'Gate', S: 'Source', D: 'Drain'}[q.n];
    return [name, `<span class="sv">${esc(v)}</span> ${SIG_INFO[v]}. ${G.pending ? 'Click to finish the wire.' : 'Click to start a wire.'}`];
  }
  if (t.dataset && t.dataset.wire !== undefined) {
    const w = G.circuit.wires[+t.dataset.wire]; if (!w) return null;
    const v = pv(w.from.part, w.from.pin);
    return ['Trace', `<span class="sv">${esc(v)}</span> ${SIG_INFO[v]}. Click to select, right-click to cut.`];
  }
  const g = t.closest && t.closest('.part');
  if (g) {
    const p = partById(g.dataset.part); if (!p) return null;
    if (PART_INFO[p.kind]) return PART_INFO[p.kind];
    if (p.kind === 'in') return [`Input ${lvl().inputs[p.pin]}`, G.free ? 'Click to toggle. Double-click to rename. Name it CLK to drive it with the clock.' : 'Click to toggle between 0 and 1. Diagnostics drive this pin during tests.'];
    if (p.kind === 'out') return [`Output ${lvl().outputs[p.pin]}`, G.free ? 'Double-click to rename.' : 'Diagnostics read this pin. It must be driven to 0 or 1, never left floating.'];
    const e = lvl().palette.find(x => x.chip === p.chip);
    const d = chipDef(p.chip);
    return [`${d.label} chip`, `${esc(CHIP_FN[p.chip] || d.summary || '')} ${G.free ? 'Library component' : `Fabricated on Mk ${mkNo(p.chip)}`}.`];
  }
  const tile = t.closest && t.closest('[data-pal]');
  if (tile) {
    const e = lvl().palette[+tile.dataset.pal];
    if (e.kind === 'chip') return [`${e.label} chip`, `${esc(CHIP_FN[e.chip] || '')} ${G.free ? 'Library component' : `Your design from Mk ${mkNo(e.chip)}`}.`];
    if (e.kind === 'in' || e.kind === 'out') return [e.kind === 'in' ? 'Input pin' : 'Output pin', 'Adds a named pin to your circuit. Double-click a pin on the board to rename it.'];
    return PART_INFO[e.kind];
  }
  return null;
}

document.addEventListener('mousemove', e => {
  const sc = $('#scan');
  if (G.drag || G.pan) { sc.classList.remove('show'); return; }
  const info = scanFor(e.target);
  if (!info) { sc.classList.remove('show'); return; }
  sc.innerHTML = `<b>${esc(info[0])}</b>${info[1]}`;
  const x = Math.min(e.clientX + 18, innerWidth - 310), y = Math.min(e.clientY + 18, innerHeight - sc.offsetHeight - 10);
  sc.style.left = x + 'px'; sc.style.top = y + 'px';
  sc.classList.add('show');
});
