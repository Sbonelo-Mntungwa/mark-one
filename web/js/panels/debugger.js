/* Program debugger for boards with a ROM: single-step, breakpoints, and the
   registers and memory inside every chip while the program runs. */

import {$, esc} from '../core/dom.js';
import {boardKey, chipDef, G, lvl} from '../core/state.js';
import {hex2, ISA} from '../isa/assembler.js';

const hasRom = () => G.circuit.parts.some(p => p.kind === 'rom');

const num = bits => bits.every(b => b === '0' || b === '1') ? bits.reduce((a, b, k) => a + (b === '1' ? 2 ** k : 0), 0) : null;

export function loadBreakpoints() {
  G.breakpoints = new Set();
  try { G.breakpoints = new Set(JSON.parse(localStorage.getItem('m1-bp:' + boardKey()) || '[]')); } catch (e) {}
}

export function toggleBreakpoint(a) {
  if (G.breakpoints.has(a)) G.breakpoints.delete(a); else G.breakpoints.add(a);
  try { localStorage.setItem('m1-bp:' + boardKey(), JSON.stringify([...G.breakpoints])); } catch (e) {}
}

function partLabel(id) {
  const p = G.circuit.parts.find(q => q.id === id);
  if (!p) return id;
  const d = p.kind === 'chip' ? chipDef(p.chip) : null;
  return d ? `${d.label} ${id}` : id;
}

/* What the instruction at the current address is. */
function current() {
  const rom = G.circuit.parts.find(p => p.kind === 'rom');
  if (!rom || G.pc == null) return 'address unknown';
  const x = (rom.data || [])[G.pc] || 0, op = ISA.find(o => o[1] === x >> 4);
  return `${String(G.pc).padStart(2, '0')}: ${hex2(x)} ${op ? (op[2] ? `${op[0]} ${x & 15}` : op[0]) : 'NOP (reserved)'}`;
}

export function renderDebugger() {
  const wrap = $('#dbgWrap'); if (!wrap) return;
  if (!hasRom()) { wrap.classList.add('hidden'); return; }
  wrap.classList.remove('hidden');
  const prev = new Map((G.prevInternals || []).map(c => [c.part, c]));
  const changed = (part, kind, name, i, v) => {
    const c = prev.get(part); if (!c) return false;
    const o = (c[kind] || []).find(x => x.name === name); if (!o) return false;
    return kind === 'regs' ? num(o.bits) !== v : o.bytes[i] !== v;
  };
  let s = `<div class="row"><button id="dbStep" title="Execute one instruction (Space)">Step</button><button id="dbRun" class="prime" title="Run to a breakpoint or HLT (H)">Run</button><button id="dbReset" title="Restart from address 0 (X)">Reset</button></div>
    <div class="dbcur">Next: <b>${esc(current())}</b></div>`;
  if (lvl().inputs.indexOf('CLK') < 0) s += `<div class="note mis">Name an input CLK and wire it to the clock to step and run.</div>`;
  if (!G.internals.length) s += `<div class="note">No chip on this board stores state yet.</div>`;
  for (const c of G.internals) {
    s += `<div class="dbchip"><div class="dbname">${esc(partLabel(c.part))}</div>`;
    if (c.regs.length) s += `<div class="kv">${c.regs.map(r => {
      const v = num(r.bits);
      return `<span class="k">${esc(r.name)}</span><span class="v${changed(c.part, 'regs', r.name, 0, v) ? ' chg' : ''}">${v == null ? r.bits.slice().reverse().join('') : `${v} <small>0x${v.toString(16).toUpperCase().padStart(Math.ceil(r.bits.length / 4), '0')}</small>`}</span>`;
    }).join('')}</div>`;
    for (const m of c.mems) {
      s += `<div class="note">${esc(m.name)}</div><div class="mem">${m.bytes.map((b, i) =>
        `<span class="${changed(c.part, 'mems', m.name, i, b) ? 'chg' : ''}${b ? ' nz' : ''}" title="[${i}] = ${b}"><i>${i.toString(16).toUpperCase()}</i>${hex2(b)}</span>`).join('')}</div>`;
    }
    s += '</div>';
  }
  s += `<div class="note">Click an address in the listing above to set a breakpoint. Run stops before that instruction executes.</div>`;
  $('#dbg').innerHTML = s;
  $('#dbStep').onclick = () => import('../machine/clock.js').then(m => m.pulse());
  $('#dbRun').onclick = () => import('../machine/clock.js').then(m => m.runToHalt());
  $('#dbReset').onclick = () => import('../core/session.js').then(m => m.runSim(true));
}
