package eranos

/**
 * A person in the group: a first name, and optionally more ways to reach and pay them.
 *
 * A new member has no identifier; [ExpenseGroup.add] gives them one. To change a member, read them from the group, [copy] them with
 * the changes, and pass the copy to [ExpenseGroup.replace].
 */
public class Member internal constructor(
    /** The identifier: `null` for a member to add, set for one the group holds. */
    public val id: MemberId?,
    /** The first name. */
    public val firstName: String,
    /** The last name. */
    public val lastName: String?,
    /** The email address. */
    public val email: Email?,
    /** The phone number. */
    public val phone: PhoneNumber?,
    accounts: List<Account>,
) {
    /** Where the member can be paid, in the order they were given. */
    public val accounts: List<Account> = accounts.toList()

    /** A member to add, called [firstName]. */
    public constructor(
        firstName: String,
        lastName: String? = null,
        email: Email? = null,
        phone: PhoneNumber? = null,
        accounts: List<Account> = emptyList(),
    ) : this(null, firstName, lastName, email, phone, accounts)

    /** The member as the generated code holds them. */
    internal val generated: eranos.internal.Member
        get() =
            eranos.internal.Member(
                id?.value,
                firstName,
                lastName,
                email?.toString(),
                phone?.e164,
                accounts.map { it.generated },
            )

    /** This member with the given details changed; the identifier stays, so the copy can replace the member. */
    public fun copy(
        firstName: String = this.firstName,
        lastName: String? = this.lastName,
        email: Email? = this.email,
        phone: PhoneNumber? = this.phone,
        accounts: List<Account> = this.accounts,
    ): Member = Member(id, firstName, lastName, email, phone, accounts)

    /**
     * Whether [other] is of the same class, with the same [id], [firstName], [lastName], [email], [phone], and [accounts].
     */
    override fun equals(other: Any?): Boolean =
        other is Member &&
            id == other.id &&
            firstName == other.firstName &&
            lastName == other.lastName &&
            email == other.email &&
            phone == other.phone &&
            accounts == other.accounts

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, firstName, lastName, email, phone, accounts)

    /** A description of every field, for logs. */
    override fun toString(): String = "Member(id=$id, firstName=$firstName, lastName=$lastName, email=$email, phone=$phone, accounts=$accounts)"
}

/** The member the generated code holds as [this]. */
internal fun eranos.internal.Member.converted(): Member =
    Member(
        id?.let(::MemberId),
        firstName,
        lastName,
        email?.let(::Email),
        phone?.let(::PhoneNumber),
        accounts.map { it.converted() },
    )

/** A place to send a member money, with a label: "Main account". */
public class Account(
    /** The label. */
    public val name: String,
    /** The IBAN or the username. */
    public val details: AccountDetails,
) {
    /** The account as the generated code holds it. */
    internal val generated: eranos.internal.Account
        get() = eranos.internal.Account(name, details.generated)

    /** Whether [other] is of the same class, with the same [name] and [details]. */
    override fun equals(other: Any?): Boolean = other is Account && name == other.name && details == other.details

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(name, details)

    /** A description of every field, for logs. */
    override fun toString(): String = "Account(name=$name, details=$details)"
}

/** The account the generated code holds as [this]. */
internal fun eranos.internal.Account.converted(): Account = Account(name, details.converted())

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
