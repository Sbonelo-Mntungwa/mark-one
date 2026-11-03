/* DOM helpers and small pure utilities. */

export const $ = s => document.querySelector(s);

export const RM = matchMedia('(prefers-reduced-motion: reduce)').matches;

/* Positions snap to the board grid (GRID in board/geometry.js). */
export const snap = v => Math.round(v / 20) * 20;

export const esc = s => String(s).replace(/[&<>"]/g, c => ({'&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;'}[c]));

export const bit = b => b ? '1' : '0';

export const sleep = ms => new Promise(r => setTimeout(r, ms));
