package eranos.examples.grandtour;

import eranos.CalendarDate;
import eranos.CategoryId;
import eranos.Cost;
import eranos.CurrencyCode;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.ExpenseGroup;
import eranos.ExpenseGroupFailure;
import eranos.ExpenseId;
import eranos.Member;
import eranos.MemberId;
import eranos.Money;

import java.util.List;
import java.util.Optional;
import java.util.function.Function;

/**
 * What the sections of the tour share: the members, categories, and expenses they name, the amounts
 * and dates of the trip, and the outcomes of the changes a section tries.
 */
final class Tour {
    private Tour() {}

    /** The members of the trip. */
    record People(
            MemberId ana,
            MemberId bob,
            MemberId carol,
            MemberId dan,
            MemberId eva,
            MemberId filip) {}

    /** The categories the group adds to the built-in one. */
    record Categories(CategoryId food, CategoryId lodging, CategoryId transport) {}

    /** The seven expenses, one for every way of dividing one. */
    record Expenses(
            ExpenseId apartment,
            ExpenseId fado,
            ExpenseId groceries,
            ExpenseId museum,
            ExpenseId seafood,
            ExpenseId tour,
            ExpenseId train) {}

    /** A value the library makes, or refuses to. */
    interface Make<Value> {
        /** Makes the value. */
        Value make() throws EranosException;
    }

    /** A change the library makes, or refuses to. */
    interface Change {
        /** Makes the change. */
        void make() throws EranosException;
    }

    static Money eur(String text) throws EranosException {
        return Money.parse(text, CurrencyCode.of("EUR"));
    }

    static Money ron(String text) throws EranosException {
        return Money.parse(text, CurrencyCode.of("RON"));
    }

    /** A date of the trip, in October 2026. */
    static CalendarDate october(int date) throws EranosException {
        return CalendarDate.of(2026, 10, date);
    }

    static String name(ExpenseGroup group, MemberId member) throws EranosException {
        return group.member(member).map(Member::firstName).orElse("?");
    }

    /** {@code lines}, one below the other in a table cell. */
    static String cell(List<String> lines) {
        return String.join("\n", lines);
    }

    /**
     * "valid: …" with {@code shown} of the value {@code make} makes, or "refused: …" with the
     * message.
     */
    static <Value> String outcome(Make<Value> make, Function<Value, String> shown) {
        try {
            return "valid: " + shown.apply(make.make());
        } catch (EranosException refused) {
            return "refused: " + refused.getMessage();
        }
    }

    /** What the library refuses {@code change} with, if it refuses it. */
    static Optional<EranosException> refusal(Change change) {
        try {
            change.make();
            return Optional.empty();
        } catch (EranosException refused) {
            return Optional.of(refused);
        }
    }

    /** The message of {@code refused}, or "accepted". */
    static String message(Optional<EranosException> refused) {
        return refused.map(Throwable::getMessage).orElse("accepted");
    }

    /** The failure of the group that {@code refused} holds, if it is the group's. */
    static Optional<ExpenseGroupFailure> groupFailure(Optional<EranosException> refused) {
        return refused.filter(EranosException.ExpenseGroup.class::isInstance)
                .map(EranosException.ExpenseGroup.class::cast)
                .map(EranosException.ExpenseGroup::failure);
    }

    /** The sum of {@code amounts}, all in {@code currency}. */
    static Money sum(CurrencyCode currency, List<Money> amounts) throws EranosException {
        Money total = new Money(0, currency);
        for (Money amount : amounts) {
            total = total.plus(amount);
        }
        return total;
    }

    /** The rate {@code cost} was entered at: 1 for a cost in the group currency. */
    static ExchangeRate rate(Cost cost) throws EranosException {
        return switch (cost) {
            case Cost.Local _ -> ExchangeRate.parse("1");
            case Cost.Converted converted -> converted.rate();
        };
    }
}
