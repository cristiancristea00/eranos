// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <format>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos::examples {
namespace {

/// A row of the receipt: a line, who shared it, its quantity, its unit price, and its total.
struct LineRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<5> HEADERS{"Item", "Shared by", "Quantity", "Unit price", "Line total"};

    /// What was ordered.
    std::string name;
    /// Who shared it.
    std::string participants;
    /// How many were ordered.
    std::string quantity;
    /// The price of one.
    std::string unitPrice;
    /// The quantity times the unit price.
    std::string lineTotal;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<5> cells() const noexcept { return {name, participants, quantity, unitPrice, lineTotal}; }
};

/// A row of the table of tips: what a friend ordered, and what they pay with the tip shared by consumption and equally.
struct TipRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<6> HEADERS{"Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays"};

    /// Their first name.
    std::string name;
    /// What they ordered.
    std::string subtotal;
    /// Their part of the tip shared by consumption.
    std::string proportionalTip;
    /// What they pay with the tip shared by consumption.
    std::string proportionalTotal;
    /// Their part of the tip shared equally.
    std::string equalTip;
    /// What they pay with the tip shared equally.
    std::string equalTotal;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<6> cells() const noexcept { return {name, subtotal, proportionalTip, proportionalTotal, equalTip, equalTotal}; }
};

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) { return must(Money::parse(text, must(Currency::of("EUR")))); }

/// The line @p name of @p quantity at @p unitPrice euros, which @p split shares.
[[nodiscard]] Item line(std::string name, std::string_view const unitPrice, Split split, std::int32_t const quantity) {
    Item item(std::move(name), euros(unitPrice), std::move(split));
    item.quantity = quantity;
    return item;
}

/// The receipt: every line with the members who had it.
[[nodiscard]] std::vector<Item> receipt(MemberId const &ana, MemberId const &bob, MemberId const &carol) {
    return {
        line("Ciorbă", "8.00", EqualSplit{{ana}}, 1),
        line("Sarmale", "14.00", EqualSplit{{bob, carol}}, 2),
        line("Papanași", "9.00", PartsSplit{{MemberParts(ana, Parts{100}), MemberParts(carol, Parts{200})}}, 1),
        line("Lemonade", "3.00", EqualSplit{{ana, bob, carol}}, 3),
    };
}

/// How @p expense would divide if it were added to @p group, computed on a copy loaded from the group's saved form, so the
/// group itself is unchanged.
[[nodiscard]] ExpenseShares preview(ExpenseGroup const &group, Expense const &expense) {
    auto copy = must(ExpenseGroup::load(group.save()));
    auto const id = must(copy.addExpense(expense));
    return must(copy.expenseShares(id)).value();
}

/// What @p member consumed of the expense that @p shares divides.
[[nodiscard]] Money consumed(ExpenseShares const &shares, MemberId const &member) {
    auto const found = std::ranges::find_if(shares.members, [&member](MemberShare const &share) { return member == share.member; });
    check(shares.members.end() != found, "every member ordered");
    return found->share;
}

/// Runs the example.
void run() {
    auto group = must(ExpenseGroup::create("Dinner at Caru' cu Bere", must(Currency::of("EUR"))));
    auto const ana = must(group.addMember(Member("Ana")));
    auto const bob = must(group.addMember(Member("Bob")));
    auto const carol = must(group.addMember(Member("Carol")));
    auto const day = must(CalendarDate::of(2026, 9, 23));

    title(std::format("{}: an itemised bill", group.name()));
    printLine("Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,\n"
              "every line of the receipt is split among the people who had it.");

    section("1. The receipt");
    std::vector<LineRow> lines;
    for (auto const &item : receipt(ana, bob, carol)) {
        Money const total(item.unitPrice.minorUnits * item.quantity, item.unitPrice.currency);
        lines.push_back({
            .name = item.name,
            .participants = names(group, must(participants(item.split))),
            .quantity = std::to_string(item.quantity),
            .unitPrice = item.unitPrice.toString(),
            .lineTotal = total.toString(),
        });
    }
    printTable(lines, 2);
    printLine("The papanași are split by parts: Carol ate two of the three. The line totals add up\n"
              "to 54.00 EUR, and the expense's amount must match them exactly.");

    section("2. Who ordered what, and the tip");
    // The dinner, with its tip of 10 % shared by `distribution` when there is one.
    auto const bill = [&](std::optional<TipDistribution> const distribution) {
        Expense expense("Dinner", LocalCost{euros("54.00")}, EqualSplit{{ana}}, ItemisedExpenseSplit{receipt(ana, bob, carol)}, day);
        if (distribution.has_value()) {
            expense.tip = Tip(PercentageTipValue{must(Percentage::parse("10"))}, *distribution);
        }
        return expense;
    };
    auto const ordered = preview(group, bill(std::nullopt));
    auto const proportional = must(group.addExpense(bill(TipDistribution::PROPORTIONAL)));
    auto const byConsumption = must(group.expenseShares(proportional)).value();
    auto const even = preview(group, bill(TipDistribution::EQUAL));
    std::println("Ana paid the bill plus a 10 % tip ({}). A tip can be shared in proportion to what\n"
                 "each person ordered, or equally; both are shown:",
                 byConsumption.tip.toString());
    std::vector<TipRow> tips;
    for (auto const &member : {ana, bob, carol}) {
        auto const subtotal = consumed(ordered, member);
        auto const proportionalTotal = consumed(byConsumption, member);
        auto const equalTotal = consumed(even, member);
        tips.push_back({
            .name = firstName(group, member),
            .subtotal = subtotal.toString(),
            .proportionalTip = must(proportionalTotal.minus(subtotal)).toString(),
            .proportionalTotal = proportionalTotal.toString(),
            .equalTip = must(equalTotal.minus(subtotal)).toString(),
            .equalTotal = equalTotal.toString(),
        });
    }
    printTable(tips, 1);
    printLine("By consumption, Carol, who ordered the most, adds the largest part of the tip;\n"
              "shared equally, everyone adds the same 1.80.");

    section("3. A total that does not match is refused");
    auto const mismatch = refusal(
        group.addExpense(Expense("Dinner", LocalCost{euros("55.00")}, EqualSplit{{ana}}, ItemisedExpenseSplit{receipt(ana, bob, carol)}, day)));
    check(mismatch.has_value(), "the mismatch is refused");
    std::println("An amount of 55.00 EUR with these lines is refused when the expense is built:\n"
                 "  error: {}",
                 message(mismatch));

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

int main() {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
