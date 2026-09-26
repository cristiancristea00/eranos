// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the Rust
// example `quickstart`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <optional>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) { return Money::parse(text, Currency::of("EUR")); }

/// Runs the example.
void run() {
    auto group = Group::create(Currency::of("EUR"), "Weekend in Sinaia");
    title(group.name());
    printLine("Three friends rent a cabin and buy groceries. The group keeps its books in " + group.currency().code() + ", so\n" +
              "every total, balance, and payment below is in " + group.currency().code() + ".");

    section("1. Members");
    auto const ana = group.addMember(Member("Ana"));
    auto const bob = group.addMember(Member("Bob"));
    auto const carol = group.addMember(Member("Carol"));
    printLine(std::to_string(group.members().size()) + " people joined the group: " + names(group, {ana, bob, carol}) + ".");

    section("2. Expenses");
    printLine("Each expense records who paid and who shares it. Here everyone shares everything\n"
              "equally.");
    auto const friday = CalendarDate::of(25, 9, 2026);
    EqualSplit const everyone{{ana, bob, carol}};
    static_cast<void>(group.addExpense(Expense(LocalCost{euros("240.00")}, friday, "Cabin", EqualSplit{{ana}}, WholeExpenseSplit{everyone})));
    static_cast<void>(group.addExpense(Expense(LocalCost{euros("60.00")}, friday, "Groceries", EqualSplit{{bob}}, WholeExpenseSplit{everyone})));
    Rows expenses;
    for (auto const &expense : group.expenses()) {
        expenses.push_back(
            {expense.name, names(group, participants(expense.payers)), names(group, sharedBy(expense.split)), total(expense).toString()});
    }
    printTable({"Expense", "Paid by", "Shared by", "Amount"}, expenses, 3);

    section("3. Who paid and who consumed");
    auto const summary = group.summary();
    printLine("The group spent " + summary.totalSpent.toString() + " in total. A member's share of an expense is what they consumed,\n" +
              "and their balance is what they paid minus what they consumed.");
    Rows members;
    for (auto const &member : group.members()) {
        auto const found = member.id.has_value() ? totalsOf(summary, *member.id) : std::nullopt;
        if (found.has_value()) {
            members.push_back({member.firstName, found->paid.toString(), found->share.toString(), toString(found->balance)});
        }
    }
    printTable({"Member", "Paid", "Consumed", "Balance"}, members, 1);
    auto const anaTotals = totalsOf(summary, ana).value();
    printLine("Ana paid " + anaTotals.paid.toString() + " but consumed only " + anaTotals.share.toString() +
              ", so the others owe her the difference.");

    section("4. Settling up");
    auto const settlement = group.settlement(SettlementStrategy::MINIMUM_TRANSFERS);
    printLine("The fewest payments that bring every balance to zero (" + std::to_string(settlement.payments.size()) + " in total):");
    Rows payments;
    for (auto const &payment : settlement.payments) {
        payments.push_back({firstName(group, payment.from), firstName(group, payment.to), payment.amount.toString()});
    }
    printTable({"From", "To", "Amount"}, payments, 2);
    printLine("Once these payments are made, nobody owes anything.");

    // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
    check(euros("300.00") == summary.totalSpent, "the group spent 300.00 EUR");
    check(Balance(GetsBackBalance{euros("140.00")}) == anaTotals.balance, "Ana gets back 140.00 EUR");
    check(2 == settlement.payments.size(), "two payments settle the group");
}

} // namespace
} // namespace eranos::examples

int main() { return eranos::examples::runPort(eranos::examples::run); }
