/* Instruction set, assembler and disassembler. */

import {pv} from '../board/geometry.js';
import {esc} from '../core/dom.js';

export const ISA = [
  ['NOP', 0, false, 'Do nothing', ''],
  ['LDI', 1, true, 'ACC = n', 'BIMM PASS ACCW'],
  ['ADI', 2, true, 'ACC = ACC + n', 'BIMM ACCW'],
  ['ADD', 3, true, 'ACC = ACC + RAM[n]', 'ACCW'],
  ['SUB', 4, true, 'ACC = ACC - RAM[n]', 'SUB ACCW'],
  ['LDA', 5, true, 'ACC = RAM[n]', 'PASS ACCW'],
  ['STA', 6, true, 'RAM[n] = ACC', 'RAMW'],
  ['JMP', 7, true, 'PC = n', 'JUMP'],
  ['JZ', 8, true, 'If ACC is 0: PC = n', 'JUMP if Z'],
  ['OUT', 9, false, 'OUT = ACC', 'OUTW'],
  ['HLT', 15, false, 'Stop', 'HALT'],
];

export const STARTER = '; Write your program here.\n; One instruction per line, labels end with a colon.\n\nHLT';

export function isaTable(compact, signals) {
  return `<table class="isa"><tr><th>Op</th><th>Hex</th><th>Does</th>${signals ? '<th>Signals</th>' : ''}</tr>${ISA.map(([m, o, n, d, s]) =>
    `<tr><td class="m">${m}${n ? ' n' : ''}</td><td class="h">${o.toString(16).toUpperCase()}${n ? 'n' : '0'}</td><td>${esc(d)}</td>${signals ? `<td class="h">${s || '-'}</td>` : ''}</tr>`).join('')}</table>`;
}

export function parseNum(t) {
  if (/^0x[0-9a-f]+$/i.test(t)) return parseInt(t, 16);
  if (/^0b[01]+$/i.test(t)) return parseInt(t.slice(2), 2);
  if (/^\d+$/.test(t)) return parseInt(t, 10);
  return null;
}

export function assemble(src) {
  const lines = String(src || '').split('\n'), labels = {}, items = [], errors = [];
  lines.forEach((raw, ln) => {
    let t = raw.replace(/[;#].*$/, '').trim(), m;
    while ((m = t.match(/^([A-Za-z_]\w*):\s*(.*)$/))) {
      const name = m[1].toLowerCase();
      if (name in labels) errors.push(`Line ${ln + 1}: label "${m[1]}" is already defined on line ${labels[name].ln + 1}.`);
      else labels[name] = {addr: items.length, ln};
      t = m[2].trim();
    }
    if (t) items.push({ln, t});
  });
  const bytes = new Array(16).fill(0), listing = [];
  if (items.length > 16) errors.push(`The ROM holds 16 instructions; this program has ${items.length}.`);
  items.slice(0, 16).forEach((it, addr) => {
    if (/^\S+\s*,/.test(it.t)) { errors.push(`Line ${it.ln + 1}: unexpected comma after "${it.t.split(/[\s,]/)[0]}".`); return; }
    const [mn, arg, extra] = it.t.split(/\s*,\s*|\s+/);
    const op = ISA.find(x => x[0] === mn.toUpperCase());
    if (!op) { errors.push(`Line ${it.ln + 1}: unknown instruction "${mn}".`); return; }
    let n = 0;
    if (op[2]) {
      if (arg === undefined) { errors.push(`Line ${it.ln + 1}: ${op[0]} needs an operand from 0 to 15.`); return; }
      n = labels[arg.toLowerCase()]?.addr ?? parseNum(arg);
      if (n == null && /^-\d+$/.test(arg)) { errors.push(`Line ${it.ln + 1}: operand ${arg} is out of range (0 to 15).`); return; }
      if (n == null) { errors.push(`Line ${it.ln + 1}: "${arg}" is not a number or label.`); return; }
      if (n < 0 || n > 15) { errors.push(`Line ${it.ln + 1}: operand ${n} is out of range (0 to 15).`); return; }
    } else if (arg !== undefined) { errors.push(`Line ${it.ln + 1}: ${op[0]} takes no operand.`); return; }
    if (extra !== undefined) { errors.push(`Line ${it.ln + 1}: too many operands.`); return; }
    bytes[addr] = (op[1] << 4) | n;
    listing.push(addr);
  });
  return {bytes, errors, count: items.length};
}

export function disasm(bytes) {
  const b = (bytes || []).slice(0, 16);
  let last = b.length - 1;
  while (last > 0 && !b[last]) last--;
  return b.slice(0, last + 1).map(x => {
    const op = ISA.find(i => i[1] === x >> 4);
    if (!op) return `NOP        ; byte ${hex2(x)} is a reserved opcode, reloaded as NOP`;
    return op[2] ? `${op[0]} ${x & 15}` : op[0];
  }).join('\n') || 'HLT';
}

export const hex2 = x => x.toString(16).toUpperCase().padStart(2, '0');

export function romAddr(p) {
  const v = [0, 1, 2, 3].map(k => pv(p.id, k));
  return v.every(x => x === '0' || x === '1') ? v.reduce((a, x, k) => a + (x === '1' ? 1 << k : 0), 0) : null;
}
