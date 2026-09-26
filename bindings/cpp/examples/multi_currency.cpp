// A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON and HUF carry a rate
// fixed when they are entered, and the group later moves its books to RON. The port of the Rust example
// `multi_currency`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// The amount @p text of @p code.
[[nodiscard]] Money amountOf(std::string_view const text, std::string_view const code) { return Money::parse(text, Currency::of(code)); }

/// The amount of @p cost, as entered.
[[nodiscard]] Money amountOf(Cost const &cost) {
    return std::visit([](auto const &alternative) { return alternative.amount; }, cost);
}

/// The rate @p cost was entered at: 1 for a cost in the group currency.
[[nodiscard]] ExchangeRate rateOf(Cost const &cost) {
    auto const *const converted = std::get_if<ConvertedCost>(&cost);
    return nullptr == converted ? ExchangeRate::parse("1") : converted->rate;
}

/// Prints every expense and transfer, as entered and in the group currency.
void printEntries(Group const &group) {
    Money const zero(group.currency(), 0);
    Rows rows;
    for (auto const &expense : group.expenses()) {
        auto const shares = group.expenseShares(expense.id.value());
        rows.push_back({
            expense.name,
            names(group, participants(expense.payers)),
            total(expense).toString(),
            rateOf(expense.cost).toString(),
            shares.has_value() ? shares->total.toString() : zero.toString(),
        });
    }
    for (auto const &transfer : group.transfers()) {
        auto const converted = group.transferAmount(transfer.id.value());
        rows.push_back({
            "Transfer to " + firstName(group, transfer.to),
            firstName(group, transfer.from),
            amountOf(transfer.cost).toString(),
            rateOf(transfer.cost).toString(),
            converted.has_value() ? converted->toString() : zero.toString(),
        });
    }
    printTable({"Entry", "Paid by", "Entered as", "Rate", "In the group currency"}, rows, 2);
}

/// Prints what each member paid, consumed, sent, and received, and their balance.
void printBalances(Group const &group) {
    auto const summary = group.summary();
    Rows rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id.value()).value();
        rows.push_back({
            member.firstName,
            found.paid.toString(),
            found.share.toString(),
            found.sent.toString(),
            found.received.toString(),
            toString(found.balance),
        });
    }
    printTable({"Member", "Paid", "Consumed", "Sent", "Received", "Balance"}, rows, 1);
}

/// Adds the fuel in RON, the dinner in HUF, and the ferry in EUR, which Ana and Bob share.
void expenses(Group &group, MemberId const &ana, MemberId const &bob) {
    section("1. Expenses in three currencies");
    WholeExpenseSplit const both{EqualSplit{{ana, bob}}};
    static_cast<void>(group.addExpense(Expense(ConvertedCost{amountOf("400.00", "RON"), ExchangeRate::parse("0.2")}, CalendarDate::of(1, 8, 2026),
                                               "Fuel", EqualSplit{{ana}}, both)));
    static_cast<void>(group.addExpense(Expense(ConvertedCost{amountOf("24000.00", "HUF"), ExchangeRate::parse("0.0025")},
                                               CalendarDate::of(2, 8, 2026), "Dinner", EqualSplit{{bob}}, both)));
    static_cast<void>(group.addExpense(Expense(LocalCost{amountOf("60.00", "EUR")}, CalendarDate::of(3, 8, 2026), "Ferry", EqualSplit{{bob}}, both)));
    printEntries(group);
    printLine("The rate says how many " + group.currency().code() + " one unit of the entry's currency is worth: 400.00 RON at\n" +
              "0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.");
}

/// Adds Ana's repayment to Bob, in cash and in RON.
void repayment(Group &group, MemberId const &ana, MemberId const &bob) {
    section("3. A repayment in cash, in another currency");
    Transfer transfer(ConvertedCost{amountOf("100.00", "RON"), ExchangeRate::parse("0.2")}, CalendarDate::of(4, 8, 2026), ana, bob);
    transfer.note = "Cash at the border";
    static_cast<void>(group.addTransfer(transfer));
    printLine("Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.\n"
              "Transfers move balances but are not spending, so the total stays at " +
              group.summary().totalSpent.toString() + ":");
    printBalances(group);
}

/// Moves the books of @p group to RON.
void newCurrency(Group &group) {
    section("4. Moving the books to another currency");
    printLine("Changing the group currency needs a rate into the new currency for every other\n"
              "currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies\n"
              "take the new rate; entries already in RON become local costs, at 1.");
    group.changeCurrency(Currency::of("RON"), {
                                                  CurrencyRate(Currency::of("EUR"), ExchangeRate::parse("5")),
                                                  CurrencyRate(Currency::of("HUF"), ExchangeRate::parse("0.0125")),
                                              });
    printEntries(group);
    printLine("In " + group.currency().code() + ", the trip cost " + group.summary().totalSpent.toString() + ", and everyone is still settled:");
    printBalances(group);
}

/// Runs the example.
void run() {
    auto group = Group::create(Currency::of("EUR"), "Danube road trip");
    auto const ana = group.addMember(Member("Ana"));
    auto const bob = group.addMember(Member("Bob"));

    printLine(group.name() + ": expenses in several currencies\n" + "════════════════════════════════════════════════");
    printLine("Ana and Bob drive from Romania to Hungary. The group keeps its books in " + group.currency().code() + "; an\n" +
              "expense in another currency carries the exchange rate used when it was entered,\n" +
              "and is converted once, rounding half away from zero to the cent.");
    expenses(group, ana, bob);

    section("2. Balances");
    printLine("The group spent " + group.summary().totalSpent.toString() + ". Bob paid 120.00 of it but consumed half, so he gets back the\n" +
              "difference:");
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

int main() { return eranos::examples::runPort(eranos::examples::run); }
