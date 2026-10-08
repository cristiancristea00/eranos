/**
 * The loading of the WebAssembly module.
 *
 * @packageDocumentation
 */

import { openWasm } from '@ubjs/wasm';

import bindings from './internal/eranos.js';
import { PLAYER_DEFINITIONS, setNativeModule } from './internal/eranos-ffi.js';
import * as glue from './internal/eranos_web_bg.js';

/** The source of the module, as the player of the generated code takes it: a URL, a path, a `Response`, or its bytes. */
export type WasmSource = Parameters<typeof openWasm>[0];

/**
 * The JavaScript modules the WebAssembly module imports, by the name it imports each under: wasm-bindgen's glue alone. The
 * player fills an import no module supplies with a function that throws when called, so an import missing here fails at
 * its first call, naming itself.
 */
const imports: ReadonlyMap<string, Record<string, unknown>> = new Map([['./eranos_web_bg.js', glue]]);

/** The first call's loading, which every later call returns. */
let loading: Promise<void> | undefined;

/**
 * Loads the WebAssembly module, `eranos_web.wasm`, which every other call of the package runs in; later calls return
 * the first one's promise. The host names the module: a bundler rewrites its URL when it copies the file, so there is no
 * default.
 *
 * @param source - The module: a URL, a path, a `Response`, or its bytes
 * @returns A promise of the module's loading
 */
export function init(source: WasmSource): Promise<void> {
    loading ??= load(source);
    return loading;
}

/**
 * Opens the module, with wasm-bindgen's glue for the imports that the library's clock and random source reach through
 * wasm-bindgen, and registers the functions of the generated code with it.
 */
async function load(source: WasmSource): Promise<void> {
    const module = await openWasm(source, { resolveModule: (name: string) => Promise.resolve(imports.get(name)) });
    setNativeModule(module.registerSync(PLAYER_DEFINITIONS));
    bindings.initialize();
}
