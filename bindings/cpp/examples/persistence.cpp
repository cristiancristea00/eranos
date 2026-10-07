// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <cstdlib>
#include <print>
#include <string>
#include <string_view>
#include <vector>

namespace eranos::examples {
namespace {

/// A row of the table of balances: a member's balance before saving and after loading.
struct ComparisonRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<3> HEADERS{"Member", "Balance before saving", "Balance after loading"};

    /// The member's first name.
    std::string name;
    /// The balance before saving.
    std::string before;
    /// The balance after loading.
    std::string after;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<3> cells() const noexcept { return {name, before, after}; }
};

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) { return must(Money::parse(text, must(Currency::of("EUR")))); }

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
    auto const day = must(CalendarDate::of(2026, 9, 23));
    auto group = must(ExpenseGroup::create("Road trip", must(Currency::of("EUR"))));
    auto const ana = must(group.addMember(Member("Ana")));
    auto const bob = must(group.addMember(Member("Bob")));
    WholeExpenseSplit const both{EqualSplit{{ana, bob}}};
    static_cast<void>(must(group.addExpense(Expense("Fuel", LocalCost{euros("70.00")}, EqualSplit{{ana}}, both, day))));
    Expense lunch("Lunch", LocalCost{euros("30.00")}, EqualSplit{{bob}}, both, day);
    lunch.tip = Tip(PercentageTipValue{must(Percentage::parse("10"))}, TipDistribution::PROPORTIONAL);
    static_cast<void>(must(group.addExpense(lunch)));

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
    auto const loaded = must(ExpenseGroup::load(saved));
    printLine("Loading replays every entry through the same checks as adding it, then rebuilds\n"
              "the totals. The loaded group equals the saved one, and so do its balances:");
    auto const before = group.summary();
    auto const after = loaded.summary();
    std::vector<ComparisonRow> rows;
    for (auto const &member : {ana, bob}) {
        rows.push_back({.name = firstName(group, member), .before = balanceOf(before, member), .after = balanceOf(after, member)});
    }
    printTable(rows, 1);
    auto const plan = must(loaded.settlement(SettlementStrategy::GREEDY));
    std::println("Payments needed to settle the loaded group: {}.", plan.payments.size());

    section("3. A corrupted file is refused");
    printLine("Here Ana's identifier was edited in her member entry, so the fuel's payer is an\n"
              "identifier no member has. Reading the file as a SavedExpenseGroup and loading it\n"
              "with into_expense_group replays every entry through the same checks as adding it,\n"
              "and refuses the file instead of producing balances that do not add up:");
    // Ana's own entry is the first to name her: the members come before the expenses in the saved form, so the fuel she
    // paid is left naming an identifier no member has.
    auto const corrupted = replaceFirst(saved, ana.value(), "0199a000-0000-7000-8000-00000000ffff");
    auto const refused = refusal(ExpenseGroup::load(corrupted));
    check(refused.has_value(), "the file is refused");
    std::println("  error: {}", message(refused));

    check(saved == loaded.save(), "the loaded group saves as the group did");
    check(before == after, "the loaded group has the group's totals");
    check(1 == plan.payments.size(), "one payment settles the loaded group");
}

} // namespace
} // namespace eranos::examples

int main() {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
