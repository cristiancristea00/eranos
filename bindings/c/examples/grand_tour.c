// Every feature of the library in one story: a group and its details, members with contact details and accounts,
// categories, an expense for every way of dividing one, another currency, edits and refusals, members who come and go,
// the summary, a new group currency, settling up, saving and loading, the values on their own, and the group shared
// between tasks. Every section checks its key numbers. The port of the Rust example `grand_tour`, which prints the same
// report.

#include "eranos.h"
#include "report.h"

#include <pthread.h>
#include <stdbool.h>
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

/// The amount @p text of euros.
static EranosMoney *eur(char const *const text) { return amountOf(text, "EUR"); }

/// The amount @p text of lei.
static EranosMoney *ron(char const *const text) { return amountOf(text, "RON"); }

/// A date of the trip, in October 2026.
static EranosCalendarDate october(int32_t const day) { return (EranosCalendarDate){.day = day, .month = 10, .year = 2026}; }

/// The text that @p value holds, or the message of @p error when the call that wrote it was refused; destroys @p error.
static Line valueOr(EranosError *const error, EranosText const *const value) { return NULL == error ? textLine(value) : messageLine(error); }

/// The currency of @p group, as its code.
static Line currencyOf(EranosGroup const *const group) {
    EranosText *const text = eranosTextCreate();
    must(eranosGroupCurrency(group, text));
    Line const line = textLine(text);
    eranosTextDestroy(text);
    return line;
}

/// Zero in the currency of @p group, as the library writes it.
static Line zeroOf(EranosGroup const *const group) {
    Line const currency = currencyOf(group);
    EranosMoney *zero = NULL;
    must(eranosMoneyCreate(currency.text, 0, &zero));
    return moneyLine(zero);
}

/// What @p group spent, as the library writes it.
static Line totalSpent(EranosGroup const *const group) {
    EranosSummary *summary = NULL;
    must(eranosGroupSummary(group, &summary));
    Line const line = moneyLine(eranosSummaryTotalSpent(summary));
    eranosSummaryDestroy(summary);
    return line;
}

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

/// The amount of @p cost, as entered; destroys @p cost.
static Line costAmount(EranosCost *const cost) {
    Line const line = moneyLine(eranosCostAmount(cost));
    eranosCostDestroy(cost);
    return line;
}

/// The identifier of the built-in category, which the group lists first.
static Id builtInCategory(EranosGroup const *const group) {
    EranosCategoryList *categories = NULL;
    must(eranosGroupCategories(group, &categories));
    EranosCategory *const first = eranosCategoryListGet(categories, 0);
    check(NULL != first, "every group has the built-in category");
    EranosText *const text = eranosTextCreate();
    (void)eranosCategoryId(first, text);
    Id const id = idOf(text);
    eranosTextDestroy(text);
    eranosCategoryDestroy(first);
    eranosCategoryListDestroy(categories);
    return id;
}

/// Whether @p left and @p right are the same amount.
static bool sameAmount(EranosMoney *const left, EranosMoney *const right) {
    EranosText *const leftText = eranosTextCreate();
    EranosText *const rightText = eranosTextCreate();
    eranosMoneyCurrency(left, leftText);
    eranosMoneyCurrency(right, rightText);
    bool const same = 0 == strcmp(eranosTextData(leftText), eranosTextData(rightText)) && eranosMoneyMinorUnits(left) == eranosMoneyMinorUnits(right);
    eranosTextDestroy(rightText);
    eranosTextDestroy(leftText);
    eranosMoneyDestroy(right);
    eranosMoneyDestroy(left);
    return same;
}

/// Whether @p money is @p minorUnits minor units; destroys @p money, which may be NULL.
static bool isMinorUnits(EranosMoney *const money, int64_t const minorUnits) {
    bool const same = NULL != money && minorUnits == eranosMoneyMinorUnits(money);
    eranosMoneyDestroy(money);
    return same;
}

/// Whether @p balance is of @p kind, and of @p minorUnits when it has an amount; destroys @p balance.
static bool isBalance(EranosBalance *const balance, EranosBalanceKind const kind, int64_t const minorUnits) {
    EranosMoney *const amount = eranosBalanceAmount(balance);
    bool const same = kind == eranosBalanceKind(balance) && (NULL == amount || minorUnits == eranosMoneyMinorUnits(amount));
    eranosMoneyDestroy(amount);
    eranosBalanceDestroy(balance);
    return same;
}

/// Whether the totals of every member in @p left and @p right are the same.
static bool sameMembers(EranosSummary const *const left, EranosSummary const *const right) {
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
static bool sameCategories(EranosCategoryTotalList const *const left, EranosCategoryTotalList const *const right) {
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
static bool sameSummaries(EranosSummary const *const left, EranosSummary const *const right) {
    EranosCategoryTotalList *const leftCategories = eranosSummaryCategoryTotals(left);
    EranosCategoryTotalList *const rightCategories = eranosSummaryCategoryTotals(right);
    bool const same = sameAmount(eranosSummaryTotalSpent(left), eranosSummaryTotalSpent(right)) && sameMembers(left, right) &&
                      sameCategories(leftCategories, rightCategories);
    eranosCategoryTotalListDestroy(rightCategories);
    eranosCategoryTotalListDestroy(leftCategories);
    return same;
}

/// The summary of @p group, which the caller destroys.
static EranosSummary *summaryOf(EranosGroup const *const group) {
    EranosSummary *summary = NULL;
    must(eranosGroupSummary(group, &summary));
    return summary;
}

/// The totals of @p member in @p group, which the caller destroys.
static EranosMemberSummary *memberTotals(EranosGroup const *const group, Id const member) {
    EranosSummary *const summary = summaryOf(group);
    EranosMemberSummary *const totals = totalsOf(summary, member.text);
    check(NULL != totals, "the member has totals");
    eranosSummaryDestroy(summary);
    return totals;
}

/// Prints what each member of @p group paid, consumed, sent, and received, and their balance.
static void printBalances(EranosGroup const *const group) {
    EranosSummary *const summary = summaryOf(group);
    Line const zero = zeroOf(group);
    EranosBalance *const settled = eranosBalanceCreateSettled();
    Line const settledLine = balanceLine(settled);
    EranosMemberList *members = NULL;
    must(eranosGroupMembers(group, &members));
    Table *const table = tableCreate(6, (char const *const[]){"Member", "Paid", "Consumed", "Sent", "Received", "Balance"}, 1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        eranosMemberFirstName(member, text);
        // The summary lists the members with a total; every total of the others is zero.
        if (NULL == found) {
            tableAdd(table, (Line const[]){textLine(text), zero, zero, zero, zero, settledLine});
        } else {
            tableAdd(table, (Line const[]){
                                textLine(text),
                                moneyLine(eranosMemberSummaryPaid(found)),
                                moneyLine(eranosMemberSummaryShare(found)),
                                moneyLine(eranosMemberSummarySent(found)),
                                moneyLine(eranosMemberSummaryReceived(found)),
                                balanceLine(eranosMemberSummaryBalance(found)),
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

/// How many expenses of @p group are in @p category, those without one counting as in @p builtIn.
static size_t expensesIn(EranosGroup const *const group, char const *const category, Id const builtIn) {
    EranosExpenseList *expenses = NULL;
    must(eranosGroupExpenses(group, &expenses));
    EranosText *const text = eranosTextCreate();
    size_t count = 0;
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        char const *const own = eranosExpenseCategory(expense, text) ? eranosTextData(text) : builtIn.text;
        if (0 == strcmp(own, category)) {
            ++count;
        }
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(text);
    eranosExpenseListDestroy(expenses);
    return count;
}

/// The name of the category @p id of @p group, or ? for one the group does not hold.
static Line categoryName(EranosGroup const *const group, char const *const id) {
    EranosCategory *category = NULL;
    must(eranosGroupCategory(group, id, &category));
    if (NULL == category) {
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
static void printCategories(EranosGroup const *const group) {
    Id const builtIn = builtInCategory(group);
    EranosSummary *const summary = summaryOf(group);
    EranosCategoryTotalList *const totals = eranosSummaryCategoryTotals(summary);
    Table *const table = tableCreate(3, (char const *const[]){"Category", "Expenses", "Total"}, 1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosCategoryTotalListLength(totals) > index; ++index) {
        EranosCategoryTotal *const total = eranosCategoryTotalListGet(totals, index);
        eranosCategoryTotalCategory(total, text);
        tableAdd(table, (Line const[]){
                            categoryName(group, eranosTextData(text)),
                            lineOf("%zu", expensesIn(group, eranosTextData(text), builtIn)),
                            moneyLine(eranosCategoryTotalTotal(total)),
                        });
        eranosCategoryTotalDestroy(total);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosCategoryTotalListDestroy(totals);
    eranosSummaryDestroy(summary);
}

/// Whether an entry in @p currency is shown when the entries in @p hidden are not; NULL hides none.
static bool isShown(char const *const hidden, EranosCost const *const cost) {
    if (NULL == hidden) {
        return true;
    }
    EranosMoney *const amount = eranosCostAmount(cost);
    EranosText *const text = eranosTextCreate();
    eranosMoneyCurrency(amount, text);
    bool const shown = 0 != strcmp(eranosTextData(text), hidden);
    eranosTextDestroy(text);
    eranosMoneyDestroy(amount);
    return shown;
}

/// Adds every expense of @p group but those in @p hidden to @p table, as entered and in the group currency.
static void addExpenseEntries(EranosGroup const *const group, char const *const hidden, Table *const table) {
    Line const zero = zeroOf(group);
    EranosExpenseList *expenses = NULL;
    must(eranosGroupExpenses(group, &expenses));
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        EranosCost *const cost = eranosExpenseCost(expense);
        if (isShown(hidden, cost)) {
            (void)eranosExpenseId(expense, text);
            EranosExpenseShares *shares = NULL;
            must(eranosGroupExpenseShares(group, eranosTextData(text), &shares));
            Line const converted = NULL == shares ? zero : moneyLine(eranosExpenseSharesTotal(shares));
            eranosExpenseName(expense, text);
            EranosSplit *const payers = eranosExpensePayers(expense);
            Members const paidBy = splitMembers(payers);
            EranosMoney *total = NULL;
            must(eranosExpenseTotal(expense, &total));
            tableAdd(table, (Line const[]){textLine(text), names(group, &paidBy), moneyLine(total), rateLine(eranosExpenseCost(expense)), converted});
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
static void addTransferEntries(EranosGroup const *const group, char const *const hidden, Table *const table) {
    Line const zero = zeroOf(group);
    EranosTransferList *transfers = NULL;
    must(eranosGroupTransfers(group, &transfers));
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosTransferListLength(transfers) > index; ++index) {
        EranosTransfer *const transfer = eranosTransferListGet(transfers, index);
        EranosCost *const cost = eranosTransferCost(transfer);
        if (isShown(hidden, cost)) {
            (void)eranosTransferId(transfer, text);
            EranosMoney *amount = NULL;
            must(eranosGroupTransferAmount(group, eranosTextData(text), &amount));
            Line const converted = NULL == amount ? zero : moneyLine(amount);
            eranosTransferTo(transfer, text);
            Line const to = firstName(group, eranosTextData(text));
            eranosTransferFrom(transfer, text);
            tableAdd(table, (Line const[]){
                                lineOf("Transfer to %s", to.text),
                                firstName(group, eranosTextData(text)),
                                costAmount(eranosTransferCost(transfer)),
                                rateLine(eranosTransferCost(transfer)),
                                converted,
                            });
        }
        eranosCostDestroy(cost);
        eranosTransferDestroy(transfer);
    }
    eranosTextDestroy(text);
    eranosTransferListDestroy(transfers);
}

/// Prints every expense and transfer of @p group but those in @p hidden, as entered and in the group currency.
static void printEntries(EranosGroup const *const group, char const *const hidden) {
    Table *const table = tableCreate(5, (char const *const[]){"Entry", "Paid by", "Entered as", "Rate", "In the group currency"}, 2);
    addExpenseEntries(group, hidden, table);
    addTransferEntries(group, hidden, table);
    tablePrint(table);
}

// 1. The group.

/// Creates the group, renames it, and prints its details.
static EranosGroup *theGroup(void) {
    section("1. The group");
    EranosNewGroup *details = NULL;
    must(eranosNewGroupCreate("EUR", "Lisbon", &details));
    must(eranosNewGroupSetDescription(details, "A week by the Atlantic"));
    must(eranosNewGroupSetLanguage(details, "pt-pt"));
    EranosGroup *group = NULL;
    must(eranosGroupCreate(details, &group));
    eranosNewGroupDestroy(details);
    EranosError *const blank = eranosGroupSetName(group, "   ");
    check(isFailure(blank, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_EMPTY_NAME), "a blank name is refused");
    must(eranosGroupSetName(group, "Lisbon and Porto"));
    must(eranosGroupSetDescription(group, "A week by the Atlantic, with a day in Porto"));
    EranosText *const text = eranosTextCreate();
    must(eranosGroupName(group, text));
    Line const name = textLine(text);
    Line const currency = currencyOf(group);
    must(eranosGroupLanguage(group, text));
    Line const language = textLine(text);
    must(eranosGroupDescription(group, text));
    Line const description = textLine(text);
    eranosTextDestroy(text);
    Table *const table = tableCreate(2, (char const *const[]){"Detail", "Value"}, 2);
    tableAdd(table, (Line const[]){lineOf("Name"), name});
    tableAdd(table, (Line const[]){lineOf("Currency"), currency});
    tableAdd(table, (Line const[]){lineOf("Language"), language});
    tableAdd(table, (Line const[]){lineOf("Description"), description});
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
static Line describe(EranosAccount const *const account) {
    EranosText *const name = eranosTextCreate();
    EranosText *const text = eranosTextCreate();
    eranosAccountName(account, name);
    EranosAccountDetails *const details = eranosAccountDetails(account);
    Line line = {{0}};
    if (eranosAccountDetailsIban(details, text)) {
        line = lineOf("%s, IBAN %s", eranosTextData(name), eranosTextData(text));
    } else {
        (void)eranosAccountDetailsUsername(details, text);
        line = lineOf("%s, username %s", eranosTextData(name), eranosTextData(text));
    }
    eranosAccountDetailsDestroy(details);
    eranosTextDestroy(text);
    eranosTextDestroy(name);
    return line;
}

/// The label and the details of every account of @p member, one per line.
static Line accountsOf(EranosMember const *const member) {
    EranosAccountList *const accounts = eranosMemberAccounts(member);
    Cell cell = {.lines = 0, .text = {{0}}};
    for (size_t index = 0; eranosAccountListLength(accounts) > index; ++index) {
        EranosAccount *const account = eranosAccountListGet(accounts, index);
        Line const line = describe(account);
        cellAdd(&cell, line.text);
        eranosAccountDestroy(account);
    }
    eranosAccountListDestroy(accounts);
    return cell.text;
}

/// The first and the last name of @p member, the last when there is one.
static Line fullName(EranosMember const *const member) {
    EranosText *const first = eranosTextCreate();
    EranosText *const last = eranosTextCreate();
    eranosMemberFirstName(member, first);
    Line const line = eranosMemberLastName(member, last) ? lineOf("%s %s", eranosTextData(first), eranosTextData(last)) : textLine(first);
    eranosTextDestroy(last);
    eranosTextDestroy(first);
    return line;
}

/// The email of @p member, or nothing when there is none.
static Line emailOf(EranosMember const *const member) {
    EranosText *const text = eranosTextCreate();
    Line const line = eranosMemberEmail(member, text) ? textLine(text) : lineOf("%s", "");
    eranosTextDestroy(text);
    return line;
}

/// The phone number of @p member, grouped for reading, or nothing when there is none.
static Line phoneOf(EranosMember const *const member) {
    EranosText *const text = eranosTextCreate();
    Line line = {{0}};
    if (eranosMemberPhone(member, text)) {
        EranosText *const international = eranosTextCreate();
        must(eranosPhoneNumberInternational(eranosTextData(text), international));
        line = textLine(international);
        eranosTextDestroy(international);
    }
    eranosTextDestroy(text);
    return line;
}

/// Prints every member of @p group with their contact and account details.
static void printMembers(EranosGroup const *const group) {
    EranosMemberList *members = NULL;
    must(eranosGroupMembers(group, &members));
    Table *const table = tableCreate(4, (char const *const[]){"Member", "Email", "Phone", "Accounts"}, 4);
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        tableAdd(table, (Line const[]){fullName(member), emailOf(member), phoneOf(member), accountsOf(member)});
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosMemberListDestroy(members);
}

/// "valid: " and @p value, or "refused: " and the message of @p error; destroys @p error.
static Line outcomeOf(EranosError *const error, char const *const value) {
    bool const valid = NULL == error;
    Line const message = messageLine(error);
    return valid ? lineOf("valid: %s", value) : lineOf("refused: %s", message.text);
}

/// Adds the row of the IBAN @p input to @p table: its electronic form and its country, or its refusal.
static void addIban(Table *const table, char const *const input) {
    EranosText *const iban = eranosTextCreate();
    EranosText *const country = eranosTextCreate();
    EranosError *const refused = eranosIbanParse(input, iban);
    Line value = {{0}};
    if (NULL == refused) {
        must(eranosIbanCountryCode(input, country));
        value = lineOf("%s, country %s", eranosTextData(iban), eranosTextData(country));
    }
    tableAdd(table, (Line const[]){lineOf("\"%s\"", input), lineOf("IBAN"), outcomeOf(refused, value.text)});
    eranosTextDestroy(country);
    eranosTextDestroy(iban);
}

/// Adds the row of the phone number @p input to @p table: its form for reading and its stored form, or its refusal.
static void addPhone(Table *const table, char const *const input) {
    EranosText *const international = eranosTextCreate();
    EranosText *const stored = eranosTextCreate();
    EranosError *const refused = eranosPhoneNumberInternational(input, international);
    Line value = {{0}};
    if (NULL == refused) {
        must(eranosPhoneNumberParse(input, stored));
        value = lineOf("%s, stored as %s", eranosTextData(international), eranosTextData(stored));
    }
    tableAdd(table, (Line const[]){lineOf("\"%s\"", input), lineOf("phone"), outcomeOf(refused, value.text)});
    eranosTextDestroy(stored);
    eranosTextDestroy(international);
}

/// Prints what validation makes of IBANs, an email, phone numbers, and blank required text.
static void validation(EranosGroup *const group) {
    printLine("IBANs are checked against the SWIFT registry and their mod-97 check digits,\n"
              "emails against RFC 5321 and 5322, and phone numbers against their country's\n"
              "numbering plan. Required text must not be blank:");
    Table *const table = tableCreate(3, (char const *const[]){"Input", "Checked as", "Result"}, 3);
    addIban(table, "ro49 aaaa 1b31 0075 9384 0000");
    addIban(table, "RO49 AAAA 1B31 0075 9384 0001");
    EranosText *const text = eranosTextCreate();
    EranosError *const email = eranosEmailParse("ana..popescu@example.com", text);
    tableAdd(table, (Line const[]){lineOf("\"ana..popescu@example.com\""), lineOf("email"), outcomeOf(email, eranosTextData(text))});
    addPhone(table, "+351 912 345 678");
    addPhone(table, "+40 612 345 678");
    EranosError *const username = eranosUsernameParse("   ", text);
    tableAdd(table, (Line const[]){lineOf("\"   \""), lineOf("username"), outcomeOf(username, eranosTextData(text))});
    EranosMember *member = NULL;
    must(eranosMemberCreate("  ", &member));
    EranosError *const firstName = eranosGroupAddMember(group, member, text);
    tableAdd(table, (Line const[]){lineOf("\"  \""), lineOf("first name"), outcomeOf(firstName, eranosTextData(text))});
    eranosMemberDestroy(member);
    eranosTextDestroy(text);
    tablePrint(table);
}

/// Adds Ana, with every detail, to @p group.
static Id addAna(EranosGroup *const group) {
    EranosMember *member = NULL;
    must(eranosMemberCreate("Ana", &member));
    EranosAccountDetails *iban = NULL;
    must(eranosAccountDetailsCreateIban("RO49 AAAA 1B31 0075 9384 0000", &iban));
    EranosAccountDetails *username = NULL;
    must(eranosAccountDetailsCreateUsername("@ana", &username));
    EranosAccount *bank = NULL;
    must(eranosAccountCreate(iban, "ING", &bank));
    EranosAccount *card = NULL;
    must(eranosAccountCreate(username, "Revolut", &card));
    EranosAccountList *const accounts = eranosAccountListCreate();
    eranosAccountListPush(accounts, bank);
    eranosAccountListPush(accounts, card);
    eranosMemberSetAccounts(member, accounts);
    must(eranosMemberSetEmail(member, "ana@example.com"));
    must(eranosMemberSetLastName(member, "Popescu"));
    must(eranosMemberSetPhone(member, "+40 712 345 678"));
    EranosText *const text = eranosTextCreate();
    must(eranosGroupAddMember(group, member, text));
    Id const ana = idOf(text);
    eranosTextDestroy(text);
    eranosAccountListDestroy(accounts);
    eranosAccountDestroy(card);
    eranosAccountDestroy(bank);
    eranosAccountDetailsDestroy(username);
    eranosAccountDetailsDestroy(iban);
    eranosMemberDestroy(member);
    return ana;
}

/// Adds the members of the trip to @p group, edits Bob, and prints them and what validation accepts.
static People members(EranosGroup *const group) {
    section("2. Members");
    Id const ana = addAna(group);
    Id const bob = addMember(group, "Bob");
    Id const carol = addMember(group, "Carol");
    Id const dan = addMember(group, "Dan");
    Id const eva = addMember(group, "Eva");
    Id const filip = addMember(group, "Filip");
    // An edit starts from a copy of the member, which keeps its identifier.
    EranosMember *stored = NULL;
    must(eranosGroupMember(group, bob.text, &stored));
    check(NULL != stored, "Bob is a member");
    must(eranosMemberSetEmail(stored, "bob@example.com"));
    must(eranosMemberSetLastName(stored, "Ionescu"));
    EranosMember *before = NULL;
    must(eranosGroupReplaceMember(group, stored, &before));
    printMembers(group);
    printFormatted("Only a first name is required. Ana gave every detail; Bob added his last name and\n"
                   "email later, through an edit that kept his identifier:\n"
                   "  %s\n"
                   "Eva is Carol's daughter, and Filip was added by mistake.",
                   bob.text);
    validation(group);

    EranosText *const text = eranosTextCreate();
    check(!eranosMemberLastName(before, text), "Bob had no last name");
    EranosMember *edited = NULL;
    must(eranosGroupMember(group, bob.text, &edited));
    check(eranosMemberLastName(edited, text) && 0 == strcmp(eranosTextData(text), "Ionescu"), "Bob has his last name");
    EranosMember *first = NULL;
    must(eranosGroupMember(group, ana.text, &first));
    EranosAccountList *const accounts = eranosMemberAccounts(first);
    check(2 == eranosAccountListLength(accounts), "Ana has two accounts");
    eranosAccountListDestroy(accounts);
    eranosMemberDestroy(first);
    eranosMemberDestroy(edited);
    eranosTextDestroy(text);
    eranosMemberDestroy(before);
    eranosMemberDestroy(stored);
    return (People){.ana = ana, .bob = bob, .carol = carol, .dan = dan, .eva = eva, .filip = filip};
}

// 3. Categories.

/// Adds the category @p name to @p group.
static Id addCategory(EranosGroup *const group, char const *const name) {
    EranosCategory *category = NULL;
    must(eranosCategoryCreate(name, &category));
    EranosText *const text = eranosTextCreate();
    must(eranosGroupAddCategory(group, category, text));
    Id const id = idOf(text);
    eranosTextDestroy(text);
    eranosCategoryDestroy(category);
    return id;
}

/// What @p group refuses the category @p name with.
static EranosError *categoryRefusal(EranosGroup *const group, char const *const name) {
    EranosCategory *category = NULL;
    must(eranosCategoryCreate(name, &category));
    EranosText *const text = eranosTextCreate();
    EranosError *const refusal = eranosGroupAddCategory(group, category, text);
    eranosTextDestroy(text);
    eranosCategoryDestroy(category);
    return refusal;
}

/// What @p group does when @p category is renamed @p name: NULL when it renames it, or its refusal.
static EranosError *renamed(EranosGroup *const group, Id const category, char const *const name) {
    EranosCategory *stored = NULL;
    must(eranosGroupCategory(group, category.text, &stored));
    check(NULL != stored, "the group holds the category");
    must(eranosCategorySetName(stored, name));
    EranosCategory *previous = NULL;
    EranosError *const refusal = eranosGroupReplaceCategory(group, stored, &previous);
    eranosCategoryDestroy(previous);
    eranosCategoryDestroy(stored);
    return refusal;
}

/// The names of every category of @p group, in identifier order, joined by commas.
static Line categoryNames(EranosGroup const *const group) {
    EranosCategoryList *categories = NULL;
    must(eranosGroupCategories(group, &categories));
    EranosText *const text = eranosTextCreate();
    Line line = {{0}};
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

/// Whether @p error names @p category as the one with the name it refused.
static bool namesExisting(EranosError const *const error, Id const category) {
    EranosText *const text = eranosTextCreate();
    bool const same = eranosErrorExisting(error, text) && 0 == strcmp(eranosTextData(text), category.text);
    eranosTextDestroy(text);
    return same;
}

/// Adds Food, Lodging, and Transport to @p group, renames Transport, and shows what renaming refuses.
static Categories addCategories(EranosGroup *const group) {
    section("3. Categories");
    Id const food = addCategory(group, "Food");
    Id const lodging = addCategory(group, "Lodging");
    Id const transport = addCategory(group, "Transport");
    EranosError *const homonym = categoryRefusal(group, "FOOD");
    must(renamed(group, transport, "Getting around"));
    Id const uncategorised = builtInCategory(group);
    EranosError *const builtIn = renamed(group, uncategorised, "Other");
    Line const names = categoryNames(group);
    printFormatted("Every group has the built-in Uncategorised; this one adds Food, Lodging, and\n"
                   "Transport, then renames Transport, which keeps its identifier. In identifier order:\n"
                   "  %s\n"
                   "Names are compared caselessly, and the built-in category keeps its name:",
                   names.text);
    check(isFailure(homonym, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_DUPLICATE_CATEGORY_NAME) && namesExisting(homonym, food),
          "a caseless homonym is refused");
    check(isFailure(builtIn, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_BUILT_IN_CATEGORY), "the built-in category keeps its name");
    Table *const table = tableCreate(2, (char const *const[]){"Change", "Refused with"}, 2);
    tableAdd(table, (Line const[]){lineOf("Add a category named FOOD"), messageLine(homonym)});
    tableAdd(table, (Line const[]){lineOf("Rename Uncategorised to Other"), messageLine(builtIn)});
    tablePrint(table);

    Line const name = categoryName(group, transport.text);
    check(0 == strcmp(name.text, "Getting around"), "Transport is renamed");
    return (Categories){.food = food, .lodging = lodging, .transport = transport};
}

// 4. Expenses, one for every way of dividing.

/// How a whole expense's amount divides, in a word or two.
static char const *divided(EranosSplit const *const split) {
    switch (eranosSplitKind(split)) {
    case ERANOS_SPLIT_KIND_AMOUNTS:
        return "by amounts";
    case ERANOS_SPLIT_KIND_EQUAL:
        return "equally";
    case ERANOS_SPLIT_KIND_PARTS:
        return "by parts";
    case ERANOS_SPLIT_KIND_PERCENTAGES:
        return "by percentage";
    }
}

/// The tip of @p expense, as ", 10% tip" or ", 5.00 EUR tip", or nothing when it has none.
static Line tipOf(EranosExpense const *const expense) {
    EranosMoney *amount = NULL;
    must(eranosExpenseTipAmount(expense, &amount));
    EranosTip *const tip = eranosExpenseTip(expense);
    Line line = {{0}};
    if (0 != eranosMoneyMinorUnits(amount) && NULL != tip) {
        EranosTipValue *const value = eranosTipValue(tip);
        int32_t basisPoints = 0;
        if (eranosTipValueBasisPoints(value, &basisPoints)) {
            EranosText *const text = eranosTextCreate();
            must(eranosPercentageFormat(basisPoints, text));
            line = lineOf(", %s tip", eranosTextData(text));
            eranosTextDestroy(text);
        } else {
            Line const fixed = moneyLine(eranosTipValueAmount(value));
            line = lineOf(", %s tip", fixed.text);
        }
        eranosTipValueDestroy(value);
    }
    eranosTipDestroy(tip);
    eranosMoneyDestroy(amount);
    return line;
}

/// How @p expense divides, in a few words.
static Line how(EranosExpense const *const expense) {
    EranosExpenseSplit *const split = eranosExpenseSplit(expense);
    EranosItemList *const items = eranosExpenseSplitItems(split);
    EranosSplit *const whole = eranosExpenseSplitSplit(split);
    Line const kind = NULL == items ? lineOf("%s", divided(whole)) : lineOf("%zu items", eranosItemListLength(items));
    EranosSplit *const payers = eranosExpensePayers(expense);
    size_t const count = splitMembers(payers).count;
    Line const paid = 1 == count ? lineOf("%s", "") : lineOf(", %zu payers", count);
    Line const tip = tipOf(expense);
    eranosSplitDestroy(payers);
    eranosSplitDestroy(whole);
    eranosItemListDestroy(items);
    eranosExpenseSplitDestroy(split);
    return lineOf("%s%s%s", kind.text, paid.text, tip.text);
}

/// Adds to @p table the row of the expense @p id: every involved member's payment and consumption.
static void addShares(EranosGroup const *const group, Table *const table, Id const id) {
    EranosExpense *expense = NULL;
    must(eranosGroupExpense(group, id.text, &expense));
    EranosExpenseShares *shares = NULL;
    must(eranosGroupExpenseShares(group, id.text, &shares));
    check(NULL != expense && NULL != shares, "the group holds the expense");
    EranosMemberShareList *const members = eranosExpenseSharesMembers(shares);
    EranosText *const text = eranosTextCreate();
    Cell names = {.lines = 0, .text = {{0}}};
    Cell paid = {.lines = 0, .text = {{0}}};
    Cell consumed = {.lines = 0, .text = {{0}}};
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
    tableAdd(table, (Line const[]){textLine(text), how(expense), names.text, paid.text, consumed.text});
    eranosTextDestroy(text);
    eranosMemberShareListDestroy(members);
    eranosExpenseSharesDestroy(shares);
    eranosExpenseDestroy(expense);
}

/// Prints the receipt of the expense @p id of @p group: who shared each line, its quantity, its unit price, and its total.
static void printItems(EranosGroup const *const group, Id const id) {
    EranosExpense *expense = NULL;
    must(eranosGroupExpense(group, id.text, &expense));
    check(NULL != expense, "the group holds the expense");
    EranosExpenseSplit *const split = eranosExpenseSplit(expense);
    EranosItemList *const items = eranosExpenseSplitItems(split);
    check(NULL != items, "the expense is itemised");
    Table *const table = tableCreate(5, (char const *const[]){"Item", "Shared by", "Quantity", "Unit price", "Line total"}, 2);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosItemListLength(items) > index; ++index) {
        EranosItem *const item = eranosItemListGet(items, index);
        EranosSplit *const itemSplit = eranosItemSplit(item);
        Members const sharedBy = splitMembers(itemSplit);
        EranosMoney *const unitPrice = eranosItemUnitPrice(item);
        int32_t const quantity = eranosItemQuantity(item);
        eranosMoneyCurrency(unitPrice, text);
        EranosMoney *total = NULL;
        must(eranosMoneyCreate(eranosTextData(text), eranosMoneyMinorUnits(unitPrice) * quantity, &total));
        eranosItemName(item, text);
        tableAdd(table, (Line const[]){
                            textLine(text),
                            names(group, &sharedBy),
                            lineOf("%ld", (long)quantity),
                            moneyLine(unitPrice),
                            moneyLine(total),
                        });
        eranosSplitDestroy(itemSplit);
        eranosItemDestroy(item);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosItemListDestroy(items);
    eranosExpenseSplitDestroy(split);
    eranosExpenseDestroy(expense);
}

/// Adds the line @p name of a receipt to @p items: @p quantity of @p unitPrice euros, which @p split shares; destroys
/// @p split.
static void addItem(EranosItemList *const items, char const *const name, EranosSplit *const split, char const *const unitPrice,
                    int32_t const quantity) {
    EranosMoney *const price = eur(unitPrice);
    EranosItem *item = NULL;
    must(eranosItemCreate(name, split, price, &item));
    eranosItemSetQuantity(item, quantity);
    eranosItemListPush(items, item);
    eranosItemDestroy(item);
    eranosMoneyDestroy(price);
    eranosSplitDestroy(split);
}

/// The receipt of the fado night: every line with its own split.
static EranosItemList *fadoItems(People const *const people) {
    EranosItemList *const items = eranosItemListCreate();
    Members const wine = {.count = 3, .ids = {people->ana, people->bob, people->dan}};
    addItem(items, "Vinho verde", equalSplit(&wine), "12.00", 2);
    addItem(items, "Bacalhau", soleSplit(people->carol), "18.00", 1);
    Share const tarts[] = {
        {.member = people->ana, .value = "1"}, {.member = people->bob, .value = "1"}, {.member = people->carol, .value = "1"},
        {.member = people->dan, .value = "1"}, {.member = people->eva, .value = "2"},
    };
    addItem(items, "Custard tarts", partsSplit(tarts, 5), "1.50", 6);
    addItem(items, "Juice", soleSplit(people->eva), "3.00", 1);
    return items;
}

/// Gives @p expense a tip of @p value, shared by @p distribution; destroys @p value.
static void tipWith(EranosExpense *const expense, EranosTipValue *const value, EranosTipDistribution const distribution) {
    EranosTip *const tip = eranosTipCreate(value, distribution);
    eranosExpenseSetTip(expense, tip);
    eranosTipDestroy(tip);
    eranosTipValueDestroy(value);
}

/// The percentage @p text as a tip's value.
static EranosTipValue *percentageTip(char const *const text) {
    int32_t basisPoints = 0;
    must(eranosPercentageParse(text, &basisPoints));
    return eranosTipValueCreatePercentage(basisPoints);
}

/// The fado night: itemised, paid by two, at a time and a place, with a tip of 10 %.
static EranosExpense *fadoNight(People const *const people, Categories const *const categories) {
    Share const cards[] = {
        {.member = people->carol, .value = "39.40"},
        {.member = people->dan, .value = "20.00"},
    };
    EranosCost *const cost = localCost(eur("54.00"));
    EranosSplit *const payers = amountsSplit("EUR", cards, 2);
    EranosItemList *const items = fadoItems(people);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateItemised(items);
    EranosExpense *expense = NULL;
    must(eranosExpenseCreate(cost, october(4), "Fado night", payers, split, &expense));
    eranosExpenseSplitDestroy(split);
    eranosItemListDestroy(items);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    must(eranosExpenseSetCategory(expense, categories->food.text));
    must(eranosExpenseSetDescription(expense, "Tasca do Chico"));
    eranosExpenseSetTime(expense, (EranosTimeOfDay){.hour = 22, .minute = 15});
    tipWith(expense, percentageTip("10"), ERANOS_TIP_DISTRIBUTION_PROPORTIONAL);
    return expense;
}

/// @p expense in @p category; the expense comes back for the caller to add.
static EranosExpense *inCategory(EranosExpense *const expense, Id const category) {
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
static Expenses everyWay(EranosGroup *const group, People const *const people, Categories const *const categories) {
    Members const thirds = {.count = 3, .ids = {people->ana, people->bob, people->carol}};
    Id const groceries =
        addExpense(group, inCategory(expenseOf(localCost(eur("10.00")), october(1), "Groceries", soleSplit(people->ana), equalSplit(&thirds)),
                                     categories->food));
    Share const rooms[] = {
        {.member = people->ana, .value = "25"},
        {.member = people->bob, .value = "25"},
        {.member = people->carol, .value = "35"},
        {.member = people->dan, .value = "15"},
    };
    Id const apartment =
        addExpense(group, inCategory(expenseOf(localCost(eur("800.00")), october(1), "Apartment", soleSplit(people->ana), percentagesSplit(rooms, 4)),
                                     categories->lodging));
    Share const portions[] = {
        {.member = people->ana, .value = "1"}, {.member = people->bob, .value = "1"},   {.member = people->carol, .value = "1"},
        {.member = people->dan, .value = "1"}, {.member = people->eva, .value = "0.5"},
    };
    Id const seafood = addExpense(
        group, inCategory(expenseOf(localCost(eur("90.00")), october(2), "Seafood dinner", soleSplit(people->carol), partsSplit(portions, 5)),
                          categories->food));
    Share const tickets[] = {
        {.member = people->ana, .value = "24.50"},
        {.member = people->bob, .value = "24.50"},
        {.member = people->carol, .value = "24.50"},
        {.member = people->eva, .value = "15.00"},
    };
    Id const train = addExpense(
        group, inCategory(expenseOf(localCost(eur("88.50")), october(3), "Train to Porto", soleSplit(people->bob), amountsSplit("EUR", tickets, 4)),
                          categories->transport));
    Members const cards = {.count = 2, .ids = {people->ana, people->bob}};
    Members const visitors = {.count = 4, .ids = {people->ana, people->bob, people->carol, people->dan}};
    Id const museum = addExpense(group, expenseOf(localCost(eur("60.00")), october(3), "Museum", equalSplit(&cards), equalSplit(&visitors)));
    Id const fado = addExpense(group, fadoNight(people, categories));
    Members const everyone = {.count = 5, .ids = {people->ana, people->bob, people->carol, people->dan, people->eva}};
    EranosExpense *const ride = inCategory(
        expenseOf(localCost(eur("45.00")), october(5), "Tuk-tuk tour", soleSplit(people->dan), equalSplit(&everyone)), categories->transport);
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
static EranosMemberShare *shareOf(EranosGroup const *const group, Id const expense, Id const member) {
    EranosExpenseShares *shares = NULL;
    must(eranosGroupExpenseShares(group, expense.text, &shares));
    check(NULL != shares, "the group holds the expense");
    EranosMemberShareList *const members = eranosExpenseSharesMembers(shares);
    EranosText *const text = eranosTextCreate();
    EranosMemberShare *found = NULL;
    for (size_t index = 0; NULL == found && eranosMemberShareListLength(members) > index; ++index) {
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
    check(NULL != found, "the member shares the expense");
    return found;
}

/// What @p member consumed of the expense @p expense of @p group, in minor units.
static int64_t consumedOf(EranosGroup const *const group, Id const expense, Id const member) {
    EranosMemberShare *const share = shareOf(group, expense, member);
    EranosMoney *const amount = eranosMemberShareShare(share);
    int64_t const minorUnits = eranosMoneyMinorUnits(amount);
    eranosMoneyDestroy(amount);
    eranosMemberShareDestroy(share);
    return minorUnits;
}

/// Prints how the fado night's tip was shared, and its receipt.
static void printFado(EranosGroup const *const group, Id const id) {
    EranosExpense *fado = NULL;
    must(eranosGroupExpense(group, id.text, &fado));
    check(NULL != fado, "the fado night was added");
    EranosText *const text = eranosTextCreate();
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    Line when = {{0}};
    if (eranosExpenseTime(fado, &time)) {
        must(eranosTimeOfDayFormat(time, text));
        when = textLine(text);
    }
    must(eranosCalendarDateFormat(eranosExpenseDate(fado), text));
    Line const date = textLine(text);
    Line const place = eranosExpenseDescription(fado, text) ? textLine(text) : lineOf("%s", "");
    EranosTip *const tip = eranosExpenseTip(fado);
    check(NULL != tip, "the fado night has a tip");
    EranosTipValue *const value = eranosTipValue(tip);
    int32_t basisPoints = 0;
    Line rule = {{0}};
    if (eranosTipValueBasisPoints(value, &basisPoints)) {
        must(eranosPercentageFormat(basisPoints, text));
        rule = textLine(text);
    } else {
        rule = moneyLine(eranosTipValueAmount(value));
    }
    char const *const shared = ERANOS_TIP_DISTRIBUTION_EQUAL == eranosTipDistribution(tip) ? "equally" : "in proportion to what each member ordered";
    EranosMoney *amount = NULL;
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
    eranosTipDestroy(tip);
    eranosTextDestroy(text);
    eranosExpenseDestroy(fado);
}

/// Checks the key numbers of the seven expenses of @p group.
static void checkExpenses(EranosGroup const *const group, People const *const people, Expenses const *const expenses) {
    int64_t const thirds[] = {
        consumedOf(group, expenses->groceries, people->ana),
        consumedOf(group, expenses->groceries, people->bob),
        consumedOf(group, expenses->groceries, people->carol),
    };
    size_t const leastParts = (size_t)(333 == thirds[0]) + (size_t)(333 == thirds[1]) + (size_t)(333 == thirds[2]);
    check(2 == leastParts && 1000 == thirds[0] + thirds[1] + thirds[2], "the groceries divide into 3.33, 3.33, and 3.34");
    check(28000 == consumedOf(group, expenses->apartment, people->carol), "Carol's 35 % of the apartment is 280.00 EUR");
    check(1000 == consumedOf(group, expenses->seafood, people->eva), "Eva's half part of the dinner is 10.00 EUR");
    check(1500 == consumedOf(group, expenses->train, people->eva), "Eva's ticket is 15.00 EUR");
    EranosMemberShare *const bob = shareOf(group, expenses->museum, people->bob);
    check(isMinorUnits(eranosMemberSharePaid(bob), 3000), "Bob paid 30.00 EUR of the museum");
    eranosMemberShareDestroy(bob);
    check(1500 == consumedOf(group, expenses->museum, people->dan), "Dan's quarter of the museum is 15.00 EUR");
    EranosExpense *museum = NULL;
    must(eranosGroupExpense(group, expenses->museum.text, &museum));
    EranosTextList *involved = NULL;
    must(eranosExpenseParticipants(museum, &involved));
    check(4 == eranosTextListLength(involved), "the museum names four members");
    eranosTextListDestroy(involved);
    eranosExpenseDestroy(museum);
    EranosExpense *fado = NULL;
    must(eranosGroupExpense(group, expenses->fado.text, &fado));
    EranosMoney *total = NULL;
    must(eranosExpenseTotal(fado, &total));
    check(isMinorUnits(total, 5940), "the fado night comes to 59.40 EUR");
    EranosMoney *tip = NULL;
    must(eranosExpenseTipAmount(fado, &tip));
    check(isMinorUnits(tip, 540), "its tip is 5.40 EUR");
    eranosExpenseDestroy(fado);
    check(2145 == consumedOf(group, expenses->fado, people->carol), "Carol's part of the fado night is 21.45 EUR");
    check(660 == consumedOf(group, expenses->fado, people->eva), "Eva's part of the fado night is 6.60 EUR");
    check(1000 == consumedOf(group, expenses->tour, people->eva), "Eva's part of the tour is 10.00 EUR");
}

/// Adds the seven expenses to @p group and prints how each divides; returns the seafood dinner's identifier.
static Id expenses(EranosGroup *const group, People const *const people, Categories const *const categories) {
    section("4. Expenses, one for every way of dividing");
    Expenses const added = everyWay(group, people, categories);
    printLine("Seven expenses cover every way of dividing one: equally, by percentage, by parts\n"
              "(Eva counts as half), by exact amounts, with two payers, item by item, and with a\n"
              "tip. Every member's consumption adds up to the expense's total, to the cent:");
    Table *const table = tableCreate(5, (char const *const[]){"Expense", "Split", "Member", "Paid", "Consumed"}, 3);
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
static void anotherCurrency(EranosGroup *const group, People const *const people, Categories const *const categories) {
    section("5. Another currency");
    Members const both = {.count = 2, .ids = {people->ana, people->bob}};
    Id const parking = addExpense(
        group, inCategory(expenseOf(convertedCost(ron("250.00"), "0.2"), october(6), "Airport parking", soleSplit(people->ana), equalSplit(&both)),
                          categories->transport));
    EranosTransfer *const repayment = transferOf(convertedCost(ron("100.00"), "0.2"), october(6), people->bob, people->ana);
    must(eranosTransferSetNote(repayment, "Cash for the parking"));
    eranosTransferSetTime(repayment, (EranosTimeOfDay){.hour = 7, .minute = 40});
    Id const cash = addTransfer(group, repayment);
    printLine("Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in\n"
              "cash. An entry in another currency carries the rate into the group currency, how\n"
              "many EUR one RON is worth, and is converted once, when it is entered:");
    printEntries(group, "EUR");
    EranosTransfer *transfer = NULL;
    must(eranosGroupTransfer(group, cash.text, &transfer));
    check(NULL != transfer, "the cash was recorded");
    EranosText *const text = eranosTextCreate();
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    Line when = {{0}};
    if (eranosTransferTime(transfer, &time)) {
        must(eranosTimeOfDayFormat(time, text));
        when = textLine(text);
    }
    Line const note = eranosTransferNote(transfer, text) ? textLine(text) : lineOf("%s", "");
    EranosMoney *counted = NULL;
    must(eranosGroupTransferAmount(group, cash.text, &counted));
    Line const counts = NULL == counted ? zeroOf(group) : moneyLine(counted);
    EranosMoney *const parked = ron("250.00");
    EranosMoney *converted = NULL;
    must(eranosExchangeRateConvert("0.2011", parked, "EUR", &converted));
    eranosMoneyDestroy(parked);
    int64_t const convertedUnits = eranosMoneyMinorUnits(converted);
    Line const becomes = moneyLine(converted);
    printFormatted("Bob's transfer, at %s (\"%s\"), counts as %s.\n"
                   "A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,\n"
                   "which becomes %s.",
                   when.text, note.text, counts.text, becomes.text);

    check(5028 == convertedUnits, "the conversion rounds to 50.28 EUR");
    EranosExpenseShares *shares = NULL;
    must(eranosGroupExpenseShares(group, parking.text, &shares));
    check(NULL != shares && isMinorUnits(eranosExpenseSharesTotal(shares), 5000), "the parking counts as 50.00 EUR");
    EranosMoney *amount = NULL;
    must(eranosGroupTransferAmount(group, cash.text, &amount));
    check(isMinorUnits(amount, 2000), "the cash counts as 20.00 EUR");
    EranosCost *const cost = eranosTransferCost(transfer);
    check(ERANOS_COST_KIND_CONVERTED == eranosCostKind(cost), "the cash was converted");
    eranosCostDestroy(cost);
    eranosExpenseSharesDestroy(shares);
    eranosTextDestroy(text);
    eranosTransferDestroy(transfer);
}

// 6. Edits.

/// The edits of section 6, each an entry and what happened to it.
typedef struct Changes {
    /// The coffee, entered twice.
    Line coffee;
    /// What the coffee's edit did.
    Line coffeeEdit;
    /// The seafood dinner.
    Line dinner;
    /// What the dinner's edit did.
    Line dinnerEdit;
    /// Carol's transfer to the wrong member.
    Line mistake;
    /// What undoing it did.
    Line mistakeEdit;
    /// Dan's transfer for the apartment.
    Line rent;
    /// What its edit did.
    Line rentEdit;
} Changes;

/// The coffee that Bob entered twice.
static EranosExpense *coffeeOf(People const *const people, Categories const *const categories) {
    Members const drinkers = {.count = 3, .ids = {people->ana, people->bob, people->carol}};
    return inCategory(expenseOf(localCost(eur("6.00")), october(5), "Coffee", soleSplit(people->bob), equalSplit(&drinkers)), categories->food);
}

/// The dinner's receipt said 99.00: replaces its amount and gives its place.
static void correctDinner(EranosGroup *const group, Id const seafood, Changes *const changes) {
    EranosExpense *dinner = NULL;
    must(eranosGroupExpense(group, seafood.text, &dinner));
    check(NULL != dinner, "the dinner was added");
    EranosCost *const cost = localCost(eur("99.00"));
    eranosExpenseSetCost(dinner, cost);
    eranosCostDestroy(cost);
    must(eranosExpenseSetDescription(dinner, "Cervejaria Ramiro"));
    EranosExpense *previous = NULL;
    must(eranosGroupReplaceExpense(group, dinner, &previous));
    EranosExpense *corrected = NULL;
    must(eranosGroupExpense(group, seafood.text, &corrected));
    EranosText *const text = eranosTextCreate();
    eranosExpenseName(previous, text);
    changes->dinner = textLine(text);
    Line const place = eranosExpenseDescription(corrected, text) ? textLine(text) : lineOf("%s", "");
    Line const before = costAmount(eranosExpenseCost(previous));
    Line const after = costAmount(eranosExpenseCost(corrected));
    changes->dinnerEdit = lineOf("%s became %s, at %s", before.text, after.text, place.text);
    EranosCost *const previousCost = eranosExpenseCost(previous);
    check(isMinorUnits(eranosCostAmount(previousCost), 9000), "the dinner was 90.00 EUR");
    eranosCostDestroy(previousCost);
    eranosTextDestroy(text);
    eranosExpenseDestroy(corrected);
    eranosExpenseDestroy(previous);
    eranosExpenseDestroy(dinner);
}

/// Bob entered the coffee twice: removes the copy.
static void removeCopy(EranosGroup *const group, People const *const people, Categories const *const categories, Changes *const changes) {
    addExpense(group, coffeeOf(people, categories));
    Id const copy = addExpense(group, coffeeOf(people, categories));
    EranosExpense *removed = NULL;
    must(eranosGroupRemoveExpense(group, copy.text, &removed));
    EranosText *const text = eranosTextCreate();
    eranosExpenseName(removed, text);
    changes->coffee = textLine(text);
    Line const amount = costAmount(eranosExpenseCost(removed));
    changes->coffeeEdit = lineOf("entered twice; the copy of %s was removed", amount.text);
    EranosExpense *gone = NULL;
    must(eranosGroupExpense(group, copy.text, &gone));
    check(NULL == gone, "the copy is gone");
    eranosTextDestroy(text);
    eranosExpenseDestroy(removed);
}

/// Dan's part of the apartment was 120.00, not 100.00: replaces his transfer's amount and notes what it was for.
static void correctTransfer(EranosGroup *const group, People const *const people, Changes *const changes) {
    Id const first = addTransfer(group, transferOf(localCost(eur("100.00")), october(5), people->dan, people->ana));
    EranosTransfer *stored = NULL;
    must(eranosGroupTransfer(group, first.text, &stored));
    check(NULL != stored, "the transfer was added");
    EranosCost *const cost = localCost(eur("120.00"));
    eranosTransferSetCost(stored, cost);
    eranosCostDestroy(cost);
    must(eranosTransferSetNote(stored, "Apartment"));
    EranosTransfer *replaced = NULL;
    must(eranosGroupReplaceTransfer(group, stored, &replaced));
    EranosTransfer *apartment = NULL;
    must(eranosGroupTransfer(group, first.text, &apartment));
    Line const dan = firstName(group, people->dan.text);
    Line const ana = firstName(group, people->ana.text);
    changes->rent = lineOf("%s to %s", dan.text, ana.text);
    EranosText *const text = eranosTextCreate();
    Line const note = eranosTransferNote(apartment, text) ? textLine(text) : lineOf("%s", "");
    Line const before = costAmount(eranosTransferCost(replaced));
    Line const after = costAmount(eranosTransferCost(apartment));
    changes->rentEdit = lineOf("%s became %s, noted %s", before.text, after.text, note.text);
    EranosMoney *amount = NULL;
    must(eranosGroupTransferAmount(group, first.text, &amount));
    check(isMinorUnits(amount, 12000), "Dan's transfer is 120.00 EUR");
    eranosTextDestroy(text);
    eranosTransferDestroy(apartment);
    eranosTransferDestroy(replaced);
    eranosTransferDestroy(stored);
}

/// Carol sent 10.00 to the wrong member: removes the transfer.
static void undoMistake(EranosGroup *const group, People const *const people, Changes *const changes) {
    Id const mistake = addTransfer(group, transferOf(localCost(eur("10.00")), october(5), people->carol, people->bob));
    EranosTransfer *undone = NULL;
    must(eranosGroupRemoveTransfer(group, mistake.text, &undone));
    Line const carol = firstName(group, people->carol.text);
    Line const bob = firstName(group, people->bob.text);
    changes->mistake = lineOf("%s to %s", carol.text, bob.text);
    Line const amount = costAmount(eranosTransferCost(undone));
    changes->mistakeEdit = lineOf("%s sent to the wrong member; removed", amount.text);
    EranosTransfer *gone = NULL;
    must(eranosGroupTransfer(group, mistake.text, &gone));
    check(NULL == gone, "the mistake is gone");
    eranosTransferDestroy(undone);
}

/// Whether @p error is the currency @p currency, as the currency it names.
static bool namesCurrency(EranosError const *const error, char const *const currency) {
    EranosText *const text = eranosTextCreate();
    bool const same = eranosErrorCurrency(error, text) && 0 == strcmp(eranosTextData(text), currency);
    eranosTextDestroy(text);
    return same;
}

/// Replaces an expense and a transfer and removes one of each, then shows that a change that does not fit is refused.
static void edits(EranosGroup *const group, People const *const people, Categories const *const categories, Id const seafood) {
    section("6. Edits");
    Changes changes;
    correctDinner(group, seafood, &changes);
    removeCopy(group, people, categories, &changes);
    correctTransfer(group, people, &changes);
    undoMistake(group, people, &changes);
    printLine("Entries are values: an edit starts from a copy made with to_builder, keeps the\n"
              "identifier, and replaces the entry; a removal takes it out with its totals.");
    Table *const table = tableCreate(2, (char const *const[]){"Entry", "Edit"}, 2);
    tableAdd(table, (Line const[]){changes.dinner, changes.dinnerEdit});
    tableAdd(table, (Line const[]){changes.coffee, changes.coffeeEdit});
    tableAdd(table, (Line const[]){changes.rent, changes.rentEdit});
    tableAdd(table, (Line const[]){changes.mistake, changes.mistakeEdit});
    tablePrint(table);
    printLine("The balances after the edits:");
    printBalances(group);

    EranosSummary *const before = summaryOf(group);
    Share const halves[] = {
        {.member = people->ana, .value = "50"},
        {.member = people->bob, .value = "49.99"},
    };
    EranosError *const unbalanced =
        expenseRefusal(group, expenseOf(localCost(eur("30.00")), october(6), "Taxi", soleSplit(people->ana), percentagesSplit(halves, 2)));
    EranosTransfer *const toHerself = transferOf(localCost(eur("5.00")), october(6), people->ana, people->ana);
    EranosText *const text = eranosTextCreate();
    EranosError *const sameMember = eranosGroupAddTransfer(group, toHerself, text);
    eranosTransferDestroy(toHerself);
    EranosError *const unrated =
        expenseRefusal(group, expenseOf(localCost(ron("40.00")), october(6), "Souvenirs", soleSplit(people->ana), soleSplit(people->ana)));
    printLine("A change that does not fit is refused, and the group stays as it was:");
    int64_t total = 0;
    check(isFailure(unbalanced, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_PERCENTAGES_DO_NOT_ADD_UP) &&
              eranosErrorTotalBasisPoints(unbalanced, &total) && 9999 == total,
          "percentages that do not add up are refused");
    check(isFailure(sameMember, ERANOS_ERROR_KIND_TRANSFER, ERANOS_FAILURE_TRANSFER_SAME_MEMBER), "a transfer to oneself is refused");
    check(isFailure(unrated, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_MISSING_EXCHANGE_RATE) && namesCurrency(unrated, "RON"),
          "an amount in RON without a rate is refused");
    Table *const refusals = tableCreate(2, (char const *const[]){"Change", "Refused with"}, 2);
    tableAdd(refusals, (Line const[]){lineOf("A taxi split 50%% and 49.99%%"), messageLine(unbalanced)});
    tableAdd(refusals, (Line const[]){lineOf("Ana paying herself back"), messageLine(sameMember)});
    tableAdd(refusals, (Line const[]){lineOf("Souvenirs in RON, without a rate"), messageLine(unrated)});
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
static void comingsAndGoings(EranosGroup *const group, People const *const people) {
    section("7. Members come and go");
    EranosMember *removed = NULL;
    must(eranosGroupRemoveMember(group, people->filip.text, &removed));
    must(eranosGroupArchiveMember(group, people->dan.text));
    Members const diners = {.count = 4, .ids = {people->ana, people->bob, people->carol, people->dan}};
    EranosError *const farewell =
        expenseRefusal(group, expenseOf(localCost(eur("40.00")), october(6), "Farewell dinner", soleSplit(people->ana), equalSplit(&diners)));
    EranosMember *kept = NULL;
    EranosError *const inUse = eranosGroupRemoveMember(group, people->dan.text, &kept);
    EranosText *const text = eranosTextCreate();
    eranosMemberFirstName(removed, text);
    Line const removedName = textLine(text);
    printFormatted("%s was never part of an entry, so removing him leaves no trace. Dan flew home\n"
                   "early and was archived: his history and his balance stay, but a new expense\n"
                   "cannot include him, and he cannot be removed while entries reference him.",
                   removedName.text);
    EranosSummary *const summary = summaryOf(group);
    EranosMemberList *members = NULL;
    must(eranosGroupMembers(group, &members));
    Table *const table = tableCreate(3, (char const *const[]){"Member", "Status", "Balance"}, 2);
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        bool archived = false;
        must(eranosGroupIsArchived(group, eranosTextData(text), &archived));
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        if (NULL != found) {
            eranosMemberFirstName(member, text);
            tableAdd(table,
                     (Line const[]){textLine(text), lineOf("%s", archived ? "archived" : "active"), balanceLine(eranosMemberSummaryBalance(found))});
        }
        eranosMemberSummaryDestroy(found);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
    bool const namesDan = eranosErrorMember(farewell, text) && 0 == strcmp(eranosTextData(text), people->dan.text);
    check(isFailure(farewell, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_ARCHIVED_MEMBER) && namesDan, "an archived member joins no expense");
    check(isFailure(inUse, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_MEMBER_IN_USE) && NULL == kept, "a referenced member stays");
    Table *const refusals = tableCreate(2, (char const *const[]){"Change", "Refused with"}, 2);
    tableAdd(refusals, (Line const[]){lineOf("A farewell dinner with Dan"), messageLine(farewell)});
    tableAdd(refusals, (Line const[]){lineOf("Removing Dan"), messageLine(inUse)});
    tablePrint(refusals);
    must(eranosGroupUnarchiveMember(group, people->dan.text));
    printLine("When Dan joins the next trip, unarchiving him makes him active again.");

    check(0 == strcmp(removedName.text, "Filip"), "Filip was removed");
    EranosMember *gone = NULL;
    must(eranosGroupMember(group, people->filip.text, &gone));
    check(NULL == gone, "Filip is gone");
    bool archived = true;
    must(eranosGroupIsArchived(group, people->dan.text, &archived));
    check(!archived, "Dan is active again");
    eranosTextDestroy(text);
    eranosMemberDestroy(removed);
}

// 8. The summary.

/// The totals per category of @p member in @p summary, which the caller destroys, or NULL when it lists none.
static EranosCategoryTotalList *byCategory(EranosSummary const *const summary, char const *const member) {
    EranosMemberCategoryTotalsList *const list = eranosSummaryMemberCategoryTotals(summary);
    EranosText *const text = eranosTextCreate();
    EranosCategoryTotalList *found = NULL;
    for (size_t index = 0; NULL == found && eranosMemberCategoryTotalsListLength(list) > index; ++index) {
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

/// The total of @p category among @p totals, which the caller destroys, or NULL when they list none.
static EranosMoney *categoryTotal(EranosCategoryTotalList const *const totals, char const *const category) {
    EranosText *const text = eranosTextCreate();
    EranosMoney *found = NULL;
    for (size_t index = 0; NULL != totals && NULL == found && eranosCategoryTotalListLength(totals) > index; ++index) {
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
static Line categoryLine(EranosCategoryTotalList const *const totals, Id const category, Line const zero) {
    EranosMoney *const total = categoryTotal(totals, category.text);
    return NULL == total ? zero : moneyLine(total);
}

/// Prints each member's consumption per category.
static void printMemberCategories(EranosGroup const *const group, Categories const *const categories) {
    Id const builtIn = builtInCategory(group);
    Line const zero = zeroOf(group);
    EranosSummary *const summary = summaryOf(group);
    EranosMemberList *members = NULL;
    must(eranosGroupMembers(group, &members));
    Table *const table = tableCreate(5, (char const *const[]){"Member", "Food", "Lodging", "Getting around", "Uncategorised"}, 1);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        EranosCategoryTotalList *const own = byCategory(summary, eranosTextData(text));
        eranosMemberFirstName(member, text);
        tableAdd(table, (Line const[]){
                            textLine(text),
                            categoryLine(own, categories->food, zero),
                            categoryLine(own, categories->lodging, zero),
                            categoryLine(own, categories->transport, zero),
                            categoryLine(own, builtIn, zero),
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
static bool totalsAddUp(EranosSummary const *const summary) {
    EranosMemberSummaryList *const members = eranosSummaryMembers(summary);
    EranosText *const text = eranosTextCreate();
    bool addsUp = true;
    int64_t nets = 0;
    for (size_t index = 0; eranosMemberSummaryListLength(members) > index; ++index) {
        EranosMemberSummary *const totals = eranosMemberSummaryListGet(members, index);
        eranosMemberSummaryMember(totals, text);
        EranosCategoryTotalList *const own = byCategory(summary, eranosTextData(text));
        int64_t sum = 0;
        for (size_t category = 0; NULL != own && eranosCategoryTotalListLength(own) > category; ++category) {
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
static void summary(EranosGroup const *const group, People const *const people, Categories const *const categories) {
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
    check(isMinorUnits(eranosSummaryTotalSpent(totals), 122290), "the group spent 1222.90 EUR");
    EranosMemberSummary *const ana = memberTotals(group, people->ana);
    check(isMinorUnits(eranosMemberSummaryPaid(ana), 89000) && isMinorUnits(eranosMemberSummaryReceived(ana), 14000),
          "Ana paid 890.00 EUR and received 140.00 EUR");
    EranosMemberSummary *const bob = memberTotals(group, people->bob);
    check(isMinorUnits(eranosMemberSummarySent(bob), 2000), "Bob sent 20.00 EUR");
    EranosMemberSummary *const dan = memberTotals(group, people->dan);
    check(isBalance(eranosMemberSummaryBalance(dan), ERANOS_BALANCE_KIND_GETS_BACK, 1255), "Dan gets back 12.55 EUR");
    EranosMemberSummary *const eva = memberTotals(group, people->eva);
    check(isBalance(eranosMemberSummaryBalance(eva), ERANOS_BALANCE_KIND_OWES, 4260), "Eva owes 42.60 EUR");
    EranosCategoryTotalList *const categoryTotals = eranosSummaryCategoryTotals(totals);
    Id const builtIn = builtInCategory(group);
    check(isMinorUnits(categoryTotal(categoryTotals, categories->food.text), 17440), "the food came to 174.40 EUR");
    check(isMinorUnits(categoryTotal(categoryTotals, categories->lodging.text), 80000), "the lodging came to 800.00 EUR");
    check(isMinorUnits(categoryTotal(categoryTotals, categories->transport.text), 18850), "getting around came to 188.50 EUR");
    check(isMinorUnits(categoryTotal(categoryTotals, builtIn.text), 6000), "the rest came to 60.00 EUR");
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
static bool anyIn(EranosGroup const *const group, Id const category) {
    EranosExpenseList *expenses = NULL;
    must(eranosGroupExpenses(group, &expenses));
    EranosText *const text = eranosTextCreate();
    bool found = false;
    for (size_t index = 0; eranosExpenseListLength(expenses) > index; ++index) {
        EranosExpense *const expense = eranosExpenseListGet(expenses, index);
        found = found || (eranosExpenseCategory(expense, text) && 0 == strcmp(eranosTextData(text), category.text));
        eranosExpenseDestroy(expense);
    }
    eranosTextDestroy(text);
    eranosExpenseListDestroy(expenses);
    return found;
}

/// Removes Getting around, whose expenses move to Uncategorised, and shows that the built-in category stays.
static void removingACategory(EranosGroup *const group, Categories const *const categories) {
    section("9. Removing a category");
    EranosCategory *removed = NULL;
    must(eranosGroupRemoveCategory(group, categories->transport.text, &removed));
    Id const uncategorised = builtInCategory(group);
    EranosCategory *kept = NULL;
    EranosError *const builtIn = eranosGroupRemoveCategory(group, uncategorised.text, &kept);
    check(isFailure(builtIn, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_BUILT_IN_CATEGORY) && NULL == kept,
          "the built-in category cannot be removed");
    EranosText *const text = eranosTextCreate();
    eranosCategoryName(removed, text);
    Line const message = messageLine(builtIn);
    printFormatted("Removing %s moves its expenses to Uncategorised and the category\n"
                   "totals with them; no balance changes. The built-in category cannot be removed:\n"
                   "  error: %s",
                   eranosTextData(text), message.text);
    printCategories(group);

    EranosCategory *gone = NULL;
    must(eranosGroupCategory(group, categories->transport.text, &gone));
    check(NULL == gone, "Getting around is gone");
    EranosSummary *const summary = summaryOf(group);
    EranosCategoryTotalList *const totals = eranosSummaryCategoryTotals(summary);
    check(isMinorUnits(categoryTotal(totals, uncategorised.text), 24850), "Uncategorised comes to 248.50 EUR");
    check(!anyIn(group, categories->transport), "no expense is in Getting around");
    eranosCategoryTotalListDestroy(totals);
    eranosSummaryDestroy(summary);
    eranosTextDestroy(text);
    eranosCategoryDestroy(removed);
}

// 10. A new currency.

/// Moves the books of @p group to RON, which a missing rate first refuses.
static void newCurrency(EranosGroup *const group, People const *const people) {
    section("10. A new currency");
    EranosCurrencyRateList *const none = eranosCurrencyRateListCreate();
    EranosError *const missing = eranosGroupChangeCurrency(group, "RON", none);
    eranosCurrencyRateListDestroy(none);
    check(isFailure(missing, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_MISSING_EXCHANGE_RATE) && namesCurrency(missing, "EUR"),
          "a change without the rate from EUR is refused");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = NULL;
    must(eranosCurrencyRateCreate("EUR", "5", &rate));
    eranosCurrencyRateListPush(rates, rate);
    eranosCurrencyRateDestroy(rate);
    must(eranosGroupChangeCurrency(group, "RON", rates));
    eranosCurrencyRateListDestroy(rates);
    Line const message = messageLine(missing);
    printFormatted("Home again, the group moves its books to RON. The change needs a rate into RON\n"
                   "for every other currency in use, here only EUR, or it is refused:\n"
                   "  error: %s\n"
                   "Entries in EUR take the new rate, and the RON parking and cash become local:",
                   message.text);
    printEntries(group, NULL);
    Line const spent = totalSpent(group);
    printFormatted("Each entry keeps its currency and converts at the new rate. At 5 every share\n"
                   "converts exactly, so each balance is five times what it was. The trip cost\n"
                   "%s:",
                   spent.text);
    printBalances(group);

    Line const currency = currencyOf(group);
    check(0 == strcmp(currency.text, "RON"), "the group keeps its books in RON");
    EranosSummary *const summary = summaryOf(group);
    check(isMinorUnits(eranosSummaryTotalSpent(summary), 611450), "the trip cost 6114.50 RON");
    eranosSummaryDestroy(summary);
    EranosMemberSummary *const dan = memberTotals(group, people->dan);
    check(isBalance(eranosMemberSummaryBalance(dan), ERANOS_BALANCE_KIND_GETS_BACK, 6275), "Dan gets back 62.75 RON");
    EranosMemberSummary *const eva = memberTotals(group, people->eva);
    check(isBalance(eranosMemberSummaryBalance(eva), ERANOS_BALANCE_KIND_OWES, 21300), "Eva owes 213.00 RON");
    eranosMemberSummaryDestroy(eva);
    eranosMemberSummaryDestroy(dan);
}

// 11. Settling up.

/// The name of @p strategy as the apps show it.
static char const *strategyName(EranosSettlementStrategy const strategy) {
    switch (strategy) {
    case ERANOS_SETTLEMENT_STRATEGY_DIRECT:
        return "Direct";
    case ERANOS_SETTLEMENT_STRATEGY_GREEDY:
        return "Greedy";
    case ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS:
        return "MinimumTransfers";
    }
}

/// The payments of the plan that @p strategy makes for @p group, which the caller destroys.
static EranosPaymentList *planOf(EranosGroup const *const group, EranosSettlementStrategy const strategy) {
    EranosSettlement *settlement = NULL;
    must(eranosGroupSettlement(group, strategy, &settlement));
    check(strategy == eranosSettlementStrategy(settlement), "the plan is the strategy's");
    EranosPaymentList *const payments = eranosSettlementPayments(settlement);
    eranosSettlementDestroy(settlement);
    return payments;
}

/// Whether every member with totals in @p group is settled.
static bool everyoneSettled(EranosGroup const *const group) {
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
static void settlingUp(EranosGroup *const group) {
    section("11. Settling up");
    EranosSettlementStrategy const strategies[] = {
        ERANOS_SETTLEMENT_STRATEGY_DIRECT,
        ERANOS_SETTLEMENT_STRATEGY_GREEDY,
        ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS,
    };
    printLine("Each strategy turns the balances into payments: the direct one pays back every\n"
              "pair's own debts, greedy pays the largest amounts first, and the minimum one\n"
              "finds the fewest payments.");
    Table *const counts = tableCreate(2, (char const *const[]){"Strategy", "Payments"}, 1);
    for (size_t index = 0; 3 > index; ++index) {
        EranosPaymentList *const payments = planOf(group, strategies[index]);
        tableAdd(counts, (Line const[]){lineOf("%s", strategyName(strategies[index])), lineOf("%zu", eranosPaymentListLength(payments))});
        eranosPaymentListDestroy(payments);
    }
    tablePrint(counts);
    EranosPaymentList *const plan = planOf(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS);
    printLine("The group follows the minimum plan:");
    Table *const table = tableCreate(3, (char const *const[]){"From", "To", "Amount"}, 2);
    EranosText *const from = eranosTextCreate();
    EranosText *const to = eranosTextCreate();
    for (size_t index = 0; eranosPaymentListLength(plan) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(plan, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        tableAdd(table, (Line const[]){
                            firstName(group, eranosTextData(from)),
                            firstName(group, eranosTextData(to)),
                            moneyLine(eranosPaymentAmount(payment)),
                        });
        eranosPaymentDestroy(payment);
    }
    tablePrint(table);
    for (size_t index = 0; eranosPaymentListLength(plan) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(plan, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        addTransfer(group, transferOf(localCost(eranosPaymentAmount(payment)), october(7), idOf(from), idOf(to)));
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
static bool namesFormat(EranosError const *const error, int64_t const format) {
    int64_t found = 0;
    return eranosErrorFormat(error, &found) && format == found;
}

/// Prints Ana's accounts as @p saved holds them, in clear text.
static void printAccounts(EranosGroup const *const group, Id const ana, char const *const saved) {
    EranosMember *member = NULL;
    must(eranosGroupMember(group, ana.text, &member));
    check(NULL != member, "Ana is a member");
    EranosAccountList *const accounts = eranosMemberAccounts(member);
    EranosText *const name = eranosTextCreate();
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosAccountListLength(accounts) > index; ++index) {
        EranosAccount *const account = eranosAccountListGet(accounts, index);
        eranosAccountName(account, name);
        EranosAccountDetails *const details = eranosAccountDetails(account);
        if (!eranosAccountDetailsIban(details, text)) {
            (void)eranosAccountDetailsUsername(details, text);
        }
        check(NULL != strstr(saved, eranosTextData(text)), "the details are saved as they are");
        printFormatted("  %s: %s", eranosTextData(name), eranosTextData(text));
        eranosAccountDetailsDestroy(details);
        eranosAccountDestroy(account);
    }
    eranosTextDestroy(text);
    eranosTextDestroy(name);
    eranosAccountListDestroy(accounts);
    eranosMemberDestroy(member);
}

/// What loading @p saved, with its first @p pattern replaced by @p replacement, is refused with.
static EranosError *changedRefusal(char const *const saved, char const *const pattern, char const *const replacement) {
    char *const changed = replaceFirst(saved, pattern, replacement);
    check(0 != strcmp(changed, saved), "the change changes the file");
    EranosGroup *loaded = NULL;
    EranosError *const refusal = eranosGroupLoad(changed, &loaded);
    check(NULL != refusal && NULL == loaded, "the changed file is refused");
    free(changed);
    return refusal;
}

/// Saves @p group, loads it again, and shows that a changed file is refused with a typed error.
static void savingAndLoading(EranosGroup const *const group, People const *const people, Categories const *const categories) {
    section("12. Saving and loading");
    EranosText *const saved = eranosTextCreate();
    must(eranosGroupSave(group, saved));
    printFormatted("The group saves to %zu bytes of JSON: its details and entries, never the totals.\n"
                   "Ana's accounts are saved in clear text, as the library holds them:",
                   eranosTextLength(saved));
    printAccounts(group, people->ana, eranosTextData(saved));
    EranosGroup *loaded = NULL;
    must(eranosGroupLoad(eranosTextData(saved), &loaded));
    printLine("Loading replays every entry through the same checks as adding it and rebuilds the\n"
              "totals: the loaded group equals the saved one. Reading a SavedGroup and loading it\n"
              "with into_group reports a refusal as a typed error instead of a message:");
    EranosError *const format = changedRefusal(eranosTextData(saved), "\"format\": 2", "\"format\": 3");
    EranosError *const blank = changedRefusal(eranosTextData(saved), "\"name\": \"Lisbon and Porto\"", "\"name\": \" \"");
    EranosError *const homonym = changedRefusal(eranosTextData(saved), "\"name\": \"Lodging\"", "\"name\": \"FOOD\"");
    check(isFailure(format, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNSUPPORTED_FORMAT) && namesFormat(format, 3), "a later format is refused");
    check(isFailure(blank, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_EMPTY_NAME), "a blank name is refused");
    check(isFailure(homonym, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_DUPLICATE_CATEGORY_NAME) && namesExisting(homonym, categories->food),
          "a caseless homonym is refused");
    Table *const table = tableCreate(2, (char const *const[]){"Change to the file", "Message"}, 2);
    tableAdd(table, (Line const[]){lineOf("\"format\": 3"), messageLine(format)});
    tableAdd(table, (Line const[]){lineOf("a blank group name"), messageLine(blank)});
    tableAdd(table, (Line const[]){lineOf("Lodging renamed FOOD"), messageLine(homonym)});
    tablePrint(table);

    EranosText *const resaved = eranosTextCreate();
    must(eranosGroupSave(loaded, resaved));
    check(0 == strcmp(eranosTextData(resaved), eranosTextData(saved)), "the loaded group saves as the group did");
    EranosSummary *const before = summaryOf(group);
    EranosSummary *const after = summaryOf(loaded);
    check(sameSummaries(before, after), "the loaded group has the group's totals");
    eranosSummaryDestroy(after);
    eranosSummaryDestroy(before);
    eranosTextDestroy(resaved);
    eranosGroupDestroy(loaded);
    eranosTextDestroy(saved);
}

// 13. Values on their own.

/// Prints four currencies, and that an unknown code is refused.
static void currencies(void) {
    char const *const codes[] = {"EUR", "RON", "JPY", "BHD"};
    Table *const table = tableCreate(4, (char const *const[]){"Code", "Name", "Numeric code", "Decimal places"}, 2);
    EranosText *const code = eranosTextCreate();
    EranosText *const name = eranosTextCreate();
    for (size_t index = 0; 4 > index; ++index) {
        EranosCurrencyDetails *details = NULL;
        must(eranosCurrencyDetailsOf(codes[index], &details));
        eranosCurrencyDetailsCode(details, code);
        eranosCurrencyDetailsName(details, name);
        tableAdd(table, (Line const[]){
                            textLine(code),
                            textLine(name),
                            lineOf("%ld", (long)eranosCurrencyDetailsNumericCode(details)),
                            lineOf("%ld", (long)eranosCurrencyDetailsDecimalPlaces(details)),
                        });
        eranosCurrencyDetailsDestroy(details);
    }
    eranosTextDestroy(name);
    eranosTextDestroy(code);
    EranosCurrencyDetails *unknownDetails = NULL;
    EranosError *const unknown = eranosCurrencyDetailsOf("XYZ", &unknownDetails);
    check(NULL != unknown && NULL == unknownDetails, "an unknown code is refused");
    EranosCurrencyDetailsList *const all = eranosCurrencyDetailsListAll();
    Line const message = messageLine(unknown);
    printFormatted("Currencies are the %zu active ISO 4217 codes with a minor unit; an unknown code is\n"
                   "refused (%s):",
                   eranosCurrencyDetailsListLength(all), message.text);
    eranosCurrencyDetailsListDestroy(all);
    tablePrint(table);
    EranosCurrencyDetails *dinar = NULL;
    must(eranosCurrencyDetailsOf("BHD", &dinar));
    check(3 == eranosCurrencyDetailsDecimalPlaces(dinar), "the dinar has three decimal places");
    eranosCurrencyDetailsDestroy(dinar);
}

/// The sum of @p left and @p right, or the refusal of the sum; destroys both.
static Line sumOf(EranosMoney *const left, EranosMoney *const right, int64_t *const minorUnits) {
    EranosMoney *sum = NULL;
    EranosError *const refused = eranosMoneyAdd(left, right, &sum);
    eranosMoneyDestroy(right);
    eranosMoneyDestroy(left);
    if (NULL != refused) {
        return messageLine(refused);
    }
    *minorUnits = eranosMoneyMinorUnits(sum);
    return moneyLine(sum);
}

/// The message of the refusal of the amount @p text of @p currency.
static Line amountRefusal(char const *const text, char const *const currency) {
    EranosMoney *money = NULL;
    EranosError *const refused = eranosMoneyParse(text, currency, &money);
    check(NULL != refused && NULL == money, "the amount is refused");
    return messageLine(refused);
}

/// Prints what amounts do: sums, differences, refusals, minor units, and a cost's rate kept for another amount.
static void amounts(void) {
    int64_t total = 0;
    Line const sum = sumOf(eur("12.34"), eur("0.66"), &total);
    EranosMoney *const five = eur("5.00");
    EranosMoney *const more = eur("7.50");
    EranosMoney *change = NULL;
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
    EranosMoney *dinars = NULL;
    must(eranosMoneyCreate("BHD", 1234, &dinars));
    Line const dinarsLine = moneyLine(dinars);
    EranosMoney *nothing = NULL;
    must(eranosMoneyCreate("RON", 0, &nothing));
    bool const zero = 0 == eranosMoneyMinorUnits(nothing);
    Line const nothingLine = moneyLine(nothing);
    EranosCost *const cost = convertedCost(eur("10.00"), "4.9765");
    EranosText *const rate = eranosTextCreate();
    (void)eranosCostRate(cost, rate);
    EranosCost *const doubled = convertedCost(eur("20.00"), eranosTextData(rate));
    Line const costLine = moneyLine(eranosCostAmount(cost));
    Line const doubledLine = moneyLine(eranosCostAmount(doubled));
    printLine("Amounts are exact minor units in their currency:");
    Table *const table = tableCreate(2, (char const *const[]){"Input", "Result"}, 2);
    tableAdd(table, (Line const[]){lineOf("12.34 EUR + 0.66 EUR"), sum});
    tableAdd(table, (Line const[]){lineOf("5.00 EUR − 7.50 EUR"), lineOf("%s, negative: %s", changeLine.text, 0 > changeUnits ? "true" : "false")});
    tableAdd(table, (Line const[]){lineOf("1.00 EUR + 1.00 RON"), mixed});
    tableAdd(table, (Line const[]){lineOf("1.005 EUR"), decimals});
    tableAdd(table, (Line const[]){lineOf("1500 JPY"), lineOf("%s, %lld minor units", yenLine.text, (long long)yenUnits)});
    tableAdd(table, (Line const[]){lineOf("1234 minor units of BHD"), dinarsLine});
    tableAdd(table, (Line const[]){lineOf("zero RON"), lineOf("%s, zero: %s", nothingLine.text, zero ? "true" : "false")});
    tableAdd(table, (Line const[]){
                        lineOf("10.00 EUR at 4.9765"),
                        lineOf("%s at %s, then %s at the same rate", costLine.text, eranosTextData(rate), doubledLine.text),
                    });
    tablePrint(table);
    check(1300 == total, "12.34 EUR and 0.66 EUR make 13.00 EUR");
    check(-250 == changeUnits, "5.00 EUR less 7.50 EUR is -2.50 EUR");
    check(0 == strcmp(mixed.text, "cannot combine EUR and RON amounts"), "amounts in two currencies do not combine");
    check(0 == strcmp(dinarsLine.text, "1.234 BHD"), "a dinar has three decimal places");
    eranosCostDestroy(doubled);
    eranosTextDestroy(rate);
    eranosCostDestroy(cost);
}

/// The text of the percentage of @p basisPoints.
static Line percentageLine(int32_t const basisPoints) {
    EranosText *const text = eranosTextCreate();
    must(eranosPercentageFormat(basisPoints, text));
    Line const line = textLine(text);
    eranosTextDestroy(text);
    return line;
}

/// The text of @p hundredths hundredths of a part.
static Line partsLine(int32_t const hundredths) {
    EranosText *const text = eranosTextCreate();
    must(eranosPartsFormat(hundredths, text));
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
    Table *const table = tableCreate(2, (char const *const[]){"Input", "Result"}, 2);
    tableAdd(table, (Line const[]){lineOf("12.5%%"), lineOf("%s, %ld basis points", percentage.text, (long)basisPoints)});
    tableAdd(table, (Line const[]){lineOf("333 basis points"), percentageLine(333)});
    tableAdd(table, (Line const[]){lineOf("33.333%%"), tooPrecise});
    tableAdd(table, (Line const[]){lineOf("1.5 parts"), lineOf("%s, %ld hundredths", parts.text, (long)hundredths)});
    tableAdd(table, (Line const[]){lineOf("3 whole parts"), partsLine(300)});
    tableAdd(table, (Line const[]){lineOf("25 hundredths of a part"), partsLine(25)});
    tableAdd(table, (Line const[]){lineOf("0 parts"), noParts});
    tableAdd(table, (Line const[]){lineOf("a rate of 4.97650"), exchange});
    tableAdd(table, (Line const[]){lineOf("a rate of 0"), noRate});
    tablePrint(table);
    check(1250 == basisPoints, "12.5 % is 1250 basis points");
    check(150 == hundredths, "1.5 parts are 150 hundredths");
}

/// Prints what dates, times, identifiers, and language tags read and print.
static void datesAndIdentifiers(EranosGroup const *const group, People const *const people) {
    EranosText *const text = eranosTextCreate();
    EranosCalendarDate date = {.day = 0, .month = 0, .year = 0};
    must(eranosCalendarDateParse("2026-10-01", &date));
    must(eranosCalendarDateFormat(date, text));
    Line const dateLine = lineOf("%s: day %ld, month %ld, year %ld", eranosTextData(text), (long)date.day, (long)date.month, (long)date.year);
    Line const leapDay = messageLine(eranosCalendarDateFormat((EranosCalendarDate){.day = 29, .month = 2, .year = 2027}, text));
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    must(eranosTimeOfDayParse("22:15", &time));
    must(eranosTimeOfDayFormat(time, text));
    Line const timeLine = lineOf("%s: hour %ld, minute %ld", eranosTextData(text), (long)time.hour, (long)time.minute);
    Line const midnight = messageLine(eranosTimeOfDayFormat((EranosTimeOfDay){.hour = 24, .minute = 0}, text));
    EranosMember *member = NULL;
    must(eranosGroupMember(group, people->ana.text, &member));
    check(NULL != member, "Ana is a member");
    bool const found = eranosMemberId(member, text) && 0 == strcmp(eranosTextData(text), people->ana.text);
    eranosMemberDestroy(member);
    EranosMember *nobody = NULL;
    Line const notAnId = messageLine(eranosGroupMember(group, "not-a-uuid", &nobody));
    Line const language = valueOr(eranosLanguageParse("pt-pt", text), text);
    Line const underscore = messageLine(eranosLanguageParse("pt_PT", text));
    eranosTextDestroy(text);
    printLine("Dates, times, identifiers, and language tags read and print the ISO forms:");
    Table *const table = tableCreate(2, (char const *const[]){"Input", "Result"}, 2);
    tableAdd(table, (Line const[]){lineOf("2026-10-01"), dateLine});
    tableAdd(table, (Line const[]){lineOf("29 February 2027"), leapDay});
    tableAdd(table, (Line const[]){lineOf("22:15"), timeLine});
    tableAdd(table, (Line const[]){lineOf("24:00"), midnight});
    tableAdd(table, (Line const[]){
                        lineOf("Ana's identifier"),
                        lineOf("%s, the same after a round trip: %s", people->ana.text, found ? "true" : "false"),
                    });
    tableAdd(table, (Line const[]){lineOf("not-a-uuid"), notAnId});
    tableAdd(table, (Line const[]){lineOf("pt-pt"), language});
    tableAdd(table, (Line const[]){lineOf("pt_PT"), underscore});
    tablePrint(table);
    check(1 == date.day && 10 == date.month && 2026 == date.year, "the date reads as 1 October 2026");
    check(22 == time.hour && 15 == time.minute, "the time reads as 22:15");
    check(found, "Ana's identifier comes back the same");
}

/// Prints the values on their own: currencies, amounts, ratios, dates, times, identifiers, and language tags.
static void values(EranosGroup const *const group, People const *const people) {
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
    /// The error the call reported, or NULL.
    EranosError *error;
    /// Guards the rest.
    pthread_mutex_t mutex;
    /// The value the call reported, or NULL.
    void *value;
} Waiter;

/// @p waiter, ready for a call.
static Waiter *waiting(Waiter *const waiter) {
    waiter->done = false;
    waiter->error = NULL;
    waiter->value = NULL;
    (void)pthread_mutex_init(&waiter->mutex, NULL);
    (void)pthread_cond_init(&waiter->condition, NULL);
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
    EranosError *error = NULL;
    void *const value = outcome(waiter, &error);
    must(error);
    return value;
}

/// Receives an outcome without a value.
static void receiveUnit(void *const context, EranosError *const error) { finish(context, error, NULL); }

/// Receives text.
static void receiveText(void *const context, EranosError *const error, EranosText *const text) { finish(context, error, text); }

/// Receives the expenses.
static void receiveExpenses(void *const context, EranosError *const error, EranosExpenseList *const expenses) { finish(context, error, expenses); }

/// Receives a read handle.
static void receiveReader(void *const context, EranosError *const error, EranosGroupReader *const reader) { finish(context, error, reader); }

/// Receives a settlement.
static void receiveSettlement(void *const context, EranosError *const error, EranosSettlement *const settlement) {
    finish(context, error, settlement);
}

/// Receives a summary.
static void receiveSummary(void *const context, EranosError *const error, EranosSummary *const summary) { finish(context, error, summary); }

/// The summary of @p group, read asynchronously; the caller destroys it.
static EranosSummary *summaryAsync(EranosGroup const *const group) {
    Waiter waiter;
    EranosSummaryCallback const callback = {.context = waiting(&waiter), .run = receiveSummary};
    eranosGroupSummaryAsync(group, NULL, &callback);
    return awaited(&waiter);
}

/// A read handle on @p group, taken asynchronously; the caller destroys it.
static EranosGroupReader *readAsync(EranosGroup const *const group) {
    Waiter waiter;
    EranosGroupReaderCallback const callback = {.context = waiting(&waiter), .run = receiveReader};
    eranosGroupReadAsync(group, NULL, &callback);
    return awaited(&waiter);
}

/// The first name of @p member, read through @p reader, or ? for one the group does not hold.
static Line readerName(EranosGroupReader const *const reader, char const *const member) {
    EranosMember *found = NULL;
    must(eranosGroupReaderMember(reader, member, &found));
    if (NULL == found) {
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
static void settleCoffee(EranosGroup *const other, EranosGroup const *const group, People const *const people) {
    Members const everyone = {.count = 5, .ids = {people->ana, people->bob, people->carol, people->dan, people->eva}};
    EranosExpense *const coffee = expenseOf(localCost(ron("75.00")), october(8), "Airport coffee", soleSplit(people->bob), equalSplit(&everyone));
    Waiter added;
    EranosTextCallback const addedCallback = {.context = waiting(&added), .run = receiveText};
    eranosGroupAddExpenseAsync(other, coffee, NULL, &addedCallback);
    eranosTextDestroy(awaited(&added));
    eranosExpenseDestroy(coffee);
    Waiter planned;
    EranosSettlementCallback const plannedCallback = {.context = waiting(&planned), .run = receiveSettlement};
    eranosGroupSettlementAsync(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS, NULL, &plannedCallback);
    EranosSettlement *const plan = awaited(&planned);
    EranosPaymentList *const payments = eranosSettlementPayments(plan);
    printLine("An app shares the group between its tasks and threads through a SharedGroup,\n"
              "whose calls are futures any executor runs; here the library's block_on runs\n"
              "each one on this thread. Bob adds the airport coffee through another handle,\n"
              "and the plan that settles it comes through the first:");
    EranosGroupReader *const reader = readAsync(group);
    Table *const table = tableCreate(3, (char const *const[]){"From", "To", "Amount"}, 2);
    EranosText *const from = eranosTextCreate();
    EranosText *const to = eranosTextCreate();
    bool toBob = true;
    for (size_t index = 0; eranosPaymentListLength(payments) > index; ++index) {
        EranosPayment *const payment = eranosPaymentListGet(payments, index);
        eranosPaymentFrom(payment, from);
        eranosPaymentTo(payment, to);
        EranosMoney *const amount = eranosPaymentAmount(payment);
        toBob = toBob && 0 == strcmp(eranosTextData(to), people->bob.text) && 1500 == eranosMoneyMinorUnits(amount);
        tableAdd(table, (Line const[]){readerName(reader, eranosTextData(from)), readerName(reader, eranosTextData(to)), moneyLine(amount)});
        eranosPaymentDestroy(payment);
    }
    tablePrint(table);
    eranosGroupReaderRelease(reader);
    check(4 == eranosPaymentListLength(payments) && toBob, "everyone pays Bob 15.00 RON");
    eranosTextDestroy(to);
    eranosTextDestroy(from);
    eranosGroupReaderDestroy(reader);
    eranosPaymentListDestroy(payments);
    eranosSettlementDestroy(plan);
}

/// Shares @p group between tasks: an expense added through another handle, a settlement computed asynchronously, and a
/// change that waits for a report's read and is cancelled.
static void sharing(EranosGroup *const group, People const *const people) {
    section("14. Sharing the group");
    // Another handle on the same group, as another task holds: a group is shared as it is.
    EranosGroup *const other = group;
    settleCoffee(other, group, people);

    EranosSummary *const before = summaryAsync(group);
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = NULL;
    must(eranosCurrencyRateCreate("RON", "0.2", &rate));
    eranosCurrencyRateListPush(rates, rate);
    eranosCurrencyRateDestroy(rate);
    // A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
    // behind the change, so the report reads through its own handle.
    EranosGroupReader *const report = readAsync(group);
    EranosCancellation *const cancellation = eranosCancellationCreate();
    Waiter changed;
    EranosUnitCallback const changedCallback = {.context = waiting(&changed), .run = receiveUnit};
    eranosGroupChangeCurrencyAsync(group, "EUR", rates, cancellation, &changedCallback);
    EranosSummary *reportedSummary = NULL;
    must(eranosGroupReaderSummary(report, &reportedSummary));
    // Cancelling stops the waiting change, which leaves the group as it was.
    eranosCancellationCancel(cancellation);
    eranosGroupReaderRelease(report);
    EranosError *cancelled = NULL;
    (void)outcome(&changed, &cancelled);
    bool const wasCancelled = NULL != cancelled && ERANOS_ERROR_KIND_CANCELLED == eranosErrorKind(cancelled);
    eranosErrorDestroy(cancelled);
    EranosSummary *const after = summaryAsync(group);
    Waiter currency;
    EranosTextCallback const currencyCallback = {.context = waiting(&currency), .run = receiveText};
    eranosGroupCurrencyAsync(group, NULL, &currencyCallback);
    EranosText *const code = awaited(&currency);
    Line const spent = moneyLine(eranosSummaryTotalSpent(after));
    printFormatted("While a report reads the group, a change of its currency to EUR waits for the\n"
                   "report. The app drops the waiting change, which cancels it: the group still\n"
                   "keeps its books in %s, and the trip cost %s, as the report read.",
                   eranosTextData(code), spent.text);
    Waiter listed;
    EranosExpenseListCallback const listedCallback = {.context = waiting(&listed), .run = receiveExpenses};
    eranosGroupExpensesAsync(group, NULL, &listedCallback);
    EranosExpenseList *const expenses = awaited(&listed);
    printFormatted("With the other handle gone, the app takes the group back: %zu expenses.", eranosExpenseListLength(expenses));

    check(wasCancelled, "the change is cancelled");
    check(sameSummaries(before, after), "the cancelled change leaves the group as it was");
    check(sameAmount(eranosSummaryTotalSpent(reportedSummary), eranosSummaryTotalSpent(after)), "the report read what the group holds");
    check(0 == strcmp(eranosTextData(code), "RON"), "the group keeps its books in RON");
    eranosExpenseListDestroy(expenses);
    eranosTextDestroy(code);
    eranosSummaryDestroy(after);
    eranosSummaryDestroy(reportedSummary);
    eranosCancellationDestroy(cancellation);
    eranosGroupReaderDestroy(report);
    eranosCurrencyRateListDestroy(rates);
    eranosSummaryDestroy(before);
}

int main(void) {
    printLine("Lisbon and Porto: a grand tour of eranos\n"
              "════════════════════════════════════════");
    printLine("Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each\n"
              "section below uses another part of the library, from creating the group to saving\n"
              "it, and checks its key numbers.");
    EranosGroup *const group = theGroup();
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
    eranosGroupDestroy(group);
    return EXIT_SUCCESS;
}
