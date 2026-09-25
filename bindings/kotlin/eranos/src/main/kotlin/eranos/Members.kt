package eranos

/**
 * A person in the group: a first name, and optionally more ways to reach and pay them.
 *
 * A new member has no identifier; [Group.add] gives them one. To change a member, read them from the group, [copy] them with
 * the changes, and pass the copy to [Group.replace].
 */
public class Member internal constructor(
    /** The identifier: `null` for a member to add, set for one the group holds. */
    public val id: MemberId?,
    /** The first name. */
    public val firstName: String,
    accounts: List<Account>,
    /** The email address. */
    public val email: Email?,
    /** The last name. */
    public val lastName: String?,
    /** The phone number. */
    public val phone: PhoneNumber?,
) {
    /** Where the member can be paid, in the order they were given. */
    public val accounts: List<Account> = accounts.toList()

    /** A member to add, called [firstName]. */
    public constructor(
        firstName: String,
        accounts: List<Account> = emptyList(),
        email: Email? = null,
        lastName: String? = null,
        phone: PhoneNumber? = null,
    ) : this(null, firstName, accounts, email, lastName, phone)

    /** The member as the generated code holds them. */
    internal val generated: eranos.internal.Member
        get() =
            eranos.internal.Member(
                id?.value,
                firstName,
                accounts.map { it.generated },
                email?.toString(),
                lastName,
                phone?.e164,
            )

    /** This member with the given details changed; the identifier stays, so the copy can replace the member. */
    public fun copy(
        firstName: String = this.firstName,
        accounts: List<Account> = this.accounts,
        email: Email? = this.email,
        lastName: String? = this.lastName,
        phone: PhoneNumber? = this.phone,
    ): Member = Member(id, firstName, accounts, email, lastName, phone)

    /**
     * Whether [other] is of the same class, with the same [id], [firstName], [accounts], [email], [lastName], and [phone].
     */
    override fun equals(other: Any?): Boolean =
        other is Member &&
            id == other.id &&
            firstName == other.firstName &&
            accounts == other.accounts &&
            email == other.email &&
            lastName == other.lastName &&
            phone == other.phone

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, firstName, accounts, email, lastName, phone)

    /** A description of every field, for logs. */
    override fun toString(): String = "Member(id=$id, firstName=$firstName, accounts=$accounts, email=$email, lastName=$lastName, phone=$phone)"
}

/** The member the generated code holds as [this]. */
internal fun eranos.internal.Member.converted(): Member =
    Member(
        id?.let(::MemberId),
        firstName,
        accounts.map { it.converted() },
        email?.let(::Email),
        lastName,
        phone?.let(::PhoneNumber),
    )

/** A place to send a member money, with a label: "Main account". */
public class Account(
    /** The IBAN or the username. */
    public val details: AccountDetails,
    /** The label. */
    public val name: String,
) {
    /** The account as the generated code holds it. */
    internal val generated: eranos.internal.Account
        get() = eranos.internal.Account(details.generated, name)

    /** Whether [other] is of the same class, with the same [details] and [name]. */
    override fun equals(other: Any?): Boolean = other is Account && details == other.details && name == other.name

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(details, name)

    /** A description of every field, for logs. */
    override fun toString(): String = "Account(details=$details, name=$name)"
}

/** The account the generated code holds as [this]. */
internal fun eranos.internal.Account.converted(): Account = Account(details.converted(), name)

/** Where a member can be paid: a bank account or a payment-app username. */
public sealed interface AccountDetails {
    /** A bank account. */
    public class Iban(
        /** The account's IBAN. */
        public val iban: eranos.Iban,
    ) : AccountDetails {
        /** Whether [other] is of the same class, with the same [iban]. */
        override fun equals(other: Any?): Boolean = other is Iban && iban == other.iban

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = iban.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Iban(iban=$iban)"
    }

    /** A username in a payment app. */
    public class Username(
        /** The username. */
        public val username: eranos.Username,
    ) : AccountDetails {
        /** Whether [other] is of the same class, with the same [username]. */
        override fun equals(other: Any?): Boolean = other is Username && username == other.username

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = username.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Username(username=$username)"
    }
}

/** The account details as the generated code holds them. */
internal val AccountDetails.generated: eranos.internal.AccountDetails
    get() =
        when (this) {
            is AccountDetails.Iban -> eranos.internal.AccountDetails.Iban(iban.toString())
            is AccountDetails.Username -> eranos.internal.AccountDetails.Username(username.toString())
        }

/** The account details the generated code holds as [this]. */
internal fun eranos.internal.AccountDetails.converted(): AccountDetails =
    when (this) {
        is eranos.internal.AccountDetails.Iban -> AccountDetails.Iban(Iban(iban))
        is eranos.internal.AccountDetails.Username -> AccountDetails.Username(Username(username))
    }
