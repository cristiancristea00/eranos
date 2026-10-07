// The value types. Every text they show comes from the library, every value an app builds is
// checked by the library's rules, and a value the library never sends still prints something: a
// type that formatted on its own, let a refused value through, or threw on an impossible one would
// show an app another value than the library holds.

package eranos;

import static eranos.Flat.assertValue;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/** The currencies, amounts, ratios, dates, times, contact details, and identifiers. */
class ValuesTest {
    /**
     * A currency, an amount, or a check that read or wrote another way than the library's would
     * show an app another amount than the group holds.
     */
    @Test
    void currenciesAndAmountsReadAndWriteAsTheLibraryDoes() throws EranosException {
        Currency euro = Currency.of("EUR");
        assertEquals("EUR", euro.code());
        assertEquals("Euro", euro.name());
        assertEquals("EUR", euro.toString());
        assertEquals(978, euro.numericCode());
        assertEquals(2, euro.decimalPlaces());
        assertTrue(Currency.all().contains(euro));
        Currency lei = Currency.of("RON");
        assertValue(euro, Currency.known("EUR"), lei);
        EranosException.Money unknown =
                assertThrows(EranosException.Money.class, () -> Currency.of("XYZ"));
        assertEquals(new MoneyFailure.UnknownCurrency(), unknown.failure());
        assertEquals("unknown currency code", unknown.getMessage());

        Money price = Money.parse("12.5", euro);
        assertEquals(new Money(1_250, euro), price);
        assertEquals("12.50 EUR", price.toString());
        assertEquals(new Money(1_300, euro), price.plus(new Money(50, euro)));
        assertEquals(new Money(1_200, euro), price.minus(new Money(50, euro)));
        EranosException.Money mismatch =
                assertThrows(EranosException.Money.class, () -> price.plus(new Money(1, lei)));
        assertEquals(new MoneyFailure.CurrencyMismatch(euro, lei), mismatch.failure());
        assertThrows(
                EranosException.Money.class,
                () -> new Money(Long.MAX_VALUE, euro).plus(new Money(1, euro)));
        assertThrows(
                EranosException.Money.class,
                () -> new Money(Long.MIN_VALUE, euro).minus(new Money(1, euro)));
        assertThrows(EranosException.Money.class, () -> Money.parse("twelve", euro));
    }

    /**
     * A rate, a percentage, or a number of parts read or written another way than the library's
     * would divide an expense another way.
     */
    @Test
    void ratesPercentagesAndPartsReadAndWriteAsTheLibraryDoes() throws EranosException {
        ExchangeRate rate = ExchangeRate.parse("4.970");
        assertEquals("4.97", rate.toString());
        assertValue(rate, ExchangeRate.parse("4.97"), ExchangeRate.parse("5"));
        Currency lei = Currency.of("RON");
        Currency euro = Currency.of("EUR");
        assertEquals(
                new Money(1, euro), ExchangeRate.parse("0.25").convert(new Money(2, lei), euro));
        assertThrows(EranosException.Money.class, () -> ExchangeRate.parse("0"));
        assertThrows(
                EranosException.Money.class,
                () -> ExchangeRate.parse("2").convert(new Money(Long.MAX_VALUE, lei), euro));

        Percentage percentage = Percentage.parse("12.5 %");
        assertEquals(1_250, percentage.basisPoints());
        assertEquals("12.5%", percentage.toString());
        assertValue(percentage, Percentage.ofBasisPoints(1_250), Percentage.ofBasisPoints(1));
        assertThrows(EranosException.Money.class, () -> Percentage.ofBasisPoints(-1));
        assertThrows(EranosException.Money.class, () -> Percentage.parse("half"));

        Parts parts = Parts.parse("0.5");
        assertEquals(50, parts.hundredths());
        assertEquals("0.5", parts.toString());
        assertValue(parts, Parts.ofHundredths(50), Parts.ofHundredths(1));
        assertThrows(EranosException.Money.class, () -> Parts.ofHundredths(0));
        assertThrows(EranosException.Money.class, () -> Parts.parse("half"));
    }

    /**
     * A date or a time read or written another way than the library's, or one that does not exist,
     * would put an entry on another day.
     */
    @Test
    void datesAndTimesReadAndWriteAsTheLibraryDoes() throws EranosException {
        CalendarDate date = CalendarDate.of(2026, 9, 23);
        assertEquals("2026-09-23", date.toString());
        assertEquals(2026, date.year());
        assertEquals(9, date.month());
        assertEquals(23, date.day());
        assertValue(date, CalendarDate.parse("2026-09-23"), CalendarDate.of(2026, 9, 24));
        EranosException.Calendar invalid =
                assertThrows(EranosException.Calendar.class, () -> CalendarDate.of(2026, 2, 30));
        assertEquals(new CalendarFailure.InvalidDate(), invalid.failure());
        assertEquals(
                "invalid date: expected an existing ISO 8601 date between years 1 and 9999",
                invalid.getMessage());
        assertThrows(EranosException.Calendar.class, () -> CalendarDate.parse("2026-02-30"));

        TimeOfDay time = TimeOfDay.of(18, 30);
        assertEquals("18:30", time.toString());
        assertEquals(18, time.hour());
        assertEquals(30, time.minute());
        assertValue(time, TimeOfDay.parse("18:30"), TimeOfDay.of(18, 31));
        assertThrows(EranosException.Calendar.class, () -> TimeOfDay.of(24, 0));
        assertThrows(EranosException.Calendar.class, () -> TimeOfDay.parse("25:00"));
    }

    /**
     * Contact and account details kept in another form than the library's, or refused ones let
     * through, would reach the group unchecked.
     */
    @Test
    void contactAndAccountDetailsReadAsTheLibraryDoes() throws EranosException {
        Email email = Email.parse(" ana@example.com ");
        assertEquals("ana@example.com", email.toString());
        assertValue(email, Email.parse("ana@example.com"), Email.parse("bob@example.com"));
        assertThrows(EranosException.Contact.class, () -> Email.parse("ana"));

        PhoneNumber phone = PhoneNumber.parse("+40 712 345 678");
        assertEquals("+40712345678", phone.e164());
        assertEquals("+40 712 345 678", phone.toString());
        assertValue(phone, PhoneNumber.parse("+40712345678"), PhoneNumber.parse("+40712345679"));
        assertThrows(EranosException.Contact.class, () -> PhoneNumber.parse("0712345678"));

        Iban iban = Iban.parse("ro49 aaaa 1b31 0075 9384 0000");
        assertEquals("RO49AAAA1B31007593840000", iban.toString());
        assertEquals("RO", iban.countryCode());
        assertValue(
                iban, Iban.parse("RO49AAAA1B31007593840000"), Iban.parse("GB82WEST12345698765432"));
        EranosException.Account refused =
                assertThrows(
                        EranosException.Account.class,
                        () -> Iban.parse("RO49AAAA1B31007593840001"));
        assertEquals(
                new AccountFailure.InvalidIban(new IbanReason.InvalidChecksum()),
                refused.failure());
        assertEquals("invalid IBAN: the check digits do not match", refused.getMessage());

        Username username = Username.parse(" ana.pays ");
        assertEquals("ana.pays", username.toString());
        assertValue(username, Username.parse("ana.pays"), Username.parse("bob.pays"));
        assertThrows(EranosException.Account.class, () -> Username.parse(" "));

        Language language = Language.parse("pt-pt");
        assertEquals("pt-PT", language.toString());
        assertValue(language, Language.parse("pt-PT"), Language.parse("ro-RO"));
        EranosException.Language tag =
                assertThrows(EranosException.Language.class, () -> Language.parse("not a tag"));
        assertEquals(new LanguageFailure.InvalidLanguage(), tag.failure());
        assertEquals(
                "invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`",
                tag.getMessage());
    }

    /**
     * Identifiers of different entities that compared, or printed, as each other's text would let
     * an app store one and read back another.
     */
    @Test
    void anIdentifierIsItsText() {
        String text = "0192f1c3-0000-7000-8000-000000000001";
        assertEquals(text, new GroupId(text).value());
        assertEquals(text, new GroupId(text).toString());
        assertEquals(text, new MemberId(text).toString());
        assertEquals(text, new CategoryId(text).toString());
        assertEquals(text, new ExpenseId(text).toString());
        assertEquals(text, new TransferId(text).toString());
        assertThrows(NullPointerException.class, () -> new MemberId(null));
    }

    /**
     * A value the library never sends, such as an amount in a currency it does not list, would
     * throw from its text instead of printing.
     */
    @Test
    void aValueTheLibraryNeverSendsStillPrints() {
        Currency unlisted = Currency.known("XYZ");
        assertEquals("XYZ", unlisted.code());
        assertEquals("XYZ", unlisted.name());
        assertEquals(0, unlisted.numericCode());
        assertEquals(0, unlisted.decimalPlaces());
        assertEquals("5 XYZ", new Money(5, unlisted).toString());
        assertEquals("-1 bp", Percentage.known(-1).toString());
        assertEquals("0/100", Parts.known(0).toString());
        assertEquals(
                "2026/2/30",
                CalendarDate.converted(new eranos.internal.CalendarDate(2026, 2, 30)).toString());
        assertEquals("25h0", TimeOfDay.converted(new eranos.internal.TimeOfDay(25, 0)).toString());
        assertEquals("0712", PhoneNumber.known("0712").toString());
        assertEquals("XY", Iban.known("XY").countryCode());
        assertTrue(new Balance.Owes(new Money(1, unlisted)).toString().contains("XYZ"));
    }
}
