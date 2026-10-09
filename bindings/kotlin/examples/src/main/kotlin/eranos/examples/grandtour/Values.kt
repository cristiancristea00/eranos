// Section 13 of the tour: currencies, amounts, percentages, parts, rates, quantities, dates, times, identifiers, and
// language tags, on their own.
package eranos.examples.grandtour

import eranos.CalendarDate
import eranos.Cost
import eranos.CurrencyCode
import eranos.EranosException
import eranos.ExchangeRate
import eranos.ExpenseGroup
import eranos.ExpenseItem
import eranos.IdFailure
import eranos.Language
import eranos.MemberId
import eranos.Money
import eranos.MoneyFailure
import eranos.Parts
import eranos.Percentage
import eranos.Quantity
import eranos.Split
import eranos.TimeOfDay
import eranos.examples.Row
import eranos.examples.printTable
import eranos.examples.section

private data class CurrencyRow(
    val code: String,
    val name: String,
    val numericCode: Int,
    val decimalPlaces: Int,
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
            val currency = CurrencyCode.of(code)
            CurrencyRow(code = currency.code, name = currency.name, numericCode = currency.numericCode, decimalPlaces = currency.decimalPlaces)
        }
    val unknown = refusal { CurrencyCode.of("XYZ") }
    println(
        """
        Currencies are the ${CurrencyCode.all.size} active ISO 4217 codes with a minor unit; an unknown code is
        refused (${message(unknown)}):
        """.trimIndent(),
    )
    printTable(CurrencyRow.HEADERS, rows, numbersFrom = 2)
    check(3 == CurrencyCode.of("BHD").decimalPlaces)
}

internal fun amounts() {
    val total = eur("12.34") + eur("0.66")
    val change = eur("5.00") - eur("7.50")
    val mixed = refusal { eur("1.00") + ron("1.00") }
    val yen = Money.parse("1500", CurrencyCode.of("JPY"))
    val dinars = Money(1_234, CurrencyCode.of("BHD"))
    val nothing = Money.zero(CurrencyCode.of("RON"))
    val cost = Cost.Converted(eur("10.00"), ExchangeRate.parse("4.9765"))
    val doubled = Cost.Converted(eur("20.00"), rate(cost))
    // The double nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
    val fromDouble = Money.fromDouble(1.005, CurrencyCode.of("EUR"))
    val notANumber = refusal { Money.fromDouble(Double.NaN, CurrencyCode.of("EUR")) }
    println("Amounts are exact minor units in their currency:")
    printTable(
        ValueRow.HEADERS,
        listOf(
            ValueRow("12.34 EUR + 0.66 EUR", "$total"),
            ValueRow("5.00 EUR − 7.50 EUR", "$change, negative: ${0L > change.minorUnits}"),
            ValueRow("1.00 EUR + 1.00 RON", message(mixed)),
            ValueRow("1.005 EUR", message(refusal { eur("1.005") })),
            ValueRow("1.005 as a double, in EUR", "$fromDouble"),
            ValueRow("NaN as a double, in EUR", message(notANumber)),
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
    check(eur("1.01") == fromDouble)
    check(null != notANumber)
    check("1.234 BHD" == "$dinars")
}

internal fun ratios() {
    val percentage = Percentage.parse("12.5")
    val parts = Parts.parse("1.5")
    val rate = ExchangeRate.parse("4.97650")
    // 0.30000000000000004 as a double, which rounds to two decimal places.
    val sum = Percentage.fromDouble(0.1 + 0.2)
    println("Percentages, parts, and rates are exact decimals:")
    printTable(
        ValueRow.HEADERS,
        listOf(
            ValueRow("12.5%", "$percentage, ${percentage.basisPoints} basis points"),
            ValueRow("333 basis points", "${Percentage.ofBasisPoints(333)}"),
            ValueRow("33.333%", message(refusal { Percentage.parse("33.333") })),
            ValueRow("0.1 + 0.2 as a double, a percentage", "$sum"),
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
    check(30 == sum.basisPoints)
    check(150 == parts.hundredths)
}

internal fun quantities(people: People) {
    val weight = Quantity.parse("0.452")
    // 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
    val olives = ExpenseItem("Olives", eur("1.00"), Split.Equal(listOf(people.ana)), Quantity.parse("0.335"))
    val fiveDecimals = refusal { Quantity.parse("0.00005") }
    val zero = refusal { Quantity.parse("0") }
    println("Quantities have four decimal places; line totals round half away from zero:")
    printTable(
        ValueRow.HEADERS,
        listOf(
            ValueRow("a quantity of 0.452", "$weight, ${weight.tenThousandths} ten-thousandths"),
            ValueRow("a quantity of 0.00005", message(fiveDecimals)),
            ValueRow("a quantity of 0", message(zero)),
            ValueRow("0.335 at 1.00 EUR", "${olives.lineTotal()}"),
        ),
        numbersFrom = 2,
    )
    check(4_520L == weight.tenThousandths)
    check(
        listOf(MoneyFailure.InvalidQuantity, MoneyFailure.InvalidQuantity) ==
            listOf(fiveDecimals, zero).map { error -> (error as? EranosException.Money)?.failure },
    )
    check(eur("0.34") == olives.lineTotal())
}

internal fun datesAndIdentifiers(
    group: ExpenseGroup,
    people: People,
) {
    val date = CalendarDate.parse("2026-10-01")
    val time = TimeOfDay.parse("22:15")
    val parsed = MemberId(people.ana.value)
    val found = group.memberBlocking(parsed)?.id
    val empty = refusal { group.memberBlocking(MemberId("")) }
    val spaced = refusal { group.memberBlocking(MemberId(" ana")) }
    val tabbed = refusal { group.memberBlocking(MemberId("a\tb")) }
    val tooLong = refusal { group.memberBlocking(MemberId("a".repeat(65))) }
    val language = Language.parse("pt-pt")
    println("Dates, times, identifiers, and language tags read and print the ISO forms:")
    printTable(
        ValueRow.HEADERS,
        listOf(
            ValueRow("2026-10-01", "$date: day ${date.day}, month ${date.month}, year ${date.year}"),
            ValueRow("29 February 2027", message(refusal { CalendarDate.of(2027, 2, 29) })),
            ValueRow("22:15", "$time: hour ${time.hour}, minute ${time.minute}"),
            ValueRow("24:00", message(refusal { TimeOfDay.of(24, 0) })),
            ValueRow("Ana's identifier", "$parsed, the same after a round trip: ${people.ana == found}"),
            ValueRow("empty text", message(empty)),
            ValueRow("ana after a space", message(spaced)),
            ValueRow("a tab between a and b", message(tabbed)),
            ValueRow("65 letters a", message(tooLong)),
            ValueRow("pt-pt", "$language"),
            ValueRow("pt_PT", message(refusal { Language.parse("pt_PT") })),
        ),
        numbersFrom = 2,
    )
    check(1 == date.day && 10 == date.month && 2026 == date.year)
    check(22 == time.hour && 15 == time.minute)
    check(people.ana == parsed)
    check(
        listOf(IdFailure.Empty, IdFailure.SurroundingWhitespace, IdFailure.ControlCharacter, IdFailure.TooLong) ==
            listOf(empty, spaced, tabbed, tooLong).map { error -> (error as? EranosException.Id)?.failure },
    )
}

internal fun values(
    group: ExpenseGroup,
    people: People,
) {
    section("13. Values on their own")
    currencies()
    amounts()
    ratios()
    quantities(people)
    datesAndIdentifiers(group, people)
}
