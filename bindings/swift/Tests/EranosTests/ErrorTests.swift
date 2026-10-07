// The error every call throws. A failure converted to another module's case, to another
// failure, with another payload, or without the library's message would show an app
// another failure than the library's, so every failure the generated code can report is
// converted here, and every case's message is read back.

@testable import Eranos
import EranosGenerated
import Testing

/// A failure of the generated code, beside the error it must become.
private struct Conversion {
    let expected: Eranos.EranosError
    let generated: EranosGenerated.EranosError
}

/// `minorUnits` cents, as the generated code holds them.
private func generatedEuros(_ minorUnits: Int64) -> EranosGenerated.Money {
    EranosGenerated.Money(minorUnits: minorUnits, currency: "EUR")
}

/// `minorUnits` cents.
private func euros(_ minorUnits: Int64) -> Eranos.Money {
    Eranos.Money(minorUnits: minorUnits, currency: known("EUR"))
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
        expected: .id(.invalidID, detail: "detail"),
        generated: .Id(failure: .invalidId, detail: "detail")
    ),
    Conversion(
        expected: .account(.emptyName, detail: "detail"),
        generated: .Account(failure: .emptyName, detail: "detail")
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidChecksum), detail: "detail"),
        generated: .Account(failure: .invalidIban(reason: .invalidChecksum), detail: "detail")
    ),
    Conversion(
        expected: .account(.emptyUsername, detail: "detail"),
        generated: .Account(failure: .emptyUsername, detail: "detail")
    ),
    Conversion(
        expected: .calendar(.invalidDate, detail: "detail"),
        generated: .Calendar(failure: .invalidDate, detail: "detail")
    ),
    Conversion(
        expected: .calendar(.invalidTime, detail: "detail"),
        generated: .Calendar(failure: .invalidTime, detail: "detail")
    ),
    Conversion(
        expected: .category(.emptyName, detail: "detail"),
        generated: .Category(failure: .emptyName, detail: "detail")
    ),
    Conversion(
        expected: .contact(.invalidEmail, detail: "detail"),
        generated: .Contact(failure: .invalidEmail, detail: "detail")
    ),
    Conversion(
        expected: .contact(.invalidPhoneNumber, detail: "detail"),
        generated: .Contact(failure: .invalidPhoneNumber, detail: "detail")
    ),
    Conversion(
        expected: .money(.invalidAmount, detail: "detail"),
        generated: .Money(failure: .invalidAmount, detail: "detail")
    ),
    Conversion(
        expected: .money(.tooManyDecimals(currency: known("RON"), allowed: 29), detail: "detail"),
        generated: .Money(failure: .tooManyDecimals(currency: "RON", allowed: 29), detail: "detail")
    ),
    Conversion(
        expected: .money(.currencyMismatch(left: known("RON"), right: known("USD")), detail: "detail"),
        generated: .Money(failure: .currencyMismatch(left: "RON", right: "USD"), detail: "detail")
    ),
    Conversion(
        expected: .money(.unknownCurrency, detail: "detail"),
        generated: .Money(failure: .unknownCurrency, detail: "detail")
    ),
    Conversion(
        expected: .money(.invalidExchangeRate, detail: "detail"),
        generated: .Money(failure: .invalidExchangeRate, detail: "detail")
    ),
    Conversion(
        expected: .money(.invalidParts, detail: "detail"),
        generated: .Money(failure: .invalidParts, detail: "detail")
    ),
    Conversion(
        expected: .money(.invalidPercentage, detail: "detail"),
        generated: .Money(failure: .invalidPercentage, detail: "detail")
    ),
    Conversion(
        expected: .money(.overflow, detail: "detail"),
        generated: .Money(failure: .overflow, detail: "detail")
    ),
    Conversion(
        expected: .member(.emptyFirstName, detail: "detail"),
        generated: .Member(failure: .emptyFirstName, detail: "detail")
    ),
    Conversion(
        expected: .expense(.emptyName, detail: "detail"),
        generated: .Expense(failure: .emptyName, detail: "detail")
    ),
    Conversion(
        expected: .expense(.nonPositiveAmount, detail: "detail"),
        generated: .Expense(failure: .nonPositiveAmount, detail: "detail")
    ),
    Conversion(
        expected: .expense(.currencyMismatch(expected: known("USD"), found: known("GBP")), detail: "detail"),
        generated: .Expense(failure: .currencyMismatch(expected: "USD", found: "GBP"), detail: "detail")
    ),
    Conversion(
        expected: .expense(.emptySplit, detail: "detail"),
        generated: .Expense(failure: .emptySplit, detail: "detail")
    ),
    Conversion(
        expected: .expense(.duplicateMember(id("member-6")), detail: "detail"),
        generated: .Expense(failure: .duplicateMember(member: "member-6"), detail: "detail")
    ),
    Conversion(
        expected: .expense(.nonPositiveShare(id("member-9")), detail: "detail"),
        generated: .Expense(failure: .nonPositiveShare(member: "member-9"), detail: "detail")
    ),
    Conversion(
        expected: .expense(.amountsDoNotAddUp(expected: euros(3), actual: euros(2)), detail: "detail"),
        generated: .Expense(failure: .amountsDoNotAddUp(expected: generatedEuros(3), actual: generatedEuros(2)), detail: "detail")
    ),
    Conversion(
        expected: .expense(.percentagesDoNotAddUp(totalBasisPoints: 10), detail: "detail"),
        generated: .Expense(failure: .percentagesDoNotAddUp(totalBasisPoints: 10), detail: "detail")
    ),
    Conversion(
        expected: .expense(.noItems, detail: "detail"),
        generated: .Expense(failure: .noItems, detail: "detail")
    ),
    Conversion(
        expected: .expense(.emptyItemName, detail: "detail"),
        generated: .Expense(failure: .emptyItemName, detail: "detail")
    ),
    Conversion(
        expected: .expense(.zeroQuantity, detail: "detail"),
        generated: .Expense(failure: .zeroQuantity, detail: "detail")
    ),
    Conversion(
        expected: .expense(.itemsDoNotAddUp(expected: euros(8), actual: euros(7)), detail: "detail"),
        generated: .Expense(failure: .itemsDoNotAddUp(expected: generatedEuros(8), actual: generatedEuros(7)), detail: "detail")
    ),
    Conversion(
        expected: .expense(.negativeTip, detail: "detail"),
        generated: .Expense(failure: .negativeTip, detail: "detail")
    ),
    Conversion(
        expected: .expense(.overflow, detail: "detail"),
        generated: .Expense(failure: .overflow, detail: "detail")
    ),
    Conversion(
        expected: .settlement(.outOfMemory(balances: 31), detail: "detail"),
        generated: .Settlement(failure: .outOfMemory(balances: 31), detail: "detail")
    ),
    Conversion(
        expected: .transfer(.sameMember, detail: "detail"),
        generated: .Transfer(failure: .sameMember, detail: "detail")
    ),
    Conversion(
        expected: .transfer(.nonPositiveAmount, detail: "detail"),
        generated: .Transfer(failure: .nonPositiveAmount, detail: "detail")
    ),
    Conversion(
        expected: .group(.unsupportedFormat(format: 26), detail: "detail"),
        generated: .Group(failure: .unsupportedFormat(format: 26), detail: "detail")
    ),
    Conversion(
        expected: .group(.emptyName, detail: "detail"),
        generated: .Group(failure: .emptyName, detail: "detail")
    ),
    Conversion(
        expected: .group(.missingExchangeRate(known("RON")), detail: "detail"),
        generated: .Group(failure: .missingExchangeRate(currency: "RON"), detail: "detail")
    ),
    Conversion(
        expected: .group(.unexpectedExchangeRate, detail: "detail"),
        generated: .Group(failure: .unexpectedExchangeRate, detail: "detail")
    ),
    Conversion(
        expected: .group(.duplicateExchangeRate(known("GBP")), detail: "detail"),
        generated: .Group(failure: .duplicateExchangeRate(currency: "GBP"), detail: "detail")
    ),
    Conversion(
        expected: .group(.duplicateMember(id("member-16")), detail: "detail"),
        generated: .Group(failure: .duplicateMember(member: "member-16"), detail: "detail")
    ),
    Conversion(
        expected: .group(.unknownMember(id("member-24")), detail: "detail"),
        generated: .Group(failure: .unknownMember(member: "member-24"), detail: "detail")
    ),
    Conversion(
        expected: .group(.archivedMember(id("member-11")), detail: "detail"),
        generated: .Group(failure: .archivedMember(member: "member-11"), detail: "detail")
    ),
    Conversion(
        expected: .group(.memberInUse(member: id("member-19"), expenses: [id("expenses-18")], transfers: [id("transfers-20")]), detail: "detail"),
        generated: .Group(failure: .memberInUse(member: "member-19", expenses: ["expenses-18"], transfers: ["transfers-20"]), detail: "detail")
    ),
    Conversion(
        expected: .group(.duplicateCategory(id("category-12")), detail: "detail"),
        generated: .Group(failure: .duplicateCategory(category: "category-12"), detail: "detail")
    ),
    Conversion(
        expected: .group(.duplicateCategoryName(id("existing-13")), detail: "detail"),
        generated: .Group(failure: .duplicateCategoryName(existing: "existing-13"), detail: "detail")
    ),
    Conversion(
        expected: .group(.unknownCategory(id("category-22")), detail: "detail"),
        generated: .Group(failure: .unknownCategory(category: "category-22"), detail: "detail")
    ),
    Conversion(
        expected: .group(.builtInCategory, detail: "detail"),
        generated: .Group(failure: .builtInCategory, detail: "detail")
    ),
    Conversion(
        expected: .group(.duplicateExpense(id("expense-15")), detail: "detail"),
        generated: .Group(failure: .duplicateExpense(expense: "expense-15"), detail: "detail")
    ),
    Conversion(
        expected: .group(.unknownExpense(id("expense-23")), detail: "detail"),
        generated: .Group(failure: .unknownExpense(expense: "expense-23"), detail: "detail")
    ),
    Conversion(
        expected: .group(.duplicateTransfer(id("transfer-17")), detail: "detail"),
        generated: .Group(failure: .duplicateTransfer(transfer: "transfer-17"), detail: "detail")
    ),
    Conversion(
        expected: .group(.unknownTransfer(id("transfer-25")), detail: "detail"),
        generated: .Group(failure: .unknownTransfer(transfer: "transfer-25"), detail: "detail")
    ),
    Conversion(
        expected: .group(.busy, detail: "detail"),
        generated: .Group(failure: .busy, detail: "detail")
    ),
    Conversion(
        expected: .group(.overflow, detail: "detail"),
        generated: .Group(failure: .overflow, detail: "detail")
    ),
    Conversion(
        expected: .language(.invalidLanguage, detail: "detail"),
        generated: .Language(failure: .invalidLanguage, detail: "detail")
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidCharacter), detail: "detail"),
        generated: .Account(failure: .invalidIban(reason: .invalidCharacter), detail: "detail")
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidFormat), detail: "detail"),
        generated: .Account(failure: .invalidIban(reason: .invalidFormat), detail: "detail")
    ),
    Conversion(
        expected: .account(.invalidIBAN(.unknownCountry), detail: "detail"),
        generated: .Account(failure: .invalidIban(reason: .unknownCountry), detail: "detail")
    ),
    Conversion(
        expected: .account(.invalidIBAN(.wrongLength(expected: 24, found: 22)), detail: "detail"),
        generated: .Account(failure: .invalidIban(reason: .wrongLength(expected: 24, found: 22)), detail: "detail")
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidStructure), detail: "detail"),
        generated: .Account(failure: .invalidIban(reason: .invalidStructure), detail: "detail")
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidChecksum), detail: "detail"),
        generated: .Account(failure: .invalidIban(reason: .invalidChecksum), detail: "detail")
    ),
    Conversion(
        expected: .identifierGiven(detail: "detail"),
        generated: .IdentifierGiven(detail: "detail")
    ),
    Conversion(
        expected: .identifierMissing(detail: "detail"),
        generated: .IdentifierMissing(detail: "detail")
    ),
    Conversion(
        expected: .invalidJSON(line: 1, column: 2, detail: "detail"),
        generated: .InvalidJson(line: 1, column: 2, detail: "detail")
    ),
    Conversion(
        expected: .cancelled(detail: "detail"),
        generated: .Cancelled(detail: "detail")
    ),
    Conversion(
        expected: .released(detail: "detail"),
        generated: .Released(detail: "detail")
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
