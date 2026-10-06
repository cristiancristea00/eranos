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
     * A conversion that reached another module or failure, dropped what the failure holds, or lost the message would show
     * an app another failure than the library's; a failure that hashed or printed with a field left out would merge two
     * failures an app tells apart.
     */
    @Test
    fun everyFailureOfTheGeneratedCodeConvertsToItsModuleAndFailure() {
        assertEquals(61, conversions.size)
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
                    .InvalidJson(2, "detail", 1)
                    .converted(),
            )
        assertEquals(2L, json.column)
        assertEquals(1L, json.line)
        assertEquals("detail", json.message)
        assertStandard(eranos.internal.EranosException.Cancelled("detail"), CancellationException::class.java)
        assertStandard(eranos.internal.EranosException.IdentifierGiven("detail"), IllegalArgumentException::class.java)
        assertStandard(eranos.internal.EranosException.IdentifierMissing("detail"), IllegalArgumentException::class.java)
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
    val column: Long,
    val line: Long,
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
        is EranosException.Account -> failure
        is EranosException.Calendar -> failure
        is EranosException.Category -> failure
        is EranosException.Contact -> failure
        is EranosException.Expense -> failure
        is EranosException.Group -> failure
        is EranosException.Id -> failure
        is EranosException.Language -> failure
        is EranosException.Member -> failure
        is EranosException.Money -> failure
        is EranosException.Settlement -> failure
        is EranosException.Transfer -> failure
        is EranosException.InvalidJson -> Position(column, line)
    }

/** [minorUnits] cents. */
private fun euros(minorUnits: Long): Money = Money(Currency.known("EUR"), minorUnits)

/** Every failure the generated code can report, with every reason of a refused IBAN, beside the failure it becomes. */
private val conversions: List<Conversion> =
    listOf(
        Conversion(
            failure = AccountFailure.EmptyName,
            generated = eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.EmptyName),
        ),
        Conversion(
            failure = AccountFailure.EmptyUsername,
            generated = eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.EmptyUsername),
        ),
        Conversion(
            failure = AccountFailure.InvalidIban(IbanReason.InvalidChecksum),
            generated =
                eranos.internal.EranosException.Account(
                    "detail",
                    eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidChecksum),
                ),
        ),
        Conversion(
            failure = CalendarFailure.InvalidDate,
            generated = eranos.internal.EranosException.Calendar("detail", eranos.internal.CalendarFailure.INVALID_DATE),
        ),
        Conversion(
            failure = CalendarFailure.InvalidTime,
            generated = eranos.internal.EranosException.Calendar("detail", eranos.internal.CalendarFailure.INVALID_TIME),
        ),
        Conversion(
            failure = CategoryFailure.EmptyName,
            generated = eranos.internal.EranosException.Category("detail", eranos.internal.CategoryFailure.EMPTY_NAME),
        ),
        Conversion(
            failure = ContactFailure.InvalidEmail,
            generated = eranos.internal.EranosException.Contact("detail", eranos.internal.ContactFailure.INVALID_EMAIL),
        ),
        Conversion(
            failure = ContactFailure.InvalidPhoneNumber,
            generated = eranos.internal.EranosException.Contact("detail", eranos.internal.ContactFailure.INVALID_PHONE_NUMBER),
        ),
        Conversion(
            failure = ExpenseFailure.AmountsDoNotAddUp(euros(2), euros(3)),
            generated =
                eranos.internal.EranosException.Expense(
                    "detail",
                    eranos.internal.ExpenseFailure.AmountsDoNotAddUp(eranos.internal.Money("EUR", 2), eranos.internal.Money("EUR", 3)),
                ),
        ),
        Conversion(
            failure = ExpenseFailure.CurrencyMismatch(Currency.known("USD"), Currency.known("GBP")),
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.CurrencyMismatch("USD", "GBP")),
        ),
        Conversion(
            failure = ExpenseFailure.DuplicateMember(MemberId("member-6")),
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.DuplicateMember("member-6")),
        ),
        Conversion(
            failure = ExpenseFailure.EmptyItemName,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.EmptyItemName),
        ),
        Conversion(
            failure = ExpenseFailure.EmptyName,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.EmptyName),
        ),
        Conversion(
            failure = ExpenseFailure.EmptySplit,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.EmptySplit),
        ),
        Conversion(
            failure = ExpenseFailure.ItemsDoNotAddUp(euros(7), euros(8)),
            generated =
                eranos.internal.EranosException.Expense(
                    "detail",
                    eranos.internal.ExpenseFailure.ItemsDoNotAddUp(eranos.internal.Money("EUR", 7), eranos.internal.Money("EUR", 8)),
                ),
        ),
        Conversion(
            failure = ExpenseFailure.NegativeTip,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NegativeTip),
        ),
        Conversion(
            failure = ExpenseFailure.NoItems,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NoItems),
        ),
        Conversion(
            failure = ExpenseFailure.NonPositiveAmount,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NonPositiveAmount),
        ),
        Conversion(
            failure = ExpenseFailure.NonPositiveShare(MemberId("member-9")),
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NonPositiveShare("member-9")),
        ),
        Conversion(
            failure = ExpenseFailure.Overflow,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.Overflow),
        ),
        Conversion(
            failure = ExpenseFailure.PercentagesDoNotAddUp(10),
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.PercentagesDoNotAddUp(10)),
        ),
        Conversion(
            failure = ExpenseFailure.ZeroQuantity,
            generated = eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.ZeroQuantity),
        ),
        Conversion(
            failure = GroupFailure.ArchivedMember(MemberId("member-11")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.ArchivedMember("member-11")),
        ),
        Conversion(
            failure = GroupFailure.BuiltInCategory,
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.BuiltInCategory),
        ),
        Conversion(failure = GroupFailure.Busy, generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.Busy)),
        Conversion(
            failure = GroupFailure.DuplicateCategory(CategoryId("category-12")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateCategory("category-12")),
        ),
        Conversion(
            failure = GroupFailure.DuplicateCategoryName(CategoryId("existing-13")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateCategoryName("existing-13")),
        ),
        Conversion(
            failure = GroupFailure.DuplicateExchangeRate(Currency.known("GBP")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateExchangeRate("GBP")),
        ),
        Conversion(
            failure = GroupFailure.DuplicateExpense(ExpenseId("expense-15")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateExpense("expense-15")),
        ),
        Conversion(
            failure = GroupFailure.DuplicateMember(MemberId("member-16")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateMember("member-16")),
        ),
        Conversion(
            failure = GroupFailure.DuplicateTransfer(TransferId("transfer-17")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateTransfer("transfer-17")),
        ),
        Conversion(
            failure = GroupFailure.EmptyName,
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.EmptyName),
        ),
        Conversion(
            failure = GroupFailure.MemberInUse(listOf(ExpenseId("expenses-18")), MemberId("member-19"), listOf(TransferId("transfers-20"))),
            generated =
                eranos.internal.EranosException.Group(
                    "detail",
                    eranos.internal.GroupFailure.MemberInUse(listOf("expenses-18"), "member-19", listOf("transfers-20")),
                ),
        ),
        Conversion(
            failure = GroupFailure.MissingExchangeRate(Currency.known("RON")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.MissingExchangeRate("RON")),
        ),
        Conversion(
            failure = GroupFailure.Overflow,
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.Overflow),
        ),
        Conversion(
            failure = GroupFailure.UnexpectedExchangeRate,
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnexpectedExchangeRate),
        ),
        Conversion(
            failure = GroupFailure.UnknownCategory(CategoryId("category-22")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownCategory("category-22")),
        ),
        Conversion(
            failure = GroupFailure.UnknownExpense(ExpenseId("expense-23")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownExpense("expense-23")),
        ),
        Conversion(
            failure = GroupFailure.UnknownMember(MemberId("member-24")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownMember("member-24")),
        ),
        Conversion(
            failure = GroupFailure.UnknownTransfer(TransferId("transfer-25")),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownTransfer("transfer-25")),
        ),
        Conversion(
            failure = GroupFailure.UnsupportedFormat(26),
            generated = eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnsupportedFormat(26)),
        ),
        Conversion(failure = IdFailure.InvalidId, generated = eranos.internal.EranosException.Id("detail", eranos.internal.IdFailure.INVALID_ID)),
        Conversion(
            failure = LanguageFailure.InvalidLanguage,
            generated = eranos.internal.EranosException.Language("detail", eranos.internal.LanguageFailure.INVALID_LANGUAGE),
        ),
        Conversion(
            failure = MemberFailure.EmptyFirstName,
            generated = eranos.internal.EranosException.Member("detail", eranos.internal.MemberFailure.EMPTY_FIRST_NAME),
        ),
        Conversion(
            failure = MoneyFailure.CurrencyMismatch(Currency.known("RON"), Currency.known("USD")),
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.CurrencyMismatch("RON", "USD")),
        ),
        Conversion(
            failure = MoneyFailure.InvalidAmount,
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidAmount),
        ),
        Conversion(
            failure = MoneyFailure.InvalidExchangeRate,
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidExchangeRate),
        ),
        Conversion(
            failure = MoneyFailure.InvalidParts,
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidParts),
        ),
        Conversion(
            failure = MoneyFailure.InvalidPercentage,
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidPercentage),
        ),
        Conversion(
            failure = MoneyFailure.Overflow,
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.Overflow),
        ),
        Conversion(
            failure = MoneyFailure.TooManyDecimals(29, Currency.known("RON")),
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.TooManyDecimals(29, "RON")),
        ),
        Conversion(
            failure = MoneyFailure.UnknownCurrency,
            generated = eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.UnknownCurrency),
        ),
        Conversion(
            failure = SettlementFailure.OutOfMemory(31),
            generated = eranos.internal.EranosException.Settlement("detail", eranos.internal.SettlementFailure.OutOfMemory(31)),
        ),
        Conversion(
            failure = TransferFailure.NonPositiveAmount,
            generated = eranos.internal.EranosException.Transfer("detail", eranos.internal.TransferFailure.NON_POSITIVE_AMOUNT),
        ),
        Conversion(
            failure = TransferFailure.SameMember,
            generated = eranos.internal.EranosException.Transfer("detail", eranos.internal.TransferFailure.SAME_MEMBER),
        ),
        Conversion(
            failure = AccountFailure.InvalidIban(IbanReason.InvalidCharacter),
            generated =
                eranos.internal.EranosException.Account(
                    "detail",
                    eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidCharacter),
                ),
        ),
        Conversion(
            failure = AccountFailure.InvalidIban(IbanReason.InvalidChecksum),
            generated =
                eranos.internal.EranosException.Account(
                    "detail",
                    eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidChecksum),
                ),
        ),
        Conversion(
            failure = AccountFailure.InvalidIban(IbanReason.InvalidFormat),
            generated =
                eranos.internal.EranosException.Account(
                    "detail",
                    eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidFormat),
                ),
        ),
        Conversion(
            failure = AccountFailure.InvalidIban(IbanReason.InvalidStructure),
            generated =
                eranos.internal.EranosException.Account(
                    "detail",
                    eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidStructure),
                ),
        ),
        Conversion(
            failure = AccountFailure.InvalidIban(IbanReason.UnknownCountry),
            generated =
                eranos.internal.EranosException.Account(
                    "detail",
                    eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.UnknownCountry),
                ),
        ),
        Conversion(
            failure = AccountFailure.InvalidIban(IbanReason.WrongLength(24, 22)),
            generated =
                eranos.internal.EranosException.Account(
                    "detail",
                    eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.WrongLength(24, 22)),
                ),
        ),
    )
