internal import EranosGenerated

/// A label the group gives expenses: "Food".
///
/// A new category has no identifier; `ExpenseGroup.add(_:)` gives it one. The built-in
/// category, Uncategorised, holds every expense without another.
public struct Category: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a category.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier: absent for a category to add, present for one the group holds.
    public private(set) var id: ID?

    /// The name, which no other category of the group shares.
    public var name: String

    var generated: EranosGenerated.Category {
        .init(id: id?.rawValue, name: name)
    }

    /// A category to add, called `name`.
    public init(name: String) {
        self.name = name
    }

    init(_ category: EranosGenerated.Category) {
        self.init(name: category.name)
        id = category.id.map { ID(rawValue: $0) }
    }
}
