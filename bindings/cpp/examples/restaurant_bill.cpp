// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <algorithm>
#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos::examples {
namespace {

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) { return Money::parse(text, Currency::of("EUR")); }

/// The line @p name of @p quantity at @p unitPrice euros, which @p split shares.
[[nodiscard]] Item line(std::string name, Split split, std::string_view const unitPrice, std::int32_t const quantity) {
    Item item(std::move(name), std::move(split), euros(unitPrice));
    item.quantity = quantity;
    return item;
}

/// The receipt: every line with the members who had it.
[[nodiscard]] std::vector<Item> receipt(MemberId const &ana, MemberId const &bob, MemberId const &carol) {
    return {
        line("Ciorbă", EqualSplit{{ana}}, "8.00", 1),
        line("Sarmale", EqualSplit{{bob, carol}}, "14.00", 2),
        line("Papanași", PartsSplit{{MemberParts(ana, Parts{100}), MemberParts(carol, Parts{200})}}, "9.00", 1),
        line("Lemonade", EqualSplit{{ana, bob, carol}}, "3.00", 3),
    };
}

/// How @p expense would divide if it were added to @p group, computed on a copy loaded from the group's saved form, so the
/// group itself is unchanged.
[[nodiscard]] ExpenseShares preview(Group const &group, Expense const &expense) {
    auto copy = Group::load(group.save());
    auto const id = copy.addExpense(expense);
    return copy.expenseShares(id).value();
}

/// What @p member consumed of the expense that @p shares divides.
[[nodiscard]] Money consumed(ExpenseShares const &shares, MemberId const &member) {
    auto const found =
        std::find_if(shares.members.begin(), shares.members.end(), [&member](MemberShare const &share) { return member == share.member; });
    check(shares.members.end() != found, "every member ordered");
    return found->share;
}

/// Runs the example.
void run() {
    auto group = Group::create(Currency::of("EUR"), "Dinner at Caru' cu Bere");
    auto const ana = group.addMember(Member("Ana"));
    auto const bob = group.addMember(Member("Bob"));
    auto const carol = group.addMember(Member("Carol"));
    auto const day = CalendarDate::of(23, 9, 2026);

    title(group.name() + ": an itemised bill");
    printLine("Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,\n"
              "every line of the receipt is split among the people who had it.");

    section("1. The receipt");
    Rows lines;
    for (auto const &item : receipt(ana, bob, carol)) {
        Money const total(item.unitPrice.currency, item.unitPrice.minorUnits * item.quantity);
        lines.push_back(
            {item.name, names(group, participants(item.split)), std::to_string(item.quantity), item.unitPrice.toString(), total.toString()});
    }
    printTable({"Item", "Shared by", "Quantity", "Unit price", "Line total"}, lines, 2);
    printLine("The papanași are split by parts: Carol ate two of the three. The line totals add up\n"
              "to 54.00 EUR, and the expense's amount must match them exactly.");

    section("2. Who ordered what, and the tip");
    // The dinner, with its tip of 10 % shared by `distribution` when there is one.
    auto const bill = [&](std::optional<TipDistribution> const distribution) {
        Expense expense(LocalCost{euros("54.00")}, day, "Dinner", EqualSplit{{ana}}, ItemisedExpenseSplit{receipt(ana, bob, carol)});
        if (distribution.has_value()) {
            expense.tip = Tip(*distribution, PercentageTipValue{Percentage::parse("10")});
        }
        return expense;
    };
    auto const ordered = preview(group, bill(std::nullopt));
    auto const proportional = group.addExpense(bill(TipDistribution::PROPORTIONAL));
    auto const byConsumption = group.expenseShares(proportional).value();
    auto const even = preview(group, bill(TipDistribution::EQUAL));
    printLine("Ana paid the bill plus a 10 % tip (" + byConsumption.tip.toString() + "). A tip can be shared in proportion to what\n" +
              "each person ordered, or equally; both are shown:");
    Rows tips;
    for (auto const &member : {ana, bob, carol}) {
        auto const subtotal = consumed(ordered, member);
        auto const proportionalTotal = consumed(byConsumption, member);
        auto const equalTotal = consumed(even, member);
        tips.push_back({
            firstName(group, member),
            subtotal.toString(),
            proportionalTotal.minus(subtotal).toString(),
            proportionalTotal.toString(),
            equalTotal.minus(subtotal).toString(),
            equalTotal.toString(),
        });
    }
    printTable({"Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays"}, tips, 1);
    printLine("By consumption, Carol, who ordered the most, adds the largest part of the tip;\n"
              "shared equally, everyone adds the same 1.80.");

    section("3. A total that does not match is refused");
    auto const mismatch = refusal([&] {
        return group.addExpense(Expense(LocalCost{euros("55.00")}, day, "Dinner", EqualSplit{{ana}}, ItemisedExpenseSplit{receipt(ana, bob, carol)}));
    });
    check(mismatch.has_value(), "the mismatch is refused");
    printLine("An amount of 55.00 EUR with these lines is refused when the expense is built:\n  error: " + message(mismatch));

    // Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to each;
    // an equal one 1.80 to everyone.
    check(euros("15.40") == consumed(byConsumption, ana), "Ana pays 15.40 EUR with the tip by consumption");
    check(euros("18.70") == consumed(byConsumption, bob), "Bob pays 18.70 EUR with the tip by consumption");
    check(euros("25.30") == consumed(byConsumption, carol), "Carol pays 25.30 EUR with the tip by consumption");
    check(euros("24.80") == consumed(even, carol), "Carol pays 24.80 EUR with the tip shared equally");
    check(euros("59.40") == even.total, "the dinner comes to 59.40 EUR with its tip");
}

} // namespace
} // namespace eranos::examples

int main() { return eranos::examples::runPort(eranos::examples::run); }
