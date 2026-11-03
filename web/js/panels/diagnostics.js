/* Running diagnostics and showing the result in place. */

import {api} from '../core/api.js';
import {$, esc} from '../core/dom.js';
import {refreshState, runSim} from '../core/session.js';
import {G, lvl} from '../core/state.js';
import {say} from '../ui/log.js';

export const sv = v => `<span class="${v === '1' ? 'one' : ''}">${esc(v)}</span>`;

export async function runDiagnostics() {
  if (G.free || lvl().mode === 'sandbox') return;
  const l = lvl(), box = $('#result');
  box.className = ''; box.innerHTML = '<div class="note">Running diagnostics...</div>';
  let r;
  try { r = await api('POST', `/api/levels/${l.id}/verify`, G.circuit); }
  catch (e) { box.className = 'fail'; box.innerHTML = `<div class="hd">Server error</div>${esc(e.message)}`; return; }
  if (r.pass) {
    box.className = 'pass';
    const was = l.status, idx = G.idx;
    await refreshState();
    const next = G.levels[idx + 1];
    const parts = [r.program ? `Output matched on your computer, in ${r.cost} cycles.` : `${r.checked} check${r.checked === 1 ? '' : 's'} passed.`];
    if (l.chip) parts.push(`${l.label} is now a chip in the fabricator.`);
    if (next && was !== 'done') parts.push(`Mk ${idx + 2}, ${next.name}, is unlocked in the Marks list.`);
    box.className = 'pass';
    box.innerHTML = `<div class="hd">Diagnostics passed</div><div class="note">${parts.map(esc).join(' ')}</div>`;
    say(parts.join(' '), 'good');
    const row = next && $(`#marks [data-lv="${idx + 1}"]`);
    if (row && was !== 'done') { row.classList.add('fresh'); row.scrollIntoView({block: 'nearest'}); }
    return;
  }
  box.className = 'fail';
  if (r.error) { box.innerHTML = `<div class="hd">Cannot run diagnostics</div><div class="note">${esc(r.error)}</div>`; say(r.error, 'bad'); return; }
  if (r.program && !r.failure) {
    const pr = r.program;
    const firstBad = pr.outs.findIndex((v, i) => v !== pr.want[i]);
    box.innerHTML = `<div class="hd">Program output does not match</div><div class="kv">
      <span class="k">Wanted</span><span class="v">${pr.want.join(', ')}</span>
      <span class="k">Got</span><span class="v">${pr.outs.length ? pr.outs.map((v, i) => `<span class="${v !== pr.want[i] ? 'mis' : ''}">${v}</span>`).join(', ') : 'nothing'}</span>
      <span class="k">Halted</span><span class="v">${pr.halted ? `yes, after ${pr.cycles} cycles` : 'no, still running after 2000 cycles'}</span></div>
      <div class="note">${!pr.halted ? 'The program never reached HLT. Check your loop exit.' : firstBad >= 0 ? `Output ${firstBad + 1} is wrong.` : pr.outs.length < pr.want.length ? 'It halted too early.' : 'It output too many values.'}</div>`;
    say('Program output does not match the goal.', 'bad');
    return;
  }
  const f = r.failure;
  let s = `<div class="hd">Failed at step ${f.step} of ${f.total}</div><div class="note">${esc(f.note)}</div><div class="kv">`;
  s += `<span class="k">Inputs</span><span class="v">${f.inputs.map(([n, v]) => `${esc(n)}=${sv(v)}`).join(' ')}</span>`;
  s += `<span class="k">Expected</span><span class="v">${f.expected.map(([n, v]) => `${esc(n)}=${sv(v)}`).join(' ')}</span>`;
  s += `<span class="k">Got</span><span class="v">${f.got.map(([n, v], i) => `<span class="${v !== f.expected[i][1] ? 'mis' : ''}">${esc(n)}=${esc(v)}</span>`).join(' ')}</span></div>`;
  if (f.trace && f.trace.length > 1) {
    s += `<div class="note">Last steps, as ${l.inputs.map(esc).join(' ')} giving ${l.outputs.map(esc).join(' ')}:</div><div class="kv">`;
    f.trace.forEach(t => { s += `<span class="k v">${t.inputs.join('')}</span><span class="v">${t.expected.join('')}</span>`; });
    s += `</div>`;
  }
  if (!l.sequential) s += `<button id="loadFail">Load these inputs on the board</button>`;
  box.innerHTML = s;
  say(`Diagnostics failed at step ${f.step}. ${f.note}`, 'bad');
  const lb = $('#loadFail');
  if (lb) lb.onclick = () => { G.inputs = f.inputs.map(([, v]) => v === '1'); runSim(); say('Failing inputs loaded. The red output shows what to fix.', 'warn'); };
}
