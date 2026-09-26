package eranos.examples.grandtour;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.grandtour.Tour.groupFailure;
import static eranos.examples.grandtour.Tour.message;
import static eranos.examples.grandtour.Tour.refusal;

import eranos.Account;
import eranos.AccountDetails;
import eranos.EranosException;
import eranos.Group;
import eranos.GroupFailure;
import eranos.examples.grandtour.Tour.Categories;
import eranos.examples.grandtour.Tour.People;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/** Section 12 of the tour: the group saved as JSON, loaded again, and refused when changed. */
final class Saving {
    private Saving() {}

    /** A change to the saved form, and the text it makes of it. */
    private record Tampering(String change, String text) {}

    /** {@code text} with its first {@code found} replaced by {@code replacement}. */
    private static String replaceFirst(String text, String found, String replacement) {
        return text.replaceFirst(Pattern.quote(found), Matcher.quoteReplacement(replacement));
    }

    static void savingAndLoading(Group group, People people, Categories categories)
            throws EranosException {
        section("12. Saving and loading");
        String saved = group.save();
        System.out.print(
                """
                The group saves to %d bytes of JSON: its details and entries, never the totals.
                Ana's accounts are saved in clear text, as the library holds them:
                """
                        .formatted(saved.getBytes(StandardCharsets.UTF_8).length));
        for (Account account : group.member(people.ana()).orElseThrow().accounts()) {
            String details =
                    switch (account.details()) {
                        case AccountDetails.Iban iban -> iban.iban().toString();
                        case AccountDetails.Username username -> username.username().toString();
                    };
            check(saved.contains(details));
            System.out.println("  " + account.name() + ": " + details);
        }
        try (Group loaded = Group.load(saved)) {
            System.out.print(
                    """
                    Loading replays every entry through the same checks as adding it and rebuilds the
                    totals: the loaded group equals the saved one. Reading a SavedGroup and loading it
                    with into_group reports a refusal as a typed error instead of a message:
                    """);
            List<Tampering> tampered =
                    List.of(
                            new Tampering(
                                    "\"format\": 3",
                                    replaceFirst(saved, "\"format\": 2", "\"format\": 3")),
                            new Tampering(
                                    "a blank group name",
                                    replaceFirst(
                                            saved,
                                            "\"name\": \"Lisbon and Porto\"",
                                            "\"name\": \" \"")),
                            new Tampering(
                                    "Lodging renamed FOOD",
                                    replaceFirst(
                                            saved, "\"name\": \"Lodging\"", "\"name\": \"FOOD\"")));
            List<List<String>> rows = new ArrayList<>();
            List<Optional<GroupFailure>> failures = new ArrayList<>();
            for (Tampering tampering : tampered) {
                check(!saved.equals(tampering.text()));
                Optional<EranosException> refused =
                        refusal(() -> Group.load(tampering.text()).close());
                check(refused.isPresent());
                rows.add(List.of(tampering.change(), message(refused)));
                failures.add(groupFailure(refused));
            }
            printTable(List.of("Change to the file", "Message"), rows, 2);

            check(saved.equals(loaded.save()));
            check(group.summary().equals(loaded.summary()));
            check(
                    List.of(
                                    Optional.of(new GroupFailure.UnsupportedFormat(3)),
                                    Optional.of(new GroupFailure.EmptyName()),
                                    Optional.of(
                                            new GroupFailure.DuplicateCategoryName(
                                                    categories.food())))
                            .equals(failures));
        }
    }
}
