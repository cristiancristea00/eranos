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
import eranos.Currency;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.Group;
import eranos.Language;
import eranos.Member;
import eranos.MemberId;
import eranos.Money;
import eranos.Parts;
import eranos.Percentage;
import eranos.TimeOfDay;
import eranos.examples.grandtour.Tables.CurrencyRow;
import eranos.examples.grandtour.Tables.ValueRow;
import eranos.examples.grandtour.Tour.People;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Section 13 of the tour: currencies, amounts, percentages, parts, rates, dates, times,
 * identifiers, and language tags, on their own.
 */
final class Values {
    private Values() {}

    static void currencies() throws EranosException {
        List<CurrencyRow> rows = new ArrayList<>();
        for (String code : List.of("EUR", "RON", "JPY", "BHD")) {
            Currency currency = Currency.of(code);
            rows.add(
                    new CurrencyRow(
                            currency.code(),
                            currency.decimalPlaces(),
                            currency.name(),
                            currency.numericCode()));
        }
        Optional<EranosException> unknown = refusal(() -> Currency.of("XYZ"));
        System.out.print(
                """
                Currencies are the %d active ISO 4217 codes with a minor unit; an unknown code is
                refused (%s):
                """
                        .formatted(Currency.all().size(), message(unknown)));
        printTable(CurrencyRow.HEADERS, rows, 2);
        check(3 == Currency.of("BHD").decimalPlaces());
    }

    static void amounts() throws EranosException {
        Money total = eur("12.34").plus(eur("0.66"));
        Money change = eur("5.00").minus(eur("7.50"));
        Optional<EranosException> mixed = refusal(() -> eur("1.00").plus(ron("1.00")));
        Money yen = Money.parse("1500", Currency.of("JPY"));
        Money dinars = new Money(Currency.of("BHD"), 1_234);
        Money nothing = new Money(Currency.of("RON"), 0);
        Cost cost = new Cost.Converted(eur("10.00"), ExchangeRate.parse("4.9765"));
        Cost doubled = new Cost.Converted(eur("20.00"), rate(cost));
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
        check("1.234 BHD".equals(dinars.toString()));
    }

    static void ratios() throws EranosException {
        Percentage percentage = Percentage.parse("12.5");
        Parts parts = Parts.parse("1.5");
        ExchangeRate rate = ExchangeRate.parse("4.97650");
        System.out.println("Percentages, parts, and rates are exact decimals:");
        printTable(
                ValueRow.HEADERS,
                List.of(
                        new ValueRow(
                                "12.5%",
                                percentage + ", " + percentage.basisPoints() + " basis points"),
                        new ValueRow("333 basis points", Percentage.ofBasisPoints(333).toString()),
                        new ValueRow("33.333%", message(refusal(() -> Percentage.parse("33.333")))),
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
        check(150 == parts.hundredths());
    }

    static void datesAndIdentifiers(Group group, People people) throws EranosException {
        CalendarDate date = CalendarDate.parse("2026-10-01");
        TimeOfDay time = TimeOfDay.parse("22:15");
        MemberId parsed = new MemberId(people.ana().value());
        Optional<MemberId> found = group.member(parsed).flatMap(Member::id);
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
                                message(refusal(() -> CalendarDate.of(29, 2, 2027)))),
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
                        new ValueRow(
                                "not-a-uuid",
                                message(refusal(() -> group.member(new MemberId("not-a-uuid"))))),
                        new ValueRow("pt-pt", language.toString()),
                        new ValueRow("pt_PT", message(refusal(() -> Language.parse("pt_PT"))))),
                2);
        check(1 == date.day() && 10 == date.month() && 2026 == date.year());
        check(22 == time.hour() && 15 == time.minute());
        check(people.ana().equals(parsed));
    }

    static void values(Group group, People people) throws EranosException {
        section("13. Values on their own");
        currencies();
        amounts();
        ratios();
        datesAndIdentifiers(group, people);
    }
}
