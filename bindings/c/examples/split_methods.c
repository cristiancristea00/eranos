// Every way to divide an expense: equally, by percentage, by parts (including half a part for a child), by exact
// amounts, and with several payers. The port of the Rust example `split_methods`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdlib.h>
#include <string.h>

/// The family and the day every expense happened.
typedef struct Family {
    /// The first parent.
    Id ana;
    /// The second parent.
    Id bob;
    /// Their daughter.
    Id mia;
    /// The day of every expense.
    EranosCalendarDate day;
} Family;

/// The rule by which a member shares an expense, as the report names it.
typedef struct Rule {
    /// The member.
    Id member;
    /// The rule.
    char const *text;
} Rule;

/// A row of the table of an expense's shares: a member's rule, what they paid, and what they consumed.
typedef struct ShareRow {
    /// Their first name.
    Line name;
    /// The rule by which they share the expense.
    Line rule;
    /// What they paid.
    Line paid;
    /// What they consumed.
    Line consumed;
} ShareRow;

/// A table of an expense's shares, its columns from @p numbersFrom on aligned to the right.
static Table *shareTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Member", "Rule", "Paid", "Consumed"}, numbersFrom);
}

/// Adds @p row to @p table, a table of an expense's shares, its cells in the order of the table's headers.
static void addShareRow(Table *const table, ShareRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->rule, row->paid, row->consumed});
}

/// A row of the table of totals: what a member paid and consumed over the holiday, and their balance.
typedef struct TotalRow {
    /// Their first name.
    Line name;
    /// What they paid.
    Line paid;
    /// What they consumed.
    Line consumed;
    /// What is left: what they paid minus what they consumed.
    Line balance;
} TotalRow;

/// A table of totals, its columns from @p numbersFrom on aligned to the right.
static Table *totalTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Member", "Paid", "Consumed", "Balance"}, numbersFrom);
}

/// Adds @p row to @p table, a table of totals, its cells in the order of the table's headers.
static void addTotalRow(Table *const table, TotalRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->paid, row->consumed, row->balance});
}

/// The amount @p text of euros.
static EranosMoney *euros(char const *const text) { return amountOf(text, "EUR"); }

/// The three members of @p family.
static Members everyone(Family const *const family) { return (Members){.count = 3, .ids = {family->ana, family->bob, family->mia}}; }

/// The rule of @p member among the @p count rules of @p rules, or (payer only) for a member they do not name.
static char const *ruleOf(Rule const *const rules, size_t const count, char const *const member) {
    for (size_t index = 0; count > index; ++index) {
        if (0 == strcmp(rules[index].member.text, member)) {
            return rules[index].text;
        }
    }
    return "(payer only)";
}

/// Prints how the expense @p id divides: every involved member's rule among the @p count rules of @p rules, what they
/// paid, and what they consumed.
static void report(EranosExpenseGroup const *const group, Id const id, Rule const *const rules, size_t const count) {
    EranosExpense *expense = nullptr;
    must(eranosExpenseGroupExpense(group, id.text, &expense));
    EranosExpenseShares *shares = nullptr;
    must(eranosExpenseGroupExpenseShares(group, id.text, &shares));
    if (nullptr == expense || nullptr == shares) {
        eranosExpenseSharesDestroy(shares);
        eranosExpenseDestroy(expense);
        return;
    }
    EranosText *const text = eranosTextCreate();
    eranosExpenseName(expense, text);
    Line const total = moneyLine(eranosExpenseSharesTotal(shares));
    printFormatted("%s, %s:", eranosTextData(text), total.text);
    Table *const table = shareTable(2);
    EranosMemberShareList *const members = eranosExpenseSharesMembers(shares);
    for (size_t index = 0; eranosMemberShareListLength(members) > index; ++index) {
        EranosMemberShare *const share = eranosMemberShareListGet(members, index);
        eranosMemberShareMember(share, text);
        addShareRow(table, &(ShareRow){
                               .name = firstName(group, eranosTextData(text)),
                               .rule = lineOf("%s", ruleOf(rules, count, eranosTextData(text))),
                               .paid = moneyLine(eranosMemberSharePaid(share)),
                               .consumed = moneyLine(eranosMemberShareShare(share)),
                           });
        eranosMemberShareDestroy(share);
    }
    tablePrint(table);
    eranosMemberShareListDestroy(members);
    eranosTextDestroy(text);
    eranosExpenseSharesDestroy(shares);
    eranosExpenseDestroy(expense);
}

/// Adds the taxi, which the three share equally, the leftover cent going to one of them.
static void equally(EranosExpenseGroup *const group, Family const *const family) {
    section("1. Equally");
    printLine("Everyone pays the same. 10.00 does not divide by three, so one member pays the\n"
              "leftover cent; which member it is rotates from expense to expense, so over a trip\n"
              "nobody always pays it.");
    Members const members = everyone(family);
    Id const taxi = addExpense(group, expenseOf("Taxi", localCost(euros("10.00")), soleSplit(family->ana), equalSplit(&members), family->day));
    Rule const thirds[] = {
        {.member = family->ana, .text = "an equal third"},
        {.member = family->bob, .text = "an equal third"},
        {.member = family->mia, .text = "an equal third"},
    };
    report(group, taxi, thirds, 3);
}

/// Adds the flat, which the three share by percentage.
static void byPercentage(EranosExpenseGroup *const group, Family const *const family) {
    section("2. By percentage");
    printLine("Everyone pays a percentage of the amount; the percentages must add up to exactly\n"
              "100 %. Mia's room is smaller, so she takes 20 %.");
    Share const shares[] = {
        {.member = family->ana, .value = "40"},
        {.member = family->bob, .value = "40"},
        {.member = family->mia, .value = "20"},
    };
    Id const rent =
        addExpense(group, expenseOf("Flat", localCost(euros("500.00")), soleSplit(family->bob), percentagesSplit(shares, 3), family->day));
    Rule const rules[] = {
        {.member = family->ana, .text = "40%"},
        {.member = family->bob, .text = "40%"},
        {.member = family->mia, .text = "20%"},
    };
    report(group, rent, rules, 3);
}

/// Adds the food, which the three share by parts, Mia counting as half an adult.
static Id byParts(EranosExpenseGroup *const group, Family const *const family) {
    section("3. By parts");
    printLine("Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:\n"
              "2.5 parts in total, so one part is 125.00 / 2.5 = 50.00.");
    Share const shares[] = {
        {.member = family->ana, .value = "1"},
        {.member = family->bob, .value = "1"},
        {.member = family->mia, .value = "0.5"},
    };
    Id const food = addExpense(group, expenseOf("Food", localCost(euros("125.00")), soleSplit(family->ana), partsSplit(shares, 3), family->day));
    Rule const rules[] = {
        {.member = family->ana, .text = "1 part"},
        {.member = family->bob, .text = "1 part"},
        {.member = family->mia, .text = "0.5 parts"},
    };
    report(group, food, rules, 3);
    return food;
}

/// Adds the museum tickets, which the three share by exact amounts.
static void byAmounts(EranosExpenseGroup *const group, Family const *const family) {
    section("4. By exact amounts");
    printLine("Everyone pays a fixed amount, such as a ticket price; the amounts must add up to\n"
              "exactly the expense amount.");
    Share const shares[] = {
        {.member = family->ana, .value = "15.00"},
        {.member = family->bob, .value = "15.00"},
        {.member = family->mia, .value = "5.00"},
    };
    Id const tickets =
        addExpense(group, expenseOf("Museum", localCost(euros("35.00")), soleSplit(family->bob), amountsSplit("EUR", shares, 3), family->day));
    Rule const rules[] = {
        {.member = family->ana, .text = "15.00 EUR"},
        {.member = family->bob, .text = "15.00 EUR"},
        {.member = family->mia, .text = "5.00 EUR"},
    };
    report(group, tickets, rules, 3);
}

/// Adds the dinner, which two of them paid and the three share equally.
static void severalPayers(EranosExpenseGroup *const group, Family const *const family) {
    section("5. Several payers");
    printLine("The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are\n"
              "divided with the same methods as the participants.");
    Share const cards[] = {
        {.member = family->ana, .value = "50.00"},
        {.member = family->bob, .value = "40.00"},
    };
    Members const members = everyone(family);
    Id const dinner =
        addExpense(group, expenseOf("Dinner", localCost(euros("90.00")), amountsSplit("EUR", cards, 2), equalSplit(&members), family->day));
    Rule const thirds[] = {
        {.member = family->ana, .text = "an equal third"},
        {.member = family->bob, .text = "an equal third"},
        {.member = family->mia, .text = "an equal third"},
    };
    report(group, dinner, thirds, 3);
}

/// Shows that a split whose percentages do not add up to 100 % is refused.
static void mistakes(EranosExpenseGroup *const group, Family const *const family) {
    section("6. Mistakes are refused");
    printLine("A percentage split that adds up to 90 % is refused when the expense is built:");
    Share const shares[] = {
        {.member = family->ana, .value = "60"},
        {.member = family->bob, .value = "30"},
    };
    EranosError *const refused =
        expenseRefusal(group, expenseOf("Invalid", localCost(euros("10.00")), soleSplit(family->ana), percentagesSplit(shares, 2), family->day));
    check(nullptr != refused, "the split is refused");
    Line const message = messageLine(refused);
    printFormatted("  error: %s", message.text);
}

/// Prints what each member paid and consumed over the holiday, and their balance.
static void totals(EranosExpenseGroup const *const group) {
    section("7. The holiday in total");
    EranosSummary *summary = nullptr;
    must(eranosExpenseGroupSummary(group, &summary));
    Line const spent = moneyLine(eranosSummaryTotalSpent(summary));
    printFormatted("%s spent over five expenses:", spent.text);
    EranosMemberList *members = nullptr;
    must(eranosExpenseGroupMembers(group, &members));
    Table *const table = totalTable(1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        check(nullptr != found, "every member has totals");
        eranosMemberFirstName(member, text);
        addTotalRow(table, &(TotalRow){
                               .name = textLine(text),
                               .paid = moneyLine(eranosMemberSummaryPaid(found)),
                               .consumed = moneyLine(eranosMemberSummaryShare(found)),
                               .balance = balanceLine(eranosMemberSummaryBalance(found)),
                           });
        eranosMemberSummaryDestroy(found);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
}

/// Whether Mia consumed 25.00 EUR of the food: 125.00 over 2.5 parts is 50.00 per part.
static bool miaConsumedHalfAPart(EranosExpenseGroup const *const group, Family const *const family, Id const food) {
    EranosExpenseShares *shares = nullptr;
    must(eranosExpenseGroupExpenseShares(group, food.text, &shares));
    check(nullptr != shares, "food was added");
    EranosMemberShareList *const members = eranosExpenseSharesMembers(shares);
    EranosText *const text = eranosTextCreate();
    bool consumed = false;
    for (size_t index = 0; eranosMemberShareListLength(members) > index; ++index) {
        EranosMemberShare *const share = eranosMemberShareListGet(members, index);
        eranosMemberShareMember(share, text);
        if (0 == strcmp(eranosTextData(text), family->mia.text)) {
            EranosMoney *const amount = eranosMemberShareShare(share);
            consumed = 2'500 == eranosMoneyMinorUnits(amount);
            eranosMoneyDestroy(amount);
        }
        eranosMemberShareDestroy(share);
    }
    eranosTextDestroy(text);
    eranosMemberShareListDestroy(members);
    eranosExpenseSharesDestroy(shares);
    return consumed;
}

int main(void) {
    EranosNewExpenseGroup *details = nullptr;
    must(eranosNewExpenseGroupCreate("Family holiday", "EUR", &details));
    EranosExpenseGroup *group = nullptr;
    must(eranosExpenseGroupCreate(details, &group));
    eranosNewExpenseGroupDestroy(details);
    // The members join in this order, which gives them their identifiers: an initialiser evaluates its values in no set
    // order.
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");
    Id const mia = addMember(group, "Mia");
    Family const family = {.ana = ana, .bob = bob, .mia = mia, .day = {.year = 2026, .month = 7, .day = 14}};

    printLine("Family holiday: every way to split an expense\n"
              "═════════════════════════════════════════════");
    printLine("Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a\n"
              "different split method; whatever the method, the shares add up to exactly the\n"
              "amount, to the cent.");
    equally(group, &family);
    byPercentage(group, &family);
    Id const food = byParts(group, &family);
    byAmounts(group, &family);
    severalPayers(group, &family);
    mistakes(group, &family);
    totals(group);

    check(miaConsumedHalfAPart(group, &family, food), "Mia consumed 25.00 EUR of the food");
    eranosExpenseGroupDestroy(group);
    return EXIT_SUCCESS;
}
