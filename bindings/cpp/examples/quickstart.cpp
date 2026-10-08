// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the Rust
// example `quickstart`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <cstdlib>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// A row of the table of expenses: what was bought, who paid it, who shares it, and its amount.
struct ExpenseRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<4> HEADERS{"Expense", "Paid by", "Shared by", "Amount"};

    /// What was bought.
    std::string name;
    /// Who paid it.
    std::string payers;
    /// Who shares it.
    std::string participants;
    /// What it cost.
    std::string amount;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] inline constexpr Cells<4> cells() const noexcept { return {name, payers, participants, amount}; }
};

/// A row of the table of members: what a member paid and consumed, and their balance.
struct MemberRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<4> HEADERS{"Member", "Paid", "Consumed", "Balance"};

    /// Their first name.
    std::string name;
    /// What they paid.
    std::string paid;
    /// What they consumed.
    std::string consumed;
    /// What is left: what they paid minus what they consumed.
    std::string balance;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] inline constexpr Cells<4> cells() const noexcept { return {name, paid, consumed, balance}; }
};

/// A row of the table of payments: who pays whom, and how much.
struct PaymentRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<3> HEADERS{"From", "To", "Amount"};

    /// Who pays.
    std::string sender;
    /// Who is paid.
    std::string receiver;
    /// How much is paid.
    std::string amount;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] inline constexpr Cells<3> cells() const noexcept { return {sender, receiver, amount}; }
};

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) noexcept { return must(Money::parse(text, must(CurrencyCode::of("EUR")))); }

/// Runs the example.
void run() noexcept {
    auto group = must(ExpenseGroup::create("Weekend in Sinaia", must(CurrencyCode::of("EUR"))));
    title(group.name());
    std::println("Three friends rent a cabin and buy groceries. The group keeps its books in {}, so\n"
                 "every total, balance, and payment below is in {}.",
                 group.currency().code(), group.currency().code());

    section("1. Members");
    auto const ana = must(group.addMember(Member("Ana")));
    auto const bob = must(group.addMember(Member("Bob")));
    auto const carol = must(group.addMember(Member("Carol")));
    std::println("{} people joined the group: {}.", group.members().size(), names(group, {ana, bob, carol}));

    section("2. Expenses");
    printLine("Each expense records who paid and who shares it. Here everyone shares everything\n"
              "equally.");
    auto const friday = must(CalendarDate::of(2026, 9, 25));
    EqualSplit const everyone{{ana, bob, carol}};
    static_cast<void>(must(group.addExpense(Expense("Cabin", LocalCost{euros("240.00")}, EqualSplit{{ana}}, WholeExpenseSplit{everyone}, friday))));
    static_cast<void>(
        must(group.addExpense(Expense("Groceries", LocalCost{euros("60.00")}, EqualSplit{{bob}}, WholeExpenseSplit{everyone}, friday))));
    std::vector<ExpenseRow> expenses;
    for (auto const &expense : group.expenses()) {
        expenses.push_back({
            .name = expense.name,
            .payers = names(group, must(participants(expense.payers))),
            .participants = names(group, sharedBy(expense.split)),
            .amount = must(total(expense)).toString(),
        });
    }
    printTable(expenses, 3);

    section("3. Who paid and who consumed");
    auto const summary = group.summary();
    std::println("The group spent {} in total. A member's share of an expense is what they consumed,\n"
                 "and their balance is what they paid minus what they consumed.",
                 summary.totalSpent.toString());
    std::vector<MemberRow> members;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id);
        if (found.has_value()) {
            members.push_back({
                .name = member.firstName,
                .paid = found->paid.toString(),
                .consumed = found->share.toString(),
                .balance = toString(found->balance),
            });
        }
    }
    printTable(members, 1);
    auto const anaTotals = totalsOf(summary, ana).value();
    std::println("Ana paid {} but consumed only {}, so the others owe her the difference.", anaTotals.paid.toString(), anaTotals.share.toString());

    section("4. Settling up");
    auto const settlement = must(group.settlement(SettlementStrategy::MINIMUM_TRANSFERS));
    std::println("The fewest payments that bring every balance to zero ({} in total):", settlement.payments.size());
    std::vector<PaymentRow> payments;
    payments.reserve(settlement.payments.size());
    for (auto const &payment : settlement.payments) {
        payments.push_back(
            {.sender = firstName(group, payment.sender), .receiver = firstName(group, payment.receiver), .amount = payment.amount.toString()});
    }
    printTable(payments, 2);
    printLine("Once these payments are made, nobody owes anything.");

    // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
    check(euros("300.00") == summary.totalSpent, "the group spent 300.00 EUR");
    check(Balance(GetsBackBalance{euros("140.00")}) == anaTotals.balance, "Ana gets back 140.00 EUR");
    check(2 == settlement.payments.size(), "two payments settle the group");
}

} // namespace
} // namespace eranos::examples

int main() noexcept {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
