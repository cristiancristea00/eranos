// The value types. Every text they show comes from the library, and every value an app builds
// is checked by the library's rules: a type that formatted on its own or let a refused value
// through would show an app another value than the library holds.

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
    #expect(throws: EranosError.money(.unknownCurrency, detail: "unknown currency code")) {
        try CurrencyCode(code: "XYZ")
    }

    let price = try Money(parsing: "12.5", in: euro)
    #expect(price == Money(minorUnits: 1_250, currency: euro))
    #expect("12.50 EUR" == price.description)
    #expect(try price.adding(Money(minorUnits: 50, currency: euro)) == Money(minorUnits: 1_300, currency: euro))
    #expect(try price.subtracting(Money(minorUnits: 50, currency: euro)) == Money(minorUnits: 1_200, currency: euro))
    let zero = Money.zero(in: euro)
    #expect(zero == Money(minorUnits: 0, currency: euro))
    #expect([zero.isZero, zero.isPositive, zero.isNegative] == [true, false, false])
    #expect([price.isZero, price.isPositive, price.isNegative] == [false, true, false])
    let refund = try zero.subtracting(price)
    #expect([refund.isZero, refund.isPositive, refund.isNegative] == [false, false, true])
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

/// An amount or a balance written with other decimal places than its currency has would show
/// an app another amount than the group holds.
@Test
func amountsAndBalancesWriteTheDecimalPlacesOfTheirCurrency() throws {
    let yen = try Money(minorUnits: 12_345, currency: CurrencyCode(code: "JPY"))
    #expect("12345 JPY" == yen.description)
    #expect("owes 12345 JPY" == Balance.owes(yen).description)
    #expect("gets back 12345 JPY" == Balance.getsBack(yen).description)
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
    #expect(throws: EranosError.money(.invalidPercentage, detail: "invalid percentage: expected a number with at most 2 decimal places")) {
        try Percentage(basisPoints: -1)
    }
    #expect(throws: EranosError.self) {
        try Percentage(parsing: "half")
    }

    let parts = try Parts(parsing: "0.5")
    #expect(50 == parts.hundredths)
    #expect("0.5" == parts.description)
    #expect(try Parts(hundredths: 50) == parts)
    #expect(throws: EranosError.money(.invalidParts, detail: "invalid parts: expected a positive number with at most 2 decimal places")) {
        try Parts(hundredths: 0)
    }
    #expect(throws: EranosError.self) {
        try Parts(parsing: "half")
    }
}

/// An app that holds amounts as doubles would get other cents in Swift than the library rounds
/// to, or an amount back from a number past the range.
@Test
func amountsConvertFromDoublesAsTheLibraryConvertsThem() throws {
    let euro = try CurrencyCode(code: "EUR")
    // The double nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
    #expect(try Money(amount: 1.005, currency: euro) == Money(minorUnits: 101, currency: euro))
    #expect(throws: EranosError.money(.overflow, detail: "value out of range")) {
        try Money(amount: 1e21, currency: euro)
    }
}

/// A rate, a percentage, or parts held as a double would become other digits in Swift than in
/// the library, or a number too large for the layer's integers would wrap.
@Test
func ratesPercentagesAndPartsConvertFromDoublesAsTheLibraryConvertsThem() throws {
    #expect(try "0.30000000000000004" == ExchangeRate(0.1 + 0.2).description)
    #expect(throws: EranosError.money(.invalidExchangeRate, detail: "invalid exchange rate: expected a positive number with at most 18 decimal places")) {
        try ExchangeRate(0)
    }

    #expect(try 1_235 == Percentage(12.345).basisPoints)
    // Above 21 474 836.47 %, the basis points no longer fit an `Int32`.
    #expect(throws: EranosError.money(.invalidPercentage, detail: "invalid percentage: expected a number with at most 2 decimal places")) {
        try Percentage(30_000_000.0)
    }

    #expect(try 101 == Parts(1.005).hundredths)
    #expect(throws: EranosError.money(.invalidParts, detail: "invalid parts: expected a positive number with at most 2 decimal places")) {
        try Parts(30_000_000.0)
    }
}

/// The refusal of a number, text, or double that is not a quantity.
private let invalidQuantity = EranosError.money(.invalidQuantity, detail: "invalid quantity: expected a positive number with at most 4 decimal places")

/// A quantity read or written another way than the library's, or one the library refuses let
/// through, would multiply a line of a bill into another total than the group holds.
@Test
func quantitiesReadAndWriteAsTheLibraryDoes() throws {
    let weight = try Quantity(parsing: "0.452")
    #expect(4_520 == weight.tenThousandths)
    #expect("0.452" == weight.description)
    #expect(try Quantity(tenThousandths: 4_520) == weight)
    let three = try Quantity(whole: 3)
    #expect(30_000 == three.tenThousandths)
    #expect("3" == three.description)
    #expect(10_000 == Quantity.one.tenThousandths)
    #expect("1" == Quantity.one.description)
    #expect(try Quantity.one == Quantity(whole: 1))
    #expect(throws: invalidQuantity) {
        try Quantity(tenThousandths: 0)
    }
    // Above 922 337 203 685 477 whole units, the ten-thousandths no longer fit an `Int64`.
    #expect(throws: invalidQuantity) {
        try Quantity(whole: .max)
    }
    #expect(throws: invalidQuantity) {
        try Quantity(parsing: "half")
    }
}

/// A quantity held as a double would become other ten-thousandths in Swift than in the library,
/// or a number too large for an `Int64` of ten-thousandths would wrap.
@Test
func quantitiesConvertFromDoublesAsTheLibraryConvertsThem() throws {
    #expect(try 4_521 == Quantity(0.45205).tenThousandths)
    #expect(throws: invalidQuantity) {
        try Quantity(1e15)
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
    #expect(throws: EranosError.calendar(.invalidTime, detail: "invalid time: expected an ISO 8601 time between 00:00 and 23:59 with zero seconds")) {
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
    #expect("+40 712 345 678" == phone.international)
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

/// A generated identifier that repeated an earlier one would put two entries under one identifier,
/// one that the library's check refused would give an entry an identifier the app cannot store and
/// check again, and one out of order would list entries out of their creation order.
@Test
func generatedIdentifiersAreNewAcceptedAndInOrder() throws {
    let member = Member.ID.generate()
    let expense = Expense.ID.generate()
    #expect(member.rawValue != expense.rawValue)
    #expect(member.rawValue.utf8.lexicographicallyPrecedes(expense.rawValue.utf8))
    #expect(try member == Member.ID(member.rawValue))
    #expect(try expense == Expense.ID(expense.rawValue))
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
