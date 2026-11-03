/* Experiment tools: test benches, equivalence checks, static timing and
   export. Each runs on the server against the current design. */

import {api} from '../core/api.js';
import {$, esc} from '../core/dom.js';
import {G, lvl} from '../core/state.js';
import {say} from '../ui/log.js';

export const TABS = [['table', 'Table'], ['bench', 'Bench'], ['equiv', 'Equiv'], ['timing', 'Timing'], ['export', 'Export']];

const BENCH_HELP = `<details class="help"><summary>How to write a bench</summary>
<div class="note">The first line names inputs, then <code>|</code>, then outputs. Each row sets inputs, lets the circuit settle and checks outputs. Rows run in order on one circuit, so state carries over.</div>
<pre class="ex"># comments start with # or ;
A[3:0] B[3:0] CIN | S[3:0] COUT
3      4      0   | 7      0
0xF    1      0   | 0      1</pre>
<div class="note"><code>S[3:0]</code> is a bus of pins S3 to S0 and takes a number (decimal, 0x, 0b). Inputs: <code>-</code> keeps the last value, <code>P</code> pulses a 1-bit input (0, 1, 0). Outputs: <code>X</code>, <code>Z</code>, or <code>-</code> to skip. A line <code>reset</code> powers the circuit up again. Every bench runs in both chip evaluation orders, like diagnostics.</div></details>`;

/* ---- Test bench ---- */

let saveT = null, runT = null;

export async function loadBench() {
  G.bench = {text: '', report: null};
  try { G.bench.text = (await api('GET', `/api/designs/${encodeURIComponent(G.free.name)}/bench`)).bench || ''; } catch (e) {}
  if (G.bench.text) runBench();
}

function saveBench() {
  clearTimeout(saveT);
  const name = G.free.name, text = G.bench.text;
  saveT = setTimeout(() => api('PUT', `/api/designs/${encodeURIComponent(name)}/bench`, {bench: text}).catch(() => {}), 400);
}

export async function runBench() {
  if (!G.free || !G.bench || !G.bench.text.trim()) { if (G.bench) G.bench.report = null; refreshTools('bench'); return; }
  try { G.bench.report = await api('POST', '/api/bench', {circuit: G.circuit, bench: G.bench.text}); }
  catch (e) { G.bench.report = {error: e.message}; }
  refreshTools('bench');
  benchBadge();
}

/* Rerun the bench a moment after each edit, so it always reflects the design. */
export function benchChanged() {
  if (!G.free || !G.bench || !G.bench.text.trim()) return;
  clearTimeout(runT);
  runT = setTimeout(runBench, 300);
}

export function benchSummary() {
  const r = G.bench && G.bench.report;
  if (!r) return null;
  if (r.error) return {ok: false, text: 'Bench has an error'};
  return {ok: r.failed === 0, text: r.failed ? `Bench: ${r.failed} of ${r.steps.length} rows fail` : `Bench: all ${r.steps.length} rows pass`};
}

/* A pass/fail chip in the mission panel, visible from every tab. */
export function benchBadge() {
  const el = $('#benchBadge'); if (!el) return;
  const s = benchSummary();
  el.className = 'badge' + (s ? (s.ok ? ' ok' : ' bad') : '');
  el.textContent = s ? s.text : 'No test bench';
}

function benchTab() {
  $('#ftool').innerHTML = `<div class="note">Expected behaviour for this design. It reruns after every change.</div>
    <textarea id="benchText" class="code" spellcheck="false" aria-label="Test bench" placeholder="A B | Y&#10;0 0 | 0&#10;0 1 | 1">${esc(G.bench.text)}</textarea>
    <div class="row"><button id="bRun" class="prime">Run</button><button id="bRec" title="Replace each expected output with what the circuit does now">Record outputs</button><button id="bAll" title="A row for every input combination, with the outputs the circuit gives now">Full table</button></div>
    <div id="bOut"></div>${BENCH_HELP}`;
  const ta = $('#benchText');
  ta.oninput = () => { G.bench.text = ta.value; saveBench(); benchChanged(); };
  $('#bRun').onclick = runBench;
  const record = async all => {
    try {
      const r = await api('POST', '/api/bench/record', {circuit: G.circuit, bench: all ? '' : G.bench.text});
      if (r.error) { G.bench.report = {error: r.error}; refreshTools('bench'); return; }
      G.bench.text = r.bench; ta.value = r.bench; saveBench(); runBench();
      say(all ? 'Recorded a row for every input combination. Check them: they are only as right as the circuit is now.' : 'Recorded the current outputs as expected. Check them before trusting the bench.', 'warn');
    } catch (e) { G.bench.report = {error: e.message}; refreshTools('bench'); }
  };
  $('#bRec').onclick = () => record(false);
  $('#bAll').onclick = () => record(true);
  benchResult();
}

function benchResult() {
  const el = $('#bOut'); if (!el) return;
  const r = G.bench.report;
  if (!r) { el.innerHTML = ''; return; }
  if (r.error) { el.innerHTML = `<div class="note mis">${esc(r.error)}</div>`; return; }
  const bad = r.steps.filter(s => !s.pass);
  let s = `<div class="verdict ${bad.length ? 'bad' : 'ok'}">${bad.length ? `${bad.length} of ${r.steps.length} rows fail` : `All ${r.steps.length} rows pass`}</div>`;
  if (bad.length) {
    s += `<table class="spec"><tr><th>Line</th><th>Expected</th><th>Got</th><th></th></tr>${bad.slice(0, 40).map(st =>
      `<tr><td>${st.line}</td><td>${st.expected.map(esc).join(' ')}</td><td>${st.got.map((g, i) => `<span class="${g !== st.expected[i] && st.expected[i] !== '-' ? 'mis' : ''}">${esc(g)}</span>`).join(' ')}</td><td class="bad">${st.short ? 'short' : st.race ? 'race' : st.unstable ? 'osc' : ''}</td></tr>`).join('')}</table>`;
    if (bad.length > 40) s += `<div class="note">Showing the first 40.</div>`;
  }
  el.innerHTML = s;
}

/* ---- Equivalence ---- */

function equivTab() {
  const designs = (G.designs || []).filter(n => n !== G.free.name);
  const comps = [...(G.custom || []), ...(G.components || []).filter(c => c.unlocked)];
  $('#ftool').innerHTML = `<div class="note">Check this design against another one or a component. Pins are matched by name.</div>
    <div class="drow"><select id="eqSel" aria-label="Compare against">
      ${designs.length ? `<optgroup label="Designs">${designs.map(n => `<option value="d:${esc(n)}">${esc(n)}</option>`).join('')}</optgroup>` : ''}
      <optgroup label="Components">${comps.map(c => `<option value="c:${esc(c.id)}">${esc(c.label)}: ${esc(c.name)}</option>`).join('')}</optgroup>
    </select><button id="eqGo" class="prime">Check</button></div>
    <div id="eqOut"></div>
    <div class="note" style="margin-top:8px">Combinational designs with up to 16 inputs are proved over every input combination. Designs with memory, or more inputs, get 200 random sequences from power-up: strong evidence, not a proof.</div>`;
  if (G.eqLast) $('#eqSel').value = G.eqLast;
  $('#eqGo').onclick = async () => {
    const v = $('#eqSel').value; if (!v) return;
    G.eqLast = v;
    const against = v.startsWith('d:') ? {design: v.slice(2)} : {component: v.slice(2)};
    $('#eqOut').innerHTML = '<div class="note">Checking...</div>';
    try { G.equiv = await api('POST', '/api/equiv', {circuit: G.circuit, against}); } catch (e) { G.equiv = {error: e.message}; }
    equivResult();
  };
  equivResult();
}

function equivResult() {
  const el = $('#eqOut'), r = G.equiv; if (!el) return;
  if (!r) { el.innerHTML = ''; return; }
  if (r.error) { el.innerHTML = `<div class="note mis">${esc(r.error)}</div>`; return; }
  const cls = {proved: 'ok', bounded: 'warn', different: 'bad'}[r.verdict];
  const head = {proved: 'Equivalent (proved)', bounded: 'No difference found', different: 'Different'}[r.verdict];
  let s = `<div class="verdict ${cls}">${head}</div><div class="note">${esc(r.detail)}</div>`;
  if (r.counterexample.length) {
    const rows = r.counterexample.slice(-12);
    s += `<div class="note" style="margin-top:6px">${r.counterexample.length > 12 ? `Last 12 of ${r.counterexample.length} steps` : 'Counterexample'}, from power-up:</div>
      <table class="spec"><tr>${r.inputs.map(n => `<th>${esc(n)}</th>`).join('')}${r.outputs.map((n, i) => `<th class="${i ? '' : 'sep'}">${esc(n)}</th>`).join('')}</tr>
      ${rows.map(st => `<tr>${st.ins.map(b => `<td class="${b ? 'one' : ''}">${b ? 1 : 0}</td>`).join('')}${st.a.map((v, i) => v === st.b[i]
        ? `<td class="${i ? '' : 'sep'}">${v}</td>` : `<td class="${i ? '' : 'sep'} mis" title="this design ${v}, the other ${st.b[i]}">${v}/${st.b[i]}</td>`).join('')}</tr>`).join('')}</table>
      <div class="note">Where they differ: this design / the other.</div>`;
  }
  el.innerHTML = s;
}

/* ---- Timing ---- */

function timingTab() {
  $('#ftool').innerHTML = `<div class="note">Delays are in gate delays: one CMOS stage (an inverter or NAND) is 1. A library chip counts as the longest path through the design that fabricated it.</div>
    <button id="tmGo" class="prime" style="margin-top:8px">Find critical path</button><div id="tmOut"></div>
    <div class="ptitle" style="padding:14px 0 4px">Timing mode</div>
    <div class="note">Simulates with these delays, so glitches, oscillation and setup problems show in the waveform. Press <kbd>T</kbd> or use the waveform panel. Diagnostics and benches always run zero-delay.</div>
    <div id="tmLive"></div>`;
  $('#tmGo').onclick = async () => {
    try { G.crit = await api('POST', '/api/timing', G.circuit); } catch (e) { G.crit = {error: e.message}; }
    timingResult();
  };
  timingResult();
}

function timingResult() {
  const el = $('#tmOut'); if (!el) return;
  const r = G.crit;
  let s = '';
  if (r && r.error) s = `<div class="note mis">${esc(r.error)}</div>`;
  else if (r) {
    s = `<div class="verdict ok">${r.depth == null ? 'No output depends on an input' : `Critical path: ${r.depth} gate delays`}</div>`;
    if (r.loops) s += `<div class="note mis">The design has a loop with no register in it (a latch or oscillator). It was cut to measure the rest.</div>`;
    if (r.path.length) s += `<div class="note" style="margin-top:6px">Path, input first: ${r.path.map(p => `<b>${esc(p.what)}</b> <span class="tm">${esc(p.part)} @${p.at}</span>`).join(' → ')}</div>`;
    s += `<div class="kv">${r.outputs.map(([n, t]) => `<span class="k">${esc(n)}</span><span class="v">${t == null ? 'not driven from an input' : t}</span>`).join('')}</div>`;
  }
  el.innerHTML = s;
  const live = $('#tmLive');
  if (live) {
    const t = G.timing;
    live.innerHTML = !G.timed ? '<div class="note">Timing mode is off.</div>' : !t ? '' :
      `<div class="kv"><span class="k">Last settle</span><span class="v">${t.max_settle} gate delays</span>
       <span class="k">Setup problems</span><span class="v ${t.violations.length ? 'mis' : ''}">${t.violations.length || 'none'}</span></div>`;
  }
}

/* ---- Export ---- */

function exportTab() {
  const b = benchSummary();
  $('#ftool').innerHTML = `<div class="note">Write this design out for other tools. Custom components become modules of their own.</div>
    <div class="verdict ${b ? (b.ok ? 'ok' : 'bad') : 'warn'}">${b ? esc(b.text) : 'No test bench yet'}</div>
    <div class="note">${b && b.ok ? 'Exporting a design that passes its bench.' : 'Write a bench (Bench tab) so you export something you have verified.'}</div>
    <div class="row"><button id="exV" class="prime">Verilog (.v)</button><button id="exK">KiCad netlist (.net)</button></div>
    <div class="note" style="margin-top:8px"><b>Verilog</b> goes to simulators and FPGA tools. Library chips are behavioural modules matching the engine, checked against it in Icarus Verilog. Transistors become switch primitives, which simulate but do not synthesise.<br><b>KiCad netlist</b> goes to PCB layout. Parts have no footprints: a library chip is a function, so choose a real part for each (a 4-bit adder might be a 74HC283).</div>
    <pre id="exOut" class="ex hidden"></pre>`;
  const go = async fmt => {
    let r;
    try { r = await api('POST', '/api/export', {circuit: G.circuit, name: lvl().name, format: fmt}); } catch (e) { r = {error: e.message}; }
    const pre = $('#exOut'); pre.classList.remove('hidden');
    if (r.error) { pre.textContent = r.error; pre.classList.add('mis'); return; }
    pre.classList.remove('mis');
    pre.textContent = r.text.split('\n').slice(0, 40).join('\n') + (r.text.split('\n').length > 40 ? '\n...' : '');
    const url = URL.createObjectURL(new Blob([r.text], {type: 'text/plain'}));
    const a = document.createElement('a'); a.href = url; a.download = r.filename; document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    say(`Exported ${r.filename}.`, 'good');
  };
  $('#exV').onclick = () => go('verilog');
  $('#exK').onclick = () => go('kicad');
}

export function renderTools(tab) {
  ({bench: benchTab, equiv: equivTab, timing: timingTab, export: exportTab})[tab]?.();
}

/* Update results only, leaving what the user is typing alone. */
export function refreshTools(tab) {
  if (!$('#ftool') || $('#ftool').dataset.tab !== tab) return;
  if (tab === 'bench') benchResult();
  else if (tab === 'timing') timingResult();
}
