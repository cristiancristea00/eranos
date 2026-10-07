package eranos.examples.membersandaccounts;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.totals;

import eranos.Account;
import eranos.AccountDetails;
import eranos.Balance;
import eranos.CalendarDate;
import eranos.Cost;
import eranos.Currency;
import eranos.Email;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseId;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.GroupFailure;
import eranos.Iban;
import eranos.Member;
import eranos.MemberId;
import eranos.MemberSummary;
import eranos.Money;
import eranos.PhoneNumber;
import eranos.Split;
import eranos.Summary;
import eranos.Username;
import eranos.examples.Report.Row;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.Stream;

/**
 * A flat's members with their contact and account details, what validation accepts of each, and a
 * flatmate who moves out. The port of the Rust example {@code members_and_accounts}, which prints
 * the same report.
 */
public final class Main {
    private Main() {}

    /** A value the library checks, made from its text. */
    private interface Parse<Value> {
        /** The value of {@code text}, which the library checks. */
        Value of(String text) throws EranosException;
    }

    /** A change to the group, which the library may refuse. */
    private interface Change {
        /** Makes the change. */
        void make() throws EranosException;
    }

    /** The two flatmates. */
    private record Flatmates(MemberId ana, MemberId bob) {}

    /** A member with their contact details and accounts, a row of the members' table. */
    private record MemberRow(String name, String email, String phone, String accounts)
            implements Row {
        static final List<String> HEADERS = List.of("Member", "Email", "Phone", "Accounts");

        @Override
        public List<String> cells() {
            return List.of(name, email, phone, accounts);
        }
    }

    /** An input with what it was checked as and the outcome, a row of the checks' table. */
    private record CheckRow(String input, String kind, String result) implements Row {
        static final List<String> HEADERS = List.of("Input", "Checked as", "Result");

        @Override
        public List<String> cells() {
            return List.of(input, kind, result);
        }
    }

    /** A member's status and totals, a row of the balances' table. */
    private record BalanceRow(String name, String status, Money consumed, Balance balance)
            implements Row {
        static final List<String> HEADERS = List.of("Member", "Status", "Consumed", "Balance");

        @Override
        public List<String> cells() {
            return List.of(name, status, consumed.toString(), balance.toString());
        }
    }

    /** An account with its label and its details. */
    private static String describe(Account account) {
        String details =
                switch (account.details()) {
                    case AccountDetails.Iban iban -> iban.iban().toString();
                    case AccountDetails.Username username -> username.username().toString();
                };
        return account.name() + ": " + details;
    }

    /**
     * The check of {@code input} as a {@code kind}: {@code valid: } and what {@code shown} writes
     * of the value {@code parse} makes of it, or {@code refused: } and the library's message.
     */
    private static <Value> CheckRow validated(
            String input, String kind, Parse<Value> parse, Function<Value, String> shown) {
        String result;
        try {
            result = "valid: " + shown.apply(parse.of(input));
        } catch (EranosException refusal) {
            result = "refused: " + refusal.getMessage();
        }
        return new CheckRow(input, kind, result);
    }

    /** What the library refuses {@code change} with. */
    private static EranosException refusal(Change change) {
        try {
            change.make();
        } catch (EranosException refused) {
            return refused;
        }
        throw new IllegalStateException("the change must be refused");
    }

    private static void printMembers(Group group) {
        List<MemberRow> rows = new ArrayList<>();
        for (Member member : group.members()) {
            rows.add(
                    new MemberRow(
                            Stream.concat(Stream.of(member.firstName()), member.lastName().stream())
                                    .collect(Collectors.joining(" ")),
                            member.email().map(Email::toString).orElse(""),
                            member.phone().map(PhoneNumber::toString).orElse(""),
                            member.accounts().stream()
                                    .map(Main::describe)
                                    .collect(Collectors.joining("\n"))));
        }
        printTable(MemberRow.HEADERS, rows, 4);
    }

    private static void printBalances(Group group) throws EranosException {
        Summary summary = group.summary();
        List<BalanceRow> rows = new ArrayList<>();
        for (Member member : group.members()) {
            Optional<MemberId> id = member.id();
            Optional<MemberSummary> found = id.flatMap(known -> totals(known, summary));
            if (id.isPresent() && found.isPresent()) {
                rows.add(
                        new BalanceRow(
                                member.firstName(),
                                group.isArchived(id.orElseThrow()) ? "archived" : "active",
                                found.orElseThrow().share(),
                                found.orElseThrow().balance()));
            }
        }
        printTable(BalanceRow.HEADERS, rows, 2);
    }

    private static Flatmates members(Group group) throws EranosException {
        section("1. Members");
        Account bank =
                new Account(
                        "ING",
                        new AccountDetails.Iban(Iban.parse("ro49 aaaa 1b31 0075 9384 0000")));
        Account app = new Account("Revolut", new AccountDetails.Username(Username.parse("@ana")));
        MemberId ana =
                group.addMember(
                        Member.builder("Ana")
                                .lastName("Popescu")
                                .email(Email.parse("ana@example.com"))
                                .phone(PhoneNumber.parse("+40 712 345 678"))
                                .accounts(List.of(bank, app))
                                .build());
        MemberId bob = group.addMember(Member.builder("Bob").build());
        printMembers(group);
        System.out.print(
                """
                The phone number was stored in E.164 form, +40712345678, and the table shows it
                grouped for reading; the IBAN was stored in its electronic form, uppercase and
                without spaces, the way banking apps accept it.
                """);
        return new Flatmates(ana, bob);
    }

    private static void validation() {
        section("2. What validation accepts");
        System.out.print(
                """
                IBANs are checked against the SWIFT registry: the country, its length, which
                positions hold digits and which letters, and the mod-97 check digits, which catch
                any single mistyped character. Emails are checked against the rules of RFC 5321
                and 5322, which refuse the doubled dots and stray commas of hurried typing, and
                phone numbers against their country's numbering plan, stored in E.164 form.
                """);
        List<CheckRow> rows = new ArrayList<>();
        for (String input :
                List.of(
                        "ro49 aaaa 1b31 0075 9384 0000",
                        "RO49 AAAA 1B31 0075 9384 0001",
                        "RO49 AAAA 1B31 0075",
                        "DE46 37O4 0044 0532 0130 00")) {
            rows.add(validated(input, "IBAN", Iban::parse, Iban::toString));
        }
        for (String input :
                List.of("ana@example.com", "ana.example.com", "ana..popescu@example.com")) {
            rows.add(validated(input, "email", Email::parse, Email::toString));
        }
        for (String input :
                List.of(
                        "+40 (712) 345-678",
                        "0712 345 678",
                        "+40 612 345 678",
                        "+44 (0)20 7946 0958")) {
            rows.add(validated(input, "phone", PhoneNumber::parse, PhoneNumber::e164));
        }
        printTable(CheckRow.HEADERS, rows, 3);
    }

    private static void movingOut(Group group, Flatmates flatmates) throws EranosException {
        section("3. A flatmate moves out");
        Currency euro = Currency.of("EUR");
        MemberId ana = flatmates.ana();
        MemberId bob = flatmates.bob();
        ExpenseSplit both = new ExpenseSplit.Whole(new Split.Equal(List.of(ana, bob)));
        ExpenseId rent =
                group.addExpense(
                        Expense.builder(
                                        "Rent",
                                        new Cost.Local(Money.parse("900.00", euro)),
                                        new Split.Equal(List.of(ana)),
                                        both,
                                        CalendarDate.of(2026, 9, 1))
                                .build());
        group.archiveMember(bob);
        System.out.print(
                """
                Bob shared September's rent, then moved out. He is archived, not removed: his
                history and his balance stay, and he can still pay Ana back.
                """);
        printBalances(group);

        Expense utilities =
                Expense.builder(
                                "Utilities",
                                new Cost.Local(Money.parse("60.00", euro)),
                                new Split.Equal(List.of(ana)),
                                both,
                                CalendarDate.of(2026, 10, 1))
                        .build();
        EranosException refused = refusal(() -> group.addExpense(utilities));
        System.out.println(
                "October's utilities cannot include him:\n  error: " + refused.getMessage());
        EranosException removal = refusal(() -> group.removeMember(bob));
        System.out.println(
                "Nor can he be removed while the rent references him:\n  error: "
                        + removal.getMessage());

        check(
                refused instanceof EranosException.Group archived
                        && new GroupFailure.ArchivedMember(bob).equals(archived.failure()));
        check(
                removal instanceof EranosException.Group inUse
                        && inUse.failure() instanceof GroupFailure.MemberInUse);
        check(
                Optional.of(new Balance.Owes(Money.parse("450.00", euro)))
                        .equals(totals(bob, group.summary()).map(MemberSummary::balance)));
        check(group.expense(rent).isPresent());
    }

    /**
     * Prints the report of the flat's members.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        try (Group group = Group.builder("Flat 4B", Currency.of("EUR")).build()) {
            System.out.println("Flat 4B: members and their accounts");
            System.out.println("═══════════════════════════════════");
            System.out.print(
                    """
                    Ana and Bob share a flat. A member needs only a first name; the last name,
                    email, phone, and the accounts they can be paid to are optional.
                    """);
            Flatmates flatmates = members(group);
            validation();
            movingOut(group, flatmates);
        }
    }
}
