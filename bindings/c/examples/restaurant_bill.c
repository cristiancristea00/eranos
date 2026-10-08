// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/// The three friends and the day they had dinner.
typedef struct Friends {
    /// Who paid.
    Id ana;
    /// The second friend.
    Id bob;
    /// The third friend, who ordered the most.
    Id carol;
    /// The day of the dinner.
    EranosCalendarDate day;
} Friends;

/// A row of the receipt: a line, who shared it, its quantity, its unit price, and its total.
typedef struct LineRow {
    /// What was ordered.
    Line name;
    /// Who shared it.
    Line participants;
    /// How many were ordered.
    Line quantity;
    /// The price of one.
    Line unitPrice;
    /// The quantity times the unit price.
    Line lineTotal;
} LineRow;

/// A table of the receipt's lines, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *lineTable(size_t const numbersFrom) {
    return tableCreate(5, (char const *const[]){"Item", "Shared by", "Quantity", "Unit price", "Line total"}, numbersFrom);
}

/// Adds @p row to @p table, a table of the receipt's lines, its cells in the order of the table's headers.
static void addLineRow(Table *const table, LineRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->participants, row->quantity, row->unitPrice, row->lineTotal});
}

/// A row of the table of tips: what a friend ordered, and what they pay with the tip shared by consumption and equally.
typedef struct TipRow {
    /// Their first name.
    Line name;
    /// What they ordered.
    Line subtotal;
    /// Their part of the tip shared by consumption.
    Line proportionalTip;
    /// What they pay with the tip shared by consumption.
    Line proportionalTotal;
    /// Their part of the tip shared equally.
    Line equalTip;
    /// What they pay with the tip shared equally.
    Line equalTotal;
} TipRow;

/// A table of tips, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *tipTable(size_t const numbersFrom) {
    return tableCreate(6, (char const *const[]){"Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays"}, numbersFrom);
}

/// Adds @p row to @p table, a table of tips, its cells in the order of the table's headers.
static void addTipRow(Table *const table, TipRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->subtotal, row->proportionalTip, row->proportionalTotal, row->equalTip, row->equalTotal});
}

/// The amount @p text of euros.
[[nodiscard]] static EranosMoney *euros(char const *const text) { return amountOf(text, "EUR"); }

/// Adds the line @p name of the receipt to @p items: @p quantity of @p unitPrice euros, which @p split shares; destroys
/// @p split.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the name, then the unit price, in the order the library's constructor reads them.
static void addItem(EranosExpenseItemList *const items, char const *const name, char const *const unitPrice, EranosSplit *const split,
                    int32_t const quantity) {
    EranosMoney *const price = euros(unitPrice);
    EranosExpenseItem *item = nullptr;
    must(eranosExpenseItemCreate(name, price, split, &item));
    eranosExpenseItemSetQuantity(item, quantity);
    eranosExpenseItemListPush(items, item);
    eranosExpenseItemDestroy(item);
    eranosMoneyDestroy(price);
    eranosSplitDestroy(split);
}

/// The receipt: every line with the members who had it.
[[nodiscard]] static EranosExpenseItemList *receipt(Friends const *const friends) {
    EranosExpenseItemList *const items = eranosExpenseItemListCreate();
    addItem(items, "Ciorbă", "8.00", soleSplit(friends->ana), 1);
    Members const sarmale = {.count = 2, .ids = {friends->bob, friends->carol}};
    addItem(items, "Sarmale", "14.00", equalSplit(&sarmale), 2);
    Share const papanasi[] = {
        {.member = friends->ana, .value = "1"},
        {.member = friends->carol, .value = "2"},
    };
    addItem(items, "Papanași", "9.00", partsSplit(papanasi, 2), 1);
    Members const everyone = {.count = 3, .ids = {friends->ana, friends->bob, friends->carol}};
    addItem(items, "Lemonade", "3.00", equalSplit(&everyone), 3);
    return items;
}

/// The dinner of @p amount euros that Ana paid, over the receipt's lines.
[[nodiscard]] static EranosExpense *billOf(Friends const *const friends, char const *const amount) {
    EranosCost *const cost = localCost(euros(amount));
    EranosSplit *const payers = soleSplit(friends->ana);
    EranosExpenseItemList *const items = receipt(friends);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateItemised(items);
    EranosExpense *expense = nullptr;
    must(eranosExpenseCreate("Dinner", cost, payers, split, friends->day, &expense));
    eranosExpenseSplitDestroy(split);
    eranosExpenseItemListDestroy(items);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    return expense;
}

/// The dinner with a tip of 10 %, shared by @p distribution.
[[nodiscard]] static EranosExpense *tippedBillOf(Friends const *const friends, EranosTipDistribution const distribution) {
    EranosExpense *const expense = billOf(friends, "54.00");
    int32_t basisPoints = 0;
    must(eranosPercentageParse("10", &basisPoints));
    EranosTipValue *const value = eranosTipValueCreatePercentage(basisPoints);
    EranosTipRule *const tip = eranosTipRuleCreate(value, distribution);
    eranosExpenseSetTip(expense, tip);
    eranosTipRuleDestroy(tip);
    eranosTipValueDestroy(value);
    return expense;
}

/// How @p expense would divide if it were added to @p group, computed on a copy loaded from the group's saved form, so
/// the group itself is unchanged; destroys @p expense.
[[nodiscard]] static EranosExpenseShares *preview(EranosExpenseGroup const *const group, EranosExpense *const expense) {
    EranosText *const text = eranosTextCreate();
    eranosExpenseGroupSave(group, text);
    EranosExpenseGroup *copy = nullptr;
    must(eranosExpenseGroupLoad(eranosTextData(text), &copy));
    must(eranosExpenseGroupAddExpense(copy, expense, text));
    EranosExpenseShares *shares = nullptr;
    must(eranosExpenseGroupExpenseShares(copy, eranosTextData(text), &shares));
    check(nullptr != shares, "the copy holds the dinner");
    eranosExpenseGroupDestroy(copy);
    eranosTextDestroy(text);
    eranosExpenseDestroy(expense);
    return shares;
}

/// What @p member consumed of the expense that @p shares divides, which the caller destroys.
[[nodiscard]] static EranosMoney *consumed(EranosExpenseShares const *const shares, Id const member) {
    EranosMemberShareList *const members = eranosExpenseSharesMembers(shares);
    EranosText *const text = eranosTextCreate();
    EranosMoney *found = nullptr;
    for (size_t index = 0; nullptr == found && eranosMemberShareListLength(members) > index; ++index) {
        EranosMemberShare *const share = eranosMemberShareListGet(members, index);
        eranosMemberShareMember(share, text);
        if (0 == strcmp(eranosTextData(text), member.text)) {
            found = eranosMemberShareShare(share);
        }
        eranosMemberShareDestroy(share);
    }
    eranosTextDestroy(text);
    eranosMemberShareListDestroy(members);
    check(nullptr != found, "every member ordered");
    return found;
}

/// What @p total adds to @p subtotal.
[[nodiscard]] static Line addedTo(EranosMoney const *const total, EranosMoney const *const subtotal) {
    EranosMoney *difference = nullptr;
    must(eranosMoneySubtract(total, subtotal, &difference));
    return moneyLine(difference);
}

/// Whether @p member consumed @p minorUnits of the expense that @p shares divides.
[[nodiscard]] static bool consumedIs(EranosExpenseShares const *const shares, Id const member, int64_t const minorUnits) {
    EranosMoney *const amount = consumed(shares, member);
    bool const same = minorUnits == eranosMoneyMinorUnits(amount);
    eranosMoneyDestroy(amount);
    return same;
}

/// Prints the receipt's lines: who shared each, its quantity, its unit price, and its total.
static void printReceipt(EranosExpenseGroup const *const group, Friends const *const friends) {
    EranosExpenseItemList *const items = receipt(friends);
    Table *const table = lineTable(2);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosExpenseItemListLength(items) > index; ++index) {
        EranosExpenseItem *const item = eranosExpenseItemListGet(items, index);
        EranosSplit *const split = eranosExpenseItemSplit(item);
        Members const sharedBy = splitMembers(split);
        EranosMoney *const unitPrice = eranosExpenseItemUnitPrice(item);
        int32_t const quantity = eranosExpenseItemQuantity(item);
        EranosCurrencyDetails *const currency = eranosMoneyCurrency(unitPrice);
        EranosMoney *const total = eranosMoneyCreate(eranosMoneyMinorUnits(unitPrice) * quantity, currency);
        eranosCurrencyDetailsDestroy(currency);
        eranosExpenseItemName(item, text);
        addLineRow(table, &(LineRow){
                              .name = textLine(text),
                              .participants = names(group, &sharedBy),
                              .quantity = lineOf("%ld", (long)quantity),
                              .unitPrice = moneyLine(unitPrice),
                              .lineTotal = moneyLine(total),
                          });
        eranosSplitDestroy(split);
        eranosExpenseItemDestroy(item);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosExpenseItemListDestroy(items);
}

/// Prints what each friend ordered, and what they pay with the tip shared by consumption, in @p byConsumption, and
/// equally, in @p even; @p ordered divides the dinner without a tip.
static void printTips(EranosExpenseGroup const *const group, Friends const *const friends, EranosExpenseShares const *const ordered,
                      EranosExpenseShares const *const byConsumption, EranosExpenseShares const *const even) {
    Table *const table = tipTable(1);
    Id const members[] = {friends->ana, friends->bob, friends->carol};
    for (size_t index = 0; 3 > index; ++index) {
        EranosMoney *const subtotal = consumed(ordered, members[index]);
        EranosMoney *const proportionalTotal = consumed(byConsumption, members[index]);
        EranosMoney *const equalTotal = consumed(even, members[index]);
        // The tips first: the fields of a row are made in no set order, and three of them destroy the amounts they write.
        Line const proportionalTip = addedTo(proportionalTotal, subtotal);
        Line const equalTip = addedTo(equalTotal, subtotal);
        addTipRow(table, &(TipRow){
                             .name = firstName(group, members[index].text),
                             .subtotal = moneyLine(subtotal),
                             .proportionalTip = proportionalTip,
                             .proportionalTotal = moneyLine(proportionalTotal),
                             .equalTip = equalTip,
                             .equalTotal = moneyLine(equalTotal),
                         });
    }
    tablePrint(table);
}

int main(void) {
    EranosNewExpenseGroup *details = nullptr;
    must(eranosNewExpenseGroupCreate("Dinner at Caru' cu Bere", "EUR", &details));
    EranosExpenseGroup *group = nullptr;
    must(eranosExpenseGroupCreate(details, &group));
    eranosNewExpenseGroupDestroy(details);
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");
    Id const carol = addMember(group, "Carol");
    Friends const friends = {.ana = ana, .bob = bob, .carol = carol, .day = {.year = 2026, .month = 9, .day = 23}};

    EranosText *const name = eranosTextCreate();
    eranosExpenseGroupName(group, name);
    Line const heading = lineOf("%s: an itemised bill", eranosTextData(name));
    eranosTextDestroy(name);
    title(heading.text);
    printLine("Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,\n"
              "every line of the receipt is split among the people who had it.");

    section("1. The receipt");
    printReceipt(group, &friends);
    printLine("The papanași are split by parts: Carol ate two of the three. The line totals add up\n"
              "to 54.00 EUR, and the expense's amount must match them exactly.");

    section("2. Who ordered what, and the tip");
    EranosExpenseShares *const ordered = preview(group, billOf(&friends, "54.00"));
    Id const proportional = addExpense(group, tippedBillOf(&friends, ERANOS_TIP_DISTRIBUTION_PROPORTIONAL));
    EranosExpenseShares *byConsumption = nullptr;
    must(eranosExpenseGroupExpenseShares(group, proportional.text, &byConsumption));
    check(nullptr != byConsumption, "the group holds the dinner");
    EranosExpenseShares *const even = preview(group, tippedBillOf(&friends, ERANOS_TIP_DISTRIBUTION_EQUAL));
    Line const tip = moneyLine(eranosExpenseSharesTip(byConsumption));
    printFormatted("Ana paid the bill plus a 10 %% tip (%s). A tip can be shared in proportion to what\n"
                   "each person ordered, or equally; both are shown:",
                   tip.text);
    printTips(group, &friends, ordered, byConsumption, even);
    printLine("By consumption, Carol, who ordered the most, adds the largest part of the tip;\n"
              "shared equally, everyone adds the same 1.80.");

    section("3. A total that does not match is refused");
    EranosError *const mismatch = expenseRefusal(group, billOf(&friends, "55.00"));
    check(nullptr != mismatch, "the mismatch is refused");
    Line const message = messageLine(mismatch);
    printFormatted("An amount of 55.00 EUR with these lines is refused when the expense is built:\n"
                   "  error: %s",
                   message.text);

    // Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to
    // each; an equal one 1.80 to everyone.
    check(consumedIs(byConsumption, ana, 1'540), "Ana pays 15.40 EUR with the tip by consumption");
    check(consumedIs(byConsumption, bob, 1'870), "Bob pays 18.70 EUR with the tip by consumption");
    check(consumedIs(byConsumption, carol, 2'530), "Carol pays 25.30 EUR with the tip by consumption");
    check(consumedIs(even, carol, 2'480), "Carol pays 24.80 EUR with the tip shared equally");
    EranosMoney *const total = eranosExpenseSharesTotal(even);
    check(5'940 == eranosMoneyMinorUnits(total), "the dinner comes to 59.40 EUR with its tip");
    eranosMoneyDestroy(total);
    eranosExpenseSharesDestroy(even);
    eranosExpenseSharesDestroy(byConsumption);
    eranosExpenseSharesDestroy(ordered);
    eranosExpenseGroupDestroy(group);
    return EXIT_SUCCESS;
}
