/**
 * The calls into the generated code, the check of the integers that cross into it, and the conversion of what it reports
 * into this package's errors.
 *
 * @packageDocumentation
 */

import { type EranosError, convertedError } from './errors.js';

/** The smallest signed 32-bit integer, the type of the days, months, years, hours, minutes, basis points, and hundredths that cross. */
const SIGNED_32_BIT_MIN = -(2 ** 31);

/** The largest signed 32-bit integer. */
const SIGNED_32_BIT_MAX = 2 ** 31 - 1;

/** The smallest signed 64-bit integer, the type of the minor units and ten-thousandths that cross. */
const SIGNED_64_BIT_MIN = -(2n ** 63n);

/** The largest signed 64-bit integer. */
const SIGNED_64_BIT_MAX = 2n ** 63n - 1n;

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
        return await call(undefined === signal ? undefined : { signal });
    } catch (error: unknown) {
        signal?.throwIfAborted();
        throw convertedError(error);
    }
}

/**
 * `value`, which crosses into the generated code as a signed 32-bit integer, when it is an integer of that range; for any
 * other number, the error `refusal` makes is thrown.
 *
 * The runtime converts the number as `value | 0` does, as an argument and in a record alike: it wraps one outside the range
 * into it, drops a fraction, and turns NaN and the infinities into 0, so without this check the core would check and keep
 * another value than the layer holds.
 *
 * @internal
 */
export function checked32Bits(value: number, refusal: () => EranosError): number {
    if (Number.isInteger(value) && SIGNED_32_BIT_MIN <= value && SIGNED_32_BIT_MAX >= value) {
        return value;
    }
    throw refusal();
}

/**
 * `value`, which crosses into the generated code as a signed 64-bit integer, when it is in that range; for any other
 * integer, the error `refusal` makes is thrown.
 *
 * The runtime converts the integer as `BigInt.asIntN(64, value)` does, as an argument and in a record alike: it wraps one
 * outside the range into it, so without this check the core would check and keep another value than the layer holds.
 *
 * @internal
 */
export function checked64Bits(value: bigint, refusal: () => EranosError): bigint {
    if (SIGNED_64_BIT_MIN <= value && SIGNED_64_BIT_MAX >= value) {
        return value;
    }
    throw refusal();
}
