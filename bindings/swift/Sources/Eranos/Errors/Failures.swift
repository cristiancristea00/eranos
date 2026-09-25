// The failure of every module of the library, with the identifiers,
// amounts, and currencies it holds typed as the rest of the module.

internal import EranosGenerated

/// What the library's `AccountError` reports: a case per failure, with what it holds.
public enum AccountFailure: Sendable, Hashable {
    /// An account whose name is blank.
    case emptyName

    /// A username that is blank.
    case emptyUsername

    /// An IBAN that failed validation, and why.
    case invalidIBAN(IBANReason)
}

extension AccountFailure {
    init(_ failure: EranosGenerated.AccountFailure) {
        switch failure {
        case .emptyName:
            self = .emptyName
        case .emptyUsername:
            self = .emptyUsername
        case let .invalidIban(reason):
            self = .invalidIBAN(IBANReason(reason))
        }
    }
}

/// What the library's `CalendarError` reports: a case per failure, with what it holds.
public enum CalendarFailure: Sendable, Hashable {
    /// A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text.
    case invalidDate

    /// A time of day that does not exist, has seconds, or is not ISO 8601 text.
    case invalidTime
}

extension CalendarFailure {
    init(_ failure: EranosGenerated.CalendarFailure) {
        switch failure {
        case .invalidDate:
            self = .invalidDate
        case .invalidTime:
            self = .invalidTime
        }
    }
}

/// What the library's `CategoryError` reports: a case per failure, with what it holds.
public enum CategoryFailure: Sendable, Hashable {
    /// A name that is blank.
    case emptyName
}

extension CategoryFailure {
    init(_ failure: EranosGenerated.CategoryFailure) {
        switch failure {
        case .emptyName:
            self = .emptyName
        }
    }
}

/// What the library's `ContactError` reports: a case per failure, with what it holds.
public enum ContactFailure: Sendable, Hashable {
    /// An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around an
    /// `@`, a malformed one, one too long, or whitespace or an invisible character.
    case invalidEmail

    /// A phone number that does not start with `+`, holds a letter or an extension, or is not valid
    /// in its country's numbering plan.
    case invalidPhoneNumber
}

extension ContactFailure {
    init(_ failure: EranosGenerated.ContactFailure) {
        switch failure {
        case .invalidEmail:
            self = .invalidEmail
        case .invalidPhoneNumber:
            self = .invalidPhoneNumber
        }
    }
}

/// What the library's `ExpenseError` reports: a case per failure, with what it holds.
public enum ExpenseFailure: Sendable, Hashable {
    /// The amounts of a split do not add up to the amount being split.
    case amountsDoNotAddUp(actual: Money, expected: Money)

    /// An amount inside the expense is in a different currency.
    case currencyMismatch(expected: Currency, found: Currency)

    /// A member appears more than once in one split.
    case duplicateMember(Member.ID)

    /// An item whose name is blank.
    case emptyItemName

    /// An expense whose name is blank.
    case emptyName

    /// A split lists no member.
    case emptySplit

    /// The item line totals do not add up to the expense amount.
    case itemsDoNotAddUp(actual: Money, expected: Money)

    /// A fixed tip below zero.
    case negativeTip

    /// An itemised expense without items.
    case noItems

    /// The amount or unit price is zero or negative.
    case nonPositiveAmount

    /// A member's percentage or amount is zero or negative.
    case nonPositiveShare(Member.ID)

    /// A total that does not fit the range of an amount.
    case overflow

    /// The percentages of a split do not add up to exactly 100 %.
    case percentagesDoNotAddUp(totalBasisPoints: Int)

    /// An item quantity of zero.
    case zeroQuantity
}

extension ExpenseFailure {
    init(_ failure: EranosGenerated.ExpenseFailure) {
        switch failure {
        case let .amountsDoNotAddUp(actual, expected):
            self = .amountsDoNotAddUp(actual: Money(actual), expected: Money(expected))
        case let .currencyMismatch(expected, found):
            self = .currencyMismatch(expected: Currency(known: expected), found: Currency(known: found))
        case let .duplicateMember(member):
            self = .duplicateMember(Member.ID(rawValue: member))
        case .emptyItemName:
            self = .emptyItemName
        case .emptyName:
            self = .emptyName
        case .emptySplit:
            self = .emptySplit
        case let .itemsDoNotAddUp(actual, expected):
            self = .itemsDoNotAddUp(actual: Money(actual), expected: Money(expected))
        case .negativeTip:
            self = .negativeTip
        case .noItems:
            self = .noItems
        case .nonPositiveAmount:
            self = .nonPositiveAmount
        case let .nonPositiveShare(member):
            self = .nonPositiveShare(Member.ID(rawValue: member))
        case .overflow:
            self = .overflow
        case let .percentagesDoNotAddUp(totalBasisPoints):
            self = .percentagesDoNotAddUp(totalBasisPoints: Int(totalBasisPoints))
        case .zeroQuantity:
            self = .zeroQuantity
        }
    }
}

/// What the library's `GroupError` reports: a case per failure, with what it holds.
public enum GroupFailure: Sendable, Hashable {
    /// An archived member cannot join a new expense.
    case archivedMember(Member.ID)

    /// The built-in Uncategorised category cannot be renamed or removed.
    case builtInCategory

    /// A shared group's lock was asked for without waiting, while another call held it.
    case busy

    /// A category with this ID already exists.
    case duplicateCategory(Category.ID)

    /// Another category already has this name, compared by Unicode canonical caseless matching.
    case duplicateCategoryName(Category.ID)

    /// The same currency appears twice among the rates of a currency change.
    case duplicateExchangeRate(Currency)

    /// An expense with this ID already exists.
    case duplicateExpense(Expense.ID)

    /// A member with this ID already exists.
    case duplicateMember(Member.ID)

    /// A transfer with this ID already exists.
    case duplicateTransfer(Transfer.ID)

    /// A group name that is blank.
    case emptyName

    /// The member is still referenced and cannot be removed; archive it instead.
    case memberInUse(expenses: [Expense.ID], member: Member.ID, transfers: [Transfer.ID])

    /// An entry in another currency than the group's has no exchange rate into the group currency:
    /// its cost is local, or a currency change gave no rate for its currency.
    case missingExchangeRate(Currency)

    /// A total would leave the range of an amount.
    case overflow

    /// An entry in the group currency has a converted cost; only a local cost is in the group
    /// currency.
    case unexpectedExchangeRate

    /// No category with this ID.
    case unknownCategory(Category.ID)

    /// No expense with this ID.
    case unknownExpense(Expense.ID)

    /// No member with this ID.
    case unknownMember(Member.ID)

    /// No transfer with this ID.
    case unknownTransfer(Transfer.ID)

    /// A saved group in a format this version does not read: a newer one, whose fields it would
    /// drop, or an earlier one.
    case unsupportedFormat(format: Int)
}

extension GroupFailure {
    init(_ failure: EranosGenerated.GroupFailure) {
        switch failure {
        case let .archivedMember(member):
            self = .archivedMember(Member.ID(rawValue: member))
        case .builtInCategory:
            self = .builtInCategory
        case .busy:
            self = .busy
        case let .duplicateCategory(category):
            self = .duplicateCategory(Category.ID(rawValue: category))
        case let .duplicateCategoryName(existing):
            self = .duplicateCategoryName(Category.ID(rawValue: existing))
        case let .duplicateExchangeRate(currency):
            self = .duplicateExchangeRate(Currency(known: currency))
        case let .duplicateExpense(expense):
            self = .duplicateExpense(Expense.ID(rawValue: expense))
        case let .duplicateMember(member):
            self = .duplicateMember(Member.ID(rawValue: member))
        case let .duplicateTransfer(transfer):
            self = .duplicateTransfer(Transfer.ID(rawValue: transfer))
        case .emptyName:
            self = .emptyName
        case let .memberInUse(expenses, member, transfers):
            self = .memberInUse(expenses: expenses.map { Expense.ID(rawValue: $0) }, member: Member.ID(rawValue: member), transfers: transfers.map { Transfer.ID(rawValue: $0) })
        case let .missingExchangeRate(currency):
            self = .missingExchangeRate(Currency(known: currency))
        case .overflow:
            self = .overflow
        case .unexpectedExchangeRate:
            self = .unexpectedExchangeRate
        case let .unknownCategory(category):
            self = .unknownCategory(Category.ID(rawValue: category))
        case let .unknownExpense(expense):
            self = .unknownExpense(Expense.ID(rawValue: expense))
        case let .unknownMember(member):
            self = .unknownMember(Member.ID(rawValue: member))
        case let .unknownTransfer(transfer):
            self = .unknownTransfer(Transfer.ID(rawValue: transfer))
        case let .unsupportedFormat(format):
            self = .unsupportedFormat(format: Int(format))
        }
    }
}

/// What the library's `IdError` reports: a case per failure, with what it holds.
public enum IDFailure: Sendable, Hashable {
    /// Text that is not a UUID.
    case invalidID
}

extension IDFailure {
    init(_ failure: EranosGenerated.IdFailure) {
        switch failure {
        case .invalidId:
            self = .invalidID
        }
    }
}

/// What the library's `LanguageError` reports: a case per failure, with what it holds.
public enum LanguageFailure: Sendable, Hashable {
    /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
    case invalidLanguage
}

extension LanguageFailure {
    init(_ failure: EranosGenerated.LanguageFailure) {
        switch failure {
        case .invalidLanguage:
            self = .invalidLanguage
        }
    }
}

/// What the library's `MemberError` reports: a case per failure, with what it holds.
public enum MemberFailure: Sendable, Hashable {
    /// A first name that is blank.
    case emptyFirstName
}

extension MemberFailure {
    init(_ failure: EranosGenerated.MemberFailure) {
        switch failure {
        case .emptyFirstName:
            self = .emptyFirstName
        }
    }
}

/// What the library's `MoneyError` reports: a case per failure, with what it holds.
public enum MoneyFailure: Sendable, Hashable {
    /// Arithmetic between amounts of two different currencies.
    case currencyMismatch(left: Currency, right: Currency)

    /// An amount that is not a decimal number.
    case invalidAmount

    /// An exchange rate that is not a positive decimal number with at most 18 decimal places.
    case invalidExchangeRate

    /// A number of parts that is not a positive decimal number with at most two decimal places.
    case invalidParts

    /// A percentage that is not a decimal number with at most two decimal places.
    case invalidPercentage

    /// A number or a result that does not fit its range.
    case overflow

    /// An amount with more decimal places than its currency has.
    case tooManyDecimals(allowed: Int, currency: Currency)

    /// A currency code that is not an active ISO 4217 code.
    case unknownCurrency
}

extension MoneyFailure {
    init(_ failure: EranosGenerated.MoneyFailure) {
        switch failure {
        case let .currencyMismatch(left, right):
            self = .currencyMismatch(left: Currency(known: left), right: Currency(known: right))
        case .invalidAmount:
            self = .invalidAmount
        case .invalidExchangeRate:
            self = .invalidExchangeRate
        case .invalidParts:
            self = .invalidParts
        case .invalidPercentage:
            self = .invalidPercentage
        case .overflow:
            self = .overflow
        case let .tooManyDecimals(allowed, currency):
            self = .tooManyDecimals(allowed: Int(allowed), currency: Currency(known: currency))
        case .unknownCurrency:
            self = .unknownCurrency
        }
    }
}

/// What the library's `SettlementError` reports: a case per failure, with what it holds.
public enum SettlementFailure: Sendable, Hashable {
    /// The exact solver needs 2ⁿ bytes for `n` open balances, and that table does not fit in the
    /// address space or the allocator refused it.
    case outOfMemory(balances: Int)
}

extension SettlementFailure {
    init(_ failure: EranosGenerated.SettlementFailure) {
        switch failure {
        case let .outOfMemory(balances):
            self = .outOfMemory(balances: Int(balances))
        }
    }
}

/// What the library's `TransferError` reports: a case per failure, with what it holds.
public enum TransferFailure: Sendable, Hashable {
    /// The amount is zero or negative.
    case nonPositiveAmount

    /// The sender and the recipient are the same member.
    case sameMember
}

extension TransferFailure {
    init(_ failure: EranosGenerated.TransferFailure) {
        switch failure {
        case .nonPositiveAmount:
            self = .nonPositiveAmount
        case .sameMember:
            self = .sameMember
        }
    }
}

/// Why the library refused an IBAN: a case per check it failed.
public enum IBANReason: Sendable, Hashable {
    /// A character other than an ASCII letter, a digit, or whitespace (which is dropped).
    case invalidCharacter

    /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97
    /// check failed: the number contains a typo.
    case invalidChecksum

    /// Not two letters followed by two check digits.
    case invalidFormat

    /// A letter where the country's account number has a digit, or a digit where it has a letter,
    /// by the BBAN structure of the SWIFT IBAN registry.
    case invalidStructure

    /// A country code without an entry in the SWIFT IBAN registry.
    case unknownCountry

    /// The wrong number of characters for the country.
    case wrongLength(expected: Int, found: Int)
}

extension IBANReason {
    init(_ reason: EranosGenerated.IbanReason) {
        switch reason {
        case .invalidCharacter:
            self = .invalidCharacter
        case .invalidChecksum:
            self = .invalidChecksum
        case .invalidFormat:
            self = .invalidFormat
        case .invalidStructure:
            self = .invalidStructure
        case .unknownCountry:
            self = .unknownCountry
        case let .wrongLength(expected, found):
            self = .wrongLength(expected: Int(expected), found: Int(found))
        }
    }
}
