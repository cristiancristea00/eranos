package eranos.examples.grandtour;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.grandtour.Tour.cell;
import static eranos.examples.grandtour.Tour.groupFailure;
import static eranos.examples.grandtour.Tour.message;
import static eranos.examples.grandtour.Tour.outcome;
import static eranos.examples.grandtour.Tour.refusal;

import eranos.Account;
import eranos.AccountDetails;
import eranos.Category;
import eranos.CategoryId;
import eranos.Currency;
import eranos.Email;
import eranos.EranosException;
import eranos.Group;
import eranos.GroupFailure;
import eranos.Iban;
import eranos.Language;
import eranos.Member;
import eranos.MemberId;
import eranos.PhoneNumber;
import eranos.Username;
import eranos.examples.grandtour.Tables.CheckRow;
import eranos.examples.grandtour.Tables.DetailRow;
import eranos.examples.grandtour.Tables.MemberRow;
import eranos.examples.grandtour.Tables.RefusalRow;
import eranos.examples.grandtour.Tour.Categories;
import eranos.examples.grandtour.Tour.People;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;
import java.util.stream.Stream;

/**
 * Sections 1 to 3 of the tour: the group and its details, the members with their contact details
 * and accounts, and the categories.
 */
final class Members {
    private Members() {}

    static Group theGroup() throws EranosException {
        section("1. The group");
        Group group =
                Group.builder(Currency.of("EUR"), "Lisbon")
                        .description("A week by the Atlantic")
                        .language(Language.parse("pt-pt"))
                        .build();
        Optional<EranosException> blank = refusal(() -> group.setName("   "));
        group.setName("Lisbon and Porto");
        group.setDescription("A week by the Atlantic, with a day in Porto");
        String language = group.language().map(Language::toString).orElse("");
        printTable(
                DetailRow.HEADERS,
                List.of(
                        new DetailRow("Name", group.name()),
                        new DetailRow("Currency", group.currency().toString()),
                        new DetailRow("Language", language),
                        new DetailRow("Description", group.description().orElse(""))),
                2);
        System.out.print(
                """
                The group started as Lisbon and was renamed when a day in Porto joined the plan.
                Every total is kept in its currency, %s. The language is a BCP 47 tag the apps
                show the group in, stored in canonical form: pt-pt became %s. A blank name is
                refused:
                  error: %s
                """
                        .formatted(group.currency(), language, message(blank)));

        check(Optional.of(new GroupFailure.EmptyName()).equals(groupFailure(blank)));
        check("Lisbon and Porto".equals(group.name()));
        check("pt-PT".equals(language));
        return group;
    }

    /** An account with its label, its kind, and its details. */
    static String describe(Account account) {
        return switch (account.details()) {
            case AccountDetails.Iban iban -> account.name() + ", IBAN " + iban.iban();
            case AccountDetails.Username username ->
                    account.name() + ", username " + username.username();
        };
    }

    static void printMembers(Group group) {
        List<MemberRow> rows = new ArrayList<>();
        for (Member member : group.members()) {
            rows.add(
                    new MemberRow(
                            cell(member.accounts().stream().map(Members::describe).toList()),
                            member.email().map(Email::toString).orElse(""),
                            Stream.concat(Stream.of(member.firstName()), member.lastName().stream())
                                    .collect(Collectors.joining(" ")),
                            member.phone().map(PhoneNumber::toString).orElse("")));
        }
        printTable(MemberRow.HEADERS, rows, 4);
    }

    static void validation(Group group) {
        System.out.print(
                """
                IBANs are checked against the SWIFT registry and their mod-97 check digits,
                emails against RFC 5321 and 5322, and phone numbers against their country's
                numbering plan. Required text must not be blank:
                """);
        List<CheckRow> rows = new ArrayList<>();
        for (String input :
                List.of("ro49 aaaa 1b31 0075 9384 0000", "RO49 AAAA 1B31 0075 9384 0001")) {
            rows.add(
                    new CheckRow(
                            "\"" + input + "\"",
                            "IBAN",
                            outcome(
                                    () -> Iban.parse(input),
                                    iban -> iban + ", country " + iban.countryCode())));
        }
        rows.add(
                new CheckRow(
                        "\"ana..popescu@example.com\"",
                        "email",
                        outcome(() -> Email.parse("ana..popescu@example.com"), Email::toString)));
        for (String input : List.of("+351 912 345 678", "+40 612 345 678")) {
            rows.add(
                    new CheckRow(
                            "\"" + input + "\"",
                            "phone",
                            outcome(
                                    () -> PhoneNumber.parse(input),
                                    phone -> phone + ", stored as " + phone.e164())));
        }
        rows.add(
                new CheckRow(
                        "\"   \"",
                        "username",
                        outcome(() -> Username.parse("   "), Username::toString)));
        rows.add(
                new CheckRow(
                        "\"  \"",
                        "first name",
                        outcome(
                                () -> group.addMember(Member.builder("  ").build()),
                                MemberId::value)));
        printTable(CheckRow.HEADERS, rows, 3);
    }

    static People members(Group group) throws EranosException {
        section("2. Members");
        Account bank =
                new Account(
                        new AccountDetails.Iban(Iban.parse("RO49 AAAA 1B31 0075 9384 0000")),
                        "ING");
        Account app = new Account(new AccountDetails.Username(Username.parse("@ana")), "Revolut");
        MemberId ana =
                group.addMember(
                        Member.builder("Ana")
                                .accounts(List.of(bank, app))
                                .email(Email.parse("ana@example.com"))
                                .lastName("Popescu")
                                .phone(PhoneNumber.parse("+40 712 345 678"))
                                .build());
        MemberId bob = group.addMember(Member.builder("Bob").build());
        MemberId carol = group.addMember(Member.builder("Carol").build());
        MemberId dan = group.addMember(Member.builder("Dan").build());
        MemberId eva = group.addMember(Member.builder("Eva").build());
        MemberId filip = group.addMember(Member.builder("Filip").build());
        // An edit starts from a copy of the member, which keeps its identifier.
        Member edited =
                group.member(bob).orElseThrow().toBuilder()
                        .lastName("Ionescu")
                        .email(Email.parse("bob@example.com"))
                        .build();
        Member before = group.replaceMember(edited);
        printMembers(group);
        System.out.print(
                """
                Only a first name is required. Ana gave every detail; Bob added his last name and
                email later, through an edit that kept his identifier:
                  %s
                Eva is Carol's daughter, and Filip was added by mistake.
                """
                        .formatted(bob));
        validation(group);

        check(before.lastName().isEmpty());
        check(Optional.of("Ionescu").equals(group.member(bob).flatMap(Member::lastName)));
        check(Optional.of(2).equals(group.member(ana).map(member -> member.accounts().size())));
        return new People(ana, bob, carol, dan, eva, filip);
    }

    static Categories addCategories(Group group) throws EranosException {
        section("3. Categories");
        CategoryId food = group.addCategory(new Category("Food"));
        CategoryId lodging = group.addCategory(new Category("Lodging"));
        CategoryId transport = group.addCategory(new Category("Transport"));
        Optional<EranosException> homonym = refusal(() -> group.addCategory(new Category("FOOD")));
        group.replaceCategory(group.category(transport).orElseThrow().renamed("Getting around"));
        Category other = group.categories().getFirst().renamed("Other");
        Optional<EranosException> builtIn = refusal(() -> group.replaceCategory(other));
        System.out.print(
                """
                Every group has the built-in Uncategorised; this one adds Food, Lodging, and
                Transport, then renames Transport, which keeps its identifier. In identifier order:
                  %s
                Names are compared caselessly, and the built-in category keeps its name:
                """
                        .formatted(
                                group.categories().stream()
                                        .map(Category::name)
                                        .collect(Collectors.joining(", "))));
        printTable(
                RefusalRow.HEADERS,
                List.of(
                        new RefusalRow("Add a category named FOOD", message(homonym)),
                        new RefusalRow("Rename Uncategorised to Other", message(builtIn))),
                2);

        check(
                Optional.of(new GroupFailure.DuplicateCategoryName(food))
                        .equals(groupFailure(homonym)));
        check(Optional.of(new GroupFailure.BuiltInCategory()).equals(groupFailure(builtIn)));
        check(Optional.of("Getting around").equals(group.category(transport).map(Category::name)));
        return new Categories(food, lodging, transport);
    }
}
