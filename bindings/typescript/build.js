// The package's bundle: one ES module of the layer and the generated code, and the WebAssembly module beside it. The
// player's packages stay imports, so that the app's resolver picks their browser or their Node build. A warning of the
// bundler fails the build, as the compiler's warnings do.

import { copyFileSync } from 'node:fs';
import process from 'node:process';

import { build } from 'esbuild';

const result = await build({
    bundle: true,
    entryPoints: ['src/index.ts'],
    external: ['@ubjs/core', '@ubjs/wasm'],
    format: 'esm',
    logLevel: 'warning',
    outfile: 'dist/eranos.js',
    platform: 'neutral',
    sourcemap: true,
    target: 'es2024',
});
copyFileSync('src/internal/eranos_web.wasm', 'dist/eranos_web.wasm');
if (result.warnings.length > 0) {
    process.exitCode = 1;
}
