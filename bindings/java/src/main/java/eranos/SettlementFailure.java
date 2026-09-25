package eranos;

/** What the library's {@code SettlementError} reports: a type per failure, with what it holds. */
public sealed interface SettlementFailure {
    /**
     * The exact solver needs 2<sup>n</sup> bytes for {@code n} open balances, and that table does
     * not fit in the address space or the allocator refused it.
     *
     * @param balances how many open balances the plan has
     */
    record OutOfMemory(long balances) implements SettlementFailure {
        /**
         * The failure of these components.
         *
         * @param balances how many open balances the plan has
         */
        public OutOfMemory {}
    }
}
