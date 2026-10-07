// Section 13 of the tour: currencies, amounts, percentages, parts, rates, dates, times, identifiers, and language tags,
// on their own.

import Eranos
import Report

func currencies() throws(EranosError) {
    var rows: [CurrencyRow] = []
    for code in ["EUR", "RON", "JPY", "BHD"] {
        let currency = try Currency(code: code)
        rows.append(CurrencyRow(code: currency.code, name: currency.name, numericCode: currency.numericCode, decimalPlaces: currency.decimalPlaces))
    }
    let unknown = Result { () throws(EranosError) in try Currency(code: "XYZ") }
    print("""
        Currencies are the \(Currency.all.count) active ISO 4217 codes with a minor unit; an unknown code is
        refused (\(message(unknown))):
        """)
    printTable(rows, numbersFrom: 2)
    expect(3, try Currency(code: "BHD").decimalPlaces, "the dinar's decimal places")
}

func amounts() throws(EranosError) {
    let total = try eur("12.34").adding(eur("0.66"))
    let change = try eur("5.00").subtracting(eur("7.50"))
    let mixed = Result { () throws(EranosError) in try eur("1.00").adding(ron("1.00")) }
    let yen = try Money(parsing: "1500", in: Currency(code: "JPY"))
    let dinars = Money(minorUnits: 1_234, currency: try Currency(code: "BHD"))
    let nothing = Money(minorUnits: 0, currency: try Currency(code: "RON"))
    let cost = Cost.converted(try eur("10.00"), rate: try ExchangeRate(parsing: "4.9765"))
    let doubled = Cost.converted(try eur("20.00"), rate: try rate(of: cost))
    print("Amounts are exact minor units in their currency:")
    printTable(
        [
            ValueRow(input: "12.34 EUR + 0.66 EUR", result: "\(total)"),
            ValueRow(input: "5.00 EUR − 7.50 EUR", result: "\(change), negative: \(0 > change.minorUnits)"),
            ValueRow(input: "1.00 EUR + 1.00 RON", result: message(mixed)),
            ValueRow(input: "1.005 EUR", result: message(Result { () throws(EranosError) in try eur("1.005") })),
            ValueRow(input: "1500 JPY", result: "\(yen), \(yen.minorUnits) minor units"),
            ValueRow(input: "1234 minor units of BHD", result: "\(dinars)"),
            ValueRow(input: "zero RON", result: "\(nothing), zero: \(0 == nothing.minorUnits)"),
            ValueRow(input: "10.00 EUR at 4.9765", result: "\(cost.amount) at \(try rate(of: cost)), then \(doubled.amount) at the same rate")
        ],
        numbersFrom: 2
    )
    expect(try eur("13.00"), total, "the sum")
    precondition(-250 == change.minorUnits, "the difference")
    guard case .failure = mixed else {
        preconditionFailure("amounts in two currencies do not add up")
    }
    expect("1.234 BHD", dinars.description, "the dinars")
}

func ratios() throws(EranosError) {
    let percentage = try Percentage(parsing: "12.5")
    let parts = try Parts(parsing: "1.5")
    let rate = try ExchangeRate(parsing: "4.97650")
    print("Percentages, parts, and rates are exact decimals:")
    printTable(
        [
            ValueRow(input: "12.5%", result: "\(percentage), \(percentage.basisPoints) basis points"),
            ValueRow(input: "333 basis points", result: "\(try Percentage(basisPoints: 333))"),
            ValueRow(input: "33.333%", result: message(Result { () throws(EranosError) in try Percentage(parsing: "33.333") })),
            ValueRow(input: "1.5 parts", result: "\(parts), \(parts.hundredths) hundredths"),
            ValueRow(input: "3 whole parts", result: "\(try Parts(hundredths: 300))"),
            ValueRow(input: "25 hundredths of a part", result: "\(try Parts(hundredths: 25))"),
            ValueRow(input: "0 parts", result: message(Result { () throws(EranosError) in try Parts(parsing: "0") })),
            ValueRow(input: "a rate of 4.97650", result: "\(rate)"),
            ValueRow(input: "a rate of 0", result: message(Result { () throws(EranosError) in try ExchangeRate(parsing: "0") }))
        ],
        numbersFrom: 2
    )
    precondition(1_250 == percentage.basisPoints, "the percentage in basis points")
    precondition(150 == parts.hundredths, "the parts in hundredths")
}

func datesAndIdentifiers(_ group: ExpenseGroup, _ people: People) throws(EranosError) {
    let date = try CalendarDate(parsing: "2026-10-01")
    let time = try TimeOfDay(parsing: "22:15")
    let parsed = Member.ID(rawValue: people.ana.rawValue)
    let found = try group.member(withID: parsed)?.id
    let language = try Language("pt-pt")
    print("Dates, times, identifiers, and language tags read and print the ISO forms:")
    printTable(
        [
            ValueRow(input: "2026-10-01", result: "\(date): day \(date.day), month \(date.month), year \(date.year)"),
            ValueRow(input: "29 February 2027", result: message(Result { () throws(EranosError) in try CalendarDate(year: 2027, month: 2, day: 29) })),
            ValueRow(input: "22:15", result: "\(time): hour \(time.hour), minute \(time.minute)"),
            ValueRow(input: "24:00", result: message(Result { () throws(EranosError) in try TimeOfDay(hour: 24, minute: 0) })),
            ValueRow(input: "Ana's identifier", result: "\(parsed), the same after a round trip: \(people.ana == found)"),
            ValueRow(input: "not-a-uuid", result: message(Result { () throws(EranosError) in try group.member(withID: Member.ID(rawValue: "not-a-uuid")) })),
            ValueRow(input: "pt-pt", result: "\(language)"),
            ValueRow(input: "pt_PT", result: message(Result { () throws(EranosError) in try Language("pt_PT") }))
        ],
        numbersFrom: 2
    )
    precondition(1 == date.day && 10 == date.month && 2026 == date.year, "the date's parts")
    precondition(22 == time.hour && 15 == time.minute, "the time's parts")
    precondition(people.ana == parsed, "the identifier survives the round trip")
}

func values(_ group: ExpenseGroup, _ people: People) throws(EranosError) {
    section("13. Values on their own")
    try currencies()
    try amounts()
    try ratios()
    try datesAndIdentifiers(group, people)
}
