package eranos;

import eranos.internal.Eranos;

/**
 * The conversions of the sealed interfaces between this package and the generated code, which an
 * interface cannot hold as package-private methods of its own. Every switch is exhaustive, so a new
 * variant on either side fails the build.
 */
final class Conversions {
    private Conversions() {}

    /**
     * The details the generated code holds as {@code details}.
     *
     * @param details the generated details
     * @return the details
     */
    static AccountDetails converted(eranos.internal.AccountDetails details) {
        return switch (details) {
            case eranos.internal.AccountDetails.Iban iban ->
                    new AccountDetails.Iban(Iban.known(iban.iban()));
            case eranos.internal.AccountDetails.Username username ->
                    new AccountDetails.Username(Username.known(username.username()));
        };
    }

    /**
     * The details as the generated code holds them.
     *
     * @param details the details
     * @return the generated details
     */
    static eranos.internal.AccountDetails generated(AccountDetails details) {
        return switch (details) {
            case AccountDetails.Iban iban ->
                    new eranos.internal.AccountDetails.Iban(iban.iban().toString());
            case AccountDetails.Username username ->
                    new eranos.internal.AccountDetails.Username(username.username().toString());
        };
    }

    /**
     * The balance the generated code holds as {@code balance}.
     *
     * @param balance the generated balance
     * @return the balance
     */
    static Balance converted(eranos.internal.Balance balance) {
        return switch (balance) {
            case eranos.internal.Balance.GetsBack getsBack ->
                    new Balance.GetsBack(Money.converted(getsBack.amount()));
            case eranos.internal.Balance.Owes owes ->
                    new Balance.Owes(Money.converted(owes.amount()));
            case eranos.internal.Balance.Settled _ -> new Balance.Settled();
        };
    }

    /**
     * The balance as the generated code holds it.
     *
     * @param balance the balance
     * @return the generated balance
     */
    static eranos.internal.Balance generated(Balance balance) {
        return switch (balance) {
            case Balance.GetsBack getsBack ->
                    new eranos.internal.Balance.GetsBack(getsBack.amount().generated());
            case Balance.Owes owes -> new eranos.internal.Balance.Owes(owes.amount().generated());
            case Balance.Settled _ -> new eranos.internal.Balance.Settled();
        };
    }

    /**
     * The balance as the library writes it; {@code description} if the library cannot format its
     * amount, which happens only for a currency it does not list.
     *
     * @param balance the balance
     * @param description the balance's description by its components
     * @return the text: "owes 12.00 EUR"
     */
    static String formatted(Balance balance, String description) {
        try {
            return Eranos.balanceFormat(generated(balance));
        } catch (eranos.internal.EranosException exception) {
            return description;
        }
    }

    /**
     * The cost the generated code holds as {@code cost}.
     *
     * @param cost the generated cost
     * @return the cost
     */
    static Cost converted(eranos.internal.Cost cost) {
        return switch (cost) {
            case eranos.internal.Cost.Converted converted ->
                    new Cost.Converted(
                            Money.converted(converted.amount()),
                            ExchangeRate.known(converted.rate()));
            case eranos.internal.Cost.Local local ->
                    new Cost.Local(Money.converted(local.amount()));
        };
    }

    /**
     * The cost as the generated code holds it.
     *
     * @param cost the cost
     * @return the generated cost
     */
    static eranos.internal.Cost generated(Cost cost) {
        return switch (cost) {
            case Cost.Converted converted ->
                    new eranos.internal.Cost.Converted(
                            converted.amount().generated(), converted.rate().toString());
            case Cost.Local local -> new eranos.internal.Cost.Local(local.amount().generated());
        };
    }

    /**
     * The expense split the generated code holds as {@code split}.
     *
     * @param split the generated split
     * @return the split
     */
    static ExpenseSplit converted(eranos.internal.ExpenseSplit split) {
        return switch (split) {
            case eranos.internal.ExpenseSplit.Itemised itemised ->
                    new ExpenseSplit.Itemised(
                            itemised.items().stream().map(Item::converted).toList());
            case eranos.internal.ExpenseSplit.Whole whole ->
                    new ExpenseSplit.Whole(converted(whole.split()));
        };
    }

    /**
     * The expense split as the generated code holds it.
     *
     * @param split the split
     * @return the generated split
     */
    static eranos.internal.ExpenseSplit generated(ExpenseSplit split) {
        return switch (split) {
            case ExpenseSplit.Itemised itemised ->
                    new eranos.internal.ExpenseSplit.Itemised(
                            itemised.items().stream().map(Item::generated).toList());
            case ExpenseSplit.Whole whole ->
                    new eranos.internal.ExpenseSplit.Whole(generated(whole.split()));
        };
    }

    /**
     * The split the generated code holds as {@code split}.
     *
     * @param split the generated split
     * @return the split
     */
    static Split converted(eranos.internal.Split split) {
        return switch (split) {
            case eranos.internal.Split.Amounts amounts ->
                    new Split.Amounts(
                            amounts.shares().stream().map(MemberAmount::converted).toList());
            case eranos.internal.Split.Equal equal ->
                    new Split.Equal(equal.members().stream().map(MemberId::new).toList());
            case eranos.internal.Split.Parts parts ->
                    new Split.Parts(parts.shares().stream().map(MemberParts::converted).toList());
            case eranos.internal.Split.Percentages percentages ->
                    new Split.Percentages(
                            percentages.shares().stream()
                                    .map(MemberPercentage::converted)
                                    .toList());
        };
    }

    /**
     * The split as the generated code holds it.
     *
     * @param split the split
     * @return the generated split
     */
    static eranos.internal.Split generated(Split split) {
        return switch (split) {
            case Split.Amounts amounts ->
                    new eranos.internal.Split.Amounts(
                            amounts.shares().stream().map(MemberAmount::generated).toList());
            case Split.Equal equal ->
                    new eranos.internal.Split.Equal(
                            equal.members().stream().map(MemberId::value).toList());
            case Split.Parts parts ->
                    new eranos.internal.Split.Parts(
                            parts.shares().stream().map(MemberParts::generated).toList());
            case Split.Percentages percentages ->
                    new eranos.internal.Split.Percentages(
                            percentages.shares().stream()
                                    .map(MemberPercentage::generated)
                                    .toList());
        };
    }

    /**
     * The tip's value the generated code holds as {@code value}.
     *
     * @param value the generated value
     * @return the value
     */
    static TipValue converted(eranos.internal.TipValue value) {
        return switch (value) {
            case eranos.internal.TipValue.Fixed fixed ->
                    new TipValue.Fixed(Money.converted(fixed.amount()));
            case eranos.internal.TipValue.Percentage percentage ->
                    new TipValue.Percentage(Percentage.known(percentage.basisPoints()));
        };
    }

    /**
     * The tip's value as the generated code holds it.
     *
     * @param value the value
     * @return the generated value
     */
    static eranos.internal.TipValue generated(TipValue value) {
        return switch (value) {
            case TipValue.Fixed fixed ->
                    new eranos.internal.TipValue.Fixed(fixed.amount().generated());
            case TipValue.Percentage percentage ->
                    new eranos.internal.TipValue.Percentage(percentage.percentage().basisPoints());
        };
    }
}
