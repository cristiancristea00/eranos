// Every feature of the library in one story: a group and its details, members with contact details and accounts,
// categories, an expense for every way of dividing one, another currency, edits and refusals, members who come and go,
// the summary, a new group currency, settling up, saving and loading, the values on their own, and the group shared
// between tasks. Every section checks its key numbers. The port of the Rust example `grand_tour`, which prints the same
// report.

#include "eranos.h"
#include "report.h"

#include <pthread.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/// The members of the trip.
typedef struct People {
    /// Who gave every detail.
    Id ana;
    /// Who edits his details.
    Id bob;
    /// Eva's mother.
    Id carol;
    /// Who flies home early.
    Id dan;
    /// Carol's daughter.
    Id eva;
    /// Who was added by mistake.
    Id filip;
} People;

/// The categories the group adds to the built-in one.
typedef struct Categories {
    /// Food.
    Id food;
    /// Lodging.
    Id lodging;
    /// Transport, renamed Getting around.
    Id transport;
} Categories;

/// A row of the table of the group's details: a detail and its value.
typedef struct DetailRow {
    /// What the detail is.
    Line detail;
    /// Its value.
    Line value;
} DetailRow;

/// A table of the group's details, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *detailTable(size_t const numbersFrom) { return tableCreate(2, (char const *const[]){"Detail", "Value"}, numbersFrom); }

/// Adds @p row to @p table, a table of the group's details, its cells in the order of the table's headers.
static void addDetailRow(Table *const table, DetailRow const *const row) { tableAdd(table, (Line const[]){row->detail, row->value}); }

/// A row of the table of members: a member's name, contact details, and accounts.
typedef struct MemberRow {
    /// Their first and last name.
    Line name;
    /// Their email, or nothing.
    Line email;
    /// Their phone number, or nothing.
    Line phone;
    /// Their accounts, one per line.
    Line accounts;
} MemberRow;

/// A table of members, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *memberTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Member", "Email", "Phone", "Accounts"}, numbersFrom);
}

/// Adds @p row to @p table, a table of members, its cells in the order of the table's headers.
static void addMemberRow(Table *const table, MemberRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->email, row->phone, row->accounts});
}

/// A row of the table of checks: a text, what it was checked as, and what validation made of it.
typedef struct CheckRow {
    /// The text checked, quoted.
    Line input;
    /// What it was checked as.
    Line kind;
    /// The value it makes, or its refusal.
    Line result;
} CheckRow;

/// A table of checks, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *checkTable(size_t const numbersFrom) {
    return tableCreate(3, (char const *const[]){"Input", "Checked as", "Result"}, numbersFrom);
}

/// Adds @p row to @p table, a table of checks, its cells in the order of the table's headers.
static void addCheckRow(Table *const table, CheckRow const *const row) { tableAdd(table, (Line const[]){row->input, row->kind, row->result}); }

/// A row of the table of refusals: a change and the message the library refused it with.
typedef struct RefusalRow {
    /// The change.
    Line change;
    /// The library's message.
    Line error;
} RefusalRow;

/// A table of refusals, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *refusalTable(size_t const numbersFrom) {
    return tableCreate(2, (char const *const[]){"Change", "Refused with"}, numbersFrom);
}

/// Adds @p row to @p table, a table of refusals, its cells in the order of the table's headers.
static void addRefusalRow(Table *const table, RefusalRow const *const row) { tableAdd(table, (Line const[]){row->change, row->error}); }

/// A row of the table of shares: an expense, how it divides, and every involved member's payment and consumption.
typedef struct ShareRow {
    /// The expense's name.
    Line expense;
    /// How it divides, in a few words.
    Line how;
    /// The involved members, one per line.
    Line members;
    /// What each member paid, one per line, or nothing for one who paid nothing.
    Line paid;
    /// What each member consumed, one per line.
    Line consumed;
} ShareRow;

/// A table of shares, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *shareTable(size_t const numbersFrom) {
    return tableCreate(5, (char const *const[]){"Expense", "Split", "Member", "Paid", "Consumed"}, numbersFrom);
}

/// Adds @p row to @p table, a table of shares, its cells in the order of the table's headers.
static void addShareRow(Table *const table, ShareRow const *const row) {
    tableAdd(table, (Line const[]){row->expense, row->how, row->members, row->paid, row->consumed});
}

/// A row of a receipt: a line, who shared it, its quantity, its unit price, and its total.
typedef struct ItemRow {
    /// What was ordered.
    Line name;
    /// Who shared it.
    Line sharedBy;
    /// How many were ordered.
    Line quantity;
    /// The price of one.
    Line unitPrice;
    /// The quantity times the unit price.
    Line lineTotal;
} ItemRow;

/// A table of a receipt's lines, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *itemTable(size_t const numbersFrom) {
    return tableCreate(5, (char const *const[]){"Item", "Shared by", "Quantity", "Unit price", "Line total"}, numbersFrom);
}

/// Adds @p row to @p table, a table of a receipt's lines, its cells in the order of the table's headers.
static void addItemRow(Table *const table, ItemRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->sharedBy, row->quantity, row->unitPrice, row->lineTotal});
}

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

/// A row of the table of edits: an entry and what happened to it.
typedef struct EditRow {
    /// The entry.
    Line entry;
    /// What happened to it.
    Line edit;
} EditRow;

/// A table of edits, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *editTable(size_t const numbersFrom) { return tableCreate(2, (char const *const[]){"Entry", "Edit"}, numbersFrom); }

/// Adds @p row to @p table, a table of edits, its cells in the order of the table's headers.
static void addEditRow(Table *const table, EditRow const *const row) { tableAdd(table, (Line const[]){row->entry, row->edit}); }

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

/// A row of the table of statuses: whether a member is active or archived, and their balance.
typedef struct StatusRow {
    /// Their first name.
    Line name;
    /// Whether they are active or archived.
    Line status;
    /// Their balance.
    Line balance;
} StatusRow;

/// A table of statuses, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *statusTable(size_t const numbersFrom) {
    return tableCreate(3, (char const *const[]){"Member", "Status", "Balance"}, numbersFrom);
}

/// Adds @p row to @p table, a table of statuses, its cells in the order of the table's headers.
static void addStatusRow(Table *const table, StatusRow const *const row) { tableAdd(table, (Line const[]){row->name, row->status, row->balance}); }

/// What a table says of a member's status: archived or active, as @p archived says, when @p held says that the group holds them.
[[nodiscard]] static char const *statusOf(bool const held, bool const archived) {
    if (!held) {
        return "not in the group";
    }
    return archived ? "archived" : "active";
}

/// A row of the table of categories: a category, how many expenses it holds, and their total.
typedef struct CategoryRow {
    /// Its name.
    Line name;
    /// How many expenses it holds.
    Line expenses;
    /// What its expenses came to.
    Line total;
} CategoryRow;

/// A table of categories, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *categoryTable(size_t const numbersFrom) {
    return tableCreate(3, (char const *const[]){"Category", "Expenses", "Total"}, numbersFrom);
}

/// Adds @p row to @p table, a table of categories, its cells in the order of the table's headers.
static void addCategoryRow(Table *const table, CategoryRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->expenses, row->total});
}

/// A row of the table of each member's consumption per category.
typedef struct MemberCategoryRow {
    /// Their first name.
    Line name;
    /// What they consumed of Food.
    Line food;
    /// What they consumed of Lodging.
    Line lodging;
    /// What they consumed of Getting around.
    Line transport;
    /// What they consumed of Uncategorised.
    Line uncategorised;
} MemberCategoryRow;

/// A table of each member's consumption per category, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *memberCategoryTable(size_t const numbersFrom) {
    return tableCreate(5, (char const *const[]){"Member", "Food", "Lodging", "Getting around", "Uncategorised"}, numbersFrom);
}

/// Adds @p row to @p table, a table of each member's consumption per category, its cells in the order of the table's
/// headers.
static void addMemberCategoryRow(Table *const table, MemberCategoryRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->food, row->lodging, row->transport, row->uncategorised});
}

/// A row of the table of strategies: a strategy, the strategy whose plan it chose, and how many payments that plan makes.
typedef struct StrategyRow {
    /// Its name.
    Line strategy;
    /// The name of the strategy whose plan it chose.
    Line computedBy;
    /// How many payments the plan makes.
    Line payments;
} StrategyRow;

/// A table of strategies, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *strategyTable(size_t const numbersFrom) {
    return tableCreate(3, (char const *const[]){"Strategy", "Computed by", "Payments"}, numbersFrom);
}

/// Adds @p row to @p table, a table of strategies, its cells in the order of the table's headers.
static void addStrategyRow(Table *const table, StrategyRow const *const row) {
    tableAdd(table, (Line const[]){row->strategy, row->computedBy, row->payments});
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

/// A row of the table of loads: a change to a saved group, and the message its load is refused with.
typedef struct LoadRow {
    /// The change to the file.
    Line change;
    /// The library's message.
    Line message;
} LoadRow;

/// A table of loads, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *loadTable(size_t const numbersFrom) {
    return tableCreate(2, (char const *const[]){"Change to the file", "Message"}, numbersFrom);
}

/// Adds @p row to @p table, a table of loads, its cells in the order of the table's headers.
static void addLoadRow(Table *const table, LoadRow const *const row) { tableAdd(table, (Line const[]){row->change, row->message}); }

/// A row of the table of currencies: a currency's code, name, numeric code, and decimal places.
typedef struct CurrencyRow {
    /// Its ISO 4217 code.
    Line code;
    /// Its name.
    Line name;
    /// Its ISO 4217 number.
    Line numericCode;
    /// How many decimal places its minor unit has.
    Line decimalPlaces;
} CurrencyRow;

/// A table of currencies, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *currencyTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Code", "Name", "Numeric code", "Decimal places"}, numbersFrom);
}

/// Adds @p row to @p table, a table of currencies, its cells in the order of the table's headers.
static void addCurrencyRow(Table *const table, CurrencyRow const *const row) {
    tableAdd(table, (Line const[]){row->code, row->name, row->numericCode, row->decimalPlaces});
}

/// A row of the table of values: an input and what the library makes of it.
typedef struct ValueRow {
    /// The input.
    Line input;
    /// What the library makes of it, or its refusal.
    Line result;
} ValueRow;

/// A table of values, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *valueTable(size_t const numbersFrom) { return tableCreate(2, (char const *const[]){"Input", "Result"}, numbersFrom); }

/// Adds @p row to @p table, a table of values, its cells in the order of the table's headers.
static void addValueRow(Table *const table, ValueRow const *const row) { tableAdd(table, (Line const[]){row->input, row->result}); }

/// The amount @p text of euros.
[[nodiscard]] static EranosMoney *eur(char const *const text) { return amountOf(text, "EUR"); }

/// The amount @p text of lei.
[[nodiscard]] static EranosMoney *ron(char const *const text) { return amountOf(text, "RON"); }

/// A date of the trip, in October 2026.
[[nodiscard]] static EranosCalendarDate october(int32_t const day) { return (EranosCalendarDate){.year = 2026, .month = 10, .day = day}; }

/// The text that @p value holds, or the message of @p error when the call that wrote it was refused; destroys @p error.
[[nodiscard]] static Line valueOr(EranosError *const error, EranosText const *const value) {
    return nullptr == error ? textLine(value) : messageLine(error);
}

/// The currency of @p group, as its code.
[[nodiscard]] static Line currencyOf(EranosExpenseGroup const *const group) { return currencyLine(eranosExpenseGroupCurrency(group)); }

/// Zero in the currency of @p group, as the library writes it.
[[nodiscard]] static Line zeroOf(EranosExpenseGroup const *const group) {
    EranosCurrencyDetails *const currency = eranosExpenseGroupCurrency(group);
    EranosMoney *const zero = eranosMoneyCreate(0, currency);
    eranosCurrencyDetailsDestroy(currency);
    return moneyLine(zero);
}

/// What @p group spent, as the library writes it.
[[nodiscard]] static Line totalSpent(EranosExpenseGroup const *const group) {
    EranosSummary *const summary = eranosExpenseGroupSummary(group);
    Line const line = moneyLine(eranosSummaryTotalSpent(summary));
    eranosSummaryDestroy(summary);
    return line;
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

/// The amount of @p cost, as entered; destroys @p cost.
[[nodiscard]] static Line costAmount(EranosCost *const cost) {
    Line const line = moneyLine(eranosCostAmount(cost));
    eranosCostDestroy(cost);
    return line;
}

/// The identifier of the built-in category, which every group has.
[[nodiscard]] static Id builtInCategory(void) {
    EranosText *const text = eranosTextCreate();
    eranosIdentifierUncategorised(text);
    Id const id = idOf(text);
    eranosTextDestroy(text);
    return id;
}

/// Whether @p left and @p right are the same amount.
[[nodiscard]] static bool sameAmount(EranosMoney *const left, EranosMoney *const right) {
    Line const leftCurrency = currencyLine(eranosMoneyCurrency(left));
    Line const rightCurrency = currencyLine(eranosMoneyCurrency(right));
    bool const same = 0 == strcmp(leftCurrency.text, rightCurrency.text) && eranosMoneyMinorUnits(left) == eranosMoneyMinorUnits(right);
    eranosMoneyDestroy(right);
    eranosMoneyDestroy(left);
    return same;
}

/// Whether @p money is @p minorUnits minor units; destroys @p money, which may be nullptr.
[[nodiscard]] static bool isMinorUnits(EranosMoney *const money, int64_t const minorUnits) {
    bool const same = nullptr != money && minorUnits == eranosMoneyMinorUnits(money);
    eranosMoneyDestroy(money);
    return same;
}

/// Whether @p balance is of @p kind, and of @p minorUnits when it has an amount; destroys @p balance.
[[nodiscard]] static bool isBalance(EranosBalance *const balance, EranosBalanceKind const kind, int64_t const minorUnits) {
    EranosMoney *const amount = eranosBalanceAmount(balance);
    bool const same = kind == eranosBalanceKind(balance) && (nullptr == amount || minorUnits == eranosMoneyMinorUnits(amount));
    eranosMoneyDestroy(amount);
    eranosBalanceDestroy(balance);
    return same;
}

/// Whether the totals of every member in @p left and @p right are the same.
[[nodiscard]] static bool sameMembers(EranosSummary const *const left, EranosSummary const *const right) {
    EranosMemberSummaryList *const leftMembers = eranosSummaryMembers(left);
    EranosMemberSummaryList *const rightMembers = eranosSummaryMembers(right);
    EranosText *const leftText = eranosTextCreate();
    EranosText *const rightText = eranosTextCreate();
    bool same = eranosMemberSummaryListLength(leftMembers) == eranosMemberSummaryListLength(rightMembers);
    for (size_t index = 0; same && eranosMemberSummaryListLength(leftMembers) > index; ++index) {
        EranosMemberSummary *const leftTotals = eranosMemberSummaryListGet(leftMembers, index);
        EranosMemberSummary *const rightTotals = eranosMemberSummaryListGet(rightMembers, index);
        eranosMemberSummaryMember(leftTotals, leftText);
        eranosMemberSummaryMember(rightTotals, rightText);
        same = 0 == strcmp(eranosTextData(leftText), eranosTextData(rightText)) &&
               sameAmount(eranosMemberSummaryPaid(leftTotals), eranosMemberSummaryPaid(rightTotals)) &&
               sameAmount(eranosMemberSummaryShare(leftTotals), eranosMemberSummaryShare(rightTotals)) &&
               sameAmount(eranosMemberSummarySent(leftTotals), eranosMemberSummarySent(rightTotals)) &&
               sameAmount(eranosMemberSummaryReceived(leftTotals), eranosMemberSummaryReceived(rightTotals)) &&
               sameAmount(eranosMemberSummaryNet(leftTotals), eranosMemberSummaryNet(rightTotals));
        eranosMemberSummaryDestroy(rightTotals);
        eranosMemberSummaryDestroy(leftTotals);
    }
    eranosTextDestroy(rightText);
    eranosTextDestroy(leftText);
    eranosMemberSummaryListDestroy(rightMembers);
    eranosMemberSummaryListDestroy(leftMembers);
    return same;
}

/// Whether the category totals in @p left and @p right are the same.
[[nodiscard]] static bool sameCategories(EranosCategoryTotalList const *const left, EranosCategoryTotalList const *const right) {
    EranosText *const leftText = eranosTextCreate();
    EranosText *const rightText = eranosTextCreate();
    bool same = eranosCategoryTotalListLength(left) == eranosCategoryTotalListLength(right);
    for (size_t index = 0; same && eranosCategoryTotalListLength(left) > index; ++index) {
        EranosCategoryTotal *const leftTotal = eranosCategoryTotalListGet(left, index);
        EranosCategoryTotal *const rightTotal = eranosCategoryTotalListGet(right, index);
        eranosCategoryTotalCategory(leftTotal, leftText);
        eranosCategoryTotalCategory(rightTotal, rightText);
        same = 0 == strcmp(eranosTextData(leftText), eranosTextData(rightText)) &&
               sameAmount(eranosCategoryTotalTotal(leftTotal), eranosCategoryTotalTotal(rightTotal));
        eranosCategoryTotalDestroy(rightTotal);
        eranosCategoryTotalDestroy(leftTotal);
    }
    eranosTextDestroy(rightText);
    eranosTextDestroy(leftText);
    return same;
}

/// Whether @p left and @p right hold the same totals: the total spent, every member's, and every category's.
[[nodiscard]] static bool sameSummaries(EranosSummary const *const left, EranosSummary const *const right) {
    EranosCategoryTotalList *const leftCategories = eranosSummaryCategoryTotals(left);
    EranosCategoryTotalList *const rightCategories = eranosSummaryCategoryTotals(right);
    bool const same = sameAmount(eranosSummaryTotalSpent(left), eranosSummaryTotalSpent(right)) && sameMembers(left, right) &&
                      sameCategories(leftCategories, rightCategories);
    eranosCategoryTotalListDestroy(rightCategories);
    eranosCategoryTotalListDestroy(leftCategories);
    return same;
}

/// The summary of @p group, which the caller destroys.
[[nodiscard]] static EranosSummary *summaryOf(EranosExpenseGroup const *const group) {
    EranosSummary *const summary = eranosExpenseGroupSummary(group);
    return summary;
}

/// The totals of @p member in @p group, which the caller destroys.
[[nodiscard]] static EranosMemberSummary *memberTotals(EranosExpenseGroup const *const group, Id const member) {
    EranosSummary *const summary = summaryOf(group);
    EranosMemberSummary *const totals = totalsOf(summary, member.text);
    check(nullptr != totals, "the member has totals");
    eranosSummaryDestroy(summary);
    return totals;
}

/// Prints what each member of @p group paid, consumed, sent, and received, and their balance.
static void printBalances(EranosExpenseGroup const *const group) {
    EranosSummary *const summary = summaryOf(group);
    Line const zero = zeroOf(group);
    EranosBalance *const settled = eranosBalanceCreateSettled();
    Line const settledLine = balanceLine(settled);
    EranosMemberList *const members = eranosExpenseGroupMembers(group);
    Table *const table = balanceTable(1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        eranosMemberId(member, text);
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        eranosMemberFirstName(member, text);
        // The summary lists the members with a total; every total of the others is zero.
        if (nullptr == found) {
            addBalanceRow(table, &(BalanceRow){
                                     .name = textLine(text),
                                     .paid = zero,
                                     .consumed = zero,
                                     .sent = zero,
                                     .received = zero,
                                     .balance = settledLine,
                                 });
        } else {
            addBalanceRow(table, &(BalanceRow){
                                     .name = textLine(text),
                                     .paid = moneyLine(eranosMemberSummaryPaid(found)),
                                     .consumed = moneyLine(eranosMemberSummaryShare(found)),
                                     .sent = moneyLine(eranosMemberSummarySent(found)),
                                     .received = moneyLine(eranosMemberSummaryReceived(found)),
                                     .balance = balanceLine(eranosMemberSummaryBalance(found)),
                                 });
        }
        eranosMemberSummaryDestroy(found);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
}

/// How many expenses of @p group are in @p category.
[[nodiscard]] static size_t expensesIn(EranosExpenseGroup const *const group, char const *const category) {
    EranosExpenseList *const expenses = eranosExpenseGroupExpenses(group);
    EranosText *const text = eranosTextCreate();
    size_t count = 0;
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        eranosExpenseCategory(expense, text);
        if (0 == strcmp(eranosTextData(text), category)) {
            ++count;
        }
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(text);
    eranosExpenseListDestroy(expenses);
    return count;
}

/// The name of the category @p id of @p group, or ? for one the group does not hold.
[[nodiscard]] static Line categoryName(EranosExpenseGroup const *const group, char const *const id) {
    EranosCategory *category = nullptr;
    must(eranosExpenseGroupCategory(group, id, &category));
    if (nullptr == category) {
        return lineOf("%s", "?");
    }
    EranosText *const text = eranosTextCreate();
    eranosCategoryName(category, text);
    Line const line = textLine(text);
    eranosTextDestroy(text);
    eranosCategoryDestroy(category);
    return line;
}

/// Prints what @p group spent per category, and on how many expenses.
static void printCategories(EranosExpenseGroup const *const group) {
    EranosSummary *const summary = summaryOf(group);
    EranosCategoryTotalList *const totals = eranosSummaryCategoryTotals(summary);
    Table *const table = categoryTable(1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosCategoryTotalListLength(totals) > index; ++index) {
        EranosCategoryTotal *const total = eranosCategoryTotalListGet(totals, index);
        eranosCategoryTotalCategory(total, text);
        addCategoryRow(table, &(CategoryRow){
                                  .name = categoryName(group, eranosTextData(text)),
                                  .expenses = lineOf("%zu", expensesIn(group, eranosTextData(text))),
                                  .total = moneyLine(eranosCategoryTotalTotal(total)),
                              });
        eranosCategoryTotalDestroy(total);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosCategoryTotalListDestroy(totals);
    eranosSummaryDestroy(summary);
}

/// Whether an entry in @p currency is shown when the entries in @p hidden are not; nullptr hides none.
[[nodiscard]] static bool isShown(char const *const hidden, EranosCost const *const cost) {
    if (nullptr == hidden) {
        return true;
    }
    EranosMoney *const amount = eranosCostAmount(cost);
    Line const currency = currencyLine(eranosMoneyCurrency(amount));
    bool const shown = 0 != strcmp(currency.text, hidden);
    eranosMoneyDestroy(amount);
    return shown;
}

/// Adds every expense of @p group but those in @p hidden to @p table, as entered and in the group currency.
static void addExpenseEntries(EranosExpenseGroup const *const group, char const *const hidden, Table *const table) {
    Line const zero = zeroOf(group);
    EranosExpenseList *const expenses = eranosExpenseGroupExpenses(group);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        EranosCost *const cost = eranosExpenseCost(expense);
        if (isShown(hidden, cost)) {
            eranosExpenseId(expense, text);
            EranosExpenseShares *shares = nullptr;
            must(eranosExpenseGroupExpenseShares(group, eranosTextData(text), &shares));
            Line const converted = nullptr == shares ? zero : moneyLine(eranosExpenseSharesTotal(shares));
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
                                   .converted = converted,
                               });
            eranosSplitDestroy(payers);
            eranosExpenseSharesDestroy(shares);
        }
        eranosCostDestroy(cost);
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(text);
    eranosExpenseListDestroy(expenses);
}

/// Adds every transfer of @p group but those in @p hidden to @p table, as entered and in the group currency.
static void addTransferEntries(EranosExpenseGroup const *const group, char const *const hidden, Table *const table) {
    Line const zero = zeroOf(group);
    EranosTransferList *const transfers = eranosExpenseGroupTransfers(group);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosTransferListLength(transfers) > index; ++index) {
        EranosTransfer *const transfer = eranosTransferListGet(transfers, index);
        EranosCost *const cost = eranosTransferCost(transfer);
        if (isShown(hidden, cost)) {
            eranosTransferId(transfer, text);
            EranosMoney *amount = nullptr;
            must(eranosExpenseGroupTransferAmount(group, eranosTextData(text), &amount));
            Line const converted = nullptr == amount ? zero : moneyLine(amount);
            eranosTransferTo(transfer, text);
            Line const to = firstName(group, eranosTextData(text));
            eranosTransferFrom(transfer, text);
            addEntryRow(table, &(EntryRow){
                                   .name = lineOf("Transfer to %s", to.text),
                                   .payer = firstName(group, eranosTextData(text)),
                                   .original = costAmount(eranosTransferCost(transfer)),
                                   .rate = rateLine(eranosTransferCost(transfer)),
                                   .converted = converted,
                               });
        }
        eranosCostDestroy(cost);
        eranosTransferDestroy(transfer);
    }
    eranosTextDestroy(text);
    eranosTransferListDestroy(transfers);
}

/// Prints every expense and transfer of @p group but those in @p hidden, as entered and in the group currency.
static void printEntries(EranosExpenseGroup const *const group, char const *const hidden) {
    Table *const table = entryTable(2);
    addExpenseEntries(group, hidden, table);
    addTransferEntries(group, hidden, table);
    tablePrint(table);
}

// 1. The group.

/// Creates the group, renames it, and prints its details.
[[nodiscard]] static EranosExpenseGroup *theGroup(void) {
    section("1. The group");
    EranosNewExpenseGroup *details = nullptr;
    must(eranosNewExpenseGroupCreate("Lisbon", "EUR", &details));
    must(eranosNewExpenseGroupSetDescription(details, "A week by the Atlantic"));
    must(eranosNewExpenseGroupSetLanguage(details, "pt-pt"));
    EranosExpenseGroup *group = nullptr;
    must(eranosExpenseGroupCreate(details, &group));
    eranosNewExpenseGroupDestroy(details);
    EranosError *const blank = eranosExpenseGroupSetName(group, "   ");
    check(isFailure(blank, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_EMPTY_NAME), "a blank name is refused");
    must(eranosExpenseGroupSetName(group, "Lisbon and Porto"));
    must(eranosExpenseGroupSetDescription(group, "A week by the Atlantic, with a day in Porto"));
    EranosText *const text = eranosTextCreate();
    eranosExpenseGroupName(group, text);
    Line const name = textLine(text);
    Line const currency = currencyOf(group);
    check(eranosExpenseGroupLanguage(group, text), "the group has a language");
    Line const language = textLine(text);
    (void)eranosExpenseGroupDescription(group, text);
    Line const description = textLine(text);
    eranosTextDestroy(text);
    Table *const table = detailTable(2);
    addDetailRow(table, &(DetailRow){.detail = lineOf("Name"), .value = name});
    addDetailRow(table, &(DetailRow){.detail = lineOf("Currency"), .value = currency});
    addDetailRow(table, &(DetailRow){.detail = lineOf("Language"), .value = language});
    addDetailRow(table, &(DetailRow){.detail = lineOf("Description"), .value = description});
    tablePrint(table);
    Line const message = messageLine(blank);
    printFormatted("The group started as Lisbon and was renamed when a day in Porto joined the plan.\n"
                   "Every total is kept in its currency, %s. The language is a BCP 47 tag the apps\n"
                   "show the group in, stored in canonical form: pt-pt became %s. A blank name is\n"
                   "refused:\n"
                   "  error: %s",
                   currency.text, language.text, message.text);

    check(0 == strcmp(name.text, "Lisbon and Porto"), "the group is renamed");
    check(0 == strcmp(language.text, "pt-PT"), "the language is canonical");
    return group;
}

// 2. Members.

/// An account with its label, its kind, and its details.
[[nodiscard]] static Line describe(EranosPaymentAccount const *const account) {
    EranosText *const name = eranosTextCreate();
    EranosText *const text = eranosTextCreate();
    eranosPaymentAccountName(account, name);
    EranosPaymentAccountDetails *const details = eranosPaymentAccountDetails(account);
    Line line = {};
    if (eranosPaymentAccountDetailsIban(details, text)) {
        line = lineOf("%s, IBAN %s", eranosTextData(name), eranosTextData(text));
    } else {
        (void)eranosPaymentAccountDetailsUsername(details, text);
        line = lineOf("%s, username %s", eranosTextData(name), eranosTextData(text));
    }
    eranosPaymentAccountDetailsDestroy(details);
    eranosTextDestroy(text);
    eranosTextDestroy(name);
    return line;
}

/// The label and the details of every account of @p member, one per line.
[[nodiscard]] static Line accountsOf(EranosMember const *const member) {
    EranosPaymentAccountList *const accounts = eranosMemberAccounts(member);
    Cell cell = {.lines = 0, .text = {}};
    for (size_t index = 0; eranosPaymentAccountListLength(accounts) > index; ++index) {
        EranosPaymentAccount *const account = eranosPaymentAccountListGet(accounts, index);
        Line const line = describe(account);
        cellAdd(&cell, line.text);
        eranosPaymentAccountDestroy(account);
    }
    eranosPaymentAccountListDestroy(accounts);
    return cell.text;
}

/// The first and the last name of @p member, the last when there is one.
[[nodiscard]] static Line fullName(EranosMember const *const member) {
    EranosText *const first = eranosTextCreate();
    EranosText *const last = eranosTextCreate();
    eranosMemberFirstName(member, first);
    Line const line = eranosMemberLastName(member, last) ? lineOf("%s %s", eranosTextData(first), eranosTextData(last)) : textLine(first);
    eranosTextDestroy(last);
    eranosTextDestroy(first);
    return line;
}

/// The email of @p member, or nothing when there is none.
[[nodiscard]] static Line emailOf(EranosMember const *const member) {
    EranosText *const text = eranosTextCreate();
    Line const line = eranosMemberEmail(member, text) ? textLine(text) : lineOf("%s", "");
    eranosTextDestroy(text);
    return line;
}

/// The phone number of @p member, grouped for reading, or nothing when there is none.
[[nodiscard]] static Line phoneOf(EranosMember const *const member) {
    EranosPhoneNumber *const phone = eranosMemberPhone(member);
    Line line = {};
    if (nullptr != phone) {
        EranosText *const international = eranosTextCreate();
        eranosPhoneNumberInternational(phone, international);
        line = textLine(international);
        eranosTextDestroy(international);
    }
    eranosPhoneNumberDestroy(phone);
    return line;
}

/// Prints every member of @p group with their contact and account details.
static void printMembers(EranosExpenseGroup const *const group) {
    EranosMemberList *const members = eranosExpenseGroupMembers(group);
    Table *const table = memberTable(4);
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        addMemberRow(table, &(MemberRow){
                                .name = fullName(member),
                                .email = emailOf(member),
                                .phone = phoneOf(member),
                                .accounts = accountsOf(member),
                            });
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosMemberListDestroy(members);
}

/// "valid: " and @p value, or "refused: " and the message of @p error; destroys @p error.
[[nodiscard]] static Line outcomeOf(EranosError *const error, char const *const value) {
    bool const valid = nullptr == error;
    Line const message = messageLine(error);
    return valid ? lineOf("valid: %s", value) : lineOf("refused: %s", message.text);
}

/// Adds the row of the IBAN @p input to @p table: its electronic form and its country, or its refusal.
static void addIban(Table *const table, char const *const input) {
    EranosIban *iban = nullptr;
    EranosError *const refused = eranosIbanParse(input, &iban);
    Line value = {};
    if (nullptr != iban) {
        EranosText *const text = eranosTextCreate();
        EranosText *const country = eranosTextCreate();
        eranosIbanIban(iban, text);
        eranosIbanCountryCode(iban, country);
        value = lineOf("%s, country %s", eranosTextData(text), eranosTextData(country));
        eranosTextDestroy(country);
        eranosTextDestroy(text);
    }
    addCheckRow(table, &(CheckRow){.input = lineOf("\"%s\"", input), .kind = lineOf("IBAN"), .result = outcomeOf(refused, value.text)});
    eranosIbanDestroy(iban);
}

/// Adds the row of the phone number @p input to @p table: its form for reading and its stored form, or its refusal.
static void addPhone(Table *const table, char const *const input) {
    EranosPhoneNumber *phone = nullptr;
    EranosError *const refused = eranosPhoneNumberParse(input, &phone);
    Line value = {};
    if (nullptr != phone) {
        EranosText *const international = eranosTextCreate();
        EranosText *const stored = eranosTextCreate();
        eranosPhoneNumberInternational(phone, international);
        eranosPhoneNumberE164(phone, stored);
        value = lineOf("%s, stored as %s", eranosTextData(international), eranosTextData(stored));
        eranosTextDestroy(stored);
        eranosTextDestroy(international);
    }
    addCheckRow(table, &(CheckRow){.input = lineOf("\"%s\"", input), .kind = lineOf("phone"), .result = outcomeOf(refused, value.text)});
    eranosPhoneNumberDestroy(phone);
}

/// Prints what validation makes of IBANs, an email, phone numbers, and blank required text.
static void validation(EranosExpenseGroup *const group) {
    printLine("IBANs are checked against the SWIFT registry and their mod-97 check digits,\n"
              "emails against RFC 5321 and 5322, and phone numbers against their country's\n"
              "numbering plan. Required text must not be blank:");
    Table *const table = checkTable(3);
    addIban(table, "ro49 aaaa 1b31 0075 9384 0000");
    addIban(table, "RO49 AAAA 1B31 0075 9384 0001");
    EranosText *const text = eranosTextCreate();
    EranosError *const email = eranosEmailParse("ana..popescu@example.com", text);
    addCheckRow(table, &(CheckRow){
                           .input = lineOf("\"ana..popescu@example.com\""),
                           .kind = lineOf("email"),
                           .result = outcomeOf(email, eranosTextData(text)),
                       });
    addPhone(table, "+351 912 345 678");
    addPhone(table, "+40 612 345 678");
    EranosError *const username = eranosUsernameParse("   ", text);
    addCheckRow(table, &(CheckRow){.input = lineOf("\"   \""), .kind = lineOf("username"), .result = outcomeOf(username, eranosTextData(text))});
    EranosMember *member = nullptr;
    must(eranosMemberCreate("  ", &member));
    EranosError *const firstName = eranosExpenseGroupAddMember(group, member, text);
    addCheckRow(table, &(CheckRow){.input = lineOf("\"  \""), .kind = lineOf("first name"), .result = outcomeOf(firstName, eranosTextData(text))});
    eranosMemberDestroy(member);
    eranosTextDestroy(text);
    tablePrint(table);
}

/// Adds Ana, with every detail, to @p group.
[[nodiscard]] static Id addAna(EranosExpenseGroup *const group) {
    EranosMember *member = nullptr;
    must(eranosMemberCreate("Ana", &member));
    EranosPaymentAccountDetails *iban = nullptr;
    must(eranosPaymentAccountDetailsCreateIban("RO49 AAAA 1B31 0075 9384 0000", &iban));
    EranosPaymentAccountDetails *username = nullptr;
    must(eranosPaymentAccountDetailsCreateUsername("@ana", &username));
    EranosPaymentAccount *bank = nullptr;
    must(eranosPaymentAccountCreate("ING", iban, &bank));
    EranosPaymentAccount *card = nullptr;
    must(eranosPaymentAccountCreate("Revolut", username, &card));
    EranosPaymentAccountList *const accounts = eranosPaymentAccountListCreate();
    eranosPaymentAccountListPush(accounts, bank);
    eranosPaymentAccountListPush(accounts, card);
    eranosMemberSetAccounts(member, accounts);
    must(eranosMemberSetEmail(member, "ana@example.com"));
    must(eranosMemberSetLastName(member, "Popescu"));
    must(eranosMemberSetPhone(member, "+40 712 345 678"));
    EranosText *const text = eranosTextCreate();
    must(eranosExpenseGroupAddMember(group, member, text));
    Id const ana = idOf(text);
    eranosTextDestroy(text);
    eranosPaymentAccountListDestroy(accounts);
    eranosPaymentAccountDestroy(card);
    eranosPaymentAccountDestroy(bank);
    eranosPaymentAccountDetailsDestroy(username);
    eranosPaymentAccountDetailsDestroy(iban);
    eranosMemberDestroy(member);
    return ana;
}

/// Adds the members of the trip to @p group, edits Bob, and prints them and what validation accepts.
[[nodiscard]] static People members(EranosExpenseGroup *const group) {
    section("2. Members");
    Id const ana = addAna(group);
    Id const bob = addMember(group, "Bob");
    Id const carol = addMember(group, "Carol");
    Id const dan = addMember(group, "Dan");
    Id const eva = addMember(group, "Eva");
    Id const filip = addMember(group, "Filip");
    // An edit starts from a copy of the member, which keeps its identifier.
    EranosMember *stored = nullptr;
    must(eranosExpenseGroupMember(group, bob.text, &stored));
    check(nullptr != stored, "Bob is a member");
    must(eranosMemberSetEmail(stored, "bob@example.com"));
    must(eranosMemberSetLastName(stored, "Ionescu"));
    EranosMember *before = nullptr;
    must(eranosExpenseGroupReplaceMember(group, stored, &before));
    printMembers(group);
    printFormatted("Only a first name is required. Ana gave every detail; Bob added his last name and\n"
                   "email later, through an edit that kept his identifier:\n"
                   "  %s\n"
                   "Eva is Carol's daughter, and Filip was added by mistake.",
                   bob.text);
    validation(group);

    EranosText *const text = eranosTextCreate();
    check(!eranosMemberLastName(before, text), "Bob had no last name");
    EranosMember *edited = nullptr;
    must(eranosExpenseGroupMember(group, bob.text, &edited));
    check(eranosMemberLastName(edited, text) && 0 == strcmp(eranosTextData(text), "Ionescu"), "Bob has his last name");
    EranosMember *first = nullptr;
    must(eranosExpenseGroupMember(group, ana.text, &first));
    EranosPaymentAccountList *const accounts = eranosMemberAccounts(first);
    check(2 == eranosPaymentAccountListLength(accounts), "Ana has two accounts");
    eranosPaymentAccountListDestroy(accounts);
    eranosMemberDestroy(first);
    eranosMemberDestroy(edited);
    eranosTextDestroy(text);
    eranosMemberDestroy(before);
    eranosMemberDestroy(stored);
    return (People){.ana = ana, .bob = bob, .carol = carol, .dan = dan, .eva = eva, .filip = filip};
}

// 3. Categories.

/// Adds the category @p name to @p group, under the identifier @p id, or a generated one when @p id is nullptr.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the identifier, then the name, in the order the library's constructor reads them.
[[nodiscard]] static Id addCategory(EranosExpenseGroup *const group, char const *const id, char const *const name) {
    EranosCategory *category = nullptr;
    EranosText *const text = eranosTextCreate();
    if (nullptr == id) {
        must(eranosCategoryCreate(name, &category));
    } else {
        must(eranosIdentifierParse(id, text));
        must(eranosCategoryCreateWithId(eranosTextData(text), name, &category));
    }
    must(eranosExpenseGroupAddCategory(group, category, text));
    Id const added = idOf(text);
    eranosTextDestroy(text);
    eranosCategoryDestroy(category);
    return added;
}

/// What @p group refuses the category @p name with.
[[nodiscard]] static EranosError *categoryRefusal(EranosExpenseGroup *const group, char const *const name) {
    EranosCategory *category = nullptr;
    must(eranosCategoryCreate(name, &category));
    EranosText *const text = eranosTextCreate();
    EranosError *const refusal = eranosExpenseGroupAddCategory(group, category, text);
    eranosTextDestroy(text);
    eranosCategoryDestroy(category);
    return refusal;
}

/// What @p group does when @p category is renamed @p name: nullptr when it renames it, or its refusal.
[[nodiscard]] static EranosError *renamed(EranosExpenseGroup *const group, Id const category, char const *const name) {
    EranosCategory *stored = nullptr;
    must(eranosExpenseGroupCategory(group, category.text, &stored));
    check(nullptr != stored, "the group holds the category");
    must(eranosCategorySetName(stored, name));
    EranosCategory *previous = nullptr;
    EranosError *const refusal = eranosExpenseGroupReplaceCategory(group, stored, &previous);
    eranosCategoryDestroy(previous);
    eranosCategoryDestroy(stored);
    return refusal;
}

/// The names of every category of @p group, in identifier order, joined by commas.
[[nodiscard]] static Line categoryNames(EranosExpenseGroup const *const group) {
    EranosCategoryList *const categories = eranosExpenseGroupCategories(group);
    EranosText *const text = eranosTextCreate();
    Line line = {};
    size_t length = 0;
    for (size_t index = 0; eranosCategoryListLength(categories) > index; ++index) {
        EranosCategory *const category = eranosCategoryListGet(categories, index);
        eranosCategoryName(category, text);
        Line const name = lineOf("%s%s", 0 == index ? "" : ", ", eranosTextData(text));
        check(LINE_SIZE > length + strlen(name.text), "the names fit a line");
        memcpy(line.text + length, name.text, strlen(name.text) + 1);
        length += strlen(name.text);
        eranosCategoryDestroy(category);
    }
    eranosTextDestroy(text);
    eranosCategoryListDestroy(categories);
    return line;
}

/// Whether the categories of @p group, in identifier order, are the @p count identifiers of @p expected.
[[nodiscard]] static bool categoriesAre(EranosExpenseGroup const *const group, Id const *const expected, size_t const count) {
    EranosCategoryList *const categories = eranosExpenseGroupCategories(group);
    EranosText *const text = eranosTextCreate();
    bool same = count == eranosCategoryListLength(categories);
    for (size_t index = 0; same && count > index; ++index) {
        EranosCategory *const category = eranosCategoryListGet(categories, index);
        eranosCategoryId(category, text);
        same = 0 == strcmp(eranosTextData(text), expected[index].text);
        eranosCategoryDestroy(category);
    }
    eranosTextDestroy(text);
    eranosCategoryListDestroy(categories);
    return same;
}

/// Whether @p error names @p category as the one with the name it refused.
[[nodiscard]] static bool namesExisting(EranosError const *const error, Id const category) {
    EranosText *const text = eranosTextCreate();
    bool const same = eranosErrorExisting(error, text) && 0 == strcmp(eranosTextData(text), category.text);
    eranosTextDestroy(text);
    return same;
}

/// Adds Food, Lodging, and Transport to @p group, renames Transport, and shows what renaming refuses.
[[nodiscard]] static Categories addCategories(EranosExpenseGroup *const group) {
    section("3. Categories");
    Id const food = addCategory(group, nullptr, "Food");
    // An app that keeps its categories elsewhere gives them its own keys.
    Id const lodging = addCategory(group, "lodging", "Lodging");
    Id const transport = addCategory(group, nullptr, "Transport");
    EranosError *const homonym = categoryRefusal(group, "FOOD");
    must(renamed(group, transport, "Getting around"));
    Id const uncategorised = builtInCategory();
    EranosError *const builtIn = renamed(group, uncategorised, "Other");
    Line const names = categoryNames(group);
    printFormatted("Every group has the built-in Uncategorised; this one adds Food, Lodging, under the\n"
                   "identifier lodging that the app gives it, and Transport, then renames Transport, which\n"
                   "keeps its identifier. In identifier order, where lodging follows the generated ones:\n"
                   "  %s\n"
                   "Names are compared caselessly, and the built-in category keeps its name:",
                   names.text);
    check(isFailure(homonym, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME) && namesExisting(homonym, food),
          "a caseless homonym is refused");
    check(isFailure(builtIn, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_BUILT_IN_CATEGORY),
          "the built-in category keeps its name");
    Table *const table = refusalTable(2);
    addRefusalRow(table, &(RefusalRow){.change = lineOf("Add a category named FOOD"), .error = messageLine(homonym)});
    addRefusalRow(table, &(RefusalRow){.change = lineOf("Rename Uncategorised to Other"), .error = messageLine(builtIn)});
    tablePrint(table);

    Line const name = categoryName(group, transport.text);
    check(0 == strcmp(name.text, "Getting around"), "Transport is renamed");
    check(0 == strcmp(lodging.text, "lodging"), "Lodging is under the identifier the app gave it");
    Id const order[] = {uncategorised, food, transport, lodging};
    check(categoriesAre(group, order, sizeof order / sizeof order[0]), "lodging follows the generated identifiers");
    return (Categories){.food = food, .lodging = lodging, .transport = transport};
}

// 4. Expenses, one for every way of dividing.

/// How a whole expense's amount divides, in a word or two.
[[nodiscard]] static char const *divided(EranosSplit const *const split) {
    switch (eranosSplitKind(split)) {
    case ERANOS_SPLIT_KIND_EQUAL:
        return "equally";
    case ERANOS_SPLIT_KIND_PARTS:
        return "by parts";
    case ERANOS_SPLIT_KIND_PERCENTAGES:
        return "by percentage";
    case ERANOS_SPLIT_KIND_AMOUNTS:
        return "by amounts";
    }
    unreachable();
}

/// The tip of @p expense, as ", 10% tip" or ", 5.00 EUR tip", or nothing when it comes to zero.
[[nodiscard]] static Line tipOf(EranosExpense const *const expense) {
    EranosMoney *amount = nullptr;
    must(eranosExpenseTipAmount(expense, &amount));
    EranosTipRule *const tip = eranosExpenseTip(expense);
    Line line = {};
    if (0 != eranosMoneyMinorUnits(amount)) {
        EranosTipValue *const value = eranosTipRuleValue(tip);
        int32_t basisPoints = 0;
        if (eranosTipValueBasisPoints(value, &basisPoints)) {
            EranosText *const text = eranosTextCreate();
            eranosPercentageFormat(basisPoints, text);
            line = lineOf(", %s tip", eranosTextData(text));
            eranosTextDestroy(text);
        } else {
            Line const fixed = moneyLine(eranosTipValueAmount(value));
            line = lineOf(", %s tip", fixed.text);
        }
        eranosTipValueDestroy(value);
    }
    eranosTipRuleDestroy(tip);
    eranosMoneyDestroy(amount);
    return line;
}

/// How @p expense divides, in a few words.
[[nodiscard]] static Line how(EranosExpense const *const expense) {
    EranosExpenseSplit *const split = eranosExpenseSplit(expense);
    EranosExpenseItemList *const items = eranosExpenseSplitItems(split);
    EranosSplit *const whole = eranosExpenseSplitSplit(split);
    Line const kind = nullptr == items ? lineOf("%s", divided(whole)) : lineOf("%zu items", eranosExpenseItemListLength(items));
    EranosSplit *const payers = eranosExpensePayers(expense);
    size_t const count = splitMembers(payers).count;
    Line const paid = 1 == count ? lineOf("%s", "") : lineOf(", %zu payers", count);
    Line const tip = tipOf(expense);
    eranosSplitDestroy(payers);
    eranosSplitDestroy(whole);
    eranosExpenseItemListDestroy(items);
    eranosExpenseSplitDestroy(split);
    return lineOf("%s%s%s", kind.text, paid.text, tip.text);
}

/// Adds to @p table the row of the expense @p id: every involved member's payment and consumption.
static void addShares(EranosExpenseGroup const *const group, Table *const table, Id const id) {
    EranosExpense *expense = nullptr;
    must(eranosExpenseGroupExpense(group, id.text, &expense));
    EranosExpenseShares *shares = nullptr;
    must(eranosExpenseGroupExpenseShares(group, id.text, &shares));
    check(nullptr != expense && nullptr != shares, "the group holds the expense");
    EranosMemberShareList *const members = eranosExpenseSharesMembers(shares);
    EranosText *const text = eranosTextCreate();
    Cell names = {.lines = 0, .text = {}};
    Cell paid = {.lines = 0, .text = {}};
    Cell consumed = {.lines = 0, .text = {}};
    for (size_t index = 0; eranosMemberShareListLength(members) > index; ++index) {
        EranosMemberShare *const share = eranosMemberShareListGet(members, index);
        eranosMemberShareMember(share, text);
        Line const name = firstName(group, eranosTextData(text));
        cellAdd(&names, name.text);
        EranosMoney *const payment = eranosMemberSharePaid(share);
        bool const paidNothing = 0 == eranosMoneyMinorUnits(payment);
        Line const payments = moneyLine(payment);
        cellAdd(&paid, paidNothing ? "" : payments.text);
        Line const consumption = moneyLine(eranosMemberShareShare(share));
        cellAdd(&consumed, consumption.text);
        eranosMemberShareDestroy(share);
    }
    eranosExpenseName(expense, text);
    addShareRow(table, &(ShareRow){
                           .expense = textLine(text),
                           .how = how(expense),
                           .members = names.text,
                           .paid = paid.text,
                           .consumed = consumed.text,
                       });
    eranosTextDestroy(text);
    eranosMemberShareListDestroy(members);
    eranosExpenseSharesDestroy(shares);
    eranosExpenseDestroy(expense);
}

/// Prints the receipt of the expense @p id of @p group: who shared each line, its quantity, its unit price, and its total.
static void printItems(EranosExpenseGroup const *const group, Id const id) {
    EranosExpense *expense = nullptr;
    must(eranosExpenseGroupExpense(group, id.text, &expense));
    check(nullptr != expense, "the group holds the expense");
    EranosExpenseSplit *const split = eranosExpenseSplit(expense);
    EranosExpenseItemList *const items = eranosExpenseSplitItems(split);
    check(nullptr != items, "the expense is itemised");
    Table *const table = itemTable(2);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosExpenseItemListLength(items) > index; ++index) {
        EranosExpenseItem *const item = eranosExpenseItemListGet(items, index);
        EranosSplit *const itemSplit = eranosExpenseItemSplit(item);
        Members const sharedBy = splitMembers(itemSplit);
        EranosMoney *const unitPrice = eranosExpenseItemUnitPrice(item);
        int32_t const quantity = eranosExpenseItemQuantity(item);
        EranosCurrencyDetails *const currency = eranosMoneyCurrency(unitPrice);
        EranosMoney *const total = eranosMoneyCreate(eranosMoneyMinorUnits(unitPrice) * quantity, currency);
        eranosCurrencyDetailsDestroy(currency);
        eranosExpenseItemName(item, text);
        addItemRow(table, &(ItemRow){
                              .name = textLine(text),
                              .sharedBy = names(group, &sharedBy),
                              .quantity = lineOf("%ld", (long)quantity),
                              .unitPrice = moneyLine(unitPrice),
                              .lineTotal = moneyLine(total),
                          });
        eranosSplitDestroy(itemSplit);
        eranosExpenseItemDestroy(item);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosExpenseItemListDestroy(items);
    eranosExpenseSplitDestroy(split);
    eranosExpenseDestroy(expense);
}

/// Adds the line @p name of a receipt to @p items: @p quantity of @p unitPrice euros, which @p split shares; destroys
/// @p split.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the name, then the unit price, in the order the library's constructor reads them.
static void addItem(EranosExpenseItemList *const items, char const *const name, char const *const unitPrice, EranosSplit *const split,
                    int32_t const quantity) {
    EranosMoney *const price = eur(unitPrice);
    EranosExpenseItem *item = nullptr;
    must(eranosExpenseItemCreate(name, price, split, &item));
    eranosExpenseItemSetQuantity(item, quantity);
    eranosExpenseItemListPush(items, item);
    eranosExpenseItemDestroy(item);
    eranosMoneyDestroy(price);
    eranosSplitDestroy(split);
}

/// The receipt of the fado night: every line with its own split.
[[nodiscard]] static EranosExpenseItemList *fadoItems(People const *const people) {
    EranosExpenseItemList *const items = eranosExpenseItemListCreate();
    Members const wine = {.count = 3, .ids = {people->ana, people->bob, people->dan}};
    addItem(items, "Vinho verde", "12.00", equalSplit(&wine), 2);
    addItem(items, "Bacalhau", "18.00", soleSplit(people->carol), 1);
    Share const tarts[] = {
        {.member = people->ana, .value = "1"}, {.member = people->bob, .value = "1"}, {.member = people->carol, .value = "1"},
        {.member = people->dan, .value = "1"}, {.member = people->eva, .value = "2"},
    };
    addItem(items, "Custard tarts", "1.50", partsSplit(tarts, 5), 6);
    addItem(items, "Juice", "3.00", soleSplit(people->eva), 1);
    return items;
}

/// Gives @p expense a tip of @p value, shared by @p distribution; destroys @p value.
static void tipWith(EranosExpense *const expense, EranosTipValue *const value, EranosTipDistribution const distribution) {
    EranosTipRule *const tip = eranosTipRuleCreate(value, distribution);
    eranosExpenseSetTip(expense, tip);
    eranosTipRuleDestroy(tip);
    eranosTipValueDestroy(value);
}

/// The percentage @p text as a tip's value.
[[nodiscard]] static EranosTipValue *percentageTip(char const *const text) {
    int32_t basisPoints = 0;
    must(eranosPercentageParse(text, &basisPoints));
    return eranosTipValueCreatePercentage(basisPoints);
}

/// The fado night: itemised, paid by two, at a time and a place, with a tip of 10 %.
[[nodiscard]] static EranosExpense *fadoNight(People const *const people, Categories const *const categories) {
    Share const cards[] = {
        {.member = people->carol, .value = "39.40"},
        {.member = people->dan, .value = "20.00"},
    };
    EranosCost *const cost = localCost(eur("54.00"));
    EranosSplit *const payers = amountsSplit("EUR", cards, 2);
    EranosExpenseItemList *const items = fadoItems(people);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateItemised(items);
    EranosExpense *expense = nullptr;
    must(eranosExpenseCreate("Fado night", cost, payers, split, october(4), &expense));
    eranosExpenseSplitDestroy(split);
    eranosExpenseItemListDestroy(items);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    must(eranosExpenseSetCategory(expense, categories->food.text));
    must(eranosExpenseSetDescription(expense, "Tasca do Chico"));
    eranosExpenseSetTime(expense, (EranosTimeOfDay){.hour = 22, .minute = 15});
    tipWith(expense, percentageTip("10"), ERANOS_TIP_DISTRIBUTION_PROPORTIONAL);
    return expense;
}

/// @p expense in @p category; the expense comes back for the caller to add.
[[nodiscard]] static EranosExpense *inCategory(EranosExpense *const expense, Id const category) {
    must(eranosExpenseSetCategory(expense, category.text));
    return expense;
}

/// The seven expenses, one for every way of dividing one, added to @p group in order.
typedef struct Expenses {
    /// The apartment, by percentage.
    Id apartment;
    /// The fado night, item by item with a tip.
    Id fado;
    /// The groceries, equally.
    Id groceries;
    /// The museum, with two payers.
    Id museum;
    /// The seafood dinner, by parts.
    Id seafood;
    /// The tuk-tuk tour, with a fixed tip shared equally.
    Id tour;
    /// The train to Porto, by exact amounts.
    Id train;
} Expenses;

/// Adds one expense for every way of dividing one to @p group, in order.
[[nodiscard]] static Expenses everyWay(EranosExpenseGroup *const group, People const *const people, Categories const *const categories) {
    Members const thirds = {.count = 3, .ids = {people->ana, people->bob, people->carol}};
    Id const groceries =
        addExpense(group, inCategory(expenseOf("Groceries", localCost(eur("10.00")), soleSplit(people->ana), equalSplit(&thirds), october(1)),
                                     categories->food));
    Share const rooms[] = {
        {.member = people->ana, .value = "25"},
        {.member = people->bob, .value = "25"},
        {.member = people->carol, .value = "35"},
        {.member = people->dan, .value = "15"},
    };
    Id const apartment =
        addExpense(group, inCategory(expenseOf("Apartment", localCost(eur("800.00")), soleSplit(people->ana), percentagesSplit(rooms, 4), october(1)),
                                     categories->lodging));
    Share const portions[] = {
        {.member = people->ana, .value = "1"}, {.member = people->bob, .value = "1"},   {.member = people->carol, .value = "1"},
        {.member = people->dan, .value = "1"}, {.member = people->eva, .value = "0.5"},
    };
    Id const seafood = addExpense(
        group, inCategory(expenseOf("Seafood dinner", localCost(eur("90.00")), soleSplit(people->carol), partsSplit(portions, 5), october(2)),
                          categories->food));
    Share const tickets[] = {
        {.member = people->ana, .value = "24.50"},
        {.member = people->bob, .value = "24.50"},
        {.member = people->carol, .value = "24.50"},
        {.member = people->eva, .value = "15.00"},
    };
    Id const train = addExpense(
        group, inCategory(expenseOf("Train to Porto", localCost(eur("88.50")), soleSplit(people->bob), amountsSplit("EUR", tickets, 4), october(3)),
                          categories->transport));
    Members const cards = {.count = 2, .ids = {people->ana, people->bob}};
    Members const visitors = {.count = 4, .ids = {people->ana, people->bob, people->carol, people->dan}};
    Id const museum = addExpense(group, expenseOf("Museum", localCost(eur("60.00")), equalSplit(&cards), equalSplit(&visitors), october(3)));
    Id const fado = addExpense(group, fadoNight(people, categories));
    Members const everyone = {.count = 5, .ids = {people->ana, people->bob, people->carol, people->dan, people->eva}};
    EranosExpense *const ride = inCategory(
        expenseOf("Tuk-tuk tour", localCost(eur("45.00")), soleSplit(people->dan), equalSplit(&everyone), october(5)), categories->transport);
    EranosMoney *const fixed = eur("5.00");
    tipWith(ride, eranosTipValueCreateFixed(fixed), ERANOS_TIP_DISTRIBUTION_EQUAL);
    eranosMoneyDestroy(fixed);
    Id const tour = addExpense(group, ride);
    return (Expenses){
        .apartment = apartment,
        .fado = fado,
        .groceries = groceries,
        .museum = museum,
        .seafood = seafood,
        .tour = tour,
        .train = train,
    };
}

/// The share of @p member in the expense @p expense of @p group, which the caller destroys.
[[nodiscard]] static EranosMemberShare *shareOf(EranosExpenseGroup const *const group, Id const expense, Id const member) {
    EranosExpenseShares *shares = nullptr;
    must(eranosExpenseGroupExpenseShares(group, expense.text, &shares));
    check(nullptr != shares, "the group holds the expense");
    EranosMemberShareList *const members = eranosExpenseSharesMembers(shares);
    EranosText *const text = eranosTextCreate();
    EranosMemberShare *found = nullptr;
    for (size_t index = 0; nullptr == found && eranosMemberShareListLength(members) > index; ++index) {
        EranosMemberShare *const share = eranosMemberShareListGet(members, index);
        eranosMemberShareMember(share, text);
        if (0 == strcmp(eranosTextData(text), member.text)) {
            found = share;
        } else {
            eranosMemberShareDestroy(share);
        }
    }
    eranosTextDestroy(text);
    eranosMemberShareListDestroy(members);
    eranosExpenseSharesDestroy(shares);
    check(nullptr != found, "the member shares the expense");
    return found;
}

/// What @p member consumed of the expense @p expense of @p group, in minor units.
[[nodiscard]] static int64_t consumedOf(EranosExpenseGroup const *const group, Id const expense, Id const member) {
    EranosMemberShare *const share = shareOf(group, expense, member);
    EranosMoney *const amount = eranosMemberShareShare(share);
    int64_t const minorUnits = eranosMoneyMinorUnits(amount);
    eranosMoneyDestroy(amount);
    eranosMemberShareDestroy(share);
    return minorUnits;
}

/// Prints how the fado night's tip was shared, and its receipt.
static void printFado(EranosExpenseGroup const *const group, Id const id) {
    EranosExpense *fado = nullptr;
    must(eranosExpenseGroupExpense(group, id.text, &fado));
    check(nullptr != fado, "the fado night was added");
    EranosText *const text = eranosTextCreate();
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    Line when = {};
    if (eranosExpenseTime(fado, &time)) {
        eranosTimeOfDayFormat(time, text);
        when = textLine(text);
    }
    eranosCalendarDateFormat(eranosExpenseDate(fado), text);
    Line const date = textLine(text);
    Line const place = eranosExpenseDescription(fado, text) ? textLine(text) : lineOf("%s", "");
    EranosTipRule *const tip = eranosExpenseTip(fado);
    EranosTipValue *const value = eranosTipRuleValue(tip);
    int32_t basisPoints = 0;
    Line rule = {};
    if (eranosTipValueBasisPoints(value, &basisPoints)) {
        eranosPercentageFormat(basisPoints, text);
        rule = textLine(text);
    } else {
        rule = moneyLine(eranosTipValueAmount(value));
    }
    char const *const shared =
        ERANOS_TIP_DISTRIBUTION_EQUAL == eranosTipRuleDistribution(tip) ? "equally" : "in proportion to what each member ordered";
    EranosMoney *amount = nullptr;
    must(eranosExpenseTipAmount(fado, &amount));
    Line const tipAmount = moneyLine(amount);
    printFormatted("The groceries' 10.00 EUR did not divide into thirds, so one member consumed the\n"
                   "leftover cent; the expense's identifier picks which, so it moves from expense to\n"
                   "expense. The fado night, at %s on %s at %s, was itemised; its\n"
                   "%s tip of %s was shared %s:",
                   when.text, date.text, place.text, rule.text, tipAmount.text, shared);
    printItems(group, id);
    printLine("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.");
    eranosTipValueDestroy(value);
    eranosTipRuleDestroy(tip);
    eranosTextDestroy(text);
    eranosExpenseDestroy(fado);
}

/// Checks the key numbers of the seven expenses of @p group.
static void checkExpenses(EranosExpenseGroup const *const group, People const *const people, Expenses const *const expenses) {
    int64_t const thirds[] = {
        consumedOf(group, expenses->groceries, people->ana),
        consumedOf(group, expenses->groceries, people->bob),
        consumedOf(group, expenses->groceries, people->carol),
    };
    size_t const leastParts = (size_t)(333 == thirds[0]) + (size_t)(333 == thirds[1]) + (size_t)(333 == thirds[2]);
    check(2 == leastParts && 1'000 == thirds[0] + thirds[1] + thirds[2], "the groceries divide into 3.33, 3.33, and 3.34");
    check(28'000 == consumedOf(group, expenses->apartment, people->carol), "Carol's 35 % of the apartment is 280.00 EUR");
    check(1'000 == consumedOf(group, expenses->seafood, people->eva), "Eva's half part of the dinner is 10.00 EUR");
    check(1'500 == consumedOf(group, expenses->train, people->eva), "Eva's ticket is 15.00 EUR");
    EranosMemberShare *const bob = shareOf(group, expenses->museum, people->bob);
    check(isMinorUnits(eranosMemberSharePaid(bob), 3'000), "Bob paid 30.00 EUR of the museum");
    eranosMemberShareDestroy(bob);
    check(1'500 == consumedOf(group, expenses->museum, people->dan), "Dan's quarter of the museum is 15.00 EUR");
    EranosExpense *museum = nullptr;
    must(eranosExpenseGroupExpense(group, expenses->museum.text, &museum));
    EranosTextList *involved = nullptr;
    must(eranosExpenseParticipants(museum, &involved));
    check(4 == eranosTextListLength(involved), "the museum names four members");
    eranosTextListDestroy(involved);
    eranosExpenseDestroy(museum);
    EranosExpense *fado = nullptr;
    must(eranosExpenseGroupExpense(group, expenses->fado.text, &fado));
    EranosMoney *total = nullptr;
    must(eranosExpenseTotal(fado, &total));
    check(isMinorUnits(total, 5'940), "the fado night comes to 59.40 EUR");
    EranosMoney *tip = nullptr;
    must(eranosExpenseTipAmount(fado, &tip));
    check(isMinorUnits(tip, 540), "its tip is 5.40 EUR");
    eranosExpenseDestroy(fado);
    check(2'145 == consumedOf(group, expenses->fado, people->carol), "Carol's part of the fado night is 21.45 EUR");
    check(660 == consumedOf(group, expenses->fado, people->eva), "Eva's part of the fado night is 6.60 EUR");
    check(1'000 == consumedOf(group, expenses->tour, people->eva), "Eva's part of the tour is 10.00 EUR");
}

/// Adds the seven expenses to @p group and prints how each divides; returns the seafood dinner's identifier.
[[nodiscard]] static Id expenses(EranosExpenseGroup *const group, People const *const people, Categories const *const categories) {
    section("4. Expenses, one for every way of dividing");
    Expenses const added = everyWay(group, people, categories);
    printLine("Seven expenses cover every way of dividing one: equally, by percentage, by parts\n"
              "(Eva counts as half), by exact amounts, with two payers, item by item, and with a\n"
              "tip. Every member's consumption adds up to the expense's total, to the cent:");
    Table *const table = shareTable(3);
    Id const ids[] = {added.groceries, added.apartment, added.seafood, added.train, added.museum, added.fado, added.tour};
    for (size_t index = 0; 7 > index; ++index) {
        addShares(group, table, ids[index]);
    }
    tablePrint(table);
    printFado(group, added.fado);
    checkExpenses(group, people, &added);
    return added.seafood;
}

// 5. Another currency.

/// Adds the airport parking in RON and Bob's repayment in cash, and shows how they convert.
static void anotherCurrency(EranosExpenseGroup *const group, People const *const people, Categories const *const categories) {
    section("5. Another currency");
    Members const both = {.count = 2, .ids = {people->ana, people->bob}};
    Id const parking = addExpense(
        group, inCategory(expenseOf("Airport parking", convertedCost(ron("250.00"), "0.2"), soleSplit(people->ana), equalSplit(&both), october(6)),
                          categories->transport));
    EranosTransfer *const repayment = transferOf(people->bob, people->ana, convertedCost(ron("100.00"), "0.2"), october(6));
    must(eranosTransferSetNote(repayment, "Cash for the parking"));
    eranosTransferSetTime(repayment, (EranosTimeOfDay){.hour = 7, .minute = 40});
    Id const cash = addTransfer(group, repayment);
    printLine("Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in\n"
              "cash. An entry in another currency carries the rate into the group currency, how\n"
              "many EUR one RON is worth, and is converted once, when it is entered:");
    printEntries(group, "EUR");
    EranosTransfer *transfer = nullptr;
    must(eranosExpenseGroupTransfer(group, cash.text, &transfer));
    check(nullptr != transfer, "the cash was recorded");
    EranosText *const text = eranosTextCreate();
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    Line when = {};
    if (eranosTransferTime(transfer, &time)) {
        eranosTimeOfDayFormat(time, text);
        when = textLine(text);
    }
    Line const note = eranosTransferNote(transfer, text) ? textLine(text) : lineOf("%s", "");
    EranosMoney *counted = nullptr;
    must(eranosExpenseGroupTransferAmount(group, cash.text, &counted));
    Line const counts = nullptr == counted ? zeroOf(group) : moneyLine(counted);
    EranosMoney *const parked = ron("250.00");
    EranosMoney *converted = nullptr;
    must(eranosExchangeRateConvert("0.2011", parked, "EUR", &converted));
    eranosMoneyDestroy(parked);
    int64_t const convertedUnits = eranosMoneyMinorUnits(converted);
    Line const becomes = moneyLine(converted);
    printFormatted("Bob's transfer, at %s (\"%s\"), counts as %s.\n"
                   "A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,\n"
                   "which becomes %s.",
                   when.text, note.text, counts.text, becomes.text);

    check(5'028 == convertedUnits, "the conversion rounds to 50.28 EUR");
    EranosExpenseShares *shares = nullptr;
    must(eranosExpenseGroupExpenseShares(group, parking.text, &shares));
    check(nullptr != shares && isMinorUnits(eranosExpenseSharesTotal(shares), 5'000), "the parking counts as 50.00 EUR");
    EranosMoney *amount = nullptr;
    must(eranosExpenseGroupTransferAmount(group, cash.text, &amount));
    check(isMinorUnits(amount, 2'000), "the cash counts as 20.00 EUR");
    EranosCost *const cost = eranosTransferCost(transfer);
    check(ERANOS_COST_KIND_CONVERTED == eranosCostKind(cost), "the cash was converted");
    eranosCostDestroy(cost);
    eranosExpenseSharesDestroy(shares);
    eranosTextDestroy(text);
    eranosTransferDestroy(transfer);
}

// 6. Edits.

/// The coffee that Bob entered twice.
[[nodiscard]] static EranosExpense *coffeeOf(People const *const people, Categories const *const categories) {
    Members const drinkers = {.count = 3, .ids = {people->ana, people->bob, people->carol}};
    return inCategory(expenseOf("Coffee", localCost(eur("6.00")), soleSplit(people->bob), equalSplit(&drinkers), october(5)), categories->food);
}

/// The dinner's receipt said 99.00: replaces its amount and gives its place; returns the edit.
[[nodiscard]] static EditRow correctDinner(EranosExpenseGroup *const group, Id const seafood) {
    EranosExpense *dinner = nullptr;
    must(eranosExpenseGroupExpense(group, seafood.text, &dinner));
    check(nullptr != dinner, "the dinner was added");
    EranosCost *const cost = localCost(eur("99.00"));
    eranosExpenseSetCost(dinner, cost);
    eranosCostDestroy(cost);
    must(eranosExpenseSetDescription(dinner, "Cervejaria Ramiro"));
    EranosExpense *previous = nullptr;
    must(eranosExpenseGroupReplaceExpense(group, dinner, &previous));
    EranosExpense *corrected = nullptr;
    must(eranosExpenseGroupExpense(group, seafood.text, &corrected));
    EranosText *const text = eranosTextCreate();
    eranosExpenseName(previous, text);
    Line const entry = textLine(text);
    Line const place = eranosExpenseDescription(corrected, text) ? textLine(text) : lineOf("%s", "");
    Line const before = costAmount(eranosExpenseCost(previous));
    Line const after = costAmount(eranosExpenseCost(corrected));
    EditRow const edit = {.entry = entry, .edit = lineOf("%s became %s, at %s", before.text, after.text, place.text)};
    EranosCost *const previousCost = eranosExpenseCost(previous);
    check(isMinorUnits(eranosCostAmount(previousCost), 9'000), "the dinner was 90.00 EUR");
    eranosCostDestroy(previousCost);
    eranosTextDestroy(text);
    eranosExpenseDestroy(corrected);
    eranosExpenseDestroy(previous);
    eranosExpenseDestroy(dinner);
    return edit;
}

/// Bob entered the coffee twice: removes the copy; returns the edit.
[[nodiscard]] static EditRow removeCopy(EranosExpenseGroup *const group, People const *const people, Categories const *const categories) {
    (void)addExpense(group, coffeeOf(people, categories));
    Id const copy = addExpense(group, coffeeOf(people, categories));
    EranosExpense *removed = nullptr;
    must(eranosExpenseGroupRemoveExpense(group, copy.text, &removed));
    EranosText *const text = eranosTextCreate();
    eranosExpenseName(removed, text);
    Line const entry = textLine(text);
    Line const amount = costAmount(eranosExpenseCost(removed));
    EditRow const edit = {.entry = entry, .edit = lineOf("entered twice; the copy of %s was removed", amount.text)};
    EranosExpense *gone = nullptr;
    must(eranosExpenseGroupExpense(group, copy.text, &gone));
    check(nullptr == gone, "the copy is gone");
    eranosTextDestroy(text);
    eranosExpenseDestroy(removed);
    return edit;
}

/// Dan's part of the apartment was 120.00, not 100.00: replaces his transfer's amount and notes what it was for; returns
/// the edit.
[[nodiscard]] static EditRow correctTransfer(EranosExpenseGroup *const group, People const *const people) {
    Id const first = addTransfer(group, transferOf(people->dan, people->ana, localCost(eur("100.00")), october(5)));
    EranosTransfer *stored = nullptr;
    must(eranosExpenseGroupTransfer(group, first.text, &stored));
    check(nullptr != stored, "the transfer was added");
    EranosCost *const cost = localCost(eur("120.00"));
    eranosTransferSetCost(stored, cost);
    eranosCostDestroy(cost);
    must(eranosTransferSetNote(stored, "Apartment"));
    EranosTransfer *replaced = nullptr;
    must(eranosExpenseGroupReplaceTransfer(group, stored, &replaced));
    EranosTransfer *apartment = nullptr;
    must(eranosExpenseGroupTransfer(group, first.text, &apartment));
    Line const dan = firstName(group, people->dan.text);
    Line const ana = firstName(group, people->ana.text);
    EranosText *const text = eranosTextCreate();
    Line const note = eranosTransferNote(apartment, text) ? textLine(text) : lineOf("%s", "");
    Line const before = costAmount(eranosTransferCost(replaced));
    Line const after = costAmount(eranosTransferCost(apartment));
    EditRow const edit = {
        .entry = lineOf("%s to %s", dan.text, ana.text),
        .edit = lineOf("%s became %s, noted %s", before.text, after.text, note.text),
    };
    EranosMoney *amount = nullptr;
    must(eranosExpenseGroupTransferAmount(group, first.text, &amount));
    check(isMinorUnits(amount, 12'000), "Dan's transfer is 120.00 EUR");
    eranosTextDestroy(text);
    eranosTransferDestroy(apartment);
    eranosTransferDestroy(replaced);
    eranosTransferDestroy(stored);
    return edit;
}

/// Carol sent 10.00 to the wrong member: removes the transfer; returns the edit.
[[nodiscard]] static EditRow undoMistake(EranosExpenseGroup *const group, People const *const people) {
    Id const mistake = addTransfer(group, transferOf(people->carol, people->bob, localCost(eur("10.00")), october(5)));
    EranosTransfer *undone = nullptr;
    must(eranosExpenseGroupRemoveTransfer(group, mistake.text, &undone));
    Line const carol = firstName(group, people->carol.text);
    Line const bob = firstName(group, people->bob.text);
    Line const amount = costAmount(eranosTransferCost(undone));
    EditRow const edit = {.entry = lineOf("%s to %s", carol.text, bob.text), .edit = lineOf("%s sent to the wrong member; removed", amount.text)};
    EranosTransfer *gone = nullptr;
    must(eranosExpenseGroupTransfer(group, mistake.text, &gone));
    check(nullptr == gone, "the mistake is gone");
    eranosTransferDestroy(undone);
    return edit;
}

/// Whether @p error is the currency @p currency, as the currency it names.
[[nodiscard]] static bool namesCurrency(EranosError const *const error, char const *const currency) {
    EranosCurrencyDetails *const named = eranosErrorCurrency(error);
    return nullptr != named && 0 == strcmp(currencyLine(named).text, currency);
}

/// Replaces an expense and a transfer and removes one of each, then shows that a change that does not fit is refused.
static void edits(EranosExpenseGroup *const group, People const *const people, Categories const *const categories, Id const seafood) {
    section("6. Edits");
    EditRow const dinner = correctDinner(group, seafood);
    EditRow const coffee = removeCopy(group, people, categories);
    EditRow const rent = correctTransfer(group, people);
    EditRow const mistake = undoMistake(group, people);
    printLine("Entries are values: an edit starts from a copy made with to_builder, keeps the\n"
              "identifier, and replaces the entry; a removal takes it out with its totals.");
    Table *const table = editTable(2);
    addEditRow(table, &dinner);
    addEditRow(table, &coffee);
    addEditRow(table, &rent);
    addEditRow(table, &mistake);
    tablePrint(table);
    printLine("The balances after the edits:");
    printBalances(group);

    EranosSummary *const before = summaryOf(group);
    Share const halves[] = {
        {.member = people->ana, .value = "50"},
        {.member = people->bob, .value = "49.99"},
    };
    EranosError *const unbalanced =
        expenseRefusal(group, expenseOf("Taxi", localCost(eur("30.00")), soleSplit(people->ana), percentagesSplit(halves, 2), october(6)));
    EranosTransfer *const toHerself = transferOf(people->ana, people->ana, localCost(eur("5.00")), october(6));
    EranosText *const text = eranosTextCreate();
    EranosError *const sameMember = eranosExpenseGroupAddTransfer(group, toHerself, text);
    eranosTransferDestroy(toHerself);
    EranosError *const unrated =
        expenseRefusal(group, expenseOf("Souvenirs", localCost(ron("40.00")), soleSplit(people->ana), soleSplit(people->ana), october(6)));
    printLine("A change that does not fit is refused, and the group stays as it was:");
    int64_t total = 0;
    check(isFailure(unbalanced, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_PERCENTAGES_DO_NOT_ADD_UP) &&
              eranosErrorTotalBasisPoints(unbalanced, &total) && 9'999 == total,
          "percentages that do not add up are refused");
    check(isFailure(sameMember, ERANOS_ERROR_KIND_TRANSFER, ERANOS_FAILURE_TRANSFER_SAME_MEMBER), "a transfer to oneself is refused");
    check(isFailure(unrated, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_MISSING_EXCHANGE_RATE) && namesCurrency(unrated, "RON"),
          "an amount in RON without a rate is refused");
    Table *const refusals = refusalTable(2);
    addRefusalRow(refusals, &(RefusalRow){.change = lineOf("A taxi split 50%% and 49.99%%"), .error = messageLine(unbalanced)});
    addRefusalRow(refusals, &(RefusalRow){.change = lineOf("Ana paying herself back"), .error = messageLine(sameMember)});
    addRefusalRow(refusals, &(RefusalRow){.change = lineOf("Souvenirs in RON, without a rate"), .error = messageLine(unrated)});
    tablePrint(refusals);
    printLine("The balances again, unchanged:");
    printBalances(group);

    EranosSummary *const after = summaryOf(group);
    check(sameSummaries(before, after), "the refusals change nothing");
    eranosSummaryDestroy(after);
    eranosTextDestroy(text);
    eranosSummaryDestroy(before);
}

// 7. Members come and go.

/// Removes Filip, archives Dan, shows what that refuses, and unarchives him.
static void comingsAndGoings(EranosExpenseGroup *const group, People const *const people) {
    section("7. Members come and go");
    EranosMember *removed = nullptr;
    must(eranosExpenseGroupRemoveMember(group, people->filip.text, &removed));
    must(eranosExpenseGroupArchiveMember(group, people->dan.text));
    Members const diners = {.count = 4, .ids = {people->ana, people->bob, people->carol, people->dan}};
    EranosError *const farewell =
        expenseRefusal(group, expenseOf("Farewell dinner", localCost(eur("40.00")), soleSplit(people->ana), equalSplit(&diners), october(6)));
    EranosMember *kept = nullptr;
    EranosError *const inUse = eranosExpenseGroupRemoveMember(group, people->dan.text, &kept);
    EranosText *const text = eranosTextCreate();
    eranosMemberFirstName(removed, text);
    Line const removedName = textLine(text);
    printFormatted("%s was never part of an entry, so removing him leaves no trace. Dan flew home\n"
                   "early and was archived: his history and his balance stay, but a new expense\n"
                   "cannot include him, and he cannot be removed while entries reference him.",
                   removedName.text);
    EranosSummary *const summary = summaryOf(group);
    EranosMemberList *const members = eranosExpenseGroupMembers(group);
    Table *const table = statusTable(2);
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        eranosMemberId(member, text);
        bool held = false;
        bool archived = false;
        must(eranosExpenseGroupIsArchived(group, eranosTextData(text), &held, &archived));
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        if (nullptr != found) {
            eranosMemberFirstName(member, text);
            addStatusRow(table, &(StatusRow){
                                    .name = textLine(text),
                                    .status = lineOf("%s", statusOf(held, archived)),
                                    .balance = balanceLine(eranosMemberSummaryBalance(found)),
                                });
        }
        eranosMemberSummaryDestroy(found);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
    bool const namesDan = eranosErrorMember(farewell, text) && 0 == strcmp(eranosTextData(text), people->dan.text);
    check(isFailure(farewell, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_ARCHIVED_MEMBER) && namesDan,
          "an archived member joins no expense");
    check(isFailure(inUse, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_MEMBER_IN_USE) && nullptr == kept,
          "a referenced member stays");
    Table *const refusals = refusalTable(2);
    addRefusalRow(refusals, &(RefusalRow){.change = lineOf("A farewell dinner with Dan"), .error = messageLine(farewell)});
    addRefusalRow(refusals, &(RefusalRow){.change = lineOf("Removing Dan"), .error = messageLine(inUse)});
    tablePrint(refusals);
    must(eranosExpenseGroupUnarchiveMember(group, people->dan.text));
    printLine("When Dan joins the next trip, unarchiving him makes him active again.");

    check(0 == strcmp(removedName.text, "Filip"), "Filip was removed");
    EranosMember *gone = nullptr;
    must(eranosExpenseGroupMember(group, people->filip.text, &gone));
    check(nullptr == gone, "Filip is gone");
    bool held = false;
    bool archived = true;
    must(eranosExpenseGroupIsArchived(group, people->dan.text, &held, &archived));
    check(held && !archived, "Dan is active again");
    eranosTextDestroy(text);
    eranosMemberDestroy(removed);
}

// 8. The summary.

/// The totals per category of @p member in @p summary, which the caller destroys, or nullptr when it lists none.
[[nodiscard]] static EranosCategoryTotalList *byCategory(EranosSummary const *const summary, char const *const member) {
    EranosMemberCategoryTotalsList *const list = eranosSummaryMemberCategoryTotals(summary);
    EranosText *const text = eranosTextCreate();
    EranosCategoryTotalList *found = nullptr;
    for (size_t index = 0; nullptr == found && eranosMemberCategoryTotalsListLength(list) > index; ++index) {
        EranosMemberCategoryTotals *const entry = eranosMemberCategoryTotalsListGet(list, index);
        eranosMemberCategoryTotalsMember(entry, text);
        if (0 == strcmp(eranosTextData(text), member)) {
            found = eranosMemberCategoryTotalsTotals(entry);
        }
        eranosMemberCategoryTotalsDestroy(entry);
    }
    eranosTextDestroy(text);
    eranosMemberCategoryTotalsListDestroy(list);
    return found;
}

/// The total of @p category among @p totals, which the caller destroys, or nullptr when they list none.
[[nodiscard]] static EranosMoney *categoryTotal(EranosCategoryTotalList const *const totals, char const *const category) {
    EranosText *const text = eranosTextCreate();
    EranosMoney *found = nullptr;
    for (size_t index = 0; nullptr != totals && nullptr == found && eranosCategoryTotalListLength(totals) > index; ++index) {
        EranosCategoryTotal *const total = eranosCategoryTotalListGet(totals, index);
        eranosCategoryTotalCategory(total, text);
        if (0 == strcmp(eranosTextData(text), category)) {
            found = eranosCategoryTotalTotal(total);
        }
        eranosCategoryTotalDestroy(total);
    }
    eranosTextDestroy(text);
    return found;
}

/// The total of @p category among @p totals, or @p zero when they list none.
[[nodiscard]] static Line categoryLine(EranosCategoryTotalList const *const totals, Id const category, Line const zero) {
    EranosMoney *const total = categoryTotal(totals, category.text);
    return nullptr == total ? zero : moneyLine(total);
}

/// Prints each member's consumption per category.
static void printMemberCategories(EranosExpenseGroup const *const group, Categories const *const categories) {
    Id const builtIn = builtInCategory();
    Line const zero = zeroOf(group);
    EranosSummary *const summary = summaryOf(group);
    EranosMemberList *const members = eranosExpenseGroupMembers(group);
    Table *const table = memberCategoryTable(1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        eranosMemberId(member, text);
        EranosCategoryTotalList *const own = byCategory(summary, eranosTextData(text));
        eranosMemberFirstName(member, text);
        addMemberCategoryRow(table, &(MemberCategoryRow){
                                        .name = textLine(text),
                                        .food = categoryLine(own, categories->food, zero),
                                        .lodging = categoryLine(own, categories->lodging, zero),
                                        .transport = categoryLine(own, categories->transport, zero),
                                        .uncategorised = categoryLine(own, builtIn, zero),
                                    });
        eranosCategoryTotalListDestroy(own);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
}

/// Whether every member's consumption per category adds up to their consumption, and every net to zero.
[[nodiscard]] static bool totalsAddUp(EranosSummary const *const summary) {
    EranosMemberSummaryList *const members = eranosSummaryMembers(summary);
    EranosText *const text = eranosTextCreate();
    bool addsUp = true;
    int64_t nets = 0;
    for (size_t index = 0; eranosMemberSummaryListLength(members) > index; ++index) {
        EranosMemberSummary *const totals = eranosMemberSummaryListGet(members, index);
        eranosMemberSummaryMember(totals, text);
        EranosCategoryTotalList *const own = byCategory(summary, eranosTextData(text));
        int64_t sum = 0;
        for (size_t category = 0; nullptr != own && eranosCategoryTotalListLength(own) > category; ++category) {
            EranosCategoryTotal *const total = eranosCategoryTotalListGet(own, category);
            EranosMoney *const amount = eranosCategoryTotalTotal(total);
            sum += eranosMoneyMinorUnits(amount);
            eranosMoneyDestroy(amount);
            eranosCategoryTotalDestroy(total);
        }
        EranosMoney *const share = eranosMemberSummaryShare(totals);
        addsUp = addsUp && sum == eranosMoneyMinorUnits(share);
        eranosMoneyDestroy(share);
        EranosMoney *const net = eranosMemberSummaryNet(totals);
        nets += eranosMoneyMinorUnits(net);
        eranosMoneyDestroy(net);
        eranosCategoryTotalListDestroy(own);
        eranosMemberSummaryDestroy(totals);
    }
    eranosTextDestroy(text);
    eranosMemberSummaryListDestroy(members);
    return addsUp && 0 == nets;
}

/// Prints the summary: every member's totals, the spending per category, and each member's consumption per category.
static void summary(EranosExpenseGroup const *const group, People const *const people, Categories const *const categories) {
    section("8. The summary");
    Line const spent = totalSpent(group);
    printFormatted("The group spent %s in all. For each member, what they paid, what they\n"
                   "consumed, the transfers they sent and received, and what is left:",
                   spent.text);
    printBalances(group);
    printLine("The spending per category:");
    printCategories(group);
    printLine("And each member's consumption per category:");
    printMemberCategories(group, categories);

    EranosSummary *const totals = summaryOf(group);
    check(isMinorUnits(eranosSummaryTotalSpent(totals), 122'290), "the group spent 1222.90 EUR");
    EranosMemberSummary *const ana = memberTotals(group, people->ana);
    check(isMinorUnits(eranosMemberSummaryPaid(ana), 89'000) && isMinorUnits(eranosMemberSummaryReceived(ana), 14'000),
          "Ana paid 890.00 EUR and received 140.00 EUR");
    EranosMemberSummary *const bob = memberTotals(group, people->bob);
    check(isMinorUnits(eranosMemberSummarySent(bob), 2'000), "Bob sent 20.00 EUR");
    EranosMemberSummary *const dan = memberTotals(group, people->dan);
    check(isBalance(eranosMemberSummaryBalance(dan), ERANOS_BALANCE_KIND_GETS_BACK, 1'255), "Dan gets back 12.55 EUR");
    EranosMemberSummary *const eva = memberTotals(group, people->eva);
    check(isBalance(eranosMemberSummaryBalance(eva), ERANOS_BALANCE_KIND_OWES, 4'260), "Eva owes 42.60 EUR");
    EranosCategoryTotalList *const categoryTotals = eranosSummaryCategoryTotals(totals);
    Id const builtIn = builtInCategory();
    check(isMinorUnits(categoryTotal(categoryTotals, categories->food.text), 17'440), "the food came to 174.40 EUR");
    check(isMinorUnits(categoryTotal(categoryTotals, categories->lodging.text), 80'000), "the lodging came to 800.00 EUR");
    check(isMinorUnits(categoryTotal(categoryTotals, categories->transport.text), 18'850), "getting around came to 188.50 EUR");
    check(isMinorUnits(categoryTotal(categoryTotals, builtIn.text), 6'000), "the rest came to 60.00 EUR");
    check(totalsAddUp(totals), "every member's totals add up");
    eranosCategoryTotalListDestroy(categoryTotals);
    eranosMemberSummaryDestroy(eva);
    eranosMemberSummaryDestroy(dan);
    eranosMemberSummaryDestroy(bob);
    eranosMemberSummaryDestroy(ana);
    eranosSummaryDestroy(totals);
}

// 9. Removing a category.

/// Whether an expense of @p group is in @p category.
[[nodiscard]] static bool anyIn(EranosExpenseGroup const *const group, Id const category) {
    EranosExpenseList *const expenses = eranosExpenseGroupExpenses(group);
    EranosText *const text = eranosTextCreate();
    bool found = false;
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        eranosExpenseCategory(expense, text);
        found = found || 0 == strcmp(eranosTextData(text), category.text);
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(text);
    eranosExpenseListDestroy(expenses);
    return found;
}

/// Removes Getting around, whose expenses move to Uncategorised, and shows that the built-in category stays.
static void removingACategory(EranosExpenseGroup *const group, Categories const *const categories) {
    section("9. Removing a category");
    EranosCategory *removed = nullptr;
    must(eranosExpenseGroupRemoveCategory(group, categories->transport.text, &removed));
    Id const uncategorised = builtInCategory();
    EranosCategory *kept = nullptr;
    EranosError *const builtIn = eranosExpenseGroupRemoveCategory(group, uncategorised.text, &kept);
    check(isFailure(builtIn, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_BUILT_IN_CATEGORY) && nullptr == kept,
          "the built-in category cannot be removed");
    EranosText *const text = eranosTextCreate();
    eranosCategoryName(removed, text);
    Line const message = messageLine(builtIn);
    printFormatted("Removing %s moves its expenses to Uncategorised and the category\n"
                   "totals with them; no balance changes. The built-in category cannot be removed:\n"
                   "  error: %s",
                   eranosTextData(text), message.text);
    printCategories(group);

    EranosCategory *gone = nullptr;
    must(eranosExpenseGroupCategory(group, categories->transport.text, &gone));
    check(nullptr == gone, "Getting around is gone");
    EranosSummary *const summary = summaryOf(group);
    EranosCategoryTotalList *const totals = eranosSummaryCategoryTotals(summary);
    check(isMinorUnits(categoryTotal(totals, uncategorised.text), 24'850), "Uncategorised comes to 248.50 EUR");
    check(!anyIn(group, categories->transport), "no expense is in Getting around");
    eranosCategoryTotalListDestroy(totals);
    eranosSummaryDestroy(summary);
    eranosTextDestroy(text);
    eranosCategoryDestroy(removed);
}

// 10. A new currency.

/// Moves the books of @p group to RON, which a missing rate first refuses.
static void newCurrency(EranosExpenseGroup *const group, People const *const people) {
    section("10. A new currency");
    EranosCurrencyRateList *const none = eranosCurrencyRateListCreate();
    EranosError *const missing = eranosExpenseGroupChangeCurrency(group, "RON", none);
    eranosCurrencyRateListDestroy(none);
    check(isFailure(missing, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_MISSING_EXCHANGE_RATE) && namesCurrency(missing, "EUR"),
          "a change without the rate from EUR is refused");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = nullptr;
    must(eranosCurrencyRateCreate("EUR", "5", &rate));
    eranosCurrencyRateListPush(rates, rate);
    eranosCurrencyRateDestroy(rate);
    must(eranosExpenseGroupChangeCurrency(group, "RON", rates));
    eranosCurrencyRateListDestroy(rates);
    Line const message = messageLine(missing);
    printFormatted("Home again, the group moves its books to RON. The change needs a rate into RON\n"
                   "for every other currency in use, here only EUR, or it is refused:\n"
                   "  error: %s\n"
                   "Entries in EUR take the new rate, and the RON parking and cash become local:",
                   message.text);
    printEntries(group, nullptr);
    Line const spent = totalSpent(group);
    printFormatted("Each entry keeps its currency and converts at the new rate. At 5 every share\n"
                   "converts exactly, so each balance is five times what it was. The trip cost\n"
                   "%s:",
                   spent.text);
    printBalances(group);

    Line const currency = currencyOf(group);
    check(0 == strcmp(currency.text, "RON"), "the group keeps its books in RON");
    EranosSummary *const summary = summaryOf(group);
    check(isMinorUnits(eranosSummaryTotalSpent(summary), 611'450), "the trip cost 6114.50 RON");
    eranosSummaryDestroy(summary);
    EranosMemberSummary *const dan = memberTotals(group, people->dan);
    check(isBalance(eranosMemberSummaryBalance(dan), ERANOS_BALANCE_KIND_GETS_BACK, 6'275), "Dan gets back 62.75 RON");
    EranosMemberSummary *const eva = memberTotals(group, people->eva);
    check(isBalance(eranosMemberSummaryBalance(eva), ERANOS_BALANCE_KIND_OWES, 21'300), "Eva owes 213.00 RON");
    eranosMemberSummaryDestroy(eva);
    eranosMemberSummaryDestroy(dan);
}

// 11. Settling up.

/// The name of @p strategy as the apps show it.
[[nodiscard]] static char const *strategyName(EranosSettlementStrategy const strategy) {
    switch (strategy) {
    case ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE:
        return "Adaptive";
    case ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS:
        return "MinimumTransfers";
    case ERANOS_SETTLEMENT_STRATEGY_GREEDY:
        return "Greedy";
    case ERANOS_SETTLEMENT_STRATEGY_DIRECT:
        return "Direct";
    }
    unreachable();
}

/// The payments of the plan that @p strategy makes for @p group, which the caller destroys.
[[nodiscard]] static EranosPaymentList *planOf(EranosExpenseGroup const *const group, EranosSettlementStrategy const strategy) {
    EranosSettlementPlan *settlement = nullptr;
    must(eranosExpenseGroupSettlement(group, strategy, &settlement));
    check(strategy == eranosSettlementPlanStrategy(settlement), "the plan is the strategy's");
    EranosPaymentList *const payments = eranosSettlementPlanPayments(settlement);
    eranosSettlementPlanDestroy(settlement);
    return payments;
}

/// Whether every member with totals in @p group is settled.
[[nodiscard]] static bool everyoneSettled(EranosExpenseGroup const *const group) {
    EranosSummary *const summary = summaryOf(group);
    EranosMemberSummaryList *const members = eranosSummaryMembers(summary);
    bool settled = true;
    for (size_t index = 0; eranosMemberSummaryListLength(members) > index; ++index) {
        EranosMemberSummary *const totals = eranosMemberSummaryListGet(members, index);
        settled = settled && isBalance(eranosMemberSummaryBalance(totals), ERANOS_BALANCE_KIND_SETTLED, 0);
        eranosMemberSummaryDestroy(totals);
    }
    eranosMemberSummaryListDestroy(members);
    eranosSummaryDestroy(summary);
    return settled;
}

/// Prints how many payments each strategy plans, then records the minimum plan as transfers.
static void settlingUp(EranosExpenseGroup *const group) {
    section("11. Settling up");
    constexpr EranosSettlementStrategy STRATEGIES[] = {
        ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE,
        ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS,
        ERANOS_SETTLEMENT_STRATEGY_GREEDY,
        ERANOS_SETTLEMENT_STRATEGY_DIRECT,
    };
    printLine("Each strategy turns the balances into payments: the direct one pays back every\n"
              "pair's own debts, greedy pays the largest amounts first, the minimum one finds\n"
              "the fewest payments, and the adaptive one finds the fewest while that is quick\n"
              "and greedy's beyond, naming the strategy whose plan it chose.");
    Table *const counts = strategyTable(2);
    for (size_t index = 0; 4 > index; ++index) {
        EranosSettlementPlan *settlement = nullptr;
        must(eranosExpenseGroupSettlement(group, STRATEGIES[index], &settlement));
        // The group's open balances are well within the adaptive strategy's limit, so it settles them exactly.
        EranosSettlementStrategy const computedBy =
            ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE == STRATEGIES[index] ? ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS : STRATEGIES[index];
        check(computedBy == eranosSettlementPlanStrategy(settlement), "the settlement names the strategy whose plan it is");
        EranosPaymentList *const payments = eranosSettlementPlanPayments(settlement);
        addStrategyRow(counts, &(StrategyRow){
                                   .strategy = lineOf("%s", strategyName(STRATEGIES[index])),
                                   .computedBy = lineOf("%s", strategyName(eranosSettlementPlanStrategy(settlement))),
                                   .payments = lineOf("%zu", eranosPaymentListLength(payments)),
                               });
        eranosPaymentListDestroy(payments);
        eranosSettlementPlanDestroy(settlement);
    }
    tablePrint(counts);
    EranosPaymentList *const plan = planOf(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS);
    printLine("The group follows the minimum plan:");
    Table *const table = paymentTable(2);
    EranosText *const from = eranosTextCreate();
    EranosText *const to = eranosTextCreate();
    for (size_t index = 0; eranosPaymentListLength(plan) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(plan, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        addPaymentRow(table, &(PaymentRow){
                                 .from = firstName(group, eranosTextData(from)),
                                 .to = firstName(group, eranosTextData(to)),
                                 .amount = moneyLine(eranosPaymentAmount(payment)),
                             });
        eranosPaymentDestroy(payment);
    }
    tablePrint(table);
    for (size_t index = 0; eranosPaymentListLength(plan) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(plan, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        (void)addTransfer(group, transferOf(idOf(from), idOf(to), localCost(eranosPaymentAmount(payment)), october(7)));
        eranosPaymentDestroy(payment);
    }
    printLine("Each payment, once made, is recorded as a transfer, and every balance is settled:");
    printBalances(group);

    EranosPaymentList *const remaining = planOf(group, ERANOS_SETTLEMENT_STRATEGY_GREEDY);
    check(0 == eranosPaymentListLength(remaining), "nothing is left to pay");
    check(4 == eranosPaymentListLength(plan), "the minimum plan takes four payments");
    check(everyoneSettled(group), "every balance is settled");
    eranosPaymentListDestroy(remaining);
    eranosTextDestroy(to);
    eranosTextDestroy(from);
    eranosPaymentListDestroy(plan);
}

// 12. Saving and loading.

/// Whether @p error names @p format as the format it cannot read.
[[nodiscard]] static bool namesFormat(EranosError const *const error, int64_t const format) {
    int64_t found = 0;
    return eranosErrorFormat(error, &found) && format == found;
}

/// Prints Ana's accounts as @p saved holds them, in clear text.
static void printAccounts(EranosExpenseGroup const *const group, Id const ana, char const *const saved) {
    EranosMember *member = nullptr;
    must(eranosExpenseGroupMember(group, ana.text, &member));
    check(nullptr != member, "Ana is a member");
    EranosPaymentAccountList *const accounts = eranosMemberAccounts(member);
    EranosText *const name = eranosTextCreate();
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosPaymentAccountListLength(accounts) > index; ++index) {
        EranosPaymentAccount *const account = eranosPaymentAccountListGet(accounts, index);
        eranosPaymentAccountName(account, name);
        EranosPaymentAccountDetails *const details = eranosPaymentAccountDetails(account);
        if (!eranosPaymentAccountDetailsIban(details, text)) {
            (void)eranosPaymentAccountDetailsUsername(details, text);
        }
        check(nullptr != strstr(saved, eranosTextData(text)), "the details are saved as they are");
        printFormatted("  %s: %s", eranosTextData(name), eranosTextData(text));
        eranosPaymentAccountDetailsDestroy(details);
        eranosPaymentAccountDestroy(account);
    }
    eranosTextDestroy(text);
    eranosTextDestroy(name);
    eranosPaymentAccountListDestroy(accounts);
    eranosMemberDestroy(member);
}

/// What loading @p saved, with its first @p pattern replaced by @p replacement, is refused with.
[[nodiscard]] static EranosError *changedRefusal(char const *const saved, char const *const pattern, char const *const replacement) {
    char *const changed = replaceFirst(saved, pattern, replacement);
    check(0 != strcmp(changed, saved), "the change changes the file");
    EranosExpenseGroup *loaded = nullptr;
    EranosError *const refusal = eranosExpenseGroupLoad(changed, &loaded);
    check(nullptr != refusal && nullptr == loaded, "the changed file is refused");
    free(changed);
    return refusal;
}

/// Saves @p group, loads it again, and shows that a changed file is refused with a typed error.
static void savingAndLoading(EranosExpenseGroup const *const group, People const *const people, Categories const *const categories) {
    section("12. Saving and loading");
    EranosText *const saved = eranosTextCreate();
    eranosExpenseGroupSave(group, saved);
    printFormatted("The group saves to %zu bytes of JSON: its details and entries, never the totals.\n"
                   "Ana's accounts are saved in clear text, as the library holds them:",
                   eranosTextLength(saved));
    printAccounts(group, people->ana, eranosTextData(saved));
    EranosExpenseGroup *loaded = nullptr;
    must(eranosExpenseGroupLoad(eranosTextData(saved), &loaded));
    printLine("Loading replays every entry through the same checks as adding it and rebuilds the\n"
              "totals: the loaded group equals the saved one. Reading a SavedExpenseGroup and\n"
              "loading it with into_expense_group reports a refusal as a typed error, not as text:");
    EranosError *const format = changedRefusal(eranosTextData(saved), "\"format\": 3", "\"format\": 4");
    EranosError *const blank = changedRefusal(eranosTextData(saved), "\"name\": \"Lisbon and Porto\"", "\"name\": \" \"");
    EranosError *const homonym = changedRefusal(eranosTextData(saved), "\"name\": \"Lodging\"", "\"name\": \"FOOD\"");
    check(isFailure(format, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNSUPPORTED_FORMAT) && namesFormat(format, 4),
          "a later format is refused");
    check(isFailure(blank, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_EMPTY_NAME), "a blank name is refused");
    check(isFailure(homonym, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME) &&
              namesExisting(homonym, categories->food),
          "a caseless homonym is refused");
    Table *const table = loadTable(2);
    addLoadRow(table, &(LoadRow){.change = lineOf("\"format\": 4"), .message = messageLine(format)});
    addLoadRow(table, &(LoadRow){.change = lineOf("a blank group name"), .message = messageLine(blank)});
    addLoadRow(table, &(LoadRow){.change = lineOf("Lodging renamed FOOD"), .message = messageLine(homonym)});
    tablePrint(table);

    EranosText *const resaved = eranosTextCreate();
    eranosExpenseGroupSave(loaded, resaved);
    check(0 == strcmp(eranosTextData(resaved), eranosTextData(saved)), "the loaded group saves as the group did");
    EranosSummary *const before = summaryOf(group);
    EranosSummary *const after = summaryOf(loaded);
    check(sameSummaries(before, after), "the loaded group has the group's totals");
    eranosSummaryDestroy(after);
    eranosSummaryDestroy(before);
    eranosTextDestroy(resaved);
    eranosExpenseGroupDestroy(loaded);
    eranosTextDestroy(saved);
}

// 13. Values on their own.

/// Prints four currencies, and that an unknown code is refused.
static void currencies(void) {
    char const *const codes[] = {"EUR", "RON", "JPY", "BHD"};
    Table *const table = currencyTable(2);
    EranosText *const code = eranosTextCreate();
    EranosText *const name = eranosTextCreate();
    for (size_t index = 0; 4 > index; ++index) {
        EranosCurrencyDetails *details = nullptr;
        must(eranosCurrencyDetailsOf(codes[index], &details));
        eranosCurrencyDetailsCode(details, code);
        eranosCurrencyDetailsName(details, name);
        addCurrencyRow(table, &(CurrencyRow){
                                  .code = textLine(code),
                                  .name = textLine(name),
                                  .numericCode = lineOf("%ld", (long)eranosCurrencyDetailsNumericCode(details)),
                                  .decimalPlaces = lineOf("%ld", (long)eranosCurrencyDetailsDecimalPlaces(details)),
                              });
        eranosCurrencyDetailsDestroy(details);
    }
    eranosTextDestroy(name);
    eranosTextDestroy(code);
    EranosCurrencyDetails *unknownDetails = nullptr;
    EranosError *const unknown = eranosCurrencyDetailsOf("XYZ", &unknownDetails);
    check(nullptr != unknown && nullptr == unknownDetails, "an unknown code is refused");
    EranosCurrencyDetailsList *const all = eranosCurrencyDetailsListAll();
    Line const message = messageLine(unknown);
    printFormatted("Currencies are the %zu active ISO 4217 codes with a minor unit; an unknown code is\n"
                   "refused (%s):",
                   eranosCurrencyDetailsListLength(all), message.text);
    eranosCurrencyDetailsListDestroy(all);
    tablePrint(table);
    EranosCurrencyDetails *dinar = nullptr;
    must(eranosCurrencyDetailsOf("BHD", &dinar));
    check(3 == eranosCurrencyDetailsDecimalPlaces(dinar), "the dinar has three decimal places");
    eranosCurrencyDetailsDestroy(dinar);
}

/// The sum of @p left and @p right, or the refusal of the sum; destroys both.
[[nodiscard]] static Line sumOf(EranosMoney *const left, EranosMoney *const right, int64_t *const minorUnits) {
    EranosMoney *sum = nullptr;
    EranosError *const refused = eranosMoneyAdd(left, right, &sum);
    eranosMoneyDestroy(right);
    eranosMoneyDestroy(left);
    if (nullptr != refused) {
        return messageLine(refused);
    }
    *minorUnits = eranosMoneyMinorUnits(sum);
    return moneyLine(sum);
}

/// The message of the refusal of the amount @p text of @p currency.
[[nodiscard]] static Line amountRefusal(char const *const text, char const *const currency) {
    EranosMoney *money = nullptr;
    EranosError *const refused = eranosMoneyParse(text, currency, &money);
    check(nullptr != refused && nullptr == money, "the amount is refused");
    return messageLine(refused);
}

/// Prints what amounts do: sums, differences, refusals, minor units, and a cost's rate kept for another amount.
static void amounts(void) {
    int64_t total = 0;
    Line const sum = sumOf(eur("12.34"), eur("0.66"), &total);
    EranosMoney *const five = eur("5.00");
    EranosMoney *const more = eur("7.50");
    EranosMoney *change = nullptr;
    must(eranosMoneySubtract(five, more, &change));
    eranosMoneyDestroy(more);
    eranosMoneyDestroy(five);
    int64_t const changeUnits = eranosMoneyMinorUnits(change);
    Line const changeLine = moneyLine(change);
    int64_t unused = 0;
    Line const mixed = sumOf(eur("1.00"), ron("1.00"), &unused);
    Line const decimals = amountRefusal("1.005", "EUR");
    EranosMoney *const yen = amountOf("1500", "JPY");
    int64_t const yenUnits = eranosMoneyMinorUnits(yen);
    Line const yenLine = moneyLine(yen);
    EranosMoney *const dinars = minorUnitsOf(1'234, "BHD");
    Line const dinarsLine = moneyLine(dinars);
    EranosMoney *const nothing = minorUnitsOf(0, "RON");
    bool const zero = 0 == eranosMoneyMinorUnits(nothing);
    Line const nothingLine = moneyLine(nothing);
    EranosCost *const cost = convertedCost(eur("10.00"), "4.9765");
    EranosText *const rate = eranosTextCreate();
    (void)eranosCostRate(cost, rate);
    EranosCost *const doubled = convertedCost(eur("20.00"), eranosTextData(rate));
    Line const costLine = moneyLine(eranosCostAmount(cost));
    Line const doubledLine = moneyLine(eranosCostAmount(doubled));
    printLine("Amounts are exact minor units in their currency:");
    Table *const table = valueTable(2);
    addValueRow(table, &(ValueRow){.input = lineOf("12.34 EUR + 0.66 EUR"), .result = sum});
    addValueRow(table, &(ValueRow){
                           .input = lineOf("5.00 EUR − 7.50 EUR"),
                           .result = lineOf("%s, negative: %s", changeLine.text, 0 > changeUnits ? "true" : "false"),
                       });
    addValueRow(table, &(ValueRow){.input = lineOf("1.00 EUR + 1.00 RON"), .result = mixed});
    addValueRow(table, &(ValueRow){.input = lineOf("1.005 EUR"), .result = decimals});
    addValueRow(table, &(ValueRow){.input = lineOf("1500 JPY"), .result = lineOf("%s, %lld minor units", yenLine.text, (long long)yenUnits)});
    addValueRow(table, &(ValueRow){.input = lineOf("1234 minor units of BHD"), .result = dinarsLine});
    addValueRow(table, &(ValueRow){.input = lineOf("zero RON"), .result = lineOf("%s, zero: %s", nothingLine.text, zero ? "true" : "false")});
    addValueRow(table, &(ValueRow){
                           .input = lineOf("10.00 EUR at 4.9765"),
                           .result = lineOf("%s at %s, then %s at the same rate", costLine.text, eranosTextData(rate), doubledLine.text),
                       });
    tablePrint(table);
    check(1'300 == total, "12.34 EUR and 0.66 EUR make 13.00 EUR");
    check(-250 == changeUnits, "5.00 EUR less 7.50 EUR is -2.50 EUR");
    check(0 == strcmp(mixed.text, "cannot combine EUR and RON amounts"), "amounts in two currencies do not combine");
    check(0 == strcmp(dinarsLine.text, "1.234 BHD"), "a dinar has three decimal places");
    eranosCostDestroy(doubled);
    eranosTextDestroy(rate);
    eranosCostDestroy(cost);
}

/// The text of the percentage of @p basisPoints.
[[nodiscard]] static Line percentageLine(int32_t const basisPoints) {
    EranosText *const text = eranosTextCreate();
    eranosPercentageFormat(basisPoints, text);
    Line const line = textLine(text);
    eranosTextDestroy(text);
    return line;
}

/// The text of @p hundredths hundredths of a part.
[[nodiscard]] static Line partsLine(int32_t const hundredths) {
    EranosText *const text = eranosTextCreate();
    eranosPartsFormat(hundredths, text);
    Line const line = textLine(text);
    eranosTextDestroy(text);
    return line;
}

/// Prints what percentages, parts, and rates read and print.
static void ratios(void) {
    int32_t basisPoints = 0;
    must(eranosPercentageParse("12.5", &basisPoints));
    int32_t hundredths = 0;
    must(eranosPartsParse("1.5", &hundredths));
    int32_t unusedPoints = 0;
    Line const tooPrecise = messageLine(eranosPercentageParse("33.333", &unusedPoints));
    int32_t unusedHundredths = 0;
    Line const noParts = messageLine(eranosPartsParse("0", &unusedHundredths));
    EranosText *const rate = eranosTextCreate();
    Line const exchange = valueOr(eranosExchangeRateParse("4.97650", rate), rate);
    Line const noRate = messageLine(eranosExchangeRateParse("0", rate));
    eranosTextDestroy(rate);
    Line const percentage = percentageLine(basisPoints);
    Line const parts = partsLine(hundredths);
    printLine("Percentages, parts, and rates are exact decimals:");
    Table *const table = valueTable(2);
    addValueRow(table, &(ValueRow){.input = lineOf("12.5%%"), .result = lineOf("%s, %ld basis points", percentage.text, (long)basisPoints)});
    addValueRow(table, &(ValueRow){.input = lineOf("333 basis points"), .result = percentageLine(333)});
    addValueRow(table, &(ValueRow){.input = lineOf("33.333%%"), .result = tooPrecise});
    addValueRow(table, &(ValueRow){.input = lineOf("1.5 parts"), .result = lineOf("%s, %ld hundredths", parts.text, (long)hundredths)});
    addValueRow(table, &(ValueRow){.input = lineOf("3 whole parts"), .result = partsLine(300)});
    addValueRow(table, &(ValueRow){.input = lineOf("25 hundredths of a part"), .result = partsLine(25)});
    addValueRow(table, &(ValueRow){.input = lineOf("0 parts"), .result = noParts});
    addValueRow(table, &(ValueRow){.input = lineOf("a rate of 4.97650"), .result = exchange});
    addValueRow(table, &(ValueRow){.input = lineOf("a rate of 0"), .result = noRate});
    tablePrint(table);
    check(1'250 == basisPoints, "12.5 % is 1250 basis points");
    check(150 == hundredths, "1.5 parts are 150 hundredths");
}

/// The message the library refuses @p text with as a member's identifier, which it checks is @p failure.
[[nodiscard]] static Line identifierRefusal(EranosExpenseGroup const *const group, char const *const text, EranosFailure const failure) {
    EranosMember *member = nullptr;
    EranosError *const refusal = eranosExpenseGroupMember(group, text, &member);
    check(isFailure(refusal, ERANOS_ERROR_KIND_ID, failure) && nullptr == member, "the text is refused as an identifier");
    return messageLine(refusal);
}

/// Prints what dates, times, identifiers, and language tags read and print.
static void datesAndIdentifiers(EranosExpenseGroup const *const group, People const *const people) {
    EranosText *const text = eranosTextCreate();
    EranosCalendarDate date = {.year = 0, .month = 0, .day = 0};
    must(eranosCalendarDateParse("2026-10-01", &date));
    eranosCalendarDateFormat(date, text);
    Line const dateLine = lineOf("%s: day %ld, month %ld, year %ld", eranosTextData(text), (long)date.day, (long)date.month, (long)date.year);
    Line const leapDay = messageLine(eranosCalendarDateCheck((EranosCalendarDate){.year = 2027, .month = 2, .day = 29}));
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    must(eranosTimeOfDayParse("22:15", &time));
    eranosTimeOfDayFormat(time, text);
    Line const timeLine = lineOf("%s: hour %ld, minute %ld", eranosTextData(text), (long)time.hour, (long)time.minute);
    Line const midnight = messageLine(eranosTimeOfDayCheck((EranosTimeOfDay){.hour = 24, .minute = 0}));
    EranosMember *member = nullptr;
    must(eranosExpenseGroupMember(group, people->ana.text, &member));
    check(nullptr != member, "Ana is a member");
    eranosMemberId(member, text);
    bool const found = 0 == strcmp(eranosTextData(text), people->ana.text);
    eranosMemberDestroy(member);
    Line const empty = identifierRefusal(group, "", ERANOS_FAILURE_ID_EMPTY);
    Line const afterASpace = identifierRefusal(group, " ana", ERANOS_FAILURE_ID_SURROUNDING_WHITESPACE);
    Line const withATab = identifierRefusal(group, "a\tb", ERANOS_FAILURE_ID_CONTROL_CHARACTER);
    char letters[66] = {};
    memset(letters, 'a', sizeof letters - 1);
    Line const tooLong = identifierRefusal(group, letters, ERANOS_FAILURE_ID_TOO_LONG);
    Line const language = valueOr(eranosLanguageParse("pt-pt", text), text);
    Line const underscore = messageLine(eranosLanguageParse("pt_PT", text));
    eranosTextDestroy(text);
    printLine("Dates, times, identifiers, and language tags read and print the ISO forms:");
    Table *const table = valueTable(2);
    addValueRow(table, &(ValueRow){.input = lineOf("2026-10-01"), .result = dateLine});
    addValueRow(table, &(ValueRow){.input = lineOf("29 February 2027"), .result = leapDay});
    addValueRow(table, &(ValueRow){.input = lineOf("22:15"), .result = timeLine});
    addValueRow(table, &(ValueRow){.input = lineOf("24:00"), .result = midnight});
    addValueRow(table, &(ValueRow){
                           .input = lineOf("Ana's identifier"),
                           .result = lineOf("%s, the same after a round trip: %s", people->ana.text, found ? "true" : "false"),
                       });
    addValueRow(table, &(ValueRow){.input = lineOf("empty text"), .result = empty});
    addValueRow(table, &(ValueRow){.input = lineOf("ana after a space"), .result = afterASpace});
    addValueRow(table, &(ValueRow){.input = lineOf("a tab between a and b"), .result = withATab});
    addValueRow(table, &(ValueRow){.input = lineOf("65 letters a"), .result = tooLong});
    addValueRow(table, &(ValueRow){.input = lineOf("pt-pt"), .result = language});
    addValueRow(table, &(ValueRow){.input = lineOf("pt_PT"), .result = underscore});
    tablePrint(table);
    check(1 == date.day && 10 == date.month && 2026 == date.year, "the date reads as 1 October 2026");
    check(22 == time.hour && 15 == time.minute, "the time reads as 22:15");
    check(found, "Ana's identifier comes back the same");
}

/// Prints the values on their own: currencies, amounts, ratios, dates, times, identifiers, and language tags.
static void values(EranosExpenseGroup const *const group, People const *const people) {
    section("13. Values on their own");
    currencies();
    amounts();
    ratios();
    datesAndIdentifiers(group, people);
}

// 14. Sharing the group.

/// What an asynchronous call reported, and whether it has: its callback runs on the group's worker thread, which
/// signals the waiting one.
typedef struct Waiter {
    /// Signalled once the callback ran.
    pthread_cond_t condition;
    /// Whether the callback ran.
    bool done;
    /// The error the call reported, or nullptr.
    EranosError *error;
    /// Guards the rest.
    pthread_mutex_t mutex;
    /// The value the call reported, or nullptr.
    void *value;
} Waiter;

/// @p waiter, ready for a call.
[[nodiscard]] static Waiter *waiting(Waiter *const waiter) {
    waiter->done = false;
    waiter->error = nullptr;
    waiter->value = nullptr;
    (void)pthread_mutex_init(&waiter->mutex, nullptr);
    (void)pthread_cond_init(&waiter->condition, nullptr);
    return waiter;
}

/// Records what the call reported in @p waiter and wakes its thread.
static void finish(Waiter *const waiter, EranosError *const error, void *const value) {
    (void)pthread_mutex_lock(&waiter->mutex);
    waiter->error = error;
    waiter->value = value;
    waiter->done = true;
    (void)pthread_cond_signal(&waiter->condition);
    (void)pthread_mutex_unlock(&waiter->mutex);
}

/// Waits for the callback of @p waiter, and gives what the call reported.
static void *outcome(Waiter *const waiter, EranosError **const error) {
    (void)pthread_mutex_lock(&waiter->mutex);
    while (!waiter->done) {
        (void)pthread_cond_wait(&waiter->condition, &waiter->mutex);
    }
    (void)pthread_mutex_unlock(&waiter->mutex);
    (void)pthread_cond_destroy(&waiter->condition);
    (void)pthread_mutex_destroy(&waiter->mutex);
    *error = waiter->error;
    return waiter->value;
}

/// Waits for the callback of @p waiter, which reports a call the example expects to succeed, and gives its value.
static void *awaited(Waiter *const waiter) {
    EranosError *error = nullptr;
    void *const value = outcome(waiter, &error);
    must(error);
    return value;
}

/// Receives an outcome without a value.
static void receiveUnit(void *const context, EranosError *const error) { finish(context, error, nullptr); }

/// Receives text.
static void receiveText(void *const context, EranosError *const error, EranosText *const text) { finish(context, error, text); }

/// Receives a currency's details.
static void receiveCurrency(void *const context, EranosError *const error, EranosCurrencyDetails *const currency) {
    finish(context, error, currency);
}

/// Receives the expenses.
static void receiveExpenses(void *const context, EranosError *const error, EranosExpenseList *const expenses) { finish(context, error, expenses); }

/// Receives a read handle.
static void receiveReader(void *const context, EranosError *const error, EranosExpenseGroupReader *const reader) { finish(context, error, reader); }

/// Receives a settlement.
static void receiveSettlement(void *const context, EranosError *const error, EranosSettlementPlan *const settlement) {
    finish(context, error, settlement);
}

/// Receives a summary.
static void receiveSummary(void *const context, EranosError *const error, EranosSummary *const summary) { finish(context, error, summary); }

/// The summary of @p group, read asynchronously; the caller destroys it.
[[nodiscard]] static EranosSummary *summaryAsync(EranosExpenseGroup const *const group) {
    Waiter waiter;
    EranosSummaryCallback const callback = {.context = waiting(&waiter), .run = receiveSummary};
    eranosExpenseGroupSummaryAsync(group, nullptr, &callback);
    return awaited(&waiter);
}

/// A read handle on @p group, taken asynchronously; the caller destroys it.
[[nodiscard]] static EranosExpenseGroupReader *readAsync(EranosExpenseGroup const *const group) {
    Waiter waiter;
    EranosExpenseGroupReaderCallback const callback = {.context = waiting(&waiter), .run = receiveReader};
    eranosExpenseGroupReadAsync(group, nullptr, &callback);
    return awaited(&waiter);
}

/// The first name of @p member, read through @p reader, or ? for one the group does not hold.
[[nodiscard]] static Line readerName(EranosExpenseGroupReader const *const reader, char const *const member) {
    EranosMember *found = nullptr;
    must(eranosExpenseGroupReaderMember(reader, member, &found));
    if (nullptr == found) {
        return lineOf("%s", "?");
    }
    EranosText *const text = eranosTextCreate();
    eranosMemberFirstName(found, text);
    Line const line = textLine(text);
    eranosTextDestroy(text);
    eranosMemberDestroy(found);
    return line;
}

/// Adds the airport coffee through @p other, another handle on the group, and prints the plan that settles it.
static void settleCoffee(EranosExpenseGroup *const other, EranosExpenseGroup const *const group, People const *const people) {
    Members const everyone = {.count = 5, .ids = {people->ana, people->bob, people->carol, people->dan, people->eva}};
    EranosExpense *const coffee = expenseOf("Airport coffee", localCost(ron("75.00")), soleSplit(people->bob), equalSplit(&everyone), october(8));
    Waiter added;
    EranosTextCallback const addedCallback = {.context = waiting(&added), .run = receiveText};
    eranosExpenseGroupAddExpenseAsync(other, coffee, nullptr, &addedCallback);
    eranosTextDestroy(awaited(&added));
    eranosExpenseDestroy(coffee);
    Waiter planned;
    EranosSettlementPlanCallback const plannedCallback = {.context = waiting(&planned), .run = receiveSettlement};
    eranosExpenseGroupSettlementAsync(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS, nullptr, &plannedCallback);
    EranosSettlementPlan *const plan = awaited(&planned);
    EranosPaymentList *const payments = eranosSettlementPlanPayments(plan);
    printLine("An app shares the group between its tasks and threads through a\n"
              "SharedExpenseGroup, whose calls are futures any executor runs; here the library's\n"
              "block_on runs each one on this thread. Bob adds the airport coffee through another\n"
              "handle, and the plan that settles it comes through the first:");
    EranosExpenseGroupReader *const reader = readAsync(group);
    Table *const table = paymentTable(2);
    EranosText *const from = eranosTextCreate();
    EranosText *const to = eranosTextCreate();
    bool toBob = true;
    for (size_t index = 0; eranosPaymentListLength(payments) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(payments, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        EranosMoney *const amount = eranosPaymentAmount(payment);
        toBob = toBob && 0 == strcmp(eranosTextData(to), people->bob.text) && 1'500 == eranosMoneyMinorUnits(amount);
        addPaymentRow(table, &(PaymentRow){
                                 .from = readerName(reader, eranosTextData(from)),
                                 .to = readerName(reader, eranosTextData(to)),
                                 .amount = moneyLine(amount),
                             });
        eranosPaymentDestroy(payment);
    }
    tablePrint(table);
    eranosExpenseGroupReaderRelease(reader);
    check(4 == eranosPaymentListLength(payments) && toBob, "everyone pays Bob 15.00 RON");
    eranosTextDestroy(to);
    eranosTextDestroy(from);
    eranosExpenseGroupReaderDestroy(reader);
    eranosPaymentListDestroy(payments);
    eranosSettlementPlanDestroy(plan);
}

/// Shares @p group between tasks: an expense added through another handle, a settlement computed asynchronously, and a
/// change that waits for a report's read and is cancelled.
static void sharing(EranosExpenseGroup *const group, People const *const people) {
    section("14. Sharing the group");
    // Another handle on the same group, as another task holds: a group is shared as it is.
    EranosExpenseGroup *const other = group;
    settleCoffee(other, group, people);

    EranosSummary *const before = summaryAsync(group);
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = nullptr;
    must(eranosCurrencyRateCreate("RON", "0.2", &rate));
    eranosCurrencyRateListPush(rates, rate);
    eranosCurrencyRateDestroy(rate);
    // A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
    // behind the change, so the report reads through its own handle.
    EranosExpenseGroupReader *const report = readAsync(group);
    EranosCancellation *const cancellation = eranosCancellationCreate();
    Waiter changed;
    EranosUnitCallback const changedCallback = {.context = waiting(&changed), .run = receiveUnit};
    eranosExpenseGroupChangeCurrencyAsync(group, "EUR", rates, cancellation, &changedCallback);
    EranosSummary *reportedSummary = nullptr;
    must(eranosExpenseGroupReaderSummary(report, &reportedSummary));
    // Cancelling stops the waiting change, which leaves the group as it was.
    eranosCancellationCancel(cancellation);
    eranosExpenseGroupReaderRelease(report);
    EranosError *cancelled = nullptr;
    (void)outcome(&changed, &cancelled);
    bool const wasCancelled = nullptr != cancelled && ERANOS_ERROR_KIND_CANCELLED == eranosErrorKind(cancelled);
    eranosErrorDestroy(cancelled);
    EranosSummary *const after = summaryAsync(group);
    Waiter currency;
    EranosCurrencyDetailsCallback const currencyCallback = {.context = waiting(&currency), .run = receiveCurrency};
    eranosExpenseGroupCurrencyAsync(group, nullptr, &currencyCallback);
    Line const code = currencyLine(awaited(&currency));
    Line const spent = moneyLine(eranosSummaryTotalSpent(after));
    printFormatted("While a report reads the group, a change of its currency to EUR waits for the\n"
                   "report. The app drops the waiting change, which cancels it: the group still\n"
                   "keeps its books in %s, and the trip cost %s, as the report read.",
                   code.text, spent.text);
    Waiter listed;
    EranosExpenseListCallback const listedCallback = {.context = waiting(&listed), .run = receiveExpenses};
    eranosExpenseGroupExpensesAsync(group, nullptr, &listedCallback);
    EranosExpenseList *const expenses = awaited(&listed);
    printFormatted("With the other handle gone, the app takes the group back: %zu expenses.", eranosExpenseListLength(expenses));

    check(wasCancelled, "the change is cancelled");
    check(sameSummaries(before, after), "the cancelled change leaves the group as it was");
    check(sameAmount(eranosSummaryTotalSpent(reportedSummary), eranosSummaryTotalSpent(after)), "the report read what the group holds");
    check(0 == strcmp(code.text, "RON"), "the group keeps its books in RON");
    eranosExpenseListDestroy(expenses);
    eranosSummaryDestroy(after);
    eranosSummaryDestroy(reportedSummary);
    eranosCancellationDestroy(cancellation);
    eranosExpenseGroupReaderDestroy(report);
    eranosCurrencyRateListDestroy(rates);
    eranosSummaryDestroy(before);
}

int main(void) {
    printLine("Lisbon and Porto: a grand tour of eranos\n"
              "════════════════════════════════════════");
    printLine("Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each\n"
              "section below uses another part of the library, from creating the group to saving\n"
              "it, and checks its key numbers.");
    EranosExpenseGroup *const group = theGroup();
    People const people = members(group);
    Categories const categories = addCategories(group);
    Id const seafood = expenses(group, &people, &categories);
    anotherCurrency(group, &people, &categories);
    edits(group, &people, &categories, seafood);
    comingsAndGoings(group, &people);
    summary(group, &people, &categories);
    removingACategory(group, &categories);
    newCurrency(group, &people);
    settlingUp(group);
    savingAndLoading(group, &people, &categories);
    values(group, &people);
    sharing(group, &people);
    eranosExpenseGroupDestroy(group);
    return EXIT_SUCCESS;
}
