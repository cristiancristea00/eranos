// The value types. Every text they show comes from the library, every value an app builds is
// checked by the library's rules, and a value the library never sends still prints something:
// a type that formatted on its own, let a refused value through, or trapped on an impossible
// one would show an app another value than the library holds.

@testable import Eranos
import Testing

/// A currency, an amount, or a check that read or wrote another way than the library's would
/// show an app another amount than the group holds.
@Test
func currenciesAndAmountsReadAndWriteAsTheLibraryDoes() throws {
    let euro = try Currency(code: "EUR")
    #expect("EUR" == euro.code)
    #expect("Euro" == euro.name)
    #expect(978 == euro.numericCode)
    #expect(2 == euro.decimalPlaces)
    #expect("EUR" == euro.description)
    #expect(Currency.all.contains(euro))
    #expect(Currency(known: "EUR") == euro)
    #expect(throws: EranosError.money(.unknownCurrency, detail: "unknown currency code")) {
        try Currency(code: "XYZ")
    }

    let price = try Money(parsing: "12.5", in: euro)
    #expect(price == Money(currency: euro, minorUnits: 1_250))
    #expect("12.50 EUR" == price.description)
    #expect(try price.adding(Money(currency: euro, minorUnits: 50)) == Money(currency: euro, minorUnits: 1_300))
    #expect(try price.subtracting(Money(currency: euro, minorUnits: 50)) == Money(currency: euro, minorUnits: 1_200))
    let lei = try Currency(code: "RON")
    #expect {
        try price.adding(Money(currency: lei, minorUnits: 1))
    } throws: { error in
        guard case .money(.currencyMismatch(left: euro, right: lei), _) = error as? EranosError else {
            return false
        }
        return true
    }
    #expect(throws: EranosError.self) {
        try Money(currency: euro, minorUnits: .max).adding(Money(currency: euro, minorUnits: 1))
    }
    #expect(throws: EranosError.self) {
        try Money(parsing: "twelve", in: euro)
    }
}

/// A rate, a percentage, or a number of parts read or written another way than the library's
/// would divide an expense another way than the group does.
@Test
func ratesPercentagesAndPartsReadAndWriteAsTheLibraryDoes() throws {
    let rate = try ExchangeRate(parsing: "4.970")
    #expect("4.97" == rate.description)
    #expect(rate == ExchangeRate(known: "4.97"))
    let lei = try Currency(code: "RON")
    let euro = try Currency(code: "EUR")
    #expect(try ExchangeRate(parsing: "0.25").convert(Money(currency: lei, minorUnits: 2), into: euro) == Money(currency: euro, minorUnits: 1))
    #expect(throws: EranosError.self) {
        try ExchangeRate(parsing: "0")
    }
    #expect(throws: EranosError.self) {
        try ExchangeRate(parsing: "2").convert(Money(currency: lei, minorUnits: .max), into: euro)
    }

    let percentage = try Percentage(parsing: "12.5 %")
    #expect(1_250 == percentage.basisPoints)
    #expect("12.5%" == percentage.description)
    #expect(try Percentage(basisPoints: 1_250) == percentage)
    #expect(throws: EranosError.self) {
        try Percentage(basisPoints: -1)
    }
    #expect(throws: EranosError.self) {
        try Percentage(parsing: "half")
    }

    let parts = try Parts(parsing: "0.5")
    #expect(50 == parts.hundredths)
    #expect("0.5" == parts.description)
    #expect(try Parts(hundredths: 50) == parts)
    #expect(throws: EranosError.self) {
        try Parts(hundredths: 0)
    }
    #expect(throws: EranosError.self) {
        try Parts(parsing: "half")
    }
}

/// A date or a time read or written another way than the library's, or one that does not exist,
/// would put an entry on another day.
@Test
func datesAndTimesReadAndWriteAsTheLibraryDoes() throws {
    let date = try CalendarDate(day: 23, month: 9, year: 2026)
    #expect("2026-09-23" == date.description)
    #expect(try CalendarDate(parsing: "2026-09-23") == date)
    #expect([date.day, date.month, date.year] == [23, 9, 2026])
    #expect(throws: EranosError.calendar(.invalidDate, detail: "invalid date: expected an existing ISO 8601 date between years 1 and 9999")) {
        try CalendarDate(day: 30, month: 2, year: 2026)
    }
    #expect(throws: EranosError.self) {
        try CalendarDate(parsing: "2026-02-30")
    }

    let time = try TimeOfDay(hour: 18, minute: 30)
    #expect("18:30" == time.description)
    #expect(try TimeOfDay(parsing: "18:30") == time)
    #expect([time.hour, time.minute] == [18, 30])
    #expect(throws: EranosError.self) {
        try TimeOfDay(hour: 24, minute: 0)
    }
    #expect(throws: EranosError.self) {
        try TimeOfDay(parsing: "25:00")
    }
}

/// Contact and account details kept in another form than the library's, or refused ones let
/// through, would reach the group unchecked.
@Test
func contactAndAccountDetailsReadAsTheLibraryDoes() throws {
    #expect(try "ana@example.com" == Email(" ana@example.com ").description)
    #expect(throws: EranosError.self) {
        try Email("ana")
    }
    let phone = try PhoneNumber("+40 712 345 678")
    #expect("+40712345678" == phone.e164)
    #expect("+40 712 345 678" == phone.description)
    #expect(throws: EranosError.self) {
        try PhoneNumber("0712345678")
    }
    let iban = try IBAN("ro49 aaaa 1b31 0075 9384 0000")
    #expect("RO49AAAA1B31007593840000" == iban.description)
    #expect("RO" == iban.countryCode)
    #expect(throws: EranosError.account(.invalidIBAN(.invalidChecksum), detail: "invalid IBAN: the check digits do not match")) {
        try IBAN("RO49AAAA1B31007593840001")
    }
    #expect(try "ana.pays" == Username(" ana.pays ").description)
    #expect(throws: EranosError.self) {
        try Username(" ")
    }
    #expect(try "pt-PT" == Language("pt-pt").description)
    #expect(throws: EranosError.language(.invalidLanguage, detail: "invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`")) {
        try Language("not a tag")
    }
}

/// Identifiers of different entities that compared, or printed, as each other's text would let
/// an app store one and read back another.
@Test
func anIdentifierIsItsText() {
    let id = Member.ID(rawValue: "0192f1c3-0000-7000-8000-000000000001")
    #expect("0192f1c3-0000-7000-8000-000000000001" == id.rawValue)
    #expect(id.description == id.rawValue)
    #expect(id == Member.ID(rawValue: id.rawValue))
    #expect(id != Member.ID(rawValue: "other"))
}
