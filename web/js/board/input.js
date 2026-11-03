/* Mouse and keyboard input on the board. */

import {addPart, clickPin, deleteSel, toggleInput} from './editing.js';
import {partById} from './geometry.js';
import {render, renderSoon} from './render.js';
import {setVB, svgPt} from './view.js';
import {$, snap} from '../core/dom.js';
import {changed, openLevel, saveDraft} from '../core/session.js';
import {G, lvl, svg} from '../core/state.js';
import {renderProgram} from '../isa/editor.js';
import {openFree} from '../modes/free.js';
import {openLibrary} from '../panels/library.js';
import {renderMission} from '../panels/mission.js';
import {dialog} from '../ui/dialog.js';

svg.addEventListener('mousedown', e => {
  if (e.button !== 0) return;
  const t = e.target, pt = svgPt(e);
  if (t.classList.contains('pin')) { e.preventDefault(); clickPin(t.dataset.part, +t.dataset.pin); return; }
  if (t.dataset.wire !== undefined) { G.sel = {type: 'wire', i: +t.dataset.wire}; G.pending = null; render(); return; }
  const g = t.closest('.part');
  if (g) {
    const p = partById(g.dataset.part);
    G.sel = {type: 'part', id: p.id};
    G.drag = {p, sx: pt.x, sy: pt.y, px: p.x, py: p.y, moved: false};
    if (p.kind === 'rom') renderProgram();
    render(); return;
  }
  G.sel = null;
  if (G.pending) { G.pending = null; render(); return; }
  G.pan = {cx: e.clientX, cy: e.clientY, vx: G.vb.x, vy: G.vb.y};
  render();
});

addEventListener('mousemove', e => {
  if (!G.levels.length) return;
  const pt = svgPt(e); G.mouse = pt;
  if (G.drag) {
    const d = G.drag, dx = pt.x - d.sx, dy = pt.y - d.sy;
    if (Math.abs(dx) + Math.abs(dy) > 4) d.moved = true;
    if (d.moved) { d.p.x = snap(d.px + dx); d.p.y = snap(d.py + dy); renderSoon(); }
  } else if (G.pan) {
    const k = G.vb.w / innerWidth;
    G.vb.x = G.pan.vx - (e.clientX - G.pan.cx) * k; G.vb.y = G.pan.vy - (e.clientY - G.pan.cy) * k; setVB();
  } else if (G.pending) renderSoon();
});

addEventListener('mouseup', () => {
  const d = G.drag;
  if (d) { if (d.moved) saveDraft(); else if (d.p.kind === 'in') toggleInput(d.p.pin); }
  G.drag = null; G.pan = null;
});

svg.addEventListener('wheel', e => {
  e.preventDefault();
  const pt = svgPt(e), f = Math.exp(e.deltaY * 0.0012);
  const w = Math.min(Math.max(G.vb.w * f, 300), 9000), k = w / G.vb.w;
  G.vb.x = pt.x - (pt.x - G.vb.x) * k; G.vb.y = pt.y - (pt.y - G.vb.y) * k; G.vb.w = w; G.vb.h = w * innerHeight / innerWidth; setVB();
}, {passive: false});

svg.addEventListener('dblclick', async e => {
  const g = e.target.closest('.part'); if (!g || !G.free) return;
  const p = partById(g.dataset.part); if (!p || (p.kind !== 'in' && p.kind !== 'out')) return;
  const v = await dialog({title: `Rename ${p.kind === 'in' ? 'input' : 'output'} pin`, ok: 'Rename',
    fields: [{name: 'label', label: 'Pin name', value: p.label || '', max: 12, hint: 'Name an input CLK to drive it with the clock buttons.'}]});
  if (!v) return;
  p.label = v.label || undefined;
  renderMission(); changed();
});

svg.addEventListener('contextmenu', e => {
  const g = e.target.closest('.part'), w = e.target.dataset.wire;
  if (g) { e.preventDefault(); G.sel = {type: 'part', id: g.dataset.part}; deleteSel(); }
  else if (w !== undefined) { e.preventDefault(); G.sel = {type: 'wire', i: +w}; deleteSel(); }
});


$('#dock').addEventListener('click', e => { const b = e.target.closest('[data-pal]'); if (b) addPart(lvl().palette[+b.dataset.pal]); });

$('#marks').addEventListener('click', e => {
  if (e.target.closest('[data-free]')) { openFree(G.lastDesign || 'untitled'); return; }
  if (e.target.closest('[data-library]')) { openLibrary(); return; }
  const b = e.target.closest('[data-lv]'); if (b && !b.disabled) openLevel(+b.dataset.lv);
});
