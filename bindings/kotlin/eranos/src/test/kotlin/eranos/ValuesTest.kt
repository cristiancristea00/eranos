// The value types. Every text they show comes from the library, every value an app builds is checked by the library's rules,
// and a value the library never sends still prints something: a type that formatted on its own, let a refused value through,
// or threw on an impossible one would show an app another value than the library holds.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
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
        val euro = Currency.of("EUR")
        assertEquals("EUR", euro.code)
        assertEquals("Euro", euro.name)
        assertEquals("EUR", euro.toString())
        assertEquals(978, euro.numericCode)
        assertEquals(2, euro.decimalPlaces)
        assertTrue(euro in Currency.all)
        val lei = Currency.of("RON")
        assertValue(euro, Currency.known("EUR"), lei)
        val unknown = assertThrows<EranosException.Money> { Currency.of("XYZ") }
        assertEquals(MoneyFailure.UnknownCurrency, unknown.failure)
        assertEquals("unknown currency code", unknown.message)

        val price = Money.parse("12.5", euro)
        assertValue(price, Money(euro, 1_250), Money(euro, 1_251))
        assertEquals("12.50 EUR", price.toString())
        assertEquals(Money(euro, 1_300), price + Money(euro, 50))
        assertEquals(Money(euro, 1_200), price - Money(euro, 50))
        val mismatch = assertThrows<EranosException.Money> { price + Money(lei, 1) }
        assertEquals(MoneyFailure.CurrencyMismatch(euro, lei), mismatch.failure)
        assertThrows<EranosException.Money> { Money(euro, Long.MAX_VALUE) + Money(euro, 1) }
        assertThrows<EranosException.Money> { Money(euro, Long.MIN_VALUE) - Money(euro, 1) }
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
        val lei = Currency.of("RON")
        val euro = Currency.of("EUR")
        assertEquals(Money(euro, 1), ExchangeRate.parse("0.25").convert(Money(lei, 2), euro))
        assertThrows<EranosException.Money> { ExchangeRate.parse("0") }
        assertThrows<EranosException.Money> { ExchangeRate.parse("2").convert(Money(lei, Long.MAX_VALUE), euro) }

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
        val date = CalendarDate.of(23, 9, 2026)
        assertEquals("2026-09-23", date.toString())
        assertEquals(23, date.day)
        assertEquals(9, date.month)
        assertEquals(2026, date.year)
        assertValue(date, CalendarDate.parse("2026-09-23"), CalendarDate.of(24, 9, 2026))
        val invalid = assertThrows<EranosException.Calendar> { CalendarDate.of(30, 2, 2026) }
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
        val refused = assertThrows<EranosException.Account> { Iban.parse("RO49AAAA1B31007593840001") }
        assertEquals(AccountFailure.InvalidIban(IbanReason.InvalidChecksum), refused.failure)
        assertEquals("invalid IBAN: the check digits do not match", refused.message)

        val username = Username.parse(" ana.pays ")
        assertEquals("ana.pays", username.toString())
        assertValue(username, Username.parse("ana.pays"), Username.parse("bob.pays"))
        assertThrows<EranosException.Account> { Username.parse(" ") }

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
        assertEquals(text, GroupId(text).value)
        assertEquals(text, GroupId(text).toString())
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
     * A value the library never sends, such as an amount in a currency it does not list, would throw from its text instead of
     * printing.
     */
    @Test
    fun aValueTheLibraryNeverSendsStillPrints() {
        val unlisted = Currency.known("XYZ")
        assertEquals("XYZ", unlisted.code)
        assertEquals("XYZ", unlisted.name)
        assertEquals(0, unlisted.decimalPlaces)
        assertEquals(0, unlisted.numericCode)
        assertEquals("5 XYZ", Money(unlisted, 5).toString())
        assertEquals("-1 bp", Percentage(-1).toString())
        assertEquals("0/100", Parts(0).toString())
        assertEquals("2026/2/30", CalendarDate(30, 2, 2026).toString())
        assertEquals("25h0", TimeOfDay(25, 0).toString())
        assertEquals("0712", PhoneNumber("0712").toString())
        assertEquals("XY", Iban("XY").countryCode)
        assertTrue("XYZ" in Balance.Owes(Money(unlisted, 1)).toString())
    }
}
