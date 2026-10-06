// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until everyone
// is settled. The port of the Rust example `settling_up`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdlib.h>

/// How many payments each strategy plans.
typedef struct Counts {
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

/// The payments of the plan that @p strategy makes for @p group, which the caller destroys.
static EranosPaymentList *planOf(EranosGroup const *const group, EranosSettlementStrategy const strategy) {
    EranosSettlement *settlement = nullptr;
    must(eranosGroupSettlement(group, strategy, &settlement));
    EranosPaymentList *const payments = eranosSettlementPayments(settlement);
    eranosSettlementDestroy(settlement);
    return payments;
}

/// Prints each strategy's plan with what it promises.
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

    section("6. The strategies side by side");
    Counts const counts = {
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
    tablePrint(table);
    eranosPaymentListDestroy(direct);
    eranosPaymentListDestroy(greedy);
    eranosPaymentListDestroy(minimum);
    return counts;
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

    printLine("Ski week: three ways to settle up\n"
              "═════════════════════════════════");
    printLine("Six friends spent a week in the mountains. The library can turn their balances\n"
              "into payments in three ways; the app picks one, and they differ here.");

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

    section("7. Settling up for real");
    settle(group);
    printLine("Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards\n"
              "every balance is zero and a new settlement has nothing left to pay:");
    printBalances(group);
    EranosPaymentList *const remaining = planOf(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS);
    printFormatted("Payments still needed: %zu.", eranosPaymentListLength(remaining));
    eranosPaymentListDestroy(remaining);

    // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice; greedy pays the largest amounts
    // first and needs five; the unsimplified debts between pairs need seven.
    check(4 == counts.minimum && 5 == counts.greedy && 7 == counts.direct, "the plans take 4, 5, and 7 payments");
    Id const everyone[] = {ana, bob, carol, dan, eva, filip};
    for (size_t index = 0; 6 > index; ++index) {
        check(settled(group, everyone[index]), "every member is settled");
    }
    eranosGroupDestroy(group);
    return EXIT_SUCCESS;
}
