/* tslint:disable */
/* eslint-disable */
/**
* @param {number} width
* @param {number} height
* @param {string} bind_id
* @returns {Promise<WgpuToyRenderer>}
*/
export function create_renderer(width: number, height: number, bind_id: string): Promise<WgpuToyRenderer>;
/**
*/
export class SourceMap {
  free(): void;
}
/**
*/
export class WgpuToyRenderer {
  free(): void;
/**
*/
  render(): void;
/**
* @returns {string}
*/
  prelude(): string;
/**
* @param {string} shader
* @returns {Promise<any>}
*/
  preprocess(shader: string): Promise<any>;
/**
* @param {SourceMap} source
*/
  compile(source: SourceMap): void;
/**
* @param {number} t
*/
  set_time_elapsed(t: number): void;
/**
* @param {number} t
*/
  set_time_delta(t: number): void;
/**
* @param {number} x
* @param {number} y
*/
  set_mouse_pos(x: number, y: number): void;
/**
* @param {boolean} click
*/
  set_mouse_click(click: boolean): void;
/**
* @param {number} keycode
* @param {boolean} keydown
*/
  set_keydown(keycode: number, keydown: boolean): void;
/**
* @param {(string)[]} names
* @param {Float32Array} values
*/
  set_custom_floats(names: (string)[], values: Float32Array): void;
/**
* @param {boolean} pass_f32
*/
  set_pass_f32(pass_f32: boolean): void;
/**
* @param {number} width
* @param {number} height
* @param {number} scale
*/
  resize(width: number, height: number, scale: number): void;
/**
*/
  reset(): void;
/**
* @param {Function} callback
*/
  on_success(callback: Function): void;
/**
* @param {number} index
* @param {Uint8Array} bytes
*/
  load_channel(index: number, bytes: Uint8Array): void;
/**
* @param {number} index
* @param {Uint8Array} bytes
*/
  load_channel_hdr(index: number, bytes: Uint8Array): void;
}
