// The value types. Every text they show comes from the library, and every value an app builds is
// checked by the library's rules: a type that formatted on its own, kept derived text of its own,
// or let a refused value through would show an app another value than the library holds.

package eranos;

import static eranos.Flat.assertValue;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import java.util.List;
import java.util.Set;

/**
 * The currencies, amounts, ratios, quantities, dates, times, contact details, and identifiers, and
 * the values an expense starts at.
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
        assertEquals("0%", Percentage.ofBasisPoints(0).toString());
        EranosException.Money negative =
                assertThrows(EranosException.Money.class, () -> Percentage.ofBasisPoints(-1));
        assertEquals(new MoneyFailure.InvalidPercentage(), negative.failure());
        assertThrows(EranosException.Money.class, () -> Percentage.parse("half"));

        Parts parts = Parts.parse("0.5");
        assertEquals(50, parts.hundredths());
        assertEquals("0.5", parts.toString());
        assertValue(parts, Parts.ofHundredths(50), Parts.ofHundredths(1));
        assertEquals("0.01", Parts.ofHundredths(1).toString());
        for (int hundredths : List.of(0, -50)) {
            EranosException.Money refused =
                    assertThrows(EranosException.Money.class, () -> Parts.ofHundredths(hundredths));
            assertEquals(new MoneyFailure.InvalidParts(), refused.failure());
        }
        assertThrows(EranosException.Money.class, () -> Parts.parse("half"));
    }

    /**
     * A quantity read, built, or written another way than the library's would divide another line
     * total, and one the library refuses, or one too large for a {@code long}, would reach an item.
     */
    @Test
    void quantitiesReadAndWriteAsTheLibraryDoes() throws EranosException {
        Quantity weight = Quantity.parse("0.452");
        assertEquals(4_520, weight.tenThousandths());
        assertEquals("0.452", weight.toString());
        assertValue(weight, Quantity.ofTenThousandths(4_520), Quantity.ONE);
        assertEquals(30_000, Quantity.parse(" 3 ").tenThousandths());
        assertEquals(Long.MAX_VALUE, Quantity.parse("922337203685477.5807").tenThousandths());
        assertEquals("0.0001", Quantity.ofTenThousandths(1).toString());
        assertEquals(10_000, Quantity.ONE.tenThousandths());
        assertEquals("1", Quantity.ONE.toString());
        Quantity three = Quantity.ofWhole(3);
        assertEquals(30_000, three.tenThousandths());
        assertEquals("3", three.toString());
        assertEquals(
                9_223_372_036_854_770_000L,
                Quantity.ofWhole(922_337_203_685_477L).tenThousandths());
        for (long tenThousandths : List.of(0L, -1L, Long.MIN_VALUE)) {
            assertEquals(
                    new MoneyFailure.InvalidQuantity(),
                    moneyFailure(() -> Quantity.ofTenThousandths(tenThousandths)));
        }
        for (String text : List.of("0", "0.00005", "-1", "1e2", "half", "922337203685477.5808")) {
            assertEquals(
                    new MoneyFailure.InvalidQuantity(), moneyFailure(() -> Quantity.parse(text)));
        }
        for (long units : List.of(0L, -1L, Long.MIN_VALUE, 922_337_203_685_478L, Long.MAX_VALUE)) {
            assertEquals(
                    new MoneyFailure.InvalidQuantity(),
                    moneyFailure(() -> Quantity.ofWhole(units)));
        }
        EranosException.Money zero =
                assertThrows(EranosException.Money.class, () -> Quantity.parse("0"));
        assertEquals(
                "invalid quantity: expected a positive number with at most 4 decimal places",
                zero.getMessage());
    }

    /** The failure of the money module that {@code call} throws. */
    private static MoneyFailure moneyFailure(Executable call) {
        return assertThrows(EranosException.Money.class, call).failure();
    }

    /**
     * An app that holds amounts as doubles would get other cents in Java than the library rounds
     * to, or an amount back from NaN, from an infinity, or from a value past the range.
     */
    @Test
    void amountsConvertFromDoublesAsTheLibraryConvertsThem() throws EranosException {
        CurrencyCode euro = CurrencyCode.of("EUR");
        CurrencyCode lei = CurrencyCode.of("RON");
        CurrencyCode yen = CurrencyCode.of("JPY");
        // The double nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
        Money rounded = Money.fromDouble(1.005, euro);
        assertEquals(new Money(101, euro), rounded);
        assertEquals("1.01 EUR", rounded.toString());
        assertEquals(new Money(-268, lei), Money.fromDouble(-2.675, lei));
        assertEquals(new Money(1_235, yen), Money.fromDouble(1_234.5, yen));
        EranosException.Money notANumber =
                assertThrows(EranosException.Money.class, () -> Money.fromDouble(Double.NaN, euro));
        assertEquals(new MoneyFailure.NotFinite(), notANumber.failure());
        assertEquals("the number is not finite", notANumber.getMessage());
        assertEquals(
                new MoneyFailure.NotFinite(),
                moneyFailure(() -> Money.fromDouble(Double.NEGATIVE_INFINITY, euro)));
        assertEquals(new MoneyFailure.Overflow(), moneyFailure(() -> Money.fromDouble(1e21, euro)));
    }

    /**
     * A rate, a percentage, parts, or a quantity held as a double would become other digits in Java
     * than in the library, a number too large for the layer's integers would wrap, or NaN or an
     * infinity would come back as a value.
     */
    @Test
    void ratesRatiosAndQuantitiesConvertFromDoublesAsTheLibraryConvertsThem()
            throws EranosException {
        assertEquals(ExchangeRate.parse("25.5"), ExchangeRate.fromDouble(25.5));
        assertEquals("0.30000000000000004", ExchangeRate.fromDouble(0.1 + 0.2).toString());
        assertEquals(
                new MoneyFailure.InvalidExchangeRate(),
                moneyFailure(() -> ExchangeRate.fromDouble(0.0)));

        assertEquals(30, Percentage.fromDouble(0.1 + 0.2).basisPoints());
        assertEquals(1_235, Percentage.fromDouble(12.345).basisPoints());
        // Above 21 474 836.47 %, the basis points do not fit an int.
        for (double value : List.of(-1.0, 30_000_000.0)) {
            assertEquals(
                    new MoneyFailure.InvalidPercentage(),
                    moneyFailure(() -> Percentage.fromDouble(value)));
        }

        assertEquals(101, Parts.fromDouble(1.005).hundredths());
        for (double value : List.of(0.004, 30_000_000.0)) {
            assertEquals(
                    new MoneyFailure.InvalidParts(), moneyFailure(() -> Parts.fromDouble(value)));
        }

        assertEquals(4_521, Quantity.fromDouble(0.45205).tenThousandths());
        assertEquals(Quantity.parse("1.5"), Quantity.fromDouble(1.5));
        // Above 922 337 203 685 477.5807, the ten-thousandths do not fit a long.
        for (double value : List.of(0.00004, -1.0, 1e21)) {
            assertEquals(
                    new MoneyFailure.InvalidQuantity(),
                    moneyFailure(() -> Quantity.fromDouble(value)));
        }

        for (double value :
                List.of(Double.NaN, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY)) {
            assertEquals(
                    new MoneyFailure.NotFinite(),
                    moneyFailure(() -> ExchangeRate.fromDouble(value)));
            assertEquals(
                    new MoneyFailure.NotFinite(), moneyFailure(() -> Percentage.fromDouble(value)));
            assertEquals(new MoneyFailure.NotFinite(), moneyFailure(() -> Parts.fromDouble(value)));
            assertEquals(
                    new MoneyFailure.NotFinite(), moneyFailure(() -> Quantity.fromDouble(value)));
        }
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
        assertEquals("0001-01-01", CalendarDate.of(1, 1, 1).toString());
        assertEquals("9999-12-31", CalendarDate.of(9_999, 12, 31).toString());
        assertEquals(
                new CalendarFailure.InvalidDate(),
                assertThrows(EranosException.Calendar.class, () -> CalendarDate.of(0, 12, 31))
                        .failure());
        assertEquals(
                new CalendarFailure.InvalidDate(),
                assertThrows(EranosException.Calendar.class, () -> CalendarDate.of(10_000, 1, 1))
                        .failure());
        assertThrows(EranosException.Calendar.class, () -> CalendarDate.parse("2026-02-30"));

        TimeOfDay time = TimeOfDay.of(18, 30);
        assertEquals("18:30", time.toString());
        assertEquals(18, time.hour());
        assertEquals(30, time.minute());
        assertValue(time, TimeOfDay.parse("18:30"), TimeOfDay.of(18, 31));
        assertEquals("00:00", TimeOfDay.of(0, 0).toString());
        assertEquals("23:59", TimeOfDay.of(23, 59).toString());
        assertEquals(
                new CalendarFailure.InvalidTime(),
                assertThrows(EranosException.Calendar.class, () -> TimeOfDay.of(24, 0)).failure());
        assertEquals(
                new CalendarFailure.InvalidTime(),
                assertThrows(EranosException.Calendar.class, () -> TimeOfDay.of(0, 60)).failure());
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
        assertEquals("+40 712 345 678", phone.international());
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
     * A phone number or an IBAN that asked the library for its derived text on every read could
     * fail to show a value it holds, and one that derived the text itself could show another form
     * of the number or another country than the library's; both read it from the record the library
     * sent, which crosses back whole.
     */
    @Test
    void aPhoneNumberAndAnIbanReadTheirDerivedTextFromTheirRecord() {
        eranos.internal.PhoneNumber sent =
                new eranos.internal.PhoneNumber("+40712345678", "as the record holds it");
        PhoneNumber phone = PhoneNumber.converted(sent);
        assertEquals("+40712345678", phone.e164());
        assertEquals("as the record holds it", phone.international());
        assertEquals("as the record holds it", phone.toString());
        assertEquals(sent, phone.generated());
        assertNotEquals(
                phone,
                PhoneNumber.converted(
                        new eranos.internal.PhoneNumber("+40712345678", "+40 712 345 678")));

        Iban iban = Iban.known("RO49AAAA1B31007593840000", "as the record holds it");
        assertEquals("RO49AAAA1B31007593840000", iban.toString());
        assertEquals("as the record holds it", iban.countryCode());
        assertEquals(
                new eranos.internal.PaymentAccountDetails.Iban(
                        "RO49AAAA1B31007593840000", "as the record holds it"),
                Conversions.generated(new PaymentAccountDetails.Iban(iban)));
        assertNotEquals(iban, Iban.known("RO49AAAA1B31007593840000", "RO"));
    }

    /**
     * A phone number whose international form, or an IBAN whose country code, is not the one the
     * library derives would reach the group as a value the library never wrote. The layer's public
     * types build no such value: only the records the library sends do.
     */
    @Test
    void aPhoneNumberOrAnIbanWhoseDerivedTextIsNotTheLibrarysIsRefused() throws EranosException {
        Flat flat = new Flat();
        Member grouped =
                Member.builder("Carol")
                        .phone(
                                PhoneNumber.converted(
                                        new eranos.internal.PhoneNumber(
                                                "+40712345678", "+40 7123 45678")))
                        .build();
        EranosException.Contact phone =
                assertThrows(EranosException.Contact.class, () -> flat.group.addMember(grouped));
        assertEquals(new ContactFailure.InvalidPhoneNumber(), phone.failure());

        Member banked =
                Member.builder("Dan")
                        .accounts(
                                List.of(
                                        new PaymentAccount(
                                                "Bank",
                                                new PaymentAccountDetails.Iban(
                                                        Iban.known(
                                                                "RO49AAAA1B31007593840000",
                                                                "GB")))))
                        .build();
        EranosException.PaymentAccount iban =
                assertThrows(
                        EranosException.PaymentAccount.class, () -> flat.group.addMember(banked));
        assertEquals(
                new PaymentAccountFailure.InvalidIban(new IbanReason.UnknownCountry()),
                iban.failure());
        assertEquals(
                List.of(flat.ana, flat.bob),
                flat.group.members().stream().map(Member::id).toList());
    }
}
