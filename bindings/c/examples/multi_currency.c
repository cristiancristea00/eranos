// A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON and HUF carry a rate
// fixed when they are entered, and the group later moves its books to RON. The port of the Rust example
// `multi_currency`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/// A row of the table of entries: an expense or a transfer, who paid it, and its amount as entered and in the group
/// currency.
typedef struct EntryRow {
    /// What it was.
    Line name;
    /// Who paid it.
    Line payer;
    /// Its amount as entered.
    Line original;
    /// The rate it was entered at.
    Line rate;
    /// Its amount in the group currency.
    Line converted;
} EntryRow;

/// A table of entries, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *entryTable(size_t const numbersFrom) {
    return tableCreate(5, (char const *const[]){"Entry", "Paid by", "Entered as", "Rate", "In the group currency"}, numbersFrom);
}

/// Adds @p row to @p table, a table of entries, its cells in the order of the table's headers.
static void addEntryRow(Table *const table, EntryRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->payer, row->original, row->rate, row->converted});
}

/// A row of the table of balances: what a member paid, consumed, sent, and received, and their balance.
typedef struct BalanceRow {
    /// Their first name.
    Line name;
    /// What they paid.
    Line paid;
    /// What they consumed.
    Line consumed;
    /// What they sent.
    Line sent;
    /// What they received.
    Line received;
    /// What is left.
    Line balance;
} BalanceRow;

/// A table of balances, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *balanceTable(size_t const numbersFrom) {
    return tableCreate(6, (char const *const[]){"Member", "Paid", "Consumed", "Sent", "Received", "Balance"}, numbersFrom);
}

/// Adds @p row to @p table, a table of balances, its cells in the order of the table's headers.
static void addBalanceRow(Table *const table, BalanceRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->paid, row->consumed, row->sent, row->received, row->balance});
}

/// The rate @p cost was entered at: 1 for a cost in the group currency; destroys @p cost.
[[nodiscard]] static Line rateLine(EranosCost *const cost) {
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
[[nodiscard]] static Line amountOrZero(EranosExpenseGroup const *const group, EranosMoney *const amount) {
    if (nullptr != amount) {
        return moneyLine(amount);
    }
    EranosCurrencyDetails *const currency = eranosExpenseGroupCurrency(group);
    EranosMoney *const zero = eranosMoneyCreate(0, currency);
    eranosCurrencyDetailsDestroy(currency);
    return moneyLine(zero);
}

/// Adds every expense of @p group to @p table, as entered and in the group currency.
static void addExpenses(EranosExpenseGroup const *const group, Table *const table) {
    EranosExpenseList *const expenses = eranosExpenseGroupExpenses(group);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        eranosExpenseId(expense, text);
        EranosExpenseShares *shares = nullptr;
        must(eranosExpenseGroupExpenseShares(group, eranosTextData(text), &shares));
        eranosExpenseName(expense, text);
        EranosSplit *const payers = eranosExpensePayers(expense);
        Members const paidBy = splitMembers(payers);
        EranosMoney *total = nullptr;
        must(eranosExpenseTotal(expense, &total));
        addEntryRow(table, &(EntryRow){
                               .name = textLine(text),
                               .payer = names(group, &paidBy),
                               .original = moneyLine(total),
                               .rate = rateLine(eranosExpenseCost(expense)),
                               .converted = amountOrZero(group, nullptr == shares ? nullptr : eranosExpenseSharesTotal(shares)),
                           });
        eranosSplitDestroy(payers);
        eranosExpenseSharesDestroy(shares);
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(text);
    eranosExpenseListDestroy(expenses);
}

/// Adds every transfer of @p group to @p table, as entered and in the group currency.
static void addTransfers(EranosExpenseGroup const *const group, Table *const table) {
    EranosTransferList *const transfers = eranosExpenseGroupTransfers(group);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosTransferListLength(transfers) > index; ++index) {
        EranosTransfer *const transfer = eranosTransferListGet(transfers, index);
        eranosTransferId(transfer, text);
        EranosMoney *converted = nullptr;
        must(eranosExpenseGroupTransferAmount(group, eranosTextData(text), &converted));
        eranosTransferReceiver(transfer, text);
        Line const receiver = firstName(group, eranosTextData(text));
        eranosTransferSender(transfer, text);
        EranosCost *const cost = eranosTransferCost(transfer);
        addEntryRow(table, &(EntryRow){
                               .name = lineOf("Transfer to %s", receiver.text),
                               .payer = firstName(group, eranosTextData(text)),
                               .original = moneyLine(eranosCostAmount(cost)),
                               .rate = rateLine(eranosTransferCost(transfer)),
                               .converted = amountOrZero(group, converted),
                           });
        eranosCostDestroy(cost);
        eranosTransferDestroy(transfer);
    }
    eranosTextDestroy(text);
    eranosTransferListDestroy(transfers);
}

/// Prints every expense and transfer of @p group, as entered and in the group currency.
static void printEntries(EranosExpenseGroup const *const group) {
    Table *const table = entryTable(2);
    addExpenses(group, table);
    addTransfers(group, table);
    tablePrint(table);
}

/// Prints what each member of @p group paid, consumed, sent, and received, and their balance.
static void printBalances(EranosExpenseGroup const *const group) {
    EranosSummary *const summary = eranosExpenseGroupSummary(group);
    EranosMemberList *const members = eranosExpenseGroupMembers(group);
    Table *const table = balanceTable(1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        eranosMemberId(member, text);
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        eranosMemberFirstName(member, text);
        addBalanceRow(table, &(BalanceRow){
                                 .name = textLine(text),
                                 .paid = moneyLine(eranosMemberSummaryPaid(found)),
                                 .consumed = moneyLine(eranosMemberSummaryShare(found)),
                                 .sent = moneyLine(eranosMemberSummarySent(found)),
                                 .received = moneyLine(eranosMemberSummaryReceived(found)),
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

/// The total that @p group spent, as the library writes it.
[[nodiscard]] static Line totalSpent(EranosExpenseGroup const *const group) {
    EranosSummary *const summary = eranosExpenseGroupSummary(group);
    Line const line = moneyLine(eranosSummaryTotalSpent(summary));
    eranosSummaryDestroy(summary);
    return line;
}

/// The currency of @p group, as its code.
[[nodiscard]] static Line currencyOf(EranosExpenseGroup const *const group) { return currencyLine(eranosExpenseGroupCurrency(group)); }

/// Whether the balance of @p member in @p group is of @p kind, and of @p minorUnits when it has an amount.
[[nodiscard]] static bool balanceIs(EranosExpenseGroup const *const group, Id const member, EranosBalanceKind const kind, int64_t const minorUnits) {
    EranosSummary *const summary = eranosExpenseGroupSummary(group);
    EranosMemberSummary *const totals = totalsOf(summary, member.text);
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
static void expenses(EranosExpenseGroup *const group, Id const ana, Id const bob) {
    section("1. Expenses in three currencies");
    Members const both = {.count = 2, .ids = {ana, bob}};
    (void)addExpense(group, expenseOf("Fuel", convertedCost(amountOf("400.00", "RON"), "0.2"), soleSplit(ana), equalSplit(&both),
                                      (EranosCalendarDate){.year = 2026, .month = 8, .day = 1}));
    (void)addExpense(group, expenseOf("Dinner", convertedCost(amountOf("24000.00", "HUF"), "0.0025"), soleSplit(bob), equalSplit(&both),
                                      (EranosCalendarDate){.year = 2026, .month = 8, .day = 2}));
    (void)addExpense(group, expenseOf("Ferry", localCost(amountOf("60.00", "EUR")), soleSplit(bob), equalSplit(&both),
                                      (EranosCalendarDate){.year = 2026, .month = 8, .day = 3}));
    printEntries(group);
    Line const currency = currencyOf(group);
    printFormatted("The rate says how many %s one unit of the entry's currency is worth: 400.00 RON at\n"
                   "0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.",
                   currency.text);
}

/// Adds Ana's repayment to Bob, in cash and in RON.
static void repayment(EranosExpenseGroup *const group, Id const ana, Id const bob) {
    section("3. A repayment in cash, in another currency");
    EranosTransfer *const transfer =
        transferOf(ana, bob, convertedCost(amountOf("100.00", "RON"), "0.2"), (EranosCalendarDate){.year = 2026, .month = 8, .day = 4});
    must(eranosTransferSetNote(transfer, "Cash at the border"));
    (void)addTransfer(group, transfer);
    Line const spent = totalSpent(group);
    printFormatted("Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.\n"
                   "Transfers move balances but are not spending, so the total stays at %s:",
                   spent.text);
    printBalances(group);
}

/// Moves the books of @p group to RON.
static void newCurrency(EranosExpenseGroup *const group) {
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
    must(eranosExpenseGroupChangeCurrency(group, "RON", rates));
    eranosCurrencyRateListDestroy(rates);
    printEntries(group);
    Line const currency = currencyOf(group);
    Line const spent = totalSpent(group);
    printFormatted("In %s, the trip cost %s, and everyone is still settled:", currency.text, spent.text);
    printBalances(group);
}

int main(void) {
    EranosNewExpenseGroup *details = nullptr;
    must(eranosNewExpenseGroupCreate("Danube road trip", "EUR", &details));
    EranosExpenseGroup *group = nullptr;
    must(eranosExpenseGroupCreate(details, &group));
    eranosNewExpenseGroupDestroy(details);
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");

    EranosText *const name = eranosTextCreate();
    eranosExpenseGroupName(group, name);
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
    EranosSummary *const summary = eranosExpenseGroupSummary(group);
    EranosMoney *const total = eranosSummaryTotalSpent(summary);
    check(100'000 == eranosMoneyMinorUnits(total), "the trip cost 1000.00 RON");
    check(balanceIs(group, bob, ERANOS_BALANCE_KIND_SETTLED, 0), "Bob is still settled");
    eranosMoneyDestroy(total);
    eranosSummaryDestroy(summary);
    eranosExpenseGroupDestroy(group);
    return EXIT_SUCCESS;
}
