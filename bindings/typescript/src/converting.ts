/**
 * The calls into the generated code, and the conversion of what it reports into this package's errors.
 *
 * @packageDocumentation
 */

import { convertedError } from './errors.js';

/**
 * The options of an asynchronous call.
 */
export interface CallOptions {
    /** A signal whose abort cancels the call, which leaves the group as it was; the call then rejects with its reason. */
    readonly signal?: AbortSignal;
}

/**
 * Makes `call`, which calls the generated code, and throws what it reports as this package's error.
 *
 * @internal
 */
export function calling<T>(call: () => T): T {
    try {
        return call();
    } catch (error: unknown) {
        throw convertedError(error);
    }
}

/**
 * Awaits `call`, which calls the generated code with the signal of `options`, and rejects with what it reports as this
 * package's error, or with the signal's reason once it is aborted, as `fetch` does.
 *
 * @internal
 */
export async function awaiting<T>(call: (options: { signal: AbortSignal } | undefined) => Promise<T>, options: CallOptions | undefined): Promise<T> {
    const signal = options?.signal;
    try {
        return await call(signal === undefined ? undefined : { signal });
    } catch (error: unknown) {
        signal?.throwIfAborted();
        throw convertedError(error);
    }
}
