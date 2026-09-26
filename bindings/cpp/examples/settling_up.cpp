// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until everyone
// is settled. The port of the Rust example `settling_up`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <array>
#include <cstddef>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos::examples {
namespace {

/// How many payments each strategy plans.
struct Counts final {
    /// The payments of the direct plan.
    std::size_t direct;
    /// The payments of the greedy plan.
    std::size_t greedy;
    /// The payments of the minimum plan.
    std::size_t minimum;
};

/// Records an expense of @p amount euros, named @p name, that @p payer paid and @p members share equally.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the name, then the amount, in the order the Rust example gives them.
void spend(Group &group, std::string const &name, std::string_view const amount, MemberId const &payer, std::vector<MemberId> members) {
    static_cast<void>(group.addExpense(Expense(LocalCost{Money::parse(amount, Currency::of("EUR"))}, CalendarDate::of(20, 9, 2026), name,
                                               EqualSplit{{payer}}, WholeExpenseSplit{EqualSplit{std::move(members)}})));
}

/// Prints what each member of @p group paid and consumed, and their balance.
void printBalances(Group const &group) {
    auto const summary = group.summary();
    Rows rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id.value()).value();
        rows.push_back({member.firstName, found.paid.toString(), found.share.toString(), toString(found.balance)});
    }
    printTable({"Member", "Paid", "Consumed", "Balance"}, rows, 1);
}

/// Prints the payments of @p settlement: who pays whom, and how much.
void printPayments(Group const &group, Settlement const &settlement) {
    Rows rows;
    for (auto const &payment : settlement.payments) {
        rows.push_back({firstName(group, payment.from), firstName(group, payment.to), payment.amount.toString()});
    }
    printTable({"From", "To", "Amount"}, rows, 2);
}

/// Prints each strategy's plan with what it promises.
[[nodiscard]] Counts strategies(Group const &group) {
    section("3. Minimum transfers");
    auto const minimum = group.settlement(SettlementStrategy::MINIMUM_TRANSFERS);
    printLine("The fewest payments possible. Finding them is NP-hard; the solver does it exactly by\n"
              "splitting the balances into as many groups that sum to zero as it can: here\n"
              "{Ana, Dan, Eva} and {Bob, Carol, Filip}, each settled with one payment fewer than\n"
              "its size. " +
              std::to_string(minimum.payments.size()) + " payments:");
    printPayments(group, minimum);

    section("4. Greedy");
    auto const greedy = group.settlement(SettlementStrategy::GREEDY);
    printLine("The largest debtor pays the largest creditor, again and again: fast, at most one\n"
              "payment fewer than the members with a balance, but not always the minimum. " +
              std::to_string(greedy.payments.size()) + "\npayments here:");
    printPayments(group, greedy);

    section("5. Direct");
    auto const direct = group.settlement(SettlementStrategy::DIRECT);
    printLine("No simplification: everyone pays back the people who paid for them, with the debts\n"
              "between each pair of members netted. The most payments (" +
              std::to_string(direct.payments.size()) + "), but each one traces\nback to shared expenses:");
    printPayments(group, direct);

    section("6. The strategies side by side");
    Counts const counts{direct.payments.size(), greedy.payments.size(), minimum.payments.size()};
    printTable({"Strategy", "Guarantee", "Cost", "Payments here"},
               {
                   {"MinimumTransfers", "the fewest payments", "O(2ⁿ·n) for n open balances", std::to_string(counts.minimum)},
                   {"Greedy", "at most n − 1 payments", "O(n log n)", std::to_string(counts.greedy)},
                   {"Direct", "one payment per pair in debt", "O(pairs)", std::to_string(counts.direct)},
               },
               3);
    return counts;
}

/// Runs the example.
void run() {
    auto group = Group::create(Currency::of("EUR"), "Ski week");
    auto const ana = group.addMember(Member("Ana"));
    auto const bob = group.addMember(Member("Bob"));
    auto const carol = group.addMember(Member("Carol"));
    auto const dan = group.addMember(Member("Dan"));
    auto const eva = group.addMember(Member("Eva"));
    auto const filip = group.addMember(Member("Filip"));

    printLine("Ski week: three ways to settle up\n"
              "═════════════════════════════════");
    printLine("Six friends spent a week in the mountains. The library can turn their balances\n"
              "into payments in three ways; the app picks one, and they differ here.");

    section("1. What was spent");
    spend(group, "Cabin", "12.00", dan, {dan, eva, filip});
    spend(group, "Lift passes", "9.00", bob, {bob, carol, filip});
    spend(group, "Groceries", "6.00", carol, {bob, dan});
    spend(group, "Coffee", "2.00", ana, {dan, eva});
    spend(group, "Snacks", "1.00", filip, {ana});
    Rows expenses;
    for (auto const &expense : group.expenses()) {
        expenses.push_back(
            {expense.name, names(group, participants(expense.payers)), names(group, sharedBy(expense.split)), total(expense).toString()});
    }
    printTable({"Expense", "Paid by", "Shared by", "Amount"}, expenses, 3);

    section("2. Balances");
    printLine("Four members are owed money and two owe it:");
    printBalances(group);

    auto const counts = strategies(group);

    section("7. Settling up for real");
    for (auto const &payment : group.settlement(SettlementStrategy::MINIMUM_TRANSFERS).payments) {
        static_cast<void>(group.addTransfer(Transfer(LocalCost{payment.amount}, CalendarDate::of(27, 9, 2026), payment.from, payment.to)));
    }
    printLine("Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards\n"
              "every balance is zero and a new settlement has nothing left to pay:");
    printBalances(group);
    printLine("Payments still needed: " + std::to_string(group.settlement(SettlementStrategy::MINIMUM_TRANSFERS).payments.size()) + ".");

    // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice; greedy pays the largest amounts
    // first and needs five; the unsimplified debts between pairs need seven.
    check(4 == counts.minimum && 5 == counts.greedy && 7 == counts.direct, "the plans take 4, 5, and 7 payments");
    auto const summary = group.summary();
    for (auto const &member : std::array<MemberId, 6>{ana, bob, carol, dan, eva, filip}) {
        check(Balance(SettledBalance{}) == totalsOf(summary, member).value().balance, "every member is settled");
    }
}

} // namespace
} // namespace eranos::examples

int main() { return eranos::examples::runPort(eranos::examples::run); }
