internal import EranosGenerated

/// A person in the group: a first name, and optionally more ways to reach and pay them.
///
/// A member has an identifier from the moment they are made: the app's own, which
/// ``init(id:firstName:lastName:email:phone:accounts:)`` takes, or one that
/// ``init(firstName:lastName:email:phone:accounts:)`` generates.
public struct Member: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a member.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier, which the app gives: its own, or one ``Identifier/generate()`` makes.
    public var id: ID

    /// The first name.
    public var firstName: String

    /// The last name.
    public var lastName: String?

    /// The email address.
    public var email: Email?

    /// The phone number.
    public var phone: PhoneNumber?

    /// Where the member can be paid, in the order they were given.
    public var accounts: [PaymentAccount]

    var generated: EranosGenerated.Member {
        .init(
            id: id.rawValue,
            firstName: firstName,
            lastName: lastName,
            email: email?.description,
            phone: phone?.generated,
            accounts: accounts.map(\.generated)
        )
    }

    /// A member called `firstName`, under `id`, which the app gives: its own, or one
    /// ``Identifier/generate()`` makes; ``init(firstName:lastName:email:phone:accounts:)``
    /// generates it.
    public init(
        id: ID,
        firstName: String,
        lastName: String? = nil,
        email: Email? = nil,
        phone: PhoneNumber? = nil,
        accounts: [PaymentAccount] = []
    ) {
        self.id = id
        self.firstName = firstName
        self.lastName = lastName
        self.email = email
        self.phone = phone
        self.accounts = accounts
    }

    /// A member called `firstName`, under an identifier ``Identifier/generate()`` makes when this
    /// is called; ``init(id:firstName:lastName:email:phone:accounts:)`` takes the app's own.
    public init(
        firstName: String,
        lastName: String? = nil,
        email: Email? = nil,
        phone: PhoneNumber? = nil,
        accounts: [PaymentAccount] = []
    ) {
        self.init(
            id: .generate(),
            firstName: firstName,
            lastName: lastName,
            email: email,
            phone: phone,
            accounts: accounts
        )
    }

    init(_ member: EranosGenerated.Member) {
        self.init(
            id: ID(rawValue: member.id),
            firstName: member.firstName,
            lastName: member.lastName,
            email: member.email.map { Email(known: $0) },
            phone: member.phone.map { PhoneNumber($0) },
            accounts: member.accounts.map(PaymentAccount.init)
        )
    }
}

/// A place to send a member money, with a label: "Main account".
public struct PaymentAccount: Hashable, Sendable {
    /// The label.
    public var name: String

    /// The IBAN or the username.
    public var details: PaymentAccountDetails

    var generated: EranosGenerated.PaymentAccount {
        .init(name: name, details: details.generated)
    }

    /// The account labelled `name`, at `details`.
    public init(name: String, details: PaymentAccountDetails) {
        self.name = name
        self.details = details
    }

    init(_ account: EranosGenerated.PaymentAccount) {
        self.init(name: account.name, details: PaymentAccountDetails(account.details))
    }
}

/// Where a member can be paid: a bank account or a payment-app username.
public enum PaymentAccountDetails: Hashable, Sendable {
    /// A bank account.
    case iban(IBAN)

    /// A username in a payment app.
    case username(Username)

    var generated: EranosGenerated.PaymentAccountDetails {
        switch self {
        case let .iban(iban):
            .iban(iban: iban.description, countryCode: iban.countryCode)
        case let .username(username):
            .username(username: username.description)
        }
    }

    init(_ details: EranosGenerated.PaymentAccountDetails) {
        switch details {
        case let .iban(iban, countryCode):
            self = .iban(IBAN(EranosGenerated.Iban(iban: iban, countryCode: countryCode)))
        case let .username(username):
            self = .username(Username(known: username))
        }
    }
}
