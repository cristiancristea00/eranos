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
        expected: .account(.emptyName, detail: "detail"),
        generated: .Account(detail: "detail", failure: .emptyName)
    ),
    Conversion(
        expected: .account(.emptyUsername, detail: "detail"),
        generated: .Account(detail: "detail", failure: .emptyUsername)
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidChecksum), detail: "detail"),
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidChecksum))
    ),
    Conversion(
        expected: .calendar(.invalidDate, detail: "detail"),
        generated: .Calendar(detail: "detail", failure: .invalidDate)
    ),
    Conversion(
        expected: .calendar(.invalidTime, detail: "detail"),
        generated: .Calendar(detail: "detail", failure: .invalidTime)
    ),
    Conversion(
        expected: .category(.emptyName, detail: "detail"),
        generated: .Category(detail: "detail", failure: .emptyName)
    ),
    Conversion(
        expected: .contact(.invalidEmail, detail: "detail"),
        generated: .Contact(detail: "detail", failure: .invalidEmail)
    ),
    Conversion(
        expected: .contact(.invalidPhoneNumber, detail: "detail"),
        generated: .Contact(detail: "detail", failure: .invalidPhoneNumber)
    ),
    Conversion(
        expected: .expense(.amountsDoNotAddUp(actual: euros(2), expected: euros(3)), detail: "detail"),
        generated: .Expense(detail: "detail", failure: .amountsDoNotAddUp(actual: generatedEuros(2), expected: generatedEuros(3)))
    ),
    Conversion(
        expected: .expense(.currencyMismatch(expected: known("USD"), found: known("GBP")), detail: "detail"),
        generated: .Expense(detail: "detail", failure: .currencyMismatch(expected: "USD", found: "GBP"))
    ),
    Conversion(
        expected: .expense(.duplicateMember(id("member-6")), detail: "detail"),
        generated: .Expense(detail: "detail", failure: .duplicateMember(member: "member-6"))
    ),
    Conversion(
        expected: .expense(.emptyItemName, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .emptyItemName)
    ),
    Conversion(
        expected: .expense(.emptyName, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .emptyName)
    ),
    Conversion(
        expected: .expense(.emptySplit, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .emptySplit)
    ),
    Conversion(
        expected: .expense(.itemsDoNotAddUp(actual: euros(7), expected: euros(8)), detail: "detail"),
        generated: .Expense(detail: "detail", failure: .itemsDoNotAddUp(actual: generatedEuros(7), expected: generatedEuros(8)))
    ),
    Conversion(
        expected: .expense(.negativeTip, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .negativeTip)
    ),
    Conversion(
        expected: .expense(.noItems, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .noItems)
    ),
    Conversion(
        expected: .expense(.nonPositiveAmount, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .nonPositiveAmount)
    ),
    Conversion(
        expected: .expense(.nonPositiveShare(id("member-9")), detail: "detail"),
        generated: .Expense(detail: "detail", failure: .nonPositiveShare(member: "member-9"))
    ),
    Conversion(
        expected: .expense(.overflow, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .overflow)
    ),
    Conversion(
        expected: .expense(.percentagesDoNotAddUp(totalBasisPoints: 10), detail: "detail"),
        generated: .Expense(detail: "detail", failure: .percentagesDoNotAddUp(totalBasisPoints: 10))
    ),
    Conversion(
        expected: .expense(.zeroQuantity, detail: "detail"),
        generated: .Expense(detail: "detail", failure: .zeroQuantity)
    ),
    Conversion(
        expected: .group(.archivedMember(id("member-11")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .archivedMember(member: "member-11"))
    ),
    Conversion(
        expected: .group(.builtInCategory, detail: "detail"),
        generated: .Group(detail: "detail", failure: .builtInCategory)
    ),
    Conversion(
        expected: .group(.busy, detail: "detail"),
        generated: .Group(detail: "detail", failure: .busy)
    ),
    Conversion(
        expected: .group(.duplicateCategory(id("category-12")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .duplicateCategory(category: "category-12"))
    ),
    Conversion(
        expected: .group(.duplicateCategoryName(id("existing-13")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .duplicateCategoryName(existing: "existing-13"))
    ),
    Conversion(
        expected: .group(.duplicateExchangeRate(known("GBP")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .duplicateExchangeRate(currency: "GBP"))
    ),
    Conversion(
        expected: .group(.duplicateExpense(id("expense-15")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .duplicateExpense(expense: "expense-15"))
    ),
    Conversion(
        expected: .group(.duplicateMember(id("member-16")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .duplicateMember(member: "member-16"))
    ),
    Conversion(
        expected: .group(.duplicateTransfer(id("transfer-17")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .duplicateTransfer(transfer: "transfer-17"))
    ),
    Conversion(
        expected: .group(.emptyName, detail: "detail"),
        generated: .Group(detail: "detail", failure: .emptyName)
    ),
    Conversion(
        expected: .group(.memberInUse(expenses: [id("expenses-18")], member: id("member-19"), transfers: [id("transfers-20")]), detail: "detail"),
        generated: .Group(detail: "detail", failure: .memberInUse(expenses: ["expenses-18"], member: "member-19", transfers: ["transfers-20"]))
    ),
    Conversion(
        expected: .group(.missingExchangeRate(known("RON")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .missingExchangeRate(currency: "RON"))
    ),
    Conversion(
        expected: .group(.overflow, detail: "detail"),
        generated: .Group(detail: "detail", failure: .overflow)
    ),
    Conversion(
        expected: .group(.unexpectedExchangeRate, detail: "detail"),
        generated: .Group(detail: "detail", failure: .unexpectedExchangeRate)
    ),
    Conversion(
        expected: .group(.unknownCategory(id("category-22")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .unknownCategory(category: "category-22"))
    ),
    Conversion(
        expected: .group(.unknownExpense(id("expense-23")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .unknownExpense(expense: "expense-23"))
    ),
    Conversion(
        expected: .group(.unknownMember(id("member-24")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .unknownMember(member: "member-24"))
    ),
    Conversion(
        expected: .group(.unknownTransfer(id("transfer-25")), detail: "detail"),
        generated: .Group(detail: "detail", failure: .unknownTransfer(transfer: "transfer-25"))
    ),
    Conversion(
        expected: .group(.unsupportedFormat(format: 26), detail: "detail"),
        generated: .Group(detail: "detail", failure: .unsupportedFormat(format: 26))
    ),
    Conversion(
        expected: .id(.invalidID, detail: "detail"),
        generated: .Id(detail: "detail", failure: .invalidId)
    ),
    Conversion(
        expected: .language(.invalidLanguage, detail: "detail"),
        generated: .Language(detail: "detail", failure: .invalidLanguage)
    ),
    Conversion(
        expected: .member(.emptyFirstName, detail: "detail"),
        generated: .Member(detail: "detail", failure: .emptyFirstName)
    ),
    Conversion(
        expected: .money(.currencyMismatch(left: known("RON"), right: known("USD")), detail: "detail"),
        generated: .Money(detail: "detail", failure: .currencyMismatch(left: "RON", right: "USD"))
    ),
    Conversion(
        expected: .money(.invalidAmount, detail: "detail"),
        generated: .Money(detail: "detail", failure: .invalidAmount)
    ),
    Conversion(
        expected: .money(.invalidExchangeRate, detail: "detail"),
        generated: .Money(detail: "detail", failure: .invalidExchangeRate)
    ),
    Conversion(
        expected: .money(.invalidParts, detail: "detail"),
        generated: .Money(detail: "detail", failure: .invalidParts)
    ),
    Conversion(
        expected: .money(.invalidPercentage, detail: "detail"),
        generated: .Money(detail: "detail", failure: .invalidPercentage)
    ),
    Conversion(
        expected: .money(.overflow, detail: "detail"),
        generated: .Money(detail: "detail", failure: .overflow)
    ),
    Conversion(
        expected: .money(.tooManyDecimals(allowed: 29, currency: known("RON")), detail: "detail"),
        generated: .Money(detail: "detail", failure: .tooManyDecimals(allowed: 29, currency: "RON"))
    ),
    Conversion(
        expected: .money(.unknownCurrency, detail: "detail"),
        generated: .Money(detail: "detail", failure: .unknownCurrency)
    ),
    Conversion(
        expected: .settlement(.outOfMemory(balances: 31), detail: "detail"),
        generated: .Settlement(detail: "detail", failure: .outOfMemory(balances: 31))
    ),
    Conversion(
        expected: .transfer(.nonPositiveAmount, detail: "detail"),
        generated: .Transfer(detail: "detail", failure: .nonPositiveAmount)
    ),
    Conversion(
        expected: .transfer(.sameMember, detail: "detail"),
        generated: .Transfer(detail: "detail", failure: .sameMember)
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidCharacter), detail: "detail"),
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidCharacter))
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidChecksum), detail: "detail"),
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidChecksum))
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidFormat), detail: "detail"),
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidFormat))
    ),
    Conversion(
        expected: .account(.invalidIBAN(.invalidStructure), detail: "detail"),
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .invalidStructure))
    ),
    Conversion(
        expected: .account(.invalidIBAN(.unknownCountry), detail: "detail"),
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .unknownCountry))
    ),
    Conversion(
        expected: .account(.invalidIBAN(.wrongLength(expected: 24, found: 22)), detail: "detail"),
        generated: .Account(detail: "detail", failure: .invalidIban(reason: .wrongLength(expected: 24, found: 22)))
    ),
    Conversion(
        expected: .cancelled(detail: "detail"),
        generated: .Cancelled(detail: "detail")
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
        expected: .released(detail: "detail"),
        generated: .Released(detail: "detail")
    ),
    Conversion(
        expected: .invalidJSON(line: 1, column: 2, detail: "detail"),
        generated: .InvalidJson(column: 2, detail: "detail", line: 1)
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
