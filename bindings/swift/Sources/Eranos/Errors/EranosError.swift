internal import EranosGenerated
public import Foundation

/// Why a call failed: a case per module of the library, holding what the module reports and the
/// library's message, and the cases of the bindings' own.
///
/// Every throwing call of the package throws this type, so a `catch` matches a module's failures
/// together or one of them:
///
/// ```swift
/// do {
///     try group.remove(memberID)
/// } catch .expenseGroup(.memberInUse(_, let expenses, let transfers), _) {
///     // The member still has entries; archive them instead.
/// } catch {
///     print(error.detail)
/// }
/// ```
public enum EranosError: Error, Sendable, Hashable {
    /// The library's `IdError`, with the message it words.
    case id(IDFailure, detail: String)

    /// The library's `PaymentAccountError`, with the message it words.
    case paymentAccount(PaymentAccountFailure, detail: String)

    /// The library's `CalendarError`, with the message it words.
    case calendar(CalendarFailure, detail: String)

    /// The library's `CategoryError`, with the message it words.
    case category(CategoryFailure, detail: String)

    /// The library's `ContactError`, with the message it words.
    case contact(ContactFailure, detail: String)

    /// The library's `MoneyError`, with the message it words.
    case money(MoneyFailure, detail: String)

    /// The library's `MemberError`, with the message it words.
    case member(MemberFailure, detail: String)

    /// The library's `ExpenseError`, with the message it words.
    case expense(ExpenseFailure, detail: String)

    /// The library's `SettlementError`, with the message it words.
    case settlement(SettlementFailure, detail: String)

    /// The library's `TransferError`, with the message it words.
    case transfer(TransferFailure, detail: String)

    /// The library's `ExpenseGroupError`, with the message it words.
    case expenseGroup(ExpenseGroupFailure, detail: String)

    /// The library's `LanguageError`, with the message it words.
    case language(LanguageFailure, detail: String)

    /// A record whose derived text is not what the library derives from its other fields, so it
    /// names no value the library writes: the field it gave another way, and what went wrong.
    case mismatchedRecord(MismatchedField, detail: String)

    /// A saved group that is not JSON of the saved form, at `line` and `column`; the detail names
    /// the fault but never the text there, which can hold an account number.
    case invalidJSON(line: Int, column: Int, detail: String)

    /// The call was cancelled, by its task's cancellation, before it changed anything.
    case cancelled(detail: String)

    /// A read or write handle was released, so it runs the group's calls no more.
    case released(detail: String)

    /// A failure the library does not describe, such as a defect in the bindings.
    case unexpected(detail: String)

    /// What went wrong, as the library words it.
    public var detail: String {
        switch self {
        case .id(_, let detail),
             .paymentAccount(_, let detail),
             .calendar(_, let detail),
             .category(_, let detail),
             .contact(_, let detail),
             .money(_, let detail),
             .member(_, let detail),
             .expense(_, let detail),
             .settlement(_, let detail),
             .transfer(_, let detail),
             .expenseGroup(_, let detail),
             .language(_, let detail),
             .mismatchedRecord(_, let detail),
             .invalidJSON(_, _, let detail),
             .cancelled(let detail),
             .released(let detail),
             .unexpected(let detail):
            detail
        }
    }
}

extension EranosError: LocalizedError {
    /// The library's message.
    public var errorDescription: String? {
        detail
    }
}

extension EranosError {
    init(_ error: EranosGenerated.EranosError) {
        switch error {
        case let .Id(failure, detail):
            self = .id(IDFailure(failure), detail: detail)
        case let .PaymentAccount(failure, detail):
            self = .paymentAccount(PaymentAccountFailure(failure), detail: detail)
        case let .Calendar(failure, detail):
            self = .calendar(CalendarFailure(failure), detail: detail)
        case let .Category(failure, detail):
            self = .category(CategoryFailure(failure), detail: detail)
        case let .Contact(failure, detail):
            self = .contact(ContactFailure(failure), detail: detail)
        case let .Money(failure, detail):
            self = .money(MoneyFailure(failure), detail: detail)
        case let .Member(failure, detail):
            self = .member(MemberFailure(failure), detail: detail)
        case let .Expense(failure, detail):
            self = .expense(ExpenseFailure(failure), detail: detail)
        case let .Settlement(failure, detail):
            self = .settlement(SettlementFailure(failure), detail: detail)
        case let .Transfer(failure, detail):
            self = .transfer(TransferFailure(failure), detail: detail)
        case let .ExpenseGroup(failure, detail):
            self = .expenseGroup(ExpenseGroupFailure(failure), detail: detail)
        case let .Language(failure, detail):
            self = .language(LanguageFailure(failure), detail: detail)
        case let .MismatchedRecord(field, detail):
            self = .mismatchedRecord(MismatchedField(field), detail: detail)
        case let .InvalidJson(line, column, detail):
            self = .invalidJSON(line: Int(line), column: Int(column), detail: detail)
        case let .Cancelled(detail):
            self = .cancelled(detail: detail)
        case let .Released(detail):
            self = .released(detail: detail)
        }
    }
}
