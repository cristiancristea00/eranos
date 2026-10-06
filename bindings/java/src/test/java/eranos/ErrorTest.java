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
    private record Position(long column, long line) {}

    /**
     * A conversion that reached another module or failure, dropped what the failure holds, or lost
     * the message would show an app another failure than the library's.
     */
    @Test
    void everyFailureOfTheGeneratedCodeConvertsToItsModuleAndFailure() {
        assertEquals(61, CONVERSIONS.size());
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
                                new eranos.internal.EranosException.InvalidJson(2L, "detail", 1L)));
        assertEquals(new Position(2L, 1L), failure(json));
        assertEquals("detail", json.getMessage());
        assertStandard(
                new eranos.internal.EranosException.Cancelled("detail"),
                CancellationException.class);
        assertStandard(
                new eranos.internal.EranosException.IdentifierGiven("detail"),
                IllegalArgumentException.class);
        assertStandard(
                new eranos.internal.EranosException.IdentifierMissing("detail"),
                IllegalArgumentException.class);
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
        assertEquals(17, cases);
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
            case EranosException.Account account -> account.failure();
            case EranosException.Calendar calendar -> calendar.failure();
            case EranosException.Category category -> category.failure();
            case EranosException.Contact contact -> contact.failure();
            case EranosException.Expense expense -> expense.failure();
            case EranosException.Group group -> group.failure();
            case EranosException.Id id -> id.failure();
            case EranosException.Language language -> language.failure();
            case EranosException.Member member -> member.failure();
            case EranosException.Money money -> money.failure();
            case EranosException.Settlement settlement -> settlement.failure();
            case EranosException.Transfer transfer -> transfer.failure();
            case EranosException.InvalidJson json -> new Position(json.column(), json.line());
        };
    }

    /** {@code minorUnits} cents. */
    private static Money euros(long minorUnits) {
        return new Money(Currency.known("EUR"), minorUnits);
    }

    /** Every failure the generated code can report, with every reason of a refused IBAN. */
    private static final List<Conversion> CONVERSIONS =
            List.of(
                    new Conversion(
                            new AccountFailure.EmptyName(),
                            new eranos.internal.EranosException.Account(
                                    "detail", new eranos.internal.AccountFailure.EmptyName())),
                    new Conversion(
                            new AccountFailure.EmptyUsername(),
                            new eranos.internal.EranosException.Account(
                                    "detail", new eranos.internal.AccountFailure.EmptyUsername())),
                    new Conversion(
                            new AccountFailure.InvalidIban(new IbanReason.InvalidChecksum()),
                            new eranos.internal.EranosException.Account(
                                    "detail",
                                    new eranos.internal.AccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidChecksum()))),
                    new Conversion(
                            new CalendarFailure.InvalidDate(),
                            new eranos.internal.EranosException.Calendar(
                                    "detail", eranos.internal.CalendarFailure.INVALID_DATE)),
                    new Conversion(
                            new CalendarFailure.InvalidTime(),
                            new eranos.internal.EranosException.Calendar(
                                    "detail", eranos.internal.CalendarFailure.INVALID_TIME)),
                    new Conversion(
                            new CategoryFailure.EmptyName(),
                            new eranos.internal.EranosException.Category(
                                    "detail", eranos.internal.CategoryFailure.EMPTY_NAME)),
                    new Conversion(
                            new ContactFailure.InvalidEmail(),
                            new eranos.internal.EranosException.Contact(
                                    "detail", eranos.internal.ContactFailure.INVALID_EMAIL)),
                    new Conversion(
                            new ContactFailure.InvalidPhoneNumber(),
                            new eranos.internal.EranosException.Contact(
                                    "detail", eranos.internal.ContactFailure.INVALID_PHONE_NUMBER)),
                    new Conversion(
                            new ExpenseFailure.AmountsDoNotAddUp(euros(2), euros(3)),
                            new eranos.internal.EranosException.Expense(
                                    "detail",
                                    new eranos.internal.ExpenseFailure.AmountsDoNotAddUp(
                                            new eranos.internal.Money("EUR", 2),
                                            new eranos.internal.Money("EUR", 3)))),
                    new Conversion(
                            new ExpenseFailure.CurrencyMismatch(
                                    Currency.known("USD"), Currency.known("GBP")),
                            new eranos.internal.EranosException.Expense(
                                    "detail",
                                    new eranos.internal.ExpenseFailure.CurrencyMismatch(
                                            "USD", "GBP"))),
                    new Conversion(
                            new ExpenseFailure.DuplicateMember(new MemberId("member-6")),
                            new eranos.internal.EranosException.Expense(
                                    "detail",
                                    new eranos.internal.ExpenseFailure.DuplicateMember(
                                            "member-6"))),
                    new Conversion(
                            new ExpenseFailure.EmptyItemName(),
                            new eranos.internal.EranosException.Expense(
                                    "detail", new eranos.internal.ExpenseFailure.EmptyItemName())),
                    new Conversion(
                            new ExpenseFailure.EmptyName(),
                            new eranos.internal.EranosException.Expense(
                                    "detail", new eranos.internal.ExpenseFailure.EmptyName())),
                    new Conversion(
                            new ExpenseFailure.EmptySplit(),
                            new eranos.internal.EranosException.Expense(
                                    "detail", new eranos.internal.ExpenseFailure.EmptySplit())),
                    new Conversion(
                            new ExpenseFailure.ItemsDoNotAddUp(euros(7), euros(8)),
                            new eranos.internal.EranosException.Expense(
                                    "detail",
                                    new eranos.internal.ExpenseFailure.ItemsDoNotAddUp(
                                            new eranos.internal.Money("EUR", 7),
                                            new eranos.internal.Money("EUR", 8)))),
                    new Conversion(
                            new ExpenseFailure.NegativeTip(),
                            new eranos.internal.EranosException.Expense(
                                    "detail", new eranos.internal.ExpenseFailure.NegativeTip())),
                    new Conversion(
                            new ExpenseFailure.NoItems(),
                            new eranos.internal.EranosException.Expense(
                                    "detail", new eranos.internal.ExpenseFailure.NoItems())),
                    new Conversion(
                            new ExpenseFailure.NonPositiveAmount(),
                            new eranos.internal.EranosException.Expense(
                                    "detail",
                                    new eranos.internal.ExpenseFailure.NonPositiveAmount())),
                    new Conversion(
                            new ExpenseFailure.NonPositiveShare(new MemberId("member-9")),
                            new eranos.internal.EranosException.Expense(
                                    "detail",
                                    new eranos.internal.ExpenseFailure.NonPositiveShare(
                                            "member-9"))),
                    new Conversion(
                            new ExpenseFailure.Overflow(),
                            new eranos.internal.EranosException.Expense(
                                    "detail", new eranos.internal.ExpenseFailure.Overflow())),
                    new Conversion(
                            new ExpenseFailure.PercentagesDoNotAddUp(10L),
                            new eranos.internal.EranosException.Expense(
                                    "detail",
                                    new eranos.internal.ExpenseFailure.PercentagesDoNotAddUp(10L))),
                    new Conversion(
                            new ExpenseFailure.ZeroQuantity(),
                            new eranos.internal.EranosException.Expense(
                                    "detail", new eranos.internal.ExpenseFailure.ZeroQuantity())),
                    new Conversion(
                            new GroupFailure.ArchivedMember(new MemberId("member-11")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.ArchivedMember("member-11"))),
                    new Conversion(
                            new GroupFailure.BuiltInCategory(),
                            new eranos.internal.EranosException.Group(
                                    "detail", new eranos.internal.GroupFailure.BuiltInCategory())),
                    new Conversion(
                            new GroupFailure.Busy(),
                            new eranos.internal.EranosException.Group(
                                    "detail", new eranos.internal.GroupFailure.Busy())),
                    new Conversion(
                            new GroupFailure.DuplicateCategory(new CategoryId("category-12")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.DuplicateCategory(
                                            "category-12"))),
                    new Conversion(
                            new GroupFailure.DuplicateCategoryName(new CategoryId("existing-13")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.DuplicateCategoryName(
                                            "existing-13"))),
                    new Conversion(
                            new GroupFailure.DuplicateExchangeRate(Currency.known("GBP")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.DuplicateExchangeRate("GBP"))),
                    new Conversion(
                            new GroupFailure.DuplicateExpense(new ExpenseId("expense-15")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.DuplicateExpense(
                                            "expense-15"))),
                    new Conversion(
                            new GroupFailure.DuplicateMember(new MemberId("member-16")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.DuplicateMember("member-16"))),
                    new Conversion(
                            new GroupFailure.DuplicateTransfer(new TransferId("transfer-17")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.DuplicateTransfer(
                                            "transfer-17"))),
                    new Conversion(
                            new GroupFailure.EmptyName(),
                            new eranos.internal.EranosException.Group(
                                    "detail", new eranos.internal.GroupFailure.EmptyName())),
                    new Conversion(
                            new GroupFailure.MemberInUse(
                                    List.of(new ExpenseId("expenses-18")),
                                    new MemberId("member-19"),
                                    List.of(new TransferId("transfers-20"))),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.MemberInUse(
                                            List.of("expenses-18"),
                                            "member-19",
                                            List.of("transfers-20")))),
                    new Conversion(
                            new GroupFailure.MissingExchangeRate(Currency.known("RON")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.MissingExchangeRate("RON"))),
                    new Conversion(
                            new GroupFailure.Overflow(),
                            new eranos.internal.EranosException.Group(
                                    "detail", new eranos.internal.GroupFailure.Overflow())),
                    new Conversion(
                            new GroupFailure.UnexpectedExchangeRate(),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.UnexpectedExchangeRate())),
                    new Conversion(
                            new GroupFailure.UnknownCategory(new CategoryId("category-22")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.UnknownCategory(
                                            "category-22"))),
                    new Conversion(
                            new GroupFailure.UnknownExpense(new ExpenseId("expense-23")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.UnknownExpense("expense-23"))),
                    new Conversion(
                            new GroupFailure.UnknownMember(new MemberId("member-24")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.UnknownMember("member-24"))),
                    new Conversion(
                            new GroupFailure.UnknownTransfer(new TransferId("transfer-25")),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.UnknownTransfer(
                                            "transfer-25"))),
                    new Conversion(
                            new GroupFailure.UnsupportedFormat(26L),
                            new eranos.internal.EranosException.Group(
                                    "detail",
                                    new eranos.internal.GroupFailure.UnsupportedFormat(26L))),
                    new Conversion(
                            new IdFailure.InvalidId(),
                            new eranos.internal.EranosException.Id(
                                    "detail", eranos.internal.IdFailure.INVALID_ID)),
                    new Conversion(
                            new LanguageFailure.InvalidLanguage(),
                            new eranos.internal.EranosException.Language(
                                    "detail", eranos.internal.LanguageFailure.INVALID_LANGUAGE)),
                    new Conversion(
                            new MemberFailure.EmptyFirstName(),
                            new eranos.internal.EranosException.Member(
                                    "detail", eranos.internal.MemberFailure.EMPTY_FIRST_NAME)),
                    new Conversion(
                            new MoneyFailure.CurrencyMismatch(
                                    Currency.known("RON"), Currency.known("USD")),
                            new eranos.internal.EranosException.Money(
                                    "detail",
                                    new eranos.internal.MoneyFailure.CurrencyMismatch(
                                            "RON", "USD"))),
                    new Conversion(
                            new MoneyFailure.InvalidAmount(),
                            new eranos.internal.EranosException.Money(
                                    "detail", new eranos.internal.MoneyFailure.InvalidAmount())),
                    new Conversion(
                            new MoneyFailure.InvalidExchangeRate(),
                            new eranos.internal.EranosException.Money(
                                    "detail",
                                    new eranos.internal.MoneyFailure.InvalidExchangeRate())),
                    new Conversion(
                            new MoneyFailure.InvalidParts(),
                            new eranos.internal.EranosException.Money(
                                    "detail", new eranos.internal.MoneyFailure.InvalidParts())),
                    new Conversion(
                            new MoneyFailure.InvalidPercentage(),
                            new eranos.internal.EranosException.Money(
                                    "detail",
                                    new eranos.internal.MoneyFailure.InvalidPercentage())),
                    new Conversion(
                            new MoneyFailure.Overflow(),
                            new eranos.internal.EranosException.Money(
                                    "detail", new eranos.internal.MoneyFailure.Overflow())),
                    new Conversion(
                            new MoneyFailure.TooManyDecimals(29L, Currency.known("RON")),
                            new eranos.internal.EranosException.Money(
                                    "detail",
                                    new eranos.internal.MoneyFailure.TooManyDecimals(29L, "RON"))),
                    new Conversion(
                            new MoneyFailure.UnknownCurrency(),
                            new eranos.internal.EranosException.Money(
                                    "detail", new eranos.internal.MoneyFailure.UnknownCurrency())),
                    new Conversion(
                            new SettlementFailure.OutOfMemory(31L),
                            new eranos.internal.EranosException.Settlement(
                                    "detail",
                                    new eranos.internal.SettlementFailure.OutOfMemory(31L))),
                    new Conversion(
                            new TransferFailure.NonPositiveAmount(),
                            new eranos.internal.EranosException.Transfer(
                                    "detail", eranos.internal.TransferFailure.NON_POSITIVE_AMOUNT)),
                    new Conversion(
                            new TransferFailure.SameMember(),
                            new eranos.internal.EranosException.Transfer(
                                    "detail", eranos.internal.TransferFailure.SAME_MEMBER)),
                    new Conversion(
                            new AccountFailure.InvalidIban(new IbanReason.InvalidCharacter()),
                            new eranos.internal.EranosException.Account(
                                    "detail",
                                    new eranos.internal.AccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidCharacter()))),
                    new Conversion(
                            new AccountFailure.InvalidIban(new IbanReason.InvalidChecksum()),
                            new eranos.internal.EranosException.Account(
                                    "detail",
                                    new eranos.internal.AccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidChecksum()))),
                    new Conversion(
                            new AccountFailure.InvalidIban(new IbanReason.InvalidFormat()),
                            new eranos.internal.EranosException.Account(
                                    "detail",
                                    new eranos.internal.AccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidFormat()))),
                    new Conversion(
                            new AccountFailure.InvalidIban(new IbanReason.InvalidStructure()),
                            new eranos.internal.EranosException.Account(
                                    "detail",
                                    new eranos.internal.AccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.InvalidStructure()))),
                    new Conversion(
                            new AccountFailure.InvalidIban(new IbanReason.UnknownCountry()),
                            new eranos.internal.EranosException.Account(
                                    "detail",
                                    new eranos.internal.AccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.UnknownCountry()))),
                    new Conversion(
                            new AccountFailure.InvalidIban(new IbanReason.WrongLength(24L, 22L)),
                            new eranos.internal.EranosException.Account(
                                    "detail",
                                    new eranos.internal.AccountFailure.InvalidIban(
                                            new eranos.internal.IbanReason.WrongLength(
                                                    24L, 22L)))));
}
