// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the Rust
// example `quickstart`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdlib.h>

/// The amount @p text of euros.
static EranosMoney *euros(char const *const text) { return amountOf(text, "EUR"); }

/// Prints the expenses of @p group: who paid each, who shares it, and its amount.
static void printExpenses(EranosGroup const *const group) {
    EranosExpenseList *expenses = NULL;
    must(eranosGroupExpenses(group, &expenses));
    Table *const table = tableCreate(4, (char const *const[]){"Expense", "Paid by", "Shared by", "Amount"}, 3);
    EranosText *const name = eranosTextCreate();
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        eranosExpenseName(expense, name);
        EranosSplit *const payers = eranosExpensePayers(expense);
        Members const paidBy = splitMembers(payers);
        Members const sharedBy = participants(expense);
        EranosMoney *total = NULL;
        must(eranosExpenseTotal(expense, &total));
        tableAdd(table, (Line const[]){textLine(name), names(group, &paidBy), names(group, &sharedBy), moneyLine(total)});
        eranosSplitDestroy(payers);
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(name);
    eranosExpenseListDestroy(expenses);
    tablePrint(table);
}

/// Prints what each member of @p group paid and consumed in @p summary, and their balance.
static void printMembers(EranosGroup const *const group, EranosSummary const *const summary) {
    EranosMemberList *members = NULL;
    must(eranosGroupMembers(group, &members));
    Table *const table = tableCreate(4, (char const *const[]){"Member", "Paid", "Consumed", "Balance"}, 1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        EranosMemberSummary *const totals = totalsOf(summary, eranosTextData(text));
        check(NULL != totals, "every member has totals");
        eranosMemberFirstName(member, text);
        tableAdd(table, (Line const[]){
                            textLine(text),
                            moneyLine(eranosMemberSummaryPaid(totals)),
                            moneyLine(eranosMemberSummaryShare(totals)),
                            balanceLine(eranosMemberSummaryBalance(totals)),
                        });
        eranosMemberSummaryDestroy(totals);
        eranosMemberDestroy(member);
    }
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    tablePrint(table);
}

/// Prints @p payments: who pays whom, and how much.
static void printPayments(EranosGroup const *const group, EranosPaymentList const *const payments) {
    Table *const table = tableCreate(3, (char const *const[]){"From", "To", "Amount"}, 2);
    EranosText *const from = eranosTextCreate();
    EranosText *const to = eranosTextCreate();
    for (size_t index = 0; eranosPaymentListLength(payments) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(payments, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        tableAdd(table, (Line const[]){
                            firstName(group, eranosTextData(from)),
                            firstName(group, eranosTextData(to)),
                            moneyLine(eranosPaymentAmount(payment)),
                        });
        eranosPaymentDestroy(payment);
    }
    eranosTextDestroy(to);
    eranosTextDestroy(from);
    tablePrint(table);
}

int main(void) {
    EranosNewGroup *details = NULL;
    must(eranosNewGroupCreate("EUR", "Weekend in Sinaia", &details));
    EranosGroup *group = NULL;
    must(eranosGroupCreate(details, &group));
    eranosNewGroupDestroy(details);
    EranosText *const text = eranosTextCreate();
    must(eranosGroupName(group, text));
    title(eranosTextData(text));
    must(eranosGroupCurrency(group, text));
    printFormatted("Three friends rent a cabin and buy groceries. The group keeps its books in %s, so\n"
                   "every total, balance, and payment below is in %s.",
                   eranosTextData(text), eranosTextData(text));

    section("1. Members");
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");
    Id const carol = addMember(group, "Carol");
    Members const everyone = {.count = 3, .ids = {ana, bob, carol}};
    EranosMemberList *members = NULL;
    must(eranosGroupMembers(group, &members));
    Line const joined = names(group, &everyone);
    printFormatted("%zu people joined the group: %s.", eranosMemberListLength(members), joined.text);
    eranosMemberListDestroy(members);

    section("2. Expenses");
    printLine("Each expense records who paid and who shares it. Here everyone shares everything\n"
              "equally.");
    EranosCalendarDate const friday = {.day = 25, .month = 9, .year = 2026};
    addExpense(group, expenseOf(localCost(euros("240.00")), friday, "Cabin", soleSplit(ana), equalSplit(&everyone)));
    addExpense(group, expenseOf(localCost(euros("60.00")), friday, "Groceries", soleSplit(bob), equalSplit(&everyone)));
    printExpenses(group);

    section("3. Who paid and who consumed");
    EranosSummary *summary = NULL;
    must(eranosGroupSummary(group, &summary));
    Line const spent = moneyLine(eranosSummaryTotalSpent(summary));
    printFormatted("The group spent %s in total. A member's share of an expense is what they consumed,\n"
                   "and their balance is what they paid minus what they consumed.",
                   spent.text);
    printMembers(group, summary);
    EranosMemberSummary *const anaTotals = totalsOf(summary, ana.text);
    check(NULL != anaTotals, "Ana has totals");
    Line const paid = moneyLine(eranosMemberSummaryPaid(anaTotals));
    Line const consumed = moneyLine(eranosMemberSummaryShare(anaTotals));
    printFormatted("Ana paid %s but consumed only %s, so the others owe her the difference.", paid.text, consumed.text);

    section("4. Settling up");
    EranosSettlement *settlement = NULL;
    must(eranosGroupSettlement(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS, &settlement));
    EranosPaymentList *const payments = eranosSettlementPayments(settlement);
    printFormatted("The fewest payments that bring every balance to zero (%zu in total):", eranosPaymentListLength(payments));
    printPayments(group, payments);
    printLine("Once these payments are made, nobody owes anything.");

    // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
    EranosMoney *const total = eranosSummaryTotalSpent(summary);
    check(30000 == eranosMoneyMinorUnits(total), "the group spent 300.00 EUR");
    EranosBalance *const balance = eranosMemberSummaryBalance(anaTotals);
    EranosMoney *const owed = eranosBalanceAmount(balance);
    check(ERANOS_BALANCE_KIND_GETS_BACK == eranosBalanceKind(balance) && 14000 == eranosMoneyMinorUnits(owed), "Ana gets back 140.00 EUR");
    check(2 == eranosPaymentListLength(payments), "two payments settle the group");
    eranosMoneyDestroy(owed);
    eranosBalanceDestroy(balance);
    eranosMoneyDestroy(total);
    eranosPaymentListDestroy(payments);
    eranosSettlementDestroy(settlement);
    eranosMemberSummaryDestroy(anaTotals);
    eranosSummaryDestroy(summary);
    eranosTextDestroy(text);
    eranosGroupDestroy(group);
    return EXIT_SUCCESS;
}
