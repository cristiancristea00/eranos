// The error every call throws. A failure converted to another module's case, to another
// failure, with another payload, or without the library's message would show an app
// another failure than the library's, so every failure the generated code can report is
// converted here, and every case's message is read back.

@testable import Eranos
import EranosGenerated
import Testing

/// A failure of the generated code, beside the error it must become.
private struct Conversion {
    let generated: EranosGenerated.EranosError
    let expected: Eranos.EranosError
}

/// `minorUnits` cents, as the generated code holds them.
private func generatedEuros(_ minorUnits: Int64) -> EranosGenerated.Money {
    EranosGenerated.Money(currency: "EUR", minorUnits: minorUnits)
}

/// `minorUnits` cents.
private func euros(_ minorUnits: Int64) -> Eranos.Money {
    Eranos.Money(currency: known("EUR"), minorUnits: minorUnits)
}

/// The identifier written `text`, of the entity the call site expects.
private func id<Entity>(_ text: String) -> Identifier<Entity> {
    Identifier(rawValue: text)
}

/// The currency of `code`.
private func known(_ code: String) -> Eranos.Currency {
    Eranos.Currency(known: code)
}

/// Every failure the generated code can report, with every reason of a refused IBAN.
private let conversions: [Conversion] = [
    Conversion(
        generated: .Account(detail: "detail", failure: .emptyName),
        expected: .account(.emptyName, detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .emptyUsername),
        expected: .account(.emptyUsername, detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidChecksum)),
        expected: .account(.invalidIBAN(.invalidChecksum), detail: "detail")
    ),
    Conversion(
        generated: .Calendar(detail: "detail", failure: .invalidDate),
        expected: .calendar(.invalidDate, detail: "detail")
    ),
    Conversion(
        generated: .Calendar(detail: "detail", failure: .invalidTime),
        expected: .calendar(.invalidTime, detail: "detail")
    ),
    Conversion(
        generated: .Category(detail: "detail", failure: .emptyName),
        expected: .category(.emptyName, detail: "detail")
    ),
    Conversion(
        generated: .Contact(detail: "detail", failure: .invalidEmail),
        expected: .contact(.invalidEmail, detail: "detail")
    ),
    Conversion(
        generated: .Contact(detail: "detail", failure: .invalidPhoneNumber),
        expected: .contact(.invalidPhoneNumber, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .amountsDoNotAddUp(actual: generatedEuros(2), expected: generatedEuros(3))),
        expected: .expense(.amountsDoNotAddUp(actual: euros(2), expected: euros(3)), detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .currencyMismatch(expected: "USD", found: "GBP")),
        expected: .expense(.currencyMismatch(expected: known("USD"), found: known("GBP")), detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .duplicateMember(member: "member-6")),
        expected: .expense(.duplicateMember(id("member-6")), detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .emptyItemName),
        expected: .expense(.emptyItemName, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .emptyName),
        expected: .expense(.emptyName, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .emptySplit),
        expected: .expense(.emptySplit, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .itemsDoNotAddUp(actual: generatedEuros(7), expected: generatedEuros(8))),
        expected: .expense(.itemsDoNotAddUp(actual: euros(7), expected: euros(8)), detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .negativeTip),
        expected: .expense(.negativeTip, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .noItems),
        expected: .expense(.noItems, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .nonPositiveAmount),
        expected: .expense(.nonPositiveAmount, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .nonPositiveShare(member: "member-9")),
        expected: .expense(.nonPositiveShare(id("member-9")), detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .overflow),
        expected: .expense(.overflow, detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .percentagesDoNotAddUp(totalBasisPoints: 10)),
        expected: .expense(.percentagesDoNotAddUp(totalBasisPoints: 10), detail: "detail")
    ),
    Conversion(
        generated: .Expense(detail: "detail", failure: .zeroQuantity),
        expected: .expense(.zeroQuantity, detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .archivedMember(member: "member-11")),
        expected: .group(.archivedMember(id("member-11")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .builtInCategory),
        expected: .group(.builtInCategory, detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .busy),
        expected: .group(.busy, detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .duplicateCategory(category: "category-12")),
        expected: .group(.duplicateCategory(id("category-12")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .duplicateCategoryName(existing: "existing-13")),
        expected: .group(.duplicateCategoryName(id("existing-13")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .duplicateExchangeRate(currency: "GBP")),
        expected: .group(.duplicateExchangeRate(known("GBP")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .duplicateExpense(expense: "expense-15")),
        expected: .group(.duplicateExpense(id("expense-15")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .duplicateMember(member: "member-16")),
        expected: .group(.duplicateMember(id("member-16")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .duplicateTransfer(transfer: "transfer-17")),
        expected: .group(.duplicateTransfer(id("transfer-17")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .emptyName),
        expected: .group(.emptyName, detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .memberInUse(expenses: ["expenses-18"], member: "member-19", transfers: ["transfers-20"])),
        expected: .group(.memberInUse(expenses: [id("expenses-18")], member: id("member-19"), transfers: [id("transfers-20")]), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .missingExchangeRate(currency: "RON")),
        expected: .group(.missingExchangeRate(known("RON")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .overflow),
        expected: .group(.overflow, detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .unexpectedExchangeRate),
        expected: .group(.unexpectedExchangeRate, detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .unknownCategory(category: "category-22")),
        expected: .group(.unknownCategory(id("category-22")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .unknownExpense(expense: "expense-23")),
        expected: .group(.unknownExpense(id("expense-23")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .unknownMember(member: "member-24")),
        expected: .group(.unknownMember(id("member-24")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .unknownTransfer(transfer: "transfer-25")),
        expected: .group(.unknownTransfer(id("transfer-25")), detail: "detail")
    ),
    Conversion(
        generated: .Group(detail: "detail", failure: .unsupportedFormat(format: 26)),
        expected: .group(.unsupportedFormat(format: 26), detail: "detail")
    ),
    Conversion(
        generated: .Id(detail: "detail", failure: .invalidId),
        expected: .id(.invalidID, detail: "detail")
    ),
    Conversion(
        generated: .Language(detail: "detail", failure: .invalidLanguage),
        expected: .language(.invalidLanguage, detail: "detail")
    ),
    Conversion(
        generated: .Member(detail: "detail", failure: .emptyFirstName),
        expected: .member(.emptyFirstName, detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .currencyMismatch(left: "RON", right: "USD")),
        expected: .money(.currencyMismatch(left: known("RON"), right: known("USD")), detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .invalidAmount),
        expected: .money(.invalidAmount, detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .invalidExchangeRate),
        expected: .money(.invalidExchangeRate, detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .invalidParts),
        expected: .money(.invalidParts, detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .invalidPercentage),
        expected: .money(.invalidPercentage, detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .overflow),
        expected: .money(.overflow, detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .tooManyDecimals(allowed: 29, currency: "RON")),
        expected: .money(.tooManyDecimals(allowed: 29, currency: known("RON")), detail: "detail")
    ),
    Conversion(
        generated: .Money(detail: "detail", failure: .unknownCurrency),
        expected: .money(.unknownCurrency, detail: "detail")
    ),
    Conversion(
        generated: .Settlement(detail: "detail", failure: .outOfMemory(balances: 31)),
        expected: .settlement(.outOfMemory(balances: 31), detail: "detail")
    ),
    Conversion(
        generated: .Transfer(detail: "detail", failure: .nonPositiveAmount),
        expected: .transfer(.nonPositiveAmount, detail: "detail")
    ),
    Conversion(
        generated: .Transfer(detail: "detail", failure: .sameMember),
        expected: .transfer(.sameMember, detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidCharacter)),
        expected: .account(.invalidIBAN(.invalidCharacter), detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidChecksum)),
        expected: .account(.invalidIBAN(.invalidChecksum), detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidFormat)),
        expected: .account(.invalidIBAN(.invalidFormat), detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidStructure)),
        expected: .account(.invalidIBAN(.invalidStructure), detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .unknownCountry)),
        expected: .account(.invalidIBAN(.unknownCountry), detail: "detail")
    ),
    Conversion(
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .wrongLength(expected: 24, found: 22))),
        expected: .account(.invalidIBAN(.wrongLength(expected: 24, found: 22)), detail: "detail")
    ),
    Conversion(
        generated: .Cancelled(detail: "detail"),
        expected: .cancelled(detail: "detail")
    ),
    Conversion(
        generated: .IdentifierGiven(detail: "detail"),
        expected: .identifierGiven(detail: "detail")
    ),
    Conversion(
        generated: .IdentifierMissing(detail: "detail"),
        expected: .identifierMissing(detail: "detail")
    ),
    Conversion(
        generated: .Released(detail: "detail"),
        expected: .released(detail: "detail")
    ),
    Conversion(
        generated: .InvalidJson(column: 2, detail: "detail", line: 1),
        expected: .invalidJSON(line: 1, column: 2, detail: "detail")
    )
]

/// A conversion that reached another case or failure, dropped what the failure holds, or lost
/// the message would show an app another failure than the library's.
@Test
func everyFailureOfTheGeneratedCodeConvertsToItsCase() {
    #expect(66 == conversions.count)
    for conversion in conversions {
        let error = Eranos.EranosError(conversion.generated)
        #expect(error == conversion.expected)
        #expect("detail" == error.detail)
        #expect("detail" == error.errorDescription)
    }
}
