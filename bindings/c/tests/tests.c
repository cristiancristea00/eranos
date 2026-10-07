// The tests of the C layer: every value, record, and list through the functions that make and read it, every group call in
// both forms, the read handle, the cancellation, and every error the library reports with what it holds. Each test is a
// claim, named for what it states, whose comment says what breaks in an app if it regresses. The program prints every claim
// that fails and ends with a failure status when one does.

#include "eranos.h"

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

/// The amount of @p minorUnits minor units of @p currency.
[[nodiscard]] static EranosMoney *amountOf(int64_t const minorUnits, char const *const currency) {
    EranosMoney *money = nullptr;
    EranosError *const error = eranosMoneyCreate(minorUnits, currency, &money);
    eranosErrorDestroy(error);
    return money;
}

/// @p cents euro cents.
[[nodiscard]] static EranosMoney *euros(int64_t const cents) { return amountOf(cents, "EUR"); }

/// Whether @p money is @p minorUnits minor units of @p currency; destroys it.
[[nodiscard]] static bool isAmount(EranosMoney *const money, int64_t const minorUnits, char const *const currency) {
    if (nullptr == money) {
        return false;
    }
    EranosText *const code = eranosTextCreate();
    eranosMoneyCurrency(money, code);
    bool const same = eranosMoneyMinorUnits(money) == minorUnits && isText(code, currency);
    eranosTextDestroy(code);
    eranosMoneyDestroy(money);
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
    /// The error the call reported, or nullptr.
    EranosError *error;
    /// The answer the call reported.
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

/// Receives an answer.
static void receiveFlag(void *const context, EranosError *const error, bool const value) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->flag = value;
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
    checkSucceeded(check, eranosMoneyFormat(price, text), "an amount formats");
    checkThat(check, isText(text, "12.50 EUR") && eranosTextLength(text) == strlen("12.50 EUR"), "the amount as the library writes it");
    eranosMoneyCurrency(price, text);
    checkThat(check, isText(text, "EUR"), "a second call replaces the first's text");
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
    EranosText *const text = eranosTextCreate();
    checkThat(check, eranosErrorLeft(refused, text) && isText(text, "EUR") && eranosErrorRight(refused, text) && isText(text, "RON"),
              "the error names both currencies");
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
                  2 == allowed && eranosErrorCurrency(precise, text) && isText(text, "EUR"),
              "the euro has two decimal places, which the error holds with the currency");
    eranosErrorDestroy(precise);
    EranosMoney *const unlisted = amountOf(5, "XYZ");
    checkThat(check, isFailure(eranosMoneyFormat(unlisted, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY),
              "an amount in a currency the library does not list does not format");
    checkThat(check, isKind(eranosMoneyCreate(1, "\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT), "a currency's code is text");
    eranosMoneyDestroy(unlisted);
    eranosMoneyDestroy(largest);
    eranosTextDestroy(text);
    eranosMoneyDestroy(lei);
    eranosMoneyDestroy(quarter);
    eranosMoneyDestroy(price);
}

/// A rate, a percentage, or a number of parts read otherwise than the library reads it would divide or convert otherwise
/// than an app shows.
static void ratesPercentagesAndPartsParseAndFormatAsTheLibraryDoes(Check *const check) {
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
    checkSucceeded(check, eranosPercentageFormat(basisPoints, text), "and formats");
    checkThat(check, 1'250 == basisPoints && isText(text, "12.5%"), "12.5 % is 1250 basis points");
    checkThat(check,
              isFailure(eranosPercentageParse("half", &basisPoints), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE) &&
                  0 == basisPoints,
              "a word is no percentage, and gives none");
    checkThat(check, isFailure(eranosPercentageFormat(-1, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PERCENTAGE), "a negative one");
    int32_t hundredths = 0;
    checkSucceeded(check, eranosPartsParse("0.5", &hundredths), "parts parse");
    checkSucceeded(check, eranosPartsFormat(hundredths, text), "and format");
    checkThat(check, 50 == hundredths && isText(text, "0.5"), "half a part is 50 hundredths");
    checkThat(check, isFailure(eranosPartsParse("half", &hundredths), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PARTS),
              "a word is no parts");
    checkThat(check, isFailure(eranosPartsFormat(-1, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_INVALID_PARTS), "negative parts");
    eranosMoneyDestroy(lei);
    eranosTextDestroy(text);
}

/// A date or a time the calendar does not have, taken, would date an expense on a day that never was.
static void datesAndTimesAreCheckedAndWritten(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosCalendarDate date = {.year = 0, .month = 0, .day = 0};
    checkSucceeded(check, eranosCalendarDateParse("2026-09-23", &date), "a date parses");
    checkThat(check, 2026 == date.year && 9 == date.month && 23 == date.day, "into its year, month, and day");
    checkSucceeded(check, eranosCalendarDateFormat(date, text), "and formats");
    checkThat(check, isText(text, "2026-09-23"), "as the library writes it");
    constexpr EranosCalendarDate LEAP = {.year = 2026, .month = 2, .day = 30};
    checkThat(check, isFailure(eranosCalendarDateFormat(LEAP, text), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_DATE),
              "30 February");
    checkThat(check,
              isFailure(eranosCalendarDateParse("2026-02-30", &date), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_DATE) &&
                  0 == date.day,
              "parsed, it gives no date");
    EranosTimeOfDay time = {.hour = 1, .minute = 1};
    checkSucceeded(check, eranosTimeOfDayParse("18:30", &time), "a time parses");
    checkSucceeded(check, eranosTimeOfDayFormat(time, text), "and formats");
    checkThat(check, 18 == time.hour && 30 == time.minute && isText(text, "18:30"), "into its hour and minute");
    constexpr EranosTimeOfDay LATE = {.hour = 25, .minute = 0};
    checkThat(check, isFailure(eranosTimeOfDayFormat(LATE, text), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_TIME), "hour 25");
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
    checkSucceeded(check, eranosPhoneNumberParse("+40 712 345 678", text), "a phone number parses");
    checkThat(check, isText(text, "+40712345678"), "into its E.164 form");
    checkSucceeded(check, eranosPhoneNumberInternational("+40712345678", text), "and writes its international form");
    checkThat(check, isText(text, "+40 712 345 678"), "with spaces");
    checkThat(check, isFailure(eranosPhoneNumberParse("0712345678", text), ERANOS_ERROR_KIND_CONTACT, ERANOS_FAILURE_CONTACT_INVALID_PHONE_NUMBER),
              "a phone number has a country code");
    checkThat(check, isFailure(eranosPhoneNumberInternational("0712", text), ERANOS_ERROR_KIND_CONTACT, ERANOS_FAILURE_CONTACT_INVALID_PHONE_NUMBER),
              "in either form");
    checkSucceeded(check, eranosIbanParse("ro49 aaaa 1b31 0075 9384 0000", text), "an IBAN parses");
    checkThat(check, isText(text, "RO49AAAA1B31007593840000"), "into its electronic form");
    checkSucceeded(check, eranosIbanCountryCode("RO49AAAA1B31007593840000", text), "and names its country");
    checkThat(check, isText(text, "RO"), "Romania");
    EranosError *const checksum = eranosIbanParse("RO49AAAA1B31007593840001", text);
    EranosIbanReason reason = ERANOS_IBAN_REASON_INVALID_FORMAT;
    checkThat(check,
              holdsFailure(checksum, ERANOS_ERROR_KIND_PAYMENT_ACCOUNT, ERANOS_FAILURE_PAYMENT_ACCOUNT_INVALID_IBAN) &&
                  eranosErrorReason(checksum, &reason) && ERANOS_IBAN_REASON_INVALID_CHECKSUM == reason,
              "an IBAN whose check digits do not match");
    eranosErrorMessage(checksum, text);
    checkThat(check, isText(text, "invalid IBAN: the check digits do not match"), "the message names the failure, not the text");
    eranosErrorDestroy(checksum);
    EranosError *const length = eranosIbanParse("RO49AAAA1B3100759384000", text);
    int64_t expected = 0;
    int64_t found = 0;
    checkThat(check, eranosErrorExpectedLength(length, &expected) && 24 == expected && eranosErrorFoundLength(length, &found) && 23 == found,
              "an IBAN one character short holds both lengths");
    eranosErrorDestroy(length);
    checkThat(check, isFailure(eranosIbanCountryCode("X", text), ERANOS_ERROR_KIND_PAYMENT_ACCOUNT, ERANOS_FAILURE_PAYMENT_ACCOUNT_INVALID_IBAN),
              "a country code");
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
    checkThat(check, 1 == eranosExpenseItemQuantity(item), "one unit, by default");
    eranosExpenseItemSetQuantity(item, 2);
    checkSucceeded(check, eranosExpenseItemSetName(item, "Pizzas"), "a name is set");
    EranosSplit *const ana = equally("ana", nullptr);
    eranosExpenseItemSetSplit(item, ana);
    EranosMoney *const eleven = euros(1'100);
    eranosExpenseItemSetUnitPrice(item, eleven);
    eranosExpenseItemName(item, text);
    EranosSplit *const itemSplit = eranosExpenseItemSplit(item);
    checkThat(check,
              isText(text, "Pizzas") && 2 == eranosExpenseItemQuantity(item) && isAmount(eranosExpenseItemUnitPrice(item), 1'100, "EUR") &&
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
    checkThat(check, !eranosExpenseCategory(expense, text) && !eranosExpenseDescription(expense, text) && nullptr == eranosExpenseTip(expense),
              "and no category, description, or tip");
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
    checkThat(check, eranosExpenseCategory(expense, text) && isText(text, "0199a000-0000-7000-8000-000000000002"), "the category");
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
                  ERANOS_EXPENSE_SPLIT_KIND_WHOLE == eranosExpenseSplitKind(readSplit) && nullptr != readTip,
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
    EranosMoney *const unlisted = amountOf(100, "XYZ");
    EranosCost *const strange = eranosCostCreateLocal(unlisted);
    eranosExpenseSetCost(expense, strange);
    checkThat(check, isFailure(eranosExpenseTipAmount(expense, &amount), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY),
              "a tip on an amount in no listed currency");
    checkThat(check, isFailure(eranosExpenseTotal(expense, &amount), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY), "its total");
    checkThat(check,
              isKind(eranosExpenseSetId(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosExpenseSetName(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosExpenseSetCategory(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosExpenseSetDescription(expense, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "every text field is text");
    eranosExpenseClearCategory(expense);
    eranosExpenseClearDescription(expense);
    eranosExpenseClearTime(expense);
    eranosExpenseClearTip(expense);
    checkThat(check,
              !eranosExpenseCategory(expense, text) && !eranosExpenseDescription(expense, text) && !eranosExpenseTime(expense, &time) &&
                  nullptr == eranosExpenseTip(expense),
              "every optional field clears");
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
    eranosCostDestroy(strange);
    eranosMoneyDestroy(unlisted);
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
    checkSucceeded(check, eranosPaymentAccountDetailsCreateIban("RO49AAAA1B31007593840000", &bank), "an account's IBAN");
    EranosPaymentAccountDetails *app = nullptr;
    checkSucceeded(check, eranosPaymentAccountDetailsCreateUsername("ana.pays", &app), "an account's username");
    checkThat(check,
              ERANOS_PAYMENT_ACCOUNT_DETAILS_KIND_IBAN == eranosPaymentAccountDetailsKind(bank) && eranosPaymentAccountDetailsIban(bank, text) &&
                  isText(text, "RO49AAAA1B31007593840000") && !eranosPaymentAccountDetailsUsername(bank, text),
              "an IBAN account holds its IBAN");
    checkThat(check,
              ERANOS_PAYMENT_ACCOUNT_DETAILS_KIND_USERNAME == eranosPaymentAccountDetailsKind(app) &&
                  eranosPaymentAccountDetailsUsername(app, text) && isText(text, "ana.pays") && !eranosPaymentAccountDetailsIban(app, text),
              "a username account holds its username");
    EranosPaymentAccountDetails *none = nullptr;
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
    checkThat(check,
              !eranosMemberEmail(member, text) && !eranosMemberLastName(member, text) && !eranosMemberPhone(member, text) &&
                  eranosPaymentAccountListIsEmpty(noAccounts),
              "and a first name only");
    checkSucceeded(check, eranosMemberSetId(member, "0199a000-0000-7000-8000-000000000004"), "an identifier is set");
    checkSucceeded(check, eranosMemberSetFirstName(member, "Carla Maria"), "a first name");
    checkSucceeded(check, eranosMemberSetEmail(member, "carla@example.com"), "an email");
    checkSucceeded(check, eranosMemberSetLastName(member, "Pop"), "a last name");
    checkSucceeded(check, eranosMemberSetPhone(member, "+40712345678"), "a phone number");
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
              eranosMemberEmail(member, text) && isText(text, "carla@example.com") && eranosMemberLastName(member, text) && isText(text, "Pop") &&
                  eranosMemberPhone(member, text) && isText(text, "+40712345678"),
              "every detail as set");
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
    checkThat(check, !eranosMemberEmail(member, text) && !eranosMemberLastName(member, text) && !eranosMemberPhone(member, text), "and clears");
    EranosMember *carla = nullptr;
    checkSucceeded(check, eranosMemberCreateWithId("carla", "Carla", &carla), "a member under the app's identifier");
    eranosMemberId(carla, text);
    checkThat(check, isText(text, "carla"), "keeps it");
    EranosMember *noMember = nullptr;
    checkThat(check, isKind(eranosMemberCreate("\xff", &noMember), ERANOS_ERROR_KIND_INVALID_TEXT), "a first name is text");
    checkThat(check, isKind(eranosMemberCreateWithId("\xff", "Carla", &noMember), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == noMember,
              "an identifier is text");
    eranosMemberDestroy(carla);
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
    checkSucceeded(check, eranosBalanceFormat(getsBack, text), "a balance owed to a member formats");
    checkThat(check,
              isText(text, "gets back 12.00 EUR") && ERANOS_BALANCE_KIND_GETS_BACK == eranosBalanceKind(getsBack) &&
                  isAmount(eranosBalanceAmount(getsBack), 1'200, "EUR"),
              "as what the member gets back");
    checkSucceeded(check, eranosBalanceFormat(owes, text), "a balance a member owes formats");
    checkThat(check, isText(text, "owes 12.00 EUR") && ERANOS_BALANCE_KIND_OWES == eranosBalanceKind(owes), "as what the member owes");
    checkSucceeded(check, eranosBalanceFormat(settled, text), "a settled balance formats");
    checkThat(check, isText(text, "settled") && ERANOS_BALANCE_KIND_SETTLED == eranosBalanceKind(settled) && nullptr == eranosBalanceAmount(settled),
              "with no amount");
    EranosMoney *const unlisted = amountOf(1, "XYZ");
    EranosBalance *const strange = eranosBalanceCreateOwes(unlisted);
    checkThat(check, isFailure(eranosBalanceFormat(strange, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY),
              "an unlisted currency");
    eranosBalanceDestroy(strange);
    eranosMoneyDestroy(unlisted);
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
    checkSucceeded(check, eranosExpenseGroupId(home, text), "has an identifier");
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
    checkSucceeded(check, eranosExpenseGroupId(group, text), "has an identifier");
    checkThat(check, isText(text, eranosTextData(id)) && eranosTextLength(text) == strlen("0199a000-0000-7000-8000-000000000001"),
              "the generated one, a hyphenated UUID");
    checkSucceeded(check, eranosExpenseGroupCurrency(group, text), "a currency");
    checkThat(check, isText(text, "RON"), "the one it started with");
    checkSucceeded(check, eranosExpenseGroupSetName(group, "Trip"), "a name is set");
    checkSucceeded(check, eranosExpenseGroupName(group, text), "and read");
    checkThat(check, isText(text, "Trip"), "the name as set");
    checkSucceeded(check, eranosExpenseGroupSetDescription(group, "Summer"), "a description is set");
    checkSucceeded(check, eranosExpenseGroupDescription(group, text), "and read");
    checkThat(check, isText(text, "Summer"), "the description as set");
    checkSucceeded(check, eranosExpenseGroupSetDescription(group, ""), "blank text clears it");
    checkSucceeded(check, eranosExpenseGroupDescription(group, text), "and it reads");
    checkThat(check, isText(text, ""), "as nothing");
    checkSucceeded(check, eranosExpenseGroupSetLanguage(group, "pt-pt"), "a language is set");
    checkSucceeded(check, eranosExpenseGroupLanguage(group, text), "and read");
    checkThat(check, isText(text, "pt-PT"), "canonical");
    checkSucceeded(check, eranosExpenseGroupClearLanguage(group), "a language clears");
    checkSucceeded(check, eranosExpenseGroupLanguage(group, text), "and reads");
    checkThat(check, isText(text, ""), "as nothing");
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
    EranosExpenseList *expenses = nullptr;
    checkSucceeded(check, eranosExpenseGroupExpenses(flat.group, &expenses), "the expenses");
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
    checkThat(check, isText(text, "Dinner"), "which comes back as it was");
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
    EranosTransferList *transfers = nullptr;
    checkSucceeded(check, eranosExpenseGroupTransfers(flat.group, &transfers), "the transfers");
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
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosExpenseGroupTransferAmount(flat.group, eranosTextData(flat.repayment), &amount), "its amount");
    checkThat(check, isAmount(amount, 1'000, "EUR"), "in the group currency");
    EranosTransfer *removedTransfer = nullptr;
    checkSucceeded(check, eranosExpenseGroupRemoveTransfer(flat.group, eranosTextData(flat.repayment), &removedTransfer), "the transfer goes");
    checkThat(check, nullptr != removedTransfer, "and comes back");
    EranosCategoryList *categories = nullptr;
    checkSucceeded(check, eranosExpenseGroupCategories(flat.group, &categories), "the categories");
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
    EranosMemberList *members = nullptr;
    checkSucceeded(check, eranosExpenseGroupMembers(flat.group, &members), "the members");
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
    bool archived = true;
    checkSucceeded(check, eranosExpenseGroupArchiveMember(flat.group, eranosTextData(flat.bob)), "Bob is archived");
    checkSucceeded(check, eranosExpenseGroupIsArchived(flat.group, eranosTextData(flat.bob), &archived), "and asked about");
    checkThat(check, archived, "and is");
    checkSucceeded(check, eranosExpenseGroupUnarchiveMember(flat.group, eranosTextData(flat.bob)), "Bob is back");
    checkSucceeded(check, eranosExpenseGroupIsArchived(flat.group, eranosTextData(flat.bob), &archived), "and asked about");
    checkThat(check, !archived, "and is not");
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

/// An identifier that is not one would reach the library as no entry's, or as another's; each of the group's lookups refuses
/// it, with the failure of the rule it breaks, and gives no object.
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
        EranosTransfer *transfer = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupTransfer(flat.group, nobody, &transfer), ERANOS_ERROR_KIND_ID, failure) && nullptr == transfer,
                  "a transfer");
        EranosMoney *amount = nullptr;
        checkThat(check, isFailure(eranosExpenseGroupTransferAmount(flat.group, nobody, &amount), ERANOS_ERROR_KIND_ID, failure) && nullptr == amount,
                  "a transfer's amount");
    }
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
    EranosSummary *summary = nullptr;
    checkSucceeded(check, eranosExpenseGroupSummary(flat.group, &summary), "the summary");
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
                  eranosErrorCurrency(missing, text) && isText(text, "EUR"),
              "the euro needs a rate");
    eranosErrorDestroy(missing);
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    checkThat(check, eranosCurrencyRateListIsEmpty(rates), "an empty list of rates");
    eranosCurrencyRateListPush(rates, rate);
    EranosCurrencyRate *const first = eranosCurrencyRateListGet(rates, 0);
    checkThat(check, 1 == eranosCurrencyRateListLength(rates) && nullptr != first && nullptr == eranosCurrencyRateListGet(rates, 1), "one rate");
    checkSucceeded(check, eranosExpenseGroupChangeCurrency(flat.group, "RON", rates), "the change");
    checkSucceeded(check, eranosExpenseGroupCurrency(flat.group, text), "the currency");
    checkThat(check, isText(text, "RON"), "is the new one");
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
    checkSucceeded(check, eranosExpenseGroupSave(flat.group, json), "the group saves");
    EranosExpenseGroup *loaded = nullptr;
    checkSucceeded(check, eranosExpenseGroupLoad(eranosTextData(json), &loaded), "and loads");
    checkSucceeded(check, eranosExpenseGroupSave(loaded, again), "and saves again");
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
                  !eranosErrorCurrency(inUse, text) && !eranosErrorExisting(inUse, text) && !eranosErrorExpectedCurrency(inUse, text) &&
                  !eranosErrorExpectedLength(inUse, &number) && !eranosErrorExpense(inUse, text) && !eranosErrorFormat(inUse, &number) &&
                  !eranosErrorFoundCurrency(inUse, text) && !eranosErrorFoundLength(inUse, &number) && !eranosErrorLeft(inUse, text) &&
                  !eranosErrorLine(inUse, &number) && !eranosErrorReason(inUse, &reason) && !eranosErrorRight(inUse, text) &&
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
                  eranosErrorExpectedCurrency(currencies, text) && isText(text, "EUR") && eranosErrorFoundCurrency(currencies, text) &&
                  isText(text, "RON"),
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
    eranosErrorDestroy(eranosExpenseGroupSave(flat.group, json));
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
    checkSucceeded(check, eranosExpenseGroupId(flat.group, text), "the identifier");
    checkThat(check, nullptr == waiter.error && isText(waiter.text, eranosTextData(text)), "the same identifier");
    eranosTextDestroy(waiter.text);
    EranosTextCallback const nameCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupNameAsync(flat.group, nullptr, &nameCallback);
    await(&waiter);
    checkThat(check, isText(waiter.text, "Flat"), "the name");
    eranosTextDestroy(waiter.text);
    EranosTextCallback const currencyCallback = {waiting(&waiter), receiveText};
    eranosExpenseGroupCurrencyAsync(flat.group, nullptr, &currencyCallback);
    await(&waiter);
    checkThat(check, isText(waiter.text, "EUR"), "the currency");
    eranosTextDestroy(waiter.text);
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
    checkSucceeded(check, eranosExpenseGroupSave(flat.group, text), "the saved group");
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
    EranosCategoryCallback const categoryCallback = {waiting(&waiter), receiveCategory};
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
    EranosExpenseCallback const expenseCallback = {waiting(&waiter), receiveExpense};
    eranosExpenseGroupExpenseAsync(flat.group, eranosTextData(flat.dinner), nullptr, &expenseCallback);
    await(&waiter);
    EranosExpense *const dinner = waiter.object;
    checkThat(check, nullptr != dinner, "an expense");
    EranosExpenseSharesCallback const sharesCallback = {waiting(&waiter), receiveExpenseShares};
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
    EranosMemberCallback const memberCallback = {waiting(&waiter), receiveMember};
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
    EranosFlagCallback const archivedCallback = {waiting(&waiter), receiveFlag};
    eranosExpenseGroupIsArchivedAsync(flat.group, eranosTextData(carlaId), nullptr, &archivedCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && waiter.flag, "an archived member is archived");
    EranosUnitCallback const unarchiveCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupUnarchiveMemberAsync(flat.group, eranosTextData(carlaId), nullptr, &unarchiveCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error, "and comes back");
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
    EranosTransferCallback const transferCallback = {waiting(&waiter), receiveTransfer};
    eranosExpenseGroupTransferAsync(flat.group, eranosTextData(flat.repayment), nullptr, &transferCallback);
    await(&waiter);
    EranosTransfer *const repayment = waiter.object;
    EranosMoneyCallback const amountCallback = {waiting(&waiter), receiveMoney};
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
    eranosExpenseGroupSetDescriptionAsync(flat.group, "", nullptr, &setDescriptionCallback);
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
    checkSucceeded(check, eranosExpenseGroupName(flat.group, text), "the name");
    checkThat(check, isText(text, "Home"), "as set");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = nullptr;
    eranosErrorDestroy(eranosCurrencyRateCreate("EUR", "5", &rate));
    eranosCurrencyRateListPush(rates, rate);
    EranosUnitCallback const changeCallback = {waiting(&waiter), receiveUnit};
    eranosExpenseGroupChangeCurrencyAsync(flat.group, "RON", rates, nullptr, &changeCallback);
    await(&waiter);
    checkSucceeded(check, eranosExpenseGroupCurrency(flat.group, text), "the currency");
    checkThat(check, nullptr == waiter.error && isText(text, "RON"), "changed");
    eranosCurrencyRateDestroy(rate);
    eranosCurrencyRateListDestroy(rates);
    EranosMemberCallback const failingCallback = {waiting(&waiter), receiveMember};
    eranosExpenseGroupRemoveMemberAsync(flat.group, "0199a000-0000-7000-8000-00000000ffff", nullptr, &failingCallback);
    await(&waiter);
    checkThat(check,
              isFailure(waiter.error, ERANOS_ERROR_KIND_EXPENSE_GROUP, ERANOS_FAILURE_EXPENSE_GROUP_UNKNOWN_MEMBER) && nullptr == waiter.object,
              "a call that fails reports its error and no value");
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
    checkSucceeded(check, eranosExpenseGroupReaderCurrency(reader, text), "the currency");
    checkThat(check, isText(text, "EUR"), "the euro");
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
    bool archived = true;
    checkSucceeded(check, eranosExpenseGroupReaderIsArchived(reader, eranosTextData(flat.ana), &archived), "whether a member is archived");
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
                  nullptr != shares && nullptr != summary && 2 == eranosMemberListLength(members) && nullptr != member && !archived &&
                  nullptr != settlement && 1 == eranosTransferListLength(transfers) && nullptr != transfer && isAmount(amount, 1'000, "EUR"),
              "every read sees the group");
    eranosExpenseGroupReaderRelease(reader);
    EranosError *const released = eranosExpenseGroupReaderName(reader, text);
    checkThat(check, isKind(released, ERANOS_ERROR_KIND_RELEASED), "a read after the release");
    EranosExpenseGroupReader *synchronous = nullptr;
    checkSucceeded(check, eranosExpenseGroupRead(flat.group, &synchronous), "a read handle from a synchronous call");
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
/// read that returns an object fails once the handle is released, and gives no object.
static void everyReadOfAReleasedReaderFails(Check *const check) {
    Flat const flat = flatOf();
    EranosExpenseGroupReader *reader = nullptr;
    checkSucceeded(check, eranosExpenseGroupRead(flat.group, &reader), "a read handle");
    eranosExpenseGroupReaderRelease(reader);
    char const *const food = eranosTextData(flat.food);
    char const *const dinner = eranosTextData(flat.dinner);
    char const *const repayment = eranosTextData(flat.repayment);
    EranosCategoryList *categories = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderCategories(reader, &categories), ERANOS_ERROR_KIND_RELEASED) && nullptr == categories,
              "the categories");
    EranosCategory *category = nullptr;
    checkThat(check, isKind(eranosExpenseGroupReaderCategory(reader, food, &category), ERANOS_ERROR_KIND_RELEASED) && nullptr == category,
              "a category");
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
    EranosExpenseGroupReader *reader = nullptr;
    checkSucceeded(check, eranosExpenseGroupRead(flat.group, &reader), "a read handle");
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
    checkSucceeded(check, eranosExpenseGroupCurrency(flat.group, text), "the currency");
    checkThat(check, isText(text, "EUR"), "is still the euro");
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
        {.body = ratesPercentagesAndPartsParseAndFormatAsTheLibraryDoes, .name = "ratesPercentagesAndPartsParseAndFormatAsTheLibraryDoes"},
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
        {.body = anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt, .name = "anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt"},
        {.body = anIdentifierThatIsNotOneIsRefusedByEveryLookup, .name = "anIdentifierThatIsNotOneIsRefusedByEveryLookup"},
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
