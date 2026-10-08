package eranos.examples.grandtour;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.grandtour.Tour.eur;
import static eranos.examples.grandtour.Tour.message;
import static eranos.examples.grandtour.Tour.rate;
import static eranos.examples.grandtour.Tour.refusal;
import static eranos.examples.grandtour.Tour.ron;

import eranos.CalendarDate;
import eranos.Cost;
import eranos.CurrencyCode;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.ExpenseGroup;
import eranos.ExpenseItem;
import eranos.IdFailure;
import eranos.Language;
import eranos.Member;
import eranos.MemberId;
import eranos.Money;
import eranos.MoneyFailure;
import eranos.Parts;
import eranos.Percentage;
import eranos.Quantity;
import eranos.Split;
import eranos.TimeOfDay;
import eranos.examples.grandtour.Tables.CurrencyRow;
import eranos.examples.grandtour.Tables.ValueRow;
import eranos.examples.grandtour.Tour.People;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Section 13 of the tour: currencies, amounts, percentages, parts, rates, quantities, dates, times,
 * identifiers, and language tags, on their own.
 */
final class Values {
    private Values() {}

    static void currencies() throws EranosException {
        List<CurrencyRow> rows = new ArrayList<>();
        for (String code : List.of("EUR", "RON", "JPY", "BHD")) {
            CurrencyCode currency = CurrencyCode.of(code);
            rows.add(
                    new CurrencyRow(
                            currency.code(),
                            currency.name(),
                            currency.numericCode(),
                            currency.decimalPlaces()));
        }
        Optional<EranosException> unknown = refusal(() -> CurrencyCode.of("XYZ"));
        System.out.print(
                """
                Currencies are the %d active ISO 4217 codes with a minor unit; an unknown code is
                refused (%s):
                """
                        .formatted(CurrencyCode.all().size(), message(unknown)));
        printTable(CurrencyRow.HEADERS, rows, 2);
        check(3 == CurrencyCode.of("BHD").decimalPlaces());
    }

    static void amounts() throws EranosException {
        Money total = eur("12.34").plus(eur("0.66"));
        Money change = eur("5.00").minus(eur("7.50"));
        Optional<EranosException> mixed = refusal(() -> eur("1.00").plus(ron("1.00")));
        Money yen = Money.parse("1500", CurrencyCode.of("JPY"));
        Money dinars = new Money(1_234, CurrencyCode.of("BHD"));
        Money nothing = new Money(0, CurrencyCode.of("RON"));
        Cost cost = new Cost.Converted(eur("10.00"), ExchangeRate.parse("4.9765"));
        Cost doubled = new Cost.Converted(eur("20.00"), rate(cost));
        // The double nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
        Money fromDouble = Money.fromDouble(1.005, CurrencyCode.of("EUR"));
        Optional<EranosException> notANumber =
                refusal(() -> Money.fromDouble(Double.NaN, CurrencyCode.of("EUR")));
        System.out.println("Amounts are exact minor units in their currency:");
        printTable(
                ValueRow.HEADERS,
                List.of(
                        new ValueRow("12.34 EUR + 0.66 EUR", total.toString()),
                        new ValueRow(
                                "5.00 EUR − 7.50 EUR",
                                change + ", negative: " + (0 > change.minorUnits())),
                        new ValueRow("1.00 EUR + 1.00 RON", message(mixed)),
                        new ValueRow("1.005 EUR", message(refusal(() -> eur("1.005")))),
                        new ValueRow("1.005 as a double, in EUR", fromDouble.toString()),
                        new ValueRow("NaN as a double, in EUR", message(notANumber)),
                        new ValueRow("1500 JPY", yen + ", " + yen.minorUnits() + " minor units"),
                        new ValueRow("1234 minor units of BHD", dinars.toString()),
                        new ValueRow(
                                "zero RON", nothing + ", zero: " + (0 == nothing.minorUnits())),
                        new ValueRow(
                                "10.00 EUR at 4.9765",
                                "%s at %s, then %s at the same rate"
                                        .formatted(cost.amount(), rate(cost), doubled.amount()))),
                2);
        check(eur("13.00").equals(total));
        check(-250 == change.minorUnits());
        check(mixed.isPresent());
        check(eur("1.01").equals(fromDouble));
        check(notANumber.isPresent());
        check("1.234 BHD".equals(dinars.toString()));
    }

    static void ratios() throws EranosException {
        Percentage percentage = Percentage.parse("12.5");
        Parts parts = Parts.parse("1.5");
        ExchangeRate rate = ExchangeRate.parse("4.97650");
        // 0.30000000000000004 as a double, which rounds to two decimal places.
        Percentage sum = Percentage.fromDouble(0.1 + 0.2);
        System.out.println("Percentages, parts, and rates are exact decimals:");
        printTable(
                ValueRow.HEADERS,
                List.of(
                        new ValueRow(
                                "12.5%",
                                percentage + ", " + percentage.basisPoints() + " basis points"),
                        new ValueRow("333 basis points", Percentage.ofBasisPoints(333).toString()),
                        new ValueRow("33.333%", message(refusal(() -> Percentage.parse("33.333")))),
                        new ValueRow("0.1 + 0.2 as a double, a percentage", sum.toString()),
                        new ValueRow(
                                "1.5 parts", parts + ", " + parts.hundredths() + " hundredths"),
                        new ValueRow("3 whole parts", Parts.ofHundredths(300).toString()),
                        new ValueRow("25 hundredths of a part", Parts.ofHundredths(25).toString()),
                        new ValueRow("0 parts", message(refusal(() -> Parts.parse("0")))),
                        new ValueRow("a rate of 4.97650", rate.toString()),
                        new ValueRow(
                                "a rate of 0", message(refusal(() -> ExchangeRate.parse("0"))))),
                2);
        check(1_250 == percentage.basisPoints());
        check(30 == sum.basisPoints());
        check(150 == parts.hundredths());
    }

    static void quantities(People people) throws EranosException {
        Quantity weight = Quantity.parse("0.452");
        // 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
        ExpenseItem olives =
                new ExpenseItem(
                        "Olives",
                        eur("1.00"),
                        new Split.Equal(List.of(people.ana())),
                        Quantity.parse("0.335"));
        Optional<EranosException> fiveDecimals = refusal(() -> Quantity.parse("0.00005"));
        Optional<EranosException> zero = refusal(() -> Quantity.parse("0"));
        System.out.println(
                "Quantities have four decimal places; line totals round half away from zero:");
        printTable(
                ValueRow.HEADERS,
                List.of(
                        new ValueRow(
                                "a quantity of 0.452",
                                weight + ", " + weight.tenThousandths() + " ten-thousandths"),
                        new ValueRow("a quantity of 0.00005", message(fiveDecimals)),
                        new ValueRow("a quantity of 0", message(zero)),
                        new ValueRow("0.335 at 1.00 EUR", olives.lineTotal().toString())),
                2);
        check(4_520 == weight.tenThousandths());
        check(Optional.of(new MoneyFailure.InvalidQuantity()).equals(moneyFailure(fiveDecimals)));
        check(Optional.of(new MoneyFailure.InvalidQuantity()).equals(moneyFailure(zero)));
        check(eur("0.34").equals(olives.lineTotal()));
    }

    static void datesAndIdentifiers(ExpenseGroup group, People people) throws EranosException {
        CalendarDate date = CalendarDate.parse("2026-10-01");
        TimeOfDay time = TimeOfDay.parse("22:15");
        MemberId parsed = new MemberId(people.ana().value());
        Optional<MemberId> found = group.member(parsed).map(Member::id);
        Optional<EranosException> empty = refusal(() -> group.member(new MemberId("")));
        Optional<EranosException> spaced = refusal(() -> group.member(new MemberId(" ana")));
        Optional<EranosException> tabbed = refusal(() -> group.member(new MemberId("a\tb")));
        Optional<EranosException> tooLong =
                refusal(() -> group.member(new MemberId("a".repeat(65))));
        Language language = Language.parse("pt-pt");
        System.out.println(
                "Dates, times, identifiers, and language tags read and print the ISO forms:");
        printTable(
                ValueRow.HEADERS,
                List.of(
                        new ValueRow(
                                "2026-10-01",
                                "%s: day %d, month %d, year %d"
                                        .formatted(date, date.day(), date.month(), date.year())),
                        new ValueRow(
                                "29 February 2027",
                                message(refusal(() -> CalendarDate.of(2027, 2, 29)))),
                        new ValueRow(
                                "22:15",
                                "%s: hour %d, minute %d"
                                        .formatted(time, time.hour(), time.minute())),
                        new ValueRow("24:00", message(refusal(() -> TimeOfDay.of(24, 0)))),
                        new ValueRow(
                                "Ana's identifier",
                                parsed
                                        + ", the same after a round trip: "
                                        + Optional.of(people.ana()).equals(found)),
                        new ValueRow("empty text", message(empty)),
                        new ValueRow("ana after a space", message(spaced)),
                        new ValueRow("a tab between a and b", message(tabbed)),
                        new ValueRow("65 letters a", message(tooLong)),
                        new ValueRow("pt-pt", language.toString()),
                        new ValueRow("pt_PT", message(refusal(() -> Language.parse("pt_PT"))))),
                2);
        check(1 == date.day() && 10 == date.month() && 2026 == date.year());
        check(22 == time.hour() && 15 == time.minute());
        check(people.ana().equals(parsed));
        check(Optional.of(new IdFailure.Empty()).equals(idFailure(empty)));
        check(Optional.of(new IdFailure.SurroundingWhitespace()).equals(idFailure(spaced)));
        check(Optional.of(new IdFailure.ControlCharacter()).equals(idFailure(tabbed)));
        check(Optional.of(new IdFailure.TooLong()).equals(idFailure(tooLong)));
    }

    /** The failure of an identifier that {@code refused} holds, if it is an identifier's. */
    private static Optional<IdFailure> idFailure(Optional<EranosException> refused) {
        return refused.filter(EranosException.Id.class::isInstance)
                .map(EranosException.Id.class::cast)
                .map(EranosException.Id::failure);
    }

    /** The failure of an amount or a number that {@code refused} holds, if it is the money's. */
    private static Optional<MoneyFailure> moneyFailure(Optional<EranosException> refused) {
        return refused.filter(EranosException.Money.class::isInstance)
                .map(EranosException.Money.class::cast)
                .map(EranosException.Money::failure);
    }

    static void values(ExpenseGroup group, People people) throws EranosException {
        section("13. Values on their own");
        currencies();
        amounts();
        ratios();
        quantities(people);
        datesAndIdentifiers(group, people);
    }
}
