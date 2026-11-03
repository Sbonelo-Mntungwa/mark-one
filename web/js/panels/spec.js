/* Spec panel: truth table, behaviour, or instruction set. */

import {$, bit, esc} from '../core/dom.js';
import {G, lvl} from '../core/state.js';
import {isaTable} from '../isa/assembler.js';
import {renderFreeSpec} from '../modes/free.js';

export function renderSpec() {
  const l = lvl();
  if (l.mode === 'free') { renderFreeSpec(); return; }
  if (l.mode === 'program') {
    $('#specTitle').textContent = 'Goal';
    $('#spec').innerHTML = `<div class="note">Output these values, in order, then halt:</div><div class="goal">${l.goal.join(', ')}</div>
      <div class="note">Diagnostics run the program in your ROM on the computer you built.</div><div style="margin-top:10px">${isaTable(true)}</div>`;
    return;
  }
  if (l.mode === 'sandbox') {
    $('#specTitle').textContent = 'Workshop';
    $('#spec').innerHTML = `<div class="note">No goal and no diagnostics. Every chip you have fabricated is in the dock.</div><div style="margin-top:10px">${isaTable(true)}</div>`;
    return;
  }
  if (['ctrl', 'cpu', 'computer'].includes(l.id)) {
    $('#specTitle').textContent = 'Instruction set';
    $('#spec').innerHTML = `<div class="note">${l.id === 'ctrl' ? 'Each opcode and the signals it raises:' : 'One byte per instruction: opcode in the high 4 bits, operand n in the low 4.'}</div>${isaTable(l.id !== 'ctrl', l.id === 'ctrl')}`;
    return;
  }
  if (!l.table) {
    $('#specTitle').textContent = l.sequential ? 'Behaviour' : 'Spec';
    $('#spec').innerHTML = `<div class="note">${l.sequential
      ? 'Sequential: the outputs depend on history. Diagnostics drive a long random sequence and check every step against the spec.'
      : `${l.inputs.length} inputs is ${Math.pow(2, l.inputs.length).toLocaleString()} combinations. Diagnostics check ${l.inputs.length > 12 ? 'about 3,000 random ones plus the extremes' : 'every one'}.`}</div>`;
    return;
  }
  $('#specTitle').textContent = 'Spec';
  let s = `<table class="spec"><tr>${l.inputs.map(n => `<th>${esc(n)}</th>`).join('')}${l.outputs.map((n, i) => `<th class="${i ? '' : 'sep'}">${esc(n)}</th>`).join('')}<th class="sep">Live</th></tr>`;
  for (const r of l.table) {
    const cur = r.ins.every((b, i) => b === !!G.inputs[i]);
    let now = '';
    if (cur) now = r.outs.every((v, i) => v === G.outputs[i]) ? '<span class="ok">match</span>' : '<span class="bad">off</span>';
    s += `<tr class="${cur ? 'cur' : ''}">${r.ins.map(b => `<td class="${b ? 'one' : ''}">${bit(b)}</td>`).join('')}${r.outs.map((v, i) => `<td class="${v === '1' ? 'one' : ''}${i ? '' : ' sep'}">${v}</td>`).join('')}<td class="sep">${now}</td></tr>`;
  }
  $('#spec').innerHTML = s + '</table>';
}
