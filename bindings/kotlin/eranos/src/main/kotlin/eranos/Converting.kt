package eranos

/**
 * Runs [call], which calls the generated code, and throws what it reports as this package's exception.
 *
 * The generated code throws its own exception for every failure the library reports. Every call of the package goes through
 * this function, so the one conversion lives here; being inline, it runs a suspending [call] in its caller's coroutine.
 */
internal inline fun <T> converting(call: () -> T): T =
    try {
        call()
    } catch (exception: eranos.internal.EranosException) {
        throw exception.converted()
    }

/** A hash of [values], in order, for the classes that write their equality out. */
internal fun hashOf(vararg values: Any?): Int = values.contentHashCode()
