package eranos.examples.grandtour;

import eranos.CalendarDate;
import eranos.CategoryId;
import eranos.Cost;
import eranos.Currency;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.Group;
import eranos.GroupFailure;
import eranos.Member;
import eranos.MemberId;
import eranos.Money;

import java.util.List;
import java.util.Optional;
import java.util.function.Function;

/**
 * What the sections of the tour share: the members and categories they name, the amounts and dates
 * of the trip, and the outcomes of the changes a section tries.
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
        return Money.parse(text, Currency.of("EUR"));
    }

    static Money ron(String text) throws EranosException {
        return Money.parse(text, Currency.of("RON"));
    }

    /** A date of the trip, in October 2026. */
    static CalendarDate october(int date) throws EranosException {
        return CalendarDate.of(date, 10, 2026);
    }

    static String name(Group group, MemberId member) throws EranosException {
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
    static Optional<GroupFailure> groupFailure(Optional<EranosException> refused) {
        return refused.filter(EranosException.Group.class::isInstance)
                .map(EranosException.Group.class::cast)
                .map(EranosException.Group::failure);
    }

    /** The sum of {@code amounts}, all in {@code currency}. */
    static Money sum(Currency currency, List<Money> amounts) throws EranosException {
        Money total = new Money(currency, 0);
        for (Money amount : amounts) {
            total = total.plus(amount);
        }
        return total;
    }

    /** The rate {@code cost} was entered at: 1 for a cost in the group currency. */
    static ExchangeRate rate(Cost cost) throws EranosException {
        return switch (cost) {
            case Cost.Converted converted -> converted.rate();
            case Cost.Local _ -> ExchangeRate.parse("1");
        };
    }
}
