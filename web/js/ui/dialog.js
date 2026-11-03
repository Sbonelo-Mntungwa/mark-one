/* In-app dialogs for confirmations and short forms. Each returns a promise:
   null when cancelled, otherwise true (confirm) or the field values (form). */

import {$, esc} from '../core/dom.js';

let open = null;

/* opts: {title, body, ok, cancel, danger, fields: [{name, label, value, max, hint}]} */
export function dialog(opts) {
  if (open) open(null);
  const el = $('#dialog'), fields = opts.fields || [];
  el.innerHTML = `<form class="dcard panel${opts.danger ? ' danger' : ''}" role="alertdialog" aria-modal="true" aria-labelledby="dTitle">
    <div class="dhd">${opts.danger ? '<span class="dwarn" aria-hidden="true">!</span>' : ''}<h3 id="dTitle">${esc(opts.title)}</h3></div>
    ${opts.body ? `<p class="dbody">${opts.body}</p>` : ''}
    ${fields.map((f, i) => `<label class="dfield"><span>${esc(f.label)}</span>
      <input name="${esc(f.name)}" value="${esc(f.value || '')}" maxlength="${f.max || 60}" autocomplete="off" ${i ? '' : 'autofocus'}>
      ${f.hint ? `<small>${esc(f.hint)}</small>` : ''}</label>`).join('')}
    <div class="derr" id="dErr"></div>
    <div class="drow2"><button type="button" id="dCancel">${esc(opts.cancel || 'Cancel')}</button>
      <button type="submit" class="${opts.danger ? 'dangerbtn' : 'prime'}">${esc(opts.ok || 'OK')}</button></div></form>`;
  el.classList.remove('hidden');
  const form = el.querySelector('form');
  (form.querySelector('input') || form.querySelector('[type=submit]')).focus();
  return new Promise(res => {
    const done = v => { open = null; el.classList.add('hidden'); el.innerHTML = ''; removeEventListener('keydown', esc_, true); res(v); };
    const esc_ = e => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); done(null); } };
    open = done;
    addEventListener('keydown', esc_, true);
    $('#dCancel').onclick = () => done(null);
    el.onclick = e => { if (e.target === el) done(null); };
    form.onsubmit = e => {
      e.preventDefault();
      if (!fields.length) { done(true); return; }
      const v = Object.fromEntries(fields.map(f => [f.name, form.elements[f.name].value.trim()]));
      const missing = fields.find(f => f.required && !v[f.name]);
      if (missing) { $('#dErr').textContent = `${missing.label} is required.`; return; }
      done(v);
    };
  });
}

/* Whether a dialog is showing. */
export function dialogOpen() { return !!open; }
