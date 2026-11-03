/* The workshop log in the corner of the board. */

import {$, RM} from '../core/dom.js';

export function say(text, kind = '') {
  const box = $('#log');
  const el = document.createElement('div');
  el.className = 'll ' + kind;
  box.appendChild(el);
  [...box.children].slice(0, -1).forEach(c => c.classList.add('old'));
  while (box.children.length > 4) box.firstChild.remove();
  if (RM) { el.textContent = text; return; }
  let i = 0;
  const t = setInterval(() => { i += 2; el.textContent = text.slice(0, i); if (i >= text.length) clearInterval(t); }, 14);
}
