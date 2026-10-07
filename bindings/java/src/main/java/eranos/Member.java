package eranos;

import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * A person in the group: a first name, and optionally more ways to reach and pay them.
 *
 * <p>A member has an identifier from the moment their builder starts: the one given to {@link
 * #builder(MemberId, String)}, or one that {@link #builder(String)} generates, and keeps it through
 * every edit. To change a member, read them from the group, edit them through {@link #toBuilder},
 * and pass the result to {@link ExpenseGroup#replaceMember}.
 */
public final class Member {
    private final MemberId id;
    private final String firstName;
    private final String lastName;
    private final Email email;
    private final PhoneNumber phone;
    private final List<PaymentAccount> accounts;

    private Member(Builder builder) {
        id = builder.id;
        firstName = builder.firstName;
        lastName = builder.lastName;
        email = builder.email;
        phone = builder.phone;
        accounts = builder.accounts;
    }

    /**
     * A builder of a member to add, called {@code firstName}, under an identifier generated when it
     * is called; {@link #builder(MemberId, String)} takes one instead.
     *
     * @param firstName the first name
     * @return the builder
     */
    public static Builder builder(String firstName) {
        return builder(MemberId.generate(), firstName);
    }

    /**
     * A builder of a member to add, called {@code firstName}, under the identifier {@code id}, such
     * as the key an app's own store gives the member; {@link #builder(String)} generates one
     * instead.
     *
     * @param id the identifier
     * @param firstName the first name
     * @return the builder
     */
    public static Builder builder(MemberId id, String firstName) {
        return new Builder(id, firstName);
    }

    /**
     * A builder that starts from this member and keeps their identifier, so the member it builds
     * can replace this one.
     *
     * @return the builder
     */
    public Builder toBuilder() {
        return new Builder(id, firstName)
                .lastName(lastName)
                .email(email)
                .phone(phone)
                .accounts(accounts);
    }

    /**
     * The member the generated code holds as {@code member}.
     *
     * @param member the generated member
     * @return the member
     */
    static Member converted(eranos.internal.Member member) {
        return new Builder(new MemberId(member.id()), member.firstName())
                .lastName(member.lastName())
                .email(null == member.email() ? null : Email.known(member.email()))
                .phone(null == member.phone() ? null : PhoneNumber.known(member.phone()))
                .accounts(member.accounts().stream().map(PaymentAccount::converted).toList())
                .build();
    }

    /**
     * The member as the generated code holds them.
     *
     * @return the generated member
     */
    eranos.internal.Member generated() {
        return new eranos.internal.Member(
                id.value(),
                firstName,
                lastName,
                null == email ? null : email.toString(),
                null == phone ? null : phone.e164(),
                accounts.stream().map(PaymentAccount::generated).toList());
    }

    /**
     * The identifier.
     *
     * @return the identifier
     */
    public MemberId id() {
        return id;
    }

    /**
     * The first name.
     *
     * @return the first name
     */
    public String firstName() {
        return firstName;
    }

    /**
     * The last name.
     *
     * @return the last name, if the member has one
     */
    public Optional<String> lastName() {
        return Optional.ofNullable(lastName);
    }

    /**
     * The email address.
     *
     * @return the address, if the member has one
     */
    public Optional<Email> email() {
        return Optional.ofNullable(email);
    }

    /**
     * The phone number.
     *
     * @return the number, if the member has one
     */
    public Optional<PhoneNumber> phone() {
        return Optional.ofNullable(phone);
    }

    /**
     * Where the member can be paid, in the order they were given.
     *
     * @return the accounts
     */
    public List<PaymentAccount> accounts() {
        return accounts;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Member member
                && id.equals(member.id)
                && firstName.equals(member.firstName)
                && Objects.equals(lastName, member.lastName)
                && Objects.equals(email, member.email)
                && Objects.equals(phone, member.phone)
                && accounts.equals(member.accounts);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, firstName, lastName, email, phone, accounts);
    }

    @Override
    public String toString() {
        return "Member[id="
                + id
                + ", firstName="
                + firstName
                + ", lastName="
                + lastName
                + ", email="
                + email
                + ", phone="
                + phone
                + ", accounts="
                + accounts
                + "]";
    }

    /** A builder of a member, whose every setter replaces what was set before. */
    public static final class Builder {
        private MemberId id;
        private String firstName;
        private String lastName;
        private Email email;
        private PhoneNumber phone;
        private List<PaymentAccount> accounts = List.of();

        private Builder(MemberId id, String firstName) {
            id(id).firstName(firstName);
        }

        /**
         * Sets the identifier.
         *
         * @param id the identifier
         * @return this builder
         */
        public Builder id(MemberId id) {
            this.id = Objects.requireNonNull(id, "id");
            return this;
        }

        /**
         * Sets the first name.
         *
         * @param firstName the first name
         * @return this builder
         */
        public Builder firstName(String firstName) {
            this.firstName = Objects.requireNonNull(firstName, "firstName");
            return this;
        }

        /**
         * Sets the last name; {@code null} leaves the member without one.
         *
         * @param lastName the last name, or {@code null}
         * @return this builder
         */
        public Builder lastName(String lastName) {
            this.lastName = lastName;
            return this;
        }

        /**
         * Sets the email address; {@code null} leaves the member without one.
         *
         * @param email the address, or {@code null}
         * @return this builder
         */
        public Builder email(Email email) {
            this.email = email;
            return this;
        }

        /**
         * Sets the phone number; {@code null} leaves the member without one.
         *
         * @param phone the number, or {@code null}
         * @return this builder
         */
        public Builder phone(PhoneNumber phone) {
            this.phone = phone;
            return this;
        }

        /**
         * Sets where the member can be paid, which it copies.
         *
         * @param accounts the accounts, in order
         * @return this builder
         */
        public Builder accounts(List<PaymentAccount> accounts) {
            this.accounts = List.copyOf(accounts);
            return this;
        }

        /**
         * The member; the group checks them when they are added or replace another.
         *
         * @return the member
         */
        public Member build() {
            return new Member(this);
        }
    }
}
