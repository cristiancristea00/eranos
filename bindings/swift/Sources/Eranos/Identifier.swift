/// The identifier of an entity of type `Entity`: the text the group gave it.
///
/// An identifier is text of at most 64 bytes of UTF-8, not empty, with no control character and no
/// whitespace at either end; one the group generates is a UUID v7 in lowercase hyphenated form. The
/// library compares identifiers byte for byte, so case matters.
///
/// Every entity has an identifier type of its own, `Member.ID`, `Expense.ID`, and the like, so an
/// expense's identifier cannot stand where a member's is expected. An identifier converts to its
/// text and back, for storing it; the library checks the text when a call carries it.
public struct Identifier<Entity>: Hashable, Sendable, RawRepresentable, CustomStringConvertible {
    /// The identifier's text, as the library holds it.
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
