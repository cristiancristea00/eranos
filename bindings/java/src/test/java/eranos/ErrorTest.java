// The exception every call throws. A failure converted to another module's subclass, to another
// failure, with another payload, or without the library's message would show an app another
// failure than the library's, so every failure the generated code can report is converted here,
// and every message is read back.

package eranos;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;

import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.List;
import java.util.concurrent.CancellationException;

/** The conversions of every failure the generated code can report. */
class ErrorTest {
    /** The failure an app must see, beside the failure of the generated code that becomes it. */
    private record Conversion(Object failure, eranos.internal.EranosException generated) {}

    /** Where a saved group that is not JSON stops being JSON. */
    private record Position(long line, long column) {}

    /**
     * A conversion that reached another module or failure, dropped what the failure holds, looked a
     * currency up by its code instead of keeping the details the library sent, or lost the message
     * would show an app another failure than the library's.
     */
    @Test
    void everyFailureOfTheGeneratedCodeConvertsToItsModuleAndFailure() {
        assertEquals(65, CONVERSIONS.size());
        for (Conversion conversion : CONVERSIONS) {
            EranosException converted =
                    assertInstanceOf(
                            EranosException.class, Converting.converted(conversion.generated()));
            assertEquals(conversion.failure(), failure(converted));
            assertEquals("detail", converted.getMessage());
        }
    }

    /**
     * A saved group that failed to load without its line and column would leave an app nothing to
     * point the user at; a call the program should not make, reported as a failure of the library,
     * would be caught as one instead of failing the program; and a cancelled call reported as
     * anything but a cancellation would read as a failure of the library.
     */
    @Test
    void theBindingsOwnFailuresConvertToTheirStandardExceptions() {
        EranosException.InvalidJson json =
                assertInstanceOf(
                        EranosException.InvalidJson.class,
                        Converting.converted(
                                new eranos.internal.EranosException.InvalidJson(1L, 2L, "detail")));
        assertEquals(new Position(1L, 2L), failure(json));
        assertEquals("detail", json.getMessage());
        assertStandard(
                new eranos.internal.EranosException.Cancelled("detail"),
                CancellationException.class);
        assertStandard(
                new eranos.internal.EranosException.Released("detail"),
                IllegalStateException.class);
    }

    /**
     * The generated exception is not sealed, so the compiler cannot say that the conversion knows
     * each of its cases; a case the generator added and the conversion did not would fail the
     * conversion instead of reporting the library's failure.
     */
    @Test
    void theConversionKnowsEveryCaseTheGeneratorWrites() {
        long cases =
                Arrays.stream(eranos.internal.EranosException.class.getDeclaredClasses())
                        .filter(eranos.internal.EranosException.class::isAssignableFrom)
                        .count();
        assertEquals(15, cases);
    }

    /** Checks that {@code generated} converts to a {@code standard} exception with its message. */
    private static void assertStandard(
            eranos.internal.EranosException generated, Class<? extends Exception> standard) {
        Exception converted = Converting.converted(generated);
        assertEquals(standard, converted.getClass());
        assertEquals("detail", converted.getMessage());
    }

    /** The failure {@code exception} holds, or its position for a saved group that is not JSON. */
    private static Object failure(EranosException exception) {
        return switch (exception) {
            case EranosException.Id id -> id.failure();
            case EranosException.PaymentAccount account -> account.failure();
            case EranosException.Calendar calendar -> calendar.failure();
            case EranosException.Category category -> category.failure();
            case EranosException.Contact contact -> contact.failure();
            case EranosException.Money money -> money.failure();
            case EranosException.Member member -> member.failure();
            case EranosException.Expense expense -> expense.failure();
            case EranosException.Settlement settlement -> settlement.failure();
            case EranosException.Transfer transfer -> transfer.failure();
            case EranosException.ExpenseGroup group -> group.failure();
            case EranosException.Language language -> language.failure();
            case EranosException.InvalidJson json -> new Position(json.line(), json.column());
        };
    }

    /**
     * The details the generated code sends for the currency {@code code}, with a name, a numeric
     * code, and decimal places that the library's currency of the code does not have, so that a
     * conversion that looked the code up would give another currency than {@link #currency}.
     */
    private static eranos.internal.CurrencyDetails details(String code) {
        return new eranos.internal.CurrencyDetails(code, "Sent " + code, 999, (byte) 3);
    }

    /** The currency an app must see for {@link #details} of {@code code}. */
    private static CurrencyCode currency(String code) {
        return CurrencyCode.converted(details(code));
    }

    /** {@code minorUnits} minor units of the euro as {@link #details} describes it. */
    private static Money euros(long minorUnits) {
        return new Money(minorUnits, currency("EUR"));
    }

    /** {@link #euros} as the generated code sends it. */
    private static eranos.internal.Money generatedEuros(long minorUnits) {
        return new eranos.internal.Money(minorUnits, details("EUR"));
    }

    /** Every failure the generated code can report, with every reason of a refused IBAN. */
    private static final List<Conversion> CONVERSIONS =
            List.of(
                    new Conversion(
                            new IdFailure.Empty(),
                            new eranos.internal.EranosException.Id(
                                    eranos.internal.IdFailure.EMPTY, "detail")),
                    new Conversion(
                            new IdFailure.TooLong(),
                            new eranos.internal.EranosException.Id(
                                    eranos.internal.IdFailure.TOO_LONG, "detail")),
                    new Conversion(
                            new IdFailure.ControlCharacter(),
                            new eranos.internal.EranosException.Id(
                                    eranos.internal.IdFailure.CONTROL_CHARACTER, "detail")),
                    new Conversion(
                            new IdFailure.SurroundingWhitespace(),
                            new eranos.internal.EranosException.Id(
                                    eranos.internal.IdFailure.SURROUNDING_WHITESPACE, "detail")),
                    new Conversion(
                            new PaymentAccountFailure.EmptyName(),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.EmptyName(),
                                    "detail")),
                    new Conversion(
                            new PaymentAccountFailure.InvalidIban(new IbanReason.InvalidChecksum()),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidChecksum()),
                                    "detail")),
                    new Conversion(
                            new PaymentAccountFailure.EmptyUsername(),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.EmptyUsername(),
                                    "detail")),
                    new Conversion(
                            new CalendarFailure.InvalidDate(),
                            new eranos.internal.EranosException.Calendar(
                                    eranos.internal.CalendarFailure.INVALID_DATE, "detail")),
                    new Conversion(
                            new CalendarFailure.InvalidTime(),
                            new eranos.internal.EranosException.Calendar(
                                    eranos.internal.CalendarFailure.INVALID_TIME, "detail")),
                    new Conversion(
                            new CategoryFailure.EmptyName(),
                            new eranos.internal.EranosException.Category(
                                    eranos.internal.CategoryFailure.EMPTY_NAME, "detail")),
                    new Conversion(
                            new ContactFailure.InvalidEmail(),
                            new eranos.internal.EranosException.Contact(
                                    eranos.internal.ContactFailure.INVALID_EMAIL, "detail")),
                    new Conversion(
                            new ContactFailure.InvalidPhoneNumber(),
                            new eranos.internal.EranosException.Contact(
                                    eranos.internal.ContactFailure.INVALID_PHONE_NUMBER, "detail")),
                    new Conversion(
                            new MoneyFailure.InvalidAmount(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.InvalidAmount(), "detail")),
                    new Conversion(
                            new MoneyFailure.TooManyDecimals(currency("RON"), 29L),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.TooManyDecimals(
                                            details("RON"), 29L),
                                    "detail")),
                    new Conversion(
                            new MoneyFailure.CurrencyMismatch(currency("RON"), currency("USD")),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.CurrencyMismatch(
                                            details("RON"), details("USD")),
                                    "detail")),
                    new Conversion(
                            new MoneyFailure.UnknownCurrency(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.UnknownCurrency(), "detail")),
                    new Conversion(
                            new MoneyFailure.InvalidExchangeRate(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.InvalidExchangeRate(),
                                    "detail")),
                    new Conversion(
                            new MoneyFailure.InvalidParts(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.InvalidParts(), "detail")),
                    new Conversion(
                            new MoneyFailure.InvalidPercentage(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.InvalidPercentage(),
                                    "detail")),
                    new Conversion(
                            new MoneyFailure.InvalidQuantity(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.InvalidQuantity(), "detail")),
                    new Conversion(
                            new MoneyFailure.NotFinite(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.NotFinite(), "detail")),
                    new Conversion(
                            new MoneyFailure.Overflow(),
                            new eranos.internal.EranosException.Money(
                                    new eranos.internal.MoneyFailure.Overflow(), "detail")),
                    new Conversion(
                            new MemberFailure.EmptyFirstName(),
                            new eranos.internal.EranosException.Member(
                                    eranos.internal.MemberFailure.EMPTY_FIRST_NAME, "detail")),
                    new Conversion(
                            new ExpenseFailure.EmptyName(),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.EmptyName(), "detail")),
                    new Conversion(
                            new ExpenseFailure.NonPositiveAmount(),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.NonPositiveAmount(),
                                    "detail")),
                    new Conversion(
                            new ExpenseFailure.CurrencyMismatch(currency("USD"), currency("GBP")),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.CurrencyMismatch(
                                            details("USD"), details("GBP")),
                                    "detail")),
                    new Conversion(
                            new ExpenseFailure.EmptySplit(),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.EmptySplit(), "detail")),
                    new Conversion(
                            new ExpenseFailure.DuplicateMember(new MemberId("member-6")),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.DuplicateMember("member-6"),
                                    "detail")),
                    new Conversion(
                            new ExpenseFailure.NonPositiveShare(new MemberId("member-9")),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.NonPositiveShare("member-9"),
                                    "detail")),
                    new Conversion(
                            new ExpenseFailure.AmountsDoNotAddUp(euros(3), euros(2)),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.AmountsDoNotAddUp(
                                            generatedEuros(3), generatedEuros(2)),
                                    "detail")),
                    new Conversion(
                            new ExpenseFailure.PercentagesDoNotAddUp(10L),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.PercentagesDoNotAddUp(10L),
                                    "detail")),
                    new Conversion(
                            new ExpenseFailure.NoItems(),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.NoItems(), "detail")),
                    new Conversion(
                            new ExpenseFailure.EmptyItemName(),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.EmptyItemName(), "detail")),
                    new Conversion(
                            new ExpenseFailure.ItemsDoNotAddUp(euros(8), euros(7)),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.ItemsDoNotAddUp(
                                            generatedEuros(8), generatedEuros(7)),
                                    "detail")),
                    new Conversion(
                            new ExpenseFailure.NegativeTip(),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.NegativeTip(), "detail")),
                    new Conversion(
                            new ExpenseFailure.Overflow(),
                            new eranos.internal.EranosException.Expense(
                                    new eranos.internal.ExpenseFailure.Overflow(), "detail")),
                    new Conversion(
                            new SettlementFailure.OutOfMemory(31L),
                            new eranos.internal.EranosException.Settlement(
                                    new eranos.internal.SettlementFailure.OutOfMemory(31L),
                                    "detail")),
                    new Conversion(
                            new TransferFailure.SameMember(),
                            new eranos.internal.EranosException.Transfer(
                                    eranos.internal.TransferFailure.SAME_MEMBER, "detail")),
                    new Conversion(
                            new TransferFailure.NonPositiveAmount(),
                            new eranos.internal.EranosException.Transfer(
                                    eranos.internal.TransferFailure.NON_POSITIVE_AMOUNT, "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.UnsupportedFormat(26L),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.UnsupportedFormat(26L),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.EmptyName(),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.EmptyName(), "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.MissingExchangeRate(currency("RON")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.MissingExchangeRate(
                                            details("RON")),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.UnexpectedExchangeRate(),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure
                                            .UnexpectedExchangeRate(),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.DuplicateExchangeRate(currency("GBP")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.DuplicateExchangeRate(
                                            details("GBP")),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.DuplicateMember(new MemberId("member-16")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.DuplicateMember(
                                            "member-16"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.UnknownMember(new MemberId("member-24")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.UnknownMember(
                                            "member-24"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.ArchivedMember(new MemberId("member-11")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.ArchivedMember(
                                            "member-11"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.MemberInUse(
                                    new MemberId("member-19"),
                                    List.of(new ExpenseId("expenses-18")),
                                    List.of(new TransferId("transfers-20"))),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.MemberInUse(
                                            "member-19",
                                            List.of("expenses-18"),
                                            List.of("transfers-20")),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.DuplicateCategory(
                                    new CategoryId("category-12")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.DuplicateCategory(
                                            "category-12"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.DuplicateCategoryName(
                                    new CategoryId("existing-13")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.DuplicateCategoryName(
                                            "existing-13"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.UnknownCategory(new CategoryId("category-22")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.UnknownCategory(
                                            "category-22"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.BuiltInCategory(),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.BuiltInCategory(),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.DuplicateExpense(new ExpenseId("expense-15")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.DuplicateExpense(
                                            "expense-15"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.UnknownExpense(new ExpenseId("expense-23")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.UnknownExpense(
                                            "expense-23"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.DuplicateTransfer(
                                    new TransferId("transfer-17")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.DuplicateTransfer(
                                            "transfer-17"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.UnknownTransfer(new TransferId("transfer-25")),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.UnknownTransfer(
                                            "transfer-25"),
                                    "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.Busy(),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.Busy(), "detail")),
                    new Conversion(
                            new ExpenseGroupFailure.Overflow(),
                            new eranos.internal.EranosException.ExpenseGroup(
                                    new eranos.internal.ExpenseGroupFailure.Overflow(), "detail")),
                    new Conversion(
                            new LanguageFailure.InvalidLanguage(),
                            new eranos.internal.EranosException.Language(
                                    eranos.internal.LanguageFailure.INVALID_LANGUAGE, "detail")),
                    new Conversion(
                            new PaymentAccountFailure.InvalidIban(
                                    new IbanReason.InvalidCharacter()),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidCharacter()),
                                    "detail")),
                    new Conversion(
                            new PaymentAccountFailure.InvalidIban(new IbanReason.InvalidFormat()),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidFormat()),
                                    "detail")),
                    new Conversion(
                            new PaymentAccountFailure.InvalidIban(new IbanReason.UnknownCountry()),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.UnknownCountry()),
                                    "detail")),
                    new Conversion(
                            new PaymentAccountFailure.InvalidIban(
                                    new IbanReason.WrongLength(24L, 22L)),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.WrongLength(24L, 22L)),
                                    "detail")),
                    new Conversion(
                            new PaymentAccountFailure.InvalidIban(
                                    new IbanReason.InvalidStructure()),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidStructure()),
                                    "detail")),
                    new Conversion(
                            new PaymentAccountFailure.InvalidIban(new IbanReason.InvalidChecksum()),
                            new eranos.internal.EranosException.PaymentAccount(
                                    new eranos.internal.PaymentAccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidChecksum()),
                                    "detail")));
}
