import { writable } from 'svelte/store';

export const canvas_el = writable(null);
export const wgpu_availability = writable('unknown');
export const wgpu_renderer = writable(null);
export const needs_initial_reset = writable(true);
export const parse_error = writable(null);
export const manual_reload = writable(false);
export const playing = writable(true);
export const code = writable("");
export const resize = writable(false);
export const resize_timer = writable(false);

export const get = (store) => {
    let value;
    store.subscribe(v => value = v)();
    return value;
};
export const get_code = () => get(code);
export const get_wgpu_renderer = () => get(wgpu_renderer);
export const get_needs_initial_reset = () => get(needs_initial_reset);
export const get_parse_error = () => get(parse_error);
export const get_manual_reload = () => get(manual_reload);

export const is_safe_context = (context) =>
    context !== null && context !== undefined && context.__wbg_ptr !== 0;
