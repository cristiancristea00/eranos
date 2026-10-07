package eranos;

import java.util.List;
import java.util.Objects;

/** What the library's {@code ExpenseGroupError} reports: a type per failure, with what it holds. */
public sealed interface ExpenseGroupFailure {
    /**
     * A saved group in a format this version does not read: a newer one, whose fields it would
     * drop, or an earlier one.
     *
     * @param format the format the saved group names
     */
    record UnsupportedFormat(long format) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param format the format the saved group names
         */
        public UnsupportedFormat {}
    }

    /** A group name that is blank. */
    record EmptyName() implements ExpenseGroupFailure {}

    /**
     * An entry in another currency than the group's has no exchange rate into the group currency:
     * its cost is local, or a currency change gave no rate for its currency.
     *
     * @param currency the currency without a rate
     */
    record MissingExchangeRate(Currency currency) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param currency the currency without a rate
         */
        public MissingExchangeRate {
            Objects.requireNonNull(currency, "currency");
        }
    }

    /**
     * An entry in the group currency has a converted cost; only a local cost is in the group
     * currency.
     */
    record UnexpectedExchangeRate() implements ExpenseGroupFailure {}

    /**
     * The same currency appears twice among the rates of a currency change.
     *
     * @param currency the currency that appears twice
     */
    record DuplicateExchangeRate(Currency currency) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param currency the currency that appears twice
         */
        public DuplicateExchangeRate {
            Objects.requireNonNull(currency, "currency");
        }
    }

    /**
     * A member with this ID already exists.
     *
     * @param member the identifier another member has
     */
    record DuplicateMember(MemberId member) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param member the identifier another member has
         */
        public DuplicateMember {
            Objects.requireNonNull(member, "member");
        }
    }

    /**
     * No member with this ID.
     *
     * @param member the identifier no member has
     */
    record UnknownMember(MemberId member) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param member the identifier no member has
         */
        public UnknownMember {
            Objects.requireNonNull(member, "member");
        }
    }

    /**
     * An archived member cannot join a new expense.
     *
     * @param member the archived member
     */
    record ArchivedMember(MemberId member) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param member the archived member
         */
        public ArchivedMember {
            Objects.requireNonNull(member, "member");
        }
    }

    /**
     * The member is still referenced and cannot be removed; archive it instead.
     *
     * @param member the member
     * @param expenses the expenses that name the member
     * @param transfers the transfers that name the member
     */
    record MemberInUse(MemberId member, List<ExpenseId> expenses, List<TransferId> transfers)
            implements ExpenseGroupFailure {
        /**
         * The failure of these components; the lists are copied.
         *
         * @param member the member
         * @param expenses the expenses that name the member
         * @param transfers the transfers that name the member
         */
        public MemberInUse {
            Objects.requireNonNull(member, "member");
            expenses = List.copyOf(expenses);
            transfers = List.copyOf(transfers);
        }
    }

    /**
     * A category with this ID already exists.
     *
     * @param category the identifier another category has
     */
    record DuplicateCategory(CategoryId category) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param category the identifier another category has
         */
        public DuplicateCategory {
            Objects.requireNonNull(category, "category");
        }
    }

    /**
     * Another category already has this name, compared by Unicode canonical caseless matching.
     *
     * @param existing the category that has the name
     */
    record DuplicateCategoryName(CategoryId existing) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param existing the category that has the name
         */
        public DuplicateCategoryName {
            Objects.requireNonNull(existing, "existing");
        }
    }

    /**
     * No category with this ID.
     *
     * @param category the identifier no category has
     */
    record UnknownCategory(CategoryId category) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param category the identifier no category has
         */
        public UnknownCategory {
            Objects.requireNonNull(category, "category");
        }
    }

    /** The built-in Uncategorised category cannot be renamed or removed. */
    record BuiltInCategory() implements ExpenseGroupFailure {}

    /**
     * An expense with this ID already exists.
     *
     * @param expense the identifier another expense has
     */
    record DuplicateExpense(ExpenseId expense) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param expense the identifier another expense has
         */
        public DuplicateExpense {
            Objects.requireNonNull(expense, "expense");
        }
    }

    /**
     * No expense with this ID.
     *
     * @param expense the identifier no expense has
     */
    record UnknownExpense(ExpenseId expense) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param expense the identifier no expense has
         */
        public UnknownExpense {
            Objects.requireNonNull(expense, "expense");
        }
    }

    /**
     * A transfer with this ID already exists.
     *
     * @param transfer the identifier another transfer has
     */
    record DuplicateTransfer(TransferId transfer) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param transfer the identifier another transfer has
         */
        public DuplicateTransfer {
            Objects.requireNonNull(transfer, "transfer");
        }
    }

    /**
     * No transfer with this ID.
     *
     * @param transfer the identifier no transfer has
     */
    record UnknownTransfer(TransferId transfer) implements ExpenseGroupFailure {
        /**
         * The failure of these components.
         *
         * @param transfer the identifier no transfer has
         */
        public UnknownTransfer {
            Objects.requireNonNull(transfer, "transfer");
        }
    }

    /** A shared group's lock was asked for without waiting, while another call held it. */
    record Busy() implements ExpenseGroupFailure {}

    /** A total would leave the range of an amount. */
    record Overflow() implements ExpenseGroupFailure {}
}
