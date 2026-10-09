// The value types. Every text they show comes from the library, every value an app builds is checked by the library's rules,
// and a phone number or an IBAN keeps the text the library derives from it: a type that formatted on its own, let a refused
// value through, or derived text of its own would show an app another value than the library holds.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/** The currencies, amounts, ratios, quantities, dates, times, contact details, and identifiers. */
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
        assertValue(euro, CurrencyCode.all.first { "EUR" == it.code }, lei)
        val unknown = assertThrows<EranosException.Money> { CurrencyCode.of("XYZ") }
        assertEquals(MoneyFailure.UnknownCurrency, unknown.failure)
        assertEquals("unknown currency code", unknown.message)

        val price = Money.parse("12.5", euro)
        assertValue(price, Money(1_250, euro), Money(1_251, euro))
        assertEquals("12.50 EUR", price.toString())
        assertEquals(Money(1_300, euro), price + Money(50, euro))
        assertEquals(Money(1_200, euro), price - Money(50, euro))
        val zero = Money.zero(euro)
        assertEquals(Money(0, euro), zero)
        assertEquals(listOf(true, false, false), listOf(zero.isZero, zero.isPositive, zero.isNegative))
        assertEquals(listOf(false, true, false), listOf(price.isZero, price.isPositive, price.isNegative))
        val refund = zero - price
        assertEquals(listOf(false, false, true), listOf(refund.isZero, refund.isPositive, refund.isNegative))
        val mismatch = assertThrows<EranosException.Money> { price + Money(1, lei) }
        assertEquals(MoneyFailure.CurrencyMismatch(euro, lei), mismatch.failure)
        assertThrows<EranosException.Money> { Money(Long.MAX_VALUE, euro) + Money(1, euro) }
        assertThrows<EranosException.Money> { Money(Long.MIN_VALUE, euro) - Money(1, euro) }
        assertThrows<EranosException.Money> { Money.parse("twelve", euro) }
    }

    /**
     * An amount or a balance printed with other decimal places than its currency's would show an app another amount than the
     * group holds.
     */
    @Test
    fun anAmountPrintsWithTheDecimalPlacesOfItsCurrency() {
        val yen = Money(12_345, CurrencyCode.of("JPY"))
        assertEquals("12345 JPY", yen.toString())
        assertEquals("owes 12345 JPY", Balance.Owes(yen).toString())
        assertEquals("gets back 12345 JPY", Balance.GetsBack(yen).toString())
    }

    /**
     * An amount that looked its currency up by its code, rather than keeping the details the library gave, would show an app
     * another currency than the library's, or none for a code its list lacks.
     */
    @Test
    fun anAmountKeepsTheCurrencyDetailsTheLibraryGives() {
        val details = eranos.internal.CurrencyDetails("ZZZ", "Unlisted", 999, 7u)
        val amount = eranos.internal.Money(-5, details).converted()
        assertEquals(-5L, amount.minorUnits)
        assertEquals("ZZZ", amount.currency.code)
        assertEquals("Unlisted", amount.currency.name)
        assertEquals(999, amount.currency.numericCode)
        assertEquals(7, amount.currency.decimalPlaces)
        assertEquals(details, amount.generated.currency)
        assertEquals("-0.0000005 ZZZ", amount.toString())
    }

    /**
     * An amount in a currency whose details are not the library's for its code would be divided by another number of
     * decimal places than the library holds, so the library refuses it. An app cannot build such a currency, whose
     * constructor is internal; a test of the package can.
     */
    @Test
    fun anAmountInCurrencyDetailsOtherThanTheLibrarysIsRefused() {
        val flat = Flat()
        val euro = flat.euro
        val forgeries =
            listOf(
                CurrencyCode("eur", euro.name, euro.numericCode, euro.decimalPlaces),
                CurrencyCode(euro.code, "Euro dollar", euro.numericCode, euro.decimalPlaces),
                CurrencyCode(euro.code, euro.name, 979, euro.decimalPlaces),
                CurrencyCode(euro.code, euro.name, euro.numericCode, 3),
            )
        for (forgery in forgeries) {
            val amount = Money(1_250, forgery)
            val sum = assertThrows<EranosException.MismatchedRecord> { amount + flat.euros(1) }
            assertEquals(MismatchedField.CURRENCY_DETAILS, sum.field)
            val taxi =
                Expense("Taxi", Cost.Local(amount), Split.Equal(listOf(flat.ana)), ExpenseSplit.Whole(Split.Equal(listOf(flat.bob))), flat.day)
            val added = assertThrows<EranosException.MismatchedRecord> { flat.group.addBlocking(taxi) }
            assertEquals(MismatchedField.CURRENCY_DETAILS, added.field)
        }
        assertEquals("1.250 EUR", Money(1_250, forgeries.last()).toString())
    }

    /**
     * A rate, a percentage, or a number of parts read or written another way than the library's, or one the library refuses
     * let through, would divide an expense another way.
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
        assertEquals("21474836.47%", Percentage.ofBasisPoints(Int.MAX_VALUE).toString())
        val negative = assertThrows<EranosException.Money> { Percentage.ofBasisPoints(Int.MIN_VALUE) }
        assertEquals(MoneyFailure.InvalidPercentage, negative.failure)
        assertThrows<EranosException.Money> { Percentage.parse("half") }

        val parts = Parts.parse("0.5")
        assertEquals(50, parts.hundredths)
        assertEquals("0.5", parts.toString())
        assertValue(parts, Parts.ofHundredths(50), Parts.ofHundredths(1))
        assertEquals("21474836.47", Parts.ofHundredths(Int.MAX_VALUE).toString())
        val none = assertThrows<EranosException.Money> { Parts.ofHundredths(0) }
        assertEquals(MoneyFailure.InvalidParts, none.failure)
        assertThrows<EranosException.Money> { Parts.parse("half") }
    }

    /**
     * A quantity read or written another way than the library's, or one the library refuses let through, would multiply a
     * line by another number of units than the group does.
     */
    @Test
    fun quantitiesReadAndWriteAsTheLibraryDoes() {
        val weight = Quantity.parse("0.452")
        assertEquals(4_520L, weight.tenThousandths)
        assertEquals("0.452", weight.toString())
        assertValue(weight, Quantity.ofTenThousandths(4_520), Quantity.ofTenThousandths(4_521))
        assertEquals(Quantity.ofTenThousandths(30_000), Quantity.ofWhole(3))
        assertEquals("3", Quantity.ofWhole(3).toString())
        assertEquals(Quantity.ofWhole(1), Quantity.ONE)
        assertEquals("1", Quantity.ONE.toString())
        assertEquals("922337203685477.5807", Quantity.ofTenThousandths(Long.MAX_VALUE).toString())
        val refusals: List<() -> Quantity> =
            listOf(
                { Quantity.ofTenThousandths(0) },
                { Quantity.ofWhole(Long.MAX_VALUE / 10_000 + 1) },
                { Quantity.parse("half") },
            )
        for (refusal in refusals) {
            val refused = assertThrows<EranosException.Money> { refusal() }
            assertEquals(MoneyFailure.InvalidQuantity, refused.failure)
            assertEquals("invalid quantity: expected a positive number with at most 4 decimal places", refused.message)
        }
    }

    /**
     * An app that holds amounts, percentages, parts, quantities, or rates as doubles would get other cents or digits in Kotlin
     * than the library rounds to, or a value back from a number too large for the bindings' integers.
     */
    @Test
    fun amountsRatiosQuantitiesAndRatesConvertFromDoublesAsTheLibraryDoes() {
        val euro = CurrencyCode.of("EUR")
        // The double nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
        assertEquals(Money(101, euro), Money.fromDouble(1.005, euro))
        assertEquals(1_235, Percentage.fromDouble(12.345).basisPoints)
        assertEquals(101, Parts.fromDouble(1.005).hundredths)
        assertEquals(4_521L, Quantity.fromDouble(0.45205).tenThousandths)
        assertEquals("0.30000000000000004", ExchangeRate.fromDouble(0.1 + 0.2).toString())

        val overflow = assertThrows<EranosException.Money> { Money.fromDouble(1e21, euro) }
        assertEquals(MoneyFailure.Overflow, overflow.failure)
        // Above 21 474 836.47, the basis points and the hundredths no longer fit an `Int`.
        val percentage = assertThrows<EranosException.Money> { Percentage.fromDouble(30_000_000.0) }
        assertEquals(MoneyFailure.InvalidPercentage, percentage.failure)
        val parts = assertThrows<EranosException.Money> { Parts.fromDouble(30_000_000.0) }
        assertEquals(MoneyFailure.InvalidParts, parts.failure)
        // Above 922 337 203 685 477.5807, the ten-thousandths no longer fit a `Long`.
        val quantity = assertThrows<EranosException.Money> { Quantity.fromDouble(1e21) }
        assertEquals(MoneyFailure.InvalidQuantity, quantity.failure)
        val zero = assertThrows<EranosException.Money> { ExchangeRate.fromDouble(0.0) }
        assertEquals(MoneyFailure.InvalidExchangeRate, zero.failure)
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
        val midnight = assertThrows<EranosException.Calendar> { TimeOfDay.of(24, 0) }
        assertEquals(CalendarFailure.InvalidTime, midnight.failure)
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
        assertEquals("+40 712 345 678", phone.international)
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
     * A phone number or an IBAN that wrote its international form or its country code itself, or asked the library for it
     * again, could show an app other text than the library gave, and one that crossed back without it would leave the
     * library nothing to compare. These records hold text the library never writes, which only a value that keeps the
     * record's text shows.
     */
    @Test
    fun aPhoneNumberAndAnIbanKeepTheTextTheLibraryDerives() {
        val record = eranos.internal.PhoneNumber("+40712345678", "+40 7123 45678")
        val phone = record.converted()
        assertEquals("+40712345678", phone.e164)
        assertEquals("+40 7123 45678", phone.international)
        assertEquals("+40 7123 45678", phone.toString())
        assertEquals(record, phone.generated)

        val iban = eranos.internal.Iban("RO49AAAA1B31007593840000", "ZZ").converted()
        assertEquals("RO49AAAA1B31007593840000", iban.toString())
        assertEquals("ZZ", iban.countryCode)
        val details = eranos.internal.PaymentAccountDetails.Iban("RO49AAAA1B31007593840000", "ZZ")
        assertEquals(PaymentAccountDetails.Iban(iban), details.converted())
        assertEquals(details, details.converted().generated)
    }

    /**
     * A phone number whose international form, or an IBAN whose country code, is not the one the library derives from it
     * would show an app text the library never wrote for it, so the library refuses it. An app cannot build such a value,
     * whose constructor is internal; a test of the package can.
     */
    @Test
    fun aPhoneNumberOrAnIbanWithDerivedTextOtherThanTheLibrarysIsRefused() {
        val flat = Flat()
        val phone = PhoneNumber("+40712345678", "+40712345678")
        val added = assertThrows<EranosException.MismatchedRecord> { flat.group.addBlocking(Member("Carol", phone = phone)) }
        assertEquals(MismatchedField.PHONE_INTERNATIONAL, added.field)
        val ana = present(flat.group.memberBlocking(flat.ana))
        val replaced = assertThrows<EranosException.MismatchedRecord> { flat.group.replaceBlocking(ana.copy(phone = phone)) }
        assertEquals(MismatchedField.PHONE_INTERNATIONAL, replaced.field)

        val account = PaymentAccount("Main account", PaymentAccountDetails.Iban(Iban("RO49AAAA1B31007593840000", "GB")))
        val refused = assertThrows<EranosException.MismatchedRecord> { flat.group.addBlocking(Member("Dan", accounts = listOf(account))) }
        assertEquals(MismatchedField.IBAN_COUNTRY_CODE, refused.field)
        assertEquals(ana, flat.group.memberBlocking(flat.ana))
        assertEquals(listOf(flat.ana, flat.bob), flat.group.membersBlocking().map { it.id })
    }

    /**
     * Identifiers of different entities that compared, or printed, as each other's text would let an app store one and read
     * back another; a getter that read another text would do the same to Java, which receives identifiers boxed, such as in
     * the list of an expense's members.
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
        // Kotlin compiles no call to a value class's getter: Java reads a boxed identifier through it.
        val boxed = listOf<Any>(ExpenseGroupId(text), MemberId(text), CategoryId(text), ExpenseId(text), TransferId(text))
        for (identifier in boxed) {
            assertEquals(text, identifier.javaClass.getMethod("getValue").invoke(identifier))
        }
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
        for (refusal in identifierRefusals) {
            val refused = assertThrows<EranosException.Id> { group.memberBlocking(MemberId(refusal.text)) }
            assertEquals(refusal.failure, refused.failure)
            assertEquals(refusal.message, refused.message)
        }
    }

    /**
     * A parse that checked by other rules than the library's, changed the text, or refused it with another failure or
     * message would let an app store an identifier no group accepts, or another than the one it gave.
     */
    @Test
    fun everyIdentifierParsesByTheLibrarysRules() {
        val parses: List<(String) -> Any> =
            listOf(
                { text -> ExpenseGroupId.parse(text).value },
                { text -> MemberId.parse(text).value },
                { text -> CategoryId.parse(text).value },
                { text -> ExpenseId.parse(text).value },
                { text -> TransferId.parse(text).value },
            )
        for (parse in parses) {
            // Kept as it is, case and inner spaces included.
            for (text in listOf("-food", "!a", "zz", "Lisboa · 2026", "é".repeat(32))) {
                assertEquals(text, parse(text))
            }
            // Each refusal names the first rule the text breaks.
            for (refusal in identifierRefusals) {
                val refused = assertThrows<EranosException.Id> { parse(refusal.text) }
                assertEquals(refusal.failure, refused.failure)
                assertEquals(refusal.message, refused.message)
            }
        }
    }

    /**
     * A generate that repeated an identifier, made one its parse refuses, or drew from a generator of its own for a kind would
     * give two entries one identifier, leave an app an identifier it cannot store, or list entries out of creation order.
     */
    @Test
    fun everyIdentifierGeneratesNewTextThatItsParseAccepts() {
        val group = ExpenseGroupId.generate()
        val member = MemberId.generate()
        val category = CategoryId.generate()
        val expense = ExpenseId.generate()
        val transfer = TransferId.generate()
        assertEquals(group, ExpenseGroupId.parse(group.value))
        assertEquals(member, MemberId.parse(member.value))
        assertEquals(category, CategoryId.parse(category.value))
        assertEquals(expense, ExpenseId.parse(expense.value))
        assertEquals(transfer, TransferId.parse(transfer.value))
        // Every kind draws from one generator, so each text is new and sorts after the one generated before it.
        val texts = listOf(group.value, member.value, category.value, expense.value, transfer.value, MemberId.generate().value)
        assertEquals(texts.distinct().sorted(), texts)
    }

    /**
     * A built-in category's identifier other than the library's would name a category no group holds: an expense filed under
     * it would be refused, and a total looked up by it would find none.
     */
    @Test
    fun theBuiltInCategoryIsIdentifiedByTheNilUuid() {
        val text = "00000000-0000-0000-0000-000000000000"
        assertEquals(text, CategoryId.UNCATEGORISED.value)
        assertEquals(CategoryId.UNCATEGORISED, CategoryId.parse(text))
    }
}

/** Text the library refuses as an identifier, with the failure and the message it reports. */
private data class IdentifierRefusal(
    val text: String,
    val failure: IdFailure,
    val message: String,
)

/** Text that breaks each rule of an identifier, in characters and in bytes. */
private val identifierRefusals: List<IdentifierRefusal> =
    run {
        val empty = "an identifier must not be empty"
        val tooLong = "an identifier can be at most 64 bytes long"
        val controlCharacter = "an identifier cannot contain control characters"
        val surroundingWhitespace = "an identifier cannot start or end with whitespace"
        listOf(
            IdentifierRefusal(text = "", failure = IdFailure.Empty, message = empty),
            IdentifierRefusal(text = "a".repeat(65), failure = IdFailure.TooLong, message = tooLong),
            IdentifierRefusal(text = "é".repeat(33), failure = IdFailure.TooLong, message = tooLong),
            IdentifierRefusal(text = "a\tb", failure = IdFailure.ControlCharacter, message = controlCharacter),
            IdentifierRefusal(text = "\u0007 ", failure = IdFailure.ControlCharacter, message = controlCharacter),
            IdentifierRefusal(text = " ana", failure = IdFailure.SurroundingWhitespace, message = surroundingWhitespace),
            IdentifierRefusal(text = "ana\u00a0", failure = IdFailure.SurroundingWhitespace, message = surroundingWhitespace),
        )
    }
