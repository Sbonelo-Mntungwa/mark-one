/* Print the current circuit as a schematic sheet: title block, drawing, bill
   of materials, pins, and the truth table or spec when there is one. */

import {box, partById, pinPos} from '../board/geometry.js';
import {render} from '../board/render.js';
import {pathD, routeAll} from '../board/wires.js';
import {$, bit, esc} from '../core/dom.js';
import {chipDef, G, lvl} from '../core/state.js';
import {designators, junctions, printGeom, symbol} from './symbols.js';

/* Transistors in the circuit: chips at their library count, ROM at 128. */
function transistorCount() {
  const pal = lvl().palette;
  return G.circuit.parts.reduce((n, p) => {
    if (p.kind === 'nmos' || p.kind === 'pmos') return n + 1;
    if (p.kind === 'chip') { const e = pal.find(x => x.chip === p.chip) || chipDef(p.chip); return n + ((e && e.cost) || 0); }
    return n + (p.kind === 'rom' ? 128 : 0);
  }, 0);
}

const KIND = {vcc: 'VCC rail', gnd: 'GND rail', nmos: 'NMOS transistor', pmos: 'PMOS transistor', rom: 'ROM, 16 bytes', display: 'Decimal display', in: 'Input pin', out: 'Output pin'};

/* Bounding box of every part and wire, with a margin for labels. */
function bounds(routes) {
  const xs = [], ys = [];
  for (const p of G.circuit.parts) { const b = box(p, printGeom); xs.push(b.x, b.x + b.w); ys.push(b.y, b.y + b.h); }
  for (const {pts} of routes) for (const [x, y] of pts) { xs.push(x); ys.push(y); }
  if (!xs.length) return {x: 0, y: 0, w: 400, h: 200};
  const m = 50, x = Math.min(...xs) - m, y = Math.min(...ys) - m;
  return {x, y, w: Math.max(...xs) + m - x, h: Math.max(...ys) + m - y};
}

/* The circuit drawn with standard symbols: wires, junction dots, parts. */
function drawing(routes, refs) {
  const ends = [];
  for (const {i, net} of routes) for (const e of [G.circuit.wires[i].from, G.circuit.wires[i].to]) {
    const p = partById(e.part); if (!p) continue;
    const v = pinPos(p, e.pin, printGeom);
    ends.push({net, x: v.x, y: v.y, dir: `${-v.dx},${-v.dy}`});
  }
  const wires = routes.map(r => `<path class="wire" d="${pathD(r.pts)}"/>`).join('');
  const dots = junctions(routes, ends).map(([x, y]) => `<circle class="dot" cx="${x}" cy="${y}" r="3.2"/>`).join('');
  return wires + G.circuit.parts.map(p => symbol(p, refs)).join('') + dots;
}

function billOfMaterials(refs) {
  const rows = new Map();
  for (const p of G.circuit.parts) {
    const d = p.kind === 'chip' ? chipDef(p.chip) : null;
    const key = p.kind === 'chip' ? `chip:${p.chip}` : p.kind;
    const name = d ? `${d.label}: ${d.name}${d.custom ? ' (your component)' : ''}` : KIND[p.kind] || p.kind;
    const each = p.kind === 'nmos' || p.kind === 'pmos' ? 1 : p.kind === 'rom' ? 128 : d ? (lvl().palette.find(e => e.chip === p.chip) || d).cost || 0 : 0;
    const r = rows.get(key) || {name, n: 0, each, refs: []};
    r.n++; if (refs[p.id]) r.refs.push(refs[p.id]);
    rows.set(key, r);
  }
  return [...rows.values()].sort((a, b) => b.each - a.each || a.name.localeCompare(b.name));
}

function tableHtml(inputs, outputs, rows) {
  return `<table class="ptab"><tr>${inputs.map(n => `<th>${esc(n)}</th>`).join('')}${outputs.map((n, i) => `<th class="${i ? '' : 'sep'}">${esc(n)}</th>`).join('')}</tr>
    ${rows.map(r => `<tr>${r.ins.map(b => `<td>${bit(b)}</td>`).join('')}${r.outs.map((v, i) => `<td class="${i ? '' : 'sep'}">${esc(v)}</td>`).join('')}</tr>`).join('')}</table>`;
}

/* Fill the sheet. Separate from printing so it can be checked on its own. */
export function buildSheet() {
  G.sel = G.pending = null;
  render();
  const l = lvl(), routes = routeAll(printGeom), refs = designators(G.circuit.parts);
  const b = bounds(routes), bom = billOfMaterials(refs);
  const title = G.free ? l.name : `Mk ${G.idx + 1}: ${l.name}`;
  const table = G.free ? (G.ftable && !G.ftable.error ? G.ftable : null) : (l.table ? {inputs: l.inputs, outputs: l.outputs, rows: l.table} : null);
  const rows = table ? table.rows.slice(0, 64) : [];
  $('#printSheet').innerHTML = `
    <header class="ptitle-block">
      <div><div class="pk">MARK ONE SCHEMATIC</div><h1>${esc(title)}</h1></div>
      <dl><dt>Mode</dt><dd>${G.free ? 'Experiment' : 'Mission'}</dd><dt>Transistors</dt><dd>${transistorCount()}</dd>
        <dt>Parts</dt><dd>${G.circuit.parts.length}</dd><dt>Wires</dt><dd>${G.circuit.wires.length}</dd>
        <dt>Printed</dt><dd>${new Date().toLocaleDateString(undefined, {year: 'numeric', month: 'short', day: 'numeric'})}</dd></dl>
    </header>
    <svg class="sch" viewBox="${b.x} ${b.y} ${b.w} ${b.h}" preserveAspectRatio="xMidYMid meet">${drawing(routes, refs)}</svg>
    <section class="pcols">
      <div><h2>Bill of materials</h2><table class="ptab"><tr><th>Qty</th><th class="l">Ref</th><th class="l">Part</th><th>Each</th><th>Total</th></tr>
        ${bom.map(r => `<tr><td>${r.n}</td><td class="l">${esc(r.refs.join(', '))}</td><td class="l">${esc(r.name)}</td><td>${r.each}</td><td>${r.n * r.each}</td></tr>`).join('')}
        <tr class="tot"><td></td><td></td><td class="l">Total transistors</td><td></td><td>${transistorCount()}</td></tr></table>
        <h2>Pins</h2><p>In: ${l.inputs.map(esc).join(', ') || 'none'}<br>Out: ${l.outputs.map(esc).join(', ') || 'none'}</p></div>
      ${table ? `<div><h2>${G.free ? 'Truth table' : 'Spec'}</h2>${tableHtml(table.inputs, table.outputs, rows)}
        ${table.rows.length > rows.length ? `<p>First ${rows.length} of ${table.rows.length} rows.</p>` : ''}</div>` : ''}
    </section>`;
}

export function printSchematic() {
  buildSheet();
  window.print();
}
