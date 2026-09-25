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

import java.util.List;

/** The currencies, amounts, ratios, dates, times, contact details, and identifiers. */
class ValuesTest {
    /**
     * A currency, an amount, or a check that read or wrote another way than the library's would
     * show an app another amount than the group holds.
     */
    @Test
    void currenciesAndAmountsReadAndWriteAsTheLibraryDoes() throws EranosException {
        Currency euro = Currency.of("EUR");
        assertEquals(
                List.of("EUR", "Euro", "EUR"), List.of(euro.code(), euro.name(), euro.toString()));
        assertEquals(List.of(978, 2), List.of(euro.numericCode(), euro.decimalPlaces()));
        assertTrue(Currency.all().contains(euro));
        Currency lei = Currency.of("RON");
        assertValue(euro, Currency.known("EUR"), lei);
        EranosException.Money unknown =
                assertThrows(EranosException.Money.class, () -> Currency.of("XYZ"));
        assertEquals(new MoneyFailure.UnknownCurrency(), unknown.failure());
        assertEquals("unknown currency code", unknown.getMessage());

        Money price = Money.parse("12.5", euro);
        assertEquals(new Money(euro, 1_250), price);
        assertEquals("12.50 EUR", price.toString());
        assertEquals(new Money(euro, 1_300), price.plus(new Money(euro, 50)));
        assertEquals(new Money(euro, 1_200), price.minus(new Money(euro, 50)));
        EranosException.Money mismatch =
                assertThrows(EranosException.Money.class, () -> price.plus(new Money(lei, 1)));
        assertEquals(new MoneyFailure.CurrencyMismatch(euro, lei), mismatch.failure());
        assertThrows(
                EranosException.Money.class,
                () -> new Money(euro, Long.MAX_VALUE).plus(new Money(euro, 1)));
        assertThrows(
                EranosException.Money.class,
                () -> new Money(euro, Long.MIN_VALUE).minus(new Money(euro, 1)));
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
                new Money(euro, 1), ExchangeRate.parse("0.25").convert(new Money(lei, 2), euro));
        assertThrows(EranosException.Money.class, () -> ExchangeRate.parse("0"));
        assertThrows(
                EranosException.Money.class,
                () -> ExchangeRate.parse("2").convert(new Money(lei, Long.MAX_VALUE), euro));

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
        CalendarDate date = CalendarDate.of(23, 9, 2026);
        assertEquals("2026-09-23", date.toString());
        assertEquals(List.of(23, 9, 2026), List.of(date.day(), date.month(), date.year()));
        assertValue(date, CalendarDate.parse("2026-09-23"), CalendarDate.of(24, 9, 2026));
        EranosException.Calendar invalid =
                assertThrows(EranosException.Calendar.class, () -> CalendarDate.of(30, 2, 2026));
        assertEquals(new CalendarFailure.InvalidDate(), invalid.failure());
        assertEquals(
                "invalid date: expected an existing ISO 8601 date between years 1 and 9999",
                invalid.getMessage());
        assertThrows(EranosException.Calendar.class, () -> CalendarDate.parse("2026-02-30"));

        TimeOfDay time = TimeOfDay.of(18, 30);
        assertEquals("18:30", time.toString());
        assertEquals(List.of(18, 30), List.of(time.hour(), time.minute()));
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
        assertEquals(
                List.of(text, text),
                List.of(new GroupId(text).value(), new GroupId(text).toString()));
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
        assertEquals(
                List.of("XYZ", "XYZ", 0, 0),
                List.of(
                        unlisted.code(),
                        unlisted.name(),
                        unlisted.decimalPlaces(),
                        unlisted.numericCode()));
        assertEquals("5 XYZ", new Money(unlisted, 5).toString());
        assertEquals("-1 bp", Percentage.known(-1).toString());
        assertEquals("0/100", Parts.known(0).toString());
        assertEquals(
                "2026/2/30",
                CalendarDate.converted(new eranos.internal.CalendarDate(30, 2, 2026)).toString());
        assertEquals("25h0", TimeOfDay.converted(new eranos.internal.TimeOfDay(25, 0)).toString());
        assertEquals("0712", PhoneNumber.known("0712").toString());
        assertEquals("XY", Iban.known("XY").countryCode());
        assertTrue(new Balance.Owes(new Money(unlisted, 1)).toString().contains("XYZ"));
    }
}
