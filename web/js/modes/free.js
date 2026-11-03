/* Experiment mode: named designs from the parts Missions have unlocked,
   truth tables, and saving a design as a reusable component. */

import {render} from '../board/render.js';
import {fitView} from '../board/view.js';
import {api} from '../core/api.js';
import {$, bit, esc, snap} from '../core/dom.js';
import {refreshState, runSim, saveDraft} from '../core/session.js';
import {G, lvl} from '../core/state.js';
import {renderProgram} from '../isa/editor.js';
import {stopClock} from '../machine/clock.js';
import {renderDock} from '../panels/dock.js';
import {renderMarks} from '../panels/marks.js';
import {renderMission} from '../panels/mission.js';
import {renderSpec} from '../panels/spec.js';
import {loadBreakpoints} from '../panels/debugger.js';
import {loadProbes} from '../panels/wave.js';
import {benchBadge, loadBench, refreshTools, renderTools, TABS} from './tools.js';
import {dialog} from '../ui/dialog.js';
import {renderKeys} from '../ui/keys.js';
import {say} from '../ui/log.js';

export async function openFree(name) {
  stopClock();
  G.free = {name}; G.lastDesign = name;
  try { G.circuit = await api('GET', `/api/designs/${encodeURIComponent(name)}`); } catch (e) { G.circuit = {parts: [], wires: []}; }
  G.circuit.parts = G.circuit.parts || []; G.circuit.wires = G.circuit.wires || [];
  for (const p of G.circuit.parts) { p.x = snap(p.x || 0); p.y = snap(p.y || 0); }
  try { G.designs = await api('GET', '/api/designs'); } catch (e) { G.designs = []; }
  G.inputs = lvl().inputs.map(() => false);
  G.sel = G.pending = null; G.pins = {}; G.outputs = []; G.last = {}; G.romTarget = null; G.ftable = null;
  loadProbes(); loadBreakpoints(); await loadBench();
  $('#spec').innerHTML = ''; // the tool tabs belong to the previous design
  $('#result').innerHTML = ''; $('#result').className = '';
  try { localStorage.setItem('lf-level', 'free:' + name); } catch (e) {}
  renderMarks(); renderMission(); renderDock(); fitView(); render();
  say(`Experiment: ${name}. Every part you have unlocked is in the fabricator. Nothing is graded.`);
  await runSim(true);
}

export function renderFreeMission() {
  const l = lvl();
  $('#mk').textContent = 'Exp'; $('#mname').textContent = l.name; $('#mprod').textContent = 'Experiment';
  const names = [...new Set([...(G.designs || []), l.name])].sort();
  const u = G.unlocks || {}, locked = (G.components || []).filter(c => !c.unlocked).length;
  $('#mission').innerHTML = `<div class="intro">Build anything from the parts you have unlocked. Add INPUT and OUTPUT pins from the fabricator, and double-click a pin to rename it.${locked ? ` <span class="note">${locked} more component${locked === 1 ? '' : 's'}${u.transistors ? '' : ' and transistors'} unlock as you pass Missions.</span>` : ''}</div>
    <div class="drow"><select id="dsel" aria-label="Design">${names.map(n => `<option${n === l.name ? ' selected' : ''}>${esc(n)}</option>`).join('')}</select>
      <button id="dNew">New</button><button id="dDel">Delete</button></div>
    <button id="dComp" class="wide">Save as component</button>
    <div class="pinrow">${l.inputs.map(n => `<span class="pin-chip">${esc(n)}</span>`).join('')}${l.outputs.map(n => `<span class="pin-chip o">${esc(n)}</span>`).join('')}</div>
    <button id="benchBadge" class="badge" title="Open the test bench"></button>`;
  benchBadge();
  $('#benchBadge').onclick = () => { G.ftab = 'bench'; renderFreeSpec(); };
  $('#dsel').onchange = e => openFree(e.target.value);
  $('#dNew').onclick = async () => {
    const v = await dialog({title: 'New design', ok: 'Create', fields: [{name: 'name', label: 'Design name', max: 60, required: true}]});
    if (v) { G.designs = [...new Set([...(G.designs || []), v.name])]; openFree(v.name).then(saveDraft); }
  };
  $('#dComp').onclick = saveAsComponent;
  $('#dDel').onclick = async () => {
    const ok = await dialog({title: `Delete "${l.name}"?`, danger: true, ok: 'Delete design', body: 'The design and its wiring are removed. This cannot be undone.'});
    if (!ok) return;
    await api('DELETE', `/api/designs/${encodeURIComponent(l.name)}`).catch(() => {});
    G.designs = (G.designs || []).filter(n => n !== l.name);
    openFree(G.designs[0] || 'untitled');
  };
  $('#databank').innerHTML = `<div class="note">Library chips behave exactly like the reference models every engine test checks. Your own components run the circuit inside them.</div>`;
  renderKeys();
  renderSpec();
  renderProgram(true);
}

/* The Experiment analysis panel: a tab per tool. */
export function renderFreeSpec() {
  $('#specTitle').textContent = 'Analysis';
  const tab = G.ftab || 'table';
  // Tool tabs keep their inputs between simulations; only results refresh.
  const tool = $('#ftool');
  if (tab !== 'table' && tool && tool.dataset.tab === tab) { refreshTools(tab); return; }
  $('#spec').innerHTML = `<div class="tabs" role="tablist">${TABS.map(([k, n]) =>
    `<button role="tab" data-ftab="${k}" aria-selected="${k === tab}" class="${k === tab ? 'on' : ''}">${n}</button>`).join('')}</div><div id="ftool" data-tab="${tab}"></div>`;
  $('#spec').querySelectorAll('[data-ftab]').forEach(b => b.onclick = () => { G.ftab = b.dataset.ftab; renderFreeSpec(); });
  if (tab === 'table') renderTable(); else renderTools(tab);
}

function renderTable() {
  const tb = G.ftable;
  let s = `<div class="note">Applies every input combination to a freshly powered copy of your circuit.</div><button id="ftGo" style="margin-top:8px">${tb ? 'Regenerate' : 'Generate truth table'}</button>`;
  if (tb && tb.error) s += `<div class="note mis" style="margin-top:8px">${esc(tb.error)}</div>`;
  else if (tb) {
    const bad = tb.rows.filter(r => r.short || r.unstable || r.race || r.degraded || r.outs.some(v => v === 'Z' || v === 'X')).length;
    s += `<div class="note" style="margin-top:8px">${tb.rows.length} rows. ${bad ? `<span class="mis">${bad} with a short, oscillation, race, degraded, floating or unknown output.</span>` : 'Every output is fully driven on every row, in either evaluation order.'}${tb.stateful ? ' Contains memory: each row starts from power-up.' : ''}</div>`;
    s += `<table class="spec"><tr>${tb.inputs.map(n => `<th>${esc(n)}</th>`).join('')}${tb.outputs.map((n, i) => `<th class="${i ? '' : 'sep'}">${esc(n)}</th>`).join('')}<th class="sep"></th></tr>`;
    for (const r of tb.rows.slice(0, 256)) {
      const cur = r.ins.every((b, i) => b === !!G.inputs[i]);
      s += `<tr class="${cur ? 'cur' : ''}">${r.ins.map(b => `<td class="${b ? 'one' : ''}">${bit(b)}</td>`).join('')}${r.outs.map((v, i) => `<td class="${v === '1' ? 'one' : v === 'Z' || v === 'X' ? 'bad' : ''}${i ? '' : ' sep'}">${v}</td>`).join('')}<td class="sep bad">${r.short ? 'short' : r.unstable ? 'osc' : r.race ? 'race' : r.degraded ? 'weak' : ''}</td></tr>`;
    }
    s += `</table>${tb.rows.length > 256 ? `<div class="note">Showing the first 256 of ${tb.rows.length} rows.</div>` : ''}`;
  }
  $('#ftool').innerHTML = s;
  $('#ftGo').onclick = async () => {
    try { G.ftable = await api('POST', '/api/table', G.circuit); } catch (e) { G.ftable = {error: e.message}; }
    renderFreeSpec();
  };
}

/* Save the current design as a reusable component. Its pins are the design's
   INPUT and OUTPUT pins; saving again under the same name replaces it. */
async function saveAsComponent() {
  const l = lvl();
  if (!l.outputs.length) {
    dialog({title: 'Add an output first', ok: 'OK', body: 'A component needs at least one OUTPUT pin. Its INPUT and OUTPUT pins become the chip\'s pins.'});
    return;
  }
  const guess = l.name.replace(/[^A-Za-z0-9]/g, '').slice(0, 6).toUpperCase() || 'CHIP';
  let err = '';
  for (;;) {
    const v = await dialog({title: 'Save as component', ok: 'Save component',
      body: `${err ? `<span class="mis">${esc(err)}</span><br>` : ''}Pins: <b>${l.inputs.map(esc).join(', ') || 'none'}</b> in, <b>${l.outputs.map(esc).join(', ')}</b> out. It appears in the fabricator in Experiment mode. Missions only accept the chips they unlock.`,
      fields: [{name: 'name', label: 'Name', value: l.name, max: 40, required: true}, {name: 'label', label: 'Chip label', value: guess, max: 8, required: true, hint: 'Up to 8 characters, printed on the chip.'}]});
    if (!v) return;
    try {
      await api('POST', '/api/custom', {name: v.name, label: v.label, circuit: G.circuit});
      await refreshState();
      say(`Saved component ${v.name}. Find it as ${v.label} in the fabricator.`, 'good');
      return;
    } catch (e) { err = e.message; }
  }
}
