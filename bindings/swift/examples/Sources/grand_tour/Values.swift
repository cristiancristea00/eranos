// Section 13 of the tour: currencies, amounts, percentages, parts, rates, quantities, dates, times, identifiers, and
// language tags, on their own.

import Eranos
import Report

func currencies() throws(EranosError) {
    var rows: [CurrencyRow] = []
    for code in ["EUR", "RON", "JPY", "BHD"] {
        let currency = try CurrencyCode(code: code)
        rows.append(CurrencyRow(code: currency.code, name: currency.name, numericCode: currency.numericCode, decimalPlaces: currency.decimalPlaces))
    }
    let unknown = Result { () throws(EranosError) in try CurrencyCode(code: "XYZ") }
    print("""
        Currencies are the \(CurrencyCode.all.count) active ISO 4217 codes with a minor unit; an unknown code is
        refused (\(message(unknown))):
        """)
    printTable(rows, numbersFrom: 2)
    expect(3, try CurrencyCode(code: "BHD").decimalPlaces, "the dinar's decimal places")
}

func amounts() throws(EranosError) {
    let total = try eur("12.34").adding(eur("0.66"))
    let change = try eur("5.00").subtracting(eur("7.50"))
    let mixed = Result { () throws(EranosError) in try eur("1.00").adding(ron("1.00")) }
    let yen = try Money(parsing: "1500", in: CurrencyCode(code: "JPY"))
    let dinars = Money(minorUnits: 1_234, currency: try CurrencyCode(code: "BHD"))
    let nothing = Money(minorUnits: 0, currency: try CurrencyCode(code: "RON"))
    let cost = Cost.converted(try eur("10.00"), rate: try ExchangeRate(parsing: "4.9765"))
    let doubled = Cost.converted(try eur("20.00"), rate: try rate(of: cost))
    // The double nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
    let fromDouble = try Money(amount: 1.005, currency: CurrencyCode(code: "EUR"))
    let notANumber = Result { () throws(EranosError) in try Money(amount: .nan, currency: CurrencyCode(code: "EUR")) }
    print("Amounts are exact minor units in their currency:")
    printTable(
        [
            ValueRow(input: "12.34 EUR + 0.66 EUR", result: "\(total)"),
            ValueRow(input: "5.00 EUR − 7.50 EUR", result: "\(change), negative: \(0 > change.minorUnits)"),
            ValueRow(input: "1.00 EUR + 1.00 RON", result: message(mixed)),
            ValueRow(input: "1.005 EUR", result: message(Result { () throws(EranosError) in try eur("1.005") })),
            ValueRow(input: "1.005 as a double, in EUR", result: "\(fromDouble)"),
            ValueRow(input: "NaN as a double, in EUR", result: message(notANumber)),
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
    expect(try eur("1.01"), fromDouble, "the amount of 1.005 as a double")
    guard case .failure = notANumber else {
        preconditionFailure("NaN is no amount")
    }
    expect("1.234 BHD", dinars.description, "the dinars")
}

func ratios() throws(EranosError) {
    let percentage = try Percentage(parsing: "12.5")
    let parts = try Parts(parsing: "1.5")
    let rate = try ExchangeRate(parsing: "4.97650")
    // 0.30000000000000004 as a double, which rounds to two decimal places.
    let sum = try Percentage(0.1 + 0.2)
    print("Percentages, parts, and rates are exact decimals:")
    printTable(
        [
            ValueRow(input: "12.5%", result: "\(percentage), \(percentage.basisPoints) basis points"),
            ValueRow(input: "333 basis points", result: "\(try Percentage(basisPoints: 333))"),
            ValueRow(input: "33.333%", result: message(Result { () throws(EranosError) in try Percentage(parsing: "33.333") })),
            ValueRow(input: "0.1 + 0.2 as a double, a percentage", result: "\(sum)"),
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
    precondition(30 == sum.basisPoints, "the percentage of 0.1 + 0.2 in basis points")
    precondition(150 == parts.hundredths, "the parts in hundredths")
}

func quantities(_ people: People) throws(EranosError) {
    let weight = try Quantity(parsing: "0.452")
    // 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
    let olives = ExpenseItem(name: "Olives", unitPrice: try eur("1.00"), split: .equal([people.ana]), quantity: try Quantity(parsing: "0.335"))
    let olivesTotal = try olives.lineTotal()
    let fiveDecimals = Result { () throws(EranosError) in try Quantity(parsing: "0.00005") }
    let zero = Result { () throws(EranosError) in try Quantity(parsing: "0") }
    print("Quantities have four decimal places; line totals round half away from zero:")
    printTable(
        [
            ValueRow(input: "a quantity of 0.452", result: "\(weight), \(weight.tenThousandths) ten-thousandths"),
            ValueRow(input: "a quantity of 0.00005", result: message(fiveDecimals)),
            ValueRow(input: "a quantity of 0", result: message(zero)),
            ValueRow(input: "0.335 at 1.00 EUR", result: "\(olivesTotal)")
        ],
        numbersFrom: 2
    )
    precondition(4_520 == weight.tenThousandths, "the quantity in ten-thousandths")
    guard case .failure(.money(.invalidQuantity, _)) = fiveDecimals else {
        preconditionFailure("a quantity of five decimal places is refused")
    }
    guard case .failure(.money(.invalidQuantity, _)) = zero else {
        preconditionFailure("a quantity of zero is refused")
    }
    expect(try eur("0.34"), olivesTotal, "the line total of 0.335 at 1.00 EUR")
}

func datesAndIdentifiers(_ group: ExpenseGroup, _ people: People) throws(EranosError) {
    let date = try CalendarDate(parsing: "2026-10-01")
    let time = try TimeOfDay(parsing: "22:15")
    let parsed = Member.ID(rawValue: people.ana.rawValue)
    let found = try group.member(withID: parsed)?.id
    let empty = Result { () throws(EranosError) in try group.member(withID: Member.ID(rawValue: "")) }
    let spaced = Result { () throws(EranosError) in try group.member(withID: Member.ID(rawValue: " ana")) }
    let tabbed = Result { () throws(EranosError) in try group.member(withID: Member.ID(rawValue: "a\tb")) }
    let long = Result { () throws(EranosError) in try group.member(withID: Member.ID(rawValue: String(repeating: "a", count: 65))) }
    let language = try Language("pt-pt")
    print("Dates, times, identifiers, and language tags read and print the ISO forms:")
    printTable(
        [
            ValueRow(input: "2026-10-01", result: "\(date): day \(date.day), month \(date.month), year \(date.year)"),
            ValueRow(input: "29 February 2027", result: message(Result { () throws(EranosError) in try CalendarDate(year: 2027, month: 2, day: 29) })),
            ValueRow(input: "22:15", result: "\(time): hour \(time.hour), minute \(time.minute)"),
            ValueRow(input: "24:00", result: message(Result { () throws(EranosError) in try TimeOfDay(hour: 24, minute: 0) })),
            ValueRow(input: "Ana's identifier", result: "\(parsed), the same after a round trip: \(people.ana == found)"),
            ValueRow(input: "empty text", result: message(empty)),
            ValueRow(input: "ana after a space", result: message(spaced)),
            ValueRow(input: "a tab between a and b", result: message(tabbed)),
            ValueRow(input: "65 letters a", result: message(long)),
            ValueRow(input: "pt-pt", result: "\(language)"),
            ValueRow(input: "pt_PT", result: message(Result { () throws(EranosError) in try Language("pt_PT") }))
        ],
        numbersFrom: 2
    )
    precondition(1 == date.day && 10 == date.month && 2026 == date.year, "the date's parts")
    precondition(22 == time.hour && 15 == time.minute, "the time's parts")
    precondition(people.ana == parsed, "the identifier survives the round trip")
    guard case .failure(.id(.empty, _)) = empty else {
        preconditionFailure("empty text is refused")
    }
    guard case .failure(.id(.surroundingWhitespace, _)) = spaced else {
        preconditionFailure("text after a space is refused")
    }
    guard case .failure(.id(.controlCharacter, _)) = tabbed else {
        preconditionFailure("text with a tab is refused")
    }
    guard case .failure(.id(.tooLong, _)) = long else {
        preconditionFailure("text of 65 bytes is refused")
    }
}

func values(_ group: ExpenseGroup, _ people: People) throws(EranosError) {
    section("13. Values on their own")
    try currencies()
    try amounts()
    try ratios()
    try quantities(people)
    try datesAndIdentifiers(group, people)
}
