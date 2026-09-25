package eranos;

import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.function.Function;

/**
 * The calls into the generated code, and the conversion of what it reports into this package's
 * exceptions.
 */
final class Converting {
    private Converting() {}

    /**
     * A call of the generated code that returns a value, and reports the library's failure as the
     * generated exception.
     *
     * @param <T> what the call returns
     */
    @FunctionalInterface
    interface Call<T> {
        /**
         * Makes the call.
         *
         * @return what the call returns
         * @throws eranos.internal.EranosException the library's failure
         */
        T call() throws eranos.internal.EranosException;
    }

    /**
     * A call of the generated code that returns nothing, and reports the library's failure as the
     * generated exception.
     */
    @FunctionalInterface
    interface Action {
        /**
         * Makes the call.
         *
         * @throws eranos.internal.EranosException the library's failure
         */
        void run() throws eranos.internal.EranosException;
    }

    /**
     * Makes {@code call} and throws what it reports as this package's exception.
     *
     * @param <T> what the call returns
     * @param call the call
     * @return what the call returns
     * @throws EranosException the library's failure
     */
    static <T> T call(Call<T> call) throws EranosException {
        try {
            return call.call();
        } catch (eranos.internal.EranosException exception) {
            throw thrown(exception);
        }
    }

    /**
     * Makes {@code action} and throws what it reports as this package's exception.
     *
     * @param action the call
     * @throws EranosException the library's failure
     */
    static void run(Action action) throws EranosException {
        try {
            action.run();
        } catch (eranos.internal.EranosException exception) {
            throw thrown(exception);
        }
    }

    /**
     * Makes {@code call}, which the library refuses only on a platform without threads, and throws
     * what it reports as an unchecked exception.
     *
     * @param <T> what the call returns
     * @param call the call
     * @return what the call returns
     */
    static <T> T callUnfailing(Call<T> call) {
        try {
            return call(call);
        } catch (EranosException exception) {
            throw new IllegalStateException(exception.getMessage(), exception);
        }
    }

    /**
     * Makes {@code action}, which the library refuses only on a platform without threads, and
     * throws what it reports as an unchecked exception.
     *
     * @param action the call
     */
    static void runUnfailing(Action action) {
        callUnfailing(
                () -> {
                    action.run();
                    return null;
                });
    }

    /**
     * The future of {@code generated}, converted by {@code conversion}, whose cancellation cancels
     * {@code generated}, which drops the library's call.
     *
     * <p>A dependent future of {@code generated}, such as {@code thenApply} makes, would cancel
     * only itself and leave the library's call running, so the future forwards its cancellation.
     *
     * @param <G> what the generated call returns
     * @param <T> what the future returns
     * @param generated the generated call's future
     * @param conversion the conversion of its result
     * @return the future
     */
    static <G, T> CompletableFuture<T> future(
            CompletableFuture<G> generated, Function<G, T> conversion) {
        CompletableFuture<T> future =
                new CompletableFuture<>() {
                    @Override
                    public boolean cancel(boolean mayInterruptIfRunning) {
                        boolean cancelled = super.cancel(mayInterruptIfRunning);
                        if (cancelled) {
                            generated.cancel(mayInterruptIfRunning);
                        }
                        return cancelled;
                    }
                };
        // The stage whenComplete returns carries only what the action throws, and the action
        // throws nothing: every outcome, a failed conversion included, completes the future.
        var _ =
                generated.whenComplete(
                        (value, failure) -> {
                            if (failure != null) {
                                future.completeExceptionally(convertedCause(failure));
                                return;
                            }
                            try {
                                future.complete(conversion.apply(value));
                            } catch (RuntimeException exception) {
                                future.completeExceptionally(exception);
                            }
                        });
        return future;
    }

    /**
     * The exception a failed future reports for {@code failure}: the library's failure as this
     * package's exception, and anything else as it is.
     *
     * @param failure what the generated future failed with, perhaps wrapped by a dependent stage
     * @return the exception
     */
    static Throwable convertedCause(Throwable failure) {
        Throwable cause =
                failure instanceof CompletionException && failure.getCause() != null
                        ? failure.getCause()
                        : failure;
        return cause instanceof eranos.internal.EranosException exception
                ? converted(exception)
                : cause;
    }

    /**
     * The exception this package throws for {@code exception}: the library's failure as an {@link
     * EranosException}, a call the program should not make as the standard exception for it, and a
     * cancelled call as a {@link CancellationException}.
     *
     * @param exception the generated code's exception
     * @return the exception to throw
     */
    static Exception converted(eranos.internal.EranosException exception) {
        return EranosException.converted(exception);
    }

    /**
     * The exception to throw for {@code exception}: the unchecked ones thrown here, and the
     * library's failure returned for the caller to throw, so that the compiler sees it thrown.
     *
     * @param exception the generated code's exception
     * @return the library's failure
     */
    private static EranosException thrown(eranos.internal.EranosException exception) {
        Exception converted = converted(exception);
        if (converted instanceof RuntimeException unchecked) {
            throw unchecked;
        }
        return (EranosException) converted;
    }
}
