// What the layer adds for apps on Apple's platforms: entries an app can identify and encode, values
// that order as the library orders them, whole numbers as `Int`, and amounts, days, and times in
// Foundation's types. Each crosses through the library's own checks, so none lets through a value
// the library refuses.

import Foundation
import Testing

@testable import Eranos

/// The value of `Value` that the JSON `text` holds.
private func decoded<Value: Decodable>(_: Value.Type, _ text: String) throws -> Value {
    try JSONDecoder().decode(Value.self, from: Data(text.utf8))
}

/// The library's refusal that decoding the JSON `text` as a `Value` reports, or nil for text it
/// decodes.
private func refusal<Value: Decodable>(_: Value.Type, _ text: String) -> EranosError? {
    do {
        _ = try decoded(Value.self, text)
        return nil
    } catch let DecodingError.dataCorrupted(context) {
        return context.underlyingError as? EranosError
    } catch {
        return nil
    }
}

/// The JSON `value` encodes as, with its keys sorted.
private func encoded(_ value: some Encodable) throws -> String {
    let encoder = JSONEncoder()
    encoder.outputFormatting = .sortedKeys
    return String(decoding: try encoder.encode(value), as: UTF8.self)
}

/// The library's error that `body` throws, or nil when it returns.
private func thrown(_ body: () throws -> some Any) -> EranosError? {
    do {
        _ = try body()
        return nil
    } catch {
        return error as? EranosError
    }
}

/// An entry an app cached or handed to a navigation stack that came back with a field changed or
/// dropped would show it another entry than the group holds.
@Test
func everyEntryEncodesAndDecodesWhole() throws {
    let flat = try Flat()
    let euro = flat.euro
    let lei = try CurrencyCode(code: "RON")
    let carol = Member(
        firstName: "Carol",
        lastName: "Popa",
        email: try Email("carol@example.com"),
        phone: try PhoneNumber("+40712345678"),
        accounts: [
            PaymentAccount(name: "Bank", details: .iban(try IBAN("RO49AAAA1B31007593840000"))),
            PaymentAccount(name: "App", details: .username(try Username("carol")))
        ]
    )
    let receipt = Expense(
        name: "Lunch",
        cost: .converted(Money(minorUnits: 5_000, currency: lei), rate: try ExchangeRate(parsing: "0.2")),
        payers: .parts([MemberParts(member: flat.ana, parts: try Parts(hundredths: 150))]),
        split: .itemised([
            ExpenseItem(
                name: "Soup",
                unitPrice: Money(minorUnits: 1_250, currency: lei),
                split: .percentages([MemberPercentage(member: flat.bob, percentage: try Percentage(basisPoints: 10_000))]),
                quantity: try Quantity(parsing: "2"),
                discount: Discount(value: .percentage(try Percentage(basisPoints: 1_000)), scope: .unit)
            ),
            ExpenseItem(
                name: "Bread",
                unitPrice: Money(minorUnits: 2_500, currency: lei),
                split: .amounts([MemberAmount(member: flat.ana, amount: Money(minorUnits: 2_500, currency: lei))]),
                discount: Discount(value: .fixed(Money(minorUnits: 0, currency: lei)))
            )
        ]),
        date: flat.day,
        tip: TipRule(value: .percentage(try Percentage(basisPoints: 500)), distribution: .equal),
        time: try TimeOfDay(hour: 13, minute: 5),
        category: flat.food,
        note: "Out"
    )
    let encoder = JSONEncoder()
    let decoder = JSONDecoder()
    #expect(try decoder.decode(Member.self, from: encoder.encode(carol)) == carol)
    #expect(try decoder.decode(Expense.self, from: encoder.encode(receipt)) == receipt)
    let members = flat.group.members()
    #expect(try decoder.decode([Member].self, from: encoder.encode(members)) == members)
    let categories = flat.group.categories()
    let decodedCategories = try decoder.decode([Eranos.Category].self, from: encoder.encode(categories))
    #expect(decodedCategories == categories)
    let expenses = flat.group.expenses()
    #expect(try decoder.decode([Expense].self, from: encoder.encode(expenses)) == expenses)
    let transfers = flat.group.transfers()
    #expect(try decoder.decode([Transfer].self, from: encoder.encode(transfers)) == transfers)
    #expect(expenses.map(\.id) == [flat.dinner])
    let cent = try decoded(Money.self, #"{"currency":"EUR","minorUnits":1}"#)
    #expect(Money(minorUnits: 1, currency: euro) == cent)
}

/// A value an app encodes in another shape than its own text or numbers would read back
/// otherwise in another app; one encoded with its identifier's text checked would refuse an
/// identifier the app stored before the rules changed.
@Test
func aValueEncodesAsItsTextOrItsNumbers() throws {
    #expect(try #""EUR""# == encoded(CurrencyCode(code: "EUR")))
    #expect(try "1250" == encoded(Percentage(basisPoints: 1_250)))
    #expect(try "50" == encoded(Parts(hundredths: 50)))
    #expect(try "15000" == encoded(Quantity(parsing: "1.5")))
    #expect(try #""4.97""# == encoded(ExchangeRate(parsing: "4.97")))
    #expect(try #""ana@example.com""# == encoded(Email("ana@example.com")))
    #expect(try #""pt-PT""# == encoded(Language("pt-pt")))
    #expect(try #""ana.pays""# == encoded(Username("ana.pays")))
    #expect(try #"{"day":23,"month":9,"year":2026}"# == encoded(CalendarDate(year: 2026, month: 9, day: 23)))
    #expect(try #"{"hour":18,"minute":30}"# == encoded(TimeOfDay(hour: 18, minute: 30)))
    #expect(try #"{"e164":"+40712345678","international":"+40 712 345 678"}"# == encoded(PhoneNumber("+40712345678")))
    #expect(try #"{"countryCode":"RO","text":"RO49AAAA1B31007593840000"}"# == encoded(IBAN("RO49AAAA1B31007593840000")))
    #expect(try #"" nobody""# == encoded(Member.ID(rawValue: " nobody")))
    #expect(try Member.ID(rawValue: " nobody") == decoded(Member.ID.self, #"" nobody""#))
    #expect(try "ana@example.com" == Email("ana@example.com").text)
    #expect(try "RO49AAAA1B31007593840000" == IBAN("RO49AAAA1B31007593840000").text)
    #expect(try "4.97" == ExchangeRate(parsing: "4.97").text)
}

/// Data an app edited or a newer release wrote that decoded into a value the library refuses
/// would reach the group unchecked; each is refused with the library's own error.
@Test
func dataTheLibraryRefusesDoesNotDecode() throws {
    #expect(refusal(CalendarDate.self, #"{"year":2026,"month":2,"day":30}"#) == thrown { try CalendarDate(year: 2026, month: 2, day: 30) })
    #expect(refusal(TimeOfDay.self, #"{"hour":24,"minute":0}"#) == thrown { try TimeOfDay(hour: 24, minute: 0) })
    #expect(refusal(Percentage.self, "-1") == thrown { try Percentage(basisPoints: -1) })
    #expect(refusal(Parts.self, "0") == thrown { try Parts(hundredths: 0) })
    #expect(refusal(Quantity.self, "0") == thrown { try Quantity(tenThousandths: 0) })
    #expect(refusal(CurrencyCode.self, #""XYZ""#) == thrown { try CurrencyCode(code: "XYZ") })
    #expect(refusal(ExchangeRate.self, #""0""#) == thrown { try ExchangeRate(parsing: "0") })
    #expect(refusal(Email.self, #""ana""#) == thrown { try Email("ana") })
    #expect(refusal(Username.self, #"" ""#) == thrown { try Username(" ") })
    #expect(refusal(Language.self, #""not a tag""#) == thrown { try Language("not a tag") })
    #expect(refusal(PhoneNumber.self, #"{"e164":"0712","international":"0712"}"#) == thrown { try PhoneNumber("0712") })
    #expect(refusal(IBAN.self, #"{"text":"RO49","countryCode":"RO"}"#) == thrown { try IBAN("RO49") })
    #expect(
        refusal(PhoneNumber.self, #"{"e164":"+40712345678","international":"+40712345678"}"#)
            == .mismatchedRecord(.phoneInternational, detail: "the phone number's international form is not the one the library writes for it")
    )
    #expect(
        refusal(IBAN.self, #"{"text":"RO49AAAA1B31007593840000","countryCode":"DE"}"#)
            == .mismatchedRecord(.ibanCountryCode, detail: "the IBAN's country code is not the one the library derives from it")
    )
    #expect(nil != refusal(CalendarDate.self, #"{"year":2026,"month":9,"day":31}"#))
}

/// Values that sorted otherwise than the library sorts them would list days, times, shares, or
/// currencies in another order than every other language.
@Test
func valuesOrderAsTheLibraryOrdersThem() throws {
    #expect(try CalendarDate(year: 2025, month: 12, day: 31) < CalendarDate(year: 2026, month: 1, day: 1))
    #expect(try CalendarDate(year: 2026, month: 9, day: 30) < CalendarDate(year: 2026, month: 10, day: 1))
    #expect(try CalendarDate(year: 2026, month: 9, day: 22) < CalendarDate(year: 2026, month: 9, day: 23))
    #expect(try !(CalendarDate(year: 2026, month: 9, day: 23) < CalendarDate(parsing: "2026-09-23")))
    #expect(try TimeOfDay(hour: 9, minute: 59) < TimeOfDay(hour: 10, minute: 0))
    #expect(try TimeOfDay(hour: 10, minute: 0) < TimeOfDay(hour: 10, minute: 1))
    #expect(try !(TimeOfDay(hour: 10, minute: 1) < TimeOfDay(parsing: "10:01")))
    #expect(try Percentage(basisPoints: 1) < Percentage(basisPoints: 2))
    #expect(try Parts(hundredths: 50) < Parts(hundredths: 100))
    #expect(try Quantity(parsing: "0.5") < Quantity.one)
    let codes = try ["RON", "EUR", "USD", "BHD"].map { code throws(EranosError) in try CurrencyCode(code: code) }
    #expect(["BHD", "EUR", "RON", "USD"] == codes.sorted().map(\.code))
}

/// A whole number beyond what the library holds that stopped the app, or reached the library cut
/// short, would refuse or accept another value than the one the app gave.
@Test
func aNumberBeyondWhatTheLibraryHoldsIsRefusedAsTheLibraryRefusesIt() throws {
    let above = Int(Int32.max) + 1
    let below = Int(Int32.min) - 1
    let invalidDate = thrown { try CalendarDate(year: 0, month: 1, day: 1) }
    #expect(nil != invalidDate)
    #expect(invalidDate == thrown { try CalendarDate(year: above, month: 1, day: 1) })
    #expect(invalidDate == thrown { try CalendarDate(year: 2026, month: above, day: 1) })
    #expect(invalidDate == thrown { try CalendarDate(year: 2026, month: 1, day: below) })
    let invalidTime = thrown { try TimeOfDay(hour: 24, minute: 0) }
    #expect(nil != invalidTime)
    #expect(invalidTime == thrown { try TimeOfDay(hour: above, minute: 0) })
    #expect(invalidTime == thrown { try TimeOfDay(hour: 0, minute: below) })
    let invalidPercentage = thrown { try Percentage(basisPoints: -1) }
    #expect(nil != invalidPercentage)
    #expect(invalidPercentage == thrown { try Percentage(basisPoints: above) })
    let invalidParts = thrown { try Parts(hundredths: 0) }
    #expect(nil != invalidParts)
    #expect(invalidParts == thrown { try Parts(hundredths: above) })
    #expect(try Int(Int32.max) == Percentage(basisPoints: Int(Int32.max)).basisPoints)
}

/// An amount read from a `Decimal` that the layer rounded, or written to one with its digits
/// changed, would differ from the one the user typed.
@Test
func anAmountCrossesToAndFromADecimalExactly() throws {
    let euro = try CurrencyCode(code: "EUR")
    let yen = try CurrencyCode(code: "JPY")
    #expect(try Money(minorUnits: 1_250, currency: euro) == Money(Decimal(sign: .plus, exponent: -1, significand: 125), currency: euro))
    #expect(try Money(minorUnits: -300, currency: euro) == Money(Decimal(-3), currency: euro))
    let precise = thrown { try Money(Decimal(sign: .plus, exponent: -3, significand: 1_234), currency: euro) }
    #expect(nil != precise)
    #expect(precise == thrown { try Money(parsing: "1.234", in: euro) })
    #expect(throws: EranosError.self) {
        try Money(Decimal.nan, currency: euro)
    }
    #expect(Decimal(sign: .minus, exponent: -1, significand: 125) == Money(minorUnits: -1_250, currency: euro).decimalValue)
    #expect(Decimal(500) == Money(minorUnits: 500, currency: yen).decimalValue)
    let largest = Money(minorUnits: .max, currency: euro)
    #expect(try largest == Money(largest.decimalValue, currency: euro))
}

/// A day or a time read from `DateComponents` with a component missing, or written back with
/// another, would put an entry on another day than the user picked.
@Test
func aDayAndATimeCrossToAndFromDateComponents() throws {
    let date = try CalendarDate(year: 2026, month: 9, day: 23)
    #expect(try date == CalendarDate(components: DateComponents(year: 2026, month: 9, day: 23, hour: 18)))
    #expect(DateComponents(year: 2026, month: 9, day: 23) == date.dateComponents)
    #expect(thrown { try CalendarDate(year: 0, month: 1, day: 1) } == thrown { try CalendarDate(components: DateComponents(year: 2026, month: 9)) })
    let time = try TimeOfDay(hour: 18, minute: 30)
    #expect(try time == TimeOfDay(components: DateComponents(hour: 18, minute: 30, second: 12)))
    #expect(DateComponents(hour: 18, minute: 30) == time.dateComponents)
    #expect(thrown { try TimeOfDay(hour: 24, minute: 0) } == thrown { try TimeOfDay(components: DateComponents(hour: 18)) })
}

/// An entry whose identity was not its identifier would make SwiftUI redraw a row as another
/// entry after an edit that kept its identifier.
@Test
func anEntryIsIdentifiedByItsIdentifier() throws {
    let flat = try Flat()
    var dinner = try #require(try flat.group.expense(withID: flat.dinner))
    let identity = dinner.id
    dinner.name = "Supper"
    #expect(identity == dinner.id)
    #expect(flat.group.members().map(\.id) == [flat.ana, flat.bob])
}
