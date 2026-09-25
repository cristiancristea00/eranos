package eranos;

import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.concurrent.CancellationException;
import java.util.function.Function;

/**
 * Why a call failed: a subclass per module of the library, holding what the module reports, with
 * the library's message as {@link #getMessage}.
 *
 * <p>Every call of the package that the library refuses throws a subclass, so a {@code catch} takes
 * a module's failures together, and a {@code switch} over its failure takes one of them:
 *
 * <pre>{@code
 * try {
 *     group.removeMember(member);
 * } catch (EranosException.Group exception) {
 *     if (!(exception.failure() instanceof GroupFailure.MemberInUse)) {
 *         throw exception;
 *     }
 *     group.archiveMember(member); // The member still has entries; archive them instead.
 * }
 * }</pre>
 *
 * <p>A call the program should not make throws the standard exception instead: {@link
 * IllegalArgumentException} for an entry to add that has an identifier or one to replace that has
 * none, and {@link IllegalStateException} for a read through a {@link GroupReader} after it is
 * closed. A future whose call is cancelled fails with a {@link CancellationException}.
 */
// The exceptions are not serialisable in full: their failures hold the package's values, which
// make no promise about a serialised form.
@SuppressWarnings("serial")
public abstract sealed class EranosException extends Exception
        permits EranosException.Account,
                EranosException.Calendar,
                EranosException.Category,
                EranosException.Contact,
                EranosException.Expense,
                EranosException.Group,
                EranosException.Id,
                EranosException.InvalidJson,
                EranosException.Language,
                EranosException.Member,
                EranosException.Money,
                EranosException.Settlement,
                EranosException.Transfer {
    private EranosException(String message) {
        super(message);
    }

    /** The library's {@code AccountError}, with the message it words. */
    public static final class Account extends EranosException {
        /** What the module reports. */
        private final AccountFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Account(AccountFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public AccountFailure failure() {
            return failure;
        }
    }

    /** The library's {@code CalendarError}, with the message it words. */
    public static final class Calendar extends EranosException {
        /** What the module reports. */
        private final CalendarFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Calendar(CalendarFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public CalendarFailure failure() {
            return failure;
        }
    }

    /** The library's {@code CategoryError}, with the message it words. */
    public static final class Category extends EranosException {
        /** What the module reports. */
        private final CategoryFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Category(CategoryFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public CategoryFailure failure() {
            return failure;
        }
    }

    /** The library's {@code ContactError}, with the message it words. */
    public static final class Contact extends EranosException {
        /** What the module reports. */
        private final ContactFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Contact(ContactFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public ContactFailure failure() {
            return failure;
        }
    }

    /** The library's {@code ExpenseError}, with the message it words. */
    public static final class Expense extends EranosException {
        /** What the module reports. */
        private final ExpenseFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Expense(ExpenseFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public ExpenseFailure failure() {
            return failure;
        }
    }

    /** The library's {@code GroupError}, with the message it words. */
    public static final class Group extends EranosException {
        /** What the module reports. */
        private final GroupFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Group(GroupFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public GroupFailure failure() {
            return failure;
        }
    }

    /** The library's {@code IdError}, with the message it words. */
    public static final class Id extends EranosException {
        /** What the module reports. */
        private final IdFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Id(IdFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public IdFailure failure() {
            return failure;
        }
    }

    /**
     * A saved group that is not JSON of the saved form, at a line and a column; the message names
     * the fault but never the text there, which can hold an account number.
     */
    public static final class InvalidJson extends EranosException {
        /** The column of the fault, from 1. */
        private final long column;

        /** The line of the fault, from 1. */
        private final long line;

        /**
         * The exception of the fault at {@code line} and {@code column}, which the library words as
         * {@code message}.
         *
         * @param column the column of the fault, from 1
         * @param line the line of the fault, from 1
         * @param message the library's message
         */
        public InvalidJson(long column, long line, String message) {
            super(message);
            this.column = column;
            this.line = line;
        }

        /**
         * The column of the fault.
         *
         * @return the column, from 1
         */
        public long column() {
            return column;
        }

        /**
         * The line of the fault.
         *
         * @return the line, from 1
         */
        public long line() {
            return line;
        }
    }

    /** The library's {@code LanguageError}, with the message it words. */
    public static final class Language extends EranosException {
        /** What the module reports. */
        private final LanguageFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Language(LanguageFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public LanguageFailure failure() {
            return failure;
        }
    }

    /** The library's {@code MemberError}, with the message it words. */
    public static final class Member extends EranosException {
        /** What the module reports. */
        private final MemberFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Member(MemberFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public MemberFailure failure() {
            return failure;
        }
    }

    /** The library's {@code MoneyError}, with the message it words. */
    public static final class Money extends EranosException {
        /** What the module reports. */
        private final MoneyFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Money(MoneyFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public MoneyFailure failure() {
            return failure;
        }
    }

    /** The library's {@code SettlementError}, with the message it words. */
    public static final class Settlement extends EranosException {
        /** What the module reports. */
        private final SettlementFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Settlement(SettlementFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public SettlementFailure failure() {
            return failure;
        }
    }

    /** The library's {@code TransferError}, with the message it words. */
    public static final class Transfer extends EranosException {
        /** What the module reports. */
        private final TransferFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public Transfer(TransferFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public TransferFailure failure() {
            return failure;
        }
    }

    /**
     * The conversion of every exception of the generated code, by its class. The generated
     * exception is not sealed, so no switch over it is exhaustive; the error test checks that this
     * table holds every case the generator writes.
     */
    private static final Map<Class<?>, Function<eranos.internal.EranosException, Exception>>
            CONVERSIONS = conversions();

    /**
     * The table of {@link #CONVERSIONS}.
     *
     * @return the table
     */
    private static Map<Class<?>, Function<eranos.internal.EranosException, Exception>>
            conversions() {
        Map<Class<?>, Function<eranos.internal.EranosException, Exception>> conversions =
                new HashMap<>();
        register(
                conversions,
                eranos.internal.EranosException.Account.class,
                exception -> new Account(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Calendar.class,
                exception -> new Calendar(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Cancelled.class,
                exception -> new CancellationException(exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Category.class,
                exception -> new Category(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Contact.class,
                exception -> new Contact(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Expense.class,
                exception -> new Expense(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Group.class,
                exception -> new Group(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Id.class,
                exception -> new Id(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.IdentifierGiven.class,
                exception -> new IllegalArgumentException(exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.IdentifierMissing.class,
                exception -> new IllegalArgumentException(exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.InvalidJson.class,
                exception ->
                        new InvalidJson(exception.column(), exception.line(), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Language.class,
                exception -> new Language(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Member.class,
                exception -> new Member(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Money.class,
                exception -> new Money(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Released.class,
                exception -> new IllegalStateException(exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Settlement.class,
                exception -> new Settlement(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Transfer.class,
                exception -> new Transfer(converted(exception.failure()), exception.detail()));
        return Map.copyOf(conversions);
    }

    /**
     * Adds the conversion of the generated exceptions of {@code type} to {@code conversions}.
     *
     * @param <G> the generated exception
     * @param conversions the table
     * @param type the generated exception's class
     * @param conversion its conversion
     */
    private static <G extends eranos.internal.EranosException> void register(
            Map<Class<?>, Function<eranos.internal.EranosException, Exception>> conversions,
            Class<G> type,
            Function<G, Exception> conversion) {
        conversions.put(type, exception -> conversion.apply(type.cast(exception)));
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
        return CONVERSIONS.get(exception.getClass()).apply(exception);
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static AccountFailure converted(eranos.internal.AccountFailure failure) {
        return switch (failure) {
            case eranos.internal.AccountFailure.EmptyName _ -> new AccountFailure.EmptyName();
            case eranos.internal.AccountFailure.EmptyUsername _ ->
                    new AccountFailure.EmptyUsername();
            case eranos.internal.AccountFailure.InvalidIban invalidIban ->
                    new AccountFailure.InvalidIban(converted(invalidIban.reason()));
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static CalendarFailure converted(eranos.internal.CalendarFailure failure) {
        return switch (failure) {
            case INVALID_DATE -> new CalendarFailure.InvalidDate();
            case INVALID_TIME -> new CalendarFailure.InvalidTime();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static CategoryFailure converted(eranos.internal.CategoryFailure failure) {
        return switch (failure) {
            case EMPTY_NAME -> new CategoryFailure.EmptyName();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static ContactFailure converted(eranos.internal.ContactFailure failure) {
        return switch (failure) {
            case INVALID_EMAIL -> new ContactFailure.InvalidEmail();
            case INVALID_PHONE_NUMBER -> new ContactFailure.InvalidPhoneNumber();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static ExpenseFailure converted(eranos.internal.ExpenseFailure failure) {
        return switch (failure) {
            case eranos.internal.ExpenseFailure.AmountsDoNotAddUp amountsDoNotAddUp ->
                    new ExpenseFailure.AmountsDoNotAddUp(
                            eranos.Money.converted(amountsDoNotAddUp.actual()),
                            eranos.Money.converted(amountsDoNotAddUp.expected()));
            case eranos.internal.ExpenseFailure.CurrencyMismatch currencyMismatch ->
                    new ExpenseFailure.CurrencyMismatch(
                            Currency.known(currencyMismatch.expected()),
                            Currency.known(currencyMismatch.found()));
            case eranos.internal.ExpenseFailure.DuplicateMember duplicateMember ->
                    new ExpenseFailure.DuplicateMember(new MemberId(duplicateMember.member()));
            case eranos.internal.ExpenseFailure.EmptyItemName _ ->
                    new ExpenseFailure.EmptyItemName();
            case eranos.internal.ExpenseFailure.EmptyName _ -> new ExpenseFailure.EmptyName();
            case eranos.internal.ExpenseFailure.EmptySplit _ -> new ExpenseFailure.EmptySplit();
            case eranos.internal.ExpenseFailure.ItemsDoNotAddUp itemsDoNotAddUp ->
                    new ExpenseFailure.ItemsDoNotAddUp(
                            eranos.Money.converted(itemsDoNotAddUp.actual()),
                            eranos.Money.converted(itemsDoNotAddUp.expected()));
            case eranos.internal.ExpenseFailure.NegativeTip _ -> new ExpenseFailure.NegativeTip();
            case eranos.internal.ExpenseFailure.NoItems _ -> new ExpenseFailure.NoItems();
            case eranos.internal.ExpenseFailure.NonPositiveAmount _ ->
                    new ExpenseFailure.NonPositiveAmount();
            case eranos.internal.ExpenseFailure.NonPositiveShare nonPositiveShare ->
                    new ExpenseFailure.NonPositiveShare(new MemberId(nonPositiveShare.member()));
            case eranos.internal.ExpenseFailure.Overflow _ -> new ExpenseFailure.Overflow();
            case eranos.internal.ExpenseFailure.PercentagesDoNotAddUp percentagesDoNotAddUp ->
                    new ExpenseFailure.PercentagesDoNotAddUp(
                            percentagesDoNotAddUp.totalBasisPoints());
            case eranos.internal.ExpenseFailure.ZeroQuantity _ -> new ExpenseFailure.ZeroQuantity();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static GroupFailure converted(eranos.internal.GroupFailure failure) {
        return switch (failure) {
            case eranos.internal.GroupFailure.ArchivedMember archivedMember ->
                    new GroupFailure.ArchivedMember(new MemberId(archivedMember.member()));
            case eranos.internal.GroupFailure.BuiltInCategory _ ->
                    new GroupFailure.BuiltInCategory();
            case eranos.internal.GroupFailure.Busy _ -> new GroupFailure.Busy();
            case eranos.internal.GroupFailure.DuplicateCategory duplicateCategory ->
                    new GroupFailure.DuplicateCategory(
                            new CategoryId(duplicateCategory.category()));
            case eranos.internal.GroupFailure.DuplicateCategoryName duplicateCategoryName ->
                    new GroupFailure.DuplicateCategoryName(
                            new CategoryId(duplicateCategoryName.existing()));
            case eranos.internal.GroupFailure.DuplicateExchangeRate duplicateExchangeRate ->
                    new GroupFailure.DuplicateExchangeRate(
                            Currency.known(duplicateExchangeRate.currency()));
            case eranos.internal.GroupFailure.DuplicateExpense duplicateExpense ->
                    new GroupFailure.DuplicateExpense(new ExpenseId(duplicateExpense.expense()));
            case eranos.internal.GroupFailure.DuplicateMember duplicateMember ->
                    new GroupFailure.DuplicateMember(new MemberId(duplicateMember.member()));
            case eranos.internal.GroupFailure.DuplicateTransfer duplicateTransfer ->
                    new GroupFailure.DuplicateTransfer(
                            new TransferId(duplicateTransfer.transfer()));
            case eranos.internal.GroupFailure.EmptyName _ -> new GroupFailure.EmptyName();
            case eranos.internal.GroupFailure.MemberInUse memberInUse ->
                    new GroupFailure.MemberInUse(
                            memberInUse.expenses().stream().map(ExpenseId::new).toList(),
                            new MemberId(memberInUse.member()),
                            memberInUse.transfers().stream().map(TransferId::new).toList());
            case eranos.internal.GroupFailure.MissingExchangeRate missingExchangeRate ->
                    new GroupFailure.MissingExchangeRate(
                            Currency.known(missingExchangeRate.currency()));
            case eranos.internal.GroupFailure.Overflow _ -> new GroupFailure.Overflow();
            case eranos.internal.GroupFailure.UnexpectedExchangeRate _ ->
                    new GroupFailure.UnexpectedExchangeRate();
            case eranos.internal.GroupFailure.UnknownCategory unknownCategory ->
                    new GroupFailure.UnknownCategory(new CategoryId(unknownCategory.category()));
            case eranos.internal.GroupFailure.UnknownExpense unknownExpense ->
                    new GroupFailure.UnknownExpense(new ExpenseId(unknownExpense.expense()));
            case eranos.internal.GroupFailure.UnknownMember unknownMember ->
                    new GroupFailure.UnknownMember(new MemberId(unknownMember.member()));
            case eranos.internal.GroupFailure.UnknownTransfer unknownTransfer ->
                    new GroupFailure.UnknownTransfer(new TransferId(unknownTransfer.transfer()));
            case eranos.internal.GroupFailure.UnsupportedFormat unsupportedFormat ->
                    new GroupFailure.UnsupportedFormat(unsupportedFormat.format());
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static IdFailure converted(eranos.internal.IdFailure failure) {
        return switch (failure) {
            case INVALID_ID -> new IdFailure.InvalidId();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static LanguageFailure converted(eranos.internal.LanguageFailure failure) {
        return switch (failure) {
            case INVALID_LANGUAGE -> new LanguageFailure.InvalidLanguage();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static MemberFailure converted(eranos.internal.MemberFailure failure) {
        return switch (failure) {
            case EMPTY_FIRST_NAME -> new MemberFailure.EmptyFirstName();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static MoneyFailure converted(eranos.internal.MoneyFailure failure) {
        return switch (failure) {
            case eranos.internal.MoneyFailure.CurrencyMismatch currencyMismatch ->
                    new MoneyFailure.CurrencyMismatch(
                            Currency.known(currencyMismatch.left()),
                            Currency.known(currencyMismatch.right()));
            case eranos.internal.MoneyFailure.InvalidAmount _ -> new MoneyFailure.InvalidAmount();
            case eranos.internal.MoneyFailure.InvalidExchangeRate _ ->
                    new MoneyFailure.InvalidExchangeRate();
            case eranos.internal.MoneyFailure.InvalidParts _ -> new MoneyFailure.InvalidParts();
            case eranos.internal.MoneyFailure.InvalidPercentage _ ->
                    new MoneyFailure.InvalidPercentage();
            case eranos.internal.MoneyFailure.Overflow _ -> new MoneyFailure.Overflow();
            case eranos.internal.MoneyFailure.TooManyDecimals tooManyDecimals ->
                    new MoneyFailure.TooManyDecimals(
                            tooManyDecimals.allowed(), Currency.known(tooManyDecimals.currency()));
            case eranos.internal.MoneyFailure.UnknownCurrency _ ->
                    new MoneyFailure.UnknownCurrency();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static SettlementFailure converted(eranos.internal.SettlementFailure failure) {
        return switch (failure) {
            case eranos.internal.SettlementFailure.OutOfMemory outOfMemory ->
                    new SettlementFailure.OutOfMemory(outOfMemory.balances());
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static TransferFailure converted(eranos.internal.TransferFailure failure) {
        return switch (failure) {
            case NON_POSITIVE_AMOUNT -> new TransferFailure.NonPositiveAmount();
            case SAME_MEMBER -> new TransferFailure.SameMember();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static IbanReason converted(eranos.internal.IbanReason failure) {
        return switch (failure) {
            case eranos.internal.IbanReason.InvalidCharacter _ -> new IbanReason.InvalidCharacter();
            case eranos.internal.IbanReason.InvalidChecksum _ -> new IbanReason.InvalidChecksum();
            case eranos.internal.IbanReason.InvalidFormat _ -> new IbanReason.InvalidFormat();
            case eranos.internal.IbanReason.InvalidStructure _ -> new IbanReason.InvalidStructure();
            case eranos.internal.IbanReason.UnknownCountry _ -> new IbanReason.UnknownCountry();
            case eranos.internal.IbanReason.WrongLength wrongLength ->
                    new IbanReason.WrongLength(wrongLength.expected(), wrongLength.found());
        };
    }
}
