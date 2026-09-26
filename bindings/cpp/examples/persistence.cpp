// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <string>
#include <string_view>
#include <vector>

namespace eranos::examples {
namespace {

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) { return Money::parse(text, Currency::of("EUR")); }

/// The balance of @p member in @p summary, as the library writes it, or ? for a member without totals.
[[nodiscard]] std::string balanceOf(Summary const &summary, MemberId const &member) {
    auto const found = totalsOf(summary, member);
    return found.has_value() ? toString(found->balance) : "?";
}

/// @p text with the first @p pattern in it replaced by @p replacement.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the text, then what to find in it and what replaces it, as a search reads.
[[nodiscard]] std::string replaceFirst(std::string text, std::string_view const pattern, std::string_view const replacement) {
    auto const found = text.find(pattern);
    check(std::string::npos != found, "the text holds what the example replaces");
    return text.replace(found, pattern.size(), replacement);
}

/// Runs the example.
void run() {
    auto const day = CalendarDate::of(23, 9, 2026);
    auto group = Group::create(Currency::of("EUR"), "Road trip");
    auto const ana = group.addMember(Member("Ana"));
    auto const bob = group.addMember(Member("Bob"));
    WholeExpenseSplit const both{EqualSplit{{ana, bob}}};
    static_cast<void>(group.addExpense(Expense(LocalCost{euros("70.00")}, day, "Fuel", EqualSplit{{ana}}, both)));
    Expense lunch(LocalCost{euros("30.00")}, day, "Lunch", EqualSplit{{bob}}, both);
    lunch.tip = Tip(TipDistribution::PROPORTIONAL, PercentageTipValue{Percentage::parse("10")});
    static_cast<void>(group.addExpense(lunch));

    title("Road trip: saving and loading a group");
    printLine("With the `serde` feature a group serialises, and so does every type it stores.\n"
              "It saves only what was entered: its details, members, categories, expenses,\n"
              "and transfers. The totals, balances, and settlements are recomputed when it is\n"
              "loaded.");

    section("1. The saved form");
    // Account details would be in clear text: store this in the platform's secure storage (Keychain, Keystore, or an
    // encrypted database).
    auto const saved = group.save();
    printLine(saved);
    printLine("Amounts are minor units with their currency, dates are YYYY-MM-DD, and the\n"
              "`format` number lets a future release refuse a file it cannot read instead of\n"
              "dropping fields.");

    section("2. Loading it back");
    auto const loaded = Group::load(saved);
    printLine("Loading replays every entry through the same checks as adding it, then rebuilds\n"
              "the totals. The loaded group equals the saved one, and so do its balances:");
    auto const before = group.summary();
    auto const after = loaded.summary();
    Rows rows;
    for (auto const &member : {ana, bob}) {
        rows.push_back({firstName(group, member), balanceOf(before, member), balanceOf(after, member)});
    }
    printTable({"Member", "Balance before saving", "Balance after loading"}, rows, 1);
    auto const plan = loaded.settlement(SettlementStrategy::GREEDY);
    printLine("Payments needed to settle the loaded group: " + std::to_string(plan.payments.size()) + ".");

    section("3. A corrupted file is refused");
    printLine("Here the fuel's payer was edited to an identifier no member has. Reading the file\n"
              "as a SavedGroup and loading it with into_group replays every entry through the\n"
              "same checks as adding it, and refuses the file instead of producing balances that\n"
              "do not add up:");
    // The fuel is the first entry that names Ana: the expenses come before the members in the saved form.
    auto const corrupted = replaceFirst(saved, ana.value(), "0199a000-0000-7000-8000-00000000ffff");
    auto const refused = refusal([&] { return Group::load(corrupted); });
    check(refused.has_value(), "the file is refused");
    printLine("  error: " + message(refused));

    check(saved == loaded.save(), "the loaded group saves as the group did");
    check(before == after, "the loaded group has the group's totals");
    check(1 == plan.payments.size(), "one payment settles the loaded group");
}

} // namespace
} // namespace eranos::examples

int main() { return eranos::examples::runPort(eranos::examples::run); }
