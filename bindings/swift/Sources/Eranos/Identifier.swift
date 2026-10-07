internal import EranosGenerated

/// The identifier of an entity of type `Entity`: the text the app gave it, or the one the library
/// generated.
///
/// An identifier is text of at most 64 bytes of UTF-8, not empty, with no control character and no
/// whitespace at either end, such as the key of the app's own store; one ``generate()`` makes is a
/// UUID v7 in lowercase hyphenated form. ``init(_:)`` checks a text by these rules, which every
/// kind of identifier shares, and ``init(rawValue:)`` keeps it unchecked, for one the app stored;
/// the library checks the text when a call carries it.
///
/// The library compares identifiers byte for byte, so case matters, and so does this type: two
/// texts that `String` holds equal, such as "é" written as one character or as "e" and a combining
/// accent, are two identifiers.
///
/// Every entity has an identifier type of its own, `Member.ID`, `Expense.ID`, and the like, so an
/// expense's identifier cannot stand where a member's is expected.
public struct Identifier<Entity>: Hashable, Sendable, RawRepresentable, CustomStringConvertible {
    /// The identifier's text, as the library holds it.
    public let rawValue: String

    /// The identifier written `text`, as it is: not trimmed, and in its case.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` naming the first rule the text breaks:
    ///   ``IDFailure/empty``, ``IDFailure/tooLong`` for more than 64 bytes of UTF-8,
    ///   ``IDFailure/controlCharacter``, and ``IDFailure/surroundingWhitespace``.
    public init(_ text: String) throws(EranosError) {
        rawValue = try converting { try EranosGenerated.identifierParse(text: text) }
    }

    /// The identifier written `rawValue`, unchecked.
    public init(rawValue: String) {
        self.rawValue = rawValue
    }

    /// A new identifier, which ``init(_:)`` accepts: a UUID v7 in lowercase hyphenated form.
    ///
    /// Every kind of identifier draws from the one generator of the library, so identifiers
    /// generated one after another are in increasing order, whichever kind each stands for.
    public static func generate() -> Self {
        Self(rawValue: EranosGenerated.identifierGenerate())
    }

    /// The identifier's text.
    public var description: String {
        rawValue
    }

    /// Whether `left` and `right` are the same identifier: the same bytes of UTF-8, as the library
    /// compares them.
    public static func == (left: Self, right: Self) -> Bool {
        left.rawValue.utf8.elementsEqual(right.rawValue.utf8)
    }

    /// Feeds the identifier's bytes of UTF-8 to `hasher`, so that equal identifiers hash alike.
    public func hash(into hasher: inout Hasher) {
        for byte in rawValue.utf8 {
            hasher.combine(byte)
        }
    }
}
