// Section 13 of the tour: currencies, amounts, percentages, parts, rates, dates, times, identifiers, and language tags,
// on their own.
package eranos.examples.grandtour

import eranos.CalendarDate
import eranos.Cost
import eranos.Currency
import eranos.ExchangeRate
import eranos.Group
import eranos.Language
import eranos.MemberId
import eranos.Money
import eranos.Parts
import eranos.Percentage
import eranos.TimeOfDay
import eranos.examples.printTable
import eranos.examples.section

internal fun currencies() {
    val rows =
        listOf("EUR", "RON", "JPY", "BHD").map { code ->
            val currency = Currency.of(code)
            listOf(currency.code, currency.name, "${currency.numericCode}", "${currency.decimalPlaces}")
        }
    val unknown = refusal { Currency.of("XYZ") }
    println(
        """
        Currencies are the ${Currency.all.size} active ISO 4217 codes with a minor unit; an unknown code is
        refused (${message(unknown)}):
        """.trimIndent(),
    )
    printTable(listOf("Code", "Name", "Numeric code", "Decimal places"), rows, numbersFrom = 2)
    check(3 == Currency.of("BHD").decimalPlaces)
}

internal fun amounts() {
    val total = eur("12.34") + eur("0.66")
    val change = eur("5.00") - eur("7.50")
    val mixed = refusal { eur("1.00") + ron("1.00") }
    val yen = Money.parse("1500", Currency.of("JPY"))
    val dinars = Money(Currency.of("BHD"), 1_234)
    val nothing = Money(Currency.of("RON"), 0)
    val cost = Cost.Converted(eur("10.00"), ExchangeRate.parse("4.9765"))
    val doubled = Cost.Converted(eur("20.00"), rate(cost))
    println("Amounts are exact minor units in their currency:")
    printTable(
        listOf("Input", "Result"),
        listOf(
            listOf("12.34 EUR + 0.66 EUR", "$total"),
            listOf("5.00 EUR − 7.50 EUR", "$change, negative: ${0L > change.minorUnits}"),
            listOf("1.00 EUR + 1.00 RON", message(mixed)),
            listOf("1.005 EUR", message(refusal { eur("1.005") })),
            listOf("1500 JPY", "$yen, ${yen.minorUnits} minor units"),
            listOf("1234 minor units of BHD", "$dinars"),
            listOf("zero RON", "$nothing, zero: ${0L == nothing.minorUnits}"),
            listOf("10.00 EUR at 4.9765", "${cost.amount} at ${rate(cost)}, then ${doubled.amount} at the same rate"),
        ),
        numbersFrom = 2,
    )
    check(eur("13.00") == total)
    check(-250L == change.minorUnits)
    check(null != mixed)
    check("1.234 BHD" == "$dinars")
}

internal fun ratios() {
    val percentage = Percentage.parse("12.5")
    val parts = Parts.parse("1.5")
    val rate = ExchangeRate.parse("4.97650")
    println("Percentages, parts, and rates are exact decimals:")
    printTable(
        listOf("Input", "Result"),
        listOf(
            listOf("12.5%", "$percentage, ${percentage.basisPoints} basis points"),
            listOf("333 basis points", "${Percentage.ofBasisPoints(333)}"),
            listOf("33.333%", message(refusal { Percentage.parse("33.333") })),
            listOf("1.5 parts", "$parts, ${parts.hundredths} hundredths"),
            listOf("3 whole parts", "${Parts.ofHundredths(300)}"),
            listOf("25 hundredths of a part", "${Parts.ofHundredths(25)}"),
            listOf("0 parts", message(refusal { Parts.parse("0") })),
            listOf("a rate of 4.97650", "$rate"),
            listOf("a rate of 0", message(refusal { ExchangeRate.parse("0") })),
        ),
        numbersFrom = 2,
    )
    check(1_250 == percentage.basisPoints)
    check(150 == parts.hundredths)
}

internal fun datesAndIdentifiers(
    group: Group,
    people: People,
) {
    val date = CalendarDate.parse("2026-10-01")
    val time = TimeOfDay.parse("22:15")
    val parsed = MemberId(people.ana.value)
    val found = group.memberBlocking(parsed)?.id
    val language = Language.parse("pt-pt")
    println("Dates, times, identifiers, and language tags read and print the ISO forms:")
    printTable(
        listOf("Input", "Result"),
        listOf(
            listOf("2026-10-01", "$date: day ${date.day}, month ${date.month}, year ${date.year}"),
            listOf("29 February 2027", message(refusal { CalendarDate.of(29, 2, 2027) })),
            listOf("22:15", "$time: hour ${time.hour}, minute ${time.minute}"),
            listOf("24:00", message(refusal { TimeOfDay.of(24, 0) })),
            listOf("Ana's identifier", "$parsed, the same after a round trip: ${people.ana == found}"),
            listOf("not-a-uuid", message(refusal { group.memberBlocking(MemberId("not-a-uuid")) })),
            listOf("pt-pt", "$language"),
            listOf("pt_PT", message(refusal { Language.parse("pt_PT") })),
        ),
        numbersFrom = 2,
    )
    check(1 == date.day && 10 == date.month && 2026 == date.year)
    check(22 == time.hour && 15 == time.minute)
    check(people.ana == parsed)
}

internal fun values(
    group: Group,
    people: People,
) {
    section("13. Values on their own")
    currencies()
    amounts()
    ratios()
    datesAndIdentifiers(group, people)
}
