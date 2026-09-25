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

    /// Where the member can be paid, in the order they were given.
    public var accounts: [Account]

    /// The email address.
    public var email: Email?

    /// The last name.
    public var lastName: String?

    /// The phone number.
    public var phone: PhoneNumber?

    var generated: EranosGenerated.Member {
        .init(
            id: id?.rawValue,
            firstName: firstName,
            accounts: accounts.map(\.generated),
            email: email?.description,
            lastName: lastName,
            phone: phone?.e164
        )
    }

    /// A member to add, called `firstName`.
    public init(
        firstName: String,
        accounts: [Account] = [],
        email: Email? = nil,
        lastName: String? = nil,
        phone: PhoneNumber? = nil
    ) {
        self.firstName = firstName
        self.accounts = accounts
        self.email = email
        self.lastName = lastName
        self.phone = phone
    }

    init(_ member: EranosGenerated.Member) {
        self.init(
            firstName: member.firstName,
            accounts: member.accounts.map(Account.init),
            email: member.email.map { Email(known: $0) },
            lastName: member.lastName,
            phone: member.phone.map { PhoneNumber(known: $0) }
        )
        id = member.id.map { ID(rawValue: $0) }
    }
}

/// A place to send a member money, with a label: "Main account".
public struct Account: Hashable, Sendable {
    /// The IBAN or the username.
    public var details: AccountDetails

    /// The label.
    public var name: String

    var generated: EranosGenerated.Account {
        .init(details: details.generated, name: name)
    }

    /// The account at `details`, labelled `name`.
    public init(details: AccountDetails, name: String) {
        self.details = details
        self.name = name
    }

    init(_ account: EranosGenerated.Account) {
        self.init(details: AccountDetails(account.details), name: account.name)
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
