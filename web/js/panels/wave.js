/* Waveform view: chosen wires drawn as timing traces, like a logic
   analyzer. Every simulation step adds a sample. In timing mode the x axis
   is time in gate delays and every change inside a step is drawn, so
   glitches and slow paths are visible. */

import {geom, partById} from '../board/geometry.js';
import {render} from '../board/render.js';
import {$, esc} from '../core/dom.js';
import {boardKey, chipDef, G, lvl} from '../core/state.js';
import {say} from '../ui/log.js';

const MAX_SAMPLES = 4000, ROW = 24, SLOT = 16;

export const waveOpen = () => !$('#wave').classList.contains('hidden');

export function toggleWave(show = !waveOpen()) {
  $('#wave').classList.toggle('hidden', !show);
  document.body.classList.toggle('wave', show);
  try { localStorage.setItem('m1-wave', show ? '1' : '0'); } catch (e) {}
  renderWave();
}

function storeProbes() {
  try { localStorage.setItem('m1-probes:' + boardKey(), JSON.stringify(G.probes)); } catch (e) {}
}

/* Probes for the board just opened. */
export function loadProbes() {
  G.probes = []; G.wave = []; G.waveGlitchy = []; G.timing = null;
  try { G.probes = JSON.parse(localStorage.getItem('m1-probes:' + boardKey()) || '[]'); } catch (e) {}
  renderWave();
}

export function clearWave() { G.wave = []; G.waveGlitchy = G.probes.map(() => false); G.timing = null; renderWave(); }

function pinName(p, k) {
  if (p.kind === 'in') return lvl().inputs[p.pin] || 'IN';
  if (p.kind === 'out') return lvl().outputs[p.pin] || 'OUT';
  const q = geom(p).pins[k] || {};
  const label = p.kind === 'chip' ? (chipDef(p.chip) || {}).label || p.chip : p.kind.toUpperCase();
  return `${label}.${q.n || k}`;
}

/* Probe pins of the selected wire or part. */
export function probeSelection() {
  const s = G.sel;
  if (!s) { say('Select a wire or a part, then press A to add it to the waveform.', 'warn'); return; }
  let ends = [];
  if (s.type === 'wire') { const w = G.circuit.wires[s.i]; if (w) ends = [w.from]; }
  else {
    const p = partById(s.id); if (!p) return;
    const n = geom(p).pins.length;
    if (p.kind === 'in' || p.kind === 'out' || p.kind === 'vcc' || p.kind === 'gnd') ends = [{part: p.id, pin: 0}];
    else if (p.kind === 'nmos' || p.kind === 'pmos') ends = [{part: p.id, pin: 0}, {part: p.id, pin: 1}, {part: p.id, pin: 2}];
    else if (p.kind === 'display') ends = Array.from({length: 8}, (_, k) => ({part: p.id, pin: k}));
    else {
      const ni = p.kind === 'rom' ? 4 : (chipDef(p.chip) || {inputs: []}).inputs.length;
      for (let k = ni; k < n && ends.length < 16; k++) ends.push({part: p.id, pin: k});
    }
  }
  let added = 0;
  for (const e of ends) {
    if (G.probes.some(q => q.part === e.part && q.pin === e.pin)) continue;
    const p = partById(e.part);
    let name = pinName(p, e.pin);
    if (G.probes.some(q => q.name === name)) name += '#' + (G.probes.length + 1);
    G.probes.push({part: e.part, pin: e.pin, name});
    added++;
  }
  if (!added) { say('Already in the waveform.'); return; }
  storeProbes(); clearWave(); toggleWave(true); render();
  say(`Probing ${added} signal${added === 1 ? '' : 's'}. Toggle inputs or pulse the clock to draw them.`);
}

function removeProbe(i) { G.probes.splice(i, 1); storeProbes(); clearWave(); render(); }

/* Add the samples from one simulation response. */
export function recordTrace(r) {
  if (!G.probes.length || !r.trace) return;
  if (!G.waveGlitchy || G.waveGlitchy.length !== G.probes.length) G.waveGlitchy = G.probes.map(() => false);
  if (r.timing) r.timing.glitchy.forEach((g, k) => { if (g) G.waveGlitchy[k] = true; });
  for (const s of r.trace) {
    if (s.v.length !== G.probes.length) continue;
    const last = G.wave[G.wave.length - 1];
    // Edits resimulate too; a step that changed nothing adds nothing.
    if (s.t0 == null && last && last.t0 == null && last.v.join() === s.v.join() && last.ins.join() === s.ins.join()) continue;
    if (s.t0 != null && last && last.t0 == null) G.wave = [];
    if (s.t0 == null && last && last.t0 != null) G.wave = [];
    G.wave.push(s);
  }
  if (G.wave.length > MAX_SAMPLES) G.wave.splice(0, G.wave.length - MAX_SAMPLES);
}

const Y = (row, v) => row * ROW + (v === '1' ? 5 : v === '0' ? ROW - 6 : ROW / 2);

/* One trace from a list of [x, value] points ending at xEnd. */
function trace(row, pts, xEnd) {
  let s = '';
  for (let i = 0; i < pts.length; i++) {
    const [x, v] = pts[i], x2 = i + 1 < pts.length ? pts[i + 1][0] : xEnd;
    if (x2 <= x) continue;
    if (v === 'X') s += `<rect class="wx" x="${x}" y="${row * ROW + 5}" width="${x2 - x}" height="${ROW - 11}"/>`;
    else s += `<path class="w${v === 'Z' ? 'z' : v === '1' ? '1' : '0'}" d="M${x},${Y(row, v)} H${x2}"/>`;
    const nx = i + 1 < pts.length ? pts[i + 1] : null;
    if (nx && nx[1] !== v && (v === '0' || v === '1') && (nx[1] === '0' || nx[1] === '1')) s += `<path class="wedge" d="M${x2},${Y(row, v)} V${Y(row, nx[1])}"/>`;
  }
  return s;
}

function zeroDelay(width) {
  const n = Math.max(1, Math.floor(width / SLOT)), samples = G.wave.slice(-n), clk = lvl().inputs.indexOf('CLK');
  let s = '';
  samples.forEach((sm, i) => {
    const prev = samples[i - 1];
    if (clk >= 0 && prev && !prev.ins[clk] && sm.ins[clk]) s += `<path class="wclk" d="M${i * SLOT},0 V${G.probes.length * ROW}"/>`;
  });
  G.probes.forEach((_, k) => { s += trace(k, samples.map((sm, i) => [i * SLOT, sm.v[k]]), samples.length * SLOT); });
  return {svg: s, w: samples.length * SLOT, axis: `${samples.length} step${samples.length === 1 ? '' : 's'}${clk >= 0 ? ', lines mark rising clock edges' : ''}`};
}

function timedView(width) {
  const w = G.wave, t1 = w[w.length - 1].t1 + 2;
  // Show the most recent stretch at no less than 2 px per gate delay.
  const span = Math.min(t1 - w[0].t0, Math.floor(width / 2)), t0 = t1 - span, k = width / Math.max(span, 1);
  const x = t => (t - t0) * k;
  let s = '';
  for (const sm of w) if (sm.t0 >= t0) s += `<path class="wstep" d="M${x(sm.t0)},0 V${G.probes.length * ROW}"/>`;
  G.probes.forEach((_, p) => {
    const pts = [];
    let cur = 'Z';
    for (const sm of w) {
      for (const [t, q, v] of sm.ev || []) if (q === p) { if (t <= t0) cur = v; else pts.push([x(t), v]); }
      if (sm.t1 <= t0) cur = sm.v[p];
    }
    pts.unshift([0, cur]);
    s += trace(p, pts, x(t1));
  });
  for (const v of (G.timing && G.timing.violations) || []) if (v.clk_at >= t0) s += `<path class="wviol" d="M${x(v.clk_at)},0 V${G.probes.length * ROW}"><title>Setup: ${esc(v.part)} input ${v.input} changed at ${v.data_at}, clock at ${v.clk_at}</title></path>`;
  const step = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000].find(d => d * k >= 60) || 5000;
  for (let t = Math.ceil(t0 / step) * step; t <= t1; t += step) s += `<text class="wt" x="${x(t) + 2}" y="${G.probes.length * ROW + 11}">${t}</text><path class="wtick" d="M${x(t)},${G.probes.length * ROW} v4"/>`;
  return {svg: s, w: width, axis: 'time in gate delays; faint lines mark each input step'};
}

export function renderWave() {
  const el = $('#wave'); if (!el || el.classList.contains('hidden')) return;
  const t = G.timing;
  const status = !G.timed ? '' : !t ? '' : `<span class="wstat">settle ${t.max_settle}${t.violations.length ? ` · <b class="mis">${t.violations.length} setup</b>` : ''}${G.unstable ? ' · <b class="mis">oscillating</b>' : ''}</span>`;
  el.innerHTML = `<div class="whd"><b>Waveform</b>
    <button id="wMode" class="${G.timed ? 'on' : ''}" title="Timing mode (T)">${G.timed ? 'Timed' : 'Zero-delay'}</button>${status}
    <span class="sp"></span><button id="wVcd" title="Save as a VCD file for GTKWave">VCD</button><button id="wClr">Clear</button><button id="wX" title="Hide (W)">Hide</button></div>
    <div class="wbody"><div class="wnames">${G.probes.map((p, i) => `<div class="wn${G.waveGlitchy && G.waveGlitchy[i] ? ' gl' : ''}" title="${G.waveGlitchy && G.waveGlitchy[i] ? 'Glitched: changed more than once in one step' : esc(p.name)}"><span>${esc(p.name)}</span><button data-rm="${i}" aria-label="Remove ${esc(p.name)}">×</button></div>`).join('')}</div>
    <div class="wplot"></div></div>`;
  $('#wMode').onclick = toggleTimed;
  $('#wClr').onclick = clearWave;
  $('#wX').onclick = () => toggleWave(false);
  $('#wVcd').onclick = saveVcd;
  el.querySelectorAll('[data-rm]').forEach(b => b.onclick = () => removeProbe(+b.dataset.rm));
  const plot = el.querySelector('.wplot');
  if (!G.probes.length) { plot.innerHTML = `<div class="note wempty">Select a wire or part on the board and press <kbd>A</kbd> to watch it here.</div>`; return; }
  if (!G.wave.length) { plot.innerHTML = `<div class="note wempty">Toggle an input or pulse the clock to draw the traces.</div>`; return; }
  const width = Math.max(plot.clientWidth - 8, 100);
  const v = G.wave[G.wave.length - 1].t0 != null ? timedView(width) : zeroDelay(width);
  const h = G.probes.length * ROW + 14;
  plot.innerHTML = `<svg class="wsvg" width="${v.w}" height="${h}" viewBox="0 0 ${v.w} ${h}">${v.svg}</svg><div class="waxis">${v.axis}</div>`;
}

export function toggleTimed() {
  G.timed = !G.timed;
  clearWave();
  say(G.timed ? 'Timing mode: every gate now takes time. Watch for glitches and slow paths in the waveform. Diagnostics still run zero-delay.' : 'Zero-delay mode: everything settles at once.', G.timed ? 'warn' : '');
  import('../core/session.js').then(m => m.runSim());
  import('../ui/keys.js').then(m => m.renderKeys());
}

/* Value Change Dump of what the waveform holds. */
function saveVcd() {
  if (!G.probes.length || !G.wave.length) { say('Nothing to save yet.', 'warn'); return; }
  const id = k => String.fromCharCode(33 + k), timed = G.wave[0].t0 != null;
  let s = `$comment Mark One waveform: ${lvl().name}. ${timed ? 'One time unit is one gate delay.' : 'One time unit is one simulation step.'} $end\n$timescale 1ns $end\n$scope module top $end\n`;
  G.probes.forEach((p, k) => { s += `$var wire 1 ${id(k)} ${p.name.replace(/\s/g, '_')} $end\n`; });
  s += '$upscope $end\n$enddefinitions $end\n';
  const lv = v => (v === '1' || v === '0') ? v : v.toLowerCase();
  const last = [];
  let lastT = null;
  const emit = (t, k, v) => {
    if (last[k] === v) return;
    last[k] = v;
    if (t !== lastT) { s += `#${t}\n`; lastT = t; }
    s += `${lv(v)}${id(k)}\n`;
  };
  G.wave.forEach((sm, i) => {
    if (timed) { for (const [t, k, v] of sm.ev || []) emit(t - G.wave[0].t0, k, v); sm.v.forEach((v, k) => emit(sm.t1 - G.wave[0].t0, k, v)); }
    else sm.v.forEach((v, k) => emit(i, k, v));
  });
  const url = URL.createObjectURL(new Blob([s], {type: 'text/plain'}));
  const a = document.createElement('a'); a.href = url; a.download = `${lvl().name.replace(/[^\w-]+/g, '_')}.vcd`; document.body.appendChild(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

addEventListener('resize', renderWave);
let stored = null;
try { stored = localStorage.getItem('m1-wave'); } catch (e) {}
if (stored === '1') { $('#wave').classList.remove('hidden'); document.body.classList.add('wave'); }
