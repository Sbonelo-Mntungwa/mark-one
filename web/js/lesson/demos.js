/* Interactive lesson demos. */

import {api} from '../core/api.js';
import {$, esc, RM} from '../core/dom.js';
import {isaTable} from '../isa/assembler.js';
import {lessonOpen} from './player.js';

export const flowPath = (d, on) => `${on ? `<path d="${d}" class="halo"/>` : ''}<path d="${d}" fill="none" stroke="${on ? '#ffb84d' : '#2a4f6d'}" stroke-width="4" stroke-linecap="round"/>${on && !RM ? `<path d="${d}" class="flow"/>` : ''}`;

export const LG = '<defs><filter id="lg" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="2.4" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge></filter></defs>';

export const T = (x, y, s, cls = '', extra = '') => `<text x="${x}" y="${y}" fill="${cls || '#d8f6ff'}" font-family="Chakra Petch, sans-serif" font-size="15" text-anchor="middle" dominant-baseline="central" ${extra}>${s}</text>`;

export function fetSVG(x, isP, g) {
  const on = isP ? !g : g;
  return `<g transform="translate(${x},0)">
    ${flowPath('M60,20 V70', on)}${flowPath('M60,130 V180', on)}
    ${on ? '<path d="M60,70 V130" class="halo"/>' : ''}<path d="M60,70 V130" stroke="${on ? '#ffb84d' : '#2a4f6d'}" stroke-width="6"/>
    <path d="M8,100 H${isP ? 30 : 40} M42,74 V126" stroke="#8fd9f5" stroke-width="2.5" fill="none"/>
    ${isP ? '<circle cx="35" cy="100" r="5" fill="none" stroke="#8fd9f5" stroke-width="2"/>' : ''}
    ${T(60, 10, isP ? 'S' : 'D', '#7fa6bd')}${T(60, 192, isP ? 'D' : 'S', '#7fa6bd')}
    ${T(84, 100, isP ? 'PMOS' : 'NMOS', '#8fd9f5', 'text-anchor="start"')}
    ${T(60, 222, on ? 'conducting' : 'open', on ? '#ffb84d' : '#48687e')}</g>`;
}

export const bitBtn = (on, label, data) => `<button class="bitb${on ? ' on' : ''}" ${data}>${on ? 1 : 0}${label ? `<small>${esc(label)}</small>` : ''}</button>`;

export const DEMOS = {
  rails(el) {
    el.innerHTML = `<div class="dm"><svg viewBox="0 0 400 200">${LG}${flowPath('M40,60 H360', true)}${flowPath('M40,140 H360', false)}
      ${T(40, 34, 'VCC', '#ffb84d', 'text-anchor="start"')}${T(360, 34, '1', '#ffb84d')}${T(40, 114, 'GND', '#7fa6bd', 'text-anchor="start"')}${T(360, 114, '0', '#7fa6bd')}</svg>
      <div class="cap">On the board, <b>gold</b> lines carry a 1 and steel lines are at 0. Watch the charge move along the gold ones.</div></div>`;
  },
  switch(el) {
    let a = false;
    const draw = () => {
      el.innerHTML = `<div class="dm"><svg viewBox="0 0 400 160">${LG}${flowPath('M60,80 H340', a)}
        <circle cx="350" cy="80" r="16" fill="${a ? '#ffb84d' : '#13283a'}" stroke="#5ad7ff" ${a ? 'filter="url(#lg)"' : ''}/>${T(60, 44, 'A', '#d8f6ff')}${T(350, 124, 'Y', '#d8f6ff')}</svg>
        <div class="brow"><span class="nm">A</span>${bitBtn(a, '', 'id="dA"')}</div><div class="cap">Toggle A. Y is the same node, so it always matches.</div></div>`;
      $('#dA').onclick = () => { a = !a; draw(); };
    };
    draw();
  },
  mosfet(el) {
    const g = [false, false];
    const draw = () => {
      el.innerHTML = `<div class="dm"><svg viewBox="0 0 380 240">${LG}${fetSVG(30, false, g[0])}${fetSVG(210, true, g[1])}</svg>
        <div class="brow"><span class="nm">NMOS G</span>${bitBtn(g[0], '', 'data-g="0"')}<span class="nm">PMOS G</span>${bitBtn(g[1], '', 'data-g="1"')}</div>
        <div class="cap">The circle on the PMOS gate means "inverted": it conducts on <b>0</b>.</div></div>`;
      el.querySelectorAll('[data-g]').forEach(b => b.onclick = () => { g[+b.dataset.g] = !g[+b.dataset.g]; draw(); });
    };
    draw();
  },
  binary(el) {
    const bits = [true, false, true, false, false, false, false, false];
    const draw = () => {
      const n = bits.reduce((a, b, k) => a + (b ? 1 << k : 0), 0);
      el.innerHTML = `<div class="dm"><div class="brow">${bits.map((_, i) => 7 - i).map(k => bitBtn(bits[k], String(1 << k), `data-k="${k}"`)).join('')}</div>
        <div style="font-family:var(--num);font-size:56px;color:var(--gold);line-height:1">${n}</div>
        <div class="cap">${bits.map((b, k) => b ? 1 << k : 0).filter(Boolean).reverse().join(' + ') || '0'}${n ? ' = ' + n : ''}</div></div>`;
      el.querySelectorAll('[data-k]').forEach(x => x.onclick = () => { bits[+x.dataset.k] = !bits[+x.dataset.k]; draw(); });
    };
    draw();
  },
  twos(el) {
    let n = 5;
    const draw = () => {
      const row = (v, label) => `<div class="brow"><span class="nm" style="min-width:84px">${label}</span>${[7, 6, 5, 4, 3, 2, 1, 0].map(k => `<span class="outb ${v >> k & 1 ? 'v1' : ''}">${v >> k & 1}</span>`).join('')}</div>`;
      const inv = ~n & 255, neg = (inv + 1) & 255;
      const signed = v => v > 127 ? v - 256 : v;
      el.innerHTML = `<div class="dm">${row(n, `B = ${n}`)}${row(inv, 'NOT B')}${row(neg, 'NOT B + 1')}
        <div class="cap">Read as signed bytes, B is <b>${signed(n)}</b> and the last row is <b>${signed(neg)}</b>.${n === 128 ? ' -128 is its own negative: +128 does not fit in a signed byte.' : ''}</div>
        <div class="brow"><button id="dm">B - 1</button><button id="dp">B + 1</button></div></div>`;
      $('#dm').onclick = () => { n = (n + 255) & 255; draw(); };
      $('#dp').onclick = () => { n = (n + 1) & 255; draw(); };
    };
    draw();
  },
  isa(el) { el.innerHTML = `<div class="dm" style="align-items:stretch">${isaTable(false)}<div class="cap" style="align-self:center">Example: <b>LDI 5</b> is opcode 1, operand 5, so the byte 15 in hex.</div></div>`; },
  asm(el) {
    el.innerHTML = `<div class="dm" style="align-items:stretch"><pre class="asm"><span class="c">; syntax only, not a solution</span>
<span class="l">top:</span>  LDI 3      <span class="c">; label "top" is address 0</span>
      STA 5      <span class="c">; RAM[5] = ACC</span>
      ADD 5      <span class="c">; ACC = ACC + RAM[5]</span>
      JMP top    <span class="c">; labels work as operands</span></pre><div class="cap" style="align-self:center">Each line becomes one byte. <b>LDI 3</b> is 13 in hex.</div></div>`;
  },
  fetch(el) {
    const node = (x, y, t) => `<circle cx="${x}" cy="${y}" r="44" fill="rgba(90,215,255,.06)" stroke="#5ad7ff"/>${T(x, y, t)}`;
    el.innerHTML = `<div class="dm"><svg viewBox="0 0 400 300">${LG}
      <circle cx="200" cy="150" r="100" fill="none" stroke="#ffb84d" stroke-dasharray="6 8" ${RM ? '' : 'class="spin" style="transform-origin:200px 150px;transform-box:view-box"'}/>
      ${node(200, 50, 'PC')}${node(300, 200, 'ROM')}${node(100, 200, 'CPU')}
      ${T(268, 104, 'address', '#7fa6bd')}${T(200, 262, 'instruction', '#7fa6bd')}${T(124, 104, 'next PC', '#7fa6bd')}</svg>
      <div class="cap">One trip around the loop per clock cycle.</div></div>`;
  },
  model(el, l) {
    if (!l.chip && l.mode !== 'circuit') { DEMOS.isa(el); return; }
    const ins = l.inputs.map(() => false);
    let outs = l.outputs.map(() => 'X'), first = true;
    const groups = names => {
      const g = [];
      names.forEach((n, i) => {
        const m = n.match(/^([A-Z]+?)(\d+)$/), key = m && names.filter(x => x.startsWith(m[1]) && /^\d+$/.test(x.slice(m[1].length))).length > 1 ? m[1] : n;
        let e = g.find(x => x.key === key);
        if (!e) g.push(e = {key, idx: []});
        e.idx.push(i);
      });
      return g;
    };
    const gi = groups(l.inputs), go = groups(l.outputs);
    const val = (idx, get) => idx.reduce((a, i, k) => a + (get(i) ? 1 << k : 0), 0);
    const draw = () => {
      const inRows = gi.map(g => {
        if (g.key === 'CLK') return `<div class="brow"><span class="nm">CLK</span>${bitBtn(ins[g.idx[0]], '', `data-i="${g.idx[0]}"`)}<button id="dPulse">Pulse</button></div>`;
        return `<div class="brow"><span class="nm">${esc(g.key)}</span>${g.idx.slice().reverse().map(i => bitBtn(ins[i], g.idx.length > 1 ? l.inputs[i] : '', `data-i="${i}"`)).join('')}${g.idx.length > 1 ? `<span class="dec">= ${val(g.idx, i => ins[i])}</span>` : ''}</div>`;
      }).join('');
      const outRows = go.map(g => {
        const known = g.idx.every(i => outs[i] === '0' || outs[i] === '1');
        return `<div class="brow"><span class="nm">${esc(g.key)}</span>${g.idx.slice().reverse().map(i => `<span class="outb v${outs[i]}">${outs[i]}${g.idx.length > 1 ? `<small>${esc(l.outputs[i])}</small>` : ''}</span>`).join('')}${g.idx.length > 1 ? `<span class="dec">= ${known ? val(g.idx, i => outs[i] === '1') : '?'}</span>` : ''}</div>`;
      }).join('');
      el.innerHTML = `<div class="dm">${inRows}<div class="chipbox">${esc(l.label)}</div>${outRows}
        <div class="cap">The finished component. Click the inputs and see what it does before you build it.</div></div>`;
      el.querySelectorAll('[data-i]').forEach(b => b.onclick = () => { ins[+b.dataset.i] = !ins[+b.dataset.i]; ask(); });
      const pb = el.querySelector('#dPulse');
      if (pb) pb.onclick = async () => { const k = l.inputs.indexOf('CLK'); ins[k] = true; await ask(); ins[k] = false; await ask(); };
    };
    const ask = async () => {
      try {
        const r = await api('POST', '/api/demo', {level: l.id, inputs: ins, reset: first});
        first = false;
        if (r.outputs) outs = r.outputs;
      } catch (e) {}
      if (lessonOpen()) draw();
    };
    draw(); ask();
  },
};
