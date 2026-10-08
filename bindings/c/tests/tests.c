// The tests of the C layer: every value, record, and list through the functions that make and read it, every group call in
// both forms, the read handle, the cancellation, and every error the library reports with what it holds. Each test is a
// claim, named for what it states, whose comment says what breaks in an app if it regresses. The program prints every claim
// that fails and ends with a failure status when one does.

#include "eranos.h"

#include <math.h>
#include <pthread.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/// The claims of one test that failed.
typedef struct Check {
    /// The test's name.
    char const *test;
    /// How many of its claims failed.
    size_t failures;
} Check;

/// Records @p claim as failed unless @p holds.
static void checkThat(Check *const check, bool const holds, char const *const claim) {
    if (!holds) {
        (void)fprintf(stderr, "%s: %s\n", check->test, claim);
        ++check->failures;
    }
}

/// Records @p claim as failed when @p error is an error, which it destroys, printing its message.
static void checkSucceeded(Check *const check, EranosError *const error, char const *const claim) {
    if (nullptr != error) {
        EranosText *const message = eranosTextCreate();
        eranosErrorMessage(error, message);
        (void)fprintf(stderr, "%s: %s: %s\n", check->test, claim, eranosTextData(message));
        eranosTextDestroy(message);
        eranosErrorDestroy(error);
        ++check->failures;
    }
}

/// Whether @p text holds @p expected.
[[nodiscard]] static bool isText(EranosText const *const text, char const *const expected) { return 0 == strcmp(eranosTextData(text), expected); }

/// Whether @p error is of @p kind, and holds no failure within a module; destroys it.
[[nodiscard]] static bool isKind(EranosError *const error, EranosErrorKind const kind) {
    if (nullptr == error) {
        return false;
    }
    EranosFailure failure = ERANOS_FAILURE_PAYMENT_ACCOUNT_EMPTY_NAME;
    bool const same = eranosErrorKind(error) == kind && !eranosErrorFailure(error, &failure);
    eranosErrorDestroy(error);
    return same;
}

/// Whether @p error is of @p kind with @p expected; destroys it.
[[nodiscard]] static bool isFailure(EranosError *const error, EranosErrorKind const kind, EranosFailure const expected) {
    if (nullptr == error) {
        return false;
    }
    EranosFailure failure = ERANOS_FAILURE_PAYMENT_ACCOUNT_EMPTY_NAME;
    bool const same = eranosErrorKind(error) == kind && eranosErrorFailure(error, &failure) && failure == expected;
    eranosErrorDestroy(error);
    return same;
}

/// Whether @p error is of @p kind with @p failure, without destroying it, so that a test reads what it holds.
[[nodiscard]] static bool holdsFailure(EranosError const *const error, EranosErrorKind const kind, EranosFailure const expected) {
    EranosFailure failure = ERANOS_FAILURE_PAYMENT_ACCOUNT_EMPTY_NAME;
    return nullptr != error && eranosErrorKind(error) == kind && eranosErrorFailure(error, &failure) && failure == expected;
}

/// The details of the currency of the ISO 4217 code @p code, which the library lists.
[[nodiscard]] static EranosCurrencyDetails *currencyOf(char const *const code) {
    EranosCurrencyDetails *currency = nullptr;
    eranosErrorDestroy(eranosCurrencyDetailsOf(code, &currency));
    return currency;
}

/// The amount of @p minorUnits minor units of the currency of the ISO 4217 code @p code, which the library lists.
[[nodiscard]] static EranosMoney *amountOf(int64_t const minorUnits, char const *const code) {
    EranosCurrencyDetails *const currency = currencyOf(code);
    EranosMoney *const money = eranosMoneyCreate(minorUnits, currency);
    eranosCurrencyDetailsDestroy(currency);
    return money;
}

/// @p cents euro cents.
[[nodiscard]] static EranosMoney *euros(int64_t const cents) { return amountOf(cents, "EUR"); }

/// Whether @p currency is the currency of the ISO 4217 code @p code; destroys it.
[[nodiscard]] static bool isCurrency(EranosCurrencyDetails *const currency, char const *const code) {
    if (nullptr == currency) {
        return false;
    }
    EranosText *const text = eranosTextCreate();
    eranosCurrencyDetailsCode(currency, text);
    bool const same = isText(text, code);
    eranosTextDestroy(text);
    eranosCurrencyDetailsDestroy(currency);
    return same;
}

/// Whether @p money is @p minorUnits minor units of the currency of the ISO 4217 code @p code; destroys it.
[[nodiscard]] static bool isAmount(EranosMoney *const money, int64_t const minorUnits, char const *const code) {
    if (nullptr == money) {
        return false;
    }
    bool const same = eranosMoneyMinorUnits(money) == minorUnits && isCurrency(eranosMoneyCurrency(money), code);
    eranosMoneyDestroy(money);
    return same;
}

/// Whether @p tip is the no-tip rule, 0 % of the amount shared proportionally; destroys it.
[[nodiscard]] static bool isNoTip(EranosTipRule *const tip) {
    if (nullptr == tip) {
        return false;
    }
    EranosTipValue *const value = eranosTipRuleValue(tip);
    int32_t basisPoints = -1;
    bool const same = ERANOS_TIP_VALUE_KIND_PERCENTAGE == eranosTipValueKind(value) && eranosTipValueBasisPoints(value, &basisPoints) &&
                      0 == basisPoints && ERANOS_TIP_DISTRIBUTION_PROPORTIONAL == eranosTipRuleDistribution(tip);
    eranosTipValueDestroy(value);
    eranosTipRuleDestroy(tip);
    return same;
}

/// Whether @p expense is in the built-in category, by the identifier eranosIdentifierUncategorised() writes.
[[nodiscard]] static bool isUncategorised(EranosExpense const *const expense) {
    EranosText *const category = eranosTextCreate();
    EranosText *const uncategorised = eranosTextCreate();
    eranosExpenseCategory(expense, category);
    eranosIdentifierUncategorised(uncategorised);
    bool const same = isText(category, eranosTextData(uncategorised));
    eranosTextDestroy(uncategorised);
    eranosTextDestroy(category);
    return same;
}

/// The list of @p first and, unless nullptr, @p second.
[[nodiscard]] static EranosTextList *membersOf(char const *const first, char const *const second) {
    EranosTextList *const members = eranosTextListCreate();
    eranosErrorDestroy(eranosTextListPush(members, first));
    if (nullptr != second) {
        eranosErrorDestroy(eranosTextListPush(members, second));
    }
    return members;
}

/// The split that divides equally among @p first and, unless nullptr, @p second.
[[nodiscard]] static EranosSplit *equally(char const *const first, char const *const second) {
    EranosTextList *const members = membersOf(first, second);
    EranosSplit *const split = eranosSplitCreateEqual(members);
    eranosTextListDestroy(members);
    return split;
}

/// Whether @p list holds @p first and then, unless nullptr, @p second; destroys it.
[[nodiscard]] static bool listsMembers(EranosTextList *const list, char const *const first, char const *const second) {
    if (nullptr == list) {
        return false;
    }
    EranosText *const text = eranosTextCreate();
    bool same = eranosTextListLength(list) == (nullptr == second ? 1U : 2U) && eranosTextListGet(list, 0, text) && isText(text, first);
    if (nullptr != second) {
        same = same && eranosTextListGet(list, 1, text) && isText(text, second);
    }
    same = same && !eranosTextListGet(list, 2, text);
    eranosTextDestroy(text);
    eranosTextListDestroy(list);
    return same;
}

/// The day the tests' entries happen on.
static constexpr EranosCalendarDate DAY = {.year = 2026, .month = 9, .day = 23};

/// An expense of 30.00 EUR, called @p name, that @p payer paid and @p first and @p second share equally.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the name, the payer, and the members, in the order the expense reads them.
[[nodiscard]] static EranosExpense *expenseOf(char const *const name, char const *const payer, char const *const first, char const *const second) {
    EranosMoney *const thirty = euros(3'000);
    EranosCost *const cost = eranosCostCreateLocal(thirty);
    EranosSplit *const payers = equally(payer, nullptr);
    EranosSplit *const shares = equally(first, second);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateWhole(shares);
    EranosExpense *expense = nullptr;
    eranosErrorDestroy(eranosExpenseCreate(name, cost, payers, split, DAY, &expense));
    eranosExpenseSplitDestroy(split);
    eranosSplitDestroy(shares);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(thirty);
    return expense;
}

/// A transfer from @p from to @p to of @p cents euro cents.
[[nodiscard]] static EranosTransfer *transferOf(char const *const from, char const *const to, int64_t const cents) {
    EranosMoney *const amount = euros(cents);
    EranosCost *const cost = eranosCostCreateLocal(amount);
    EranosTransfer *transfer = nullptr;
    eranosErrorDestroy(eranosTransferCreate(from, to, cost, DAY, &transfer));
    eranosCostDestroy(cost);
    eranosMoneyDestroy(amount);
    return transfer;
}

/// A group of two, Ana and Bob, with a category, an expense, and a transfer, and the identifiers the group gave them.
typedef struct Flat {
    /// The group.
    EranosExpenseGroup *group;
    /// Ana's identifier.
    EranosText *ana;
    /// Bob's identifier.
    EranosText *bob;
    /// The food category's identifier.
    EranosText *food;
    /// The dinner Ana paid for the two of them, in the food category.
    EranosText *dinner;
    /// The 10.00 EUR Bob paid Ana back.
    EranosText *repayment;
} Flat;

/// Adds a member called @p firstName to @p group, and returns their identifier.
[[nodiscard]] static EranosText *memberIn(EranosExpenseGroup *const group, char const *const firstName) {
    EranosMember *member = nullptr;
    eranosErrorDestroy(eranosMemberCreate(firstName, &member));
    EranosText *const id = eranosTextCreate();
    eranosErrorDestroy(eranosExpenseGroupAddMember(group, member, id));
    eranosMemberDestroy(member);
    return id;
}

/// The flat.
[[nodiscard]] static Flat flatOf(void) {
    EranosNewExpenseGroup *details = nullptr;
    eranosErrorDestroy(eranosNewExpenseGroupCreate("Flat", "EUR", &details));
    eranosErrorDestroy(eranosNewExpenseGroupSetDescription(details, "Rent"));
    eranosErrorDestroy(eranosNewExpenseGroupSetLanguage(details, "pt-pt"));
    Flat flat = {.group = nullptr};
    eranosErrorDestroy(eranosExpenseGroupCreate(details, &flat.group));
    eranosNewExpenseGroupDestroy(details);
    flat.ana = memberIn(flat.group, "Ana");
    flat.bob = memberIn(flat.group, "Bob");
    EranosCategory *food = nullptr;
    eranosErrorDestroy(eranosCategoryCreate("Food", &food));
    flat.food = eranosTextCreate();
    eranosErrorDestroy(eranosExpenseGroupAddCategory(flat.group, food, flat.food));
    eranosCategoryDestroy(food);
    EranosExpense *const dinner = expenseOf("Dinner", eranosTextData(flat.ana), eranosTextData(flat.ana), eranosTextData(flat.bob));
    eranosErrorDestroy(eranosExpenseSetCategory(dinner, eranosTextData(flat.food)));
    flat.dinner = eranosTextCreate();
    eranosErrorDestroy(eranosExpenseGroupAddExpense(flat.group, dinner, flat.dinner));
    eranosExpenseDestroy(dinner);
    EranosTransfer *const repayment = transferOf(eranosTextData(flat.bob), eranosTextData(flat.ana), 1'000);
    flat.repayment = eranosTextCreate();
    eranosErrorDestroy(eranosExpenseGroupAddTransfer(flat.group, repayment, flat.repayment));
    eranosTransferDestroy(repayment);
    return flat;
}

/// Destroys @p flat.
static void flatDestroy(Flat const flat) {
    eranosTextDestroy(flat.repayment);
    eranosTextDestroy(flat.dinner);
    eranosTextDestroy(flat.food);
    eranosTextDestroy(flat.bob);
    eranosTextDestroy(flat.ana);
    eranosExpenseGroupDestroy(flat.group);
}

/// What an asynchronous call reported, and whether it has: its callback runs on another thread, which signals the waiting one.
typedef struct Waiter {
    /// Signalled once the callback ran.
    pthread_cond_t condition;
    /// Whether the callback ran.
    bool done;
    /// Guards every other field.
    pthread_mutex_t mutex;
    /// Whether the call reported an answer.
    bool answered;
    /// The error the call reported, or nullptr.
    EranosError *error;
    /// The answer the call reported, if it reported one.
    bool flag;
    /// The object the call reported, or nullptr.
    void *object;
    /// The text the call reported, or nullptr.
    EranosText *text;
} Waiter;

/// @p waiter, ready for a call.
[[nodiscard]] static Waiter *waiting(Waiter *const waiter) {
    (void)pthread_cond_init(&waiter->condition, nullptr);
    waiter->done = false;
    (void)pthread_mutex_init(&waiter->mutex, nullptr);
    waiter->answered = false;
    waiter->error = nullptr;
    waiter->flag = false;
    waiter->object = nullptr;
    waiter->text = nullptr;
    return waiter;
}

/// Marks @p waiter done and wakes its thread.
static void finish(Waiter *const waiter) {
    (void)pthread_mutex_lock(&waiter->mutex);
    waiter->done = true;
    (void)pthread_cond_signal(&waiter->condition);
    (void)pthread_mutex_unlock(&waiter->mutex);
}

/// Waits for the callback of @p waiter.
static void await(Waiter *const waiter) {
    (void)pthread_mutex_lock(&waiter->mutex);
    while (!waiter->done) {
        (void)pthread_cond_wait(&waiter->condition, &waiter->mutex);
    }
    (void)pthread_mutex_unlock(&waiter->mutex);
    (void)pthread_cond_destroy(&waiter->condition);
    (void)pthread_mutex_destroy(&waiter->mutex);
}

/// Receives an outcome without a value.
static void receiveUnit(void *const context, EranosError *const error) {
    Waiter *const waiter = context;
    waiter->error = error;
    finish(waiter);
}

/// Receives whether a member is archived, which @p archived points to until the function returns, or none, when the call failed or found
/// no member.
static void receiveFlag(void *const context, EranosError *const error, bool const *const archived) {
    Waiter *const waiter = context;
    waiter->answered = nullptr != archived;
    waiter->error = error;
    if (nullptr != archived) {
        waiter->flag = *archived;
    }
    finish(waiter);
}

/// Receives text.
static void receiveText(void *const context, EranosError *const error, EranosText *const text) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->text = text;
    finish(waiter);
}

/// Receives a list of categories.
static void receiveCategories(void *const context, EranosError *const error, EranosCategoryList *const categories) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = categories;
    finish(waiter);
}

/// Receives a category.
static void receiveCategory(void *const context, EranosError *const error, EranosCategory *const category) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = category;
    finish(waiter);
}

/// Receives a currency's details.
static void receiveCurrency(void *const context, EranosError *const error, EranosCurrencyDetails *const currency) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = currency;
    finish(waiter);
}

/// Receives a list of expenses.
static void receiveExpenses(void *const context, EranosError *const error, EranosExpenseList *const expenses) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = expenses;
    finish(waiter);
}

/// Receives an expense.
static void receiveExpense(void *const context, EranosError *const error, EranosExpense *const expense) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = expense;
    finish(waiter);
}

/// Receives an expense's shares.
static void receiveExpenseShares(void *const context, EranosError *const error, EranosExpenseShares *const expenseShares) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = expenseShares;
    finish(waiter);
}

/// Receives a group.
static void receiveGroup(void *const context, EranosError *const error, EranosExpenseGroup *const group) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = group;
    finish(waiter);
}

/// Receives a read handle.
static void receiveReader(void *const context, EranosError *const error, EranosExpenseGroupReader *const reader) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = reader;
    finish(waiter);
}

/// Receives a list of members.
static void receiveMembers(void *const context, EranosError *const error, EranosMemberList *const members) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = members;
    finish(waiter);
}

/// Receives a member.
static void receiveMember(void *const context, EranosError *const error, EranosMember *const member) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = member;
    finish(waiter);
}

/// Receives an amount.
static void receiveMoney(void *const context, EranosError *const error, EranosMoney *const money) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = money;
    finish(waiter);
}

/// Receives a settlement.
static void receiveSettlement(void *const context, EranosError *const error, EranosSettlementPlan *const settlement) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = settlement;
    finish(waiter);
}

/// Receives a summary.
static void receiveSummary(void *const context, EranosError *const error, EranosSummary *const summary) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = summary;
    finish(waiter);
}

/// Receives a list of transfers.
static void receiveTransfers(void *const context, EranosError *const error, EranosTransferList *const transfers) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = transfers;
    finish(waiter);
}

/// Receives a transfer.
static void receiveTransfer(void *const context, EranosError *const error, EranosTransfer *const transfer) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = transfer;
    finish(waiter);
}

// --- Values -----------------------------------------------------------------------------------------------------------

/// A text that kept what an earlier call wrote, or lost its end, would print one value after another.
static void aTextHoldsWhatTheLastCallWrote(Check *const check) {
    EranosText *const text = eranosTextCreate();
    checkThat(check, isText(text, "") && 0 == eranosTextLength(text), "a text starts empty");
    EranosMoney *const price = euros(1'250);
    eranosMoneyFormat(price, text);
    checkThat(check, isText(text, "12.50 EUR") && eranosTextLength(text) == strlen("12.50 EUR"), "the amount as the library writes it");
    EranosCurrencyDetails *const currency = eranosMoneyCurrency(price);
    eranosCurrencyDetailsCode(currency, text);
    checkThat(check, isText(text, "EUR"), "a second call replaces the first's text");
    eranosCurrencyDetailsDestroy(currency);
    eranosMoneyDestroy(price);
    eranosTextDestroy(text);
}

/// A currency whose details came out wrong would format every amount in it wrongly, and a list that missed one would hide it.
static void aCurrencyHoldsWhatTheLibraryKnowsOfIt(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosCurrencyDetails *euro = nullptr;
    checkSucceeded(check, eranosCurrencyDetailsOf("EUR", &euro), "the euro is listed");
    eranosCurrencyDetailsCode(euro, text);
    checkThat(check, isText(text, "EUR"), "its code");
    eranosCurrencyDetailsName(euro, text);
    checkThat(check, isText(text, "Euro"), "its name");
    checkThat(check, 2 == eranosCurrencyDetailsDecimalPlaces(euro) && 978 == eranosCurrencyDetailsNumericCode(euro), "its decimal places and number");
    eranosCurrencyDetailsDestroy(euro);
    EranosCurrencyDetails *unknown = nullptr;
    checkThat(check,
              isFailure(eranosCurrencyDetailsOf("XYZ", &unknown), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY) &&
                  nullptr == unknown,
              "an unknown code is refused, and gives no details");
    EranosCurrencyDetailsList *const all = eranosCurrencyDetailsListAll();
    checkThat(check, !eranosCurrencyDetailsListIsEmpty(all) && 100 < eranosCurrencyDetailsListLength(all), "the library lists every currency");
    EranosCurrencyDetails *const first = eranosCurrencyDetailsListGet(all, 0);
    checkThat(check, nullptr != first && nullptr == eranosCurrencyDetailsListGet(all, eranosCurrencyDetailsListLength(all)),
              "a list ends at its length");
    eranosCurrencyDetailsDestroy(first);
    eranosCurrencyDetailsListDestroy(all);
    eranosTextDestroy(text);
}

/// Bytes that are not UTF-8, passed on, would reach the library as text it cannot hold; a null pointer would crash it.
static void textThatIsNotUtf8IsRefusedAndNullReadsAsEmpty(Check *const check) {
    EranosCurrencyDetails *details = nullptr;
    checkThat(check, isKind(eranosCurrencyDetailsOf("\xff", &details), ERANOS_ERROR_KIND_INVALID_TEXT), "the bytes are refused as text");
    checkThat(check, isFailure(eranosCurrencyDetailsOf(nullptr, &details), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY),
              "a null pointer reads as the empty code, which is no currency");
}

/// An amount that parsed, formatted, or added otherwise than the library does would show one number and store another.
static void anAmountParsesFormatsAndAdds(Check *const check) {
    EranosMoney *price = nullptr;
    checkSucceeded(check, eranosMoneyParse("12.5", "EUR", &price), "12.5 is an amount of euros");
    EranosMoney *const quarter = euros(250);
    EranosMoney *sum = nullptr;
    checkSucceeded(check, eranosMoneyAdd(price, quarter, &sum), "two amounts in euros add");
    checkThat(check, isAmount(sum, 1'500, "EUR"), "12.50 EUR and 2.50 EUR make 15.00 EUR");
    EranosMoney *difference = nullptr;
    checkSucceeded(check, eranosMoneySubtract(price, quarter, &difference), "and subtract");
    checkThat(check, isAmount(difference, 1'000, "EUR"), "12.50 EUR less 2.50 EUR is 10.00 EUR");
    EranosMoney *const lei = amountOf(100, "RON");
    EranosMoney *none = nullptr;
    EranosError *const refused = eranosMoneyAdd(price, lei, &none);
    checkThat(check, holdsFailure(refused, ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_CURRENCY_MISMATCH) && nullptr == none, "two currencies");
    checkThat(check, isCurrency(eranosErrorLeft(refused), "EUR") && isCurrency(eranosErrorRight(refused), "RON"), "the error names both currencies");
    eranosErrorDestroy(refused);
    EranosMoney *const largest = euros(INT64_MAX);
    checkThat(check, isFailure(eranosMoneySubtract(lei, largest, &none), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_CURRENCY_MISMATCH),
              "a difference of two currencies");
    checkThat(check, isFailure(eranosMoneyAdd(largest, quarter, &none), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_OVERFLOW),
              "a sum out of range");
    EranosError *const precise = eranosMoneyParse("1.234", "EUR", &none);
    int64_t allowed = 0;
    checkThat(check,
              holdsFailure(precise, ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_TOO_MANY_DECIMALS) && eranosErrorAllowed(precise, &allowed) &&
                  2 == allowed && isCurrency(eranosErrorCurrency(precise), "EUR"),
              "the euro has two decimal places, which the error holds with the currency");
    eranosErrorDestroy(precise);
    eranosMoneyDestroy(largest);
    eranosMoneyDestroy(lei);
    eranosMoneyDestroy(quarter);
    eranosMoneyDestroy(price);
}

/// An amount or a failure that lost its currency's details would leave an app to look the currency up again before it shows a
/// name or the decimal places of a field.
static void anAmountAndAFailureCarryTheirCurrencysDetails(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosMoney *const dinars = amountOf(1'234, "BHD");
    EranosCurrencyDetails *const dinar = eranosMoneyCurrency(dinars);
    eranosCurrencyDetailsName(dinar, text);
    checkThat(check,
              isText(text, "Bahraini Dinar") && 3 == eranosCurrencyDetailsDecimalPlaces(dinar) && 48 == eranosCurrencyDetailsNumericCode(dinar),
              "an amount holds its currency's name, decimal places, and number");
    EranosMoney *const copy = eranosMoneyCreate(5, dinar);
    checkThat(check, isAmount(copy, 5, "BHD"), "and makes another amount in it");
    EranosMoney *none = nullptr;
    EranosError *const precise = eranosMoneyParse("1.2345", "BHD", &none);
    EranosCurrencyDetails *const refused = eranosErrorCurrency(precise);
    checkThat(check, holdsFailure(precise, ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_TOO_MANY_DECIMALS) && nullptr != refused,
              "a failure holds the currency");
    eranosCurrencyDetailsName(refused, text);
    checkThat(check, isText(text, "Bahraini Dinar") && 3 == eranosCurrencyDetailsDecimalPlaces(refused), "with its name and decimal places");
    eranosCurrencyDetailsDestroy(refused);
    eranosErrorDestroy(precise);
    eranosCurrencyDetailsDestroy(dinar);
    eranosMoneyDestroy(dinars);
    eranosTextDestroy(text);
}

/// An amount written with other decimal places than its currency's would show a price ten or a hundred times its value.
static void anAmountFormatsWithItsCurrencysDecimalPlaces(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosMoney *const yen = amountOf(1'234, "JPY");
    eranosMoneyFormat(yen, text);
    checkThat(check, isText(text, "1234 JPY"), "the yen has none");
    EranosMoney *const euro = amountOf(1'234, "EUR");
    eranosMoneyFormat(euro, text);
    checkThat(check, isText(text, "12.34 EUR"), "the euro has two");
    EranosMoney *const dinar = amountOf(1'234, "BHD");
    eranosMoneyFormat(dinar, text);
    checkThat(check, isText(text, "1.234 BHD"), "the Bahraini dinar has three");
    EranosMoney *const unidad = amountOf(12'345, "CLF");
    eranosMoneyFormat(unidad, text);
    checkThat(check, isText(text, "1.2345 CLF"), "the Chilean unidad de fomento has four");
    eranosMoneyDestroy(unidad);
    eranosMoneyDestroy(dinar);
    eranosMoneyDestroy(euro);
    eranosMoneyDestroy(yen);
    eranosTextDestroy(text);
}

/// An app that holds amounts as doubles would get other cents from the C layer than the library rounds to, or an amount back from NaN, from an
/// infinity, or from one past the range of 64-bit minor units.
static void amountsConvertFromDoublesAsTheLibraryConvertsThem(Check *const check) {
    EranosCurrencyDetails *const euro = currencyOf("EUR");
    EranosMoney *money = nullptr;
    checkSucceeded(check, eranosMoneyCreateFromDouble(1.005, euro, &money), "1.005 is an amount of euros");
    checkThat(check, isAmount(money, 101, "EUR"), "1.005 EUR is 1.01 EUR, though the double nearest 1.005 lies below it");
    EranosCurrencyDetails *const yen = currencyOf("JPY");
    checkSucceeded(check, eranosMoneyCreateFromDouble(1'234.5, yen, &money), "1234.5 is an amount of yen");
    checkThat(check, isAmount(money, 1'235, "JPY"), "which rounds half away from zero to the yen's whole units");
    EranosError *const notFinite = eranosMoneyCreateFromDouble(NAN, euro, &money);
    checkThat(check, holdsFailure(notFinite, ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE) && nullptr == money,
              "NaN is no amount, and gives none");
    EranosText *const text = eranosTextCreate();
    eranosErrorMessage(notFinite, text);
    checkThat(check, isText(text, "the number is not finite"), "the message names the failure");
    eranosErrorDestroy(notFinite);
    checkThat(check,
              isFailure(eranosMoneyCreateFromDouble(-INFINITY, euro, &money), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE) &&
                  nullptr == money,
              "nor is an infinity");
    checkThat(check,
              isFailure(eranosMoneyCreateFromDouble(1e21, euro, &money), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_OVERFLOW) && nullptr == money,
              "nor an amount whose minor units do not fit 64 bits");
    eranosTextDestroy(text);
    eranosCurrencyDetailsDestroy(yen);
    eranosCurrencyDetailsDestroy(euro);
}

/// A rate, a percentage, or a number of parts read otherwise than the library reads it would divide or convert otherwise
/// than an app shows, and a check that took what the library refuses would let an app build a split that fails only when added.
static void ratesPercentagesAndPartsParseFormatAndCheckAsTheLibraryDoes(Check *const check) {
    EranosText *const text = eranosTextCreate();
    checkSucceeded(check, eranosExchangeRateParse("4.970", text), "a rate parses");
    checkThat(check, isText(text, "4.97"), "and is written without trailing zeros");
    checkThat(check, isFailure(eranosExchangeRateParse("0", text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_EXCHANGE_RATE),
              "a rate is positive");
    EranosMoney *const lei = amountOf(2, "RON");
    EranosMoney *converted = nullptr;
    checkSucceeded(check, eranosExchangeRateConvert("0.25", lei, "EUR", &converted), "an amount converts");
    checkThat(check, isAmount(converted, 1, "EUR"), "0.02 RON at 0.25 is 0.01 EUR, rounded half away from zero");
    checkThat(check, isKind(eranosExchangeRateConvert("0.25", lei, "\xff", &converted), ERANOS_ERROR_KIND_INVALID_TEXT), "a currency's code is text");
    int32_t basisPoints = 0;
    checkSucceeded(check, eranosPercentageParse("12.5 %", &basisPoints), "a percentage parses");
    eranosPercentageFormat(basisPoints, text);
    checkThat(check, 1'250 == basisPoints && isText(text, "12.5%"), "12.5 % is 1250 basis points");
    checkThat(check,
              isFailure(eranosPercentageParse("half", &basisPoints), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE) &&
                  0 == basisPoints,
              "a word is no percentage, and gives none");
    checkSucceeded(check, eranosPercentageCheck(0), "0 % is a percentage");
    checkSucceeded(check, eranosPercentageCheck(1'250), "and so is 12.5 %");
    checkThat(check, isFailure(eranosPercentageCheck(-1), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE), "a negative one is not");
    eranosPercentageFormat(-1'250, text);
    checkThat(check, isText(text, "-12.5%"), "but is written as any other, after a minus sign");
    int32_t hundredths = 0;
    checkSucceeded(check, eranosPartsParse("0.5", &hundredths), "parts parse");
    eranosPartsFormat(hundredths, text);
    checkThat(check, 50 == hundredths && isText(text, "0.5"), "half a part is 50 hundredths");
    checkThat(check, isFailure(eranosPartsParse("half", &hundredths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PARTS),
              "a word is no parts");
    checkSucceeded(check, eranosPartsCheck(1), "a hundredth of a part is a number of parts");
    checkThat(check,
              isFailure(eranosPartsCheck(0), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PARTS) &&
                  isFailure(eranosPartsCheck(-1), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PARTS),
              "no parts and negative parts are not");
    eranosPartsFormat(0, text);
    checkThat(check, isText(text, "0"), "but zero is written as any number");
    eranosPartsFormat(-50, text);
    checkThat(check, isText(text, "-0.5"), "and so is a negative one");
    eranosMoneyDestroy(lei);
    eranosTextDestroy(text);
}

/// A rate, a percentage, or parts held as a double would become other digits in the C layer than in the library, a number too large for
/// the layer's integers would wrap, and NaN or an infinity would come back as a value.
static void ratesPercentagesAndPartsConvertFromDoublesAsTheLibraryConvertsThem(Check *const check) {
    EranosText *const text = eranosTextCreate();
    checkSucceeded(check, eranosExchangeRateFromDouble(25.5, text), "a rate converts");
    checkThat(check, isText(text, "25.5"), "and keeps the digits of the double's shortest form");
    checkSucceeded(check, eranosExchangeRateFromDouble(0.1 + 0.2, text), "and so does the sum of two doubles");
    checkThat(check, isText(text, "0.30000000000000004"), "with every digit, since a rate has 18 decimal places");
    checkThat(check, isFailure(eranosExchangeRateFromDouble(0.0, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_EXCHANGE_RATE),
              "a rate is positive");
    checkThat(check, isFailure(eranosExchangeRateFromDouble(NAN, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE), "NaN is no rate");
    checkThat(check, isFailure(eranosExchangeRateFromDouble(INFINITY, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE),
              "nor is an infinity");
    int32_t basisPoints = 0;
    checkSucceeded(check, eranosPercentageFromDouble(0.1 + 0.2, &basisPoints), "a percentage converts");
    checkThat(check, 30 == basisPoints, "0.1 + 0.2, which is 0.30000000000000004 as a double, is 0.3 %, 30 basis points");
    checkThat(check,
              isFailure(eranosPercentageFromDouble(-1.0, &basisPoints), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE) &&
                  0 == basisPoints,
              "a negative percentage is not one, and gives none");
    checkThat(check,
              isFailure(eranosPercentageFromDouble(30'000'000.0, &basisPoints), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE),
              "nor is one whose basis points do not fit 32 bits");
    checkThat(check, isFailure(eranosPercentageFromDouble(NAN, &basisPoints), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE), "nor NaN");
    checkThat(check, isFailure(eranosPercentageFromDouble(INFINITY, &basisPoints), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE),
              "nor an infinity");
    int32_t hundredths = 0;
    checkSucceeded(check, eranosPartsFromDouble(1.005, &hundredths), "parts convert");
    checkThat(check, 101 == hundredths, "1.005 parts are 101 hundredths");
    checkThat(check,
              isFailure(eranosPartsFromDouble(0.004, &hundredths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PARTS) && 0 == hundredths,
              "0.004 rounds to no parts, which are not a number of parts, and gives none");
    checkThat(check, isFailure(eranosPartsFromDouble(NAN, &hundredths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE), "nor is NaN");
    checkThat(check, isFailure(eranosPartsFromDouble(-INFINITY, &hundredths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE),
              "nor an infinity");
    eranosTextDestroy(text);
}

/// A quantity read, converted, or written otherwise than the library does would bill a line for another amount than the library divides, a number
/// too large for the layer's integers would wrap, and a check that took what the library refuses would let an app build an item that fails only
/// when added.
static void quantitiesParseConvertFormatAndCheckAsTheLibraryDoes(Check *const check) {
    EranosText *const text = eranosTextCreate();
    int64_t tenThousandths = 0;
    checkSucceeded(check, eranosQuantityParse("0.452", &tenThousandths), "a quantity parses");
    eranosQuantityFormat(tenThousandths, text);
    checkThat(check, 4'520 == tenThousandths && isText(text, "0.452"), "0.452 is 4520 ten-thousandths");
    checkThat(check,
              isFailure(eranosQuantityParse("0.00005", &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY) &&
                  0 == tenThousandths,
              "a quantity has at most four decimal places, and one with more gives none");
    checkThat(check, isFailure(eranosQuantityParse("0", &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY),
              "no quantity is zero");
    checkThat(check, isFailure(eranosQuantityParse("-1", &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY),
              "nor negative");
    checkThat(check,
              isFailure(eranosQuantityParse("1000000000000000", &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY),
              "nor one whose ten-thousandths do not fit 64 bits");
    checkThat(check, isKind(eranosQuantityParse("\xff", &tenThousandths), ERANOS_ERROR_KIND_INVALID_TEXT), "a quantity's text is text");
    checkSucceeded(check, eranosQuantityFromWhole(3, &tenThousandths), "whole units make a quantity");
    checkThat(check, 30'000 == tenThousandths, "3 units are 30000 ten-thousandths");
    checkThat(check,
              isFailure(eranosQuantityFromWhole(0, &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY) &&
                  0 == tenThousandths &&
                  isFailure(eranosQuantityFromWhole(-1, &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY),
              "no units and negative units are no quantity, and give none");
    checkThat(check, isFailure(eranosQuantityFromWhole(INT64_MAX, &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY),
              "nor are units whose ten-thousandths do not fit 64 bits");
    checkSucceeded(check, eranosQuantityFromDouble(0.45205, &tenThousandths), "a double converts");
    checkThat(check, 4'521 == tenThousandths, "0.45205 rounds half away from zero to 4521 ten-thousandths");
    checkThat(check,
              isFailure(eranosQuantityFromDouble(0.00004, &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY) &&
                  0 == tenThousandths,
              "0.00004 rounds to no quantity, and gives none");
    checkThat(check, isFailure(eranosQuantityFromDouble(-1.0, &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY),
              "a negative double is no quantity");
    checkThat(check, isFailure(eranosQuantityFromDouble(NAN, &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE),
              "nor is NaN");
    checkThat(check, isFailure(eranosQuantityFromDouble(INFINITY, &tenThousandths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_NOT_FINITE),
              "nor an infinity");
    checkSucceeded(check, eranosQuantityCheck(1), "a ten-thousandth of a unit is a quantity");
    checkSucceeded(check, eranosQuantityCheck(ERANOS_QUANTITY_ONE), "and so is one unit");
    checkThat(check,
              isFailure(eranosQuantityCheck(0), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY) &&
                  isFailure(eranosQuantityCheck(-1), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY),
              "no quantity and a negative one are not");
    eranosQuantityFormat(ERANOS_QUANTITY_ONE, text);
    checkThat(check, isText(text, "1"), "one unit is written as a whole number");
    eranosQuantityFormat(0, text);
    checkThat(check, isText(text, "0"), "zero is written as any number");
    eranosQuantityFormat(-4'520, text);
    checkThat(check, isText(text, "-0.452"), "and so is a negative one");
    eranosTextDestroy(text);
}

/// A line total the layer multiplied itself would round a fractional quantity otherwise than the library, which divides the line, and so show an
/// app a receipt that does not add up to the shares.
static void anItemsLineTotalIsTheLibrarysProductRoundedHalfAwayFromZero(Check *const check) {
    EranosMoney *const fortyTwo = euros(4'200);
    EranosSplit *const ana = equally("ana", nullptr);
    EranosExpenseItem *item = nullptr;
    checkSucceeded(check, eranosExpenseItemCreate("Cheese", fortyTwo, ana, &item), "an item");
    EranosMoney *lineTotal = nullptr;
    checkSucceeded(check, eranosExpenseItemLineTotal(item, &lineTotal), "an item has a line total");
    checkThat(check, isAmount(lineTotal, 4'200, "EUR"), "one unit, by default, costs the unit price");
    eranosExpenseItemSetQuantity(item, 20'000);
    checkSucceeded(check, eranosExpenseItemLineTotal(item, &lineTotal), "whole units have a line total");
    checkThat(check, isAmount(lineTotal, 8'400, "EUR"), "2 at 42.00 EUR are 84.00 EUR, exactly");
    eranosExpenseItemSetQuantity(item, 3'500);
    checkSucceeded(check, eranosExpenseItemLineTotal(item, &lineTotal), "a fraction of a unit has a line total");
    checkThat(check, isAmount(lineTotal, 1'470, "EUR"), "0.35 at 42.00 EUR is 14.70 EUR");
    EranosMoney *const one = euros(100);
    eranosExpenseItemSetUnitPrice(item, one);
    eranosExpenseItemSetQuantity(item, 3'350);
    checkSucceeded(check, eranosExpenseItemLineTotal(item, &lineTotal), "and so does one that rounds");
    checkThat(check, isAmount(lineTotal, 34, "EUR"), "0.335 at 1.00 EUR is 33.5 cents, which rounds half away from zero to 0.34 EUR");
    eranosExpenseItemSetQuantity(item, 0);
    checkThat(check,
              isFailure(eranosExpenseItemLineTotal(item, &lineTotal), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_QUANTITY) &&
                  nullptr == lineTotal,
              "an item of no quantity has no line total");
    eranosMoneyDestroy(one);
    eranosExpenseItemDestroy(item);
    eranosSplitDestroy(ana);
    eranosMoneyDestroy(fortyTwo);
}

/// A discount of @p value, which it destroys, off @p scope.
[[nodiscard]] static EranosDiscount *discountOf(EranosDiscountValue *const value, EranosDiscountScope const scope) {
    EranosDiscount *const discount = eranosDiscountCreate(value);
    eranosDiscountSetScope(discount, scope);
    eranosDiscountValueDestroy(value);
    return discount;
}

/// A discount of @p basisPoints basis points off @p scope.
[[nodiscard]] static EranosDiscount *percentageOff(int32_t const basisPoints, EranosDiscountScope const scope) {
    return discountOf(eranosDiscountValueCreatePercentage(basisPoints), scope);
}

/// A discount of @p minorUnits minor units of the currency of the ISO 4217 code @p code off @p scope.
[[nodiscard]] static EranosDiscount *amountOff(int64_t const minorUnits, char const *const code, EranosDiscountScope const scope) {
    EranosMoney *const amount = amountOf(minorUnits, code);
    EranosDiscount *const discount = discountOf(eranosDiscountValueCreateFixed(amount), scope);
    eranosMoneyDestroy(amount);
    return discount;
}

/// A line of @p tenThousandths ten-thousandths of a unit at @p cents euro cents a unit, which Ana had, with @p discount, which it destroys, unless
/// nullptr.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the unit price, then the quantity, in the order of the item's fields.
[[nodiscard]] static EranosExpenseItem *lineOf(int64_t const cents, int64_t const tenThousandths, EranosDiscount *const discount) {
    EranosMoney *const unitPrice = euros(cents);
    EranosSplit *const ana = equally("ana", nullptr);
    EranosExpenseItem *item = nullptr;
    eranosErrorDestroy(eranosExpenseItemCreate("Cheese", unitPrice, ana, &item));
    eranosExpenseItemSetQuantity(item, tenThousandths);
    if (nullptr != discount) {
        eranosExpenseItemSetDiscount(item, discount);
    }
    eranosDiscountDestroy(discount);
    eranosSplitDestroy(ana);
    eranosMoneyDestroy(unitPrice);
    return item;
}

/// The totals of a line, in euro cents.
typedef struct LineTotals {
    /// The unit price times the quantity.
    int64_t lineTotalBeforeDiscount;
    /// What the discount takes off.
    int64_t discountAmount;
    /// What the line comes to.
    int64_t lineTotal;
} LineTotals;

/// Whether the library computes @p expected for @p item, which it destroys.
[[nodiscard]] static bool totalsAre(EranosExpenseItem *const item, LineTotals const expected) {
    EranosMoney *lineTotalBeforeDiscount = nullptr;
    EranosMoney *discountAmount = nullptr;
    EranosMoney *lineTotal = nullptr;
    EranosError *const beforeError = eranosExpenseItemLineTotalBeforeDiscount(item, &lineTotalBeforeDiscount);
    EranosError *const discountError = eranosExpenseItemDiscountAmount(item, &discountAmount);
    EranosError *const lineError = eranosExpenseItemLineTotal(item, &lineTotal);
    bool const computed = nullptr == beforeError && nullptr == discountError && nullptr == lineError;
    bool const sameBefore = isAmount(lineTotalBeforeDiscount, expected.lineTotalBeforeDiscount, "EUR");
    bool const sameDiscount = isAmount(discountAmount, expected.discountAmount, "EUR");
    bool const sameLineTotal = isAmount(lineTotal, expected.lineTotal, "EUR");
    eranosErrorDestroy(lineError);
    eranosErrorDestroy(discountError);
    eranosErrorDestroy(beforeError);
    eranosExpenseItemDestroy(item);
    return computed && sameBefore && sameDiscount && sameLineTotal;
}

/// Whether every total of @p item, which it destroys, is refused with an error of @p kind with @p failure, and gives no amount.
[[nodiscard]] static bool totalsRefused(EranosExpenseItem *const item, EranosErrorKind const kind, EranosFailure const failure) {
    EranosMoney *lineTotalBeforeDiscount = nullptr;
    EranosMoney *discountAmount = nullptr;
    EranosMoney *lineTotal = nullptr;
    bool const refused = isFailure(eranosExpenseItemLineTotalBeforeDiscount(item, &lineTotalBeforeDiscount), kind, failure) &&
                         isFailure(eranosExpenseItemDiscountAmount(item, &discountAmount), kind, failure) &&
                         isFailure(eranosExpenseItemLineTotal(item, &lineTotal), kind, failure);
    bool const empty = nullptr == lineTotalBeforeDiscount && nullptr == discountAmount && nullptr == lineTotal;
    eranosMoneyDestroy(lineTotal);
    eranosMoneyDestroy(discountAmount);
    eranosMoneyDestroy(lineTotalBeforeDiscount);
    eranosExpenseItemDestroy(item);
    return refused && empty;
}

/// A discount that read back otherwise than it was made would take another amount off a line than the receipt shows, and one that started off each
/// unit would take a line's discount from every unit.
static void aDiscountReadsBackAsItWasMadeAndStartsOffTheLine(Check *const check) {
    EranosMoney *const one = euros(100);
    EranosDiscountValue *const fixed = eranosDiscountValueCreateFixed(one);
    EranosDiscountValue *const half = eranosDiscountValueCreatePercentage(5'000);
    int32_t basisPoints = 0;
    checkThat(check,
              ERANOS_DISCOUNT_VALUE_KIND_FIXED == eranosDiscountValueKind(fixed) && isAmount(eranosDiscountValueAmount(fixed), 100, "EUR") &&
                  !eranosDiscountValueBasisPoints(fixed, &basisPoints),
              "a fixed discount holds its amount");
    checkThat(check,
              ERANOS_DISCOUNT_VALUE_KIND_PERCENTAGE == eranosDiscountValueKind(half) && eranosDiscountValueBasisPoints(half, &basisPoints) &&
                  5'000 == basisPoints && nullptr == eranosDiscountValueAmount(half),
              "a percentage discount holds its basis points");
    EranosDiscount *const discount = eranosDiscountCreate(fixed);
    EranosDiscountValue *const madeValue = eranosDiscountValue(discount);
    checkThat(check,
              ERANOS_DISCOUNT_SCOPE_LINE == eranosDiscountScope(discount) && ERANOS_DISCOUNT_VALUE_KIND_FIXED == eranosDiscountValueKind(madeValue),
              "a discount starts off the line");
    eranosDiscountSetValue(discount, half);
    eranosDiscountSetScope(discount, ERANOS_DISCOUNT_SCOPE_UNIT);
    EranosDiscountValue *const changedValue = eranosDiscountValue(discount);
    checkThat(check,
              ERANOS_DISCOUNT_SCOPE_UNIT == eranosDiscountScope(discount) && eranosDiscountValueBasisPoints(changedValue, &basisPoints) &&
                  5'000 == basisPoints,
              "a discount as set");
    EranosSplit *const ana = equally("ana", nullptr);
    EranosExpenseItem *item = nullptr;
    checkSucceeded(check, eranosExpenseItemCreate("Cheese", one, ana, &item), "an item");
    checkThat(check, nullptr == eranosExpenseItemDiscount(item), "an item starts without a discount");
    eranosExpenseItemSetDiscount(item, discount);
    EranosDiscount *const itemDiscount = eranosExpenseItemDiscount(item);
    EranosDiscountValue *const itemValue = nullptr == itemDiscount ? nullptr : eranosDiscountValue(itemDiscount);
    checkThat(check,
              nullptr != itemDiscount && ERANOS_DISCOUNT_SCOPE_UNIT == eranosDiscountScope(itemDiscount) && nullptr != itemValue &&
                  eranosDiscountValueBasisPoints(itemValue, &basisPoints) && 5'000 == basisPoints,
              "an item holds the discount it is given");
    eranosExpenseItemClearDiscount(item);
    checkThat(check, nullptr == eranosExpenseItemDiscount(item), "and clears it");
    eranosDiscountValueDestroy(itemValue);
    eranosDiscountDestroy(itemDiscount);
    eranosExpenseItemDestroy(item);
    eranosSplitDestroy(ana);
    eranosDiscountValueDestroy(changedValue);
    eranosDiscountValueDestroy(madeValue);
    eranosDiscountDestroy(discount);
    eranosDiscountValueDestroy(half);
    eranosDiscountValueDestroy(fixed);
    eranosMoneyDestroy(one);
}

/// A discount the layer took itself would round a line otherwise than the library, which divides it, and so show an app a receipt that does not
/// add up to the shares: off each unit, the library rounds each unit's discount, so 50 % off three units of 3.33 EUR leaves a cent less than 50 %
/// off their line.
static void aDiscountComesOffEachUnitOrOffTheLineAsTheLibraryTakesIt(Check *const check) {
    checkThat(check,
              totalsAre(lineOf(333, 30'000, percentageOff(5'000, ERANOS_DISCOUNT_SCOPE_UNIT)),
                        (LineTotals){.lineTotalBeforeDiscount = 999, .discountAmount = 501, .lineTotal = 498}),
              "50 % off each of 3 at 3.33 EUR leaves 1.66 EUR a unit, 4.98 EUR");
    checkThat(check,
              totalsAre(lineOf(333, 30'000, percentageOff(5'000, ERANOS_DISCOUNT_SCOPE_LINE)),
                        (LineTotals){.lineTotalBeforeDiscount = 999, .discountAmount = 500, .lineTotal = 499}),
              "50 % off their line of 9.99 EUR is 5.00 EUR, rounded half away from zero, which leaves 4.99 EUR");
    checkThat(check,
              totalsAre(lineOf(300, 30'000, amountOff(100, "EUR", ERANOS_DISCOUNT_SCOPE_UNIT)),
                        (LineTotals){.lineTotalBeforeDiscount = 900, .discountAmount = 300, .lineTotal = 600}),
              "1.00 EUR off each of 3 at 3.00 EUR is 3.00 EUR off their line");
    checkThat(check,
              totalsAre(lineOf(300, 30'000, amountOff(100, "EUR", ERANOS_DISCOUNT_SCOPE_LINE)),
                        (LineTotals){.lineTotalBeforeDiscount = 900, .discountAmount = 100, .lineTotal = 800}),
              "1.00 EUR off their line of 9.00 EUR leaves 8.00 EUR");
    checkThat(check,
              totalsAre(lineOf(900, ERANOS_QUANTITY_ONE, percentageOff(10'000, ERANOS_DISCOUNT_SCOPE_LINE)),
                        (LineTotals){.lineTotalBeforeDiscount = 900, .discountAmount = 900, .lineTotal = 0}),
              "100 % off makes a free line");
    checkThat(check,
              totalsAre(lineOf(900, ERANOS_QUANTITY_ONE, amountOff(900, "EUR", ERANOS_DISCOUNT_SCOPE_UNIT)),
                        (LineTotals){.lineTotalBeforeDiscount = 900, .discountAmount = 900, .lineTotal = 0}),
              "and so does the whole price off its unit");
    checkThat(
        check,
        totalsAre(lineOf(900, ERANOS_QUANTITY_ONE, nullptr), (LineTotals){.lineTotalBeforeDiscount = 900, .discountAmount = 0, .lineTotal = 900}),
        "a line without a discount loses nothing");
}

/// A discount the library refuses, taken, would bill a line below zero or above its price, or mix currencies on one receipt; every total of the
/// item refuses it with the failure of the rule it breaks, and gives no amount.
static void aDiscountTheLineCannotTakeIsRefusedByEveryTotal(Check *const check) {
    checkThat(check,
              totalsRefused(lineOf(900, ERANOS_QUANTITY_ONE, amountOff(-100, "EUR", ERANOS_DISCOUNT_SCOPE_LINE)), ERANOS_ERROR_KIND_EXPENSE,
                            ERANOS_FAILURE_EXPENSE_NEGATIVE_DISCOUNT),
              "a fixed discount below zero");
    checkThat(check,
              totalsRefused(lineOf(900, ERANOS_QUANTITY_ONE, percentageOff(10'001, ERANOS_DISCOUNT_SCOPE_LINE)), ERANOS_ERROR_KIND_EXPENSE,
                            ERANOS_FAILURE_EXPENSE_DISCOUNT_ABOVE_HUNDRED_PERCENT),
              "a percentage above 100 %");
    checkThat(check,
              totalsRefused(lineOf(900, ERANOS_QUANTITY_ONE, percentageOff(-1, ERANOS_DISCOUNT_SCOPE_LINE)), ERANOS_ERROR_KIND_MONEY,
                            ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE),
              "a percentage below zero, which no percentage is");
    checkThat(check,
              totalsRefused(lineOf(900, ERANOS_QUANTITY_ONE, amountOff(1'000, "EUR", ERANOS_DISCOUNT_SCOPE_LINE)), ERANOS_ERROR_KIND_EXPENSE,
                            ERANOS_FAILURE_EXPENSE_DISCOUNT_ABOVE_PRICE),
              "10.00 EUR off a line of 9.00 EUR");
    checkThat(check,
              totalsRefused(lineOf(300, 30'000, amountOff(301, "EUR", ERANOS_DISCOUNT_SCOPE_UNIT)), ERANOS_ERROR_KIND_EXPENSE,
                            ERANOS_FAILURE_EXPENSE_DISCOUNT_ABOVE_PRICE),
              "3.01 EUR off each unit of 3.00 EUR, though their line is 9.00 EUR");
    checkThat(check,
              totalsRefused(lineOf(900, ERANOS_QUANTITY_ONE, amountOff(100, "RON", ERANOS_DISCOUNT_SCOPE_LINE)), ERANOS_ERROR_KIND_EXPENSE,
                            ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH),
              "a fixed discount in another currency than the price");
}

/// The lines of the itemised expense @p id in @p group, which the caller destroys, or nullptr when the group holds none.
[[nodiscard]] static EranosExpenseItemList *linesOf(EranosExpenseGroup const *const group, char const *const id) {
    EranosExpense *expense = nullptr;
    eranosErrorDestroy(eranosExpenseGroupExpense(group, id, &expense));
    if (nullptr == expense) {
        return nullptr;
    }
    EranosExpenseSplit *const split = eranosExpenseSplit(expense);
    EranosExpenseItemList *const items = eranosExpenseSplitItems(split);
    eranosExpenseSplitDestroy(split);
    eranosExpenseDestroy(expense);
    return items;
}

/// The discount of the line at @p index of @p items, which the caller destroys, or nullptr when the line has none or the list ends before it.
[[nodiscard]] static EranosDiscount *discountAt(EranosExpenseItemList const *const items, size_t const index) {
    EranosExpenseItem *const item = nullptr == items ? nullptr : eranosExpenseItemListGet(items, index);
    if (nullptr == item) {
        return nullptr;
    }
    EranosDiscount *const discount = eranosExpenseItemDiscount(item);
    eranosExpenseItemDestroy(item);
    return discount;
}

/// Whether @p discount, which it destroys, takes @p basisPoints basis points off @p scope.
[[nodiscard]] static bool isPercentageOff(EranosDiscount *const discount, int32_t const basisPoints, EranosDiscountScope const scope) {
    if (nullptr == discount) {
        return false;
    }
    EranosDiscountValue *const value = eranosDiscountValue(discount);
    int32_t held = -1;
    bool const same = scope == eranosDiscountScope(discount) && eranosDiscountValueBasisPoints(value, &held) && basisPoints == held;
    eranosDiscountValueDestroy(value);
    eranosDiscountDestroy(discount);
    return same;
}

/// Whether @p discount, which it destroys, takes @p cents euro cents off @p scope.
[[nodiscard]] static bool isAmountOff(EranosDiscount *const discount, int64_t const cents, EranosDiscountScope const scope) {
    if (nullptr == discount) {
        return false;
    }
    EranosDiscountValue *const value = eranosDiscountValue(discount);
    bool const same = scope == eranosDiscountScope(discount) && isAmount(eranosDiscountValueAmount(value), cents, "EUR");
    eranosDiscountValueDestroy(value);
    eranosDiscountDestroy(discount);
    return same;
}

/// A discount lost or changed on its way through the group would bill a line at full price once an app reads the expense back, or take a unit's
/// discount off the line.
static void aDiscountedLineReadsBackFromTheGroupAsItWasAdded(Check *const check) {
    Flat const flat = flatOf();
    EranosSplit *const both = equally(eranosTextData(flat.ana), eranosTextData(flat.bob));
    EranosExpenseItemList *const items = eranosExpenseItemListCreate();
    EranosMoney *const cheesePrice = euros(333);
    EranosExpenseItem *cheese = nullptr;
    checkSucceeded(check, eranosExpenseItemCreate("Cheese", cheesePrice, both, &cheese), "a line of cheese");
    eranosExpenseItemSetQuantity(cheese, 30'000);
    EranosDiscount *const halfOffEach = percentageOff(5'000, ERANOS_DISCOUNT_SCOPE_UNIT);
    eranosExpenseItemSetDiscount(cheese, halfOffEach);
    eranosExpenseItemListPush(items, cheese);
    EranosMoney *const winePrice = euros(900);
    EranosExpenseItem *wine = nullptr;
    checkSucceeded(check, eranosExpenseItemCreate("Wine", winePrice, both, &wine), "a line of wine");
    EranosDiscount *const oneOffTheLine = amountOff(100, "EUR", ERANOS_DISCOUNT_SCOPE_LINE);
    eranosExpenseItemSetDiscount(wine, oneOffTheLine);
    eranosExpenseItemListPush(items, wine);
    EranosMoney *const breadPrice = euros(200);
    EranosExpenseItem *bread = nullptr;
    checkSucceeded(check, eranosExpenseItemCreate("Bread", breadPrice, both, &bread), "a line of bread");
    eranosExpenseItemListPush(items, bread);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateItemised(items);
    // The lines come to 4.98, 8.00, and 2.00 EUR.
    EranosMoney *const amount = euros(1'498);
    EranosCost *const cost = eranosCostCreateLocal(amount);
    EranosSplit *const payers = equally(eranosTextData(flat.ana), nullptr);
    EranosExpense *expense = nullptr;
    checkSucceeded(check, eranosExpenseCreate("Market", cost, payers, split, DAY, &expense), "an itemised expense");
    EranosText *const id = eranosTextCreate();
    checkSucceeded(check, eranosExpenseGroupAddExpense(flat.group, expense, id), "the group takes lines with discounts");
    EranosExpenseItemList *const lines = linesOf(flat.group, eranosTextData(id));
    checkThat(check, nullptr != lines && 3 == eranosExpenseItemListLength(lines), "and holds every line");
    checkThat(check, isPercentageOff(discountAt(lines, 0), 5'000, ERANOS_DISCOUNT_SCOPE_UNIT), "50 % off each unit reads back as it went in");
    checkThat(check, isAmountOff(discountAt(lines, 1), 100, ERANOS_DISCOUNT_SCOPE_LINE), "and so does 1.00 EUR off the line");
    checkThat(check, nullptr == discountAt(lines, 2), "and a line at full price, without a discount");
    eranosExpenseItemListDestroy(lines);
    eranosTextDestroy(id);
    eranosExpenseDestroy(expense);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(amount);
    eranosExpenseSplitDestroy(split);
    eranosExpenseItemDestroy(bread);
    eranosMoneyDestroy(breadPrice);
    eranosDiscountDestroy(oneOffTheLine);
    eranosExpenseItemDestroy(wine);
    eranosMoneyDestroy(winePrice);
    eranosDiscountDestroy(halfOffEach);
    eranosExpenseItemDestroy(cheese);
    eranosMoneyDestroy(cheesePrice);
    eranosExpenseItemListDestroy(items);
    eranosSplitDestroy(both);
    flatDestroy(flat);
}

/// A date or a time the calendar does not have, taken, would date an expense on a day that never was.
static void datesAndTimesAreCheckedAndWritten(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosCalendarDate date = {.year = 0, .month = 0, .day = 0};
    checkSucceeded(check, eranosCalendarDateParse("2026-09-23", &date), "a date parses");
    checkThat(check, 2026 == date.year && 9 == date.month && 23 == date.day, "into its year, month, and day");
    eranosCalendarDateFormat(date, text);
    checkThat(check, isText(text, "2026-09-23"), "and formats as the library writes it");
    checkSucceeded(check, eranosCalendarDateCheck(date), "a day of the calendar passes the check");
    constexpr EranosCalendarDate LEAP = {.year = 2026, .month = 2, .day = 30};
    checkThat(check, isFailure(eranosCalendarDateCheck(LEAP), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_DATE),
              "30 February does not");
    constexpr EranosCalendarDate FAR = {.year = 10'000, .month = 1, .day = 1};
    checkThat(check, isFailure(eranosCalendarDateCheck(FAR), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_DATE),
              "nor does year 10000");
    eranosCalendarDateFormat(LEAP, text);
    checkThat(check, isText(text, "2026-02-30"), "but a day the calendar does not have is written as any other");
    checkThat(check,
              isFailure(eranosCalendarDateParse("2026-02-30", &date), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_DATE) &&
                  0 == date.day,
              "parsed, it gives no date");
    EranosTimeOfDay time = {.hour = 1, .minute = 1};
    checkSucceeded(check, eranosTimeOfDayParse("18:30", &time), "a time parses");
    eranosTimeOfDayFormat(time, text);
    checkThat(check, 18 == time.hour && 30 == time.minute && isText(text, "18:30"), "into its hour and minute, and formats");
    checkSucceeded(check, eranosTimeOfDayCheck(time), "a time of day passes the check");
    constexpr EranosTimeOfDay MIDNIGHT = {.hour = 24, .minute = 0};
    constexpr EranosTimeOfDay LATE = {.hour = 23, .minute = 60};
    checkThat(check,
              isFailure(eranosTimeOfDayCheck(MIDNIGHT), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_TIME) &&
                  isFailure(eranosTimeOfDayCheck(LATE), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_TIME),
              "hour 24 and minute 60 do not");
    eranosTimeOfDayFormat(MIDNIGHT, text);
    checkThat(check, isText(text, "24:00"), "but a time the day does not have is written as any other");
    checkThat(check, isFailure(eranosTimeOfDayParse("25:00", &time), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_TIME), "parsed");
    eranosTextDestroy(text);
}

/// Contact details and accounts kept as typed would differ from the same details typed otherwise, and a wrong one would pass.
static void contactsAndAccountsAreCheckedAndMadeCanonical(Check *const check) {
    EranosText *const text = eranosTextCreate();
    checkSucceeded(check, eranosEmailParse(" ana@example.com ", text), "an email parses");
    checkThat(check, isText(text, "ana@example.com"), "trimmed");
    checkThat(check, isFailure(eranosEmailParse("ana", text), ERANOS_ERROR_KIND_CONTACT, ERANOS_FAILURE_CONTACT_INVALID_EMAIL),
              "an email has a domain");
    EranosPhoneNumber *phone = nullptr;
    checkSucceeded(check, eranosPhoneNumberParse("+40 712 345 678", &phone), "a phone number parses");
    eranosPhoneNumberE164(phone, text);
    checkThat(check, isText(text, "+40712345678"), "into its E.164 form");
    eranosPhoneNumberInternational(phone, text);
    checkThat(check, isText(text, "+40 712 345 678"), "and its international form, with spaces");
    eranosPhoneNumberDestroy(phone);
    checkThat(check,
              isFailure(eranosPhoneNumberParse("0712345678", &phone), ERANOS_ERROR_KIND_CONTACT, ERANOS_FAILURE_CONTACT_INVALID_PHONE_NUMBER) &&
                  nullptr == phone,
              "a phone number has a country code, and none is made without one");
    checkThat(check, isKind(eranosPhoneNumberParse("\xff", &phone), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == phone, "a phone number is text");
    EranosIban *iban = nullptr;
    checkSucceeded(check, eranosIbanParse("ro49 aaaa 1b31 0075 9384 0000", &iban), "an IBAN parses");
    eranosIbanIban(iban, text);
    checkThat(check, isText(text, "RO49AAAA1B31007593840000"), "into its electronic form");
    eranosIbanCountryCode(iban, text);
    checkThat(check, isText(text, "RO"), "and names its country, Romania");
    eranosIbanDestroy(iban);
    EranosError *const checksum = eranosIbanParse("RO49AAAA1B31007593840001", &iban);
    EranosIbanReason reason = ERANOS_IBAN_REASON_INVALID_FORMAT;
    checkThat(check,
              holdsFailure(checksum, ERANOS_ERROR_KIND_PAYMENT_ACCOUNT, ERANOS_FAILURE_PAYMENT_ACCOUNT_INVALID_IBAN) &&
                  eranosErrorReason(checksum, &reason) && ERANOS_IBAN_REASON_INVALID_CHECKSUM == reason && nullptr == iban,
              "an IBAN whose check digits do not match, and none is made");
    eranosErrorMessage(checksum, text);
    checkThat(check, isText(text, "invalid IBAN: the check digits do not match"), "the message names the failure, not the text");
    eranosErrorDestroy(checksum);
    EranosError *const length = eranosIbanParse("RO49AAAA1B3100759384000", &iban);
    int64_t expected = 0;
    int64_t found = 0;
    checkThat(check, eranosErrorExpectedLength(length, &expected) && 24 == expected && eranosErrorFoundLength(length, &found) && 23 == found,
              "an IBAN one character short holds both lengths");
    eranosErrorDestroy(length);
    checkThat(check, isKind(eranosIbanParse("\xff", &iban), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == iban, "an IBAN is text");
    checkSucceeded(check, eranosUsernameParse(" ana.pays ", text), "a username parses");
    checkThat(check, isText(text, "ana.pays"), "trimmed");
    checkThat(check, isFailure(eranosUsernameParse(" ", text), ERANOS_ERROR_KIND_PAYMENT_ACCOUNT, ERANOS_FAILURE_PAYMENT_ACCOUNT_EMPTY_USERNAME),
              "a blank username");
    checkSucceeded(check, eranosLanguageParse("pt-pt", text), "a language parses");
    checkThat(check, isText(text, "pt-PT"), "into its canonical tag");
    checkThat(check, isFailure(eranosLanguageParse("not a tag", text), ERANOS_ERROR_KIND_LANGUAGE, ERANOS_FAILURE_LANGUAGE_INVALID_LANGUAGE),
              "no tag");
    eranosTextDestroy(text);
}

/// Text that is not an identifier, and the failure the library refuses it with.
typedef struct RefusedText {
    /// The failure.
    EranosFailure failure;
    /// The text, as UTF-8 ended by a NUL.
    char const *text;
} RefusedText;

/// An identifier kept otherwise than it was written, trimmed or in another case, would no longer match the key the app gave, and text
/// the library refuses, taken, would fail every call that carries it; each refusal names the first rule the text breaks.
static void anIdentifierIsCheckedAndKeptAsItIsWritten(Check *const check) {
    EranosText *const text = eranosTextCreate();
    checkSucceeded(check, eranosIdentifierParse("Ana-1", text), "an identifier parses");
    checkThat(check, isText(text, "Ana-1"), "as it is written, in its case");
    checkSucceeded(check, eranosIdentifierParse("caf\u00e9 \u2615", text), "an identifier of any text the rules accept");
    checkThat(check, isText(text, "caf\u00e9 \u2615"), "byte for byte");
    char letters[66] = {};
    memset(letters, 'a', sizeof letters - 1);
    RefusedText const refused[] = {
        {.failure = ERANOS_FAILURE_ID_EMPTY, .text = ""},
        {.failure = ERANOS_FAILURE_ID_TOO_LONG, .text = letters},
        {.failure = ERANOS_FAILURE_ID_CONTROL_CHARACTER, .text = "a\tb"},
        {.failure = ERANOS_FAILURE_ID_SURROUNDING_WHITESPACE, .text = " ana"},
    };
    for (size_t index = 0; index < sizeof refused / sizeof refused[0]; ++index) {
        checkThat(check, isFailure(eranosIdentifierParse(refused[index].text, text), ERANOS_ERROR_KIND_ID, refused[index].failure),
                  "text that breaks a rule");
    }
    checkThat(check, isKind(eranosIdentifierParse("\xff", text), ERANOS_ERROR_KIND_INVALID_TEXT), "bytes that are not UTF-8");
    eranosTextDestroy(text);
}

/// A generated identifier that repeated an earlier one would put a new entry in the place of another, and one the rules refused would fail
/// every call that carries it.
static void aGeneratedIdentifierIsNewAndOneTheRulesAccept(Check *const check) {
    EranosText *const first = eranosTextCreate();
    EranosText *const second = eranosTextCreate();
    EranosText *const parsed = eranosTextCreate();
    eranosIdentifierGenerate(first);
    eranosIdentifierGenerate(second);
    checkThat(check, !isText(first, eranosTextData(second)), "two generated identifiers differ");
    checkSucceeded(check, eranosIdentifierParse(eranosTextData(first), parsed), "a generated identifier parses");
    checkThat(check, isText(parsed, eranosTextData(first)), "as it was generated");
    checkSucceeded(check, eranosIdentifierParse(eranosTextData(second), parsed), "and so does the next");
    checkThat(check, isText(parsed, eranosTextData(second)), "as it was generated");
    eranosTextDestroy(parsed);
    eranosTextDestroy(second);
    eranosTextDestroy(first);
}

/// A list of identifiers that reordered or dropped one would give a member's share to another.
static void aListOfIdentifiersKeepsItsOrder(Check *const check) {
    EranosTextList *const list = eranosTextListCreate();
    checkThat(check, eranosTextListIsEmpty(list), "a list starts empty");
    checkSucceeded(check, eranosTextListPush(list, "first"), "text joins a list");
    checkThat(check, isKind(eranosTextListPush(list, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) && 1 == eranosTextListLength(list),
              "bytes that are not UTF-8 do not");
    checkSucceeded(check, eranosTextListPush(list, "second"), "a second joins it");
    checkThat(check, listsMembers(list, "first", "second"), "the list holds both, in order");
}

// --- Entries ------------------------------------------------------------------------------------------------------------

/// A member's share that read back otherwise than it was made would divide an amount among the wrong members.
static void everyShareOfASplitReadsBackAsItWasMade(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosMoney *const ten = euros(1'000);
    EranosMemberAmount *amount = nullptr;
    checkSucceeded(check, eranosMemberAmountCreate("ana", ten, &amount), "a member's amount");
    checkThat(check, isAmount(eranosMemberAmountAmount(amount), 1'000, "EUR"), "its amount");
    eranosMemberAmountMember(amount, text);
    checkThat(check, isText(text, "ana"), "its member");
    EranosMoney *const twenty = euros(2'000);
    eranosMemberAmountSetAmount(amount, twenty);
    checkSucceeded(check, eranosMemberAmountSetMember(amount, "bob"), "a member is set");
    eranosMemberAmountMember(amount, text);
    checkThat(check, isAmount(eranosMemberAmountAmount(amount), 2'000, "EUR") && isText(text, "bob"), "the last value set wins");
    checkThat(check, isKind(eranosMemberAmountSetMember(amount, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a member is text");
    EranosMemberAmountList *const amounts = eranosMemberAmountListCreate();
    checkThat(check, eranosMemberAmountListIsEmpty(amounts), "an empty list of amounts");
    eranosMemberAmountListPush(amounts, amount);
    EranosMemberAmount *const firstAmount = eranosMemberAmountListGet(amounts, 0);
    checkThat(check, 1 == eranosMemberAmountListLength(amounts) && nullptr != firstAmount && nullptr == eranosMemberAmountListGet(amounts, 1),
              "one amount");
    eranosMemberAmountDestroy(firstAmount);
    EranosMemberParts *parts = nullptr;
    checkSucceeded(check, eranosMemberPartsCreate("ana", 100, &parts), "a member's parts");
    eranosMemberPartsSetHundredths(parts, 50);
    checkSucceeded(check, eranosMemberPartsSetMember(parts, "bob"), "a member is set");
    eranosMemberPartsMember(parts, text);
    checkThat(check, 50 == eranosMemberPartsHundredths(parts) && isText(text, "bob"), "half a part for Bob");
    checkThat(check, isKind(eranosMemberPartsSetMember(parts, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a member is text");
    EranosMemberPartsList *const partsList = eranosMemberPartsListCreate();
    checkThat(check, eranosMemberPartsListIsEmpty(partsList), "an empty list of parts");
    eranosMemberPartsListPush(partsList, parts);
    EranosMemberParts *const firstParts = eranosMemberPartsListGet(partsList, 0);
    checkThat(check, 1 == eranosMemberPartsListLength(partsList) && nullptr != firstParts && nullptr == eranosMemberPartsListGet(partsList, 1),
              "one");
    eranosMemberPartsDestroy(firstParts);
    EranosMemberPercentage *percentage = nullptr;
    checkSucceeded(check, eranosMemberPercentageCreate("ana", 10'000, &percentage), "a member's percentage");
    eranosMemberPercentageSetBasisPoints(percentage, 5'000);
    checkSucceeded(check, eranosMemberPercentageSetMember(percentage, "bob"), "a member is set");
    eranosMemberPercentageMember(percentage, text);
    checkThat(check, 5'000 == eranosMemberPercentageBasisPoints(percentage) && isText(text, "bob"), "half for Bob");
    checkThat(check, isKind(eranosMemberPercentageSetMember(percentage, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a member is text");
    EranosMemberPercentageList *const percentages = eranosMemberPercentageListCreate();
    checkThat(check, eranosMemberPercentageListIsEmpty(percentages), "an empty list of percentages");
    eranosMemberPercentageListPush(percentages, percentage);
    EranosMemberPercentage *const firstPercentage = eranosMemberPercentageListGet(percentages, 0);
    checkThat(check,
              1 == eranosMemberPercentageListLength(percentages) && nullptr != firstPercentage &&
                  nullptr == eranosMemberPercentageListGet(percentages, 1),
              "one percentage");
    eranosMemberPercentageDestroy(firstPercentage);
    EranosMemberAmount *none = nullptr;
    checkThat(check, isKind(eranosMemberAmountCreate("\xff", ten, &none), ERANOS_ERROR_KIND_INVALID_TEXT), "an amount's member is text");
    EranosMemberParts *noParts = nullptr;
    checkThat(check, isKind(eranosMemberPartsCreate("\xff", 1, &noParts), ERANOS_ERROR_KIND_INVALID_TEXT), "a part's member is text");
    EranosMemberPercentage *noPercentage = nullptr;
    checkThat(check, isKind(eranosMemberPercentageCreate("\xff", 1, &noPercentage), ERANOS_ERROR_KIND_INVALID_TEXT), "a percentage's member is text");

    EranosSplit *const byAmounts = eranosSplitCreateAmounts(amounts);
    EranosSplit *const byParts = eranosSplitCreateParts(partsList);
    EranosSplit *const byPercentages = eranosSplitCreatePercentages(percentages);
    EranosSplit *const equal = equally("ana", "bob");
    checkThat(check,
              ERANOS_SPLIT_KIND_AMOUNTS == eranosSplitKind(byAmounts) && ERANOS_SPLIT_KIND_PARTS == eranosSplitKind(byParts) &&
                  ERANOS_SPLIT_KIND_PERCENTAGES == eranosSplitKind(byPercentages) && ERANOS_SPLIT_KIND_EQUAL == eranosSplitKind(equal),
              "each split has its kind");
    EranosMemberAmountList *const readAmounts = eranosSplitAmountsShares(byAmounts);
    EranosMemberPartsList *const readParts = eranosSplitPartsShares(byParts);
    EranosMemberPercentageList *const readPercentages = eranosSplitPercentagesShares(byPercentages);
    checkThat(check, nullptr != readAmounts && 1 == eranosMemberAmountListLength(readAmounts) && nullptr != readParts && nullptr != readPercentages,
              "each split holds its shares");
    checkThat(check,
              nullptr == eranosSplitAmountsShares(equal) && nullptr == eranosSplitPartsShares(equal) &&
                  nullptr == eranosSplitPercentagesShares(equal) && nullptr == eranosSplitMembers(byAmounts),
              "and none of another kind's");
    checkThat(check, listsMembers(eranosSplitMembers(equal), "ana", "bob"), "an equal split holds its members");
    EranosSplit *const unreadable = equally(" ana", nullptr);
    EranosTextList *participants = nullptr;
    checkThat(check, isFailure(eranosSplitParticipants(unreadable, &participants), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_SURROUNDING_WHITESPACE),
              "a split's members are identifiers");
    eranosSplitDestroy(unreadable);
    eranosMemberPercentageListDestroy(readPercentages);
    eranosMemberPartsListDestroy(readParts);
    eranosMemberAmountListDestroy(readAmounts);
    eranosSplitDestroy(equal);
    eranosSplitDestroy(byPercentages);
    eranosSplitDestroy(byParts);
    eranosSplitDestroy(byAmounts);
    eranosMemberPercentageListDestroy(percentages);
    eranosMemberPercentageDestroy(percentage);
    eranosMemberPartsListDestroy(partsList);
    eranosMemberPartsDestroy(parts);
    eranosMemberAmountListDestroy(amounts);
    eranosMemberAmountDestroy(amount);
    eranosMoneyDestroy(twenty);
    eranosMoneyDestroy(ten);
    eranosTextDestroy(text);
}

/// An item, a tip, or a cost that read back otherwise than it was made would bill a receipt otherwise than it reads.
static void itemsTipsAndCostsReadBackAsTheyWereMade(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosMoney *const twelve = euros(1'200);
    EranosSplit *const both = equally("ana", "bob");
    EranosExpenseItem *item = nullptr;
    checkSucceeded(check, eranosExpenseItemCreate("Pizza", twelve, both, &item), "an item");
    checkThat(check, ERANOS_QUANTITY_ONE == eranosExpenseItemQuantity(item), "one unit, by default");
    eranosExpenseItemSetQuantity(item, 20'000);
    checkSucceeded(check, eranosExpenseItemSetName(item, "Pizzas"), "a name is set");
    EranosSplit *const ana = equally("ana", nullptr);
    eranosExpenseItemSetSplit(item, ana);
    EranosMoney *const eleven = euros(1'100);
    eranosExpenseItemSetUnitPrice(item, eleven);
    eranosExpenseItemName(item, text);
    EranosSplit *const itemSplit = eranosExpenseItemSplit(item);
    checkThat(check,
              isText(text, "Pizzas") && 20'000 == eranosExpenseItemQuantity(item) && isAmount(eranosExpenseItemUnitPrice(item), 1'100, "EUR") &&
                  listsMembers(eranosSplitMembers(itemSplit), "ana", nullptr),
              "the item as set");
    checkThat(check, isKind(eranosExpenseItemSetName(item, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
    EranosExpenseItem *noItem = nullptr;
    checkThat(check, isKind(eranosExpenseItemCreate("\xff", twelve, both, &noItem), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == noItem,
              "an item's name is text");
    EranosExpenseItemList *const items = eranosExpenseItemListCreate();
    checkThat(check, eranosExpenseItemListIsEmpty(items), "an empty list of items");
    eranosExpenseItemListPush(items, item);
    EranosExpenseItem *const firstItem = eranosExpenseItemListGet(items, 0);
    checkThat(check, 1 == eranosExpenseItemListLength(items) && nullptr != firstItem && nullptr == eranosExpenseItemListGet(items, 1), "one item");
    eranosExpenseItemDestroy(firstItem);
    EranosExpenseSplit *const itemised = eranosExpenseSplitCreateItemised(items);
    EranosExpenseSplit *const whole = eranosExpenseSplitCreateWhole(both);
    EranosExpenseItemList *const readItems = eranosExpenseSplitItems(itemised);
    EranosSplit *const readSplit = eranosExpenseSplitSplit(whole);
    checkThat(check,
              ERANOS_EXPENSE_SPLIT_KIND_ITEMISED == eranosExpenseSplitKind(itemised) && nullptr != readItems &&
                  nullptr == eranosExpenseSplitSplit(itemised),
              "an itemised split holds its items");
    checkThat(check,
              ERANOS_EXPENSE_SPLIT_KIND_WHOLE == eranosExpenseSplitKind(whole) && nullptr != readSplit && nullptr == eranosExpenseSplitItems(whole),
              "a whole split holds its split");
    EranosMoney *const three = euros(300);
    EranosTipValue *const fixed = eranosTipValueCreateFixed(three);
    EranosTipValue *const tenPercent = eranosTipValueCreatePercentage(1'000);
    int32_t basisPoints = 0;
    checkThat(check,
              ERANOS_TIP_VALUE_KIND_FIXED == eranosTipValueKind(fixed) && isAmount(eranosTipValueAmount(fixed), 300, "EUR") &&
                  !eranosTipValueBasisPoints(fixed, &basisPoints),
              "a fixed tip holds its amount");
    checkThat(check,
              ERANOS_TIP_VALUE_KIND_PERCENTAGE == eranosTipValueKind(tenPercent) && eranosTipValueBasisPoints(tenPercent, &basisPoints) &&
                  1'000 == basisPoints && nullptr == eranosTipValueAmount(tenPercent),
              "a percentage tip holds its basis points");
    EranosTipRule *const tip = eranosTipRuleCreate(fixed, ERANOS_TIP_DISTRIBUTION_EQUAL);
    eranosTipRuleSetValue(tip, tenPercent);
    eranosTipRuleSetDistribution(tip, ERANOS_TIP_DISTRIBUTION_PROPORTIONAL);
    EranosTipValue *const tipValue = eranosTipRuleValue(tip);
    checkThat(check,
              ERANOS_TIP_DISTRIBUTION_PROPORTIONAL == eranosTipRuleDistribution(tip) &&
                  ERANOS_TIP_VALUE_KIND_PERCENTAGE == eranosTipValueKind(tipValue),
              "a tip as set");
    EranosCost *const local = eranosCostCreateLocal(twelve);
    EranosMoney *const lei = amountOf(5'000, "RON");
    EranosCost *converted = nullptr;
    checkSucceeded(check, eranosCostCreateConverted(lei, "0.2", &converted), "a converted cost");
    checkThat(check,
              ERANOS_COST_KIND_LOCAL == eranosCostKind(local) && isAmount(eranosCostAmount(local), 1'200, "EUR") && !eranosCostRate(local, text),
              "a local cost has no rate");
    checkThat(check,
              ERANOS_COST_KIND_CONVERTED == eranosCostKind(converted) && isAmount(eranosCostAmount(converted), 5'000, "RON") &&
                  eranosCostRate(converted, text) && isText(text, "0.2"),
              "a converted cost holds its rate");
    EranosCost *noCost = nullptr;
    checkThat(check, isKind(eranosCostCreateConverted(lei, "\xff", &noCost), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == noCost, "a rate is text");
    eranosCostDestroy(converted);
    eranosCostDestroy(local);
    eranosMoneyDestroy(lei);
    eranosTipValueDestroy(tipValue);
    eranosTipRuleDestroy(tip);
    eranosTipValueDestroy(tenPercent);
    eranosTipValueDestroy(fixed);
    eranosMoneyDestroy(three);
    eranosSplitDestroy(readSplit);
    eranosExpenseItemListDestroy(readItems);
    eranosExpenseSplitDestroy(whole);
    eranosExpenseSplitDestroy(itemised);
    eranosExpenseItemListDestroy(items);
    eranosSplitDestroy(itemSplit);
    eranosMoneyDestroy(eleven);
    eranosSplitDestroy(ana);
    eranosExpenseItemDestroy(item);
    eranosSplitDestroy(both);
    eranosMoneyDestroy(twelve);
    eranosTextDestroy(text);
}

/// An expense whose fields did not read back as set would change what an app shows after it edits one.
static void anExpenseReadsBackEveryFieldItWasGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosExpense *const expense = expenseOf("Dinner", "ana", "ana", "bob");
    eranosExpenseId(expense, text);
    checkThat(check, eranosTextLength(text) == strlen("0199a000-0000-7000-8000-000000000001"), "a new expense has a generated identifier");
    checkThat(check, isUncategorised(expense) && isNoTip(eranosExpenseTip(expense)), "in the built-in category, with the no-tip rule");
    checkThat(check, !eranosExpenseDescription(expense, text), "and no description");
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    checkThat(check, !eranosExpenseTime(expense, &time), "nor time");
    checkSucceeded(check, eranosExpenseSetId(expense, "0199a000-0000-7000-8000-000000000001"), "an identifier is set");
    checkSucceeded(check, eranosExpenseSetName(expense, "Lunch"), "a name");
    checkSucceeded(check, eranosExpenseSetCategory(expense, "0199a000-0000-7000-8000-000000000002"), "a category");
    checkSucceeded(check, eranosExpenseSetDescription(expense, "By the sea"), "a description");
    constexpr EranosTimeOfDay NOON = {.hour = 12, .minute = 15};
    eranosExpenseSetTime(expense, NOON);
    constexpr EranosCalendarDate NEXT = {.year = 2026, .month = 9, .day = 24};
    eranosExpenseSetDate(expense, NEXT);
    EranosMoney *const forty = euros(4'000);
    EranosCost *const cost = eranosCostCreateLocal(forty);
    eranosExpenseSetCost(expense, cost);
    EranosSplit *const bob = equally("bob", nullptr);
    eranosExpenseSetPayers(expense, bob);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateWhole(bob);
    eranosExpenseSetSplit(expense, split);
    EranosTipValue *const tenPercent = eranosTipValueCreatePercentage(1'000);
    EranosTipRule *const tip = eranosTipRuleCreate(tenPercent, ERANOS_TIP_DISTRIBUTION_EQUAL);
    eranosExpenseSetTip(expense, tip);
    eranosExpenseId(expense, text);
    checkThat(check, isText(text, "0199a000-0000-7000-8000-000000000001"), "the identifier as set");
    eranosExpenseName(expense, text);
    checkThat(check, isText(text, "Lunch"), "the name");
    eranosExpenseCategory(expense, text);
    checkThat(check, isText(text, "0199a000-0000-7000-8000-000000000002"), "the category");
    checkThat(check, eranosExpenseDescription(expense, text) && isText(text, "By the sea"), "the description");
    checkThat(check, eranosExpenseTime(expense, &time) && 12 == time.hour && 15 == time.minute, "the time");
    EranosCalendarDate const date = eranosExpenseDate(expense);
    checkThat(check, 2026 == date.year && 9 == date.month && 24 == date.day, "the date");
    EranosCost *const readCost = eranosExpenseCost(expense);
    EranosSplit *const readPayers = eranosExpensePayers(expense);
    EranosExpenseSplit *const readSplit = eranosExpenseSplit(expense);
    EranosTipRule *const readTip = eranosExpenseTip(expense);
    checkThat(check,
              isAmount(eranosCostAmount(readCost), 4'000, "EUR") && listsMembers(eranosSplitMembers(readPayers), "bob", nullptr) &&
                  ERANOS_EXPENSE_SPLIT_KIND_WHOLE == eranosExpenseSplitKind(readSplit) &&
                  ERANOS_TIP_DISTRIBUTION_EQUAL == eranosTipRuleDistribution(readTip),
              "the cost, payers, split, and tip");
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosExpenseTipAmount(expense, &amount), "the tip's amount");
    checkThat(check, isAmount(amount, 400, "EUR"), "10 % of 40.00 EUR");
    checkSucceeded(check, eranosExpenseTotal(expense, &amount), "the total");
    checkThat(check, isAmount(amount, 4'400, "EUR"), "the amount with its tip");
    EranosExpense *const unreadable = expenseOf("Taxi", "a\tb", "ana", nullptr);
    EranosTextList *participants = nullptr;
    checkThat(check, isFailure(eranosExpenseParticipants(unreadable, &participants), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_CONTROL_CHARACTER),
              "an expense's members are identifiers");
    eranosExpenseDestroy(unreadable);
    EranosMoney *const lei = amountOf(100, "RON");
    EranosTipValue *const fixedLei = eranosTipValueCreateFixed(lei);
    EranosTipRule *const foreignTip = eranosTipRuleCreate(fixedLei, ERANOS_TIP_DISTRIBUTION_EQUAL);
    eranosExpenseSetTip(expense, foreignTip);
    checkThat(check,
              isFailure(eranosExpenseTipAmount(expense, &amount), ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH) &&
                  nullptr == amount,
              "a fixed tip in another currency than the expense's");
    checkThat(check, isFailure(eranosExpenseTotal(expense, &amount), ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH),
              "its total");
    checkThat(check,
              isKind(eranosExpenseSetId(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosExpenseSetName(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosExpenseSetCategory(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosExpenseSetDescription(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "every text field is text");
    eranosExpenseClearDescription(expense);
    eranosExpenseClearTime(expense);
    checkThat(check, !eranosExpenseDescription(expense, text) && !eranosExpenseTime(expense, &time), "every optional field clears");
    EranosTipRule *const none = eranosTipRuleCreateNone();
    eranosExpenseSetTip(expense, none);
    eranosIdentifierUncategorised(text);
    checkSucceeded(check, eranosExpenseSetCategory(expense, eranosTextData(text)), "the built-in category is set back");
    checkThat(check, isUncategorised(expense) && isNoTip(eranosExpenseTip(expense)), "and so is the no-tip rule, as any other value");
    EranosExpense *receipt = nullptr;
    checkSucceeded(check, eranosExpenseCreateWithId("receipt-17", "Receipt", cost, bob, split, NEXT, &receipt),
                   "an expense under the app's identifier");
    eranosExpenseId(receipt, text);
    checkThat(check, isText(text, "receipt-17"), "keeps it");
    EranosExpense *noExpense = nullptr;
    constexpr EranosCalendarDate NEVER = {.year = 2026, .month = 2, .day = 30};
    checkThat(check, isKind(eranosExpenseCreate("\xff", cost, bob, split, NEVER, &noExpense), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
    checkThat(check,
              isKind(eranosExpenseCreateWithId("\xff", "Receipt", cost, bob, split, NEXT, &noExpense), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  nullptr == noExpense,
              "an identifier is text");
    eranosTipRuleDestroy(foreignTip);
    eranosTipValueDestroy(fixedLei);
    eranosMoneyDestroy(lei);
    eranosTipRuleDestroy(none);
    eranosTipRuleDestroy(readTip);
    eranosExpenseSplitDestroy(readSplit);
    eranosSplitDestroy(readPayers);
    eranosCostDestroy(readCost);
    eranosTipRuleDestroy(tip);
    eranosTipValueDestroy(tenPercent);
    eranosExpenseSplitDestroy(split);
    eranosSplitDestroy(bob);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(forty);
    eranosExpenseDestroy(receipt);
    eranosExpenseDestroy(expense);
    eranosTextDestroy(text);
}

/// A transfer whose fields did not read back as set would move money between the wrong members.
static void aTransferReadsBackEveryFieldItWasGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosTransfer *const transfer = transferOf("bob", "ana", 1'000);
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    eranosTransferId(transfer, text);
    checkThat(check, eranosTextLength(text) == strlen("0199a000-0000-7000-8000-000000000001"), "a new transfer has a generated identifier");
    checkThat(check, !eranosTransferNote(transfer, text) && !eranosTransferTime(transfer, &time), "and no note or time");
    checkSucceeded(check, eranosTransferSetId(transfer, "0199a000-0000-7000-8000-000000000003"), "an identifier is set");
    checkSucceeded(check, eranosTransferSetFrom(transfer, "ana"), "who paid");
    checkSucceeded(check, eranosTransferSetTo(transfer, "bob"), "who was paid");
    checkSucceeded(check, eranosTransferSetNote(transfer, "Cash"), "a note");
    constexpr EranosTimeOfDay MORNING = {.hour = 9, .minute = 5};
    eranosTransferSetTime(transfer, MORNING);
    constexpr EranosCalendarDate NEXT = {.year = 2026, .month = 9, .day = 24};
    eranosTransferSetDate(transfer, NEXT);
    EranosMoney *const five = euros(500);
    EranosCost *const cost = eranosCostCreateLocal(five);
    eranosTransferSetCost(transfer, cost);
    eranosTransferId(transfer, text);
    checkThat(check, isText(text, "0199a000-0000-7000-8000-000000000003"), "the identifier");
    eranosTransferFrom(transfer, text);
    checkThat(check, isText(text, "ana"), "who paid");
    eranosTransferTo(transfer, text);
    checkThat(check, isText(text, "bob"), "who was paid");
    checkThat(check, eranosTransferNote(transfer, text) && isText(text, "Cash") && eranosTransferTime(transfer, &time) && 9 == time.hour,
              "the note and time");
    EranosCalendarDate const date = eranosTransferDate(transfer);
    EranosCost *const readCost = eranosTransferCost(transfer);
    checkThat(check, 24 == date.day && isAmount(eranosCostAmount(readCost), 500, "EUR"), "the date and the cost");
    checkThat(check,
              isKind(eranosTransferSetId(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosTransferSetFrom(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosTransferSetTo(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosTransferSetNote(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "every text field is text");
    eranosTransferClearNote(transfer);
    eranosTransferClearTime(transfer);
    checkThat(check, !eranosTransferNote(transfer, text) && !eranosTransferTime(transfer, &time), "and clears");
    EranosTransfer *cash = nullptr;
    checkSucceeded(check, eranosTransferCreateWithId("payment-3", "ana", "bob", cost, DAY, &cash), "a transfer under the app's identifier");
    eranosTransferId(cash, text);
    checkThat(check, isText(text, "payment-3"), "keeps it");
    EranosTransfer *none = nullptr;
    checkThat(check, isKind(eranosTransferCreate("\xff", "ana", cost, DAY, &none), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == none,
              "who paid is text");
    checkThat(check, isKind(eranosTransferCreateWithId("\xff", "ana", "bob", cost, DAY, &none), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == none,
              "an identifier is text");
    eranosTransferDestroy(cash);
    eranosCostDestroy(readCost);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(five);
    eranosTransferDestroy(transfer);
    eranosTextDestroy(text);
}

/// A member or an account whose fields did not read back as set would send a member's money to another's account.
static void aMemberAndTheirAccountsReadBackAsTheyWereGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosPaymentAccountDetails *bank = nullptr;
    checkSucceeded(check, eranosPaymentAccountDetailsCreateIban("ro49 aaaa 1b31 0075 9384 0000", &bank), "an account's IBAN");
    EranosPaymentAccountDetails *app = nullptr;
    checkSucceeded(check, eranosPaymentAccountDetailsCreateUsername("ana.pays", &app), "an account's username");
    checkThat(check,
              ERANOS_PAYMENT_ACCOUNT_DETAILS_KIND_IBAN == eranosPaymentAccountDetailsKind(bank) && eranosPaymentAccountDetailsIban(bank, text) &&
                  isText(text, "RO49AAAA1B31007593840000") && !eranosPaymentAccountDetailsUsername(bank, text),
              "an IBAN account holds its IBAN, parsed into its electronic form");
    checkThat(check, eranosPaymentAccountDetailsCountryCode(bank, text) && isText(text, "RO"), "and the country code the library derives from it");
    checkThat(check,
              ERANOS_PAYMENT_ACCOUNT_DETAILS_KIND_USERNAME == eranosPaymentAccountDetailsKind(app) &&
                  eranosPaymentAccountDetailsUsername(app, text) && isText(text, "ana.pays") && !eranosPaymentAccountDetailsIban(app, text) &&
                  !eranosPaymentAccountDetailsCountryCode(app, text),
              "a username account holds its username, and no IBAN or country code");
    EranosPaymentAccountDetails *none = nullptr;
    EranosError *const checksum = eranosPaymentAccountDetailsCreateIban("RO49AAAA1B31007593840001", &none);
    EranosIbanReason reason = ERANOS_IBAN_REASON_INVALID_FORMAT;
    checkThat(check,
              holdsFailure(checksum, ERANOS_ERROR_KIND_PAYMENT_ACCOUNT, ERANOS_FAILURE_PAYMENT_ACCOUNT_INVALID_IBAN) &&
                  eranosErrorReason(checksum, &reason) && ERANOS_IBAN_REASON_INVALID_CHECKSUM == reason && nullptr == none,
              "an account's IBAN is parsed, and one whose check digits do not match gives no details");
    eranosErrorDestroy(checksum);
    checkThat(check,
              isKind(eranosPaymentAccountDetailsCreateIban("\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosPaymentAccountDetailsCreateUsername("\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT),
              "account details are text");
    EranosPaymentAccount *account = nullptr;
    checkSucceeded(check, eranosPaymentAccountCreate("Bank", bank, &account), "an account");
    eranosPaymentAccountSetDetails(account, app);
    checkSucceeded(check, eranosPaymentAccountSetName(account, "App"), "a name is set");
    EranosPaymentAccountDetails *const details = eranosPaymentAccountDetails(account);
    eranosPaymentAccountName(account, text);
    checkThat(check, isText(text, "App") && ERANOS_PAYMENT_ACCOUNT_DETAILS_KIND_USERNAME == eranosPaymentAccountDetailsKind(details),
              "the account as set");
    checkThat(check, isKind(eranosPaymentAccountSetName(account, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
    EranosPaymentAccount *noAccount = nullptr;
    checkThat(check, isKind(eranosPaymentAccountCreate("\xff", bank, &noAccount), ERANOS_ERROR_KIND_INVALID_TEXT), "an account's name is text");
    EranosPaymentAccountList *const accounts = eranosPaymentAccountListCreate();
    checkThat(check, eranosPaymentAccountListIsEmpty(accounts), "an empty list of accounts");
    eranosPaymentAccountListPush(accounts, account);
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosMemberCreate("Carla", &member), "a member");
    EranosPaymentAccountList *const noAccounts = eranosMemberAccounts(member);
    eranosMemberId(member, text);
    checkThat(check, eranosTextLength(text) == strlen("0199a000-0000-7000-8000-000000000001"), "a new member has a generated identifier");
    EranosPhoneNumber *const noPhone = eranosMemberPhone(member);
    checkThat(check,
              !eranosMemberEmail(member, text) && !eranosMemberLastName(member, text) && nullptr == noPhone &&
                  eranosPaymentAccountListIsEmpty(noAccounts),
              "and a first name only");
    checkSucceeded(check, eranosMemberSetId(member, "0199a000-0000-7000-8000-000000000004"), "an identifier is set");
    checkSucceeded(check, eranosMemberSetFirstName(member, "Carla Maria"), "a first name");
    checkSucceeded(check, eranosMemberSetEmail(member, "carla@example.com"), "an email");
    checkSucceeded(check, eranosMemberSetLastName(member, "Pop"), "a last name");
    checkSucceeded(check, eranosMemberSetPhone(member, "+40 712 345 678"), "a phone number");
    eranosMemberSetAccounts(member, accounts);
    EranosPaymentAccountList *const readAccounts = eranosMemberAccounts(member);
    EranosPaymentAccount *const firstAccount = eranosPaymentAccountListGet(readAccounts, 0);
    checkThat(check,
              1 == eranosPaymentAccountListLength(readAccounts) && nullptr != firstAccount && nullptr == eranosPaymentAccountListGet(readAccounts, 1),
              "one account");
    eranosMemberFirstName(member, text);
    checkThat(check, isText(text, "Carla Maria"), "the first name");
    eranosMemberId(member, text);
    checkThat(check, isText(text, "0199a000-0000-7000-8000-000000000004"), "the identifier");
    checkThat(check,
              eranosMemberEmail(member, text) && isText(text, "carla@example.com") && eranosMemberLastName(member, text) && isText(text, "Pop"),
              "every detail as set");
    EranosPhoneNumber *const phone = eranosMemberPhone(member);
    checkThat(check, nullptr != phone, "the phone number too");
    eranosPhoneNumberE164(phone, text);
    checkThat(check, isText(text, "+40712345678"), "parsed into its E.164 form");
    eranosPhoneNumberInternational(phone, text);
    checkThat(check, isText(text, "+40 712 345 678"), "with the international form the library derives from it");
    EranosError *const local = eranosMemberSetPhone(member, "0712");
    checkThat(check, isFailure(local, ERANOS_ERROR_KIND_CONTACT, ERANOS_FAILURE_CONTACT_INVALID_PHONE_NUMBER),
              "a phone number set is parsed, and one without a country code refused");
    EranosPhoneNumber *const kept = eranosMemberPhone(member);
    eranosPhoneNumberE164(kept, text);
    checkThat(check, isText(text, "+40712345678"), "and one refused leaves the number as it was");
    checkThat(check,
              isKind(eranosMemberSetId(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetFirstName(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetEmail(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetLastName(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetPhone(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "every text field is text");
    eranosMemberClearEmail(member);
    eranosMemberClearLastName(member);
    eranosMemberClearPhone(member);
    EranosPhoneNumber *const cleared = eranosMemberPhone(member);
    checkThat(check, !eranosMemberEmail(member, text) && !eranosMemberLastName(member, text) && nullptr == cleared, "and clears");
    EranosMember *carla = nullptr;
    checkSucceeded(check, eranosMemberCreateWithId("carla", "Carla", &carla), "a member under the app's identifier");
    eranosMemberId(carla, text);
    checkThat(check, isText(text, "carla"), "keeps it");
    EranosMember *noMember = nullptr;
    checkThat(check, isKind(eranosMemberCreate("\xff", &noMember), ERANOS_ERROR_KIND_INVALID_TEXT), "a first name is text");
    checkThat(check, isKind(eranosMemberCreateWithId("\xff", "Carla", &noMember), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == noMember,
              "an identifier is text");
    eranosMemberDestroy(carla);
    eranosPhoneNumberDestroy(cleared);
    eranosPhoneNumberDestroy(kept);
    eranosPhoneNumberDestroy(phone);
    eranosPhoneNumberDestroy(noPhone);
    eranosPaymentAccountDestroy(firstAccount);
    eranosPaymentAccountListDestroy(readAccounts);
    eranosPaymentAccountListDestroy(noAccounts);
    eranosMemberDestroy(member);
    eranosPaymentAccountListDestroy(accounts);
    eranosPaymentAccountDetailsDestroy(details);
    eranosPaymentAccountDestroy(account);
    eranosPaymentAccountDetailsDestroy(app);
    eranosPaymentAccountDetailsDestroy(bank);
    eranosTextDestroy(text);
}

/// A category whose fields did not read back as set would file expenses under the wrong name.
static void aCategoryReadsBackAsItWasGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosCategory *category = nullptr;
    checkSucceeded(check, eranosCategoryCreate("Food", &category), "a category");
    eranosCategoryId(category, text);
    checkThat(check, eranosTextLength(text) == strlen("0199a000-0000-7000-8000-000000000001"), "a new category has a generated identifier");
    checkSucceeded(check, eranosCategorySetId(category, "0199a000-0000-7000-8000-000000000005"), "an identifier is set");
    checkSucceeded(check, eranosCategorySetName(category, "Meals"), "a name");
    eranosCategoryName(category, text);
    checkThat(check, isText(text, "Meals"), "the name as set");
    eranosCategoryId(category, text);
    checkThat(check, isText(text, "0199a000-0000-7000-8000-000000000005"), "the identifier as set");
    checkThat(check,
              isKind(eranosCategorySetId(category, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosCategorySetName(category, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "its fields are text");
    EranosCategory *lodging = nullptr;
    checkSucceeded(check, eranosCategoryCreateWithId("lodging", "Lodging", &lodging), "a category under the app's identifier");
    eranosCategoryId(lodging, text);
    checkThat(check, isText(text, "lodging"), "keeps it");
    EranosCategory *none = nullptr;
    checkThat(check, isKind(eranosCategoryCreate("\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
    checkThat(check, isKind(eranosCategoryCreateWithId("\xff", "Lodging", &none), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == none,
              "an identifier is text");
    eranosCategoryDestroy(lodging);
    eranosCategoryDestroy(category);
    eranosTextDestroy(text);
}

/// A balance that formatted otherwise than the library writes it would tell a member they owe what they are owed.
static void everyBalanceFormatsAsTheLibraryWritesIt(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosMoney *const twelve = euros(1'200);
    EranosBalance *const getsBack = eranosBalanceCreateGetsBack(twelve);
    EranosBalance *const owes = eranosBalanceCreateOwes(twelve);
    EranosBalance *const settled = eranosBalanceCreateSettled();
    eranosBalanceFormat(getsBack, text);
    checkThat(check,
              isText(text, "gets back 12.00 EUR") && ERANOS_BALANCE_KIND_GETS_BACK == eranosBalanceKind(getsBack) &&
                  isAmount(eranosBalanceAmount(getsBack), 1'200, "EUR"),
              "as what the member gets back");
    eranosBalanceFormat(owes, text);
    checkThat(check, isText(text, "owes 12.00 EUR") && ERANOS_BALANCE_KIND_OWES == eranosBalanceKind(owes), "as what the member owes");
    eranosBalanceFormat(settled, text);
    checkThat(check, isText(text, "settled") && ERANOS_BALANCE_KIND_SETTLED == eranosBalanceKind(settled) && nullptr == eranosBalanceAmount(settled),
              "with no amount");
    eranosBalanceDestroy(settled);
    eranosBalanceDestroy(owes);
    eranosBalanceDestroy(getsBack);
    eranosMoneyDestroy(twelve);
    eranosTextDestroy(text);
}

/// A destroy function that did not accept nullptr would crash the cleanup path of a caller that frees what it may not have made,
/// as free() lets it.
static void destroyingNullDoesNothing(Check *const check) {
    eranosCategoryDestroy(nullptr);
    eranosCategoryListDestroy(nullptr);
    eranosCurrencyDetailsDestroy(nullptr);
    eranosCurrencyDetailsListDestroy(nullptr);
    eranosErrorDestroy(nullptr);
    eranosCostDestroy(nullptr);
    eranosExpenseDestroy(nullptr);
    eranosExpenseListDestroy(nullptr);
    eranosExpenseSplitDestroy(nullptr);
    eranosExpenseItemDestroy(nullptr);
    eranosExpenseItemListDestroy(nullptr);
    eranosDiscountDestroy(nullptr);
    eranosDiscountValueDestroy(nullptr);
    eranosTipRuleDestroy(nullptr);
    eranosTipValueDestroy(nullptr);
    eranosCancellationDestroy(nullptr);
    eranosCurrencyRateDestroy(nullptr);
    eranosCurrencyRateListDestroy(nullptr);
    eranosExpenseGroupDestroy(nullptr);
    eranosNewExpenseGroupDestroy(nullptr);
    eranosExpenseGroupReaderDestroy(nullptr);
    eranosTextListDestroy(nullptr);
    eranosPaymentAccountDestroy(nullptr);
    eranosPaymentAccountDetailsDestroy(nullptr);
    eranosPaymentAccountListDestroy(nullptr);
    eranosPhoneNumberDestroy(nullptr);
    eranosIbanDestroy(nullptr);
    eranosMemberDestroy(nullptr);
    eranosMemberListDestroy(nullptr);
    eranosMoneyDestroy(nullptr);
    eranosMemberAmountDestroy(nullptr);
    eranosMemberAmountListDestroy(nullptr);
    eranosMemberPartsDestroy(nullptr);
    eranosMemberPartsListDestroy(nullptr);
    eranosMemberPercentageDestroy(nullptr);
    eranosMemberPercentageListDestroy(nullptr);
    eranosSplitDestroy(nullptr);
    eranosTextDestroy(nullptr);
    eranosBalanceDestroy(nullptr);
    eranosCategoryTotalDestroy(nullptr);
    eranosCategoryTotalListDestroy(nullptr);
    eranosExpenseSharesDestroy(nullptr);
    eranosMemberCategoryTotalsDestroy(nullptr);
    eranosMemberCategoryTotalsListDestroy(nullptr);
    eranosMemberShareDestroy(nullptr);
    eranosMemberShareListDestroy(nullptr);
    eranosMemberSummaryDestroy(nullptr);
    eranosMemberSummaryListDestroy(nullptr);
    eranosPaymentDestroy(nullptr);
    eranosPaymentListDestroy(nullptr);
    eranosSettlementPlanDestroy(nullptr);
    eranosSummaryDestroy(nullptr);
    eranosTransferDestroy(nullptr);
    eranosTransferListDestroy(nullptr);
    checkThat(check, true, "every destroy function returned");
}

// --- The group ---------------------------------------------------------------------------------------------------------

/// Details that did not reach the group, or a group that did not change as asked, would show an app a name the user replaced.
static void aGroupHoldsItsDetailsAndChangesThem(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosNewExpenseGroup *details = nullptr;
    checkSucceeded(check, eranosNewExpenseGroupCreateWithId("flat", "Flat", "EUR", &details), "a new group's details under the app's identifier");
    eranosNewExpenseGroupId(details, text);
    checkThat(check, isText(text, "flat"), "keep it");
    checkThat(check, !eranosNewExpenseGroupDescription(details, text) && !eranosNewExpenseGroupLanguage(details, text),
              "and start without description or language");
    checkSucceeded(check, eranosNewExpenseGroupSetId(details, "home"), "an identifier is set");
    checkSucceeded(check, eranosNewExpenseGroupSetCurrency(details, "RON"), "a currency is set");
    checkSucceeded(check, eranosNewExpenseGroupSetName(details, "Home"), "a name");
    checkSucceeded(check, eranosNewExpenseGroupSetDescription(details, "Bills"), "a description");
    checkSucceeded(check, eranosNewExpenseGroupSetLanguage(details, "ro-RO"), "a language");
    eranosNewExpenseGroupCurrency(details, text);
    checkThat(check, isText(text, "RON"), "the currency as set");
    eranosNewExpenseGroupName(details, text);
    checkThat(check, isText(text, "Home"), "the name");
    eranosNewExpenseGroupId(details, text);
    checkThat(check, isText(text, "home"), "the identifier");
    checkThat(check,
              eranosNewExpenseGroupDescription(details, text) && isText(text, "Bills") && eranosNewExpenseGroupLanguage(details, text) &&
                  isText(text, "ro-RO"),
              "the description and the language");
    checkThat(check,
              isKind(eranosNewExpenseGroupSetId(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosNewExpenseGroupSetCurrency(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosNewExpenseGroupSetName(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosNewExpenseGroupSetDescription(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosNewExpenseGroupSetLanguage(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "its fields are text");
    EranosExpenseGroup *home = nullptr;
    checkSucceeded(check, eranosExpenseGroupCreate(details, &home), "a group under the app's identifier");
    eranosExpenseGroupId(home, text);
    checkThat(check, isText(text, "home"), "the one the app gave");
    eranosExpenseGroupDestroy(home);
    eranosErrorDestroy(eranosNewExpenseGroupSetId(details, " x"));
    EranosExpenseGroup *refused = nullptr;
    checkThat(check,
              isFailure(eranosExpenseGroupCreate(details, &refused), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_SURROUNDING_WHITESPACE) &&
                  nullptr == refused,
              "an identifier the library refuses");
    eranosNewExpenseGroupClearDescription(details);
    eranosNewExpenseGroupClearLanguage(details);
    checkThat(check, !eranosNewExpenseGroupDescription(details, text) && !eranosNewExpenseGroupLanguage(details, text), "and clear");
    EranosNewExpenseGroup *noDetails = nullptr;
    checkThat(check, isKind(eranosNewExpenseGroupCreate("Flat", "\xff", &noDetails), ERANOS_ERROR_KIND_INVALID_TEXT), "a currency's code is text");
    checkThat(check,
              isKind(eranosNewExpenseGroupCreateWithId("\xff", "Flat", "EUR", &noDetails), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == noDetails,
              "an identifier is text");
    EranosNewExpenseGroup *generated = nullptr;
    checkSucceeded(check, eranosNewExpenseGroupCreate("Flat", "RON", &generated), "a new group's details under a generated identifier");
    EranosText *const id = eranosTextCreate();
    eranosNewExpenseGroupId(generated, id);
    EranosExpenseGroup *group = nullptr;
    checkSucceeded(check, eranosExpenseGroupCreate(generated, &group), "a group");
    eranosExpenseGroupId(group, text);
    checkThat(check, isText(text, eranosTextData(id)) && eranosTextLength(text) == strlen("0199a000-0000-7000-8000-000000000001"),
              "the generated one, a hyphenated UUID");
    checkThat(check, isCurrency(eranosExpenseGroupCurrency(group), "RON"), "the one it started with");
    checkSucceeded(check, eranosExpenseGroupSetName(group, "Trip"), "a name is set");
    eranosExpenseGroupName(group, text);
    checkThat(check, isText(text, "Trip"), "the name as set");
    checkSucceeded(check, eranosExpenseGroupSetDescription(group, " Summer "), "a description is set");
    checkThat(check, eranosExpenseGroupDescription(group, text) && isText(text, "Summer"), "and read as set, trimmed");
    eranosExpenseGroupClearDescription(group);
    checkThat(check, !eranosExpenseGroupDescription(group, text) && isText(text, ""), "a description clears and reads as none");
    checkSucceeded(check, eranosExpenseGroupSetDescription(group, "Summer"), "a description is set again");
    checkSucceeded(check, eranosExpenseGroupSetDescription(group, " "), "blank text clears it");
    checkThat(check, !eranosExpenseGroupDescription(group, text) && isText(text, ""), "and it reads as none");
    checkSucceeded(check, eranosExpenseGroupSetLanguage(group, "pt-pt"), "a language is set");
    checkThat(check, eranosExpenseGroupLanguage(group, text) && isText(text, "pt-PT"), "and read, canonical");
    eranosExpenseGroupClearLanguage(group);
    checkThat(check, !eranosExpenseGroupLanguage(group, text) && isText(text, ""), "a language clears and reads as none");
    checkThat(check, isFailure(eranosExpenseGroupSetName(group, " "), ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_EMPTY_NAME),
              "a blank name");
    checkThat(check,
              isFailure(eranosExpenseGroupSetLanguage(group, "not a tag"), ERANOS_ERROR_KIND_LANGUAGE, ERANOS_FAILURE_LANGUAGE_INVALID_LANGUAGE),
              "no tag");
    eranosErrorDestroy(eranosNewExpenseGroupSetName(generated, " "));
    EranosExpenseGroup *noGroup = nullptr;
    checkThat(check,
              isFailure(eranosExpenseGroupCreate(generated, &noGroup), ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_EMPTY_NAME) &&
                  nullptr == noGroup,
              "a group needs a name");
    eranosExpenseGroupDestroy(group);
    eranosTextDestroy(id);
    eranosNewExpenseGroupDestroy(generated);
    eranosNewExpenseGroupDestroy(details);
    eranosTextDestroy(text);
}

/// Entries that did not come and go as asked would let an app show a deleted expense or lose an added one.
static void entriesComeAndGoThroughTheGroup(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosExpenseList *const expenses = eranosExpenseGroupExpenses(flat.group);
    EranosExpense *const dinner = eranosExpenseListGet(expenses, 0);
    checkThat(check,
              1 == eranosExpenseListLength(expenses) && !eranosExpenseListIsEmpty(expenses) && nullptr != dinner &&
                  nullptr == eranosExpenseListGet(expenses, 1),
              "the dinner, alone");
    eranosExpenseId(dinner, text);
    checkThat(check, isText(text, eranosTextData(flat.dinner)), "under its identifier");
    checkSucceeded(check, eranosExpenseSetName(dinner, "Supper"), "an edit");
    EranosExpense *previous = nullptr;
    checkSucceeded(check, eranosExpenseGroupReplaceExpense(flat.group, dinner, &previous), "replaces the expense");
    eranosExpenseName(previous, text);
    checkThat(check, isText(text, "Dinner") && !eranosExpenseDescription(previous, text), "which comes back as it was, without a description");
    EranosExpense *expense = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpense(flat.group, eranosTextData(flat.dinner), &expense), "the expense by its identifier");
    eranosExpenseName(expense, text);
    checkThat(check, isText(text, "Supper"), "as replaced");
    EranosExpenseShares *shares = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpenseShares(flat.group, eranosTextData(flat.dinner), &shares), "its shares");
    checkThat(check, nullptr != shares, "which it has");
    eranosExpenseSharesDestroy(shares);
    EranosExpense *removed = nullptr;
    checkSucceeded(check, eranosExpenseGroupRemoveExpense(flat.group, eranosTextData(flat.dinner), &removed), "the expense goes");
    eranosExpenseName(removed, text);
    checkThat(check, isText(text, "Supper"), "and comes back as it was");
    EranosExpense *gone = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpense(flat.group, eranosTextData(flat.dinner), &gone), "a look for it");
    checkThat(check, nullptr == gone, "finds none");
    EranosTransferList *const transfers = eranosExpenseGroupTransfers(flat.group);
    EranosTransfer *const repayment = eranosTransferListGet(transfers, 0);
    checkThat(check,
              1 == eranosTransferListLength(transfers) && !eranosTransferListIsEmpty(transfers) && nullptr == eranosTransferListGet(transfers, 1),
              "the repayment, alone");
    checkSucceeded(check, eranosTransferSetNote(repayment, "Back"), "an edit");
    EranosTransfer *previousTransfer = nullptr;
    checkSucceeded(check, eranosExpenseGroupReplaceTransfer(flat.group, repayment, &previousTransfer), "replaces the transfer");
    checkThat(check, !eranosTransferNote(previousTransfer, text), "which comes back without its note");
    EranosTransfer *transfer = nullptr;
    checkSucceeded(check, eranosExpenseGroupTransfer(flat.group, eranosTextData(flat.repayment), &transfer), "the transfer by its identifier");
    checkThat(check, eranosTransferNote(transfer, text) && isText(text, "Back"), "as replaced");
    eranosTransferClearNote(transfer);
    EranosTransfer *noted = nullptr;
    checkSucceeded(check, eranosExpenseGroupReplaceTransfer(flat.group, transfer, &noted), "replaces the transfer without its note");
    EranosTransfer *unnoted = nullptr;
    checkSucceeded(check, eranosExpenseGroupTransfer(flat.group, eranosTextData(flat.repayment), &unnoted), "the transfer again");
    checkThat(check, eranosTransferNote(noted, text) && isText(text, "Back") && !eranosTransferNote(unnoted, text), "whose note it clears");
    eranosTransferDestroy(unnoted);
    eranosTransferDestroy(noted);
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosExpenseGroupTransferAmount(flat.group, eranosTextData(flat.repayment), &amount), "its amount");
    checkThat(check, isAmount(amount, 1'000, "EUR"), "in the group currency");
    EranosTransfer *removedTransfer = nullptr;
    checkSucceeded(check, eranosExpenseGroupRemoveTransfer(flat.group, eranosTextData(flat.repayment), &removedTransfer), "the transfer goes");
    checkThat(check, nullptr != removedTransfer, "and comes back");
    EranosCategoryList *const categories = eranosExpenseGroupCategories(flat.group);
    EranosCategory *const builtIn = eranosCategoryListGet(categories, 0);
    checkThat(check,
              2 == eranosCategoryListLength(categories) && !eranosCategoryListIsEmpty(categories) && nullptr != builtIn &&
                  nullptr == eranosCategoryListGet(categories, 2),
              "the built-in category and the food");
    EranosCategory *food = nullptr;
    checkSucceeded(check, eranosExpenseGroupCategory(flat.group, eranosTextData(flat.food), &food), "the food by its identifier");
    checkSucceeded(check, eranosCategorySetName(food, "Meals"), "an edit");
    EranosCategory *previousCategory = nullptr;
    checkSucceeded(check, eranosExpenseGroupReplaceCategory(flat.group, food, &previousCategory), "replaces the category");
    eranosCategoryName(previousCategory, text);
    checkThat(check, isText(text, "Food"), "which comes back as it was");
    EranosCategory *removedCategory = nullptr;
    checkSucceeded(check, eranosExpenseGroupRemoveCategory(flat.group, eranosTextData(flat.food), &removedCategory), "the category goes");
    eranosCategoryName(removedCategory, text);
    checkThat(check, isText(text, "Meals"), "and comes back as it was");
    EranosMemberList *const members = eranosExpenseGroupMembers(flat.group);
    EranosMember *const ana = eranosMemberListGet(members, 0);
    checkThat(check, 2 == eranosMemberListLength(members) && !eranosMemberListIsEmpty(members) && nullptr == eranosMemberListGet(members, 2),
              "Ana and Bob");
    checkSucceeded(check, eranosMemberSetLastName(ana, "Pop"), "an edit");
    EranosMember *previousMember = nullptr;
    checkSucceeded(check, eranosExpenseGroupReplaceMember(flat.group, ana, &previousMember), "replaces the member");
    checkThat(check, !eranosMemberLastName(previousMember, text), "who comes back as they were");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosExpenseGroupMember(flat.group, eranosTextData(flat.ana), &member), "the member by their identifier");
    checkThat(check, eranosMemberLastName(member, text) && isText(text, "Pop"), "as replaced");
    bool found = false;
    bool archived = false;
    checkSucceeded(check, eranosExpenseGroupArchiveMember(flat.group, eranosTextData(flat.bob)), "Bob is archived");
    checkSucceeded(check, eranosExpenseGroupIsArchived(flat.group, eranosTextData(flat.bob), &found, &archived), "and asked about");
    checkThat(check, found && archived, "and is");
    EranosExpenseGroupReader *const reader = eranosExpenseGroupRead(flat.group);
    found = false;
    archived = false;
    checkSucceeded(check, eranosExpenseGroupReaderIsArchived(reader, eranosTextData(flat.bob), &found, &archived),
                   "and asked about through a read handle");
    checkThat(check, found && archived, "and is there too");
    eranosExpenseGroupReaderDestroy(reader);
    checkSucceeded(check, eranosExpenseGroupUnarchiveMember(flat.group, eranosTextData(flat.bob)), "Bob is back");
    found = false;
    archived = true;
    checkSucceeded(check, eranosExpenseGroupIsArchived(flat.group, eranosTextData(flat.bob), &found, &archived), "and asked about");
    checkThat(check, found && !archived, "and is not");
    EranosMember *removedMember = nullptr;
    checkSucceeded(check, eranosExpenseGroupRemoveMember(flat.group, eranosTextData(flat.bob), &removedMember), "Bob goes, once nothing names him");
    eranosMemberFirstName(removedMember, text);
    checkThat(check, isText(text, "Bob"), "and comes back");
    eranosMemberDestroy(removedMember);
    eranosMemberDestroy(member);
    eranosMemberDestroy(previousMember);
    eranosMemberDestroy(ana);
    eranosMemberListDestroy(members);
    eranosCategoryDestroy(removedCategory);
    eranosCategoryDestroy(previousCategory);
    eranosCategoryDestroy(food);
    eranosCategoryDestroy(builtIn);
    eranosCategoryListDestroy(categories);
    eranosTransferDestroy(removedTransfer);
    eranosTransferDestroy(transfer);
    eranosTransferDestroy(previousTransfer);
    eranosTransferDestroy(repayment);
    eranosTransferListDestroy(transfers);
    eranosExpenseDestroy(removed);
    eranosExpenseDestroy(expense);
    eranosExpenseDestroy(previous);
    eranosExpenseDestroy(dinner);
    eranosExpenseListDestroy(expenses);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// An expense the group returned without the built-in category's identifier or the no-tip rule would leave an app unable to tell which category
/// it is in or what tip it has, and a built-in identifier or a no-tip rule other than the library's would file it under a category the group
/// does not hold or give it a tip.
static void anExpenseWithoutACategoryOrATipReadsBackInTheBuiltInOneWithTheNoTipRule(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const uncategorised = eranosTextCreate();
    eranosIdentifierUncategorised(uncategorised);
    checkThat(check, isText(uncategorised, "00000000-0000-0000-0000-000000000000"), "the built-in category's identifier is the nil UUID");
    EranosCategory *builtIn = nullptr;
    checkSucceeded(check, eranosExpenseGroupCategory(flat.group, eranosTextData(uncategorised), &builtIn), "a look for it");
    checkThat(check, nullptr != builtIn, "finds the built-in category");
    checkThat(check, isNoTip(eranosTipRuleCreateNone()), "the no-tip rule is 0 % shared proportionally");
    EranosExpense *const taxi = expenseOf("Taxi", eranosTextData(flat.ana), eranosTextData(flat.ana), eranosTextData(flat.bob));
    EranosText *const id = eranosTextCreate();
    checkSucceeded(check, eranosExpenseGroupAddExpense(flat.group, taxi, id), "an expense given neither");
    EranosExpense *added = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpense(flat.group, eranosTextData(id), &added), "is found");
    checkThat(check, isUncategorised(added) && isNoTip(eranosExpenseTip(added)), "in the built-in category, with the no-tip rule");
    EranosTipValue *const nothing = eranosTipValueCreatePercentage(0);
    EranosTipRule *const zero = eranosTipRuleCreate(nothing, ERANOS_TIP_DISTRIBUTION_PROPORTIONAL);
    eranosExpenseSetTip(taxi, zero);
    EranosExpense *previous = nullptr;
    checkSucceeded(check, eranosExpenseGroupReplaceExpense(flat.group, taxi, &previous), "an expense given a 0 % tip shared proportionally");
    EranosExpense *replaced = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpense(flat.group, eranosTextData(id), &replaced), "is found");
    checkThat(check, isNoTip(eranosExpenseTip(replaced)), "with the no-tip rule");
    eranosExpenseDestroy(replaced);
    eranosExpenseDestroy(previous);
    eranosTipRuleDestroy(zero);
    eranosTipValueDestroy(nothing);
    eranosExpenseDestroy(added);
    eranosTextDestroy(id);
    eranosExpenseDestroy(taxi);
    eranosCategoryDestroy(builtIn);
    eranosTextDestroy(uncategorised);
    flatDestroy(flat);
}

/// An entry the group kept under another identifier than the app gave it would break the app's link to its own record, and a
/// second entry under one identifier would stand for the first in every lookup.
static void anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosText *const named = eranosTextCreate();
    EranosMember *carla = nullptr;
    eranosErrorDestroy(eranosMemberCreateWithId("carla", "Carla", &carla));
    checkSucceeded(check, eranosExpenseGroupAddMember(flat.group, carla, text), "a member under the app's identifier");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosExpenseGroupMember(flat.group, "carla", &member), "is found by it");
    checkThat(check, isText(text, "carla") && nullptr != member, "and keeps it");
    eranosMemberId(member, named);
    checkThat(check, isText(named, "carla"), "the member holds it");
    EranosError *const memberTwice = eranosExpenseGroupAddMember(flat.group, carla, text);
    checkThat(check,
              holdsFailure(memberTwice, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_MEMBER) &&
                  eranosErrorMember(memberTwice, named) && isText(named, "carla"),
              "a second member under it is refused");
    EranosCategory *lodging = nullptr;
    eranosErrorDestroy(eranosCategoryCreateWithId("lodging", "Lodging", &lodging));
    checkSucceeded(check, eranosExpenseGroupAddCategory(flat.group, lodging, text), "a category under the app's identifier");
    EranosCategory *category = nullptr;
    checkSucceeded(check, eranosExpenseGroupCategory(flat.group, "lodging", &category), "is found by it");
    checkThat(check, isText(text, "lodging") && nullptr != category, "and keeps it");
    eranosCategoryId(category, named);
    checkThat(check, isText(named, "lodging"), "the category holds it");
    eranosErrorDestroy(eranosCategorySetName(lodging, "Rooms"));
    EranosError *const categoryTwice = eranosExpenseGroupAddCategory(flat.group, lodging, text);
    checkThat(check,
              holdsFailure(categoryTwice, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY) &&
                  eranosErrorCategory(categoryTwice, named) && isText(named, "lodging"),
              "a second category under it is refused, whatever its name");
    EranosExpense *const receipt = expenseOf("Receipt", eranosTextData(flat.ana), eranosTextData(flat.ana), eranosTextData(flat.bob));
    eranosErrorDestroy(eranosExpenseSetId(receipt, "receipt-17"));
    checkSucceeded(check, eranosExpenseGroupAddExpense(flat.group, receipt, text), "an expense under the app's identifier");
    EranosExpense *expense = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpense(flat.group, "receipt-17", &expense), "is found by it");
    checkThat(check, isText(text, "receipt-17") && nullptr != expense, "and keeps it");
    eranosExpenseId(expense, named);
    checkThat(check, isText(named, "receipt-17"), "the expense holds it");
    EranosError *const expenseTwice = eranosExpenseGroupAddExpense(flat.group, receipt, text);
    checkThat(check,
              holdsFailure(expenseTwice, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_EXPENSE) &&
                  eranosErrorExpense(expenseTwice, named) && isText(named, "receipt-17"),
              "a second expense under it is refused");
    EranosTransfer *const cash = transferOf(eranosTextData(flat.ana), eranosTextData(flat.bob), 500);
    eranosErrorDestroy(eranosTransferSetId(cash, "payment-3"));
    checkSucceeded(check, eranosExpenseGroupAddTransfer(flat.group, cash, text), "a transfer under the app's identifier");
    EranosTransfer *transfer = nullptr;
    checkSucceeded(check, eranosExpenseGroupTransfer(flat.group, "payment-3", &transfer), "is found by it");
    checkThat(check, isText(text, "payment-3") && nullptr != transfer, "and keeps it");
    eranosTransferId(transfer, named);
    checkThat(check, isText(named, "payment-3"), "the transfer holds it");
    EranosError *const transferTwice = eranosExpenseGroupAddTransfer(flat.group, cash, text);
    checkThat(check,
              holdsFailure(transferTwice, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_TRANSFER) &&
                  eranosErrorTransfer(transferTwice, named) && isText(named, "payment-3"),
              "a second transfer under it is refused");
    eranosErrorDestroy(transferTwice);
    eranosTransferDestroy(transfer);
    eranosTransferDestroy(cash);
    eranosErrorDestroy(expenseTwice);
    eranosExpenseDestroy(expense);
    eranosExpenseDestroy(receipt);
    eranosErrorDestroy(categoryTwice);
    eranosCategoryDestroy(category);
    eranosCategoryDestroy(lodging);
    eranosErrorDestroy(memberTwice);
    eranosMemberDestroy(member);
    eranosMemberDestroy(carla);
    eranosTextDestroy(named);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A phone number or an IBAN that lost on its way through the group the text the library derives from it would show an app a
/// number it could not group for reading, or an account without its country.
static void aPhoneNumberAndAnIbanComeBackFromTheGroupWithTheTextTheLibraryDerives(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosMember *carla = nullptr;
    eranosErrorDestroy(eranosMemberCreate("Carla", &carla));
    checkSucceeded(check, eranosMemberSetPhone(carla, "+351912345678"), "a phone number");
    EranosPaymentAccountDetails *details = nullptr;
    checkSucceeded(check, eranosPaymentAccountDetailsCreateIban("GB82WEST12345698765432", &details), "an IBAN");
    EranosPaymentAccount *bank = nullptr;
    eranosErrorDestroy(eranosPaymentAccountCreate("Bank", details, &bank));
    EranosPaymentAccountList *const accounts = eranosPaymentAccountListCreate();
    eranosPaymentAccountListPush(accounts, bank);
    eranosMemberSetAccounts(carla, accounts);
    checkSucceeded(check, eranosExpenseGroupAddMember(flat.group, carla, text), "join the group with a member");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosExpenseGroupMember(flat.group, eranosTextData(text), &member), "who is found");
    EranosPhoneNumber *const phone = eranosMemberPhone(member);
    checkThat(check, nullptr != phone, "with the phone number");
    eranosPhoneNumberE164(phone, text);
    checkThat(check, isText(text, "+351912345678"), "in E.164 form");
    eranosPhoneNumberInternational(phone, text);
    checkThat(check, isText(text, "+351 912 345 678"), "and in the international form the library derives from it");
    EranosPaymentAccountList *const readAccounts = eranosMemberAccounts(member);
    EranosPaymentAccount *const account = eranosPaymentAccountListGet(readAccounts, 0);
    EranosPaymentAccountDetails *const readDetails = eranosPaymentAccountDetails(account);
    checkThat(check, eranosPaymentAccountDetailsIban(readDetails, text) && isText(text, "GB82WEST12345698765432"), "and the IBAN");
    checkThat(check, eranosPaymentAccountDetailsCountryCode(readDetails, text) && isText(text, "GB"), "with the country code it derives");
    eranosPaymentAccountDetailsDestroy(readDetails);
    eranosPaymentAccountDestroy(account);
    eranosPaymentAccountListDestroy(readAccounts);
    eranosPhoneNumberDestroy(phone);
    eranosMemberDestroy(member);
    eranosPaymentAccountListDestroy(accounts);
    eranosPaymentAccountDestroy(bank);
    eranosPaymentAccountDetailsDestroy(details);
    eranosMemberDestroy(carla);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// An identifier that is not one would reach the library as no entry's, or as another's; each of the group's lookups refuses
/// it, in both forms, with the failure of the rule it breaks, and gives no object and no answer.
static void anIdentifierThatIsNotOneIsRefusedByEveryLookup(Check *const check) {
    Flat const flat = flatOf();
    char letters[66] = {};
    memset(letters, 'a', sizeof letters - 1);
    RefusedText const refused[] = {
        {.failure = ERANOS_FAILURE_ID_EMPTY, .text = ""},
        {.failure = ERANOS_FAILURE_ID_TOO_LONG, .text = letters},
        {.failure = ERANOS_FAILURE_ID_CONTROL_CHARACTER, .text = "a\tb"},
        {.failure = ERANOS_FAILURE_ID_SURROUNDING_WHITESPACE, .text = " ana"},
    };
    for (size_t index = 0; index < sizeof refused / sizeof refused[0]; ++index) {
        char const *const nobody = refused[index].text;
        EranosFailure const failure = refused[index].failure;
        EranosCategory *category = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupCategory(flat.group, nobody, &category), ERANOS_ERROR_KIND_ID, failure) && nullptr == category,
                  "a category");
        EranosExpense *expense = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupExpense(flat.group, nobody, &expense), ERANOS_ERROR_KIND_ID, failure) && nullptr == expense,
                  "an expense");
        EranosExpenseShares *shares = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupExpenseShares(flat.group, nobody, &shares), ERANOS_ERROR_KIND_ID, failure) && nullptr == shares,
                  "an expense's shares");
        EranosMember *member = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupMember(flat.group, nobody, &member), ERANOS_ERROR_KIND_ID, failure) && nullptr == member,
                  "a member");
        bool found = true;
        bool archived = true;
        checkThat(check,
                  isFailure(eranosExpenseGroupIsArchived(flat.group, nobody, &found, &archived), ERANOS_ERROR_KIND_ID, failure) && !found && archived,
                  "whether a member is archived, which gives no answer");
        EranosTransfer *transfer = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupTransfer(flat.group, nobody, &transfer), ERANOS_ERROR_KIND_ID, failure) && nullptr == transfer,
                  "a transfer");
        EranosMoney *amount = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupTransferAmount(flat.group, nobody, &amount), ERANOS_ERROR_KIND_ID, failure) && nullptr == amount,
                  "a transfer's amount");
        Waiter waiter;
        EranosFoundCategoryCallback const categoryCallback = {waiting(&waiter), receiveCategory};
        eranosExpenseGroupCategoryAsync(flat.group, nobody, nullptr, &categoryCallback);
        await(&waiter);
        checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, failure) && nullptr == waiter.object, "a category, asynchronously");
        EranosFoundExpenseCallback const expenseCallback = {waiting(&waiter), receiveExpense};
        eranosExpenseGroupExpenseAsync(flat.group, nobody, nullptr, &expenseCallback);
        await(&waiter);
        checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, failure) && nullptr == waiter.object, "an expense, asynchronously");
        EranosFoundExpenseSharesCallback const sharesCallback = {waiting(&waiter), receiveExpenseShares};
        eranosExpenseGroupExpenseSharesAsync(flat.group, nobody, nullptr, &sharesCallback);
        await(&waiter);
        checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, failure) && nullptr == waiter.object, "an expense's shares, asynchronously");
        EranosFoundMemberCallback const memberCallback = {waiting(&waiter), receiveMember};
        eranosExpenseGroupMemberAsync(flat.group, nobody, nullptr, &memberCallback);
        await(&waiter);
        checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, failure) && nullptr == waiter.object, "a member, asynchronously");
        EranosFoundFlagCallback const archivedCallback = {waiting(&waiter), receiveFlag};
        eranosExpenseGroupIsArchivedAsync(flat.group, nobody, nullptr, &archivedCallback);
        await(&waiter);
        checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, failure) && !waiter.answered, "whether a member is archived, asynchronously");
        EranosFoundTransferCallback const transferCallback = {waiting(&waiter), receiveTransfer};
        eranosExpenseGroupTransferAsync(flat.group, nobody, nullptr, &transferCallback);
        await(&waiter);
        checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, failure) && nullptr == waiter.object, "a transfer, asynchronously");
        EranosFoundMoneyCallback const amountCallback = {waiting(&waiter), receiveMoney};
        eranosExpenseGroupTransferAmountAsync(flat.group, nobody, nullptr, &amountCallback);
        await(&waiter);
        checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, failure) && nullptr == waiter.object, "a transfer's amount, asynchronously");
    }
    flatDestroy(flat);
}

/// A lookup that reported an error for an identifier nothing has, or gave an object or an answer for it, would make an app treat a deleted
/// entry as a failure, or show one that is gone, or a member who left as active; each lookup, in every form, reports no error and gives no
/// object and no answer.
static void aLookupOfAnIdentifierNothingHasReportsNoErrorAndNoObject(Check *const check) {
    Flat const flat = flatOf();
    char const *const unknown = "0199a000-0000-7000-8000-00000000ffff";
    EranosCategory *category = nullptr;
    EranosExpense *expense = nullptr;
    EranosExpenseShares *shares = nullptr;
    EranosMember *member = nullptr;
    EranosTransfer *transfer = nullptr;
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosExpenseGroupCategory(flat.group, unknown, &category), "a category");
    checkSucceeded(check, eranosExpenseGroupExpense(flat.group, unknown, &expense), "an expense");
    checkSucceeded(check, eranosExpenseGroupExpenseShares(flat.group, unknown, &shares), "an expense's shares");
    checkSucceeded(check, eranosExpenseGroupMember(flat.group, unknown, &member), "a member");
    checkSucceeded(check, eranosExpenseGroupTransfer(flat.group, unknown, &transfer), "a transfer");
    checkSucceeded(check, eranosExpenseGroupTransferAmount(flat.group, unknown, &amount), "a transfer's amount");
    checkThat(check, nullptr == category && nullptr == expense && nullptr == shares && nullptr == member && nullptr == transfer && nullptr == amount,
              "the group's lookups find none");
    bool found = true;
    bool archived = true;
    checkSucceeded(check, eranosExpenseGroupIsArchived(flat.group, unknown, &found, &archived), "whether a member is archived");
    checkThat(check, !found && archived, "which gives no answer");
    EranosExpenseGroupReader *const reader = eranosExpenseGroupRead(flat.group);
    checkSucceeded(check, eranosExpenseGroupReaderCategory(reader, unknown, &category), "a category, through a read handle");
    checkSucceeded(check, eranosExpenseGroupReaderExpense(reader, unknown, &expense), "an expense, through a read handle");
    checkSucceeded(check, eranosExpenseGroupReaderExpenseShares(reader, unknown, &shares), "an expense's shares, through a read handle");
    checkSucceeded(check, eranosExpenseGroupReaderMember(reader, unknown, &member), "a member, through a read handle");
    checkSucceeded(check, eranosExpenseGroupReaderTransfer(reader, unknown, &transfer), "a transfer, through a read handle");
    checkSucceeded(check, eranosExpenseGroupReaderTransferAmount(reader, unknown, &amount), "a transfer's amount, through a read handle");
    checkThat(check, nullptr == category && nullptr == expense && nullptr == shares && nullptr == member && nullptr == transfer && nullptr == amount,
              "the read handle's lookups find none");
    found = true;
    checkSucceeded(check, eranosExpenseGroupReaderIsArchived(reader, unknown, &found, &archived),
                   "whether a member is archived, through a read handle");
    checkThat(check, !found && archived, "which gives no answer there either");
    eranosExpenseGroupReaderDestroy(reader);
    Waiter waiter;
    EranosFoundCategoryCallback const categoryCallback = {waiting(&waiter), receiveCategory};
    eranosExpenseGroupCategoryAsync(flat.group, unknown, nullptr, &categoryCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.object, "a category, asynchronously");
    EranosFoundExpenseCallback const expenseCallback = {waiting(&waiter), receiveExpense};
    eranosExpenseGroupExpenseAsync(flat.group, unknown, nullptr, &expenseCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.object, "an expense, asynchronously");
    EranosFoundExpenseSharesCallback const sharesCallback = {waiting(&waiter), receiveExpenseShares};
    eranosExpenseGroupExpenseSharesAsync(flat.group, unknown, nullptr, &sharesCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.object, "an expense's shares, asynchronously");
    EranosFoundMemberCallback const memberCallback = {waiting(&waiter), receiveMember};
    eranosExpenseGroupMemberAsync(flat.group, unknown, nullptr, &memberCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.object, "a member, asynchronously");
    EranosFoundFlagCallback const archivedCallback = {waiting(&waiter), receiveFlag};
    eranosExpenseGroupIsArchivedAsync(flat.group, unknown, nullptr, &archivedCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && !waiter.answered, "whether a member is archived, asynchronously");
    EranosFoundTransferCallback const transferCallback = {waiting(&waiter), receiveTransfer};
    eranosExpenseGroupTransferAsync(flat.group, unknown, nullptr, &transferCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.object, "a transfer, asynchronously");
    EranosFoundMoneyCallback const amountCallback = {waiting(&waiter), receiveMoney};
    eranosExpenseGroupTransferAmountAsync(flat.group, unknown, nullptr, &amountCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.object, "a transfer's amount, asynchronously");
    flatDestroy(flat);
}

/// A split or an expense that named other participants than its members would show an app who shares a cost wrongly; a split
/// lists its members in its own order, and an expense every member it names, each once, in the order of their identifiers.
static void theParticipantsOfASplitAndOfAnExpenseAreTheirMembers(Check *const check) {
    Flat const flat = flatOf();
    char const *const ana = eranosTextData(flat.ana);
    char const *const bob = eranosTextData(flat.bob);
    EranosSplit *const split = equally(bob, ana);
    EranosTextList *participants = nullptr;
    checkSucceeded(check, eranosSplitParticipants(split, &participants), "a split's participants");
    checkThat(check, listsMembers(participants, bob, ana), "in the split's order");
    EranosExpense *const expense = expenseOf("Taxi", bob, ana, bob);
    checkSucceeded(check, eranosExpenseParticipants(expense, &participants), "an expense's participants");
    checkThat(check, listsMembers(participants, ana, bob), "each once, in the order of their identifiers");
    eranosExpenseDestroy(expense);
    eranosSplitDestroy(split);
    flatDestroy(flat);
}

/// A replacement or a removal of an entry the group does not hold would change another entry, or report a change it never
/// made; each is refused, and gives no entry back.
static void anEntryTheGroupDoesNotHoldIsNeitherReplacedNorRemoved(Check *const check) {
    Flat const flat = flatOf();
    char const *const unknown = "0199a000-0000-7000-8000-00000000ffff";
    char const *const ana = eranosTextData(flat.ana);
    char const *const bob = eranosTextData(flat.bob);
    EranosCategory *category = nullptr;
    checkSucceeded(check, eranosCategoryCreateWithId(unknown, "Travel", &category), "a category the group does not hold");
    EranosCategory *previousCategory = nullptr;
    checkThat(check,
              isFailure(eranosExpenseGroupReplaceCategory(flat.group, category, &previousCategory), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_CATEGORY) &&
                  nullptr == previousCategory,
              "a category is not replaced");
    checkThat(check,
              isFailure(eranosExpenseGroupRemoveCategory(flat.group, unknown, &previousCategory), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_CATEGORY) &&
                  nullptr == previousCategory,
              "nor removed");
    EranosExpense *const expense = expenseOf("Taxi", ana, ana, bob);
    checkSucceeded(check, eranosExpenseSetId(expense, unknown), "an expense's identifier");
    EranosExpense *previousExpense = nullptr;
    checkThat(check,
              isFailure(eranosExpenseGroupReplaceExpense(flat.group, expense, &previousExpense), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_EXPENSE) &&
                  nullptr == previousExpense,
              "an expense is not replaced");
    checkThat(check,
              isFailure(eranosExpenseGroupRemoveExpense(flat.group, unknown, &previousExpense), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_EXPENSE) &&
                  nullptr == previousExpense,
              "nor removed");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosMemberCreateWithId(unknown, "Carol", &member), "a member the group does not hold");
    EranosMember *previousMember = nullptr;
    checkThat(check,
              isFailure(eranosExpenseGroupReplaceMember(flat.group, member, &previousMember), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_MEMBER) &&
                  nullptr == previousMember,
              "a member is not replaced");
    checkThat(check,
              isFailure(eranosExpenseGroupRemoveMember(flat.group, unknown, &previousMember), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_MEMBER) &&
                  nullptr == previousMember,
              "nor removed");
    EranosTransfer *const transfer = transferOf(ana, bob, 500);
    checkSucceeded(check, eranosTransferSetId(transfer, unknown), "a transfer's identifier");
    EranosTransfer *previousTransfer = nullptr;
    checkThat(check,
              isFailure(eranosExpenseGroupReplaceTransfer(flat.group, transfer, &previousTransfer), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_TRANSFER) &&
                  nullptr == previousTransfer,
              "a transfer is not replaced");
    checkThat(check,
              isFailure(eranosExpenseGroupRemoveTransfer(flat.group, unknown, &previousTransfer), ERANOS_ERROR_KIND_EXPENSE_GROUP,
                        ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_TRANSFER) &&
                  nullptr == previousTransfer,
              "nor removed");
    eranosTransferDestroy(transfer);
    eranosMemberDestroy(member);
    eranosExpenseDestroy(expense);
    eranosCategoryDestroy(category);
    flatDestroy(flat);
}

/// Totals that did not add up, or lost a member, would tell a member they owe what they do not.
static void theSummaryAndTheSharesAddUp(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosSummary *const summary = eranosExpenseGroupSummary(flat.group);
    checkThat(check, isAmount(eranosSummaryTotalSpent(summary), 3'000, "EUR"), "the dinner is all that was spent");
    EranosCategoryTotalList *const categoryTotals = eranosSummaryCategoryTotals(summary);
    EranosCategoryTotal *const food = eranosCategoryTotalListGet(categoryTotals, 0);
    eranosCategoryTotalCategory(food, text);
    checkThat(check,
              1 == eranosCategoryTotalListLength(categoryTotals) && !eranosCategoryTotalListIsEmpty(categoryTotals) &&
                  nullptr == eranosCategoryTotalListGet(categoryTotals, 1) && isText(text, eranosTextData(flat.food)) &&
                  isAmount(eranosCategoryTotalTotal(food), 3'000, "EUR"),
              "all of it on food");
    EranosMemberCategoryTotalsList *const memberTotals = eranosSummaryMemberCategoryTotals(summary);
    EranosMemberCategoryTotals *const anaTotals = eranosMemberCategoryTotalsListGet(memberTotals, 0);
    eranosMemberCategoryTotalsMember(anaTotals, text);
    EranosCategoryTotalList *const anaFood = eranosMemberCategoryTotalsTotals(anaTotals);
    checkThat(check,
              2 == eranosMemberCategoryTotalsListLength(memberTotals) && !eranosMemberCategoryTotalsListIsEmpty(memberTotals) &&
                  nullptr == eranosMemberCategoryTotalsListGet(memberTotals, 2) && isText(text, eranosTextData(flat.ana)) &&
                  1 == eranosCategoryTotalListLength(anaFood),
              "each member's share of the food");
    EranosMemberSummaryList *const members = eranosSummaryMembers(summary);
    EranosMemberSummary *const ana = eranosMemberSummaryListGet(members, 0);
    eranosMemberSummaryMember(ana, text);
    EranosBalance *const balance = eranosMemberSummaryBalance(ana);
    checkThat(check,
              2 == eranosMemberSummaryListLength(members) && !eranosMemberSummaryListIsEmpty(members) &&
                  nullptr == eranosMemberSummaryListGet(members, 2) && isText(text, eranosTextData(flat.ana)) &&
                  ERANOS_BALANCE_KIND_GETS_BACK == eranosBalanceKind(balance),
              "Ana is owed");
    checkThat(check,
              isAmount(eranosMemberSummaryPaid(ana), 3'000, "EUR") && isAmount(eranosMemberSummaryShare(ana), 1'500, "EUR") &&
                  isAmount(eranosMemberSummaryReceived(ana), 1'000, "EUR") && isAmount(eranosMemberSummarySent(ana), 0, "EUR") &&
                  isAmount(eranosMemberSummaryNet(ana), 500, "EUR"),
              "Ana paid 30.00, consumed 15.00, and got 10.00 back, so she is owed 5.00");
    EranosExpenseShares *shares = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpenseShares(flat.group, eranosTextData(flat.dinner), &shares), "the dinner's shares");
    EranosMemberShareList *const memberShares = eranosExpenseSharesMembers(shares);
    EranosMemberShare *const anaShare = eranosMemberShareListGet(memberShares, 0);
    eranosMemberShareMember(anaShare, text);
    checkThat(check,
              isAmount(eranosExpenseSharesTotal(shares), 3'000, "EUR") && isAmount(eranosExpenseSharesTip(shares), 0, "EUR") &&
                  2 == eranosMemberShareListLength(memberShares) && !eranosMemberShareListIsEmpty(memberShares) &&
                  nullptr == eranosMemberShareListGet(memberShares, 2),
              "30.00 EUR, no tip, two shares");
    checkThat(check,
              isText(text, eranosTextData(flat.ana)) && isAmount(eranosMemberSharePaid(anaShare), 3'000, "EUR") &&
                  isAmount(eranosMemberShareShare(anaShare), 1'500, "EUR"),
              "Ana paid it and has half");
    eranosMemberShareDestroy(anaShare);
    eranosMemberShareListDestroy(memberShares);
    eranosExpenseSharesDestroy(shares);
    eranosBalanceDestroy(balance);
    eranosMemberSummaryDestroy(ana);
    eranosMemberSummaryListDestroy(members);
    eranosCategoryTotalListDestroy(anaFood);
    eranosMemberCategoryTotalsDestroy(anaTotals);
    eranosMemberCategoryTotalsListDestroy(memberTotals);
    eranosCategoryTotalDestroy(food);
    eranosCategoryTotalListDestroy(categoryTotals);
    eranosSummaryDestroy(summary);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A strategy that settled otherwise than the library would ask a member to pay twice.
static void eachStrategySettlesEveryBalance(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    constexpr EranosSettlementStrategy STRATEGIES[] = {
        ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE,
        ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS,
        ERANOS_SETTLEMENT_STRATEGY_GREEDY,
        ERANOS_SETTLEMENT_STRATEGY_DIRECT,
    };
    for (size_t index = 0; sizeof STRATEGIES / sizeof STRATEGIES[0] > index; ++index) {
        // The fixture's two balances are well within the adaptive strategy's limit, so it settles them exactly.
        EranosSettlementStrategy const computedBy =
            ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE == STRATEGIES[index] ? ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS : STRATEGIES[index];
        EranosSettlementPlan *settlement = nullptr;
        checkSucceeded(check, eranosExpenseGroupSettlement(flat.group, STRATEGIES[index], &settlement), "a settlement");
        EranosPaymentList *const payments = eranosSettlementPlanPayments(settlement);
        EranosPayment *const payment = eranosPaymentListGet(payments, 0);
        checkThat(check,
                  eranosSettlementPlanStrategy(settlement) == computedBy && 1 == eranosPaymentListLength(payments) &&
                      !eranosPaymentListIsEmpty(payments) && nullptr == eranosPaymentListGet(payments, 1),
                  "one payment, by the strategy whose plan it is");
        eranosPaymentFrom(payment, text);
        checkThat(check, isText(text, eranosTextData(flat.bob)), "from Bob");
        eranosPaymentTo(payment, text);
        checkThat(check, isText(text, eranosTextData(flat.ana)) && isAmount(eranosPaymentAmount(payment), 500, "EUR"), "to Ana, of 5.00 EUR");
        eranosPaymentDestroy(payment);
        eranosPaymentListDestroy(payments);
        eranosSettlementPlanDestroy(settlement);
    }
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// How many neighbours share the street party, each with a balance of their own: two to that power does not fit a 64-bit size,
/// so the exact solver refuses the plan before it asks for memory.
static constexpr size_t NEIGHBOURS = 64;

/// A plan whose exact solver's table did not fit in memory would end the program or stall it; the exact strategy refuses it,
/// naming how many balances it holds, and the other strategies settle the same balances.
static void anExactPlanTooLargeToComputeIsRefused(Check *const check) {
    EranosNewExpenseGroup *details = nullptr;
    checkSucceeded(check, eranosNewExpenseGroupCreate("Street party", "EUR", &details), "the group's details");
    EranosExpenseGroup *group = nullptr;
    checkSucceeded(check, eranosExpenseGroupCreate(details, &group), "the group");
    EranosText *neighbours[NEIGHBOURS] = {};
    EranosTextList *const everyone = eranosTextListCreate();
    for (size_t index = 0; NEIGHBOURS > index; ++index) {
        char name[32] = {};
        (void)snprintf(name, sizeof name, "Neighbour %zu", index + 1);
        neighbours[index] = memberIn(group, name);
        checkSucceeded(check, eranosTextListPush(everyone, eranosTextData(neighbours[index])), "a neighbour");
    }
    EranosMoney *const amount = euros((int64_t)NEIGHBOURS * 100);
    EranosCost *const cost = eranosCostCreateLocal(amount);
    EranosSplit *const payers = equally(eranosTextData(neighbours[0]), nullptr);
    EranosSplit *const shares = eranosSplitCreateEqual(everyone);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateWhole(shares);
    EranosExpense *expense = nullptr;
    checkSucceeded(check, eranosExpenseCreate("Food", cost, payers, split, DAY, &expense), "the food, which the first neighbour paid");
    EranosText *const id = eranosTextCreate();
    checkSucceeded(check, eranosExpenseGroupAddExpense(group, expense, id), "shared by every neighbour");
    EranosSettlementPlan *settlement = nullptr;
    EranosError *const refused = eranosExpenseGroupSettlement(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS, &settlement);
    int64_t balances = 0;
    checkThat(check,
              holdsFailure(refused, ERANOS_ERROR_KIND_SETTLEMENT, ERANOS_FAILURE_SETTLEMENT_OUT_OF_MEMORY) &&
                  eranosErrorBalances(refused, &balances) && NEIGHBOURS == balances && nullptr == settlement,
              "the exact plan is refused, naming its balances");
    eranosErrorDestroy(refused);
    checkSucceeded(check, eranosExpenseGroupSettlement(group, ERANOS_SETTLEMENT_STRATEGY_GREEDY, &settlement), "a greedy plan");
    EranosPaymentList *const payments = nullptr == settlement ? nullptr : eranosSettlementPlanPayments(settlement);
    checkThat(check, nullptr != payments && NEIGHBOURS - 1 == eranosPaymentListLength(payments), "one payment from each neighbour who owes");
    eranosPaymentListDestroy(payments);
    eranosSettlementPlanDestroy(settlement);
    eranosTextDestroy(id);
    eranosExpenseDestroy(expense);
    eranosExpenseSplitDestroy(split);
    eranosSplitDestroy(shares);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(amount);
    eranosTextListDestroy(everyone);
    for (size_t index = 0; NEIGHBOURS > index; ++index) {
        eranosTextDestroy(neighbours[index]);
    }
    eranosExpenseGroupDestroy(group);
    eranosNewExpenseGroupDestroy(details);
}

/// A change of currency that left an entry unconverted, or took a rate it was not given, would mix currencies in the totals.
static void aChangeOfCurrencyConvertsEveryEntry(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosCurrencyRate *rate = nullptr;
    checkSucceeded(check, eranosCurrencyRateCreate("USD", "4", &rate), "a rate into the new currency");
    checkSucceeded(check, eranosCurrencyRateSetCurrency(rate, "EUR"), "its currency is set");
    checkSucceeded(check, eranosCurrencyRateSetRate(rate, "5"), "and its rate");
    eranosCurrencyRateCurrency(rate, text);
    checkThat(check, isText(text, "EUR"), "the currency as set");
    eranosCurrencyRateRate(rate, text);
    checkThat(check, isText(text, "5"), "the rate as set");
    checkThat(check,
              isKind(eranosCurrencyRateSetCurrency(rate, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosCurrencyRateSetRate(rate, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "its fields are text");
    EranosCurrencyRate *none = nullptr;
    checkThat(check, isKind(eranosCurrencyRateCreate("\xff", "5", &none), ERANOS_ERROR_KIND_INVALID_TEXT), "a currency's code is text");
    EranosCurrencyRateList *const empty = eranosCurrencyRateListCreate();
    EranosError *const missing = eranosExpenseGroupChangeCurrency(flat.group, "RON", empty);
    checkThat(check,
              holdsFailure(missing, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_MISSING_EXCHANGE_RATE) &&
                  isCurrency(eranosErrorCurrency(missing), "EUR"),
              "the euro needs a rate");
    eranosErrorDestroy(missing);
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    checkThat(check, eranosCurrencyRateListIsEmpty(rates), "an empty list of rates");
    eranosCurrencyRateListPush(rates, rate);
    EranosCurrencyRate *const first = eranosCurrencyRateListGet(rates, 0);
    checkThat(check, 1 == eranosCurrencyRateListLength(rates) && nullptr != first && nullptr == eranosCurrencyRateListGet(rates, 1), "one rate");
    checkSucceeded(check, eranosExpenseGroupChangeCurrency(flat.group, "RON", rates), "the change");
    checkThat(check, isCurrency(eranosExpenseGroupCurrency(flat.group), "RON"), "is the new one");
    eranosCurrencyRateDestroy(first);
    eranosCurrencyRateListDestroy(rates);
    eranosCurrencyRateListDestroy(empty);
    eranosCurrencyRateDestroy(rate);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A saved group that loaded otherwise than it was saved would lose entries between two runs of an app.
static void aSavedGroupLoadsAsItWasSaved(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const json = eranosTextCreate();
    EranosText *const again = eranosTextCreate();
    eranosExpenseGroupSave(flat.group, json);
    EranosExpenseGroup *loaded = nullptr;
    checkSucceeded(check, eranosExpenseGroupLoad(eranosTextData(json), &loaded), "and loads");
    eranosExpenseGroupSave(loaded, again);
    checkThat(check, isText(again, eranosTextData(json)), "to the same text");
    EranosExpenseGroup *broken = nullptr;
    EranosError *const error = eranosExpenseGroupLoad("{", &broken);
    int64_t line = 0;
    int64_t column = 0;
    checkThat(check, isKind(eranosExpenseGroupLoad("{", &broken), ERANOS_ERROR_KIND_INVALID_JSON) && nullptr == broken,
              "text that is not a saved group");
    checkThat(check, eranosErrorLine(error, &line) && 1 == line && eranosErrorColumn(error, &column) && 1 == column, "where it stops being one");
    eranosErrorDestroy(error);
    eranosExpenseGroupDestroy(loaded);
    eranosTextDestroy(again);
    eranosTextDestroy(json);
    flatDestroy(flat);
}

/// An error that lost what it holds would leave an app unable to say which entry, amount, or category the library refused.
static void everyErrorHoldsWhatItsFailureHolds(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosMember *removed = nullptr;
    EranosError *const inUse = eranosExpenseGroupRemoveMember(flat.group, eranosTextData(flat.ana), &removed);
    checkThat(check, holdsFailure(inUse, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_MEMBER_IN_USE) && nullptr == removed,
              "Ana paid and was paid");
    checkThat(check, eranosErrorMember(inUse, text) && isText(text, eranosTextData(flat.ana)), "the error names her");
    checkThat(check, listsMembers(eranosErrorExpenses(inUse), eranosTextData(flat.dinner), nullptr), "the expense that names her");
    checkThat(check, listsMembers(eranosErrorTransfers(inUse), eranosTextData(flat.repayment), nullptr), "the transfer that names her");
    int64_t number = 0;
    EranosIbanReason reason = ERANOS_IBAN_REASON_INVALID_FORMAT;
    checkThat(check,
              nullptr == eranosErrorActual(inUse) && nullptr == eranosErrorExpectedAmount(inUse) && !eranosErrorAllowed(inUse, &number) &&
                  !eranosErrorBalances(inUse, &number) && !eranosErrorCategory(inUse, text) && !eranosErrorColumn(inUse, &number) &&
                  nullptr == eranosErrorCurrency(inUse) && !eranosErrorExisting(inUse, text) && nullptr == eranosErrorExpectedCurrency(inUse) &&
                  !eranosErrorExpectedLength(inUse, &number) && !eranosErrorExpense(inUse, text) && !eranosErrorFormat(inUse, &number) &&
                  nullptr == eranosErrorFoundCurrency(inUse) && !eranosErrorFoundLength(inUse, &number) && nullptr == eranosErrorLeft(inUse) &&
                  !eranosErrorLine(inUse, &number) && !eranosErrorReason(inUse, &reason) && nullptr == eranosErrorRight(inUse) &&
                  !eranosErrorTotalBasisPoints(inUse, &number) && !eranosErrorTransfer(inUse, text),
              "and holds nothing else");
    eranosErrorDestroy(inUse);
    EranosMoney *const thirty = euros(3'000);
    EranosCost *const cost = eranosCostCreateLocal(thirty);
    EranosSplit *const payers = equally(eranosTextData(flat.ana), nullptr);
    EranosMoney *const ten = euros(1'000);
    EranosMemberAmount *amount = nullptr;
    eranosErrorDestroy(eranosMemberAmountCreate(eranosTextData(flat.ana), ten, &amount));
    EranosMemberAmountList *const amounts = eranosMemberAmountListCreate();
    eranosMemberAmountListPush(amounts, amount);
    EranosSplit *const uneven = eranosSplitCreateAmounts(amounts);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateWhole(uneven);
    EranosExpense *expense = nullptr;
    eranosErrorDestroy(eranosExpenseCreate("Uneven", cost, payers, split, DAY, &expense));
    EranosError *const unbalanced = eranosExpenseGroupAddExpense(flat.group, expense, text);
    checkThat(check,
              holdsFailure(unbalanced, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_AMOUNTS_DO_NOT_ADD_UP) &&
                  isAmount(eranosErrorActual(unbalanced), 1'000, "EUR") && isAmount(eranosErrorExpectedAmount(unbalanced), 3'000, "EUR"),
              "the amounts and what they should add up to");
    eranosErrorDestroy(unbalanced);
    EranosMoney *const lei = amountOf(3'000, "RON");
    eranosMemberAmountSetAmount(amount, lei);
    EranosMemberAmountList *const foreignAmounts = eranosMemberAmountListCreate();
    eranosMemberAmountListPush(foreignAmounts, amount);
    EranosSplit *const foreign = eranosSplitCreateAmounts(foreignAmounts);
    EranosExpenseSplit *const foreignSplit = eranosExpenseSplitCreateWhole(foreign);
    eranosExpenseSetSplit(expense, foreignSplit);
    EranosError *const currencies = eranosExpenseGroupAddExpense(flat.group, expense, text);
    checkThat(check,
              holdsFailure(currencies, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH) &&
                  isCurrency(eranosErrorExpectedCurrency(currencies), "EUR") && isCurrency(eranosErrorFoundCurrency(currencies), "RON"),
              "the expense's currency and the share's");
    eranosErrorDestroy(currencies);
    EranosMemberPercentage *half = nullptr;
    eranosErrorDestroy(eranosMemberPercentageCreate(eranosTextData(flat.ana), 5'000, &half));
    EranosMemberPercentageList *const halves = eranosMemberPercentageListCreate();
    eranosMemberPercentageListPush(halves, half);
    EranosSplit *const byHalves = eranosSplitCreatePercentages(halves);
    EranosExpenseSplit *const halvesSplit = eranosExpenseSplitCreateWhole(byHalves);
    eranosExpenseSetSplit(expense, halvesSplit);
    EranosError *const percentages = eranosExpenseGroupAddExpense(flat.group, expense, text);
    checkThat(check,
              holdsFailure(percentages, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_PERCENTAGES_DO_NOT_ADD_UP) &&
                  eranosErrorTotalBasisPoints(percentages, &number) && 5'000 == number,
              "what the percentages add up to");
    eranosErrorDestroy(percentages);
    EranosCategory *duplicate = nullptr;
    eranosErrorDestroy(eranosCategoryCreate("food", &duplicate));
    EranosError *const name = eranosExpenseGroupAddCategory(flat.group, duplicate, text);
    checkThat(check,
              holdsFailure(name, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME) &&
                  eranosErrorExisting(name, text) && isText(text, eranosTextData(flat.food)),
              "the category that has the name");
    eranosErrorDestroy(name);
    EranosExpense *const taxi = expenseOf("Taxi", eranosTextData(flat.ana), eranosTextData(flat.ana), nullptr);
    eranosErrorDestroy(eranosExpenseSetCategory(taxi, "0199a000-0000-7000-8000-00000000ffff"));
    EranosError *const category = eranosExpenseGroupAddExpense(flat.group, taxi, text);
    checkThat(check,
              holdsFailure(category, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_CATEGORY) &&
                  eranosErrorCategory(category, text) && isText(text, "0199a000-0000-7000-8000-00000000ffff"),
              "the unknown category");
    eranosErrorDestroy(category);
    EranosExpense *noExpense = nullptr;
    EranosError *const unknownExpense = eranosExpenseGroupRemoveExpense(flat.group, "0199a000-0000-7000-8000-00000000ffff", &noExpense);
    checkThat(check,
              holdsFailure(unknownExpense, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_EXPENSE) &&
                  eranosErrorExpense(unknownExpense, text),
              "the unknown expense");
    eranosErrorDestroy(unknownExpense);
    EranosTransfer *noTransfer = nullptr;
    EranosError *const unknownTransfer = eranosExpenseGroupRemoveTransfer(flat.group, "0199a000-0000-7000-8000-00000000ffff", &noTransfer);
    checkThat(check,
              holdsFailure(unknownTransfer, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_TRANSFER) &&
                  eranosErrorTransfer(unknownTransfer, text),
              "the unknown transfer");
    eranosErrorDestroy(unknownTransfer);
    EranosText *const json = eranosTextCreate();
    eranosExpenseGroupSave(flat.group, json);
    char tampered[8192] = {};
    char const *const format = strstr(eranosTextData(json), "\"format\": 3");
    int const before = nullptr == format ? 0 : (int)(format - eranosTextData(json)) + (int)strlen("\"format\": ");
    (void)snprintf(tampered, sizeof tampered, "%.*s4%s", before, eranosTextData(json), eranosTextData(json) + before + 1);
    EranosExpenseGroup *newer = nullptr;
    EranosError *const unsupported = eranosExpenseGroupLoad(tampered, &newer);
    checkThat(check,
              holdsFailure(unsupported, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNSUPPORTED_FORMAT) &&
                  eranosErrorFormat(unsupported, &number) && 4 == number,
              "a format this release cannot read");
    eranosErrorMessage(unsupported, text);
    checkThat(check, isText(text, "unsupported group format 4: this version reads format 3"), "the format it reads");
    eranosErrorDestroy(unsupported);
    eranosTextDestroy(json);
    EranosCategory *noCategory = nullptr;
    checkThat(check,
              isFailure(eranosExpenseGroupCategory(flat.group, "a\tb", &noCategory), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_CONTROL_CHARACTER),
              "text that is no identifier");
    eranosExpenseDestroy(taxi);
    eranosCategoryDestroy(duplicate);
    eranosExpenseSplitDestroy(halvesSplit);
    eranosSplitDestroy(byHalves);
    eranosMemberPercentageListDestroy(halves);
    eranosMemberPercentageDestroy(half);
    eranosExpenseSplitDestroy(foreignSplit);
    eranosSplitDestroy(foreign);
    eranosMemberAmountListDestroy(foreignAmounts);
    eranosMoneyDestroy(lei);
    eranosExpenseDestroy(expense);
    eranosExpenseSplitDestroy(split);
    eranosSplitDestroy(uneven);
    eranosMemberAmountListDestroy(amounts);
    eranosMemberAmountDestroy(amount);
    eranosMoneyDestroy(ten);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(thirty);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

// --- Asynchronous calls, the read handle, and the cancellation -----------------------------------------------------------

/// An asynchronous call that reported otherwise than its synchronous form would give an app two answers to one question, and
/// one that swallowed its failure would leave an app waiting for a change that never happened.
static void everyAsynchronousCallReportsWhatItsSynchronousFormReturns(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    Waiter waiter;
    EranosTextCallback const textCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupIdAsync(flat.group, nullptr, &textCallback);
    await(&waiter);
    eranosExpenseGroupId(flat.group, text);
    checkThat(check, nullptr == waiter.error && isText(waiter.text, eranosTextData(text)), "the same identifier");
    eranosTextDestroy(waiter.text);
    EranosTextCallback const nameCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupNameAsync(flat.group, nullptr, &nameCallback);
    await(&waiter);
    checkThat(check, isText(waiter.text, "Flat"), "the name");
    eranosTextDestroy(waiter.text);
    EranosCurrencyDetailsCallback const currencyCallback = {waiting(&waiter), receiveCurrency};
    eranosExpenseGroupCurrencyAsync(flat.group, nullptr, &currencyCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && isCurrency(waiter.object, "EUR"), "the currency");
    EranosOptionalTextCallback const descriptionCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupDescriptionAsync(flat.group, nullptr, &descriptionCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr != waiter.text && isText(waiter.text, "Rent"), "the description");
    eranosTextDestroy(waiter.text);
    EranosOptionalTextCallback const languageCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupLanguageAsync(flat.group, nullptr, &languageCallback);
    await(&waiter);
    checkThat(check, isText(waiter.text, "pt-PT"), "the language");
    eranosTextDestroy(waiter.text);
    EranosTextCallback const saveCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupSaveAsync(flat.group, nullptr, &saveCallback);
    await(&waiter);
    eranosExpenseGroupSave(flat.group, text);
    checkThat(check, isText(waiter.text, eranosTextData(text)), "the same saved group");
    EranosText *const json = waiter.text;
    EranosExpenseGroupCallback const loadCallback = {waiting(&waiter), receiveGroup};
    eranosExpenseGroupLoadAsync(eranosTextData(json), nullptr, &loadCallback);
    await(&waiter);
    eranosTextDestroy(json);
    checkThat(check, nullptr == waiter.error && nullptr != waiter.object, "a group loaded on a thread of its own");
    eranosExpenseGroupDestroy(waiter.object);
    EranosCategoryListCallback const categoriesCallback = {waiting(&waiter), receiveCategories};
    eranosExpenseGroupCategoriesAsync(flat.group, nullptr, &categoriesCallback);
    await(&waiter);
    checkThat(check, 2 == eranosCategoryListLength(waiter.object), "the categories");
    eranosCategoryListDestroy(waiter.object);
    EranosFoundCategoryCallback const categoryCallback = {waiting(&waiter), receiveCategory};
    eranosExpenseGroupCategoryAsync(flat.group, eranosTextData(flat.food), nullptr, &categoryCallback);
    await(&waiter);
    checkThat(check, nullptr != waiter.object, "a category");
    EranosCategory *const food = waiter.object;
    checkSucceeded(check, eranosCategorySetName(food, "Meals"), "an edit");
    EranosCategoryCallback const replaceCategoryCallback = {waiting(&waiter), receiveCategory};
    eranosExpenseGroupReplaceCategoryAsync(flat.group, food, nullptr, &replaceCategoryCallback);
    await(&waiter);
    eranosCategoryName(waiter.object, text);
    checkThat(check, isText(text, "Food"), "a replaced category as it was");
    eranosCategoryDestroy(waiter.object);
    eranosCategoryDestroy(food);
    EranosCategory *drinks = nullptr;
    eranosErrorDestroy(eranosCategoryCreate("Drinks", &drinks));
    EranosTextCallback const addCategoryCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupAddCategoryAsync(flat.group, drinks, nullptr, &addCategoryCallback);
    await(&waiter);
    eranosCategoryDestroy(drinks);
    EranosText *const addedCategory = waiter.text;
    EranosCategoryCallback const removeCategoryCallback = {waiting(&waiter), receiveCategory};
    eranosExpenseGroupRemoveCategoryAsync(flat.group, eranosTextData(addedCategory), nullptr, &removeCategoryCallback);
    await(&waiter);
    eranosTextDestroy(addedCategory);
    eranosCategoryName(waiter.object, text);
    checkThat(check, isText(text, "Drinks"), "an added category, removed");
    eranosCategoryDestroy(waiter.object);
    EranosExpenseListCallback const expensesCallback = {waiting(&waiter), receiveExpenses};
    eranosExpenseGroupExpensesAsync(flat.group, nullptr, &expensesCallback);
    await(&waiter);
    checkThat(check, 1 == eranosExpenseListLength(waiter.object), "the expenses");
    eranosExpenseListDestroy(waiter.object);
    EranosFoundExpenseCallback const expenseCallback = {waiting(&waiter), receiveExpense};
    eranosExpenseGroupExpenseAsync(flat.group, eranosTextData(flat.dinner), nullptr, &expenseCallback);
    await(&waiter);
    EranosExpense *const dinner = waiter.object;
    checkThat(check, nullptr != dinner, "an expense");
    EranosFoundExpenseSharesCallback const sharesCallback = {waiting(&waiter), receiveExpenseShares};
    eranosExpenseGroupExpenseSharesAsync(flat.group, eranosTextData(flat.dinner), nullptr, &sharesCallback);
    await(&waiter);
    checkThat(check, nullptr != waiter.object, "an expense's shares");
    eranosExpenseSharesDestroy(waiter.object);
    checkSucceeded(check, eranosExpenseSetName(dinner, "Supper"), "an edit");
    EranosExpenseCallback const replaceExpenseCallback = {waiting(&waiter), receiveExpense};
    eranosExpenseGroupReplaceExpenseAsync(flat.group, dinner, nullptr, &replaceExpenseCallback);
    await(&waiter);
    eranosExpenseName(waiter.object, text);
    checkThat(check, isText(text, "Dinner"), "a replaced expense as it was");
    eranosExpenseDestroy(waiter.object);
    eranosExpenseDestroy(dinner);
    EranosExpense *const lunch = expenseOf("Lunch", eranosTextData(flat.bob), eranosTextData(flat.ana), nullptr);
    EranosTextCallback const addExpenseCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupAddExpenseAsync(flat.group, lunch, nullptr, &addExpenseCallback);
    await(&waiter);
    eranosExpenseDestroy(lunch);
    EranosText *const addedExpense = waiter.text;
    EranosExpenseCallback const removeExpenseCallback = {waiting(&waiter), receiveExpense};
    eranosExpenseGroupRemoveExpenseAsync(flat.group, eranosTextData(addedExpense), nullptr, &removeExpenseCallback);
    await(&waiter);
    eranosTextDestroy(addedExpense);
    eranosExpenseName(waiter.object, text);
    checkThat(check, isText(text, "Lunch"), "an added expense, removed");
    eranosExpenseDestroy(waiter.object);
    EranosSummaryCallback const summaryCallback = {waiting(&waiter), receiveSummary};
    eranosExpenseGroupSummaryAsync(flat.group, nullptr, &summaryCallback);
    await(&waiter);
    checkThat(check, isAmount(eranosSummaryTotalSpent(waiter.object), 3'000, "EUR"), "the summary");
    eranosSummaryDestroy(waiter.object);
    EranosSettlementPlanCallback const settlementCallback = {waiting(&waiter), receiveSettlement};
    eranosExpenseGroupSettlementAsync(flat.group, ERANOS_SETTLEMENT_STRATEGY_GREEDY, nullptr, &settlementCallback);
    await(&waiter);
    checkThat(check, ERANOS_SETTLEMENT_STRATEGY_GREEDY == eranosSettlementPlanStrategy(waiter.object), "a settlement");
    eranosSettlementPlanDestroy(waiter.object);
    EranosMemberListCallback const membersCallback = {waiting(&waiter), receiveMembers};
    eranosExpenseGroupMembersAsync(flat.group, nullptr, &membersCallback);
    await(&waiter);
    checkThat(check, 2 == eranosMemberListLength(waiter.object), "the members");
    eranosMemberListDestroy(waiter.object);
    EranosFoundMemberCallback const memberCallback = {waiting(&waiter), receiveMember};
    eranosExpenseGroupMemberAsync(flat.group, eranosTextData(flat.ana), nullptr, &memberCallback);
    await(&waiter);
    EranosMember *const ana = waiter.object;
    checkThat(check, nullptr != ana, "a member");
    checkSucceeded(check, eranosMemberSetLastName(ana, "Pop"), "an edit");
    EranosMemberCallback const replaceMemberCallback = {waiting(&waiter), receiveMember};
    eranosExpenseGroupReplaceMemberAsync(flat.group, ana, nullptr, &replaceMemberCallback);
    await(&waiter);
    checkThat(check, !eranosMemberLastName(waiter.object, text), "a replaced member as they were");
    eranosMemberDestroy(waiter.object);
    eranosMemberDestroy(ana);
    EranosMember *carla = nullptr;
    eranosErrorDestroy(eranosMemberCreate("Carla", &carla));
    EranosTextCallback const addMemberCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupAddMemberAsync(flat.group, carla, nullptr, &addMemberCallback);
    await(&waiter);
    eranosMemberDestroy(carla);
    EranosText *const carlaId = waiter.text;
    EranosUnitCallback const archiveCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupArchiveMemberAsync(flat.group, eranosTextData(carlaId), nullptr, &archiveCallback);
    await(&waiter);
    EranosFoundFlagCallback const archivedCallback = {waiting(&waiter), receiveFlag};
    eranosExpenseGroupIsArchivedAsync(flat.group, eranosTextData(carlaId), nullptr, &archivedCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && waiter.answered && waiter.flag, "an archived member is archived");
    EranosUnitCallback const unarchiveCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupUnarchiveMemberAsync(flat.group, eranosTextData(carlaId), nullptr, &unarchiveCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error, "and comes back");
    EranosFoundFlagCallback const activeCallback = {waiting(&waiter), receiveFlag};
    eranosExpenseGroupIsArchivedAsync(flat.group, eranosTextData(carlaId), nullptr, &activeCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && waiter.answered && !waiter.flag, "an active member is not archived");
    EranosMemberCallback const removeMemberCallback = {waiting(&waiter), receiveMember};
    eranosExpenseGroupRemoveMemberAsync(flat.group, eranosTextData(carlaId), nullptr, &removeMemberCallback);
    await(&waiter);
    eranosMemberFirstName(waiter.object, text);
    checkThat(check, isText(text, "Carla"), "an added member, removed");
    eranosMemberDestroy(waiter.object);
    eranosTextDestroy(carlaId);
    EranosTransferListCallback const transfersCallback = {waiting(&waiter), receiveTransfers};
    eranosExpenseGroupTransfersAsync(flat.group, nullptr, &transfersCallback);
    await(&waiter);
    checkThat(check, 1 == eranosTransferListLength(waiter.object), "the transfers");
    eranosTransferListDestroy(waiter.object);
    EranosFoundTransferCallback const transferCallback = {waiting(&waiter), receiveTransfer};
    eranosExpenseGroupTransferAsync(flat.group, eranosTextData(flat.repayment), nullptr, &transferCallback);
    await(&waiter);
    EranosTransfer *const repayment = waiter.object;
    EranosFoundMoneyCallback const amountCallback = {waiting(&waiter), receiveMoney};
    eranosExpenseGroupTransferAmountAsync(flat.group, eranosTextData(flat.repayment), nullptr, &amountCallback);
    await(&waiter);
    checkThat(check, isAmount(waiter.object, 1'000, "EUR"), "a transfer's amount");
    checkSucceeded(check, eranosTransferSetNote(repayment, "Back"), "an edit");
    EranosTransferCallback const replaceTransferCallback = {waiting(&waiter), receiveTransfer};
    eranosExpenseGroupReplaceTransferAsync(flat.group, repayment, nullptr, &replaceTransferCallback);
    await(&waiter);
    checkThat(check, !eranosTransferNote(waiter.object, text), "a replaced transfer as it was");
    eranosTransferDestroy(waiter.object);
    eranosTransferDestroy(repayment);
    EranosTransfer *const cash = transferOf(eranosTextData(flat.ana), eranosTextData(flat.bob), 100);
    EranosTextCallback const addTransferCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupAddTransferAsync(flat.group, cash, nullptr, &addTransferCallback);
    await(&waiter);
    eranosTransferDestroy(cash);
    EranosText *const addedTransfer = waiter.text;
    EranosTransferCallback const removeTransferCallback = {waiting(&waiter), receiveTransfer};
    eranosExpenseGroupRemoveTransferAsync(flat.group, eranosTextData(addedTransfer), nullptr, &removeTransferCallback);
    await(&waiter);
    eranosTextDestroy(addedTransfer);
    checkThat(check, nullptr != waiter.object, "an added transfer, removed");
    eranosTransferDestroy(waiter.object);
    EranosUnitCallback const setNameCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupSetNameAsync(flat.group, "Home", nullptr, &setNameCallback);
    await(&waiter);
    EranosUnitCallback const setDescriptionCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupSetDescriptionAsync(flat.group, "Bills", nullptr, &setDescriptionCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && eranosExpenseGroupDescription(flat.group, text) && isText(text, "Bills"), "a description is set");
    EranosUnitCallback const clearDescriptionCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupClearDescriptionAsync(flat.group, nullptr, &clearDescriptionCallback);
    await(&waiter);
    EranosOptionalTextCallback const clearedCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupDescriptionAsync(flat.group, nullptr, &clearedCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.text, "a cleared description is none");
    EranosUnitCallback const setLanguageCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupSetLanguageAsync(flat.group, "ro-RO", nullptr, &setLanguageCallback);
    await(&waiter);
    EranosUnitCallback const clearLanguageCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupClearLanguageAsync(flat.group, nullptr, &clearLanguageCallback);
    await(&waiter);
    eranosExpenseGroupName(flat.group, text);
    checkThat(check, isText(text, "Home"), "as set");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = nullptr;
    eranosErrorDestroy(eranosCurrencyRateCreate("EUR", "5", &rate));
    eranosCurrencyRateListPush(rates, rate);
    EranosUnitCallback const changeCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupChangeCurrencyAsync(flat.group, "RON", rates, nullptr, &changeCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && isCurrency(eranosExpenseGroupCurrency(flat.group), "RON"), "changed");
    eranosCurrencyRateDestroy(rate);
    eranosCurrencyRateListDestroy(rates);
    EranosMemberCallback const failingCallback = {waiting(&waiter), receiveMember};
    eranosExpenseGroupRemoveMemberAsync(flat.group, "0199a000-0000-7000-8000-00000000ffff", nullptr, &failingCallback);
    await(&waiter);
    checkThat(check,
              isFailure(waiter.error, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_MEMBER) && nullptr == waiter.object,
              "a call that fails reports its error and no value");
    EranosCategoryCallback const failingCategoryCallback = {waiting(&waiter), receiveCategory};
    eranosExpenseGroupRemoveCategoryAsync(flat.group, "0199a000-0000-7000-8000-00000000ffff", nullptr, &failingCategoryCallback);
    await(&waiter);
    checkThat(check,
              isFailure(waiter.error, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_CATEGORY) && nullptr == waiter.object,
              "a category's removal that fails reports no category");
    EranosExpenseCallback const failingExpenseCallback = {waiting(&waiter), receiveExpense};
    eranosExpenseGroupRemoveExpenseAsync(flat.group, "0199a000-0000-7000-8000-00000000ffff", nullptr, &failingExpenseCallback);
    await(&waiter);
    checkThat(check,
              isFailure(waiter.error, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_EXPENSE) && nullptr == waiter.object,
              "an expense's removal that fails reports no expense");
    EranosTransferCallback const failingTransferCallback = {waiting(&waiter), receiveTransfer};
    eranosExpenseGroupRemoveTransferAsync(flat.group, "0199a000-0000-7000-8000-00000000ffff", nullptr, &failingTransferCallback);
    await(&waiter);
    checkThat(check,
              isFailure(waiter.error, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_TRANSFER) && nullptr == waiter.object,
              "a transfer's removal that fails reports no transfer");
    EranosUnitCallback const failingUnitCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupSetNameAsync(flat.group, " ", nullptr, &failingUnitCallback);
    await(&waiter);
    checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_EMPTY_NAME),
              "a change that fails reports its error");
    EranosMember *dan = nullptr;
    eranosErrorDestroy(eranosMemberCreateWithId(" x", "Dan", &dan));
    EranosTextCallback const failingTextCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupAddMemberAsync(flat.group, dan, nullptr, &failingTextCallback);
    await(&waiter);
    checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_SURROUNDING_WHITESPACE) && nullptr == waiter.text,
              "a call that returns text reports no text");
    eranosMemberDestroy(dan);
    EranosFoundFlagCallback const failingFlagCallback = {waiting(&waiter), receiveFlag};
    eranosExpenseGroupIsArchivedAsync(flat.group, " x", nullptr, &failingFlagCallback);
    await(&waiter);
    checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_SURROUNDING_WHITESPACE) && !waiter.answered,
              "a call that answers reports no answer");
    EranosExpenseGroupCallback const brokenCallback = {waiting(&waiter), receiveGroup};
    eranosExpenseGroupLoadAsync("{", nullptr, &brokenCallback);
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_INVALID_JSON) && nullptr == waiter.object, "a load that fails");
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A read handle that let a change in, or read after its release, would show a report two states of one group.
static void aReaderReadsTheGroupUntilItIsReleased(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    Waiter waiter;
    EranosExpenseGroupReaderCallback const readerCallback = {waiting(&waiter), receiveReader};
    eranosExpenseGroupReadAsync(flat.group, nullptr, &readerCallback);
    await(&waiter);
    EranosExpenseGroupReader *const reader = waiter.object;
    checkThat(check, nullptr == waiter.error && nullptr != reader, "a read handle from an asynchronous call");
    checkSucceeded(check, eranosExpenseGroupReaderName(reader, text), "the name");
    checkThat(check, isText(text, "Flat"), "the group's");
    checkSucceeded(check, eranosExpenseGroupReaderId(reader, text), "the identifier");
    EranosCurrencyDetails *currency = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderCurrency(reader, &currency), "the currency");
    checkThat(check, isCurrency(currency, "EUR"), "the euro");
    checkSucceeded(check, eranosExpenseGroupReaderDescription(reader, text), "the description");
    checkThat(check, isText(text, "Rent"), "as given");
    checkSucceeded(check, eranosExpenseGroupReaderLanguage(reader, text), "the language");
    checkThat(check, isText(text, "pt-PT"), "as given");
    checkSucceeded(check, eranosExpenseGroupReaderSave(reader, text), "the saved group");
    EranosCategoryList *categories = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderCategories(reader, &categories), "the categories");
    EranosCategory *category = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderCategory(reader, eranosTextData(flat.food), &category), "a category");
    EranosExpenseList *expenses = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderExpenses(reader, &expenses), "the expenses");
    EranosExpense *expense = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderExpense(reader, eranosTextData(flat.dinner), &expense), "an expense");
    EranosExpenseShares *shares = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderExpenseShares(reader, eranosTextData(flat.dinner), &shares), "its shares");
    EranosSummary *summary = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderSummary(reader, &summary), "the summary");
    EranosMemberList *members = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderMembers(reader, &members), "the members");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderMember(reader, eranosTextData(flat.ana), &member), "a member");
    bool found = false;
    bool archived = true;
    checkSucceeded(check, eranosExpenseGroupReaderIsArchived(reader, eranosTextData(flat.ana), &found, &archived), "whether a member is archived");
    EranosSettlementPlan *settlement = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderSettlement(reader, ERANOS_SETTLEMENT_STRATEGY_DIRECT, &settlement), "a settlement");
    EranosTransferList *transfers = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderTransfers(reader, &transfers), "the transfers");
    EranosTransfer *transfer = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderTransfer(reader, eranosTextData(flat.repayment), &transfer), "a transfer");
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosExpenseGroupReaderTransferAmount(reader, eranosTextData(flat.repayment), &amount), "its amount");
    checkThat(check,
              2 == eranosCategoryListLength(categories) && nullptr != category && 1 == eranosExpenseListLength(expenses) && nullptr != expense &&
                  nullptr != shares && nullptr != summary && 2 == eranosMemberListLength(members) && nullptr != member && found && !archived &&
                  nullptr != settlement && 1 == eranosTransferListLength(transfers) && nullptr != transfer && isAmount(amount, 1'000, "EUR"),
              "every read sees the group");
    eranosExpenseGroupReaderRelease(reader);
    EranosError *const released = eranosExpenseGroupReaderName(reader, text);
    checkThat(check, isKind(released, ERANOS_ERROR_KIND_RELEASED), "a read after the release");
    EranosExpenseGroupReader *const synchronous = eranosExpenseGroupRead(flat.group);
    checkThat(check, nullptr != synchronous, "a read handle from a synchronous call");
    eranosExpenseGroupReaderDestroy(synchronous);
    eranosTransferDestroy(transfer);
    eranosTransferListDestroy(transfers);
    eranosSettlementPlanDestroy(settlement);
    eranosMemberDestroy(member);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
    eranosExpenseSharesDestroy(shares);
    eranosExpenseDestroy(expense);
    eranosExpenseListDestroy(expenses);
    eranosCategoryDestroy(category);
    eranosCategoryListDestroy(categories);
    eranosExpenseGroupReaderDestroy(reader);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A read handle whose reads went on after its release would read a group whose later changes it no longer waits for; every
/// read fails once the handle is released, and gives no object, no text, and no answer.
static void everyReadOfAReleasedReaderFails(Check *const check) {
    Flat const flat = flatOf();
    EranosExpenseGroupReader *const reader = eranosExpenseGroupRead(flat.group);
    eranosExpenseGroupReaderRelease(reader);
    char const *const food = eranosTextData(flat.food);
    char const *const dinner = eranosTextData(flat.dinner);
    char const *const repayment = eranosTextData(flat.repayment);
    EranosText *const text = eranosTextCreate();
    checkThat(check,
              isKind(eranosExpenseGroupReaderId(reader, text), ERANOS_ERROR_KIND_RELEASED) &&
                  isKind(eranosExpenseGroupReaderDescription(reader, text), ERANOS_ERROR_KIND_RELEASED) &&
                  isKind(eranosExpenseGroupReaderLanguage(reader, text), ERANOS_ERROR_KIND_RELEASED) &&
                  isKind(eranosExpenseGroupReaderSave(reader, text), ERANOS_ERROR_KIND_RELEASED) && 0 == eranosTextLength(text),
              "every read of text, which writes none");
    eranosTextDestroy(text);
    EranosCategoryList *categories = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderCategories(reader, &categories), ERANOS_ERROR_KIND_RELEASED) && nullptr == categories,
              "the categories");
    EranosCategory *category = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderCategory(reader, food, &category), ERANOS_ERROR_KIND_RELEASED) && nullptr == category,
              "a category");
    EranosCurrencyDetails *currency = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderCurrency(reader, &currency), ERANOS_ERROR_KIND_RELEASED) && nullptr == currency, "the currency");
    EranosExpenseList *expenses = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderExpenses(reader, &expenses), ERANOS_ERROR_KIND_RELEASED) && nullptr == expenses, "the expenses");
    EranosExpense *expense = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderExpense(reader, dinner, &expense), ERANOS_ERROR_KIND_RELEASED) && nullptr == expense,
              "an expense");
    EranosExpenseShares *shares = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderExpenseShares(reader, dinner, &shares), ERANOS_ERROR_KIND_RELEASED) && nullptr == shares,
              "an expense's shares");
    EranosSummary *summary = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderSummary(reader, &summary), ERANOS_ERROR_KIND_RELEASED) && nullptr == summary, "the summary");
    EranosMemberList *members = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderMembers(reader, &members), ERANOS_ERROR_KIND_RELEASED) && nullptr == members, "the members");
    EranosMember *member = nullptr;
    checkThat(check,
              isKind(eranosExpenseGroupReaderMember(reader, eranosTextData(flat.ana), &member), ERANOS_ERROR_KIND_RELEASED) && nullptr == member,
              "a member");
    bool found = true;
    bool archived = true;
    checkThat(check,
              isKind(eranosExpenseGroupReaderIsArchived(reader, eranosTextData(flat.ana), &found, &archived), ERANOS_ERROR_KIND_RELEASED) && !found &&
                  archived,
              "whether a member is archived, which gives no answer");
    EranosSettlementPlan *settlement = nullptr;
    checkThat(check,
              isKind(eranosExpenseGroupReaderSettlement(reader, ERANOS_SETTLEMENT_STRATEGY_DIRECT, &settlement), ERANOS_ERROR_KIND_RELEASED) &&
                  nullptr == settlement,
              "a settlement");
    EranosTransferList *transfers = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderTransfers(reader, &transfers), ERANOS_ERROR_KIND_RELEASED) && nullptr == transfers,
              "the transfers");
    EranosTransfer *transfer = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderTransfer(reader, repayment, &transfer), ERANOS_ERROR_KIND_RELEASED) && nullptr == transfer,
              "a transfer");
    EranosMoney *amount = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderTransferAmount(reader, repayment, &amount), ERANOS_ERROR_KIND_RELEASED) && nullptr == amount,
              "a transfer's amount");
    eranosExpenseGroupReaderDestroy(reader);
    flatDestroy(flat);
}

/// A cancelled change that still ran would change the group after the app gave the change up; one that kept waiting would
/// hold the worker thread for a lock it no longer needs.
static void aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosExpenseGroupReader *const reader = eranosExpenseGroupRead(flat.group);
    EranosCancellation *const cancellation = eranosCancellationCreate();
    checkThat(check, !eranosCancellationIsCancelled(cancellation), "a cancellation starts uncancelled");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    Waiter waiter;
    EranosUnitCallback const changeCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupChangeCurrencyAsync(flat.group, "RON", rates, cancellation, &changeCallback);
    eranosCancellationCancel(cancellation);
    checkThat(check, eranosCancellationIsCancelled(cancellation), "and is cancelled once cancelled");
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_CANCELLED), "the change reports its cancellation");
    eranosExpenseGroupReaderRelease(reader);
    checkThat(check, isCurrency(eranosExpenseGroupCurrency(flat.group), "EUR"), "is still the euro");
    EranosTextCallback const lateCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupNameAsync(flat.group, cancellation, &lateCallback);
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_CANCELLED) && nullptr == waiter.text,
              "a call given a cancelled cancellation does not run");
    EranosOptionalTextCallback const lateDescriptionCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupDescriptionAsync(flat.group, cancellation, &lateDescriptionCallback);
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_CANCELLED) && nullptr == waiter.text, "nor one that returns optional text");
    eranosCurrencyRateListDestroy(rates);
    eranosCancellationDestroy(cancellation);
    eranosExpenseGroupReaderDestroy(reader);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A test: its name, the claim it makes, and its body.
typedef struct Test {
    /// The test.
    void (*body)(Check *check);
    /// The test's name.
    char const *name;
} Test;

/// Runs every test and prints each claim that failed.
int main(void) {
    Test const tests[] = {
        {.body = aTextHoldsWhatTheLastCallWrote, .name = "aTextHoldsWhatTheLastCallWrote"},
        {.body = aCurrencyHoldsWhatTheLibraryKnowsOfIt, .name = "aCurrencyHoldsWhatTheLibraryKnowsOfIt"},
        {.body = textThatIsNotUtf8IsRefusedAndNullReadsAsEmpty, .name = "textThatIsNotUtf8IsRefusedAndNullReadsAsEmpty"},
        {.body = anAmountParsesFormatsAndAdds, .name = "anAmountParsesFormatsAndAdds"},
        {.body = anAmountAndAFailureCarryTheirCurrencysDetails, .name = "anAmountAndAFailureCarryTheirCurrencysDetails"},
        {.body = anAmountFormatsWithItsCurrencysDecimalPlaces, .name = "anAmountFormatsWithItsCurrencysDecimalPlaces"},
        {.body = amountsConvertFromDoublesAsTheLibraryConvertsThem, .name = "amountsConvertFromDoublesAsTheLibraryConvertsThem"},
        {.body = ratesPercentagesAndPartsParseFormatAndCheckAsTheLibraryDoes, .name = "ratesPercentagesAndPartsParseFormatAndCheckAsTheLibraryDoes"},
        {
            .body = ratesPercentagesAndPartsConvertFromDoublesAsTheLibraryConvertsThem,
            .name = "ratesPercentagesAndPartsConvertFromDoublesAsTheLibraryConvertsThem",
        },
        {.body = quantitiesParseConvertFormatAndCheckAsTheLibraryDoes, .name = "quantitiesParseConvertFormatAndCheckAsTheLibraryDoes"},
        {
            .body = anItemsLineTotalIsTheLibrarysProductRoundedHalfAwayFromZero,
            .name = "anItemsLineTotalIsTheLibrarysProductRoundedHalfAwayFromZero",
        },
        {.body = aDiscountReadsBackAsItWasMadeAndStartsOffTheLine, .name = "aDiscountReadsBackAsItWasMadeAndStartsOffTheLine"},
        {.body = aDiscountComesOffEachUnitOrOffTheLineAsTheLibraryTakesIt, .name = "aDiscountComesOffEachUnitOrOffTheLineAsTheLibraryTakesIt"},
        {.body = aDiscountTheLineCannotTakeIsRefusedByEveryTotal, .name = "aDiscountTheLineCannotTakeIsRefusedByEveryTotal"},
        {.body = aDiscountedLineReadsBackFromTheGroupAsItWasAdded, .name = "aDiscountedLineReadsBackFromTheGroupAsItWasAdded"},
        {.body = datesAndTimesAreCheckedAndWritten, .name = "datesAndTimesAreCheckedAndWritten"},
        {.body = contactsAndAccountsAreCheckedAndMadeCanonical, .name = "contactsAndAccountsAreCheckedAndMadeCanonical"},
        {.body = anIdentifierIsCheckedAndKeptAsItIsWritten, .name = "anIdentifierIsCheckedAndKeptAsItIsWritten"},
        {.body = aGeneratedIdentifierIsNewAndOneTheRulesAccept, .name = "aGeneratedIdentifierIsNewAndOneTheRulesAccept"},
        {.body = aListOfIdentifiersKeepsItsOrder, .name = "aListOfIdentifiersKeepsItsOrder"},
        {.body = everyShareOfASplitReadsBackAsItWasMade, .name = "everyShareOfASplitReadsBackAsItWasMade"},
        {.body = itemsTipsAndCostsReadBackAsTheyWereMade, .name = "itemsTipsAndCostsReadBackAsTheyWereMade"},
        {.body = anExpenseReadsBackEveryFieldItWasGiven, .name = "anExpenseReadsBackEveryFieldItWasGiven"},
        {.body = aTransferReadsBackEveryFieldItWasGiven, .name = "aTransferReadsBackEveryFieldItWasGiven"},
        {.body = aMemberAndTheirAccountsReadBackAsTheyWereGiven, .name = "aMemberAndTheirAccountsReadBackAsTheyWereGiven"},
        {.body = aCategoryReadsBackAsItWasGiven, .name = "aCategoryReadsBackAsItWasGiven"},
        {.body = everyBalanceFormatsAsTheLibraryWritesIt, .name = "everyBalanceFormatsAsTheLibraryWritesIt"},
        {.body = destroyingNullDoesNothing, .name = "destroyingNullDoesNothing"},
        {.body = aGroupHoldsItsDetailsAndChangesThem, .name = "aGroupHoldsItsDetailsAndChangesThem"},
        {.body = entriesComeAndGoThroughTheGroup, .name = "entriesComeAndGoThroughTheGroup"},
        {
            .body = anExpenseWithoutACategoryOrATipReadsBackInTheBuiltInOneWithTheNoTipRule,
            .name = "anExpenseWithoutACategoryOrATipReadsBackInTheBuiltInOneWithTheNoTipRule",
        },
        {.body = anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt, .name = "anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt"},
        {
            .body = aPhoneNumberAndAnIbanComeBackFromTheGroupWithTheTextTheLibraryDerives,
            .name = "aPhoneNumberAndAnIbanComeBackFromTheGroupWithTheTextTheLibraryDerives",
        },
        {.body = anIdentifierThatIsNotOneIsRefusedByEveryLookup, .name = "anIdentifierThatIsNotOneIsRefusedByEveryLookup"},
        {
            .body = aLookupOfAnIdentifierNothingHasReportsNoErrorAndNoObject,
            .name = "aLookupOfAnIdentifierNothingHasReportsNoErrorAndNoObject",
        },
        {.body = theParticipantsOfASplitAndOfAnExpenseAreTheirMembers, .name = "theParticipantsOfASplitAndOfAnExpenseAreTheirMembers"},
        {.body = anEntryTheGroupDoesNotHoldIsNeitherReplacedNorRemoved, .name = "anEntryTheGroupDoesNotHoldIsNeitherReplacedNorRemoved"},
        {.body = theSummaryAndTheSharesAddUp, .name = "theSummaryAndTheSharesAddUp"},
        {.body = eachStrategySettlesEveryBalance, .name = "eachStrategySettlesEveryBalance"},
        {.body = anExactPlanTooLargeToComputeIsRefused, .name = "anExactPlanTooLargeToComputeIsRefused"},
        {.body = aChangeOfCurrencyConvertsEveryEntry, .name = "aChangeOfCurrencyConvertsEveryEntry"},
        {.body = aSavedGroupLoadsAsItWasSaved, .name = "aSavedGroupLoadsAsItWasSaved"},
        {.body = everyErrorHoldsWhatItsFailureHolds, .name = "everyErrorHoldsWhatItsFailureHolds"},
        {.body = everyAsynchronousCallReportsWhatItsSynchronousFormReturns, .name = "everyAsynchronousCallReportsWhatItsSynchronousFormReturns"},
        {.body = aReaderReadsTheGroupUntilItIsReleased, .name = "aReaderReadsTheGroupUntilItIsReleased"},
        {.body = everyReadOfAReleasedReaderFails, .name = "everyReadOfAReleasedReaderFails"},
        {
            .body = aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas,
            .name = "aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas",
        },
    };
    constexpr size_t COUNT = sizeof tests / sizeof tests[0];
    size_t failed = 0;
    for (size_t index = 0; COUNT > index; ++index) {
        Check check = {.test = tests[index].name, .failures = 0};
        tests[index].body(&check);
        failed += 0 == check.failures ? 0U : 1U;
    }
    (void)printf("%zu of %zu tests passed\n", COUNT - failed, COUNT);
    return 0 == failed ? EXIT_SUCCESS : EXIT_FAILURE;
}
