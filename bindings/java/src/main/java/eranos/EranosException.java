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
 * } catch (EranosException.ExpenseGroup exception) {
 *     if (!(exception.failure() instanceof ExpenseGroupFailure.MemberInUse)) {
 *         throw exception;
 *     }
 *     group.archiveMember(member); // The member still has entries; archive them instead.
 * }
 * }</pre>
 *
 * <p>A call the program should not make throws the standard exception instead: {@link
 * IllegalStateException} for a call through an {@link ExpenseGroupReader} or an {@link
 * ExpenseGroupWriter} after it is closed. A future whose call is cancelled fails with a {@link
 * CancellationException}.
 */
// The exceptions are not serialisable in full: their failures hold the package's values, which
// make no promise about a serialised form.
@SuppressWarnings("serial")
public abstract sealed class EranosException extends Exception
        permits EranosException.Id,
                EranosException.PaymentAccount,
                EranosException.Calendar,
                EranosException.Category,
                EranosException.Contact,
                EranosException.Money,
                EranosException.Member,
                EranosException.Expense,
                EranosException.Settlement,
                EranosException.Transfer,
                EranosException.ExpenseGroup,
                EranosException.Language,
                EranosException.MismatchedRecord,
                EranosException.InvalidJson {
    private EranosException(String message) {
        super(message);
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

    /** The library's {@code PaymentAccountError}, with the message it words. */
    public static final class PaymentAccount extends EranosException {
        /** What the module reports. */
        private final PaymentAccountFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public PaymentAccount(PaymentAccountFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public PaymentAccountFailure failure() {
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

    /** The library's {@code ExpenseGroupError}, with the message it words. */
    public static final class ExpenseGroup extends EranosException {
        /** What the module reports. */
        private final ExpenseGroupFailure failure;

        /**
         * The exception of {@code failure}, which the library words as {@code message}.
         *
         * @param failure what the module reports
         * @param message the library's message
         */
        public ExpenseGroup(ExpenseGroupFailure failure, String message) {
            super(message);
            this.failure = Objects.requireNonNull(failure, "failure");
        }

        /**
         * What the module reports.
         *
         * @return the failure
         */
        public ExpenseGroupFailure failure() {
            return failure;
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

    /**
     * A record whose derived text is not what the library derives from its other fields, so it
     * names no value the library writes: the field it gave another way, and what went wrong.
     */
    public static final class MismatchedRecord extends EranosException {
        /** The field whose text is not the library's. */
        private final MismatchedField field;

        /**
         * The exception of a record that gave {@code field} another way, which the library words as
         * {@code message}.
         *
         * @param field the field whose text is not the library's
         * @param message the library's message
         */
        public MismatchedRecord(MismatchedField field, String message) {
            super(message);
            this.field = Objects.requireNonNull(field);
        }

        /**
         * The field whose text is not the library's.
         *
         * @return the field
         */
        public MismatchedField field() {
            return field;
        }
    }

    /**
     * A saved group that is not JSON of the saved form, at a line and a column; the message names
     * the fault but never the text there, which can hold an account number.
     */
    public static final class InvalidJson extends EranosException {
        /** The line of the fault, from 1. */
        private final long line;

        /** The column of the fault, from 1. */
        private final long column;

        /**
         * The exception of the fault at {@code line} and {@code column}, which the library words as
         * {@code message}.
         *
         * @param line the line of the fault, from 1
         * @param column the column of the fault, from 1
         * @param message the library's message
         */
        public InvalidJson(long line, long column, String message) {
            super(message);
            this.line = line;
            this.column = column;
        }

        /**
         * The line of the fault.
         *
         * @return the line, from 1
         */
        public long line() {
            return line;
        }

        /**
         * The column of the fault.
         *
         * @return the column, from 1
         */
        public long column() {
            return column;
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
                eranos.internal.EranosException.Id.class,
                exception -> new Id(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.PaymentAccount.class,
                exception ->
                        new PaymentAccount(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Calendar.class,
                exception -> new Calendar(converted(exception.failure()), exception.detail()));
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
                eranos.internal.EranosException.Money.class,
                exception -> new Money(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Member.class,
                exception -> new Member(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Expense.class,
                exception -> new Expense(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Settlement.class,
                exception -> new Settlement(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Transfer.class,
                exception -> new Transfer(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.ExpenseGroup.class,
                exception -> new ExpenseGroup(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Language.class,
                exception -> new Language(converted(exception.failure()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.MismatchedRecord.class,
                exception ->
                        new MismatchedRecord(
                                MismatchedField.converted(exception.field()), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.InvalidJson.class,
                exception ->
                        new InvalidJson(exception.line(), exception.column(), exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Cancelled.class,
                exception -> new CancellationException(exception.detail()));
        register(
                conversions,
                eranos.internal.EranosException.Released.class,
                exception -> new IllegalStateException(exception.detail()));
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
    static IdFailure converted(eranos.internal.IdFailure failure) {
        return switch (failure) {
            case EMPTY -> new IdFailure.Empty();
            case TOO_LONG -> new IdFailure.TooLong();
            case CONTROL_CHARACTER -> new IdFailure.ControlCharacter();
            case SURROUNDING_WHITESPACE -> new IdFailure.SurroundingWhitespace();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static PaymentAccountFailure converted(eranos.internal.PaymentAccountFailure failure) {
        return switch (failure) {
            case eranos.internal.PaymentAccountFailure.EmptyName _ ->
                    new PaymentAccountFailure.EmptyName();
            case eranos.internal.PaymentAccountFailure.InvalidIban invalidIban ->
                    new PaymentAccountFailure.InvalidIban(converted(invalidIban.reason()));
            case eranos.internal.PaymentAccountFailure.EmptyUsername _ ->
                    new PaymentAccountFailure.EmptyUsername();
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
    static MoneyFailure converted(eranos.internal.MoneyFailure failure) {
        return switch (failure) {
            case eranos.internal.MoneyFailure.InvalidAmount _ -> new MoneyFailure.InvalidAmount();
            case eranos.internal.MoneyFailure.TooManyDecimals tooManyDecimals ->
                    new MoneyFailure.TooManyDecimals(
                            CurrencyCode.converted(tooManyDecimals.currency()),
                            tooManyDecimals.allowed());
            case eranos.internal.MoneyFailure.CurrencyMismatch currencyMismatch ->
                    new MoneyFailure.CurrencyMismatch(
                            CurrencyCode.converted(currencyMismatch.left()),
                            CurrencyCode.converted(currencyMismatch.right()));
            case eranos.internal.MoneyFailure.UnknownCurrency _ ->
                    new MoneyFailure.UnknownCurrency();
            case eranos.internal.MoneyFailure.InvalidExchangeRate _ ->
                    new MoneyFailure.InvalidExchangeRate();
            case eranos.internal.MoneyFailure.InvalidParts _ -> new MoneyFailure.InvalidParts();
            case eranos.internal.MoneyFailure.InvalidPercentage _ ->
                    new MoneyFailure.InvalidPercentage();
            case eranos.internal.MoneyFailure.InvalidQuantity _ ->
                    new MoneyFailure.InvalidQuantity();
            case eranos.internal.MoneyFailure.NotFinite _ -> new MoneyFailure.NotFinite();
            case eranos.internal.MoneyFailure.Overflow _ -> new MoneyFailure.Overflow();
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
    static ExpenseFailure converted(eranos.internal.ExpenseFailure failure) {
        return switch (failure) {
            case eranos.internal.ExpenseFailure.EmptyName _ -> new ExpenseFailure.EmptyName();
            case eranos.internal.ExpenseFailure.NonPositiveAmount _ ->
                    new ExpenseFailure.NonPositiveAmount();
            case eranos.internal.ExpenseFailure.CurrencyMismatch currencyMismatch ->
                    new ExpenseFailure.CurrencyMismatch(
                            CurrencyCode.converted(currencyMismatch.expected()),
                            CurrencyCode.converted(currencyMismatch.found()));
            case eranos.internal.ExpenseFailure.EmptySplit _ -> new ExpenseFailure.EmptySplit();
            case eranos.internal.ExpenseFailure.DuplicateMember duplicateMember ->
                    new ExpenseFailure.DuplicateMember(new MemberId(duplicateMember.member()));
            case eranos.internal.ExpenseFailure.NonPositiveShare nonPositiveShare ->
                    new ExpenseFailure.NonPositiveShare(new MemberId(nonPositiveShare.member()));
            case eranos.internal.ExpenseFailure.AmountsDoNotAddUp amountsDoNotAddUp ->
                    new ExpenseFailure.AmountsDoNotAddUp(
                            eranos.Money.converted(amountsDoNotAddUp.expected()),
                            eranos.Money.converted(amountsDoNotAddUp.actual()));
            case eranos.internal.ExpenseFailure.PercentagesDoNotAddUp percentagesDoNotAddUp ->
                    new ExpenseFailure.PercentagesDoNotAddUp(
                            percentagesDoNotAddUp.totalBasisPoints());
            case eranos.internal.ExpenseFailure.NoItems _ -> new ExpenseFailure.NoItems();
            case eranos.internal.ExpenseFailure.EmptyItemName _ ->
                    new ExpenseFailure.EmptyItemName();
            case eranos.internal.ExpenseFailure.ItemsDoNotAddUp itemsDoNotAddUp ->
                    new ExpenseFailure.ItemsDoNotAddUp(
                            eranos.Money.converted(itemsDoNotAddUp.expected()),
                            eranos.Money.converted(itemsDoNotAddUp.actual()));
            case eranos.internal.ExpenseFailure.NegativeDiscount _ ->
                    new ExpenseFailure.NegativeDiscount();
            case eranos.internal.ExpenseFailure.DiscountAboveHundredPercent _ ->
                    new ExpenseFailure.DiscountAboveHundredPercent();
            case eranos.internal.ExpenseFailure.DiscountAbovePrice _ ->
                    new ExpenseFailure.DiscountAbovePrice();
            case eranos.internal.ExpenseFailure.NegativeTip _ -> new ExpenseFailure.NegativeTip();
            case eranos.internal.ExpenseFailure.Overflow _ -> new ExpenseFailure.Overflow();
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
            case SAME_MEMBER -> new TransferFailure.SameMember();
            case NON_POSITIVE_AMOUNT -> new TransferFailure.NonPositiveAmount();
        };
    }

    /**
     * The failure the generated code reports as {@code failure}.
     *
     * @param failure the generated failure
     * @return the failure
     */
    static ExpenseGroupFailure converted(eranos.internal.ExpenseGroupFailure failure) {
        return switch (failure) {
            case eranos.internal.ExpenseGroupFailure.UnsupportedFormat unsupportedFormat ->
                    new ExpenseGroupFailure.UnsupportedFormat(unsupportedFormat.format());
            case eranos.internal.ExpenseGroupFailure.EmptyName _ ->
                    new ExpenseGroupFailure.EmptyName();
            case eranos.internal.ExpenseGroupFailure.MissingExchangeRate missingExchangeRate ->
                    new ExpenseGroupFailure.MissingExchangeRate(
                            CurrencyCode.converted(missingExchangeRate.currency()));
            case eranos.internal.ExpenseGroupFailure.UnexpectedExchangeRate _ ->
                    new ExpenseGroupFailure.UnexpectedExchangeRate();
            case eranos.internal.ExpenseGroupFailure.DuplicateExchangeRate duplicateExchangeRate ->
                    new ExpenseGroupFailure.DuplicateExchangeRate(
                            CurrencyCode.converted(duplicateExchangeRate.currency()));
            case eranos.internal.ExpenseGroupFailure.DuplicateMember duplicateMember ->
                    new ExpenseGroupFailure.DuplicateMember(new MemberId(duplicateMember.member()));
            case eranos.internal.ExpenseGroupFailure.UnknownMember unknownMember ->
                    new ExpenseGroupFailure.UnknownMember(new MemberId(unknownMember.member()));
            case eranos.internal.ExpenseGroupFailure.ArchivedMember archivedMember ->
                    new ExpenseGroupFailure.ArchivedMember(new MemberId(archivedMember.member()));
            case eranos.internal.ExpenseGroupFailure.MemberInUse memberInUse ->
                    new ExpenseGroupFailure.MemberInUse(
                            new MemberId(memberInUse.member()),
                            memberInUse.expenses().stream().map(ExpenseId::new).toList(),
                            memberInUse.transfers().stream().map(TransferId::new).toList());
            case eranos.internal.ExpenseGroupFailure.DuplicateCategory duplicateCategory ->
                    new ExpenseGroupFailure.DuplicateCategory(
                            new CategoryId(duplicateCategory.category()));
            case eranos.internal.ExpenseGroupFailure.DuplicateCategoryName duplicateCategoryName ->
                    new ExpenseGroupFailure.DuplicateCategoryName(
                            new CategoryId(duplicateCategoryName.existing()));
            case eranos.internal.ExpenseGroupFailure.UnknownCategory unknownCategory ->
                    new ExpenseGroupFailure.UnknownCategory(
                            new CategoryId(unknownCategory.category()));
            case eranos.internal.ExpenseGroupFailure.BuiltInCategory _ ->
                    new ExpenseGroupFailure.BuiltInCategory();
            case eranos.internal.ExpenseGroupFailure.DuplicateExpense duplicateExpense ->
                    new ExpenseGroupFailure.DuplicateExpense(
                            new ExpenseId(duplicateExpense.expense()));
            case eranos.internal.ExpenseGroupFailure.UnknownExpense unknownExpense ->
                    new ExpenseGroupFailure.UnknownExpense(new ExpenseId(unknownExpense.expense()));
            case eranos.internal.ExpenseGroupFailure.DuplicateTransfer duplicateTransfer ->
                    new ExpenseGroupFailure.DuplicateTransfer(
                            new TransferId(duplicateTransfer.transfer()));
            case eranos.internal.ExpenseGroupFailure.UnknownTransfer unknownTransfer ->
                    new ExpenseGroupFailure.UnknownTransfer(
                            new TransferId(unknownTransfer.transfer()));
            case eranos.internal.ExpenseGroupFailure.Busy _ -> new ExpenseGroupFailure.Busy();
            case eranos.internal.ExpenseGroupFailure.Overflow _ ->
                    new ExpenseGroupFailure.Overflow();
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
    static IbanReason converted(eranos.internal.IbanReason failure) {
        return switch (failure) {
            case eranos.internal.IbanReason.InvalidCharacter _ -> new IbanReason.InvalidCharacter();
            case eranos.internal.IbanReason.InvalidFormat _ -> new IbanReason.InvalidFormat();
            case eranos.internal.IbanReason.UnknownCountry _ -> new IbanReason.UnknownCountry();
            case eranos.internal.IbanReason.WrongLength wrongLength ->
                    new IbanReason.WrongLength(wrongLength.expected(), wrongLength.found());
            case eranos.internal.IbanReason.InvalidStructure _ -> new IbanReason.InvalidStructure();
            case eranos.internal.IbanReason.InvalidChecksum _ -> new IbanReason.InvalidChecksum();
        };
    }
}
