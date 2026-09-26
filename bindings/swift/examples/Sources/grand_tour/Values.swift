// Section 13 of the tour: currencies, amounts, percentages, parts, rates, dates, times, identifiers, and language tags,
// on their own.

import Eranos
import Report

func currencies() throws(EranosError) {
    var rows: [[String]] = []
    for code in ["EUR", "RON", "JPY", "BHD"] {
        let currency = try Currency(code: code)
        rows.append([currency.code, currency.name, "\(currency.numericCode)", "\(currency.decimalPlaces)"])
    }
    let unknown = Result { () throws(EranosError) in try Currency(code: "XYZ") }
    print("""
        Currencies are the \(Currency.all.count) active ISO 4217 codes with a minor unit; an unknown code is
        refused (\(message(unknown))):
        """)
    printTable(["Code", "Name", "Numeric code", "Decimal places"], rows, numbersFrom: 2)
    expect(3, try Currency(code: "BHD").decimalPlaces, "the dinar's decimal places")
}

func amounts() throws(EranosError) {
    let total = try eur("12.34").adding(eur("0.66"))
    let change = try eur("5.00").subtracting(eur("7.50"))
    let mixed = Result { () throws(EranosError) in try eur("1.00").adding(ron("1.00")) }
    let yen = try Money(parsing: "1500", in: Currency(code: "JPY"))
    let dinars = Money(currency: try Currency(code: "BHD"), minorUnits: 1_234)
    let nothing = Money(currency: try Currency(code: "RON"), minorUnits: 0)
    let cost = Cost.converted(try eur("10.00"), rate: try ExchangeRate(parsing: "4.9765"))
    let doubled = Cost.converted(try eur("20.00"), rate: try rate(of: cost))
    print("Amounts are exact minor units in their currency:")
    printTable(
        ["Input", "Result"],
        [
            ["12.34 EUR + 0.66 EUR", "\(total)"],
            ["5.00 EUR − 7.50 EUR", "\(change), negative: \(0 > change.minorUnits)"],
            ["1.00 EUR + 1.00 RON", message(mixed)],
            ["1.005 EUR", message(Result { () throws(EranosError) in try eur("1.005") })],
            ["1500 JPY", "\(yen), \(yen.minorUnits) minor units"],
            ["1234 minor units of BHD", "\(dinars)"],
            ["zero RON", "\(nothing), zero: \(0 == nothing.minorUnits)"],
            ["10.00 EUR at 4.9765", "\(cost.amount) at \(try rate(of: cost)), then \(doubled.amount) at the same rate"]
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
        ["Input", "Result"],
        [
            ["12.5%", "\(percentage), \(percentage.basisPoints) basis points"],
            ["333 basis points", "\(try Percentage(basisPoints: 333))"],
            ["33.333%", message(Result { () throws(EranosError) in try Percentage(parsing: "33.333") })],
            ["1.5 parts", "\(parts), \(parts.hundredths) hundredths"],
            ["3 whole parts", "\(try Parts(hundredths: 300))"],
            ["25 hundredths of a part", "\(try Parts(hundredths: 25))"],
            ["0 parts", message(Result { () throws(EranosError) in try Parts(parsing: "0") })],
            ["a rate of 4.97650", "\(rate)"],
            ["a rate of 0", message(Result { () throws(EranosError) in try ExchangeRate(parsing: "0") })]
        ],
        numbersFrom: 2
    )
    precondition(1_250 == percentage.basisPoints, "the percentage in basis points")
    precondition(150 == parts.hundredths, "the parts in hundredths")
}

func datesAndIdentifiers(_ group: Group, _ people: People) throws(EranosError) {
    let date = try CalendarDate(parsing: "2026-10-01")
    let time = try TimeOfDay(parsing: "22:15")
    let parsed = Member.ID(rawValue: people.ana.rawValue)
    let found = try group.member(withID: parsed)?.id
    let language = try Language("pt-pt")
    print("Dates, times, identifiers, and language tags read and print the ISO forms:")
    printTable(
        ["Input", "Result"],
        [
            ["2026-10-01", "\(date): day \(date.day), month \(date.month), year \(date.year)"],
            ["29 February 2027", message(Result { () throws(EranosError) in try CalendarDate(day: 29, month: 2, year: 2027) })],
            ["22:15", "\(time): hour \(time.hour), minute \(time.minute)"],
            ["24:00", message(Result { () throws(EranosError) in try TimeOfDay(hour: 24, minute: 0) })],
            ["Ana's identifier", "\(parsed), the same after a round trip: \(people.ana == found)"],
            ["not-a-uuid", message(Result { () throws(EranosError) in try group.member(withID: Member.ID(rawValue: "not-a-uuid")) })],
            ["pt-pt", "\(language)"],
            ["pt_PT", message(Result { () throws(EranosError) in try Language("pt_PT") })]
        ],
        numbersFrom: 2
    )
    precondition(1 == date.day && 10 == date.month && 2026 == date.year, "the date's parts")
    precondition(22 == time.hour && 15 == time.minute, "the time's parts")
    precondition(people.ana == parsed, "the identifier survives the round trip")
}

func values(_ group: Group, _ people: People) throws(EranosError) {
    section("13. Values on their own")
    try currencies()
    try amounts()
    try ratios()
    try datesAndIdentifiers(group, people)
}
