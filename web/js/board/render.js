/* Drawing the board as SVG. */

import {cham, geom, partById, pinPos, pv, rotOf, ticks} from './geometry.js';
import {pathD, routeAll} from './wires.js';
import {bit, esc} from '../core/dom.js';
import {chipDef, G, lvl, svg} from '../core/state.js';
import {romAddr} from '../isa/assembler.js';

/* Keep labels readable on a rotated part. Names and values are turned back
   upright. Pin labels turn with their pins, but are flipped when the part is
   upside down. */
function uprightText(s, r) {
  if (!r) return s;
  return s.replace(/<text class="([^"]*)" x="([-\d.]+)" y="([-\d.]+)"/g, (m, cls, x, y) => {
    if (/\bpn\b/.test(cls) && /\b[lr]\b/.test(cls)) {
      if (r !== 2) return m;
      cls = cls.replace(/\b([lr])\b/, (_, c) => c === 'l' ? 'r' : 'l');
      return `<text class="${cls}" x="${x}" y="${y}" transform="rotate(180 ${x} ${y})"`;
    }
    return `${m} transform="rotate(${-90 * r} ${x} ${y})"`;
  });
}

export function expectedNow() {
  const l = lvl(); if (!l.table) return null;
  const row = l.table.find(r => r.ins.every((b, i) => b === !!G.inputs[i]));
  return row ? row.outs : null;
}

export function renderPart(p) {
  const g = geom(p), l = lvl();
  const sel = G.sel && G.sel.type === 'part' && G.sel.id === p.id;
  const r = rotOf(p);
  let s = `<g class="part${sel ? ' sel' : ''}" data-part="${esc(p.id)}" transform="translate(${p.x},${p.y})${r ? ` rotate(${90 * r} ${g.w / 2} ${g.h / 2})` : ''}">`;
  if (p.kind === 'vcc') {
    s += `<path class="body rail" d="${cham(40, 40, 7)}"/><text class="lbl gold" x="20" y="18">VCC</text>`;
  } else if (p.kind === 'gnd') {
    // Turned sideways there is no room for both the symbol and the name.
    s += `<path class="body gnd" d="${cham(40, 40, 7)}"/>` + (r % 2
      ? `<text class="lbl" x="20" y="20" style="font-size:10px">GND</text>`
      : `<path class="sym" d="M20,5 V13 M10,13 H30 M13,17 H27 M16,21 H24" style="stroke-width:1.4"/><text class="lbl" x="20" y="31" style="font-size:10px">GND</text>`);
  } else if (p.kind === 'nmos' || p.kind === 'pmos') {
    const gv = pv(p.id, 0), isP = p.kind === 'pmos', on = isP ? gv === '0' : gv === '1';
    s += `<path class="body" d="${cham(40, 80, 6)}"/>`;
    s += `<path class="sym" d="M0,40 H${isP ? 7 : 13} M14,24 V56"/>`;
    if (isP) s += `<circle cx="10.5" cy="40" r="3.2" class="sym" style="stroke-width:1.4"/>`;
    s += `<path class="chan${on ? ' on' : ''}" d="M20,18 V62"/><path class="sym" d="M20,18 H40 V0 M20,62 H40 V80"/>`;
    s += `<text class="lbl" x="30" y="40" style="font-size:11px;fill:var(--text-dim)">${isP ? 'P' : 'N'}</text>`;
  } else if (p.kind === 'in') {
    const on = !!G.inputs[p.pin];
    s += `<path class="body in${on ? ' on' : ''}" d="${cham(80, 40, 6)}" style="cursor:pointer"/>`;
    s += `<text class="lbl l" x="10" y="20">${esc(l.inputs[p.pin] || '?')}</text><text class="lbl val ${on ? 'v1' : ''}" x="64" y="20">${bit(on)}</text>`;
  } else if (p.kind === 'out') {
    const v = G.outputs[p.pin] || 'Z', exp = expectedNow();
    let cls = v === '1' ? ' on' : '';
    if (exp) cls += exp[p.pin] === v ? ' good' : ' badv';
    s += `<path class="body out${cls}" d="${cham(80, 40, 6)}"/>`;
    s += `<text class="lbl val v${v}" x="17" y="20">${esc(v)}</text><text class="lbl r" x="70" y="20">${esc(l.outputs[p.pin] || '?')}</text>`;
  } else if (p.kind === 'rom') {
    const a = romAddr(p);
    s += `<path class="body" d="${cham(g.w, g.h, 9)}"/>${ticks(g.w, g.h)}`;
    s += `<text class="lbl chiplbl" x="${g.w / 2}" y="12" style="fill:var(--holo)">ROM</text>`;
    s += `<text class="lbl pn" x="${g.w / 2}" y="${g.h - 14}">${a == null ? 'addr ?' : 'addr ' + a}</text>`;
    for (const q of g.pins) s += `<text class="lbl pn ${q.side}" x="${q.side === 'l' ? 9 : g.w - 9}" y="${q.y}">${esc(q.n)}</text>`;
  } else if (p.kind === 'display') {
    const vals = [0, 1, 2, 3, 4, 5, 6, 7].map(k => pv(p.id, k));
    const known = vals.every(v => v === '0' || v === '1');
    const n = known ? vals.reduce((a, v, k) => a + (v === '1' ? 1 << k : 0), 0) : null;
    s += `<path class="body rail" d="${cham(g.w, g.h, 9)}"/>${ticks(g.w, g.h)}`;
    s += `<text class="lbl disp" x="${g.w / 2 + 8}" y="90" style="font-size:42px;fill:var(--gold)">${n == null ? '--' : n}</text>`;
    s += `<text class="lbl disp pn" x="${g.w / 2 + 8}" y="132" style="font-size:12px">${vals.slice().reverse().join('')}</text>`;
    s += `<text class="lbl pn" x="${g.w / 2 + 8}" y="${g.h - 14}">DISPLAY</text>`;
    for (const q of g.pins) s += `<text class="lbl pn l" x="9" y="${q.y}">${esc(q.n)}</text>`;
  } else {
    const d = chipDef(p.chip);
    s += `<path class="body" d="${cham(g.w, g.h, 9)}"/>${ticks(g.w, g.h)}`;
    s += `<text class="lbl chiplbl" x="${g.w / 2}" y="12">${esc(d ? d.label : p.chip)}</text>`;
    for (const q of g.pins) s += `<text class="lbl pn ${q.side}" x="${q.side === 'l' ? 9 : g.w - 9}" y="${q.y}">${esc(q.n)}</text>`;
  }
  g.pins.forEach((q, k) => {
    const pend = G.pending && G.pending.part === p.id && G.pending.pin === k;
    const probed = G.probes.some(q => q.part === p.id && q.pin === k);
    s += `<circle class="pin v${pv(p.id, k)}${pend ? ' pend' : ''}${probed ? ' probed' : ''}" cx="${q.x}" cy="${q.y}" r="5.5" data-part="${esc(p.id)}" data-pin="${k}"/>`;
  });
  return uprightText(s, r) + '</g>';
}

export function render() {
  let s = `<defs>
    <pattern id="gmin" width="20" height="20" patternUnits="userSpaceOnUse"><path d="M20,0 H0 V20" fill="none" stroke="#5ad7ff" stroke-opacity=".045"/><circle r="1.3" fill="#5ad7ff" fill-opacity=".28"/></pattern>
    <pattern id="gmaj" width="100" height="100" patternUnits="userSpaceOnUse"><rect width="100" height="100" fill="url(#gmin)"/><path d="M100,0 H0 V100" fill="none" stroke="#5ad7ff" stroke-opacity=".1"/><path d="M-3,0 H3 M0,-3 V3" stroke="#5ad7ff" stroke-opacity=".5"/></pattern>
    <filter id="glow" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="2.4" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
    <filter id="glowr" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="2" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter>
  </defs><rect x="-20000" y="-20000" width="40000" height="40000" fill="url(#gmaj)"/>`;
  let ws = '', flows = '';
  for (const {i, pts} of routeAll()) {
    const w = G.circuit.wires[i], d = pathD(pts);
    const v = pv(w.from.part, w.from.pin);
    const sel = G.sel && G.sel.type === 'wire' && G.sel.i === i;
    if (v === '1' || v === 'X') ws += `<path class="halo${v === 'X' ? ' x' : ''}" d="${d}"/>`;
    ws += `<path class="wire v${v}${sel ? ' sel' : ''}" d="${d}"/><path class="hit" d="${d}" data-wire="${i}"/>`;
    if (v === '1') flows += `<path class="flow" d="${d}"/>`;
  }
  s += ws + flows;
  for (const p of G.circuit.parts) s += renderPart(p);
  if (G.pending) {
    const p = partById(G.pending.part);
    if (p) { const a = pinPos(p, G.pending.pin); s += `<path class="rubber" d="M${a.x},${a.y} L${G.mouse.x},${G.mouse.y}"/>`; }
  }
  svg.innerHTML = s;
}

export let raf = false;

export function renderSoon() { if (!raf) { raf = true; requestAnimationFrame(() => { raf = false; render(); }); } }
