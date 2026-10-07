internal import EranosGenerated

/// A person in the group: a first name, and optionally more ways to reach and pay them.
///
/// A new member has no identifier; `Group.add(_:)` gives them one.
public struct Member: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a member.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier: absent for a member to add, present for one the group holds.
    public private(set) var id: ID?

    /// The first name.
    public var firstName: String

    /// The last name.
    public var lastName: String?

    /// The email address.
    public var email: Email?

    /// The phone number.
    public var phone: PhoneNumber?

    /// Where the member can be paid, in the order they were given.
    public var accounts: [Account]

    var generated: EranosGenerated.Member {
        .init(
            id: id?.rawValue,
            firstName: firstName,
            lastName: lastName,
            email: email?.description,
            phone: phone?.e164,
            accounts: accounts.map(\.generated)
        )
    }

    /// A member to add, called `firstName`.
    public init(
        firstName: String,
        lastName: String? = nil,
        email: Email? = nil,
        phone: PhoneNumber? = nil,
        accounts: [Account] = []
    ) {
        self.firstName = firstName
        self.lastName = lastName
        self.email = email
        self.phone = phone
        self.accounts = accounts
    }

    init(_ member: EranosGenerated.Member) {
        self.init(
            firstName: member.firstName,
            lastName: member.lastName,
            email: member.email.map { Email(known: $0) },
            phone: member.phone.map { PhoneNumber(known: $0) },
            accounts: member.accounts.map(Account.init)
        )
        id = member.id.map { ID(rawValue: $0) }
    }
}

/// A place to send a member money, with a label: "Main account".
public struct Account: Hashable, Sendable {
    /// The label.
    public var name: String

    /// The IBAN or the username.
    public var details: AccountDetails

    var generated: EranosGenerated.Account {
        .init(name: name, details: details.generated)
    }

    /// The account labelled `name`, at `details`.
    public init(name: String, details: AccountDetails) {
        self.name = name
        self.details = details
    }

    init(_ account: EranosGenerated.Account) {
        self.init(name: account.name, details: AccountDetails(account.details))
    }
}

/// Where a member can be paid: a bank account or a payment-app username.
public enum AccountDetails: Hashable, Sendable {
    /// A bank account.
    case iban(IBAN)

    /// A username in a payment app.
    case username(Username)

    var generated: EranosGenerated.AccountDetails {
        switch self {
        case let .iban(iban):
            .iban(iban: iban.description)
        case let .username(username):
            .username(username: username.description)
        }
    }

    init(_ details: EranosGenerated.AccountDetails) {
        switch details {
        case let .iban(iban):
            self = .iban(IBAN(known: iban))
        case let .username(username):
            self = .username(Username(known: username))
        }
    }
}
