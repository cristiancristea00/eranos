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
/// } catch .group(.memberInUse(let expenses, _, let transfers), _) {
///     // The member still has entries; archive them instead.
/// } catch {
///     print(error.detail)
/// }
/// ```
public enum EranosError: Error, Sendable, Hashable {
    /// The library's `AccountError`, with the message it words.
    case account(AccountFailure, detail: String)

    /// The library's `CalendarError`, with the message it words.
    case calendar(CalendarFailure, detail: String)

    /// The call was cancelled, by its task's cancellation, before it changed anything.
    case cancelled(detail: String)

    /// The library's `CategoryError`, with the message it words.
    case category(CategoryFailure, detail: String)

    /// The library's `ContactError`, with the message it words.
    case contact(ContactFailure, detail: String)

    /// The library's `ExpenseError`, with the message it words.
    case expense(ExpenseFailure, detail: String)

    /// The library's `GroupError`, with the message it words.
    case group(GroupFailure, detail: String)

    /// The library's `IdError`, with the message it words.
    case id(IDFailure, detail: String)

    /// An entry to add carried an identifier; the group gives a new entry its own.
    case identifierGiven(detail: String)

    /// An entry to replace carried no identifier to find it by.
    case identifierMissing(detail: String)

    /// A saved group that is not JSON of the saved form, at `line` and `column`; the detail names
    /// the fault but never the text there, which can hold an account number.
    case invalidJSON(line: Int, column: Int, detail: String)

    /// The library's `LanguageError`, with the message it words.
    case language(LanguageFailure, detail: String)

    /// The library's `MemberError`, with the message it words.
    case member(MemberFailure, detail: String)

    /// The library's `MoneyError`, with the message it words.
    case money(MoneyFailure, detail: String)

    /// The read handle was released, so it reads the group no more.
    case released(detail: String)

    /// The library's `SettlementError`, with the message it words.
    case settlement(SettlementFailure, detail: String)

    /// The library's `TransferError`, with the message it words.
    case transfer(TransferFailure, detail: String)

    /// A failure the library does not describe, such as a defect in the bindings.
    case unexpected(detail: String)

    /// What went wrong, as the library words it.
    public var detail: String {
        switch self {
        case .account(_, let detail),
             .calendar(_, let detail),
             .cancelled(let detail),
             .category(_, let detail),
             .contact(_, let detail),
             .expense(_, let detail),
             .group(_, let detail),
             .id(_, let detail),
             .identifierGiven(let detail),
             .identifierMissing(let detail),
             .invalidJSON(_, _, let detail),
             .language(_, let detail),
             .member(_, let detail),
             .money(_, let detail),
             .released(let detail),
             .settlement(_, let detail),
             .transfer(_, let detail),
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
        case let .Account(detail, failure):
            self = .account(AccountFailure(failure), detail: detail)
        case let .Calendar(detail, failure):
            self = .calendar(CalendarFailure(failure), detail: detail)
        case let .Category(detail, failure):
            self = .category(CategoryFailure(failure), detail: detail)
        case let .Contact(detail, failure):
            self = .contact(ContactFailure(failure), detail: detail)
        case let .Expense(detail, failure):
            self = .expense(ExpenseFailure(failure), detail: detail)
        case let .Group(detail, failure):
            self = .group(GroupFailure(failure), detail: detail)
        case let .Id(detail, failure):
            self = .id(IDFailure(failure), detail: detail)
        case let .Language(detail, failure):
            self = .language(LanguageFailure(failure), detail: detail)
        case let .Member(detail, failure):
            self = .member(MemberFailure(failure), detail: detail)
        case let .Money(detail, failure):
            self = .money(MoneyFailure(failure), detail: detail)
        case let .Settlement(detail, failure):
            self = .settlement(SettlementFailure(failure), detail: detail)
        case let .Transfer(detail, failure):
            self = .transfer(TransferFailure(failure), detail: detail)
        case let .Cancelled(detail):
            self = .cancelled(detail: detail)
        case let .IdentifierGiven(detail):
            self = .identifierGiven(detail: detail)
        case let .IdentifierMissing(detail):
            self = .identifierMissing(detail: detail)
        case let .Released(detail):
            self = .released(detail: detail)
        case let .InvalidJson(column, detail, line):
            self = .invalidJSON(line: Int(line), column: Int(column), detail: detail)
        }
    }
}
