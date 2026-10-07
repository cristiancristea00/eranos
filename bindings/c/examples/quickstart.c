// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the Rust
// example `quickstart`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdlib.h>

/// A row of the table of expenses: what was bought, who paid it, who shares it, and its amount.
typedef struct ExpenseRow {
    /// What was bought.
    Line name;
    /// Who paid it.
    Line payers;
    /// Who shares it.
    Line participants;
    /// What it cost.
    Line amount;
} ExpenseRow;

/// A table of expenses, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *expenseTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Expense", "Paid by", "Shared by", "Amount"}, numbersFrom);
}

/// Adds @p row to @p table, a table of expenses, its cells in the order of the table's headers.
static void addExpenseRow(Table *const table, ExpenseRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->payers, row->participants, row->amount});
}

/// A row of the table of members: what a member paid and consumed, and their balance.
typedef struct MemberRow {
    /// Their first name.
    Line name;
    /// What they paid.
    Line paid;
    /// What they consumed.
    Line consumed;
    /// What is left: what they paid minus what they consumed.
    Line balance;
} MemberRow;

/// A table of members, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *memberTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Member", "Paid", "Consumed", "Balance"}, numbersFrom);
}

/// Adds @p row to @p table, a table of members, its cells in the order of the table's headers.
static void addMemberRow(Table *const table, MemberRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->paid, row->consumed, row->balance});
}

/// A row of the table of payments: who pays whom, and how much.
typedef struct PaymentRow {
    /// Who pays.
    Line from;
    /// Who is paid.
    Line to;
    /// How much is paid.
    Line amount;
} PaymentRow;

/// A table of payments, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *paymentTable(size_t const numbersFrom) {
    return tableCreate(3, (char const *const[]){"From", "To", "Amount"}, numbersFrom);
}

/// Adds @p row to @p table, a table of payments, its cells in the order of the table's headers.
static void addPaymentRow(Table *const table, PaymentRow const *const row) { tableAdd(table, (Line const[]){row->from, row->to, row->amount}); }

/// The amount @p text of euros.
[[nodiscard]] static EranosMoney *euros(char const *const text) { return amountOf(text, "EUR"); }

/// Prints the expenses of @p group: who paid each, who shares it, and its amount.
static void printExpenses(EranosExpenseGroup const *const group) {
    EranosExpenseList *expenses = nullptr;
    must(eranosExpenseGroupExpenses(group, &expenses));
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
                                 .name = textLine(name),
                                 .payers = names(group, &paidBy),
                                 .participants = names(group, &sharedBy),
                                 .amount = moneyLine(total),
                             });
        eranosSplitDestroy(payers);
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(name);
    eranosExpenseListDestroy(expenses);
    tablePrint(table);
}

/// Prints what each member of @p group paid and consumed in @p summary, and their balance.
static void printMembers(EranosExpenseGroup const *const group, EranosSummary const *const summary) {
    EranosMemberList *members = nullptr;
    must(eranosExpenseGroupMembers(group, &members));
    Table *const table = memberTable(1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        eranosMemberId(member, text);
        EranosMemberSummary *const totals = totalsOf(summary, eranosTextData(text));
        check(nullptr != totals, "every member has totals");
        eranosMemberFirstName(member, text);
        addMemberRow(table, &(MemberRow){
                                .name = textLine(text),
                                .paid = moneyLine(eranosMemberSummaryPaid(totals)),
                                .consumed = moneyLine(eranosMemberSummaryShare(totals)),
                                .balance = balanceLine(eranosMemberSummaryBalance(totals)),
                            });
        eranosMemberSummaryDestroy(totals);
        eranosMemberDestroy(member);
    }
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    tablePrint(table);
}

/// Prints @p payments: who pays whom, and how much.
static void printPayments(EranosExpenseGroup const *const group, EranosPaymentList const *const payments) {
    Table *const table = paymentTable(2);
    EranosText *const from = eranosTextCreate();
    EranosText *const to = eranosTextCreate();
    for (size_t index = 0; eranosPaymentListLength(payments) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(payments, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        addPaymentRow(table, &(PaymentRow){
                                 .from = firstName(group, eranosTextData(from)),
                                 .to = firstName(group, eranosTextData(to)),
                                 .amount = moneyLine(eranosPaymentAmount(payment)),
                             });
        eranosPaymentDestroy(payment);
    }
    eranosTextDestroy(to);
    eranosTextDestroy(from);
    tablePrint(table);
}

int main(void) {
    EranosNewExpenseGroup *details = nullptr;
    must(eranosNewExpenseGroupCreate("Weekend in Sinaia", "EUR", &details));
    EranosExpenseGroup *group = nullptr;
    must(eranosExpenseGroupCreate(details, &group));
    eranosNewExpenseGroupDestroy(details);
    EranosText *const text = eranosTextCreate();
    must(eranosExpenseGroupName(group, text));
    title(eranosTextData(text));
    must(eranosExpenseGroupCurrency(group, text));
    printFormatted("Three friends rent a cabin and buy groceries. The group keeps its books in %s, so\n"
                   "every total, balance, and payment below is in %s.",
                   eranosTextData(text), eranosTextData(text));

    section("1. Members");
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");
    Id const carol = addMember(group, "Carol");
    Members const everyone = {.count = 3, .ids = {ana, bob, carol}};
    EranosMemberList *members = nullptr;
    must(eranosExpenseGroupMembers(group, &members));
    Line const joined = names(group, &everyone);
    printFormatted("%zu people joined the group: %s.", eranosMemberListLength(members), joined.text);
    eranosMemberListDestroy(members);

    section("2. Expenses");
    printLine("Each expense records who paid and who shares it. Here everyone shares everything\n"
              "equally.");
    constexpr EranosCalendarDate FRIDAY = {.year = 2026, .month = 9, .day = 25};
    (void)addExpense(group, expenseOf("Cabin", localCost(euros("240.00")), soleSplit(ana), equalSplit(&everyone), FRIDAY));
    (void)addExpense(group, expenseOf("Groceries", localCost(euros("60.00")), soleSplit(bob), equalSplit(&everyone), FRIDAY));
    printExpenses(group);

    section("3. Who paid and who consumed");
    EranosSummary *summary = nullptr;
    must(eranosExpenseGroupSummary(group, &summary));
    Line const spent = moneyLine(eranosSummaryTotalSpent(summary));
    printFormatted("The group spent %s in total. A member's share of an expense is what they consumed,\n"
                   "and their balance is what they paid minus what they consumed.",
                   spent.text);
    printMembers(group, summary);
    EranosMemberSummary *const anaTotals = totalsOf(summary, ana.text);
    check(nullptr != anaTotals, "Ana has totals");
    Line const paid = moneyLine(eranosMemberSummaryPaid(anaTotals));
    Line const consumed = moneyLine(eranosMemberSummaryShare(anaTotals));
    printFormatted("Ana paid %s but consumed only %s, so the others owe her the difference.", paid.text, consumed.text);

    section("4. Settling up");
    EranosSettlementPlan *settlement = nullptr;
    must(eranosExpenseGroupSettlement(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS, &settlement));
    EranosPaymentList *const payments = eranosSettlementPlanPayments(settlement);
    printFormatted("The fewest payments that bring every balance to zero (%zu in total):", eranosPaymentListLength(payments));
    printPayments(group, payments);
    printLine("Once these payments are made, nobody owes anything.");

    // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
    EranosMoney *const total = eranosSummaryTotalSpent(summary);
    check(30'000 == eranosMoneyMinorUnits(total), "the group spent 300.00 EUR");
    EranosBalance *const balance = eranosMemberSummaryBalance(anaTotals);
    EranosMoney *const owed = eranosBalanceAmount(balance);
    check(ERANOS_BALANCE_KIND_GETS_BACK == eranosBalanceKind(balance) && 14'000 == eranosMoneyMinorUnits(owed), "Ana gets back 140.00 EUR");
    check(2 == eranosPaymentListLength(payments), "two payments settle the group");
    eranosMoneyDestroy(owed);
    eranosBalanceDestroy(balance);
    eranosMoneyDestroy(total);
    eranosPaymentListDestroy(payments);
    eranosSettlementPlanDestroy(settlement);
    eranosMemberSummaryDestroy(anaTotals);
    eranosSummaryDestroy(summary);
    eranosTextDestroy(text);
    eranosExpenseGroupDestroy(group);
    return EXIT_SUCCESS;
}
