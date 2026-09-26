// A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON and HUF carry a rate
// fixed when they are entered, and the group later moves its books to RON. The port of the Rust example
// `multi_currency`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/// The rate @p cost was entered at: 1 for a cost in the group currency; destroys @p cost.
static Line rateLine(EranosCost *const cost) {
    EranosText *const text = eranosTextCreate();
    if (!eranosCostRate(cost, text)) {
        must(eranosExchangeRateParse("1", text));
    }
    Line const line = textLine(text);
    eranosTextDestroy(text);
    eranosCostDestroy(cost);
    return line;
}

/// @p amount, or zero of the currency of @p group when it is nullptr.
static Line amountOrZero(EranosGroup const *const group, EranosMoney *const amount) {
    if (nullptr != amount) {
        return moneyLine(amount);
    }
    EranosText *const currency = eranosTextCreate();
    must(eranosGroupCurrency(group, currency));
    EranosMoney *zero = nullptr;
    must(eranosMoneyCreate(eranosTextData(currency), 0, &zero));
    eranosTextDestroy(currency);
    return moneyLine(zero);
}

/// Adds every expense of @p group to @p table, as entered and in the group currency.
static void addExpenses(EranosGroup const *const group, Table *const table) {
    EranosExpenseList *expenses = nullptr;
    must(eranosGroupExpenses(group, &expenses));
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        (void)eranosExpenseId(expense, text);
        EranosExpenseShares *shares = nullptr;
        must(eranosGroupExpenseShares(group, eranosTextData(text), &shares));
        eranosExpenseName(expense, text);
        EranosSplit *const payers = eranosExpensePayers(expense);
        Members const paidBy = splitMembers(payers);
        EranosMoney *total = nullptr;
        must(eranosExpenseTotal(expense, &total));
        tableAdd(table, (Line const[]){
                            textLine(text),
                            names(group, &paidBy),
                            moneyLine(total),
                            rateLine(eranosExpenseCost(expense)),
                            amountOrZero(group, nullptr == shares ? nullptr : eranosExpenseSharesTotal(shares)),
                        });
        eranosSplitDestroy(payers);
        eranosExpenseSharesDestroy(shares);
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(text);
    eranosExpenseListDestroy(expenses);
}

/// Adds every transfer of @p group to @p table, as entered and in the group currency.
static void addTransfers(EranosGroup const *const group, Table *const table) {
    EranosTransferList *transfers = nullptr;
    must(eranosGroupTransfers(group, &transfers));
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosTransferListLength(transfers) > index; ++index) {
        EranosTransfer *const transfer = eranosTransferListGet(transfers, index);
        (void)eranosTransferId(transfer, text);
        EranosMoney *converted = nullptr;
        must(eranosGroupTransferAmount(group, eranosTextData(text), &converted));
        eranosTransferTo(transfer, text);
        Line const to = firstName(group, eranosTextData(text));
        eranosTransferFrom(transfer, text);
        EranosCost *const cost = eranosTransferCost(transfer);
        tableAdd(table, (Line const[]){
                            lineOf("Transfer to %s", to.text),
                            firstName(group, eranosTextData(text)),
                            moneyLine(eranosCostAmount(cost)),
                            rateLine(eranosTransferCost(transfer)),
                            amountOrZero(group, converted),
                        });
        eranosCostDestroy(cost);
        eranosTransferDestroy(transfer);
    }
    eranosTextDestroy(text);
    eranosTransferListDestroy(transfers);
}

/// Prints every expense and transfer of @p group, as entered and in the group currency.
static void printEntries(EranosGroup const *const group) {
    Table *const table = tableCreate(5, (char const *const[]){"Entry", "Paid by", "Entered as", "Rate", "In the group currency"}, 2);
    addExpenses(group, table);
    addTransfers(group, table);
    tablePrint(table);
}

/// Prints what each member of @p group paid, consumed, sent, and received, and their balance.
static void printBalances(EranosGroup const *const group) {
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMemberList *members = nullptr;
    must(eranosGroupMembers(group, &members));
    Table *const table = tableCreate(6, (char const *const[]){"Member", "Paid", "Consumed", "Sent", "Received", "Balance"}, 1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        check(nullptr != found, "every member has totals");
        eranosMemberFirstName(member, text);
        tableAdd(table, (Line const[]){
                            textLine(text),
                            moneyLine(eranosMemberSummaryPaid(found)),
                            moneyLine(eranosMemberSummaryShare(found)),
                            moneyLine(eranosMemberSummarySent(found)),
                            moneyLine(eranosMemberSummaryReceived(found)),
                            balanceLine(eranosMemberSummaryBalance(found)),
                        });
        eranosMemberSummaryDestroy(found);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
}

/// The total that @p group spent, as the library writes it.
static Line totalSpent(EranosGroup const *const group) {
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    Line const line = moneyLine(eranosSummaryTotalSpent(summary));
    eranosSummaryDestroy(summary);
    return line;
}

/// The currency of @p group, as its code.
static Line currencyOf(EranosGroup const *const group) {
    EranosText *const text = eranosTextCreate();
    must(eranosGroupCurrency(group, text));
    Line const line = textLine(text);
    eranosTextDestroy(text);
    return line;
}

/// Whether the balance of @p member in @p group is of @p kind, and of @p minorUnits when it has an amount.
static bool balanceIs(EranosGroup const *const group, Id const member, EranosBalanceKind const kind, int64_t const minorUnits) {
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMemberSummary *const totals = totalsOf(summary, member.text);
    check(nullptr != totals, "the member has totals");
    EranosBalance *const balance = eranosMemberSummaryBalance(totals);
    EranosMoney *const amount = eranosBalanceAmount(balance);
    bool const same = kind == eranosBalanceKind(balance) && (nullptr == amount || minorUnits == eranosMoneyMinorUnits(amount));
    eranosMoneyDestroy(amount);
    eranosBalanceDestroy(balance);
    eranosMemberSummaryDestroy(totals);
    eranosSummaryDestroy(summary);
    return same;
}

/// Adds the fuel in RON, the dinner in HUF, and the ferry in EUR, which Ana and Bob share.
static void expenses(EranosGroup *const group, Id const ana, Id const bob) {
    section("1. Expenses in three currencies");
    Members const both = {.count = 2, .ids = {ana, bob}};
    addExpense(group, expenseOf(convertedCost(amountOf("400.00", "RON"), "0.2"), (EranosCalendarDate){.day = 1, .month = 8, .year = 2026}, "Fuel",
                                soleSplit(ana), equalSplit(&both)));
    addExpense(group, expenseOf(convertedCost(amountOf("24000.00", "HUF"), "0.0025"), (EranosCalendarDate){.day = 2, .month = 8, .year = 2026},
                                "Dinner", soleSplit(bob), equalSplit(&both)));
    addExpense(group, expenseOf(localCost(amountOf("60.00", "EUR")), (EranosCalendarDate){.day = 3, .month = 8, .year = 2026}, "Ferry",
                                soleSplit(bob), equalSplit(&both)));
    printEntries(group);
    Line const currency = currencyOf(group);
    printFormatted("The rate says how many %s one unit of the entry's currency is worth: 400.00 RON at\n"
                   "0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.",
                   currency.text);
}

/// Adds Ana's repayment to Bob, in cash and in RON.
static void repayment(EranosGroup *const group, Id const ana, Id const bob) {
    section("3. A repayment in cash, in another currency");
    EranosTransfer *const transfer =
        transferOf(convertedCost(amountOf("100.00", "RON"), "0.2"), (EranosCalendarDate){.day = 4, .month = 8, .year = 2026}, ana, bob);
    must(eranosTransferSetNote(transfer, "Cash at the border"));
    addTransfer(group, transfer);
    Line const spent = totalSpent(group);
    printFormatted("Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.\n"
                   "Transfers move balances but are not spending, so the total stays at %s:",
                   spent.text);
    printBalances(group);
}

/// Moves the books of @p group to RON.
static void newCurrency(EranosGroup *const group) {
    section("4. Moving the books to another currency");
    printLine("Changing the group currency needs a rate into the new currency for every other\n"
              "currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies\n"
              "take the new rate; entries already in RON become local costs, at 1.");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = nullptr;
    must(eranosCurrencyRateCreate("EUR", "5", &rate));
    eranosCurrencyRateListPush(rates, rate);
    eranosCurrencyRateDestroy(rate);
    must(eranosCurrencyRateCreate("HUF", "0.0125", &rate));
    eranosCurrencyRateListPush(rates, rate);
    eranosCurrencyRateDestroy(rate);
    must(eranosGroupChangeCurrency(group, "RON", rates));
    eranosCurrencyRateListDestroy(rates);
    printEntries(group);
    Line const currency = currencyOf(group);
    Line const spent = totalSpent(group);
    printFormatted("In %s, the trip cost %s, and everyone is still settled:", currency.text, spent.text);
    printBalances(group);
}

int main(void) {
    EranosNewGroup *details = nullptr;
    must(eranosNewGroupCreate("EUR", "Danube road trip", &details));
    EranosGroup *group = nullptr;
    must(eranosGroupCreate(details, &group));
    eranosNewGroupDestroy(details);
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");

    EranosText *const name = eranosTextCreate();
    must(eranosGroupName(group, name));
    printFormatted("%s: expenses in several currencies", eranosTextData(name));
    eranosTextDestroy(name);
    printLine("════════════════════════════════════════════════");
    Line const currency = currencyOf(group);
    printFormatted("Ana and Bob drive from Romania to Hungary. The group keeps its books in %s; an\n"
                   "expense in another currency carries the exchange rate used when it was entered,\n"
                   "and is converted once, rounding half away from zero to the cent.",
                   currency.text);
    expenses(group, ana, bob);

    section("2. Balances");
    Line const spent = totalSpent(group);
    printFormatted("The group spent %s. Bob paid 120.00 of it but consumed half, so he gets back the\n"
                   "difference:",
                   spent.text);
    printBalances(group);
    check(balanceIs(group, bob, ERANOS_BALANCE_KIND_GETS_BACK, 2'000), "Bob gets back 20.00 EUR");

    repayment(group, ana, bob);
    check(balanceIs(group, bob, ERANOS_BALANCE_KIND_SETTLED, 0), "Bob is settled");

    newCurrency(group);
    // 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMoney *const total = eranosSummaryTotalSpent(summary);
    check(100'000 == eranosMoneyMinorUnits(total), "the trip cost 1000.00 RON");
    check(balanceIs(group, bob, ERANOS_BALANCE_KIND_SETTLED, 0), "Bob is still settled");
    eranosMoneyDestroy(total);
    eranosSummaryDestroy(summary);
    eranosGroupDestroy(group);
    return EXIT_SUCCESS;
}
