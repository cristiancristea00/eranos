// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until everyone
// is settled. The port of the Rust example `settling_up`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <format>
#include <print>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos::examples {
namespace {

/// How many friends the festival's group holds: more open balances than the adaptive strategy settles exactly.
constexpr std::size_t FRIENDS = 21;

/// How many payments each strategy plans.
struct Counts final {
    /// The payments of the adaptive plan.
    std::size_t adaptive;
    /// The payments of the direct plan.
    std::size_t direct;
    /// The payments of the greedy plan.
    std::size_t greedy;
    /// The payments of the minimum plan.
    std::size_t minimum;
};

/// A row of the table of expenses: what was bought, who paid it, who shares it, and its amount.
struct ExpenseRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<4> HEADERS{"Expense", "Paid by", "Shared by", "Amount"};

    /// What it cost.
    std::string amount;
    /// What was bought.
    std::string name;
    /// Who shares it.
    std::string participants;
    /// Who paid it.
    std::string payer;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<4> cells() const noexcept { return {name, payer, participants, amount}; }
};

/// A row of the table of balances: what a member paid and consumed, and their balance.
struct BalanceRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<4> HEADERS{"Member", "Paid", "Consumed", "Balance"};

    /// What is left: what they paid minus what they consumed.
    std::string balance;
    /// What they consumed.
    std::string consumed;
    /// Their first name.
    std::string name;
    /// What they paid.
    std::string paid;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<4> cells() const noexcept { return {name, paid, consumed, balance}; }
};

/// A row of the table of payments: who pays whom, and how much.
struct PaymentRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<3> HEADERS{"From", "To", "Amount"};

    /// How much is paid.
    std::string amount;
    /// Who pays.
    std::string from;
    /// Who is paid.
    std::string to;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<3> cells() const noexcept { return {from, to, amount}; }
};

/// A row of the table of strategies: what a strategy guarantees, what it costs, and how many payments it plans here.
struct StrategyRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<4> HEADERS{"Strategy", "Guarantee", "Cost", "Payments here"};

    /// How long it takes.
    std::string_view cost;
    /// What it promises.
    std::string_view guarantee;
    /// How many payments it plans for the group.
    std::string payments;
    /// Its name.
    std::string_view strategy;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<4> cells() const noexcept { return {strategy, guarantee, cost, payments}; }
};

/// The name of @p strategy as the apps show it.
[[nodiscard]] std::string_view strategyName(SettlementStrategy const strategy) {
    switch (strategy) {
    case SettlementStrategy::ADAPTIVE:
        return "Adaptive";
    case SettlementStrategy::DIRECT:
        return "Direct";
    case SettlementStrategy::GREEDY:
        return "Greedy";
    case SettlementStrategy::MINIMUM_TRANSFERS:
        return "MinimumTransfers";
    }
    std::unreachable();
}

/// Records an expense of @p amount euros, named @p name, that @p payer paid and @p members share equally.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the name, then the amount, in the order the Rust example gives them.
void spend(Group &group, std::string const &name, std::string_view const amount, MemberId const &payer, std::vector<MemberId> members) {
    static_cast<void>(
        must(group.addExpense(Expense(LocalCost{must(Money::parse(amount, must(Currency::of("EUR"))))}, must(CalendarDate::of(20, 9, 2026)), name,
                                      EqualSplit{{payer}}, WholeExpenseSplit{EqualSplit{std::move(members)}}))));
}

/// Prints what each member of @p group paid and consumed, and their balance.
void printBalances(Group const &group) {
    auto const summary = group.summary();
    std::vector<BalanceRow> rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id.value()).value();
        rows.push_back({
            .balance = toString(found.balance),
            .consumed = found.share.toString(),
            .name = member.firstName,
            .paid = found.paid.toString(),
        });
    }
    printTable(rows, 1);
}

/// Prints the payments of @p settlement: who pays whom, and how much.
void printPayments(Group const &group, Settlement const &settlement) {
    std::vector<PaymentRow> rows;
    rows.reserve(settlement.payments.size());
    for (auto const &payment : settlement.payments) {
        rows.push_back({.amount = payment.amount.toString(), .from = firstName(group, payment.from), .to = firstName(group, payment.to)});
    }
    printTable(rows, 2);
}

/// Prints each strategy's plan with what it promises; returns the number of payments of each.
[[nodiscard]] Counts strategies(Group const &group) {
    section("3. Minimum transfers");
    auto const minimum = must(group.settlement(SettlementStrategy::MINIMUM_TRANSFERS));
    std::println("The fewest payments possible. Finding them is NP-hard; the solver does it exactly by\n"
                 "splitting the balances into as many groups that sum to zero as it can: here\n"
                 "{{Ana, Dan, Eva}} and {{Bob, Carol, Filip}}, each settled with one payment fewer than\n"
                 "its size. {} payments:",
                 minimum.payments.size());
    printPayments(group, minimum);

    section("4. Greedy");
    auto const greedy = must(group.settlement(SettlementStrategy::GREEDY));
    std::println("The largest debtor pays the largest creditor, again and again: fast, at most one\n"
                 "payment fewer than the members with a balance, but not always the minimum. {}\n"
                 "payments here:",
                 greedy.payments.size());
    printPayments(group, greedy);

    section("5. Direct");
    auto const direct = must(group.settlement(SettlementStrategy::DIRECT));
    std::println("No simplification: everyone pays back the people who paid for them, with the debts\n"
                 "between each pair of members netted. The most payments ({}), but each one traces\n"
                 "back to shared expenses:",
                 direct.payments.size());
    printPayments(group, direct);

    section("6. Adaptive");
    auto const adaptive = must(group.settlement(SettlementStrategy::ADAPTIVE));
    std::println("The fewest payments while they are quick to find, greedy's beyond: up to 20 open\n"
                 "balances the minimum solver settles them, past that greedy does, and the plan\n"
                 "names the strategy whose plan it is. Six balances are well within the limit, so\n"
                 "the plan is the one {} gives, the same {} payments.",
                 strategyName(adaptive.strategy), adaptive.payments.size());

    section("7. The strategies side by side");
    Counts const counts{
        .adaptive = adaptive.payments.size(),
        .direct = direct.payments.size(),
        .greedy = greedy.payments.size(),
        .minimum = minimum.payments.size(),
    };
    printTable<StrategyRow>(
        {
            {
                .cost = "O(2ⁿ·n) for n open balances",
                .guarantee = "the fewest payments",
                .payments = std::to_string(counts.minimum),
                .strategy = "MinimumTransfers",
            },
            {
                .cost = "O(n log n)",
                .guarantee = "at most n − 1 payments",
                .payments = std::to_string(counts.greedy),
                .strategy = "Greedy",
            },
            {
                .cost = "O(pairs)",
                .guarantee = "one payment per pair in debt",
                .payments = std::to_string(counts.direct),
                .strategy = "Direct",
            },
            {
                .cost = "as MinimumTransfers, then Greedy",
                .guarantee = "the fewest up to 20 balances",
                .payments = std::to_string(counts.adaptive),
                .strategy = "Adaptive",
            },
        },
        3);
    return counts;
}

/// Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly, and prints which
/// strategy's plan it took; returns the number of payments.
[[nodiscard]] std::size_t festival() {
    section("8. A larger group");
    auto const euro = must(Currency::of("EUR"));
    auto group = must(Group::create(euro, "Festival"));
    std::vector<MemberId> friends;
    friends.reserve(FRIENDS);
    for (std::size_t number = 1; FRIENDS >= number; ++number) {
        auto const member = must(group.addMember(Member(std::format("Friend {}", number))));
        friends.push_back(member);
    }
    // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's
    // balance is 21·i² − 3311 cents: never zero, and never the opposite of another's.
    auto const date = must(CalendarDate::of(18, 9, 2026));
    for (std::size_t index = 0; friends.size() > index; ++index) {
        auto const number = static_cast<std::int64_t>(index) + 1;
        Expense const expense(LocalCost{Money(euro, static_cast<std::int64_t>(FRIENDS) * number * number)}, date, "Round",
                              EqualSplit{{friends.at(index)}}, WholeExpenseSplit{EqualSplit{friends}});
        static_cast<void>(must(group.addExpense(expense)));
    }
    auto const summary = group.summary();
    auto const open =
        static_cast<std::size_t>(std::ranges::count_if(summary.members, [](MemberSummary const &totals) { return 0 != totals.net.minorUnits; }));
    auto const adaptive = must(group.settlement(SettlementStrategy::ADAPTIVE));
    auto const greedy = must(group.settlement(SettlementStrategy::GREEDY));
    std::println("Twenty-one friends at a festival each bought one round for everyone, each round\n"
                 "at another price, so all {} end with a balance and no two of them cancel out.\n"
                 "That is more than the 20 the adaptive strategy settles exactly, so it takes the\n"
                 "plan {} gives and says so: the same {} payments.",
                 open, strategyName(adaptive.strategy), adaptive.payments.size());
    check(FRIENDS == open, "every friend has a balance");
    check(SettlementStrategy::GREEDY == adaptive.strategy, "the adaptive plan is greedy's");
    check(greedy.payments == adaptive.payments, "the adaptive plan makes greedy's payments");
    return adaptive.payments.size();
}

/// Runs the example.
void run() {
    auto group = must(Group::create(must(Currency::of("EUR")), "Ski week"));
    auto const ana = must(group.addMember(Member("Ana")));
    auto const bob = must(group.addMember(Member("Bob")));
    auto const carol = must(group.addMember(Member("Carol")));
    auto const dan = must(group.addMember(Member("Dan")));
    auto const eva = must(group.addMember(Member("Eva")));
    auto const filip = must(group.addMember(Member("Filip")));

    title("Ski week: four ways to settle up");
    printLine("Six friends spent a week in the mountains. The library can turn their balances\n"
              "into payments in four ways, and the app picks one.");

    section("1. What was spent");
    spend(group, "Cabin", "12.00", dan, {dan, eva, filip});
    spend(group, "Lift passes", "9.00", bob, {bob, carol, filip});
    spend(group, "Groceries", "6.00", carol, {bob, dan});
    spend(group, "Coffee", "2.00", ana, {dan, eva});
    spend(group, "Snacks", "1.00", filip, {ana});
    std::vector<ExpenseRow> expenses;
    for (auto const &expense : group.expenses()) {
        expenses.push_back({
            .amount = must(total(expense)).toString(),
            .name = expense.name,
            .participants = names(group, sharedBy(expense.split)),
            .payer = names(group, must(participants(expense.payers))),
        });
    }
    printTable(expenses, 3);

    section("2. Balances");
    printLine("Four members are owed money and two owe it:");
    printBalances(group);

    auto const counts = strategies(group);
    auto const festivalPayments = festival();

    section("9. Settling up for real");
    for (auto const &payment : must(group.settlement(SettlementStrategy::MINIMUM_TRANSFERS)).payments) {
        static_cast<void>(
            must(group.addTransfer(Transfer(LocalCost{payment.amount}, must(CalendarDate::of(27, 9, 2026)), payment.from, payment.to))));
    }
    printLine("Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards\n"
              "every balance is zero and a new settlement has nothing left to pay:");
    printBalances(group);
    std::println("Payments still needed: {}.", must(group.settlement(SettlementStrategy::MINIMUM_TRANSFERS)).payments.size());

    // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice, and the adaptive plan is that one;
    // greedy pays the largest amounts first and needs five; the unsimplified debts between pairs need seven.
    check(4 == counts.adaptive && 7 == counts.direct && 5 == counts.greedy && 4 == counts.minimum,
          "the adaptive, direct, greedy, and minimum plans take 4, 7, 5, and 4 payments");
    // Greedy settles the festival's 21 balances in 19 payments.
    check(19 == festivalPayments, "the festival's plan takes 19 payments");
    auto const summary = group.summary();
    for (auto const &member : std::array<MemberId, 6>{ana, bob, carol, dan, eva, filip}) {
        check(Balance(SettledBalance{}) == totalsOf(summary, member).value().balance, "every member is settled");
    }
}

} // namespace
} // namespace eranos::examples

int main() {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
