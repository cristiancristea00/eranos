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
import eranos.examples.Row
import eranos.examples.printTable
import eranos.examples.section

private data class CurrencyRow(
    val code: String,
    val decimalPlaces: Int,
    val name: String,
    val numericCode: Int,
) : Row {
    override val cells: List<String>
        get() = listOf(code, name, "$numericCode", "$decimalPlaces")

    companion object {
        val HEADERS = listOf("Code", "Name", "Numeric code", "Decimal places")
    }
}

private data class ValueRow(
    val input: String,
    val result: String,
) : Row {
    override val cells: List<String>
        get() = listOf(input, result)

    companion object {
        val HEADERS = listOf("Input", "Result")
    }
}

internal fun currencies() {
    val rows =
        listOf("EUR", "RON", "JPY", "BHD").map { code ->
            val currency = Currency.of(code)
            CurrencyRow(code = currency.code, decimalPlaces = currency.decimalPlaces, name = currency.name, numericCode = currency.numericCode)
        }
    val unknown = refusal { Currency.of("XYZ") }
    println(
        """
        Currencies are the ${Currency.all.size} active ISO 4217 codes with a minor unit; an unknown code is
        refused (${message(unknown)}):
        """.trimIndent(),
    )
    printTable(CurrencyRow.HEADERS, rows, numbersFrom = 2)
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
        ValueRow.HEADERS,
        listOf(
            ValueRow("12.34 EUR + 0.66 EUR", "$total"),
            ValueRow("5.00 EUR − 7.50 EUR", "$change, negative: ${0L > change.minorUnits}"),
            ValueRow("1.00 EUR + 1.00 RON", message(mixed)),
            ValueRow("1.005 EUR", message(refusal { eur("1.005") })),
            ValueRow("1500 JPY", "$yen, ${yen.minorUnits} minor units"),
            ValueRow("1234 minor units of BHD", "$dinars"),
            ValueRow("zero RON", "$nothing, zero: ${0L == nothing.minorUnits}"),
            ValueRow("10.00 EUR at 4.9765", "${cost.amount} at ${rate(cost)}, then ${doubled.amount} at the same rate"),
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
        ValueRow.HEADERS,
        listOf(
            ValueRow("12.5%", "$percentage, ${percentage.basisPoints} basis points"),
            ValueRow("333 basis points", "${Percentage.ofBasisPoints(333)}"),
            ValueRow("33.333%", message(refusal { Percentage.parse("33.333") })),
            ValueRow("1.5 parts", "$parts, ${parts.hundredths} hundredths"),
            ValueRow("3 whole parts", "${Parts.ofHundredths(300)}"),
            ValueRow("25 hundredths of a part", "${Parts.ofHundredths(25)}"),
            ValueRow("0 parts", message(refusal { Parts.parse("0") })),
            ValueRow("a rate of 4.97650", "$rate"),
            ValueRow("a rate of 0", message(refusal { ExchangeRate.parse("0") })),
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
        ValueRow.HEADERS,
        listOf(
            ValueRow("2026-10-01", "$date: day ${date.day}, month ${date.month}, year ${date.year}"),
            ValueRow("29 February 2027", message(refusal { CalendarDate.of(29, 2, 2027) })),
            ValueRow("22:15", "$time: hour ${time.hour}, minute ${time.minute}"),
            ValueRow("24:00", message(refusal { TimeOfDay.of(24, 0) })),
            ValueRow("Ana's identifier", "$parsed, the same after a round trip: ${people.ana == found}"),
            ValueRow("not-a-uuid", message(refusal { group.memberBlocking(MemberId("not-a-uuid")) })),
            ValueRow("pt-pt", "$language"),
            ValueRow("pt_PT", message(refusal { Language.parse("pt_PT") })),
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
