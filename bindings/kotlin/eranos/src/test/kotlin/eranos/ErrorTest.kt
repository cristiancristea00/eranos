// The exception every call throws. A failure converted to another module's subclass, to another failure, with another
// payload, or without the library's message would show an app another failure than the library's, so every failure the
// generated code can report is converted here, and every message is read back.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertInstanceOf
import org.junit.jupiter.api.Test
import kotlin.coroutines.cancellation.CancellationException

/** The conversions of every failure the generated code can report. */
class ErrorTest {
    /**
     * A conversion that reached another module or failure, dropped what the failure holds, looked a currency up by its code
     * rather than keeping the details the library gave, or lost the message would show an app another failure than the
     * library's; a failure that hashed or printed with a field left out would merge two failures an app tells apart.
     */
    @Test
    fun everyFailureOfTheGeneratedCodeConvertsToItsModuleAndFailure() {
        assertEquals(65, conversions.size)
        for (conversion in conversions) {
            val converted = assertInstanceOf(EranosException::class.java, conversion.generated.converted())
            assertEquals(conversion.failure, converted.failure())
            assertEquals(conversion.failure.hashCode(), converted.failure().hashCode())
            assertEquals(conversion.failure.toString(), converted.failure().toString())
            assertEquals("detail", converted.message)
        }
    }

    /**
     * A saved group that failed to load without its line and column would leave an app nothing to point the user at; a
     * call the program should not make, reported as a failure of the library, would be caught as one instead of failing
     * the program; and a cancelled call reported as anything but a cancellation would fail the coroutine's parent.
     */
    @Test
    fun theBindingsOwnFailuresConvertToTheirStandardExceptions() {
        val json =
            assertInstanceOf(
                EranosException.InvalidJson::class.java,
                eranos.internal.EranosException
                    .InvalidJson(1, 2, "detail")
                    .converted(),
            )
        assertEquals(1L, json.line)
        assertEquals(2L, json.column)
        assertEquals("detail", json.message)
        assertStandard(eranos.internal.EranosException.Cancelled("detail"), CancellationException::class.java)
        assertStandard(eranos.internal.EranosException.Released("detail"), IllegalStateException::class.java)
    }
}

/** A failure of the generated code, beside the failure it must become. */
private data class Conversion(
    val failure: Any,
    val generated: eranos.internal.EranosException,
)

/** Where a saved group that is not JSON of the saved form stops being read. */
private data class Position(
    val line: Long,
    val column: Long,
)

/** Checks that [generated] converts to a [standard] exception with its message. */
private fun assertStandard(
    generated: eranos.internal.EranosException,
    standard: Class<out Exception>,
) {
    val converted = generated.converted()
    assertEquals(standard, converted::class.java)
    assertEquals("detail", converted.message)
}

/** The failure [this] holds, or its position for a saved group that is not JSON of the saved form. */
private fun EranosException.failure(): Any =
    when (this) {
        is EranosException.Id -> failure
        is EranosException.PaymentAccount -> failure
        is EranosException.Calendar -> failure
        is EranosException.Category -> failure
        is EranosException.Contact -> failure
        is EranosException.Money -> failure
        is EranosException.Member -> failure
        is EranosException.Expense -> failure
        is EranosException.Settlement -> failure
        is EranosException.Transfer -> failure
        is EranosException.ExpenseGroup -> failure
        is EranosException.Language -> failure
        is EranosException.InvalidJson -> Position(line, column)
    }

/**
 * A currency of [code] with details the library gives no currency, which a conversion that looked the code up would replace
 * with the library's.
 */
private fun unlisted(code: String): CurrencyCode = CurrencyCode(code, "Unlisted", 0, 7)

/** [minorUnits] in the unlisted currency of the code "EUR". */
private fun euros(minorUnits: Long): Money = Money(minorUnits, unlisted("EUR"))

/** Every failure the generated code can report, with every reason of a refused IBAN, beside the failure it becomes. */
private val conversions: List<Conversion> =
    listOf(
        Conversion(failure = IdFailure.Empty, generated = eranos.internal.EranosException.Id(eranos.internal.IdFailure.EMPTY, "detail")),
        Conversion(failure = IdFailure.TooLong, generated = eranos.internal.EranosException.Id(eranos.internal.IdFailure.TOO_LONG, "detail")),
        Conversion(
            failure = IdFailure.ControlCharacter,
            generated = eranos.internal.EranosException.Id(eranos.internal.IdFailure.CONTROL_CHARACTER, "detail"),
        ),
        Conversion(
            failure = IdFailure.SurroundingWhitespace,
            generated = eranos.internal.EranosException.Id(eranos.internal.IdFailure.SURROUNDING_WHITESPACE, "detail"),
        ),
        Conversion(
            failure = PaymentAccountFailure.EmptyName,
            generated = eranos.internal.EranosException.PaymentAccount(eranos.internal.PaymentAccountFailure.EmptyName, "detail"),
        ),
        Conversion(
            failure = PaymentAccountFailure.InvalidIban(IbanReason.InvalidChecksum),
            generated =
                eranos.internal.EranosException.PaymentAccount(
                    eranos.internal.PaymentAccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidChecksum),
                    "detail",
                ),
        ),
        Conversion(
            failure = PaymentAccountFailure.EmptyUsername,
            generated = eranos.internal.EranosException.PaymentAccount(eranos.internal.PaymentAccountFailure.EmptyUsername, "detail"),
        ),
        Conversion(
            failure = CalendarFailure.InvalidDate,
            generated = eranos.internal.EranosException.Calendar(eranos.internal.CalendarFailure.INVALID_DATE, "detail"),
        ),
        Conversion(
            failure = CalendarFailure.InvalidTime,
            generated = eranos.internal.EranosException.Calendar(eranos.internal.CalendarFailure.INVALID_TIME, "detail"),
        ),
        Conversion(
            failure = CategoryFailure.EmptyName,
            generated = eranos.internal.EranosException.Category(eranos.internal.CategoryFailure.EMPTY_NAME, "detail"),
        ),
        Conversion(
            failure = ContactFailure.InvalidEmail,
            generated = eranos.internal.EranosException.Contact(eranos.internal.ContactFailure.INVALID_EMAIL, "detail"),
        ),
        Conversion(
            failure = ContactFailure.InvalidPhoneNumber,
            generated = eranos.internal.EranosException.Contact(eranos.internal.ContactFailure.INVALID_PHONE_NUMBER, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.InvalidAmount,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.InvalidAmount, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.TooManyDecimals(unlisted("RON"), 29),
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.TooManyDecimals(unlisted("RON").generated, 29), "detail"),
        ),
        Conversion(
            failure = MoneyFailure.CurrencyMismatch(unlisted("RON"), unlisted("USD")),
            generated =
                eranos.internal.EranosException.Money(
                    eranos.internal.MoneyFailure.CurrencyMismatch(unlisted("RON").generated, unlisted("USD").generated),
                    "detail",
                ),
        ),
        Conversion(
            failure = MoneyFailure.UnknownCurrency,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.UnknownCurrency, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.InvalidExchangeRate,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.InvalidExchangeRate, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.InvalidParts,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.InvalidParts, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.InvalidPercentage,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.InvalidPercentage, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.InvalidQuantity,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.InvalidQuantity, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.NotFinite,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.NotFinite, "detail"),
        ),
        Conversion(
            failure = MoneyFailure.Overflow,
            generated = eranos.internal.EranosException.Money(eranos.internal.MoneyFailure.Overflow, "detail"),
        ),
        Conversion(
            failure = MemberFailure.EmptyFirstName,
            generated = eranos.internal.EranosException.Member(eranos.internal.MemberFailure.EMPTY_FIRST_NAME, "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.EmptyName,
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.EmptyName, "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.NonPositiveAmount,
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.NonPositiveAmount, "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.CurrencyMismatch(unlisted("USD"), unlisted("GBP")),
            generated =
                eranos.internal.EranosException.Expense(
                    eranos.internal.ExpenseFailure.CurrencyMismatch(unlisted("USD").generated, unlisted("GBP").generated),
                    "detail",
                ),
        ),
        Conversion(
            failure = ExpenseFailure.EmptySplit,
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.EmptySplit, "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.DuplicateMember(MemberId("member-6")),
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.DuplicateMember("member-6"), "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.NonPositiveShare(MemberId("member-9")),
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.NonPositiveShare("member-9"), "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.AmountsDoNotAddUp(euros(3), euros(2)),
            generated =
                eranos.internal.EranosException.Expense(
                    eranos.internal.ExpenseFailure.AmountsDoNotAddUp(euros(3).generated, euros(2).generated),
                    "detail",
                ),
        ),
        Conversion(
            failure = ExpenseFailure.PercentagesDoNotAddUp(10),
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.PercentagesDoNotAddUp(10), "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.NoItems,
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.NoItems, "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.EmptyItemName,
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.EmptyItemName, "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.ItemsDoNotAddUp(euros(8), euros(7)),
            generated =
                eranos.internal.EranosException.Expense(
                    eranos.internal.ExpenseFailure.ItemsDoNotAddUp(euros(8).generated, euros(7).generated),
                    "detail",
                ),
        ),
        Conversion(
            failure = ExpenseFailure.NegativeTip,
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.NegativeTip, "detail"),
        ),
        Conversion(
            failure = ExpenseFailure.Overflow,
            generated = eranos.internal.EranosException.Expense(eranos.internal.ExpenseFailure.Overflow, "detail"),
        ),
        Conversion(
            failure = SettlementFailure.OutOfMemory(31),
            generated = eranos.internal.EranosException.Settlement(eranos.internal.SettlementFailure.OutOfMemory(31), "detail"),
        ),
        Conversion(
            failure = TransferFailure.SameMember,
            generated = eranos.internal.EranosException.Transfer(eranos.internal.TransferFailure.SAME_MEMBER, "detail"),
        ),
        Conversion(
            failure = TransferFailure.NonPositiveAmount,
            generated = eranos.internal.EranosException.Transfer(eranos.internal.TransferFailure.NON_POSITIVE_AMOUNT, "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.UnsupportedFormat(26),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.UnsupportedFormat(26), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.EmptyName,
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.EmptyName, "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.MissingExchangeRate(unlisted("RON")),
            generated =
                eranos.internal.EranosException.ExpenseGroup(
                    eranos.internal.ExpenseGroupFailure.MissingExchangeRate(unlisted("RON").generated),
                    "detail",
                ),
        ),
        Conversion(
            failure = ExpenseGroupFailure.UnexpectedExchangeRate,
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.UnexpectedExchangeRate, "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.DuplicateExchangeRate(unlisted("GBP")),
            generated =
                eranos.internal.EranosException.ExpenseGroup(
                    eranos.internal.ExpenseGroupFailure.DuplicateExchangeRate(unlisted("GBP").generated),
                    "detail",
                ),
        ),
        Conversion(
            failure = ExpenseGroupFailure.DuplicateMember(MemberId("member-16")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.DuplicateMember("member-16"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.UnknownMember(MemberId("member-24")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.UnknownMember("member-24"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.ArchivedMember(MemberId("member-11")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.ArchivedMember("member-11"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.MemberInUse(MemberId("member-19"), listOf(ExpenseId("expenses-18")), listOf(TransferId("transfers-20"))),
            generated =
                eranos.internal.EranosException.ExpenseGroup(
                    eranos.internal.ExpenseGroupFailure.MemberInUse("member-19", listOf("expenses-18"), listOf("transfers-20")),
                    "detail",
                ),
        ),
        Conversion(
            failure = ExpenseGroupFailure.DuplicateCategory(CategoryId("category-12")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.DuplicateCategory("category-12"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.DuplicateCategoryName(CategoryId("existing-13")),
            generated =
                eranos.internal.EranosException.ExpenseGroup(
                    eranos.internal.ExpenseGroupFailure.DuplicateCategoryName("existing-13"),
                    "detail",
                ),
        ),
        Conversion(
            failure = ExpenseGroupFailure.UnknownCategory(CategoryId("category-22")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.UnknownCategory("category-22"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.BuiltInCategory,
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.BuiltInCategory, "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.DuplicateExpense(ExpenseId("expense-15")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.DuplicateExpense("expense-15"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.UnknownExpense(ExpenseId("expense-23")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.UnknownExpense("expense-23"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.DuplicateTransfer(TransferId("transfer-17")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.DuplicateTransfer("transfer-17"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.UnknownTransfer(TransferId("transfer-25")),
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.UnknownTransfer("transfer-25"), "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.Busy,
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.Busy, "detail"),
        ),
        Conversion(
            failure = ExpenseGroupFailure.Overflow,
            generated = eranos.internal.EranosException.ExpenseGroup(eranos.internal.ExpenseGroupFailure.Overflow, "detail"),
        ),
        Conversion(
            failure = LanguageFailure.InvalidLanguage,
            generated = eranos.internal.EranosException.Language(eranos.internal.LanguageFailure.INVALID_LANGUAGE, "detail"),
        ),
        Conversion(
            failure = PaymentAccountFailure.InvalidIban(IbanReason.InvalidCharacter),
            generated =
                eranos.internal.EranosException.PaymentAccount(
                    eranos.internal.PaymentAccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidCharacter),
                    "detail",
                ),
        ),
        Conversion(
            failure = PaymentAccountFailure.InvalidIban(IbanReason.InvalidFormat),
            generated =
                eranos.internal.EranosException.PaymentAccount(
                    eranos.internal.PaymentAccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidFormat),
                    "detail",
                ),
        ),
        Conversion(
            failure = PaymentAccountFailure.InvalidIban(IbanReason.UnknownCountry),
            generated =
                eranos.internal.EranosException.PaymentAccount(
                    eranos.internal.PaymentAccountFailure.InvalidIban(eranos.internal.IbanReason.UnknownCountry),
                    "detail",
                ),
        ),
        Conversion(
            failure = PaymentAccountFailure.InvalidIban(IbanReason.WrongLength(24, 22)),
            generated =
                eranos.internal.EranosException.PaymentAccount(
                    eranos.internal.PaymentAccountFailure.InvalidIban(eranos.internal.IbanReason.WrongLength(24, 22)),
                    "detail",
                ),
        ),
        Conversion(
            failure = PaymentAccountFailure.InvalidIban(IbanReason.InvalidStructure),
            generated =
                eranos.internal.EranosException.PaymentAccount(
                    eranos.internal.PaymentAccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidStructure),
                    "detail",
                ),
        ),
        Conversion(
            failure = PaymentAccountFailure.InvalidIban(IbanReason.InvalidChecksum),
            generated =
                eranos.internal.EranosException.PaymentAccount(
                    eranos.internal.PaymentAccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidChecksum),
                    "detail",
                ),
        ),
    )
