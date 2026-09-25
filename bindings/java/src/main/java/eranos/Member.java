package eranos;

import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * A person in the group: a first name, and optionally more ways to reach and pay them.
 *
 * <p>A new member has no identifier; {@link Group#addMember} gives them one. To change a member,
 * read them from the group, edit them through {@link #toBuilder}, and pass the result to {@link
 * Group#replaceMember}.
 */
public final class Member {
    private final MemberId id;
    private final String firstName;
    private final List<Account> accounts;
    private final Email email;
    private final String lastName;
    private final PhoneNumber phone;

    private Member(Builder builder) {
        id = builder.id;
        firstName = builder.firstName;
        accounts = builder.accounts;
        email = builder.email;
        lastName = builder.lastName;
        phone = builder.phone;
    }

    /**
     * A builder of a member to add, called {@code firstName}.
     *
     * @param firstName the first name
     * @return the builder
     */
    public static Builder builder(String firstName) {
        return new Builder(null, firstName);
    }

    /**
     * A builder that starts from this member and keeps their identifier, so the member it builds
     * can replace this one.
     *
     * @return the builder
     */
    public Builder toBuilder() {
        return new Builder(id, firstName)
                .accounts(accounts)
                .email(email)
                .lastName(lastName)
                .phone(phone);
    }

    /**
     * The member the generated code holds as {@code member}.
     *
     * @param member the generated member
     * @return the member
     */
    static Member converted(eranos.internal.Member member) {
        return new Builder(
                        member.id() == null ? null : new MemberId(member.id()), member.firstName())
                .accounts(member.accounts().stream().map(Account::converted).toList())
                .email(member.email() == null ? null : Email.known(member.email()))
                .lastName(member.lastName())
                .phone(member.phone() == null ? null : PhoneNumber.known(member.phone()))
                .build();
    }

    /**
     * The member as the generated code holds them.
     *
     * @return the generated member
     */
    eranos.internal.Member generated() {
        return new eranos.internal.Member(
                id == null ? null : id.value(),
                firstName,
                accounts.stream().map(Account::generated).toList(),
                email == null ? null : email.toString(),
                lastName,
                phone == null ? null : phone.e164());
    }

    /**
     * The identifier.
     *
     * @return the identifier, empty for a member to add
     */
    public Optional<MemberId> id() {
        return Optional.ofNullable(id);
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
     * Where the member can be paid, in the order they were given.
     *
     * @return the accounts
     */
    public List<Account> accounts() {
        return accounts;
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
     * The last name.
     *
     * @return the last name, if the member has one
     */
    public Optional<String> lastName() {
        return Optional.ofNullable(lastName);
    }

    /**
     * The phone number.
     *
     * @return the number, if the member has one
     */
    public Optional<PhoneNumber> phone() {
        return Optional.ofNullable(phone);
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Member member
                && Objects.equals(id, member.id)
                && firstName.equals(member.firstName)
                && accounts.equals(member.accounts)
                && Objects.equals(email, member.email)
                && Objects.equals(lastName, member.lastName)
                && Objects.equals(phone, member.phone);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, firstName, accounts, email, lastName, phone);
    }

    @Override
    public String toString() {
        return "Member[id="
                + id
                + ", firstName="
                + firstName
                + ", accounts="
                + accounts
                + ", email="
                + email
                + ", lastName="
                + lastName
                + ", phone="
                + phone
                + "]";
    }

    /** A builder of a member, whose every setter replaces what was set before. */
    public static final class Builder {
        private final MemberId id;
        private String firstName;
        private List<Account> accounts = List.of();
        private Email email;
        private String lastName;
        private PhoneNumber phone;

        private Builder(MemberId id, String firstName) {
            this.id = id;
            firstName(firstName);
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
         * Sets where the member can be paid, which it copies.
         *
         * @param accounts the accounts, in order
         * @return this builder
         */
        public Builder accounts(List<Account> accounts) {
            this.accounts = List.copyOf(accounts);
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
         * The member; the group checks them when they are added or replace another.
         *
         * @return the member
         */
        public Member build() {
            return new Member(this);
        }
    }
}
