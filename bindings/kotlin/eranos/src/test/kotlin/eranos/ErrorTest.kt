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
        for ((generated, failure) in conversions) {
            val converted = assertInstanceOf(EranosException::class.java, generated.converted())
            assertEquals(failure, converted.failure())
            assertEquals(failure.hashCode(), converted.failure().hashCode())
            assertEquals(failure.toString(), converted.failure().toString())
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
        assertEquals(listOf(2L, 1L), listOf(json.column, json.line))
        assertEquals("detail", json.message)
        for ((generated, expected) in standard) {
            val converted = generated.converted()
            assertEquals(expected, converted::class.java)
            assertEquals("detail", converted.message)
        }
    }
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
        is EranosException.InvalidJson -> listOf(column, line)
    }

/** [minorUnits] cents. */
private fun euros(minorUnits: Long): Money = Money(Currency.known("EUR"), minorUnits)

/** Every failure the generated code can report, with every reason of a refused IBAN, beside the failure it becomes. */
private val conversions: List<Pair<eranos.internal.EranosException, Any>> =
    listOf(
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.EmptyName) to AccountFailure.EmptyName,
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.EmptyUsername) to AccountFailure.EmptyUsername,
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidChecksum)) to
            AccountFailure.InvalidIban(IbanReason.InvalidChecksum),
        eranos.internal.EranosException.Calendar("detail", eranos.internal.CalendarFailure.INVALID_DATE) to CalendarFailure.InvalidDate,
        eranos.internal.EranosException.Calendar("detail", eranos.internal.CalendarFailure.INVALID_TIME) to CalendarFailure.InvalidTime,
        eranos.internal.EranosException.Category("detail", eranos.internal.CategoryFailure.EMPTY_NAME) to CategoryFailure.EmptyName,
        eranos.internal.EranosException.Contact("detail", eranos.internal.ContactFailure.INVALID_EMAIL) to ContactFailure.InvalidEmail,
        eranos.internal.EranosException.Contact("detail", eranos.internal.ContactFailure.INVALID_PHONE_NUMBER) to ContactFailure.InvalidPhoneNumber,
        eranos.internal.EranosException.Expense(
            "detail",
            eranos.internal.ExpenseFailure.AmountsDoNotAddUp(eranos.internal.Money("EUR", 2), eranos.internal.Money("EUR", 3)),
        ) to
            ExpenseFailure.AmountsDoNotAddUp(euros(2), euros(3)),
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.CurrencyMismatch("USD", "GBP")) to
            ExpenseFailure.CurrencyMismatch(Currency.known("USD"), Currency.known("GBP")),
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.DuplicateMember("member-6")) to
            ExpenseFailure.DuplicateMember(MemberId("member-6")),
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.EmptyItemName) to ExpenseFailure.EmptyItemName,
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.EmptyName) to ExpenseFailure.EmptyName,
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.EmptySplit) to ExpenseFailure.EmptySplit,
        eranos.internal.EranosException.Expense(
            "detail",
            eranos.internal.ExpenseFailure.ItemsDoNotAddUp(eranos.internal.Money("EUR", 7), eranos.internal.Money("EUR", 8)),
        ) to
            ExpenseFailure.ItemsDoNotAddUp(euros(7), euros(8)),
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NegativeTip) to ExpenseFailure.NegativeTip,
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NoItems) to ExpenseFailure.NoItems,
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NonPositiveAmount) to ExpenseFailure.NonPositiveAmount,
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.NonPositiveShare("member-9")) to
            ExpenseFailure.NonPositiveShare(MemberId("member-9")),
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.Overflow) to ExpenseFailure.Overflow,
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.PercentagesDoNotAddUp(10)) to
            ExpenseFailure.PercentagesDoNotAddUp(10),
        eranos.internal.EranosException.Expense("detail", eranos.internal.ExpenseFailure.ZeroQuantity) to ExpenseFailure.ZeroQuantity,
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.ArchivedMember("member-11")) to
            GroupFailure.ArchivedMember(MemberId("member-11")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.BuiltInCategory) to GroupFailure.BuiltInCategory,
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.Busy) to GroupFailure.Busy,
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateCategory("category-12")) to
            GroupFailure.DuplicateCategory(CategoryId("category-12")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateCategoryName("existing-13")) to
            GroupFailure.DuplicateCategoryName(CategoryId("existing-13")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateExchangeRate("GBP")) to
            GroupFailure.DuplicateExchangeRate(Currency.known("GBP")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateExpense("expense-15")) to
            GroupFailure.DuplicateExpense(ExpenseId("expense-15")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateMember("member-16")) to
            GroupFailure.DuplicateMember(MemberId("member-16")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.DuplicateTransfer("transfer-17")) to
            GroupFailure.DuplicateTransfer(TransferId("transfer-17")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.EmptyName) to GroupFailure.EmptyName,
        eranos.internal.EranosException.Group(
            "detail",
            eranos.internal.GroupFailure.MemberInUse(listOf("expenses-18"), "member-19", listOf("transfers-20")),
        ) to
            GroupFailure.MemberInUse(listOf(ExpenseId("expenses-18")), MemberId("member-19"), listOf(TransferId("transfers-20"))),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.MissingExchangeRate("RON")) to
            GroupFailure.MissingExchangeRate(Currency.known("RON")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.Overflow) to GroupFailure.Overflow,
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnexpectedExchangeRate) to GroupFailure.UnexpectedExchangeRate,
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownCategory("category-22")) to
            GroupFailure.UnknownCategory(CategoryId("category-22")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownExpense("expense-23")) to
            GroupFailure.UnknownExpense(ExpenseId("expense-23")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownMember("member-24")) to
            GroupFailure.UnknownMember(MemberId("member-24")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnknownTransfer("transfer-25")) to
            GroupFailure.UnknownTransfer(TransferId("transfer-25")),
        eranos.internal.EranosException.Group("detail", eranos.internal.GroupFailure.UnsupportedFormat(26)) to GroupFailure.UnsupportedFormat(26),
        eranos.internal.EranosException.Id("detail", eranos.internal.IdFailure.INVALID_ID) to IdFailure.InvalidId,
        eranos.internal.EranosException.Language("detail", eranos.internal.LanguageFailure.INVALID_LANGUAGE) to LanguageFailure.InvalidLanguage,
        eranos.internal.EranosException.Member("detail", eranos.internal.MemberFailure.EMPTY_FIRST_NAME) to MemberFailure.EmptyFirstName,
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.CurrencyMismatch("RON", "USD")) to
            MoneyFailure.CurrencyMismatch(Currency.known("RON"), Currency.known("USD")),
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidAmount) to MoneyFailure.InvalidAmount,
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidExchangeRate) to MoneyFailure.InvalidExchangeRate,
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidParts) to MoneyFailure.InvalidParts,
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.InvalidPercentage) to MoneyFailure.InvalidPercentage,
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.Overflow) to MoneyFailure.Overflow,
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.TooManyDecimals(29, "RON")) to
            MoneyFailure.TooManyDecimals(29, Currency.known("RON")),
        eranos.internal.EranosException.Money("detail", eranos.internal.MoneyFailure.UnknownCurrency) to MoneyFailure.UnknownCurrency,
        eranos.internal.EranosException.Settlement("detail", eranos.internal.SettlementFailure.OutOfMemory(31)) to SettlementFailure.OutOfMemory(31),
        eranos.internal.EranosException.Transfer("detail", eranos.internal.TransferFailure.NON_POSITIVE_AMOUNT) to TransferFailure.NonPositiveAmount,
        eranos.internal.EranosException.Transfer("detail", eranos.internal.TransferFailure.SAME_MEMBER) to TransferFailure.SameMember,
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidCharacter)) to
            AccountFailure.InvalidIban(IbanReason.InvalidCharacter),
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidChecksum)) to
            AccountFailure.InvalidIban(IbanReason.InvalidChecksum),
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidFormat)) to
            AccountFailure.InvalidIban(IbanReason.InvalidFormat),
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.InvalidStructure)) to
            AccountFailure.InvalidIban(IbanReason.InvalidStructure),
        eranos.internal.EranosException.Account("detail", eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.UnknownCountry)) to
            AccountFailure.InvalidIban(IbanReason.UnknownCountry),
        eranos.internal.EranosException.Account(
            "detail",
            eranos.internal.AccountFailure.InvalidIban(eranos.internal.IbanReason.WrongLength(24, 22)),
        ) to
            AccountFailure.InvalidIban(IbanReason.WrongLength(24, 22)),
    )

/** The failures the bindings report of their own, beside the standard exception each becomes. */
private val standard: List<Pair<eranos.internal.EranosException, Class<out Exception>>> =
    listOf(
        eranos.internal.EranosException.Cancelled("detail") to CancellationException::class.java,
        eranos.internal.EranosException.IdentifierGiven("detail") to IllegalArgumentException::class.java,
        eranos.internal.EranosException.IdentifierMissing("detail") to IllegalArgumentException::class.java,
        eranos.internal.EranosException.Released("detail") to IllegalStateException::class.java,
    )
