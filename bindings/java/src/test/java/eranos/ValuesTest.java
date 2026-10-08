// The value types. Every text they show comes from the library, every value an app builds is
// checked by the library's rules, and a value the library never sends still prints something: a
// type that formatted on its own, let a refused value through, or threw on an impossible one would
// show an app another value than the library holds.

package eranos;

import static eranos.Flat.assertValue;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Set;

/**
 * The currencies, amounts, ratios, dates, times, contact details, and identifiers, and the values
 * an expense starts at.
 */
class ValuesTest {
    /**
     * A text the identifier rules refuse, the failure that names why, and the library's message.
     */
    private record IdentifierRefusal(String text, IdFailure failure, String message) {}

    /**
     * A currency, an amount, or a check that read or wrote another way than the library's would
     * show an app another amount than the group holds.
     */
    @Test
    void currenciesAndAmountsReadAndWriteAsTheLibraryDoes() throws EranosException {
        CurrencyCode euro = CurrencyCode.of("EUR");
        assertEquals("EUR", euro.code());
        assertEquals("Euro", euro.name());
        assertEquals("EUR", euro.toString());
        assertEquals(978, euro.numericCode());
        assertEquals(2, euro.decimalPlaces());
        assertTrue(CurrencyCode.all().contains(euro));
        CurrencyCode lei = CurrencyCode.of("RON");
        assertValue(euro, CurrencyCode.of("EUR"), lei);
        EranosException.Money unknown =
                assertThrows(EranosException.Money.class, () -> CurrencyCode.of("XYZ"));
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
     * An amount whose currency was looked up by its code would show an app the decimal places, the
     * name, or the numeric code of the library's currency of that code instead of those the library
     * sent with it, and decimal places read as a signed byte would turn negative from 128.
     */
    @Test
    void anAmountCarriesTheDetailsOfItsCurrencyAsTheLibrarySendsThem() {
        eranos.internal.CurrencyDetails sent =
                new eranos.internal.CurrencyDetails("EUR", "Sent euro", 999, (byte) 3);
        Money amount = Money.converted(new eranos.internal.Money(-1_234, sent));
        assertEquals(-1_234, amount.minorUnits());
        CurrencyCode currency = amount.currency();
        assertEquals("EUR", currency.code());
        assertEquals("Sent euro", currency.name());
        assertEquals(999, currency.numericCode());
        assertEquals(3, currency.decimalPlaces());
        assertEquals("-1.234 EUR", amount.toString());
        assertEquals(new eranos.internal.Money(-1_234, sent), amount.generated());

        eranos.internal.CurrencyDetails manyPlaces =
                new eranos.internal.CurrencyDetails("XYZ", "Unlisted", 999, (byte) 200);
        CurrencyCode unlisted = CurrencyCode.converted(manyPlaces);
        assertEquals(200, unlisted.decimalPlaces());
        assertEquals(manyPlaces, unlisted.generated());
    }

    /** An amount in a currency with another number of places, beside the library's text for it. */
    private record AmountText(Money amount, String text) {}

    /**
     * An amount written with other decimal places than its currency's would show an app another
     * amount than the group holds.
     */
    @Test
    void amountsWriteTheDecimalPlacesOfTheirCurrency() throws EranosException {
        List<AmountText> amounts =
                List.of(
                        new AmountText(new Money(1_234, CurrencyCode.of("JPY")), "1234 JPY"),
                        new AmountText(new Money(1_234, CurrencyCode.of("EUR")), "12.34 EUR"),
                        new AmountText(new Money(1_234, CurrencyCode.of("BHD")), "1.234 BHD"),
                        new AmountText(new Money(1_234, CurrencyCode.of("CLF")), "0.1234 CLF"));
        for (AmountText written : amounts) {
            assertEquals(written.text(), written.amount().toString());
        }
        assertEquals(
                List.of(0, 2, 3, 4),
                amounts.stream()
                        .map(written -> written.amount().currency().decimalPlaces())
                        .toList());
    }

    /**
     * An amount whose currency's details are not the library's for its code would reach the library
     * as the library's currency of that code, and the group would hold another amount than the app
     * showed. The layer's public types build no such currency: only the details the library sends
     * do.
     */
    @Test
    void anAmountWhoseCurrencyIsNotTheLibrarysIsRefused() throws EranosException {
        Flat flat = new Flat();
        CurrencyCode altered =
                CurrencyCode.converted(
                        new eranos.internal.CurrencyDetails("EUR", "Euro", 978, (byte) 3));
        assertNotEquals(flat.euro, altered);
        Money amount = new Money(1_000, altered);
        EranosException.Money added =
                assertThrows(EranosException.Money.class, () -> amount.plus(flat.euros(1)));
        assertEquals(new MoneyFailure.UnknownCurrency(), added.failure());
        EranosException.Money transferred =
                assertThrows(
                        EranosException.Money.class,
                        () ->
                                flat.group.addTransfer(
                                        Transfer.builder(
                                                        flat.bob,
                                                        flat.ana,
                                                        new Cost.Local(amount),
                                                        flat.day)
                                                .build()));
        assertEquals(new MoneyFailure.UnknownCurrency(), transferred.failure());
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
        CurrencyCode lei = CurrencyCode.of("RON");
        CurrencyCode euro = CurrencyCode.of("EUR");
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
        EranosException.PaymentAccount refused =
                assertThrows(
                        EranosException.PaymentAccount.class,
                        () -> Iban.parse("RO49AAAA1B31007593840001"));
        assertEquals(
                new PaymentAccountFailure.InvalidIban(new IbanReason.InvalidChecksum()),
                refused.failure());
        assertEquals("invalid IBAN: the check digits do not match", refused.getMessage());

        Username username = Username.parse(" ana.pays ");
        assertEquals("ana.pays", username.toString());
        assertValue(username, Username.parse("ana.pays"), Username.parse("bob.pays"));
        assertThrows(EranosException.PaymentAccount.class, () -> Username.parse(" "));

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
        assertEquals(text, new ExpenseGroupId(text).value());
        assertEquals(text, new ExpenseGroupId(text).toString());
        assertEquals(text, new MemberId(text).toString());
        assertEquals(text, new CategoryId(text).toString());
        assertEquals(text, new ExpenseId(text).toString());
        assertEquals(text, new TransferId(text).toString());
        assertThrows(NullPointerException.class, () -> new MemberId(null));
    }

    /**
     * A generated identifier that the rules refuse would fail the call that carries it, and two
     * generated one after the other that were equal, of one kind or of two, would merge two
     * entries; one out of order would list a new entry before an older one.
     */
    @Test
    void generatedIdentifiersFollowTheRulesAndCountUp() throws EranosException {
        ExpenseGroupId group = ExpenseGroupId.generate();
        MemberId member = MemberId.generate();
        CategoryId category = CategoryId.generate();
        ExpenseId expense = ExpenseId.generate();
        TransferId transfer = TransferId.generate();
        assertEquals(group, ExpenseGroupId.parse(group.value()));
        assertEquals(member, MemberId.parse(member.value()));
        assertEquals(category, CategoryId.parse(category.value()));
        assertEquals(expense, ExpenseId.parse(expense.value()));
        assertEquals(transfer, TransferId.parse(transfer.value()));
        List<String> texts =
                List.of(
                        group.value(),
                        member.value(),
                        category.value(),
                        expense.value(),
                        transfer.value());
        assertEquals(texts.size(), Set.copyOf(texts).size());
        assertEquals(texts.stream().sorted().toList(), texts);
    }

    /**
     * An identifier that parsing let through against the library's rules would be refused only when
     * a call carries it, after the app stored it; one parsed into another text, or refused for
     * another rule, would show an app another identifier or another reason.
     */
    @Test
    void identifiersParseAsTheLibraryReadsThem() throws EranosException {
        // Kept as it is, case and inner spaces included; the limit is 64 bytes of UTF-8, which 32
        // two-byte characters reach.
        for (String text : List.of("-food", "!a", "Lisboa · 2026", "é".repeat(32))) {
            assertEquals(new ExpenseGroupId(text), ExpenseGroupId.parse(text));
            assertEquals(new MemberId(text), MemberId.parse(text));
            assertEquals(new CategoryId(text), CategoryId.parse(text));
            assertEquals(new ExpenseId(text), ExpenseId.parse(text));
            assertEquals(new TransferId(text), TransferId.parse(text));
        }
        // Each refusal names the first rule the text breaks.
        for (IdentifierRefusal refusal :
                List.of(
                        new IdentifierRefusal(
                                "", new IdFailure.Empty(), "an identifier must not be empty"),
                        new IdentifierRefusal(
                                "é".repeat(33),
                                new IdFailure.TooLong(),
                                "an identifier can be at most 64 bytes long"),
                        new IdentifierRefusal(
                                "a\tb",
                                new IdFailure.ControlCharacter(),
                                "an identifier cannot contain control characters"),
                        new IdentifierRefusal(
                                " x",
                                new IdFailure.SurroundingWhitespace(),
                                "an identifier cannot start or end with whitespace"))) {
            EranosException.Id refused =
                    assertThrows(EranosException.Id.class, () -> MemberId.parse(refusal.text()));
            assertEquals(refusal.failure(), refused.failure());
            assertEquals(refusal.message(), refused.getMessage());
        }
    }

    /**
     * A built-in category's identifier or a no-tip rule other than the library's would put an
     * expense the app left in the built-in category into another one, or give an expense the app
     * left without a tip a tip, or one shared another way than the library's own expense without a
     * tip.
     */
    @Test
    void theBuiltInCategoryAndTheNoTipRuleAreTheLibrarys() throws EranosException {
        assertEquals("00000000-0000-0000-0000-000000000000", CategoryId.UNCATEGORISED.value());
        assertEquals(
                new CategoryId(eranos.internal.Eranos.categoryIdUncategorised()),
                CategoryId.UNCATEGORISED);
        assertEquals(CategoryId.UNCATEGORISED, CategoryId.parse(CategoryId.UNCATEGORISED.value()));
        assertEquals(
                new TipRule(
                        new TipValue.Percentage(Percentage.ofBasisPoints(0)),
                        TipDistribution.PROPORTIONAL),
                TipRule.NONE);
        assertEquals(TipRule.converted(eranos.internal.Eranos.tipRuleNone()), TipRule.NONE);
    }

    /**
     * A value the library never sends, such as a negative percentage, would throw from its text
     * instead of printing.
     */
    @Test
    void aValueTheLibraryNeverSendsStillPrints() {
        assertEquals("-1 bp", Percentage.known(-1).toString());
        assertEquals("0/100", Parts.known(0).toString());
        assertEquals(
                "2026/2/30",
                CalendarDate.converted(new eranos.internal.CalendarDate(2026, 2, 30)).toString());
        assertEquals("25h0", TimeOfDay.converted(new eranos.internal.TimeOfDay(25, 0)).toString());
        assertEquals("0712", PhoneNumber.known("0712").toString());
        assertEquals("XY", Iban.known("XY").countryCode());
    }
}
