// The value types. Every text they show comes from the library, every value an app builds is checked by the library's rules,
// and a value the library never sends still prints something: a type that formatted on its own, let a refused value through,
// or threw on an impossible one would show an app another value than the library holds.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/** The currencies, amounts, ratios, dates, times, contact details, and identifiers. */
class ValuesTest {
    /**
     * A currency, an amount, or a check that read or wrote another way than the library's would show an app another amount
     * than the group holds.
     */
    @Test
    fun currenciesAndAmountsReadAndWriteAsTheLibraryDoes() {
        val euro = CurrencyCode.of("EUR")
        assertEquals("EUR", euro.code)
        assertEquals("Euro", euro.name)
        assertEquals("EUR", euro.toString())
        assertEquals(978, euro.numericCode)
        assertEquals(2, euro.decimalPlaces)
        assertTrue(euro in CurrencyCode.all)
        val lei = CurrencyCode.of("RON")
        assertValue(euro, CurrencyCode.known("EUR"), lei)
        val unknown = assertThrows<EranosException.Money> { CurrencyCode.of("XYZ") }
        assertEquals(MoneyFailure.UnknownCurrency, unknown.failure)
        assertEquals("unknown currency code", unknown.message)

        val price = Money.parse("12.5", euro)
        assertValue(price, Money(1_250, euro), Money(1_251, euro))
        assertEquals("12.50 EUR", price.toString())
        assertEquals(Money(1_300, euro), price + Money(50, euro))
        assertEquals(Money(1_200, euro), price - Money(50, euro))
        val mismatch = assertThrows<EranosException.Money> { price + Money(1, lei) }
        assertEquals(MoneyFailure.CurrencyMismatch(euro, lei), mismatch.failure)
        assertThrows<EranosException.Money> { Money(Long.MAX_VALUE, euro) + Money(1, euro) }
        assertThrows<EranosException.Money> { Money(Long.MIN_VALUE, euro) - Money(1, euro) }
        assertThrows<EranosException.Money> { Money.parse("twelve", euro) }
    }

    /**
     * A rate, a percentage, or a number of parts read or written another way than the library's would divide an expense
     * another way.
     */
    @Test
    fun ratesPercentagesAndPartsReadAndWriteAsTheLibraryDoes() {
        val rate = ExchangeRate.parse("4.970")
        assertEquals("4.97", rate.toString())
        assertValue(rate, ExchangeRate.parse("4.97"), ExchangeRate.parse("5"))
        val lei = CurrencyCode.of("RON")
        val euro = CurrencyCode.of("EUR")
        assertEquals(Money(1, euro), ExchangeRate.parse("0.25").convert(Money(2, lei), euro))
        assertThrows<EranosException.Money> { ExchangeRate.parse("0") }
        assertThrows<EranosException.Money> { ExchangeRate.parse("2").convert(Money(Long.MAX_VALUE, lei), euro) }

        val percentage = Percentage.parse("12.5 %")
        assertEquals(1_250, percentage.basisPoints)
        assertEquals("12.5%", percentage.toString())
        assertValue(percentage, Percentage.ofBasisPoints(1_250), Percentage.ofBasisPoints(1))
        assertThrows<EranosException.Money> { Percentage.ofBasisPoints(-1) }
        assertThrows<EranosException.Money> { Percentage.parse("half") }

        val parts = Parts.parse("0.5")
        assertEquals(50, parts.hundredths)
        assertEquals("0.5", parts.toString())
        assertValue(parts, Parts.ofHundredths(50), Parts.ofHundredths(1))
        assertThrows<EranosException.Money> { Parts.ofHundredths(0) }
        assertThrows<EranosException.Money> { Parts.parse("half") }
    }

    /**
     * A date or a time read or written another way than the library's, or one that does not exist, would put an entry on
     * another day.
     */
    @Test
    fun datesAndTimesReadAndWriteAsTheLibraryDoes() {
        val date = CalendarDate.of(2026, 9, 23)
        assertEquals("2026-09-23", date.toString())
        assertEquals(2026, date.year)
        assertEquals(9, date.month)
        assertEquals(23, date.day)
        assertValue(date, CalendarDate.parse("2026-09-23"), CalendarDate.of(2026, 9, 24))
        val invalid = assertThrows<EranosException.Calendar> { CalendarDate.of(2026, 2, 30) }
        assertEquals(CalendarFailure.InvalidDate, invalid.failure)
        assertEquals("invalid date: expected an existing ISO 8601 date between years 1 and 9999", invalid.message)
        assertThrows<EranosException.Calendar> { CalendarDate.parse("2026-02-30") }

        val time = TimeOfDay.of(18, 30)
        assertEquals("18:30", time.toString())
        assertEquals(18, time.hour)
        assertEquals(30, time.minute)
        assertValue(time, TimeOfDay.parse("18:30"), TimeOfDay.of(18, 31))
        assertThrows<EranosException.Calendar> { TimeOfDay.of(24, 0) }
        assertThrows<EranosException.Calendar> { TimeOfDay.parse("25:00") }
    }

    /**
     * Contact and account details kept in another form than the library's, or refused ones let through, would reach the
     * group unchecked.
     */
    @Test
    fun contactAndAccountDetailsReadAsTheLibraryDoes() {
        val email = Email.parse(" ana@example.com ")
        assertEquals("ana@example.com", email.toString())
        assertValue(email, Email.parse("ana@example.com"), Email.parse("bob@example.com"))
        assertThrows<EranosException.Contact> { Email.parse("ana") }

        val phone = PhoneNumber.parse("+40 712 345 678")
        assertEquals("+40712345678", phone.e164)
        assertEquals("+40 712 345 678", phone.toString())
        assertValue(phone, PhoneNumber.parse("+40712345678"), PhoneNumber.parse("+40712345679"))
        assertThrows<EranosException.Contact> { PhoneNumber.parse("0712345678") }

        val iban = Iban.parse("ro49 aaaa 1b31 0075 9384 0000")
        assertEquals("RO49AAAA1B31007593840000", iban.toString())
        assertEquals("RO", iban.countryCode)
        assertValue(iban, Iban.parse("RO49AAAA1B31007593840000"), Iban.parse("GB82WEST12345698765432"))
        val refused = assertThrows<EranosException.PaymentAccount> { Iban.parse("RO49AAAA1B31007593840001") }
        assertEquals(PaymentAccountFailure.InvalidIban(IbanReason.InvalidChecksum), refused.failure)
        assertEquals("invalid IBAN: the check digits do not match", refused.message)

        val username = Username.parse(" ana.pays ")
        assertEquals("ana.pays", username.toString())
        assertValue(username, Username.parse("ana.pays"), Username.parse("bob.pays"))
        assertThrows<EranosException.PaymentAccount> { Username.parse(" ") }

        val language = Language.parse("pt-pt")
        assertEquals("pt-PT", language.toString())
        assertValue(language, Language.parse("pt-PT"), Language.parse("ro-RO"))
        val tag = assertThrows<EranosException.Language> { Language.parse("not a tag") }
        assertEquals(LanguageFailure.InvalidLanguage, tag.failure)
        assertEquals("invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`", tag.message)
    }

    /**
     * Identifiers of different entities that compared, or printed, as each other's text would let an app store one and read
     * back another.
     */
    @Test
    fun anIdentifierIsItsText() {
        val text = "0192f1c3-0000-7000-8000-000000000001"
        assertEquals(text, ExpenseGroupId(text).value)
        assertEquals(text, ExpenseGroupId(text).toString())
        assertEquals(text, MemberId(text).value)
        assertEquals(text, MemberId(text).toString())
        assertEquals(text, CategoryId(text).value)
        assertEquals(text, CategoryId(text).toString())
        assertEquals(text, ExpenseId(text).value)
        assertEquals(text, ExpenseId(text).toString())
        assertEquals(text, TransferId(text).value)
        assertEquals(text, TransferId(text).toString())
        assertValue(MemberId(text), MemberId(text), MemberId("other"))
    }

    /**
     * A group that checked an identifier by other rules than the library's, counted its length in characters rather than
     * bytes, or refused it with another failure or message would let an app keep text no group can hold, or tell it to fix a
     * rule the text keeps.
     */
    @Test
    fun anIdentifierIsAnyTextThatKeepsTheLibrarysRules() {
        val group = Flat().group
        for (text in listOf("nobody", "Ana Lúcia", "a".repeat(64), "é".repeat(32))) {
            assertNull(group.memberBlocking(MemberId(text)))
        }
        val empty = "an identifier must not be empty"
        val tooLong = "an identifier can be at most 64 bytes long"
        val controlCharacter = "an identifier cannot contain control characters"
        val surroundingWhitespace = "an identifier cannot start or end with whitespace"
        val refusals =
            listOf(
                IdentifierRefusal(text = "", failure = IdFailure.Empty, message = empty),
                IdentifierRefusal(text = "a".repeat(65), failure = IdFailure.TooLong, message = tooLong),
                IdentifierRefusal(text = "é".repeat(33), failure = IdFailure.TooLong, message = tooLong),
                IdentifierRefusal(text = "a\tb", failure = IdFailure.ControlCharacter, message = controlCharacter),
                IdentifierRefusal(text = "\u0007 ", failure = IdFailure.ControlCharacter, message = controlCharacter),
                IdentifierRefusal(text = " ana", failure = IdFailure.SurroundingWhitespace, message = surroundingWhitespace),
                IdentifierRefusal(text = "ana\u00a0", failure = IdFailure.SurroundingWhitespace, message = surroundingWhitespace),
            )
        for (refusal in refusals) {
            val refused = assertThrows<EranosException.Id> { group.memberBlocking(MemberId(refusal.text)) }
            assertEquals(refusal.failure, refused.failure)
            assertEquals(refusal.message, refused.message)
        }
    }

    /**
     * A value the library never sends, such as an amount in a currency it does not list, would throw from its text instead of
     * printing.
     */
    @Test
    fun aValueTheLibraryNeverSendsStillPrints() {
        val unlisted = CurrencyCode.known("XYZ")
        assertEquals("XYZ", unlisted.code)
        assertEquals("XYZ", unlisted.name)
        assertEquals(0, unlisted.numericCode)
        assertEquals(0, unlisted.decimalPlaces)
        assertEquals("5 XYZ", Money(5, unlisted).toString())
        assertEquals("-1 bp", Percentage(-1).toString())
        assertEquals("0/100", Parts(0).toString())
        assertEquals("2026/2/30", CalendarDate(2026, 2, 30).toString())
        assertEquals("25h0", TimeOfDay(25, 0).toString())
        assertEquals("0712", PhoneNumber("0712").toString())
        assertEquals("XY", Iban("XY").countryCode)
        assertTrue("XYZ" in Balance.Owes(Money(1, unlisted)).toString())
    }
}

/** Text the library refuses as an identifier, with the failure and the message it reports. */
private data class IdentifierRefusal(
    val text: String,
    val failure: IdFailure,
    val message: String,
)
