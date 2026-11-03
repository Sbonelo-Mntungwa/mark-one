/* Boot screen shown once per session. */

import {$, RM} from '../core/dom.js';
import {doneCount, G} from '../core/state.js';
import {reactorSVG} from './reactor.js';

export function boot() {
  return new Promise(res => {
    let seen = false;
    try { seen = sessionStorage.getItem('lf-boot') === '1'; sessionStorage.setItem('lf-boot', '1'); } catch (e) {}
    if (seen || RM) return res();
    const b = $('#boot'), d = doneCount(), n = G.levels.length;
    b.innerHTML = `<div class="word" aria-label="Mark One">MARK<span class="sp"></span><svg viewBox="-60 -60 120 120" aria-hidden="true">${reactorSVG(true)}</svg>NE</div><div class="ln" id="bl"></div><div class="skip">Click to skip</div>`;
    b.classList.remove('hidden');
    const segs = b.querySelectorAll('.seg'), lines = [
      'Mark One systems online',
      `<b>${n}</b> Marks on file, <b>${d}</b> fabricated`,
      d ? 'Restoring fabricator inventory' : 'Fabricator empty. Starting from raw charge',
    ];
    let t = 0, stop = false;
    const finish = () => { if (stop) return; stop = true; b.classList.add('hidden'); res(); };
    b.onclick = finish;
    segs.forEach((s, i) => setTimeout(() => { if (G.levels[i]) s.classList.add(G.levels[i].status === 'done' ? 'done' : G.levels[i].status === 'open' ? 'open' : 'locked'); }, 300 + i * 45));
    lines.forEach((l, i) => setTimeout(() => { if (!stop) $('#bl').innerHTML = l; }, 200 + i * 650));
    setTimeout(finish, 2400);
  });
}
