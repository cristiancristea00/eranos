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
    let euro = try CurrencyCode(code: "EUR")
    #expect("EUR" == euro.code)
    #expect("Euro" == euro.name)
    #expect(978 == euro.numericCode)
    #expect(2 == euro.decimalPlaces)
    #expect("EUR" == euro.description)
    #expect(CurrencyCode.all.contains(euro))
    #expect(CurrencyCode(known: "EUR") == euro)
    #expect(throws: EranosError.money(.unknownCurrency, detail: "unknown currency code")) {
        try CurrencyCode(code: "XYZ")
    }

    let price = try Money(parsing: "12.5", in: euro)
    #expect(price == Money(minorUnits: 1_250, currency: euro))
    #expect("12.50 EUR" == price.description)
    #expect(try price.adding(Money(minorUnits: 50, currency: euro)) == Money(minorUnits: 1_300, currency: euro))
    #expect(try price.subtracting(Money(minorUnits: 50, currency: euro)) == Money(minorUnits: 1_200, currency: euro))
    let lei = try CurrencyCode(code: "RON")
    #expect {
        try price.adding(Money(minorUnits: 1, currency: lei))
    } throws: { error in
        guard case .money(.currencyMismatch(left: euro, right: lei), _) = error as? EranosError else {
            return false
        }
        return true
    }
    #expect(throws: EranosError.self) {
        try Money(minorUnits: .max, currency: euro).adding(Money(minorUnits: 1, currency: euro))
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
    let lei = try CurrencyCode(code: "RON")
    let euro = try CurrencyCode(code: "EUR")
    #expect(try ExchangeRate(parsing: "0.25").convert(Money(minorUnits: 2, currency: lei), into: euro) == Money(minorUnits: 1, currency: euro))
    #expect(throws: EranosError.self) {
        try ExchangeRate(parsing: "0")
    }
    #expect(throws: EranosError.self) {
        try ExchangeRate(parsing: "2").convert(Money(minorUnits: .max, currency: lei), into: euro)
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
    let date = try CalendarDate(year: 2026, month: 9, day: 23)
    #expect("2026-09-23" == date.description)
    #expect(try CalendarDate(parsing: "2026-09-23") == date)
    #expect(2026 == date.year)
    #expect(9 == date.month)
    #expect(23 == date.day)
    #expect(throws: EranosError.calendar(.invalidDate, detail: "invalid date: expected an existing ISO 8601 date between years 1 and 9999")) {
        try CalendarDate(year: 2026, month: 2, day: 30)
    }
    #expect(throws: EranosError.self) {
        try CalendarDate(parsing: "2026-02-30")
    }

    let time = try TimeOfDay(hour: 18, minute: 30)
    #expect("18:30" == time.description)
    #expect(try TimeOfDay(parsing: "18:30") == time)
    #expect(18 == time.hour)
    #expect(30 == time.minute)
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
    #expect(throws: EranosError.paymentAccount(.invalidIBAN(.invalidChecksum), detail: "invalid IBAN: the check digits do not match")) {
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

/// Identifiers that compared or hashed as `String` does, which holds canonically equivalent texts
/// equal, would take two identifiers the library holds apart for one, and read or change the
/// other entry.
@Test
func canonicallyEquivalentTextsStayTwoIdentifiers() throws {
    // "é" as one character, and as "e" and a combining acute accent.
    let composed = "\u{E9}"
    let decomposed = "e\u{301}"
    #expect(composed == decomposed)
    #expect(Member.ID(rawValue: composed) != Member.ID(rawValue: decomposed))
    #expect(2 == Set([Member.ID(rawValue: composed), Member.ID(rawValue: decomposed)]).count)
    let group = try Flat().group
    let first = try group.add(Member(id: Member.ID(composed), firstName: "Elena"))
    let second = try group.add(Member(id: Member.ID(decomposed), firstName: "Emil"))
    #expect(try "Elena" == group.member(withID: first)?.firstName)
    #expect(try "Emil" == group.member(withID: second)?.firstName)
}

/// Text that is not an identifier, with the failure and the message the library refuses it with.
private struct RefusedIdentifier {
    let text: String
    let failure: IDFailure
    let message: String
}

/// A refusal that reached an app as another failure, or without the library's message, would
/// give its user another reason than the library's; refusing text the library accepts would put
/// an entry the group holds out of the app's reach; and a check that changed the text would give
/// an entry another identifier than the app's.
@Test
func onlyTextThatCannotBeAnIdentifierIsRefused() throws {
    let group = try Flat().group
    // "â" is two bytes of UTF-8: 32 of them reach the limit of 64 bytes, and 33 pass it.
    let refusals = [
        RefusedIdentifier(text: "", failure: .empty, message: "an identifier must not be empty"),
        RefusedIdentifier(text: String(repeating: "a", count: 65), failure: .tooLong, message: "an identifier can be at most 64 bytes long"),
        RefusedIdentifier(text: String(repeating: "â", count: 33), failure: .tooLong, message: "an identifier can be at most 64 bytes long"),
        RefusedIdentifier(text: "a\tb", failure: .controlCharacter, message: "an identifier cannot contain control characters"),
        RefusedIdentifier(text: " ana", failure: .surroundingWhitespace, message: "an identifier cannot start or end with whitespace")
    ]
    for refusal in refusals {
        #expect(throws: EranosError.id(refusal.failure, detail: refusal.message)) {
            try Member.ID(refusal.text)
        }
        #expect(throws: EranosError.id(refusal.failure, detail: refusal.message)) {
            try group.member(withID: Member.ID(rawValue: refusal.text))
        }
    }
    // Kept as it is, case and inner spaces included.
    for text in ["nobody", "Ana Maria", "-food", "!a", "Lisboa · 2026", String(repeating: "a", count: 64), String(repeating: "â", count: 32)] {
        #expect(try text == Member.ID(text).rawValue)
        #expect(try nil == group.member(withID: Member.ID(rawValue: text)))
    }
}
