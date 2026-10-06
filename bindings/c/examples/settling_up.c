// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until everyone
// is settled. The port of the Rust example `settling_up`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/// How many friends the festival's group holds: more open balances than the adaptive strategy settles exactly.
static constexpr size_t FRIENDS = 21;

/// How many payments each strategy plans.
typedef struct Counts {
    /// The payments of the adaptive plan.
    size_t adaptive;
    /// The payments of the direct plan.
    size_t direct;
    /// The payments of the greedy plan.
    size_t greedy;
    /// The payments of the minimum plan.
    size_t minimum;
} Counts;

/// A row of the table of expenses: what was bought, who paid it, who shares it, and its amount.
typedef struct ExpenseRow {
    /// What it cost.
    Line amount;
    /// What was bought.
    Line name;
    /// Who shares it.
    Line participants;
    /// Who paid it.
    Line payer;
} ExpenseRow;

/// A table of expenses, its columns from @p numbersFrom on aligned to the right.
static Table *expenseTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Expense", "Paid by", "Shared by", "Amount"}, numbersFrom);
}

/// Adds @p row to @p table, a table of expenses, its cells in the order of the table's headers.
static void addExpenseRow(Table *const table, ExpenseRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->payer, row->participants, row->amount});
}

/// A row of the table of balances: what a member paid and consumed, and their balance.
typedef struct BalanceRow {
    /// What is left: what they paid minus what they consumed.
    Line balance;
    /// What they consumed.
    Line consumed;
    /// Their first name.
    Line name;
    /// What they paid.
    Line paid;
} BalanceRow;

/// A table of balances, its columns from @p numbersFrom on aligned to the right.
static Table *balanceTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Member", "Paid", "Consumed", "Balance"}, numbersFrom);
}

/// Adds @p row to @p table, a table of balances, its cells in the order of the table's headers.
static void addBalanceRow(Table *const table, BalanceRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->paid, row->consumed, row->balance});
}

/// A row of the table of payments: who pays whom, and how much.
typedef struct PaymentRow {
    /// How much is paid.
    Line amount;
    /// Who pays.
    Line from;
    /// Who is paid.
    Line to;
} PaymentRow;

/// A table of payments, its columns from @p numbersFrom on aligned to the right.
static Table *paymentTable(size_t const numbersFrom) { return tableCreate(3, (char const *const[]){"From", "To", "Amount"}, numbersFrom); }

/// Adds @p row to @p table, a table of payments, its cells in the order of the table's headers.
static void addPaymentRow(Table *const table, PaymentRow const *const row) { tableAdd(table, (Line const[]){row->from, row->to, row->amount}); }

/// A row of the table of strategies: what a strategy guarantees, what it costs, and how many payments it plans here.
typedef struct StrategyRow {
    /// How long it takes.
    Line cost;
    /// What it promises.
    Line guarantee;
    /// How many payments it plans for the group.
    Line payments;
    /// Its name.
    Line strategy;
} StrategyRow;

/// A table of strategies, its columns from @p numbersFrom on aligned to the right.
static Table *strategyTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Strategy", "Guarantee", "Cost", "Payments here"}, numbersFrom);
}

/// Adds @p row to @p table, a table of strategies, its cells in the order of the table's headers.
static void addStrategyRow(Table *const table, StrategyRow const *const row) {
    tableAdd(table, (Line const[]){row->strategy, row->guarantee, row->cost, row->payments});
}

/// A payment of a plan as values two plans compare: who pays whom, and how much.
typedef struct Payment {
    /// How much is paid, in minor units of the group's currency.
    int64_t amount;
    /// Who pays.
    Id from;
    /// Who is paid.
    Id to;
} Payment;

/// The name of @p strategy as the apps show it.
static char const *strategyName(EranosSettlementStrategy const strategy) {
    switch (strategy) {
    case ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE:
        return "Adaptive";
    case ERANOS_SETTLEMENT_STRATEGY_DIRECT:
        return "Direct";
    case ERANOS_SETTLEMENT_STRATEGY_GREEDY:
        return "Greedy";
    case ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS:
        return "MinimumTransfers";
    }
    unreachable();
}

/// Records an expense of @p amount euros, named @p name, that @p payer paid and @p members share equally.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the amount, then the name, in the order the library's constructor reads them.
static void spend(EranosGroup *const group, char const *const amount, char const *const name, Id const payer, Members const members) {
    EranosCalendarDate const date = {.day = 20, .month = 9, .year = 2026};
    addExpense(group, expenseOf(localCost(amountOf(amount, "EUR")), date, name, soleSplit(payer), equalSplit(&members)));
}

/// Prints the expenses of @p group: who paid each, who shares it, and its amount.
static void printExpenses(EranosGroup const *const group) {
    EranosExpenseList *expenses = nullptr;
    must(eranosGroupExpenses(group, &expenses));
    Table *const table = expenseTable(3);
    EranosText *const name = eranosTextCreate();
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        eranosExpenseName(expense, name);
        EranosSplit *const payers = eranosExpensePayers(expense);
        Members const paidBy = splitMembers(payers);
        Members const sharedBy = participants(expense);
        EranosMoney *total = nullptr;
        must(eranosExpenseTotal(expense, &total));
        addExpenseRow(table, &(ExpenseRow){
                                 .amount = moneyLine(total),
                                 .name = textLine(name),
                                 .participants = names(group, &sharedBy),
                                 .payer = names(group, &paidBy),
                             });
        eranosSplitDestroy(payers);
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(name);
    eranosExpenseListDestroy(expenses);
    tablePrint(table);
}

/// Prints what each member of @p group paid and consumed, and their balance.
static void printBalances(EranosGroup const *const group) {
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMemberList *members = nullptr;
    must(eranosGroupMembers(group, &members));
    Table *const table = balanceTable(1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        check(nullptr != found, "every member has totals");
        eranosMemberFirstName(member, text);
        addBalanceRow(table, &(BalanceRow){
                                 .balance = balanceLine(eranosMemberSummaryBalance(found)),
                                 .consumed = moneyLine(eranosMemberSummaryShare(found)),
                                 .name = textLine(text),
                                 .paid = moneyLine(eranosMemberSummaryPaid(found)),
                             });
        eranosMemberSummaryDestroy(found);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
}

/// Prints @p payments: who pays whom, and how much.
static void printPayments(EranosGroup const *const group, EranosPaymentList const *const payments) {
    Table *const table = paymentTable(2);
    EranosText *const from = eranosTextCreate();
    EranosText *const to = eranosTextCreate();
    for (size_t index = 0; eranosPaymentListLength(payments) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(payments, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        addPaymentRow(table, &(PaymentRow){
                                 .amount = moneyLine(eranosPaymentAmount(payment)),
                                 .from = firstName(group, eranosTextData(from)),
                                 .to = firstName(group, eranosTextData(to)),
                             });
        eranosPaymentDestroy(payment);
    }
    eranosTextDestroy(to);
    eranosTextDestroy(from);
    tablePrint(table);
}

/// The plan that @p strategy makes for @p group, which the caller destroys.
static EranosSettlement *settlementOf(EranosGroup const *const group, EranosSettlementStrategy const strategy) {
    EranosSettlement *settlement = nullptr;
    must(eranosGroupSettlement(group, strategy, &settlement));
    return settlement;
}

/// The payments of the plan that @p strategy makes for @p group, which the caller destroys.
static EranosPaymentList *planOf(EranosGroup const *const group, EranosSettlementStrategy const strategy) {
    EranosSettlement *const settlement = settlementOf(group, strategy);
    EranosPaymentList *const payments = eranosSettlementPayments(settlement);
    eranosSettlementDestroy(settlement);
    return payments;
}

/// The values of @p payment; destroys @p payment.
static Payment paymentOf(EranosPayment *const payment) {
    EranosText *const text = eranosTextCreate();
    eranosPaymentFrom(payment, text);
    Id const from = idOf(text);
    eranosPaymentTo(payment, text);
    Id const to = idOf(text);
    EranosMoney *const amount = eranosPaymentAmount(payment);
    Payment const values = {.amount = eranosMoneyMinorUnits(amount), .from = from, .to = to};
    eranosMoneyDestroy(amount);
    eranosTextDestroy(text);
    eranosPaymentDestroy(payment);
    return values;
}

/// Whether @p left and @p right hold the same payments, in the same order.
static bool samePayments(EranosPaymentList const *const left, EranosPaymentList const *const right) {
    bool same = eranosPaymentListLength(left) == eranosPaymentListLength(right);
    for (size_t index = 0; same && eranosPaymentListLength(left) > index; ++index) {
        Payment const leftPayment = paymentOf(eranosPaymentListGet(left, index));
        Payment const rightPayment = paymentOf(eranosPaymentListGet(right, index));
        same = leftPayment.amount == rightPayment.amount && 0 == strcmp(leftPayment.from.text, rightPayment.from.text) &&
               0 == strcmp(leftPayment.to.text, rightPayment.to.text);
    }
    return same;
}

/// Prints each strategy's plan with what it promises; returns the number of payments of each.
static Counts strategies(EranosGroup const *const group) {
    section("3. Minimum transfers");
    EranosPaymentList *const minimum = planOf(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS);
    printFormatted("The fewest payments possible. Finding them is NP-hard; the solver does it exactly by\n"
                   "splitting the balances into as many groups that sum to zero as it can: here\n"
                   "{Ana, Dan, Eva} and {Bob, Carol, Filip}, each settled with one payment fewer than\n"
                   "its size. %zu payments:",
                   eranosPaymentListLength(minimum));
    printPayments(group, minimum);

    section("4. Greedy");
    EranosPaymentList *const greedy = planOf(group, ERANOS_SETTLEMENT_STRATEGY_GREEDY);
    printFormatted("The largest debtor pays the largest creditor, again and again: fast, at most one\n"
                   "payment fewer than the members with a balance, but not always the minimum. %zu\n"
                   "payments here:",
                   eranosPaymentListLength(greedy));
    printPayments(group, greedy);

    section("5. Direct");
    EranosPaymentList *const direct = planOf(group, ERANOS_SETTLEMENT_STRATEGY_DIRECT);
    printFormatted("No simplification: everyone pays back the people who paid for them, with the debts\n"
                   "between each pair of members netted. The most payments (%zu), but each one traces\n"
                   "back to shared expenses:",
                   eranosPaymentListLength(direct));
    printPayments(group, direct);

    section("6. Adaptive");
    EranosSettlement *const settlement = settlementOf(group, ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE);
    EranosPaymentList *const adaptive = eranosSettlementPayments(settlement);
    printFormatted("The fewest payments while they are quick to find, greedy's beyond: up to 20 open\n"
                   "balances the minimum solver settles them, past that greedy does, and the plan\n"
                   "names the strategy whose plan it is. Six balances are well within the limit, so\n"
                   "the plan is the one %s gives, the same %zu payments.",
                   strategyName(eranosSettlementStrategy(settlement)), eranosPaymentListLength(adaptive));

    section("7. The strategies side by side");
    Counts const counts = {
        .adaptive = eranosPaymentListLength(adaptive),
        .direct = eranosPaymentListLength(direct),
        .greedy = eranosPaymentListLength(greedy),
        .minimum = eranosPaymentListLength(minimum),
    };
    Table *const table = strategyTable(3);
    addStrategyRow(table, &(StrategyRow){
                              .cost = lineOf("O(2ⁿ·n) for n open balances"),
                              .guarantee = lineOf("the fewest payments"),
                              .payments = lineOf("%zu", counts.minimum),
                              .strategy = lineOf("MinimumTransfers"),
                          });
    addStrategyRow(table, &(StrategyRow){
                              .cost = lineOf("O(n log n)"),
                              .guarantee = lineOf("at most n − 1 payments"),
                              .payments = lineOf("%zu", counts.greedy),
                              .strategy = lineOf("Greedy"),
                          });
    addStrategyRow(table, &(StrategyRow){
                              .cost = lineOf("O(pairs)"),
                              .guarantee = lineOf("one payment per pair in debt"),
                              .payments = lineOf("%zu", counts.direct),
                              .strategy = lineOf("Direct"),
                          });
    addStrategyRow(table, &(StrategyRow){
                              .cost = lineOf("as MinimumTransfers, then Greedy"),
                              .guarantee = lineOf("the fewest up to 20 balances"),
                              .payments = lineOf("%zu", counts.adaptive),
                              .strategy = lineOf("Adaptive"),
                          });
    tablePrint(table);
    eranosPaymentListDestroy(adaptive);
    eranosSettlementDestroy(settlement);
    eranosPaymentListDestroy(direct);
    eranosPaymentListDestroy(greedy);
    eranosPaymentListDestroy(minimum);
    return counts;
}

/// How many members of @p group have a balance that is not zero.
static size_t openBalances(EranosGroup const *const group) {
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMemberSummaryList *const members = eranosSummaryMembers(summary);
    size_t open = 0;
    for (size_t index = 0; eranosMemberSummaryListLength(members) > index; ++index) {
        EranosMemberSummary *const totals = eranosMemberSummaryListGet(members, index);
        EranosMoney *const net = eranosMemberSummaryNet(totals);
        if (0 != eranosMoneyMinorUnits(net)) {
            ++open;
        }
        eranosMoneyDestroy(net);
        eranosMemberSummaryDestroy(totals);
    }
    eranosMemberSummaryListDestroy(members);
    eranosSummaryDestroy(summary);
    return open;
}

/// Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly, and prints which
/// strategy's plan it took; returns the number of payments.
static size_t festival(void) {
    section("8. A larger group");
    EranosNewGroup *details = nullptr;
    must(eranosNewGroupCreate("EUR", "Festival", &details));
    EranosGroup *group = nullptr;
    must(eranosGroupCreate(details, &group));
    eranosNewGroupDestroy(details);
    Id friends[FRIENDS] = {};
    EranosTextList *const everyone = eranosTextListCreate();
    for (size_t index = 0; FRIENDS > index; ++index) {
        Line const name = lineOf("Friend %zu", index + 1);
        friends[index] = addMember(group, name.text);
        must(eranosTextListPush(everyone, friends[index].text));
    }
    // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's
    // balance is 21·i² − 3311 cents: never zero, and never the opposite of another's.
    EranosCalendarDate const date = {.day = 18, .month = 9, .year = 2026};
    for (size_t index = 0; FRIENDS > index; ++index) {
        int64_t const number = (int64_t)index + 1;
        EranosMoney *amount = nullptr;
        must(eranosMoneyCreate("EUR", (int64_t)FRIENDS * number * number, &amount));
        addExpense(group, expenseOf(localCost(amount), date, "Round", soleSplit(friends[index]), eranosSplitCreateEqual(everyone)));
    }
    size_t const open = openBalances(group);
    EranosSettlement *const settlement = settlementOf(group, ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE);
    EranosPaymentList *const adaptive = eranosSettlementPayments(settlement);
    EranosPaymentList *const greedy = planOf(group, ERANOS_SETTLEMENT_STRATEGY_GREEDY);
    printFormatted("Twenty-one friends at a festival each bought one round for everyone, each round\n"
                   "at another price, so all %zu end with a balance and no two of them cancel out.\n"
                   "That is more than the 20 the adaptive strategy settles exactly, so it takes the\n"
                   "plan %s gives and says so: the same %zu payments.",
                   open, strategyName(eranosSettlementStrategy(settlement)), eranosPaymentListLength(adaptive));
    check(FRIENDS == open, "every friend has a balance");
    check(ERANOS_SETTLEMENT_STRATEGY_GREEDY == eranosSettlementStrategy(settlement), "the adaptive plan is greedy's");
    check(samePayments(adaptive, greedy), "the adaptive plan makes greedy's payments");
    size_t const payments = eranosPaymentListLength(adaptive);
    eranosPaymentListDestroy(greedy);
    eranosPaymentListDestroy(adaptive);
    eranosSettlementDestroy(settlement);
    eranosTextListDestroy(everyone);
    eranosGroupDestroy(group);
    return payments;
}

/// Records every payment of the minimum plan for @p group as a transfer.
static void settle(EranosGroup *const group) {
    EranosPaymentList *const plan = planOf(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS);
    EranosCalendarDate const date = {.day = 27, .month = 9, .year = 2026};
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosPaymentListLength(plan) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(plan, index);
        eranosPaymentFrom(payment, text);
        Id const from = idOf(text);
        eranosPaymentTo(payment, text);
        Id const to = idOf(text);
        addTransfer(group, transferOf(localCost(eranosPaymentAmount(payment)), date, from, to));
        eranosPaymentDestroy(payment);
    }
    eranosTextDestroy(text);
    eranosPaymentListDestroy(plan);
}

/// Whether @p member of @p group is settled.
static bool settled(EranosGroup const *const group, Id const member) {
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMemberSummary *const totals = totalsOf(summary, member.text);
    check(nullptr != totals, "the member has totals");
    EranosBalance *const balance = eranosMemberSummaryBalance(totals);
    bool const zero = ERANOS_BALANCE_KIND_SETTLED == eranosBalanceKind(balance);
    eranosBalanceDestroy(balance);
    eranosMemberSummaryDestroy(totals);
    eranosSummaryDestroy(summary);
    return zero;
}

int main(void) {
    EranosNewGroup *details = nullptr;
    must(eranosNewGroupCreate("EUR", "Ski week", &details));
    EranosGroup *group = nullptr;
    must(eranosGroupCreate(details, &group));
    eranosNewGroupDestroy(details);
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");
    Id const carol = addMember(group, "Carol");
    Id const dan = addMember(group, "Dan");
    Id const eva = addMember(group, "Eva");
    Id const filip = addMember(group, "Filip");

    title("Ski week: four ways to settle up");
    printLine("Six friends spent a week in the mountains. The library can turn their balances\n"
              "into payments in four ways, and the app picks one.");

    section("1. What was spent");
    spend(group, "12.00", "Cabin", dan, (Members){.count = 3, .ids = {dan, eva, filip}});
    spend(group, "9.00", "Lift passes", bob, (Members){.count = 3, .ids = {bob, carol, filip}});
    spend(group, "6.00", "Groceries", carol, (Members){.count = 2, .ids = {bob, dan}});
    spend(group, "2.00", "Coffee", ana, (Members){.count = 2, .ids = {dan, eva}});
    spend(group, "1.00", "Snacks", filip, (Members){.count = 1, .ids = {ana}});
    printExpenses(group);

    section("2. Balances");
    printLine("Four members are owed money and two owe it:");
    printBalances(group);

    Counts const counts = strategies(group);
    size_t const festivalPayments = festival();

    section("9. Settling up for real");
    settle(group);
    printLine("Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards\n"
              "every balance is zero and a new settlement has nothing left to pay:");
    printBalances(group);
    EranosPaymentList *const remaining = planOf(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS);
    printFormatted("Payments still needed: %zu.", eranosPaymentListLength(remaining));
    eranosPaymentListDestroy(remaining);

    // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice, and the adaptive plan is that one;
    // greedy pays the largest amounts first and needs five; the unsimplified debts between pairs need seven.
    check(4 == counts.adaptive && 7 == counts.direct && 5 == counts.greedy && 4 == counts.minimum,
          "the adaptive, direct, greedy, and minimum plans take 4, 7, 5, and 4 payments");
    // Greedy settles the festival's 21 balances in 19 payments.
    check(19 == festivalPayments, "the festival's plan takes 19 payments");
    Id const everyone[] = {ana, bob, carol, dan, eva, filip};
    for (size_t index = 0; 6 > index; ++index) {
        check(settled(group, everyone[index]), "every member is settled");
    }
    eranosGroupDestroy(group);
    return EXIT_SUCCESS;
}
