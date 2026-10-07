// The conversion of the generated code's calls and futures. A future whose cancellation did not
// reach the library's call would let a change land after the app gave it up; one that stayed
// pending on a failed conversion would hang the caller; and a failure other than the library's,
// dressed as one, would hide a defect in the bindings.

package eranos;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;
import java.util.concurrent.ExecutionException;

/** The calls, the futures, and the failures that are not the library's. */
class ConvertingTest {
    /**
     * A future that cancelled only itself, as a dependent stage does, would leave the library's
     * call running.
     */
    @Test
    void cancellingAFutureCancelsTheGeneratedOne() {
        CompletableFuture<String> generated = new CompletableFuture<>();
        CompletableFuture<Integer> future = Converting.future(generated, String::length);
        assertTrue(future.cancel(true));
        assertTrue(generated.isCancelled());
        assertEquals(
                4,
                Converting.future(CompletableFuture.completedFuture("done"), String::length)
                        .join());
    }

    /**
     * A conversion that threw inside the future's callback would leave the future pending forever,
     * and the caller waiting on it.
     */
    @Test
    void aConversionThatFailsFailsTheFuture() {
        IllegalStateException failure = new IllegalStateException("conversion");
        CompletableFuture<Integer> future =
                Converting.future(
                        CompletableFuture.completedFuture("done"),
                        value -> {
                            throw failure;
                        });
        ExecutionException execution = assertThrows(ExecutionException.class, future::get);
        assertSame(failure, execution.getCause());
    }

    /**
     * The library's failure delivered through a dependent stage would reach the caller wrapped
     * twice, and anything that is not the library's failure would be taken for one.
     */
    @Test
    void aFailureIsUnwrappedAndConvertedOnlyWhenItIsTheLibrarys() {
        eranos.internal.EranosException released =
                new eranos.internal.EranosException.Released("gone");
        Throwable unwrapped = Converting.convertedCause(new CompletionException(released));
        assertEquals(IllegalStateException.class, unwrapped.getClass());
        assertEquals("gone", unwrapped.getMessage());
        RuntimeException defect = new RuntimeException("defect");
        assertSame(defect, Converting.convertedCause(defect));
        assertSame(defect, Converting.convertedCause(new CompletionException(defect)));
        CompletionException empty = new CompletionException(null);
        assertSame(empty, Converting.convertedCause(empty));
    }

    /**
     * A call the library refuses only on a platform without threads that still reported a refusal
     * would otherwise force every caller to catch a checked exception that never comes.
     */
    @Test
    void aRefusalOfAnUnfailingCallIsUnchecked() {
        IllegalStateException call =
                assertThrows(
                        IllegalStateException.class,
                        () ->
                                Converting.callUnfailing(
                                        () -> {
                                            throw new eranos.internal.EranosException.ExpenseGroup(
                                                    new eranos.internal.ExpenseGroupFailure.Busy(),
                                                    "busy");
                                        }));
        assertEquals("busy", call.getMessage());
        assertEquals(EranosException.ExpenseGroup.class, call.getCause().getClass());
        IllegalStateException run =
                assertThrows(
                        IllegalStateException.class,
                        () ->
                                Converting.runUnfailing(
                                        () -> {
                                            throw new eranos.internal.EranosException.ExpenseGroup(
                                                    new eranos.internal.ExpenseGroupFailure.Busy(),
                                                    "busy");
                                        }));
        assertEquals("busy", run.getMessage());
        EranosException.ExpenseGroup refused =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () ->
                                Converting.run(
                                        () -> {
                                            throw new eranos.internal.EranosException.ExpenseGroup(
                                                    new eranos.internal.ExpenseGroupFailure.Busy(),
                                                    "busy");
                                        }));
        assertEquals(new ExpenseGroupFailure.Busy(), refused.failure());
    }
}
