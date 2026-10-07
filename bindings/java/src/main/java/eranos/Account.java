package eranos;

import java.util.Objects;

/**
 * A place to send a member money, with a label: "Main account".
 *
 * @param name the label
 * @param details the IBAN or the username
 */
public record Account(String name, AccountDetails details) {
    /**
     * The account at {@code details}, labelled {@code name}.
     *
     * @param name the label
     * @param details the IBAN or the username
     */
    public Account {
        Objects.requireNonNull(name, "name");
        Objects.requireNonNull(details, "details");
    }

    /**
     * The account the generated code holds as {@code account}.
     *
     * @param account the generated account
     * @return the account
     */
    static Account converted(eranos.internal.Account account) {
        return new Account(account.name(), Conversions.converted(account.details()));
    }

    /**
     * The account as the generated code holds it.
     *
     * @return the generated account
     */
    eranos.internal.Account generated() {
        return new eranos.internal.Account(name, Conversions.generated(details));
    }
}
