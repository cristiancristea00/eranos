// A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON and HUF carry a rate
// fixed when they are entered, and the group later moves its books to RON. The port of the Rust example
// `multi_currency`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <array>
#include <cstdlib>
#include <format>
#include <print>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// A row of the table of entries: an expense or a transfer, who paid it, and its amount as entered and in the group
/// currency.
struct EntryRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<5> HEADERS{"Entry", "Paid by", "Entered as", "Rate", "In the group currency"};

    /// Its amount in the group currency.
    std::string converted;
    /// What it was.
    std::string name;
    /// Its amount as entered.
    std::string original;
    /// Who paid it.
    std::string payer;
    /// The rate it was entered at.
    std::string rate;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<5> cells() const noexcept { return {name, payer, original, rate, converted}; }
};

/// A row of the table of balances: what a member paid, consumed, sent, and received, and their balance.
struct BalanceRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<6> HEADERS{"Member", "Paid", "Consumed", "Sent", "Received", "Balance"};

    /// What is left.
    std::string balance;
    /// What they consumed.
    std::string consumed;
    /// Their first name.
    std::string name;
    /// What they paid.
    std::string paid;
    /// What they received.
    std::string received;
    /// What they sent.
    std::string sent;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<6> cells() const noexcept { return {name, paid, consumed, sent, received, balance}; }
};

/// The amount @p text of @p code.
[[nodiscard]] Money amountOf(std::string_view const text, std::string_view const code) { return must(Money::parse(text, must(Currency::of(code)))); }

/// The amount of @p cost, as entered.
[[nodiscard]] Money amountOf(Cost const &cost) {
    return std::visit([](auto const &alternative) { return alternative.amount; }, cost);
}

/// The rate @p cost was entered at: 1 for a cost in the group currency.
[[nodiscard]] ExchangeRate rateOf(Cost const &cost) {
    auto const *const converted = std::get_if<ConvertedCost>(&cost);
    return nullptr == converted ? must(ExchangeRate::parse("1")) : converted->rate;
}

/// Prints every expense and transfer, as entered and in the group currency.
void printEntries(Group const &group) {
    Money const zero(group.currency(), 0);
    std::vector<EntryRow> rows;
    for (auto const &expense : group.expenses()) {
        auto const shares = must(group.expenseShares(expense.id.value()));
        rows.push_back({
            .converted = shares.has_value() ? shares->total.toString() : zero.toString(),
            .name = expense.name,
            .original = must(total(expense)).toString(),
            .payer = names(group, must(participants(expense.payers))),
            .rate = rateOf(expense.cost).toString(),
        });
    }
    for (auto const &transfer : group.transfers()) {
        auto const converted = must(group.transferAmount(transfer.id.value()));
        rows.push_back({
            .converted = converted.has_value() ? converted->toString() : zero.toString(),
            .name = std::format("Transfer to {}", firstName(group, transfer.to)),
            .original = amountOf(transfer.cost).toString(),
            .payer = firstName(group, transfer.from),
            .rate = rateOf(transfer.cost).toString(),
        });
    }
    printTable(rows, 2);
}

/// Prints what each member paid, consumed, sent, and received, and their balance.
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
            .received = found.received.toString(),
            .sent = found.sent.toString(),
        });
    }
    printTable(rows, 1);
}

/// Adds the fuel in RON, the dinner in HUF, and the ferry in EUR, which Ana and Bob share.
void expenses(Group &group, MemberId const &ana, MemberId const &bob) {
    section("1. Expenses in three currencies");
    WholeExpenseSplit const both{EqualSplit{{ana, bob}}};
    static_cast<void>(must(group.addExpense(Expense(ConvertedCost{.amount = amountOf("400.00", "RON"), .rate = must(ExchangeRate::parse("0.2"))},
                                                    must(CalendarDate::of(1, 8, 2026)), "Fuel", EqualSplit{{ana}}, both))));
    static_cast<void>(must(group.addExpense(Expense(ConvertedCost{.amount = amountOf("24000.00", "HUF"), .rate = must(ExchangeRate::parse("0.0025"))},
                                                    must(CalendarDate::of(2, 8, 2026)), "Dinner", EqualSplit{{bob}}, both))));
    static_cast<void>(
        must(group.addExpense(Expense(LocalCost{amountOf("60.00", "EUR")}, must(CalendarDate::of(3, 8, 2026)), "Ferry", EqualSplit{{bob}}, both))));
    printEntries(group);
    std::println("The rate says how many {} one unit of the entry's currency is worth: 400.00 RON at\n"
                 "0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.",
                 group.currency().code());
}

/// Adds Ana's repayment to Bob, in cash and in RON.
void repayment(Group &group, MemberId const &ana, MemberId const &bob) {
    section("3. A repayment in cash, in another currency");
    Transfer transfer(ConvertedCost{.amount = amountOf("100.00", "RON"), .rate = must(ExchangeRate::parse("0.2"))},
                      must(CalendarDate::of(4, 8, 2026)), ana, bob);
    transfer.note = "Cash at the border";
    static_cast<void>(must(group.addTransfer(transfer)));
    std::println("Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.\n"
                 "Transfers move balances but are not spending, so the total stays at {}:",
                 group.summary().totalSpent.toString());
    printBalances(group);
}

/// Moves the books of @p group to RON.
void newCurrency(Group &group) {
    section("4. Moving the books to another currency");
    printLine("Changing the group currency needs a rate into the new currency for every other\n"
              "currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies\n"
              "take the new rate; entries already in RON become local costs, at 1.");
    must(group.changeCurrency(must(Currency::of("RON")), std::array{
                                                             CurrencyRate(must(Currency::of("EUR")), must(ExchangeRate::parse("5"))),
                                                             CurrencyRate(must(Currency::of("HUF")), must(ExchangeRate::parse("0.0125"))),
                                                         }));
    printEntries(group);
    std::println("In {}, the trip cost {}, and everyone is still settled:", group.currency().code(), group.summary().totalSpent.toString());
    printBalances(group);
}

/// Runs the example.
void run() {
    auto group = must(Group::create(must(Currency::of("EUR")), "Danube road trip"));
    auto const ana = must(group.addMember(Member("Ana")));
    auto const bob = must(group.addMember(Member("Bob")));

    std::println("{}: expenses in several currencies\n"
                 "════════════════════════════════════════════════",
                 group.name());
    std::println("Ana and Bob drive from Romania to Hungary. The group keeps its books in {}; an\n"
                 "expense in another currency carries the exchange rate used when it was entered,\n"
                 "and is converted once, rounding half away from zero to the cent.",
                 group.currency().code());
    expenses(group, ana, bob);

    section("2. Balances");
    std::println("The group spent {}. Bob paid 120.00 of it but consumed half, so he gets back the\n"
                 "difference:",
                 group.summary().totalSpent.toString());
    printBalances(group);
    check(Balance(GetsBackBalance{amountOf("20.00", "EUR")}) == totalsOf(group.summary(), bob).value().balance, "Bob gets back 20.00 EUR");

    repayment(group, ana, bob);
    check(Balance(SettledBalance{}) == totalsOf(group.summary(), bob).value().balance, "Bob is settled");

    newCurrency(group);
    // 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
    auto const summary = group.summary();
    check(amountOf("1000.00", "RON") == summary.totalSpent, "the trip cost 1000.00 RON");
    check(Balance(SettledBalance{}) == totalsOf(summary, bob).value().balance, "Bob is still settled");
}

} // namespace
} // namespace eranos::examples

int main() {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
