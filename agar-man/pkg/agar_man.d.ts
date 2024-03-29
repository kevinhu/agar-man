/* tslint:disable */
/* eslint-disable */
/**
* @param {string} seed
* @param {number} min_length
* @param {number} max_num_words
* @param {string} excludes
* @param {string} includes
* @param {number} top_n
* @returns {ResultsStruct}
*/
export function js_generate(seed: string, min_length: number, max_num_words: number, excludes: string, includes: string, top_n: number): ResultsStruct;
/**
*/
export class ResultsStruct {
  free(): void;
/**
*/
  anagrams: Array<any>;
/**
*/
  partials: Array<any>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_resultsstruct_free: (a: number) => void;
  readonly __wbg_get_resultsstruct_anagrams: (a: number) => number;
  readonly __wbg_set_resultsstruct_anagrams: (a: number, b: number) => void;
  readonly __wbg_get_resultsstruct_partials: (a: number) => number;
  readonly __wbg_set_resultsstruct_partials: (a: number, b: number) => void;
  readonly js_generate: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
