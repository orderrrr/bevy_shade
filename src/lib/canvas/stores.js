// stores.js
import { writable } from 'svelte/store';

export const canvasEl = writable(null);
export const wgpuAvailability = writable('unknown');
export const wgpuRenderer = writable(null);
export const needsInitialReset = writable(true);
export const parseError = writable(null);
export const manualReload = writable(false);
export const playing = writable(true);

export const isSafeContext = (context) =>
    context !== null && context.__wbg_ptr !== 0;
