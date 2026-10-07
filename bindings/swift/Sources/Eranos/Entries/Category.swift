internal import EranosGenerated

/// A label the group gives expenses: "Food".
///
/// A category to add carries the identifier the app gives it, or none, and then
/// `ExpenseGroup.add(_:)` returns one the library generates. The built-in category,
/// Uncategorised, holds every expense without another.
public struct Category: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a category.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier: the app's own for a category to add, or absent for the library to generate
    /// one, and present for a category the group holds or one to rename.
    public var id: ID?

    /// The name, which no other category of the group shares.
    public var name: String

    var generated: EranosGenerated.Category {
        .init(id: id?.rawValue, name: name)
    }

    /// A category to add, called `name`, under `id`, or one the library generates.
    public init(id: ID? = nil, name: String) {
        self.id = id
        self.name = name
    }

    init(_ category: EranosGenerated.Category) {
        self.init(id: category.id.map { ID(rawValue: $0) }, name: category.name)
    }
}
