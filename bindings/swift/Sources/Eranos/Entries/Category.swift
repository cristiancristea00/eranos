internal import EranosGenerated

/// A label the group gives expenses: "Food".
///
/// A category has an identifier from the moment it is made: the app's own, which
/// ``init(id:name:)`` takes, or one that ``init(name:)`` generates. The built-in category,
/// Uncategorised, under ``Identifier/uncategorised``, holds every expense without another.
public struct Category: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a category.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier, which the app gives: its own, or one ``Identifier/generate()`` makes.
    public var id: ID

    /// The name, which no other category of the group shares.
    public var name: String

    var generated: EranosGenerated.Category {
        .init(id: id.rawValue, name: name)
    }

    /// A category called `name`, under `id`, which the app gives: its own, or one
    /// ``Identifier/generate()`` makes; ``init(name:)`` generates it.
    public init(id: ID, name: String) {
        self.id = id
        self.name = name
    }

    /// A category called `name`, under an identifier ``Identifier/generate()`` makes when this is
    /// called; ``init(id:name:)`` takes the app's own.
    public init(name: String) {
        self.init(id: .generate(), name: name)
    }

    init(_ category: EranosGenerated.Category) {
        self.init(id: ID(rawValue: category.id), name: category.name)
    }
}

// A generic type holds no stored static property, so the identifier's text lives in this global,
// which Swift initialises once, on first use, across threads; without it, every expense that
// takes the default category would call into the library.
/// The text of the built-in category's identifier, as the library holds it.
private let uncategorisedText = EranosGenerated.categoryIdUncategorised()

public extension Identifier where Entity == Category {
    /// The identifier of the built-in category, Uncategorised, which every group has and every
    /// expense is in until it is given another: the text of the nil UUID,
    /// `00000000-0000-0000-0000-000000000000`.
    static var uncategorised: Self {
        Self(rawValue: uncategorisedText)
    }
}
