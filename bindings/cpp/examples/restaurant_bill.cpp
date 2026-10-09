// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally; then a line weighed on a scale, and
// discounts off the line and off each unit. The port of the Rust example `restaurant_bill`, which prints the same report.

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
    static inline constexpr Cells<5> HEADERS{"Item", "Shared by", "Quantity", "Unit price", "Line total"};

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
    [[nodiscard]] inline constexpr Cells<5> cells() const noexcept { return {name, participants, quantity, unitPrice, lineTotal}; }
};

/// A row of the table of discounts: a line, its quantity, its unit price, and its total before and after its discount.
struct DiscountRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<6> HEADERS{"Item", "Quantity", "Unit price", "Before discount", "Discount", "Line total"};

    /// What was ordered.
    std::string name;
    /// How many were ordered, or how much was weighed.
    std::string quantity;
    /// The price of one.
    std::string unitPrice;
    /// The quantity times the unit price.
    std::string lineTotalBeforeDiscount;
    /// What the discount takes off.
    std::string discountAmount;
    /// What the line comes to.
    std::string lineTotal;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] inline constexpr Cells<6> cells() const noexcept {
        return {name, quantity, unitPrice, lineTotalBeforeDiscount, discountAmount, lineTotal};
    }
};

/// A row of the table of tips: what a friend ordered, and what they pay with the tip shared by consumption and equally.
struct TipRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<6> HEADERS{"Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays"};

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
    [[nodiscard]] inline constexpr Cells<6> cells() const noexcept {
        return {name, subtotal, proportionalTip, proportionalTotal, equalTip, equalTotal};
    }
};

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) noexcept { return must(Money::parse(text, must(CurrencyCode::of("EUR")))); }

/// The line @p name of @p units whole units at @p unitPrice euros, which @p split shares.
[[nodiscard]] ExpenseItem line(std::string name, std::string_view const unitPrice, Split split, std::int64_t const units) noexcept {
    ExpenseItem item(std::move(name), euros(unitPrice), std::move(split));
    item.quantity = must(Quantity::whole(units));
    return item;
}

/// The receipt: every line with the members who had it.
[[nodiscard]] std::vector<ExpenseItem> receipt(MemberId const &ana, MemberId const &bob, MemberId const &carol) noexcept {
    return {
        line("Ciorbă", "8.00", EqualSplit{{ana}}, 1),
        line("Sarmale", "14.00", EqualSplit{{bob, carol}}, 2),
        line("Papanași", "9.00", PartsSplit{{MemberParts(ana, Parts{100}), MemberParts(carol, Parts{200})}}, 1),
        line("Lemonade", "3.00", EqualSplit{{ana, bob, carol}}, 3),
    };
}

/// The second round: telemea weighed on a scale with 10 % off the line, three lemonades with 1.00 off each, and papanași on the house.
struct SecondRound final {
    /// The telemea, weighed on a scale.
    ExpenseItem telemea;
    /// The lemonades.
    ExpenseItem lemonade;
    /// The papanași, on the house.
    ExpenseItem papanasi;
};

/// The second round, shared by @p ana, @p bob, and @p carol.
[[nodiscard]] SecondRound secondRound(MemberId const &ana, MemberId const &bob, MemberId const &carol) noexcept {
    ExpenseItem telemea("Telemea", euros("42.00"), EqualSplit{{ana, bob}});
    // The scale reports the weight as a double.
    telemea.quantity = must(Quantity::fromDouble(0.35));
    telemea.discount = Discount(PercentageDiscountValue{must(Percentage::parse("10"))});
    ExpenseItem lemonade("Lemonade", euros("3.00"), EqualSplit{{ana, bob, carol}});
    lemonade.quantity = must(Quantity::whole(3));
    Discount oneOffEach(FixedDiscountValue{euros("1.00")});
    oneOffEach.scope = DiscountScope::UNIT;
    lemonade.discount = oneOffEach;
    ExpenseItem papanasi("Papanași", euros("9.00"), EqualSplit{{carol}});
    papanasi.discount = Discount(PercentageDiscountValue{must(Percentage::parse("100"))});
    return {.telemea = std::move(telemea), .lemonade = std::move(lemonade), .papanasi = std::move(papanasi)};
}

/// The row of the table of discounts for @p item.
[[nodiscard]] DiscountRow discountRow(ExpenseItem const &item) noexcept {
    return {
        .name = item.name,
        .quantity = item.quantity.toString(),
        .unitPrice = item.unitPrice.toString(),
        .lineTotalBeforeDiscount = must(lineTotalBeforeDiscount(item)).toString(),
        .discountAmount = must(discountAmount(item)).toString(),
        .lineTotal = must(lineTotal(item)).toString(),
    };
}

/// Section 4: prints the second round's lines, each with its total before the discount, the discount, and its line total, and a discount refused
/// for being larger than its line.
void printWeighedLinesAndDiscounts(MemberId const &ana, MemberId const &bob, MemberId const &carol) noexcept {
    section("4. Weighed lines and discounts");
    auto const round = secondRound(ana, bob, carol);
    printTable(std::vector<DiscountRow>{discountRow(round.telemea), discountRow(round.lemonade), discountRow(round.papanasi)}, 1);
    ExpenseItem refused("Papanași", euros("9.00"), EqualSplit{{carol}});
    refused.discount = Discount(FixedDiscountValue{euros("10.00")});
    auto const error = refusal(lineTotal(refused));
    check(error.has_value(), "the discount must be refused");
    std::println("The scale reports {0} kg as a double just below it; the quantity reads its shortest\n"
                 "form, {0}, so the telemea costs exactly {1} before its discount. Taking {2}\n"
                 "off a {3} line is refused when the item is built:\n"
                 "  error: {4}",
                 round.telemea.quantity.toString(), must(lineTotalBeforeDiscount(round.telemea)).toString(), euros("10.00").toString(),
                 euros("9.00").toString(), message(error));

    // 10 % of 14.70 is 1.47; 1.00 off each of three lemonades is 3.00; the papanași on the house leave a free line.
    check(euros("14.70") == lineTotalBeforeDiscount(round.telemea), "the telemea costs 14.70 EUR before its discount");
    check(euros("1.47") == discountAmount(round.telemea), "10 % of it is 1.47 EUR");
    check(euros("13.23") == lineTotal(round.telemea), "which leaves 13.23 EUR");
    check(euros("3.00") == discountAmount(round.lemonade), "1.00 EUR off each of three lemonades is 3.00 EUR");
    check(euros("6.00") == lineTotal(round.lemonade), "which leaves 6.00 EUR");
    check(euros("9.00") == discountAmount(round.papanasi), "the papanași on the house take off all 9.00 EUR");
    check(euros("0.00") == lineTotal(round.papanasi), "which leaves a free line");
}

/// How @p expense would divide if it were added to @p group, computed on a copy loaded from the group's saved form, so the
/// group itself is unchanged.
[[nodiscard]] ExpenseShares preview(ExpenseGroup const &group, Expense const &expense) noexcept {
    auto copy = must(ExpenseGroup::load(group.save()));
    auto const id = must(copy.addExpense(expense));
    return must(copy.expenseShares(id)).value();
}

/// What @p member consumed of the expense that @p shares divides.
[[nodiscard]] Money consumed(ExpenseShares const &shares, MemberId const &member) noexcept {
    auto const found = std::ranges::find_if(shares.members, [&member](MemberShare const &share) noexcept { return member == share.member; });
    check(shares.members.end() != found, "every member ordered");
    return found->share;
}

/// Runs the example.
void run() noexcept {
    auto group = must(ExpenseGroup::create("Dinner at Caru' cu Bere", must(CurrencyCode::of("EUR"))));
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
        lines.push_back({
            .name = item.name,
            .participants = names(group, must(members(item.split))),
            .quantity = item.quantity.toString(),
            .unitPrice = item.unitPrice.toString(),
            .lineTotal = must(lineTotal(item)).toString(),
        });
    }
    printTable(lines, 2);
    printLine("The papanași are split by parts: Carol ate two of the three. The line totals add up\n"
              "to 54.00 EUR, and the expense's amount must match them exactly.");

    section("2. Who ordered what, and the tip");
    // The dinner, with its tip of 10 % shared by `distribution` when there is one.
    auto const bill = [&](std::optional<TipDistribution> const distribution) noexcept {
        Expense expense("Dinner", LocalCost{euros("54.00")}, EqualSplit{{ana}}, ItemisedExpenseSplit{receipt(ana, bob, carol)}, day);
        if (distribution.has_value()) {
            expense.tip = TipRule(PercentageTipValue{must(Percentage::parse("10"))}, *distribution);
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

    printWeighedLinesAndDiscounts(ana, bob, carol);

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

int main() noexcept {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
