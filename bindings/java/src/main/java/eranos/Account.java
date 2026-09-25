package eranos;

import java.util.Objects;

/**
 * A place to send a member money, with a label: "Main account".
 *
 * @param details the IBAN or the username
 * @param name the label
 */
public record Account(AccountDetails details, String name) {
    /**
     * The account at {@code details}, labelled {@code name}.
     *
     * @param details the IBAN or the username
     * @param name the label
     */
    public Account {
        Objects.requireNonNull(details, "details");
        Objects.requireNonNull(name, "name");
    }

    /**
     * The account the generated code holds as {@code account}.
     *
     * @param account the generated account
     * @return the account
     */
    static Account converted(eranos.internal.Account account) {
        return new Account(Conversions.converted(account.details()), account.name());
    }

    /**
     * The account as the generated code holds it.
     *
     * @return the generated account
     */
    eranos.internal.Account generated() {
        return new eranos.internal.Account(Conversions.generated(details), name);
    }
}
