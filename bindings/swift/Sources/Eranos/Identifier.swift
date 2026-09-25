/// The identifier of an entity of type `Entity`: a hyphenated UUID the group gave it.
///
/// Every entity has an identifier type of its own, `Member.ID`, `Expense.ID`, and the like, so an
/// expense's identifier cannot stand where a member's is expected. An identifier converts to its
/// text and back, for storing it; the library checks the text when a call carries it.
public struct Identifier<Entity>: Hashable, Sendable, RawRepresentable, CustomStringConvertible {
    /// The identifier's text: a hyphenated UUID.
    public let rawValue: String

    /// The identifier written `rawValue`.
    public init(rawValue: String) {
        self.rawValue = rawValue
    }

    /// The identifier's text.
    public var description: String {
        rawValue
    }
}
