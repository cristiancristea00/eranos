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
static bool isText(EranosText const *const text, char const *const expected) { return 0 == strcmp(eranosTextData(text), expected); }

/// Whether @p error is of @p kind, and holds no failure within a module; destroys it.
static bool isKind(EranosError *const error, EranosErrorKind const kind) {
    if (nullptr == error) {
        return false;
    }
    EranosFailure failure = ERANOS_FAILURE_ACCOUNT_EMPTY_NAME;
    bool const same = eranosErrorKind(error) == kind && !eranosErrorFailure(error, &failure);
    eranosErrorDestroy(error);
    return same;
}

/// Whether @p error is of @p kind with @p expected; destroys it.
static bool isFailure(EranosError *const error, EranosErrorKind const kind, EranosFailure const expected) {
    if (nullptr == error) {
        return false;
    }
    EranosFailure failure = ERANOS_FAILURE_ACCOUNT_EMPTY_NAME;
    bool const same = eranosErrorKind(error) == kind && eranosErrorFailure(error, &failure) && failure == expected;
    eranosErrorDestroy(error);
    return same;
}

/// Whether @p error is of @p kind with @p failure, without destroying it, so that a test reads what it holds.
static bool holdsFailure(EranosError const *const error, EranosErrorKind const kind, EranosFailure const expected) {
    EranosFailure failure = ERANOS_FAILURE_ACCOUNT_EMPTY_NAME;
    return nullptr != error && eranosErrorKind(error) == kind && eranosErrorFailure(error, &failure) && failure == expected;
}

/// The amount of @p minorUnits minor units of @p currency.
static EranosMoney *amountOf(char const *const currency, int64_t const minorUnits) {
    EranosMoney *money = nullptr;
    EranosError *const error = eranosMoneyCreate(currency, minorUnits, &money);
    eranosErrorDestroy(error);
    return money;
}

/// @p cents euro cents.
static EranosMoney *euros(int64_t const cents) { return amountOf("EUR", cents); }

/// Whether @p money is @p minorUnits minor units of @p currency; destroys it.
static bool isAmount(EranosMoney *const money, char const *const currency, int64_t const minorUnits) {
    if (nullptr == money) {
        return false;
    }
    EranosText *const code = eranosTextCreate();
    eranosMoneyCurrency(money, code);
    bool const same = isText(code, currency) && eranosMoneyMinorUnits(money) == minorUnits;
    eranosTextDestroy(code);
    eranosMoneyDestroy(money);
    return same;
}

/// The list of @p first and, unless nullptr, @p second.
static EranosTextList *membersOf(char const *const first, char const *const second) {
    EranosTextList *const members = eranosTextListCreate();
    eranosErrorDestroy(eranosTextListPush(members, first));
    if (nullptr != second) {
        eranosErrorDestroy(eranosTextListPush(members, second));
    }
    return members;
}

/// The split that divides equally among @p first and, unless nullptr, @p second.
static EranosSplit *equally(char const *const first, char const *const second) {
    EranosTextList *const members = membersOf(first, second);
    EranosSplit *const split = eranosSplitCreateEqual(members);
    eranosTextListDestroy(members);
    return split;
}

/// Whether @p list holds @p first and then, unless nullptr, @p second; destroys it.
static bool listsMembers(EranosTextList *const list, char const *const first, char const *const second) {
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
static constexpr EranosCalendarDate DAY = {.day = 23, .month = 9, .year = 2026};

/// An expense of 30.00 EUR, called @p name, that @p payer paid and @p first and @p second share equally.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the name, the payer, and the members, in the order the expense reads them.
static EranosExpense *expenseOf(char const *const name, char const *const payer, char const *const first, char const *const second) {
    EranosMoney *const thirty = euros(3'000);
    EranosCost *const cost = eranosCostCreateLocal(thirty);
    EranosSplit *const payers = equally(payer, nullptr);
    EranosSplit *const shares = equally(first, second);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateWhole(shares);
    EranosExpense *expense = nullptr;
    eranosErrorDestroy(eranosExpenseCreate(cost, DAY, name, payers, split, &expense));
    eranosExpenseSplitDestroy(split);
    eranosSplitDestroy(shares);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(thirty);
    return expense;
}

/// A transfer of @p cents euro cents from @p from to @p to.
static EranosTransfer *transferOf(int64_t const cents, char const *const from, char const *const to) {
    EranosMoney *const amount = euros(cents);
    EranosCost *const cost = eranosCostCreateLocal(amount);
    EranosTransfer *transfer = nullptr;
    eranosErrorDestroy(eranosTransferCreate(cost, DAY, from, to, &transfer));
    eranosCostDestroy(cost);
    eranosMoneyDestroy(amount);
    return transfer;
}

/// A group of two, Ana and Bob, with a category, an expense, and a transfer, and the identifiers the group gave them.
typedef struct Flat {
    /// The group.
    EranosGroup *group;
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
static EranosText *memberIn(EranosGroup *const group, char const *const firstName) {
    EranosMember *member = nullptr;
    eranosErrorDestroy(eranosMemberCreate(firstName, &member));
    EranosText *const id = eranosTextCreate();
    eranosErrorDestroy(eranosGroupAddMember(group, member, id));
    eranosMemberDestroy(member);
    return id;
}

/// The flat.
static Flat flatOf(void) {
    EranosNewGroup *details = nullptr;
    eranosErrorDestroy(eranosNewGroupCreate("EUR", "Flat", &details));
    eranosErrorDestroy(eranosNewGroupSetDescription(details, "Rent"));
    eranosErrorDestroy(eranosNewGroupSetLanguage(details, "pt-pt"));
    Flat flat = {.group = nullptr};
    eranosErrorDestroy(eranosGroupCreate(details, &flat.group));
    eranosNewGroupDestroy(details);
    flat.ana = memberIn(flat.group, "Ana");
    flat.bob = memberIn(flat.group, "Bob");
    EranosCategory *food = nullptr;
    eranosErrorDestroy(eranosCategoryCreate("Food", &food));
    flat.food = eranosTextCreate();
    eranosErrorDestroy(eranosGroupAddCategory(flat.group, food, flat.food));
    eranosCategoryDestroy(food);
    EranosExpense *const dinner = expenseOf("Dinner", eranosTextData(flat.ana), eranosTextData(flat.ana), eranosTextData(flat.bob));
    eranosErrorDestroy(eranosExpenseSetCategory(dinner, eranosTextData(flat.food)));
    flat.dinner = eranosTextCreate();
    eranosErrorDestroy(eranosGroupAddExpense(flat.group, dinner, flat.dinner));
    eranosExpenseDestroy(dinner);
    EranosTransfer *const repayment = transferOf(1'000, eranosTextData(flat.bob), eranosTextData(flat.ana));
    flat.repayment = eranosTextCreate();
    eranosErrorDestroy(eranosGroupAddTransfer(flat.group, repayment, flat.repayment));
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
    eranosGroupDestroy(flat.group);
}

/// What an asynchronous call reported, and whether it has: its callback runs on another thread, which signals the waiting one.
typedef struct Waiter {
    /// Guards the rest.
    pthread_mutex_t mutex;
    /// Signalled once the callback ran.
    pthread_cond_t condition;
    /// Whether the callback ran.
    bool done;
    /// The error the call reported, or nullptr.
    EranosError *error;
    /// The text the call reported, or nullptr.
    EranosText *text;
    /// The object the call reported, or nullptr.
    void *object;
    /// The answer the call reported.
    bool flag;
} Waiter;

/// @p waiter, ready for a call.
static Waiter *waiting(Waiter *const waiter) {
    waiter->done = false;
    waiter->error = nullptr;
    waiter->text = nullptr;
    waiter->object = nullptr;
    waiter->flag = false;
    (void)pthread_mutex_init(&waiter->mutex, nullptr);
    (void)pthread_cond_init(&waiter->condition, nullptr);
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
static void receiveGroup(void *const context, EranosError *const error, EranosGroup *const group) {
    Waiter *const waiter = context;
    waiter->error = error;
    waiter->object = group;
    finish(waiter);
}

/// Receives a read handle.
static void receiveReader(void *const context, EranosError *const error, EranosGroupReader *const reader) {
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
static void receiveSettlement(void *const context, EranosError *const error, EranosSettlement *const settlement) {
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
    checkThat(check, isAmount(sum, "EUR", 1'500), "12.50 EUR and 2.50 EUR make 15.00 EUR");
    EranosMoney *difference = nullptr;
    checkSucceeded(check, eranosMoneySubtract(price, quarter, &difference), "and subtract");
    checkThat(check, isAmount(difference, "EUR", 1'000), "12.50 EUR less 2.50 EUR is 10.00 EUR");
    EranosMoney *const lei = amountOf("RON", 100);
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
    EranosMoney *const unlisted = amountOf("XYZ", 5);
    checkThat(check, isFailure(eranosMoneyFormat(unlisted, text), ERANOS_ERROR_KIND_MONEY, ERANOS_FAILURE_MONEY_UNKNOWN_CURRENCY),
              "an amount in a currency the library does not list does not format");
    checkThat(check, isKind(eranosMoneyCreate("\xff", 1, &none), ERANOS_ERROR_KIND_INVALID_TEXT), "a currency's code is text");
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
    EranosMoney *const lei = amountOf("RON", 2);
    EranosMoney *converted = nullptr;
    checkSucceeded(check, eranosExchangeRateConvert("0.25", lei, "EUR", &converted), "an amount converts");
    checkThat(check, isAmount(converted, "EUR", 1), "0.02 RON at 0.25 is 0.01 EUR, rounded half away from zero");
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
    EranosCalendarDate date = {.day = 0, .month = 0, .year = 0};
    checkSucceeded(check, eranosCalendarDateParse("2026-09-23", &date), "a date parses");
    checkThat(check, 23 == date.day && 9 == date.month && 2026 == date.year, "into its day, month, and year");
    checkSucceeded(check, eranosCalendarDateFormat(date, text), "and formats");
    checkThat(check, isText(text, "2026-09-23"), "as the library writes it");
    EranosCalendarDate const leap = {.day = 30, .month = 2, .year = 2026};
    checkThat(check, isFailure(eranosCalendarDateFormat(leap, text), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_DATE),
              "30 February");
    checkThat(check,
              isFailure(eranosCalendarDateParse("2026-02-30", &date), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_DATE) &&
                  0 == date.day,
              "parsed, it gives no date");
    EranosTimeOfDay time = {.hour = 1, .minute = 1};
    checkSucceeded(check, eranosTimeOfDayParse("18:30", &time), "a time parses");
    checkSucceeded(check, eranosTimeOfDayFormat(time, text), "and formats");
    checkThat(check, 18 == time.hour && 30 == time.minute && isText(text, "18:30"), "into its hour and minute");
    EranosTimeOfDay const late = {.hour = 25, .minute = 0};
    checkThat(check, isFailure(eranosTimeOfDayFormat(late, text), ERANOS_ERROR_KIND_CALENDAR, ERANOS_FAILURE_CALENDAR_INVALID_TIME), "hour 25");
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
              holdsFailure(checksum, ERANOS_ERROR_KIND_ACCOUNT, ERANOS_FAILURE_ACCOUNT_INVALID_IBAN) && eranosErrorReason(checksum, &reason) &&
                  ERANOS_IBAN_REASON_INVALID_CHECKSUM == reason,
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
    checkThat(check, isFailure(eranosIbanCountryCode("X", text), ERANOS_ERROR_KIND_ACCOUNT, ERANOS_FAILURE_ACCOUNT_INVALID_IBAN), "a country code");
    checkSucceeded(check, eranosUsernameParse(" ana.pays ", text), "a username parses");
    checkThat(check, isText(text, "ana.pays"), "trimmed");
    checkThat(check, isFailure(eranosUsernameParse(" ", text), ERANOS_ERROR_KIND_ACCOUNT, ERANOS_FAILURE_ACCOUNT_EMPTY_USERNAME), "a blank username");
    checkSucceeded(check, eranosLanguageParse("pt-pt", text), "a language parses");
    checkThat(check, isText(text, "pt-PT"), "into its canonical tag");
    checkThat(check, isFailure(eranosLanguageParse("not a tag", text), ERANOS_ERROR_KIND_LANGUAGE, ERANOS_FAILURE_LANGUAGE_INVALID_LANGUAGE),
              "no tag");
    eranosTextDestroy(text);
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
    checkSucceeded(check, eranosMemberAmountCreate(ten, "ana", &amount), "a member's amount");
    checkThat(check, isAmount(eranosMemberAmountAmount(amount), "EUR", 1'000), "its amount");
    eranosMemberAmountMember(amount, text);
    checkThat(check, isText(text, "ana"), "its member");
    EranosMoney *const twenty = euros(2'000);
    eranosMemberAmountSetAmount(amount, twenty);
    checkSucceeded(check, eranosMemberAmountSetMember(amount, "bob"), "a member is set");
    eranosMemberAmountMember(amount, text);
    checkThat(check, isAmount(eranosMemberAmountAmount(amount), "EUR", 2'000) && isText(text, "bob"), "the last value set wins");
    checkThat(check, isKind(eranosMemberAmountSetMember(amount, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a member is text");
    EranosMemberAmountList *const amounts = eranosMemberAmountListCreate();
    checkThat(check, eranosMemberAmountListIsEmpty(amounts), "an empty list of amounts");
    eranosMemberAmountListPush(amounts, amount);
    EranosMemberAmount *const firstAmount = eranosMemberAmountListGet(amounts, 0);
    checkThat(check, 1 == eranosMemberAmountListLength(amounts) && nullptr != firstAmount && nullptr == eranosMemberAmountListGet(amounts, 1),
              "one amount");
    eranosMemberAmountDestroy(firstAmount);
    EranosMemberParts *parts = nullptr;
    checkSucceeded(check, eranosMemberPartsCreate(100, "ana", &parts), "a member's parts");
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
    checkSucceeded(check, eranosMemberPercentageCreate(10'000, "ana", &percentage), "a member's percentage");
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
    checkThat(check, isKind(eranosMemberAmountCreate(ten, "\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT), "an amount's member is text");
    EranosMemberParts *noParts = nullptr;
    checkThat(check, isKind(eranosMemberPartsCreate(1, "\xff", &noParts), ERANOS_ERROR_KIND_INVALID_TEXT), "a part's member is text");
    EranosMemberPercentage *noPercentage = nullptr;
    checkThat(check, isKind(eranosMemberPercentageCreate(1, "\xff", &noPercentage), ERANOS_ERROR_KIND_INVALID_TEXT), "a percentage's member is text");

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
    EranosTextList *participants = nullptr;
    checkThat(check, isFailure(eranosSplitParticipants(equal, &participants), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID),
              "a split's members are identifiers");
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
    EranosItem *item = nullptr;
    checkSucceeded(check, eranosItemCreate("Pizza", both, twelve, &item), "an item");
    checkThat(check, 1 == eranosItemQuantity(item), "one unit, by default");
    eranosItemSetQuantity(item, 2);
    checkSucceeded(check, eranosItemSetName(item, "Pizzas"), "a name is set");
    EranosSplit *const ana = equally("ana", nullptr);
    eranosItemSetSplit(item, ana);
    EranosMoney *const eleven = euros(1'100);
    eranosItemSetUnitPrice(item, eleven);
    eranosItemName(item, text);
    EranosSplit *const itemSplit = eranosItemSplit(item);
    checkThat(check,
              isText(text, "Pizzas") && 2 == eranosItemQuantity(item) && isAmount(eranosItemUnitPrice(item), "EUR", 1'100) &&
                  listsMembers(eranosSplitMembers(itemSplit), "ana", nullptr),
              "the item as set");
    checkThat(check, isKind(eranosItemSetName(item, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
    EranosItem *noItem = nullptr;
    checkThat(check, isKind(eranosItemCreate("\xff", both, twelve, &noItem), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == noItem,
              "an item's name is text");
    EranosItemList *const items = eranosItemListCreate();
    checkThat(check, eranosItemListIsEmpty(items), "an empty list of items");
    eranosItemListPush(items, item);
    EranosItem *const firstItem = eranosItemListGet(items, 0);
    checkThat(check, 1 == eranosItemListLength(items) && nullptr != firstItem && nullptr == eranosItemListGet(items, 1), "one item");
    eranosItemDestroy(firstItem);
    EranosExpenseSplit *const itemised = eranosExpenseSplitCreateItemised(items);
    EranosExpenseSplit *const whole = eranosExpenseSplitCreateWhole(both);
    EranosItemList *const readItems = eranosExpenseSplitItems(itemised);
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
              ERANOS_TIP_VALUE_KIND_FIXED == eranosTipValueKind(fixed) && isAmount(eranosTipValueAmount(fixed), "EUR", 300) &&
                  !eranosTipValueBasisPoints(fixed, &basisPoints),
              "a fixed tip holds its amount");
    checkThat(check,
              ERANOS_TIP_VALUE_KIND_PERCENTAGE == eranosTipValueKind(tenPercent) && eranosTipValueBasisPoints(tenPercent, &basisPoints) &&
                  1'000 == basisPoints && nullptr == eranosTipValueAmount(tenPercent),
              "a percentage tip holds its basis points");
    EranosTip *const tip = eranosTipCreate(fixed, ERANOS_TIP_DISTRIBUTION_EQUAL);
    eranosTipSetValue(tip, tenPercent);
    eranosTipSetDistribution(tip, ERANOS_TIP_DISTRIBUTION_PROPORTIONAL);
    EranosTipValue *const tipValue = eranosTipValue(tip);
    checkThat(check,
              ERANOS_TIP_DISTRIBUTION_PROPORTIONAL == eranosTipDistribution(tip) && ERANOS_TIP_VALUE_KIND_PERCENTAGE == eranosTipValueKind(tipValue),
              "a tip as set");
    EranosCost *const local = eranosCostCreateLocal(twelve);
    EranosMoney *const lei = amountOf("RON", 5'000);
    EranosCost *converted = nullptr;
    checkSucceeded(check, eranosCostCreateConverted(lei, "0.2", &converted), "a converted cost");
    checkThat(check,
              ERANOS_COST_KIND_LOCAL == eranosCostKind(local) && isAmount(eranosCostAmount(local), "EUR", 1'200) && !eranosCostRate(local, text),
              "a local cost has no rate");
    checkThat(check,
              ERANOS_COST_KIND_CONVERTED == eranosCostKind(converted) && isAmount(eranosCostAmount(converted), "RON", 5'000) &&
                  eranosCostRate(converted, text) && isText(text, "0.2"),
              "a converted cost holds its rate");
    EranosCost *noCost = nullptr;
    checkThat(check, isKind(eranosCostCreateConverted(lei, "\xff", &noCost), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == noCost, "a rate is text");
    eranosCostDestroy(converted);
    eranosCostDestroy(local);
    eranosMoneyDestroy(lei);
    eranosTipValueDestroy(tipValue);
    eranosTipDestroy(tip);
    eranosTipValueDestroy(tenPercent);
    eranosTipValueDestroy(fixed);
    eranosMoneyDestroy(three);
    eranosSplitDestroy(readSplit);
    eranosItemListDestroy(readItems);
    eranosExpenseSplitDestroy(whole);
    eranosExpenseSplitDestroy(itemised);
    eranosItemListDestroy(items);
    eranosSplitDestroy(itemSplit);
    eranosMoneyDestroy(eleven);
    eranosSplitDestroy(ana);
    eranosItemDestroy(item);
    eranosSplitDestroy(both);
    eranosMoneyDestroy(twelve);
    eranosTextDestroy(text);
}

/// An expense whose fields did not read back as set would change what an app shows after it edits one.
static void anExpenseReadsBackEveryFieldItWasGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosExpense *const expense = expenseOf("Dinner", "ana", "ana", "bob");
    checkThat(check,
              !eranosExpenseId(expense, text) && !eranosExpenseCategory(expense, text) && !eranosExpenseDescription(expense, text) &&
                  nullptr == eranosExpenseTip(expense),
              "a new expense has no identifier, category, description, or tip");
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    checkThat(check, !eranosExpenseTime(expense, &time), "nor time");
    checkSucceeded(check, eranosExpenseSetId(expense, "0199a000-0000-7000-8000-000000000001"), "an identifier is set");
    checkSucceeded(check, eranosExpenseSetName(expense, "Lunch"), "a name");
    checkSucceeded(check, eranosExpenseSetCategory(expense, "0199a000-0000-7000-8000-000000000002"), "a category");
    checkSucceeded(check, eranosExpenseSetDescription(expense, "By the sea"), "a description");
    EranosTimeOfDay const noon = {.hour = 12, .minute = 15};
    eranosExpenseSetTime(expense, noon);
    EranosCalendarDate const next = {.day = 24, .month = 9, .year = 2026};
    eranosExpenseSetDate(expense, next);
    EranosMoney *const forty = euros(4'000);
    EranosCost *const cost = eranosCostCreateLocal(forty);
    eranosExpenseSetCost(expense, cost);
    EranosSplit *const bob = equally("bob", nullptr);
    eranosExpenseSetPayers(expense, bob);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateWhole(bob);
    eranosExpenseSetSplit(expense, split);
    EranosTipValue *const tenPercent = eranosTipValueCreatePercentage(1'000);
    EranosTip *const tip = eranosTipCreate(tenPercent, ERANOS_TIP_DISTRIBUTION_EQUAL);
    eranosExpenseSetTip(expense, tip);
    checkThat(check, eranosExpenseId(expense, text) && isText(text, "0199a000-0000-7000-8000-000000000001"), "the identifier as set");
    eranosExpenseName(expense, text);
    checkThat(check, isText(text, "Lunch"), "the name");
    checkThat(check, eranosExpenseCategory(expense, text) && isText(text, "0199a000-0000-7000-8000-000000000002"), "the category");
    checkThat(check, eranosExpenseDescription(expense, text) && isText(text, "By the sea"), "the description");
    checkThat(check, eranosExpenseTime(expense, &time) && 12 == time.hour && 15 == time.minute, "the time");
    EranosCalendarDate const date = eranosExpenseDate(expense);
    checkThat(check, 24 == date.day && 9 == date.month && 2026 == date.year, "the date");
    EranosCost *const readCost = eranosExpenseCost(expense);
    EranosSplit *const readPayers = eranosExpensePayers(expense);
    EranosExpenseSplit *const readSplit = eranosExpenseSplit(expense);
    EranosTip *const readTip = eranosExpenseTip(expense);
    checkThat(check,
              isAmount(eranosCostAmount(readCost), "EUR", 4'000) && listsMembers(eranosSplitMembers(readPayers), "bob", nullptr) &&
                  ERANOS_EXPENSE_SPLIT_KIND_WHOLE == eranosExpenseSplitKind(readSplit) && nullptr != readTip,
              "the cost, payers, split, and tip");
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosExpenseTipAmount(expense, &amount), "the tip's amount");
    checkThat(check, isAmount(amount, "EUR", 400), "10 % of 40.00 EUR");
    checkSucceeded(check, eranosExpenseTotal(expense, &amount), "the total");
    checkThat(check, isAmount(amount, "EUR", 4'400), "the amount with its tip");
    EranosTextList *participants = nullptr;
    checkThat(check, isFailure(eranosExpenseParticipants(expense, &participants), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID),
              "an expense's members are identifiers");
    EranosMoney *const unlisted = amountOf("XYZ", 100);
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
    eranosExpenseClearId(expense);
    eranosExpenseClearCategory(expense);
    eranosExpenseClearDescription(expense);
    eranosExpenseClearTime(expense);
    eranosExpenseClearTip(expense);
    checkThat(check,
              !eranosExpenseId(expense, text) && !eranosExpenseCategory(expense, text) && !eranosExpenseDescription(expense, text) &&
                  !eranosExpenseTime(expense, &time) && nullptr == eranosExpenseTip(expense),
              "every optional field clears");
    EranosExpense *noExpense = nullptr;
    EranosCalendarDate const never = {.day = 30, .month = 2, .year = 2026};
    checkThat(check, isKind(eranosExpenseCreate(cost, never, "\xff", bob, split, &noExpense), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
    eranosCostDestroy(strange);
    eranosMoneyDestroy(unlisted);
    eranosTipDestroy(readTip);
    eranosExpenseSplitDestroy(readSplit);
    eranosSplitDestroy(readPayers);
    eranosCostDestroy(readCost);
    eranosTipDestroy(tip);
    eranosTipValueDestroy(tenPercent);
    eranosExpenseSplitDestroy(split);
    eranosSplitDestroy(bob);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(forty);
    eranosExpenseDestroy(expense);
    eranosTextDestroy(text);
}

/// A transfer whose fields did not read back as set would move money between the wrong members.
static void aTransferReadsBackEveryFieldItWasGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosTransfer *const transfer = transferOf(1'000, "bob", "ana");
    EranosTimeOfDay time = {.hour = 0, .minute = 0};
    checkThat(check, !eranosTransferId(transfer, text) && !eranosTransferNote(transfer, text) && !eranosTransferTime(transfer, &time),
              "a new transfer has no identifier, note, or time");
    checkSucceeded(check, eranosTransferSetId(transfer, "0199a000-0000-7000-8000-000000000003"), "an identifier is set");
    checkSucceeded(check, eranosTransferSetFrom(transfer, "ana"), "who paid");
    checkSucceeded(check, eranosTransferSetTo(transfer, "bob"), "who was paid");
    checkSucceeded(check, eranosTransferSetNote(transfer, "Cash"), "a note");
    EranosTimeOfDay const morning = {.hour = 9, .minute = 5};
    eranosTransferSetTime(transfer, morning);
    EranosCalendarDate const next = {.day = 24, .month = 9, .year = 2026};
    eranosTransferSetDate(transfer, next);
    EranosMoney *const five = euros(500);
    EranosCost *const cost = eranosCostCreateLocal(five);
    eranosTransferSetCost(transfer, cost);
    checkThat(check, eranosTransferId(transfer, text) && isText(text, "0199a000-0000-7000-8000-000000000003"), "the identifier");
    eranosTransferFrom(transfer, text);
    checkThat(check, isText(text, "ana"), "who paid");
    eranosTransferTo(transfer, text);
    checkThat(check, isText(text, "bob"), "who was paid");
    checkThat(check, eranosTransferNote(transfer, text) && isText(text, "Cash") && eranosTransferTime(transfer, &time) && 9 == time.hour,
              "the note and time");
    EranosCalendarDate const date = eranosTransferDate(transfer);
    EranosCost *const readCost = eranosTransferCost(transfer);
    checkThat(check, 24 == date.day && isAmount(eranosCostAmount(readCost), "EUR", 500), "the date and the cost");
    checkThat(check,
              isKind(eranosTransferSetId(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosTransferSetFrom(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosTransferSetTo(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosTransferSetNote(transfer, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "every text field is text");
    eranosTransferClearId(transfer);
    eranosTransferClearNote(transfer);
    eranosTransferClearTime(transfer);
    checkThat(check, !eranosTransferId(transfer, text) && !eranosTransferNote(transfer, text) && !eranosTransferTime(transfer, &time), "and clears");
    EranosTransfer *none = nullptr;
    checkThat(check, isKind(eranosTransferCreate(cost, DAY, "\xff", "ana", &none), ERANOS_ERROR_KIND_INVALID_TEXT) && nullptr == none,
              "who paid is text");
    eranosCostDestroy(readCost);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(five);
    eranosTransferDestroy(transfer);
    eranosTextDestroy(text);
}

/// A member or an account whose fields did not read back as set would send a member's money to another's account.
static void aMemberAndTheirAccountsReadBackAsTheyWereGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosAccountDetails *bank = nullptr;
    checkSucceeded(check, eranosAccountDetailsCreateIban("RO49AAAA1B31007593840000", &bank), "an account's IBAN");
    EranosAccountDetails *app = nullptr;
    checkSucceeded(check, eranosAccountDetailsCreateUsername("ana.pays", &app), "an account's username");
    checkThat(check,
              ERANOS_ACCOUNT_DETAILS_KIND_IBAN == eranosAccountDetailsKind(bank) && eranosAccountDetailsIban(bank, text) &&
                  isText(text, "RO49AAAA1B31007593840000") && !eranosAccountDetailsUsername(bank, text),
              "an IBAN account holds its IBAN");
    checkThat(check,
              ERANOS_ACCOUNT_DETAILS_KIND_USERNAME == eranosAccountDetailsKind(app) && eranosAccountDetailsUsername(app, text) &&
                  isText(text, "ana.pays") && !eranosAccountDetailsIban(app, text),
              "a username account holds its username");
    EranosAccountDetails *none = nullptr;
    checkThat(check,
              isKind(eranosAccountDetailsCreateIban("\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosAccountDetailsCreateUsername("\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT),
              "account details are text");
    EranosAccount *account = nullptr;
    checkSucceeded(check, eranosAccountCreate(bank, "Bank", &account), "an account");
    eranosAccountSetDetails(account, app);
    checkSucceeded(check, eranosAccountSetName(account, "App"), "a name is set");
    EranosAccountDetails *const details = eranosAccountDetails(account);
    eranosAccountName(account, text);
    checkThat(check, isText(text, "App") && ERANOS_ACCOUNT_DETAILS_KIND_USERNAME == eranosAccountDetailsKind(details), "the account as set");
    checkThat(check, isKind(eranosAccountSetName(account, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
    EranosAccount *noAccount = nullptr;
    checkThat(check, isKind(eranosAccountCreate(bank, "\xff", &noAccount), ERANOS_ERROR_KIND_INVALID_TEXT), "an account's name is text");
    EranosAccountList *const accounts = eranosAccountListCreate();
    checkThat(check, eranosAccountListIsEmpty(accounts), "an empty list of accounts");
    eranosAccountListPush(accounts, account);
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosMemberCreate("Carla", &member), "a member");
    EranosAccountList *const noAccounts = eranosMemberAccounts(member);
    checkThat(check,
              !eranosMemberId(member, text) && !eranosMemberEmail(member, text) && !eranosMemberLastName(member, text) &&
                  !eranosMemberPhone(member, text) && eranosAccountListIsEmpty(noAccounts),
              "a new member has a first name only");
    checkSucceeded(check, eranosMemberSetId(member, "0199a000-0000-7000-8000-000000000004"), "an identifier is set");
    checkSucceeded(check, eranosMemberSetFirstName(member, "Carla Maria"), "a first name");
    checkSucceeded(check, eranosMemberSetEmail(member, "carla@example.com"), "an email");
    checkSucceeded(check, eranosMemberSetLastName(member, "Pop"), "a last name");
    checkSucceeded(check, eranosMemberSetPhone(member, "+40712345678"), "a phone number");
    eranosMemberSetAccounts(member, accounts);
    EranosAccountList *const readAccounts = eranosMemberAccounts(member);
    EranosAccount *const firstAccount = eranosAccountListGet(readAccounts, 0);
    checkThat(check, 1 == eranosAccountListLength(readAccounts) && nullptr != firstAccount && nullptr == eranosAccountListGet(readAccounts, 1),
              "one account");
    eranosMemberFirstName(member, text);
    checkThat(check, isText(text, "Carla Maria"), "the first name");
    checkThat(check,
              eranosMemberId(member, text) && isText(text, "0199a000-0000-7000-8000-000000000004") && eranosMemberEmail(member, text) &&
                  isText(text, "carla@example.com") && eranosMemberLastName(member, text) && isText(text, "Pop") && eranosMemberPhone(member, text) &&
                  isText(text, "+40712345678"),
              "every detail as set");
    checkThat(check,
              isKind(eranosMemberSetId(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetFirstName(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetEmail(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetLastName(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosMemberSetPhone(member, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "every text field is text");
    eranosMemberClearId(member);
    eranosMemberClearEmail(member);
    eranosMemberClearLastName(member);
    eranosMemberClearPhone(member);
    checkThat(check,
              !eranosMemberId(member, text) && !eranosMemberEmail(member, text) && !eranosMemberLastName(member, text) &&
                  !eranosMemberPhone(member, text),
              "and clears");
    EranosMember *noMember = nullptr;
    checkThat(check, isKind(eranosMemberCreate("\xff", &noMember), ERANOS_ERROR_KIND_INVALID_TEXT), "a first name is text");
    eranosAccountDestroy(firstAccount);
    eranosAccountListDestroy(readAccounts);
    eranosAccountListDestroy(noAccounts);
    eranosMemberDestroy(member);
    eranosAccountListDestroy(accounts);
    eranosAccountDetailsDestroy(details);
    eranosAccountDestroy(account);
    eranosAccountDetailsDestroy(app);
    eranosAccountDetailsDestroy(bank);
    eranosTextDestroy(text);
}

/// A category whose fields did not read back as set would file expenses under the wrong name.
static void aCategoryReadsBackAsItWasGiven(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosCategory *category = nullptr;
    checkSucceeded(check, eranosCategoryCreate("Food", &category), "a category");
    checkThat(check, !eranosCategoryId(category, text), "a new category has no identifier");
    checkSucceeded(check, eranosCategorySetId(category, "0199a000-0000-7000-8000-000000000005"), "an identifier is set");
    checkSucceeded(check, eranosCategorySetName(category, "Meals"), "a name");
    eranosCategoryName(category, text);
    checkThat(check, isText(text, "Meals"), "the name as set");
    checkThat(check, eranosCategoryId(category, text) && isText(text, "0199a000-0000-7000-8000-000000000005"), "the identifier as set");
    checkThat(check,
              isKind(eranosCategorySetId(category, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosCategorySetName(category, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "its fields are text");
    eranosCategoryClearId(category);
    checkThat(check, !eranosCategoryId(category, text), "the identifier clears");
    EranosCategory *none = nullptr;
    checkThat(check, isKind(eranosCategoryCreate("\xff", &none), ERANOS_ERROR_KIND_INVALID_TEXT), "a name is text");
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
                  isAmount(eranosBalanceAmount(getsBack), "EUR", 1'200),
              "as what the member gets back");
    checkSucceeded(check, eranosBalanceFormat(owes, text), "a balance a member owes formats");
    checkThat(check, isText(text, "owes 12.00 EUR") && ERANOS_BALANCE_KIND_OWES == eranosBalanceKind(owes), "as what the member owes");
    checkSucceeded(check, eranosBalanceFormat(settled, text), "a settled balance formats");
    checkThat(check, isText(text, "settled") && ERANOS_BALANCE_KIND_SETTLED == eranosBalanceKind(settled) && nullptr == eranosBalanceAmount(settled),
              "with no amount");
    EranosMoney *const unlisted = amountOf("XYZ", 1);
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
    eranosItemDestroy(nullptr);
    eranosItemListDestroy(nullptr);
    eranosTipDestroy(nullptr);
    eranosTipValueDestroy(nullptr);
    eranosCancellationDestroy(nullptr);
    eranosCurrencyRateDestroy(nullptr);
    eranosCurrencyRateListDestroy(nullptr);
    eranosGroupDestroy(nullptr);
    eranosNewGroupDestroy(nullptr);
    eranosGroupReaderDestroy(nullptr);
    eranosTextListDestroy(nullptr);
    eranosAccountDestroy(nullptr);
    eranosAccountDetailsDestroy(nullptr);
    eranosAccountListDestroy(nullptr);
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
    eranosSettlementDestroy(nullptr);
    eranosSummaryDestroy(nullptr);
    eranosTransferDestroy(nullptr);
    eranosTransferListDestroy(nullptr);
    checkThat(check, true, "every destroy function returned");
}

// --- The group ---------------------------------------------------------------------------------------------------------

/// Details that did not reach the group, or a group that did not change as asked, would show an app a name the user replaced.
static void aGroupHoldsItsDetailsAndChangesThem(Check *const check) {
    EranosText *const text = eranosTextCreate();
    EranosNewGroup *details = nullptr;
    checkSucceeded(check, eranosNewGroupCreate("EUR", "Flat", &details), "a new group's details");
    checkThat(check, !eranosNewGroupDescription(details, text) && !eranosNewGroupLanguage(details, text), "start without description or language");
    checkSucceeded(check, eranosNewGroupSetCurrency(details, "RON"), "a currency is set");
    checkSucceeded(check, eranosNewGroupSetName(details, "Home"), "a name");
    checkSucceeded(check, eranosNewGroupSetDescription(details, "Bills"), "a description");
    checkSucceeded(check, eranosNewGroupSetLanguage(details, "ro-RO"), "a language");
    eranosNewGroupCurrency(details, text);
    checkThat(check, isText(text, "RON"), "the currency as set");
    eranosNewGroupName(details, text);
    checkThat(check, isText(text, "Home"), "the name");
    checkThat(check,
              eranosNewGroupDescription(details, text) && isText(text, "Bills") && eranosNewGroupLanguage(details, text) && isText(text, "ro-RO"),
              "the description and the language");
    checkThat(check,
              isKind(eranosNewGroupSetCurrency(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosNewGroupSetName(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosNewGroupSetDescription(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT) &&
                  isKind(eranosNewGroupSetLanguage(details, "\xff"), ERANOS_ERROR_KIND_INVALID_TEXT),
              "its fields are text");
    eranosNewGroupClearDescription(details);
    eranosNewGroupClearLanguage(details);
    checkThat(check, !eranosNewGroupDescription(details, text) && !eranosNewGroupLanguage(details, text), "and clear");
    EranosNewGroup *noDetails = nullptr;
    checkThat(check, isKind(eranosNewGroupCreate("\xff", "Flat", &noDetails), ERANOS_ERROR_KIND_INVALID_TEXT), "a currency's code is text");
    EranosGroup *group = nullptr;
    checkSucceeded(check, eranosGroupCreate(details, &group), "a group");
    checkSucceeded(check, eranosGroupId(group, text), "has an identifier");
    checkThat(check, eranosTextLength(text) == strlen("0199a000-0000-7000-8000-000000000001"), "a hyphenated UUID");
    checkSucceeded(check, eranosGroupCurrency(group, text), "a currency");
    checkThat(check, isText(text, "RON"), "the one it started with");
    checkSucceeded(check, eranosGroupSetName(group, "Trip"), "a name is set");
    checkSucceeded(check, eranosGroupName(group, text), "and read");
    checkThat(check, isText(text, "Trip"), "the name as set");
    checkSucceeded(check, eranosGroupSetDescription(group, "Summer"), "a description is set");
    checkSucceeded(check, eranosGroupDescription(group, text), "and read");
    checkThat(check, isText(text, "Summer"), "the description as set");
    checkSucceeded(check, eranosGroupSetDescription(group, ""), "blank text clears it");
    checkSucceeded(check, eranosGroupDescription(group, text), "and it reads");
    checkThat(check, isText(text, ""), "as nothing");
    checkSucceeded(check, eranosGroupSetLanguage(group, "pt-pt"), "a language is set");
    checkSucceeded(check, eranosGroupLanguage(group, text), "and read");
    checkThat(check, isText(text, "pt-PT"), "canonical");
    checkSucceeded(check, eranosGroupClearLanguage(group), "a language clears");
    checkSucceeded(check, eranosGroupLanguage(group, text), "and reads");
    checkThat(check, isText(text, ""), "as nothing");
    checkThat(check, isFailure(eranosGroupSetName(group, " "), ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_EMPTY_NAME), "a blank name");
    checkThat(check, isFailure(eranosGroupSetLanguage(group, "not a tag"), ERANOS_ERROR_KIND_LANGUAGE, ERANOS_FAILURE_LANGUAGE_INVALID_LANGUAGE),
              "no tag");
    eranosErrorDestroy(eranosNewGroupSetName(details, " "));
    EranosGroup *noGroup = nullptr;
    checkThat(check, isFailure(eranosGroupCreate(details, &noGroup), ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_EMPTY_NAME) && nullptr == noGroup,
              "a group needs a name");
    eranosGroupDestroy(group);
    eranosNewGroupDestroy(details);
    eranosTextDestroy(text);
}

/// Entries that did not come and go as asked would let an app show a deleted expense or lose an added one.
static void entriesComeAndGoThroughTheGroup(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosExpenseList *expenses = nullptr;
    checkSucceeded(check, eranosGroupExpenses(flat.group, &expenses), "the expenses");
    EranosExpense *const dinner = eranosExpenseListGet(expenses, 0);
    checkThat(check,
              1 == eranosExpenseListLength(expenses) && !eranosExpenseListIsEmpty(expenses) && nullptr != dinner &&
                  nullptr == eranosExpenseListGet(expenses, 1),
              "the dinner, alone");
    checkThat(check, eranosExpenseId(dinner, text) && isText(text, eranosTextData(flat.dinner)), "under its identifier");
    checkSucceeded(check, eranosExpenseSetName(dinner, "Supper"), "an edit");
    EranosExpense *previous = nullptr;
    checkSucceeded(check, eranosGroupReplaceExpense(flat.group, dinner, &previous), "replaces the expense");
    eranosExpenseName(previous, text);
    checkThat(check, isText(text, "Dinner"), "which comes back as it was");
    EranosExpense *expense = nullptr;
    checkSucceeded(check, eranosGroupExpense(flat.group, eranosTextData(flat.dinner), &expense), "the expense by its identifier");
    eranosExpenseName(expense, text);
    checkThat(check, isText(text, "Supper"), "as replaced");
    EranosExpenseShares *shares = nullptr;
    checkSucceeded(check, eranosGroupExpenseShares(flat.group, eranosTextData(flat.dinner), &shares), "its shares");
    checkThat(check, nullptr != shares, "which it has");
    eranosExpenseSharesDestroy(shares);
    EranosExpense *removed = nullptr;
    checkSucceeded(check, eranosGroupRemoveExpense(flat.group, eranosTextData(flat.dinner), &removed), "the expense goes");
    eranosExpenseName(removed, text);
    checkThat(check, isText(text, "Supper"), "and comes back as it was");
    EranosExpense *gone = nullptr;
    checkSucceeded(check, eranosGroupExpense(flat.group, eranosTextData(flat.dinner), &gone), "a look for it");
    checkThat(check, nullptr == gone, "finds none");
    EranosTransferList *transfers = nullptr;
    checkSucceeded(check, eranosGroupTransfers(flat.group, &transfers), "the transfers");
    EranosTransfer *const repayment = eranosTransferListGet(transfers, 0);
    checkThat(check,
              1 == eranosTransferListLength(transfers) && !eranosTransferListIsEmpty(transfers) && nullptr == eranosTransferListGet(transfers, 1),
              "the repayment, alone");
    checkSucceeded(check, eranosTransferSetNote(repayment, "Back"), "an edit");
    EranosTransfer *previousTransfer = nullptr;
    checkSucceeded(check, eranosGroupReplaceTransfer(flat.group, repayment, &previousTransfer), "replaces the transfer");
    checkThat(check, !eranosTransferNote(previousTransfer, text), "which comes back without its note");
    EranosTransfer *transfer = nullptr;
    checkSucceeded(check, eranosGroupTransfer(flat.group, eranosTextData(flat.repayment), &transfer), "the transfer by its identifier");
    checkThat(check, eranosTransferNote(transfer, text) && isText(text, "Back"), "as replaced");
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosGroupTransferAmount(flat.group, eranosTextData(flat.repayment), &amount), "its amount");
    checkThat(check, isAmount(amount, "EUR", 1'000), "in the group currency");
    EranosTransfer *removedTransfer = nullptr;
    checkSucceeded(check, eranosGroupRemoveTransfer(flat.group, eranosTextData(flat.repayment), &removedTransfer), "the transfer goes");
    checkThat(check, nullptr != removedTransfer, "and comes back");
    EranosCategoryList *categories = nullptr;
    checkSucceeded(check, eranosGroupCategories(flat.group, &categories), "the categories");
    EranosCategory *const builtIn = eranosCategoryListGet(categories, 0);
    checkThat(check,
              2 == eranosCategoryListLength(categories) && !eranosCategoryListIsEmpty(categories) && nullptr != builtIn &&
                  nullptr == eranosCategoryListGet(categories, 2),
              "the built-in category and the food");
    EranosCategory *food = nullptr;
    checkSucceeded(check, eranosGroupCategory(flat.group, eranosTextData(flat.food), &food), "the food by its identifier");
    checkSucceeded(check, eranosCategorySetName(food, "Meals"), "an edit");
    EranosCategory *previousCategory = nullptr;
    checkSucceeded(check, eranosGroupReplaceCategory(flat.group, food, &previousCategory), "replaces the category");
    eranosCategoryName(previousCategory, text);
    checkThat(check, isText(text, "Food"), "which comes back as it was");
    EranosCategory *removedCategory = nullptr;
    checkSucceeded(check, eranosGroupRemoveCategory(flat.group, eranosTextData(flat.food), &removedCategory), "the category goes");
    eranosCategoryName(removedCategory, text);
    checkThat(check, isText(text, "Meals"), "and comes back as it was");
    EranosMemberList *members = nullptr;
    checkSucceeded(check, eranosGroupMembers(flat.group, &members), "the members");
    EranosMember *const ana = eranosMemberListGet(members, 0);
    checkThat(check, 2 == eranosMemberListLength(members) && !eranosMemberListIsEmpty(members) && nullptr == eranosMemberListGet(members, 2),
              "Ana and Bob");
    checkSucceeded(check, eranosMemberSetLastName(ana, "Pop"), "an edit");
    EranosMember *previousMember = nullptr;
    checkSucceeded(check, eranosGroupReplaceMember(flat.group, ana, &previousMember), "replaces the member");
    checkThat(check, !eranosMemberLastName(previousMember, text), "who comes back as they were");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosGroupMember(flat.group, eranosTextData(flat.ana), &member), "the member by their identifier");
    checkThat(check, eranosMemberLastName(member, text) && isText(text, "Pop"), "as replaced");
    bool archived = true;
    checkSucceeded(check, eranosGroupArchiveMember(flat.group, eranosTextData(flat.bob)), "Bob is archived");
    checkSucceeded(check, eranosGroupIsArchived(flat.group, eranosTextData(flat.bob), &archived), "and asked about");
    checkThat(check, archived, "and is");
    checkSucceeded(check, eranosGroupUnarchiveMember(flat.group, eranosTextData(flat.bob)), "Bob is back");
    checkSucceeded(check, eranosGroupIsArchived(flat.group, eranosTextData(flat.bob), &archived), "and asked about");
    checkThat(check, !archived, "and is not");
    EranosMember *removedMember = nullptr;
    checkSucceeded(check, eranosGroupRemoveMember(flat.group, eranosTextData(flat.bob), &removedMember), "Bob goes, once nothing names him");
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

/// An identifier that is not one would reach the library as no entry's, or as another's; each of the group's lookups refuses it
/// and gives no object.
static void anIdentifierThatIsNotOneIsRefusedByEveryLookup(Check *const check) {
    Flat const flat = flatOf();
    char const *const nobody = "not an identifier";
    EranosExpense *expense = nullptr;
    checkThat(check,
              isFailure(eranosGroupExpense(flat.group, nobody, &expense), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID) && nullptr == expense,
              "an expense");
    EranosExpenseShares *shares = nullptr;
    checkThat(check,
              isFailure(eranosGroupExpenseShares(flat.group, nobody, &shares), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID) &&
                  nullptr == shares,
              "an expense's shares");
    EranosMember *member = nullptr;
    checkThat(check,
              isFailure(eranosGroupMember(flat.group, nobody, &member), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID) && nullptr == member,
              "a member");
    EranosTransfer *transfer = nullptr;
    checkThat(check,
              isFailure(eranosGroupTransfer(flat.group, nobody, &transfer), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID) &&
                  nullptr == transfer,
              "a transfer");
    EranosMoney *amount = nullptr;
    checkThat(check,
              isFailure(eranosGroupTransferAmount(flat.group, nobody, &amount), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID) &&
                  nullptr == amount,
              "a transfer's amount");
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
    checkSucceeded(check, eranosCategoryCreate("Travel", &category), "a category");
    checkSucceeded(check, eranosCategorySetId(category, unknown), "its identifier");
    EranosCategory *previousCategory = nullptr;
    checkThat(check,
              isFailure(eranosGroupReplaceCategory(flat.group, category, &previousCategory), ERANOS_ERROR_KIND_GROUP,
                        ERANOS_FAILURE_GROUP_UNKNOWN_CATEGORY) &&
                  nullptr == previousCategory,
              "a category is not replaced");
    checkThat(check,
              isFailure(eranosGroupRemoveCategory(flat.group, unknown, &previousCategory), ERANOS_ERROR_KIND_GROUP,
                        ERANOS_FAILURE_GROUP_UNKNOWN_CATEGORY) &&
                  nullptr == previousCategory,
              "nor removed");
    EranosExpense *const expense = expenseOf("Taxi", ana, ana, bob);
    checkSucceeded(check, eranosExpenseSetId(expense, unknown), "an expense's identifier");
    EranosExpense *previousExpense = nullptr;
    checkThat(
        check,
        isFailure(eranosGroupReplaceExpense(flat.group, expense, &previousExpense), ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_EXPENSE) &&
            nullptr == previousExpense,
        "an expense is not replaced");
    checkThat(
        check,
        isFailure(eranosGroupRemoveExpense(flat.group, unknown, &previousExpense), ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_EXPENSE) &&
            nullptr == previousExpense,
        "nor removed");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosMemberCreate("Carol", &member), "a member");
    checkSucceeded(check, eranosMemberSetId(member, unknown), "their identifier");
    EranosMember *previousMember = nullptr;
    checkThat(
        check,
        isFailure(eranosGroupReplaceMember(flat.group, member, &previousMember), ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_MEMBER) &&
            nullptr == previousMember,
        "a member is not replaced");
    checkThat(
        check,
        isFailure(eranosGroupRemoveMember(flat.group, unknown, &previousMember), ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_MEMBER) &&
            nullptr == previousMember,
        "nor removed");
    EranosTransfer *const transfer = transferOf(500, ana, bob);
    checkSucceeded(check, eranosTransferSetId(transfer, unknown), "a transfer's identifier");
    EranosTransfer *previousTransfer = nullptr;
    checkThat(check,
              isFailure(eranosGroupReplaceTransfer(flat.group, transfer, &previousTransfer), ERANOS_ERROR_KIND_GROUP,
                        ERANOS_FAILURE_GROUP_UNKNOWN_TRANSFER) &&
                  nullptr == previousTransfer,
              "a transfer is not replaced");
    checkThat(check,
              isFailure(eranosGroupRemoveTransfer(flat.group, unknown, &previousTransfer), ERANOS_ERROR_KIND_GROUP,
                        ERANOS_FAILURE_GROUP_UNKNOWN_TRANSFER) &&
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
    checkSucceeded(check, eranosGroupSummary(flat.group, &summary), "the summary");
    checkThat(check, isAmount(eranosSummaryTotalSpent(summary), "EUR", 3'000), "the dinner is all that was spent");
    EranosCategoryTotalList *const categoryTotals = eranosSummaryCategoryTotals(summary);
    EranosCategoryTotal *const food = eranosCategoryTotalListGet(categoryTotals, 0);
    eranosCategoryTotalCategory(food, text);
    checkThat(check,
              1 == eranosCategoryTotalListLength(categoryTotals) && !eranosCategoryTotalListIsEmpty(categoryTotals) &&
                  nullptr == eranosCategoryTotalListGet(categoryTotals, 1) && isText(text, eranosTextData(flat.food)) &&
                  isAmount(eranosCategoryTotalTotal(food), "EUR", 3'000),
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
              isAmount(eranosMemberSummaryPaid(ana), "EUR", 3'000) && isAmount(eranosMemberSummaryShare(ana), "EUR", 1'500) &&
                  isAmount(eranosMemberSummaryReceived(ana), "EUR", 1'000) && isAmount(eranosMemberSummarySent(ana), "EUR", 0) &&
                  isAmount(eranosMemberSummaryNet(ana), "EUR", 500),
              "Ana paid 30.00, consumed 15.00, and got 10.00 back, so she is owed 5.00");
    EranosExpenseShares *shares = nullptr;
    checkSucceeded(check, eranosGroupExpenseShares(flat.group, eranosTextData(flat.dinner), &shares), "the dinner's shares");
    EranosMemberShareList *const memberShares = eranosExpenseSharesMembers(shares);
    EranosMemberShare *const anaShare = eranosMemberShareListGet(memberShares, 0);
    eranosMemberShareMember(anaShare, text);
    checkThat(check,
              isAmount(eranosExpenseSharesTotal(shares), "EUR", 3'000) && isAmount(eranosExpenseSharesTip(shares), "EUR", 0) &&
                  2 == eranosMemberShareListLength(memberShares) && !eranosMemberShareListIsEmpty(memberShares) &&
                  nullptr == eranosMemberShareListGet(memberShares, 2),
              "30.00 EUR, no tip, two shares");
    checkThat(check,
              isText(text, eranosTextData(flat.ana)) && isAmount(eranosMemberSharePaid(anaShare), "EUR", 3'000) &&
                  isAmount(eranosMemberShareShare(anaShare), "EUR", 1'500),
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
    EranosSettlementStrategy const strategies[] = {
        ERANOS_SETTLEMENT_STRATEGY_DIRECT,
        ERANOS_SETTLEMENT_STRATEGY_GREEDY,
        ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS,
    };
    for (size_t index = 0; index < sizeof strategies / sizeof strategies[0]; ++index) {
        EranosSettlement *settlement = nullptr;
        checkSucceeded(check, eranosGroupSettlement(flat.group, strategies[index], &settlement), "a settlement");
        EranosPaymentList *const payments = eranosSettlementPayments(settlement);
        EranosPayment *const payment = eranosPaymentListGet(payments, 0);
        checkThat(check,
                  eranosSettlementStrategy(settlement) == strategies[index] && 1 == eranosPaymentListLength(payments) &&
                      !eranosPaymentListIsEmpty(payments) && nullptr == eranosPaymentListGet(payments, 1),
                  "one payment, by the strategy asked for");
        eranosPaymentFrom(payment, text);
        checkThat(check, isText(text, eranosTextData(flat.bob)), "from Bob");
        eranosPaymentTo(payment, text);
        checkThat(check, isText(text, eranosTextData(flat.ana)) && isAmount(eranosPaymentAmount(payment), "EUR", 500), "to Ana, of 5.00 EUR");
        eranosPaymentDestroy(payment);
        eranosPaymentListDestroy(payments);
        eranosSettlementDestroy(settlement);
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
    EranosNewGroup *details = nullptr;
    checkSucceeded(check, eranosNewGroupCreate("EUR", "Street party", &details), "the group's details");
    EranosGroup *group = nullptr;
    checkSucceeded(check, eranosGroupCreate(details, &group), "the group");
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
    checkSucceeded(check, eranosExpenseCreate(cost, DAY, "Food", payers, split, &expense), "the food, which the first neighbour paid");
    EranosText *const id = eranosTextCreate();
    checkSucceeded(check, eranosGroupAddExpense(group, expense, id), "shared by every neighbour");
    EranosSettlement *settlement = nullptr;
    EranosError *const refused = eranosGroupSettlement(group, ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS, &settlement);
    int64_t balances = 0;
    checkThat(check,
              holdsFailure(refused, ERANOS_ERROR_KIND_SETTLEMENT, ERANOS_FAILURE_SETTLEMENT_OUT_OF_MEMORY) &&
                  eranosErrorBalances(refused, &balances) && NEIGHBOURS == balances && nullptr == settlement,
              "the exact plan is refused, naming its balances");
    eranosErrorDestroy(refused);
    checkSucceeded(check, eranosGroupSettlement(group, ERANOS_SETTLEMENT_STRATEGY_GREEDY, &settlement), "a greedy plan");
    EranosPaymentList *const payments = nullptr == settlement ? nullptr : eranosSettlementPayments(settlement);
    checkThat(check, nullptr != payments && NEIGHBOURS - 1 == eranosPaymentListLength(payments), "one payment from each neighbour who owes");
    eranosPaymentListDestroy(payments);
    eranosSettlementDestroy(settlement);
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
    eranosGroupDestroy(group);
    eranosNewGroupDestroy(details);
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
    EranosError *const missing = eranosGroupChangeCurrency(flat.group, "RON", empty);
    checkThat(check,
              holdsFailure(missing, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_MISSING_EXCHANGE_RATE) && eranosErrorCurrency(missing, text) &&
                  isText(text, "EUR"),
              "the euro needs a rate");
    eranosErrorDestroy(missing);
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    checkThat(check, eranosCurrencyRateListIsEmpty(rates), "an empty list of rates");
    eranosCurrencyRateListPush(rates, rate);
    EranosCurrencyRate *const first = eranosCurrencyRateListGet(rates, 0);
    checkThat(check, 1 == eranosCurrencyRateListLength(rates) && nullptr != first && nullptr == eranosCurrencyRateListGet(rates, 1), "one rate");
    checkSucceeded(check, eranosGroupChangeCurrency(flat.group, "RON", rates), "the change");
    checkSucceeded(check, eranosGroupCurrency(flat.group, text), "the currency");
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
    checkSucceeded(check, eranosGroupSave(flat.group, json), "the group saves");
    EranosGroup *loaded = nullptr;
    checkSucceeded(check, eranosGroupLoad(eranosTextData(json), &loaded), "and loads");
    checkSucceeded(check, eranosGroupSave(loaded, again), "and saves again");
    checkThat(check, isText(again, eranosTextData(json)), "to the same text");
    EranosGroup *broken = nullptr;
    EranosError *const error = eranosGroupLoad("{", &broken);
    int64_t line = 0;
    int64_t column = 0;
    checkThat(check, isKind(eranosGroupLoad("{", &broken), ERANOS_ERROR_KIND_INVALID_JSON) && nullptr == broken, "text that is not a saved group");
    checkThat(check, eranosErrorLine(error, &line) && 1 == line && eranosErrorColumn(error, &column) && 1 == column, "where it stops being one");
    eranosErrorDestroy(error);
    eranosGroupDestroy(loaded);
    eranosTextDestroy(again);
    eranosTextDestroy(json);
    flatDestroy(flat);
}

/// An error that lost what it holds would leave an app unable to say which entry, amount, or category the library refused.
static void everyErrorHoldsWhatItsFailureHolds(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosMember *removed = nullptr;
    EranosError *const inUse = eranosGroupRemoveMember(flat.group, eranosTextData(flat.ana), &removed);
    checkThat(check, holdsFailure(inUse, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_MEMBER_IN_USE) && nullptr == removed, "Ana paid and was paid");
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
    eranosErrorDestroy(eranosMemberAmountCreate(ten, eranosTextData(flat.ana), &amount));
    EranosMemberAmountList *const amounts = eranosMemberAmountListCreate();
    eranosMemberAmountListPush(amounts, amount);
    EranosSplit *const uneven = eranosSplitCreateAmounts(amounts);
    EranosExpenseSplit *const split = eranosExpenseSplitCreateWhole(uneven);
    EranosExpense *expense = nullptr;
    eranosErrorDestroy(eranosExpenseCreate(cost, DAY, "Uneven", payers, split, &expense));
    EranosError *const unbalanced = eranosGroupAddExpense(flat.group, expense, text);
    checkThat(check,
              holdsFailure(unbalanced, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_AMOUNTS_DO_NOT_ADD_UP) &&
                  isAmount(eranosErrorActual(unbalanced), "EUR", 1'000) && isAmount(eranosErrorExpectedAmount(unbalanced), "EUR", 3'000),
              "the amounts and what they should add up to");
    eranosErrorDestroy(unbalanced);
    EranosMoney *const lei = amountOf("RON", 3'000);
    eranosMemberAmountSetAmount(amount, lei);
    EranosMemberAmountList *const foreignAmounts = eranosMemberAmountListCreate();
    eranosMemberAmountListPush(foreignAmounts, amount);
    EranosSplit *const foreign = eranosSplitCreateAmounts(foreignAmounts);
    EranosExpenseSplit *const foreignSplit = eranosExpenseSplitCreateWhole(foreign);
    eranosExpenseSetSplit(expense, foreignSplit);
    EranosError *const currencies = eranosGroupAddExpense(flat.group, expense, text);
    checkThat(check,
              holdsFailure(currencies, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_CURRENCY_MISMATCH) &&
                  eranosErrorExpectedCurrency(currencies, text) && isText(text, "EUR") && eranosErrorFoundCurrency(currencies, text) &&
                  isText(text, "RON"),
              "the expense's currency and the share's");
    eranosErrorDestroy(currencies);
    EranosMemberPercentage *half = nullptr;
    eranosErrorDestroy(eranosMemberPercentageCreate(5'000, eranosTextData(flat.ana), &half));
    EranosMemberPercentageList *const halves = eranosMemberPercentageListCreate();
    eranosMemberPercentageListPush(halves, half);
    EranosSplit *const byHalves = eranosSplitCreatePercentages(halves);
    EranosExpenseSplit *const halvesSplit = eranosExpenseSplitCreateWhole(byHalves);
    eranosExpenseSetSplit(expense, halvesSplit);
    EranosError *const percentages = eranosGroupAddExpense(flat.group, expense, text);
    checkThat(check,
              holdsFailure(percentages, ERANOS_ERROR_KIND_EXPENSE, ERANOS_FAILURE_EXPENSE_PERCENTAGES_DO_NOT_ADD_UP) &&
                  eranosErrorTotalBasisPoints(percentages, &number) && 5'000 == number,
              "what the percentages add up to");
    eranosErrorDestroy(percentages);
    EranosCategory *duplicate = nullptr;
    eranosErrorDestroy(eranosCategoryCreate("food", &duplicate));
    EranosError *const name = eranosGroupAddCategory(flat.group, duplicate, text);
    checkThat(check,
              holdsFailure(name, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_DUPLICATE_CATEGORY_NAME) && eranosErrorExisting(name, text) &&
                  isText(text, eranosTextData(flat.food)),
              "the category that has the name");
    eranosErrorDestroy(name);
    EranosExpense *const taxi = expenseOf("Taxi", eranosTextData(flat.ana), eranosTextData(flat.ana), nullptr);
    eranosErrorDestroy(eranosExpenseSetCategory(taxi, "0199a000-0000-7000-8000-00000000ffff"));
    EranosError *const category = eranosGroupAddExpense(flat.group, taxi, text);
    checkThat(check,
              holdsFailure(category, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_CATEGORY) && eranosErrorCategory(category, text) &&
                  isText(text, "0199a000-0000-7000-8000-00000000ffff"),
              "the unknown category");
    eranosErrorDestroy(category);
    EranosExpense *noExpense = nullptr;
    EranosError *const unknownExpense = eranosGroupRemoveExpense(flat.group, "0199a000-0000-7000-8000-00000000ffff", &noExpense);
    checkThat(check,
              holdsFailure(unknownExpense, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_EXPENSE) && eranosErrorExpense(unknownExpense, text),
              "the unknown expense");
    eranosErrorDestroy(unknownExpense);
    EranosTransfer *noTransfer = nullptr;
    EranosError *const unknownTransfer = eranosGroupRemoveTransfer(flat.group, "0199a000-0000-7000-8000-00000000ffff", &noTransfer);
    checkThat(check,
              holdsFailure(unknownTransfer, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_TRANSFER) &&
                  eranosErrorTransfer(unknownTransfer, text),
              "the unknown transfer");
    eranosErrorDestroy(unknownTransfer);
    EranosText *const json = eranosTextCreate();
    eranosErrorDestroy(eranosGroupSave(flat.group, json));
    char tampered[8192] = {};
    char const *const format = strstr(eranosTextData(json), "\"format\": 2");
    int const before = nullptr == format ? 0 : (int)(format - eranosTextData(json)) + (int)strlen("\"format\": 2");
    (void)snprintf(tampered, sizeof tampered, "%.*s6%s", before, eranosTextData(json), eranosTextData(json) + before);
    EranosGroup *newer = nullptr;
    EranosError *const unsupported = eranosGroupLoad(tampered, &newer);
    checkThat(check,
              holdsFailure(unsupported, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNSUPPORTED_FORMAT) &&
                  eranosErrorFormat(unsupported, &number) && 26 == number,
              "a format this release cannot read");
    eranosErrorDestroy(unsupported);
    eranosTextDestroy(json);
    EranosCategory *noCategory = nullptr;
    checkThat(check, isFailure(eranosGroupCategory(flat.group, "not an identifier", &noCategory), ERANOS_ERROR_KIND_ID, ERANOS_FAILURE_ID_INVALID_ID),
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
    eranosGroupIdAsync(flat.group, nullptr, &textCallback);
    await(&waiter);
    checkSucceeded(check, eranosGroupId(flat.group, text), "the identifier");
    checkThat(check, nullptr == waiter.error && isText(waiter.text, eranosTextData(text)), "the same identifier");
    eranosTextDestroy(waiter.text);
    EranosTextCallback const nameCallback = {waiting(&waiter), receiveText};
    eranosGroupNameAsync(flat.group, nullptr, &nameCallback);
    await(&waiter);
    checkThat(check, isText(waiter.text, "Flat"), "the name");
    eranosTextDestroy(waiter.text);
    EranosTextCallback const currencyCallback = {waiting(&waiter), receiveText};
    eranosGroupCurrencyAsync(flat.group, nullptr, &currencyCallback);
    await(&waiter);
    checkThat(check, isText(waiter.text, "EUR"), "the currency");
    eranosTextDestroy(waiter.text);
    EranosOptionalTextCallback const descriptionCallback = {waiting(&waiter), receiveText};
    eranosGroupDescriptionAsync(flat.group, nullptr, &descriptionCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr != waiter.text && isText(waiter.text, "Rent"), "the description");
    eranosTextDestroy(waiter.text);
    EranosOptionalTextCallback const languageCallback = {waiting(&waiter), receiveText};
    eranosGroupLanguageAsync(flat.group, nullptr, &languageCallback);
    await(&waiter);
    checkThat(check, isText(waiter.text, "pt-PT"), "the language");
    eranosTextDestroy(waiter.text);
    EranosTextCallback const saveCallback = {waiting(&waiter), receiveText};
    eranosGroupSaveAsync(flat.group, nullptr, &saveCallback);
    await(&waiter);
    checkSucceeded(check, eranosGroupSave(flat.group, text), "the saved group");
    checkThat(check, isText(waiter.text, eranosTextData(text)), "the same saved group");
    EranosText *const json = waiter.text;
    EranosGroupCallback const loadCallback = {waiting(&waiter), receiveGroup};
    eranosGroupLoadAsync(eranosTextData(json), nullptr, &loadCallback);
    await(&waiter);
    eranosTextDestroy(json);
    checkThat(check, nullptr == waiter.error && nullptr != waiter.object, "a group loaded on a thread of its own");
    eranosGroupDestroy(waiter.object);
    EranosCategoryListCallback const categoriesCallback = {waiting(&waiter), receiveCategories};
    eranosGroupCategoriesAsync(flat.group, nullptr, &categoriesCallback);
    await(&waiter);
    checkThat(check, 2 == eranosCategoryListLength(waiter.object), "the categories");
    eranosCategoryListDestroy(waiter.object);
    EranosCategoryCallback const categoryCallback = {waiting(&waiter), receiveCategory};
    eranosGroupCategoryAsync(flat.group, eranosTextData(flat.food), nullptr, &categoryCallback);
    await(&waiter);
    checkThat(check, nullptr != waiter.object, "a category");
    EranosCategory *const food = waiter.object;
    checkSucceeded(check, eranosCategorySetName(food, "Meals"), "an edit");
    EranosCategoryCallback const replaceCategoryCallback = {waiting(&waiter), receiveCategory};
    eranosGroupReplaceCategoryAsync(flat.group, food, nullptr, &replaceCategoryCallback);
    await(&waiter);
    eranosCategoryName(waiter.object, text);
    checkThat(check, isText(text, "Food"), "a replaced category as it was");
    eranosCategoryDestroy(waiter.object);
    eranosCategoryDestroy(food);
    EranosCategory *drinks = nullptr;
    eranosErrorDestroy(eranosCategoryCreate("Drinks", &drinks));
    EranosTextCallback const addCategoryCallback = {waiting(&waiter), receiveText};
    eranosGroupAddCategoryAsync(flat.group, drinks, nullptr, &addCategoryCallback);
    await(&waiter);
    eranosCategoryDestroy(drinks);
    EranosText *const addedCategory = waiter.text;
    EranosCategoryCallback const removeCategoryCallback = {waiting(&waiter), receiveCategory};
    eranosGroupRemoveCategoryAsync(flat.group, eranosTextData(addedCategory), nullptr, &removeCategoryCallback);
    await(&waiter);
    eranosTextDestroy(addedCategory);
    eranosCategoryName(waiter.object, text);
    checkThat(check, isText(text, "Drinks"), "an added category, removed");
    eranosCategoryDestroy(waiter.object);
    EranosExpenseListCallback const expensesCallback = {waiting(&waiter), receiveExpenses};
    eranosGroupExpensesAsync(flat.group, nullptr, &expensesCallback);
    await(&waiter);
    checkThat(check, 1 == eranosExpenseListLength(waiter.object), "the expenses");
    eranosExpenseListDestroy(waiter.object);
    EranosExpenseCallback const expenseCallback = {waiting(&waiter), receiveExpense};
    eranosGroupExpenseAsync(flat.group, eranosTextData(flat.dinner), nullptr, &expenseCallback);
    await(&waiter);
    EranosExpense *const dinner = waiter.object;
    checkThat(check, nullptr != dinner, "an expense");
    EranosExpenseSharesCallback const sharesCallback = {waiting(&waiter), receiveExpenseShares};
    eranosGroupExpenseSharesAsync(flat.group, eranosTextData(flat.dinner), nullptr, &sharesCallback);
    await(&waiter);
    checkThat(check, nullptr != waiter.object, "an expense's shares");
    eranosExpenseSharesDestroy(waiter.object);
    checkSucceeded(check, eranosExpenseSetName(dinner, "Supper"), "an edit");
    EranosExpenseCallback const replaceExpenseCallback = {waiting(&waiter), receiveExpense};
    eranosGroupReplaceExpenseAsync(flat.group, dinner, nullptr, &replaceExpenseCallback);
    await(&waiter);
    eranosExpenseName(waiter.object, text);
    checkThat(check, isText(text, "Dinner"), "a replaced expense as it was");
    eranosExpenseDestroy(waiter.object);
    eranosExpenseDestroy(dinner);
    EranosExpense *const lunch = expenseOf("Lunch", eranosTextData(flat.bob), eranosTextData(flat.ana), nullptr);
    EranosTextCallback const addExpenseCallback = {waiting(&waiter), receiveText};
    eranosGroupAddExpenseAsync(flat.group, lunch, nullptr, &addExpenseCallback);
    await(&waiter);
    eranosExpenseDestroy(lunch);
    EranosText *const addedExpense = waiter.text;
    EranosExpenseCallback const removeExpenseCallback = {waiting(&waiter), receiveExpense};
    eranosGroupRemoveExpenseAsync(flat.group, eranosTextData(addedExpense), nullptr, &removeExpenseCallback);
    await(&waiter);
    eranosTextDestroy(addedExpense);
    eranosExpenseName(waiter.object, text);
    checkThat(check, isText(text, "Lunch"), "an added expense, removed");
    eranosExpenseDestroy(waiter.object);
    EranosSummaryCallback const summaryCallback = {waiting(&waiter), receiveSummary};
    eranosGroupSummaryAsync(flat.group, nullptr, &summaryCallback);
    await(&waiter);
    checkThat(check, isAmount(eranosSummaryTotalSpent(waiter.object), "EUR", 3'000), "the summary");
    eranosSummaryDestroy(waiter.object);
    EranosSettlementCallback const settlementCallback = {waiting(&waiter), receiveSettlement};
    eranosGroupSettlementAsync(flat.group, ERANOS_SETTLEMENT_STRATEGY_GREEDY, nullptr, &settlementCallback);
    await(&waiter);
    checkThat(check, ERANOS_SETTLEMENT_STRATEGY_GREEDY == eranosSettlementStrategy(waiter.object), "a settlement");
    eranosSettlementDestroy(waiter.object);
    EranosMemberListCallback const membersCallback = {waiting(&waiter), receiveMembers};
    eranosGroupMembersAsync(flat.group, nullptr, &membersCallback);
    await(&waiter);
    checkThat(check, 2 == eranosMemberListLength(waiter.object), "the members");
    eranosMemberListDestroy(waiter.object);
    EranosMemberCallback const memberCallback = {waiting(&waiter), receiveMember};
    eranosGroupMemberAsync(flat.group, eranosTextData(flat.ana), nullptr, &memberCallback);
    await(&waiter);
    EranosMember *const ana = waiter.object;
    checkThat(check, nullptr != ana, "a member");
    checkSucceeded(check, eranosMemberSetLastName(ana, "Pop"), "an edit");
    EranosMemberCallback const replaceMemberCallback = {waiting(&waiter), receiveMember};
    eranosGroupReplaceMemberAsync(flat.group, ana, nullptr, &replaceMemberCallback);
    await(&waiter);
    checkThat(check, !eranosMemberLastName(waiter.object, text), "a replaced member as they were");
    eranosMemberDestroy(waiter.object);
    eranosMemberDestroy(ana);
    EranosMember *carla = nullptr;
    eranosErrorDestroy(eranosMemberCreate("Carla", &carla));
    EranosTextCallback const addMemberCallback = {waiting(&waiter), receiveText};
    eranosGroupAddMemberAsync(flat.group, carla, nullptr, &addMemberCallback);
    await(&waiter);
    eranosMemberDestroy(carla);
    EranosText *const carlaId = waiter.text;
    EranosUnitCallback const archiveCallback = {waiting(&waiter), receiveUnit};
    eranosGroupArchiveMemberAsync(flat.group, eranosTextData(carlaId), nullptr, &archiveCallback);
    await(&waiter);
    EranosFlagCallback const archivedCallback = {waiting(&waiter), receiveFlag};
    eranosGroupIsArchivedAsync(flat.group, eranosTextData(carlaId), nullptr, &archivedCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && waiter.flag, "an archived member is archived");
    EranosUnitCallback const unarchiveCallback = {waiting(&waiter), receiveUnit};
    eranosGroupUnarchiveMemberAsync(flat.group, eranosTextData(carlaId), nullptr, &unarchiveCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error, "and comes back");
    EranosMemberCallback const removeMemberCallback = {waiting(&waiter), receiveMember};
    eranosGroupRemoveMemberAsync(flat.group, eranosTextData(carlaId), nullptr, &removeMemberCallback);
    await(&waiter);
    eranosMemberFirstName(waiter.object, text);
    checkThat(check, isText(text, "Carla"), "an added member, removed");
    eranosMemberDestroy(waiter.object);
    eranosTextDestroy(carlaId);
    EranosTransferListCallback const transfersCallback = {waiting(&waiter), receiveTransfers};
    eranosGroupTransfersAsync(flat.group, nullptr, &transfersCallback);
    await(&waiter);
    checkThat(check, 1 == eranosTransferListLength(waiter.object), "the transfers");
    eranosTransferListDestroy(waiter.object);
    EranosTransferCallback const transferCallback = {waiting(&waiter), receiveTransfer};
    eranosGroupTransferAsync(flat.group, eranosTextData(flat.repayment), nullptr, &transferCallback);
    await(&waiter);
    EranosTransfer *const repayment = waiter.object;
    EranosMoneyCallback const amountCallback = {waiting(&waiter), receiveMoney};
    eranosGroupTransferAmountAsync(flat.group, eranosTextData(flat.repayment), nullptr, &amountCallback);
    await(&waiter);
    checkThat(check, isAmount(waiter.object, "EUR", 1'000), "a transfer's amount");
    checkSucceeded(check, eranosTransferSetNote(repayment, "Back"), "an edit");
    EranosTransferCallback const replaceTransferCallback = {waiting(&waiter), receiveTransfer};
    eranosGroupReplaceTransferAsync(flat.group, repayment, nullptr, &replaceTransferCallback);
    await(&waiter);
    checkThat(check, !eranosTransferNote(waiter.object, text), "a replaced transfer as it was");
    eranosTransferDestroy(waiter.object);
    eranosTransferDestroy(repayment);
    EranosTransfer *const cash = transferOf(100, eranosTextData(flat.ana), eranosTextData(flat.bob));
    EranosTextCallback const addTransferCallback = {waiting(&waiter), receiveText};
    eranosGroupAddTransferAsync(flat.group, cash, nullptr, &addTransferCallback);
    await(&waiter);
    eranosTransferDestroy(cash);
    EranosText *const addedTransfer = waiter.text;
    EranosTransferCallback const removeTransferCallback = {waiting(&waiter), receiveTransfer};
    eranosGroupRemoveTransferAsync(flat.group, eranosTextData(addedTransfer), nullptr, &removeTransferCallback);
    await(&waiter);
    eranosTextDestroy(addedTransfer);
    checkThat(check, nullptr != waiter.object, "an added transfer, removed");
    eranosTransferDestroy(waiter.object);
    EranosUnitCallback const setNameCallback = {waiting(&waiter), receiveUnit};
    eranosGroupSetNameAsync(flat.group, "Home", nullptr, &setNameCallback);
    await(&waiter);
    EranosUnitCallback const setDescriptionCallback = {waiting(&waiter), receiveUnit};
    eranosGroupSetDescriptionAsync(flat.group, "", nullptr, &setDescriptionCallback);
    await(&waiter);
    EranosOptionalTextCallback const clearedCallback = {waiting(&waiter), receiveText};
    eranosGroupDescriptionAsync(flat.group, nullptr, &clearedCallback);
    await(&waiter);
    checkThat(check, nullptr == waiter.error && nullptr == waiter.text, "a cleared description is none");
    EranosUnitCallback const setLanguageCallback = {waiting(&waiter), receiveUnit};
    eranosGroupSetLanguageAsync(flat.group, "ro-RO", nullptr, &setLanguageCallback);
    await(&waiter);
    EranosUnitCallback const clearLanguageCallback = {waiting(&waiter), receiveUnit};
    eranosGroupClearLanguageAsync(flat.group, nullptr, &clearLanguageCallback);
    await(&waiter);
    checkSucceeded(check, eranosGroupName(flat.group, text), "the name");
    checkThat(check, isText(text, "Home"), "as set");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    EranosCurrencyRate *rate = nullptr;
    eranosErrorDestroy(eranosCurrencyRateCreate("EUR", "5", &rate));
    eranosCurrencyRateListPush(rates, rate);
    EranosUnitCallback const changeCallback = {waiting(&waiter), receiveUnit};
    eranosGroupChangeCurrencyAsync(flat.group, "RON", rates, nullptr, &changeCallback);
    await(&waiter);
    checkSucceeded(check, eranosGroupCurrency(flat.group, text), "the currency");
    checkThat(check, nullptr == waiter.error && isText(text, "RON"), "changed");
    eranosCurrencyRateDestroy(rate);
    eranosCurrencyRateListDestroy(rates);
    EranosMemberCallback const failingCallback = {waiting(&waiter), receiveMember};
    eranosGroupRemoveMemberAsync(flat.group, "0199a000-0000-7000-8000-00000000ffff", nullptr, &failingCallback);
    await(&waiter);
    checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_UNKNOWN_MEMBER) && nullptr == waiter.object,
              "a call that fails reports its error and no value");
    EranosUnitCallback const failingUnitCallback = {waiting(&waiter), receiveUnit};
    eranosGroupSetNameAsync(flat.group, " ", nullptr, &failingUnitCallback);
    await(&waiter);
    checkThat(check, isFailure(waiter.error, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_EMPTY_NAME), "a change that fails reports its error");
    EranosMember *dan = nullptr;
    eranosErrorDestroy(eranosMemberCreate("Dan", &dan));
    eranosErrorDestroy(eranosMemberSetId(dan, "0199a000-0000-7000-8000-00000000ffff"));
    EranosTextCallback const failingTextCallback = {waiting(&waiter), receiveText};
    eranosGroupAddMemberAsync(flat.group, dan, nullptr, &failingTextCallback);
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_IDENTIFIER_GIVEN) && nullptr == waiter.text, "a call that returns text reports no text");
    eranosMemberDestroy(dan);
    EranosGroupCallback const brokenCallback = {waiting(&waiter), receiveGroup};
    eranosGroupLoadAsync("{", nullptr, &brokenCallback);
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
    EranosGroupReaderCallback const readerCallback = {waiting(&waiter), receiveReader};
    eranosGroupReadAsync(flat.group, nullptr, &readerCallback);
    await(&waiter);
    EranosGroupReader *const reader = waiter.object;
    checkThat(check, nullptr == waiter.error && nullptr != reader, "a read handle from an asynchronous call");
    checkSucceeded(check, eranosGroupReaderName(reader, text), "the name");
    checkThat(check, isText(text, "Flat"), "the group's");
    checkSucceeded(check, eranosGroupReaderId(reader, text), "the identifier");
    checkSucceeded(check, eranosGroupReaderCurrency(reader, text), "the currency");
    checkThat(check, isText(text, "EUR"), "the euro");
    checkSucceeded(check, eranosGroupReaderDescription(reader, text), "the description");
    checkThat(check, isText(text, "Rent"), "as given");
    checkSucceeded(check, eranosGroupReaderLanguage(reader, text), "the language");
    checkThat(check, isText(text, "pt-PT"), "as given");
    checkSucceeded(check, eranosGroupReaderSave(reader, text), "the saved group");
    EranosCategoryList *categories = nullptr;
    checkSucceeded(check, eranosGroupReaderCategories(reader, &categories), "the categories");
    EranosCategory *category = nullptr;
    checkSucceeded(check, eranosGroupReaderCategory(reader, eranosTextData(flat.food), &category), "a category");
    EranosExpenseList *expenses = nullptr;
    checkSucceeded(check, eranosGroupReaderExpenses(reader, &expenses), "the expenses");
    EranosExpense *expense = nullptr;
    checkSucceeded(check, eranosGroupReaderExpense(reader, eranosTextData(flat.dinner), &expense), "an expense");
    EranosExpenseShares *shares = nullptr;
    checkSucceeded(check, eranosGroupReaderExpenseShares(reader, eranosTextData(flat.dinner), &shares), "its shares");
    EranosSummary *summary = nullptr;
    checkSucceeded(check, eranosGroupReaderSummary(reader, &summary), "the summary");
    EranosMemberList *members = nullptr;
    checkSucceeded(check, eranosGroupReaderMembers(reader, &members), "the members");
    EranosMember *member = nullptr;
    checkSucceeded(check, eranosGroupReaderMember(reader, eranosTextData(flat.ana), &member), "a member");
    bool archived = true;
    checkSucceeded(check, eranosGroupReaderIsArchived(reader, eranosTextData(flat.ana), &archived), "whether a member is archived");
    EranosSettlement *settlement = nullptr;
    checkSucceeded(check, eranosGroupReaderSettlement(reader, ERANOS_SETTLEMENT_STRATEGY_DIRECT, &settlement), "a settlement");
    EranosTransferList *transfers = nullptr;
    checkSucceeded(check, eranosGroupReaderTransfers(reader, &transfers), "the transfers");
    EranosTransfer *transfer = nullptr;
    checkSucceeded(check, eranosGroupReaderTransfer(reader, eranosTextData(flat.repayment), &transfer), "a transfer");
    EranosMoney *amount = nullptr;
    checkSucceeded(check, eranosGroupReaderTransferAmount(reader, eranosTextData(flat.repayment), &amount), "its amount");
    checkThat(check,
              2 == eranosCategoryListLength(categories) && nullptr != category && 1 == eranosExpenseListLength(expenses) && nullptr != expense &&
                  nullptr != shares && nullptr != summary && 2 == eranosMemberListLength(members) && nullptr != member && !archived &&
                  nullptr != settlement && 1 == eranosTransferListLength(transfers) && nullptr != transfer && isAmount(amount, "EUR", 1'000),
              "every read sees the group");
    eranosGroupReaderRelease(reader);
    EranosError *const released = eranosGroupReaderName(reader, text);
    checkThat(check, isKind(released, ERANOS_ERROR_KIND_RELEASED), "a read after the release");
    EranosGroupReader *synchronous = nullptr;
    checkSucceeded(check, eranosGroupRead(flat.group, &synchronous), "a read handle from a synchronous call");
    eranosGroupReaderDestroy(synchronous);
    eranosTransferDestroy(transfer);
    eranosTransferListDestroy(transfers);
    eranosSettlementDestroy(settlement);
    eranosMemberDestroy(member);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
    eranosExpenseSharesDestroy(shares);
    eranosExpenseDestroy(expense);
    eranosExpenseListDestroy(expenses);
    eranosCategoryDestroy(category);
    eranosCategoryListDestroy(categories);
    eranosGroupReaderDestroy(reader);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A read handle whose reads went on after its release would read a group whose later changes it no longer waits for; every
/// read that returns an object fails once the handle is released, and gives no object.
static void everyReadOfAReleasedReaderFails(Check *const check) {
    Flat const flat = flatOf();
    EranosGroupReader *reader = nullptr;
    checkSucceeded(check, eranosGroupRead(flat.group, &reader), "a read handle");
    eranosGroupReaderRelease(reader);
    char const *const food = eranosTextData(flat.food);
    char const *const dinner = eranosTextData(flat.dinner);
    char const *const repayment = eranosTextData(flat.repayment);
    EranosCategoryList *categories = nullptr;
    checkThat(check, isKind(eranosGroupReaderCategories(reader, &categories), ERANOS_ERROR_KIND_RELEASED) && nullptr == categories, "the categories");
    EranosCategory *category = nullptr;
    checkThat(check, isKind(eranosGroupReaderCategory(reader, food, &category), ERANOS_ERROR_KIND_RELEASED) && nullptr == category, "a category");
    EranosExpenseList *expenses = nullptr;
    checkThat(check, isKind(eranosGroupReaderExpenses(reader, &expenses), ERANOS_ERROR_KIND_RELEASED) && nullptr == expenses, "the expenses");
    EranosExpense *expense = nullptr;
    checkThat(check, isKind(eranosGroupReaderExpense(reader, dinner, &expense), ERANOS_ERROR_KIND_RELEASED) && nullptr == expense, "an expense");
    EranosExpenseShares *shares = nullptr;
    checkThat(check, isKind(eranosGroupReaderExpenseShares(reader, dinner, &shares), ERANOS_ERROR_KIND_RELEASED) && nullptr == shares,
              "an expense's shares");
    EranosSummary *summary = nullptr;
    checkThat(check, isKind(eranosGroupReaderSummary(reader, &summary), ERANOS_ERROR_KIND_RELEASED) && nullptr == summary, "the summary");
    EranosMemberList *members = nullptr;
    checkThat(check, isKind(eranosGroupReaderMembers(reader, &members), ERANOS_ERROR_KIND_RELEASED) && nullptr == members, "the members");
    EranosMember *member = nullptr;
    checkThat(check, isKind(eranosGroupReaderMember(reader, eranosTextData(flat.ana), &member), ERANOS_ERROR_KIND_RELEASED) && nullptr == member,
              "a member");
    EranosSettlement *settlement = nullptr;
    checkThat(check,
              isKind(eranosGroupReaderSettlement(reader, ERANOS_SETTLEMENT_STRATEGY_DIRECT, &settlement), ERANOS_ERROR_KIND_RELEASED) &&
                  nullptr == settlement,
              "a settlement");
    EranosTransferList *transfers = nullptr;
    checkThat(check, isKind(eranosGroupReaderTransfers(reader, &transfers), ERANOS_ERROR_KIND_RELEASED) && nullptr == transfers, "the transfers");
    EranosTransfer *transfer = nullptr;
    checkThat(check, isKind(eranosGroupReaderTransfer(reader, repayment, &transfer), ERANOS_ERROR_KIND_RELEASED) && nullptr == transfer,
              "a transfer");
    EranosMoney *amount = nullptr;
    checkThat(check, isKind(eranosGroupReaderTransferAmount(reader, repayment, &amount), ERANOS_ERROR_KIND_RELEASED) && nullptr == amount,
              "a transfer's amount");
    eranosGroupReaderDestroy(reader);
    flatDestroy(flat);
}

/// A cancelled change that still ran would change the group after the app gave the change up; one that kept waiting would
/// hold the worker thread for a lock it no longer needs.
static void aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas(Check *const check) {
    Flat const flat = flatOf();
    EranosText *const text = eranosTextCreate();
    EranosGroupReader *reader = nullptr;
    checkSucceeded(check, eranosGroupRead(flat.group, &reader), "a read handle");
    EranosCancellation *const cancellation = eranosCancellationCreate();
    checkThat(check, !eranosCancellationIsCancelled(cancellation), "a cancellation starts uncancelled");
    EranosCurrencyRateList *const rates = eranosCurrencyRateListCreate();
    Waiter waiter;
    EranosUnitCallback const changeCallback = {waiting(&waiter), receiveUnit};
    eranosGroupChangeCurrencyAsync(flat.group, "RON", rates, cancellation, &changeCallback);
    eranosCancellationCancel(cancellation);
    checkThat(check, eranosCancellationIsCancelled(cancellation), "and is cancelled once cancelled");
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_CANCELLED), "the change reports its cancellation");
    eranosGroupReaderRelease(reader);
    checkSucceeded(check, eranosGroupCurrency(flat.group, text), "the currency");
    checkThat(check, isText(text, "EUR"), "is still the euro");
    EranosTextCallback const lateCallback = {waiting(&waiter), receiveText};
    eranosGroupNameAsync(flat.group, cancellation, &lateCallback);
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_CANCELLED) && nullptr == waiter.text,
              "a call given a cancelled cancellation does not run");
    EranosOptionalTextCallback const lateDescriptionCallback = {waiting(&waiter), receiveText};
    eranosGroupDescriptionAsync(flat.group, cancellation, &lateDescriptionCallback);
    await(&waiter);
    checkThat(check, isKind(waiter.error, ERANOS_ERROR_KIND_CANCELLED) && nullptr == waiter.text, "nor one that returns optional text");
    eranosCurrencyRateListDestroy(rates);
    eranosCancellationDestroy(cancellation);
    eranosGroupReaderDestroy(reader);
    eranosTextDestroy(text);
    flatDestroy(flat);
}

/// A test: its name, the claim it makes, and its body.
typedef struct Test {
    /// The test's name.
    char const *name;
    /// The test.
    void (*body)(Check *check);
} Test;

/// Runs every test and prints each claim that failed.
int main(void) {
    Test const tests[] = {
        {"aTextHoldsWhatTheLastCallWrote", aTextHoldsWhatTheLastCallWrote},
        {"aCurrencyHoldsWhatTheLibraryKnowsOfIt", aCurrencyHoldsWhatTheLibraryKnowsOfIt},
        {"textThatIsNotUtf8IsRefusedAndNullReadsAsEmpty", textThatIsNotUtf8IsRefusedAndNullReadsAsEmpty},
        {"anAmountParsesFormatsAndAdds", anAmountParsesFormatsAndAdds},
        {"ratesPercentagesAndPartsParseAndFormatAsTheLibraryDoes", ratesPercentagesAndPartsParseAndFormatAsTheLibraryDoes},
        {"datesAndTimesAreCheckedAndWritten", datesAndTimesAreCheckedAndWritten},
        {"contactsAndAccountsAreCheckedAndMadeCanonical", contactsAndAccountsAreCheckedAndMadeCanonical},
        {"aListOfIdentifiersKeepsItsOrder", aListOfIdentifiersKeepsItsOrder},
        {"everyShareOfASplitReadsBackAsItWasMade", everyShareOfASplitReadsBackAsItWasMade},
        {"itemsTipsAndCostsReadBackAsTheyWereMade", itemsTipsAndCostsReadBackAsTheyWereMade},
        {"anExpenseReadsBackEveryFieldItWasGiven", anExpenseReadsBackEveryFieldItWasGiven},
        {"aTransferReadsBackEveryFieldItWasGiven", aTransferReadsBackEveryFieldItWasGiven},
        {"aMemberAndTheirAccountsReadBackAsTheyWereGiven", aMemberAndTheirAccountsReadBackAsTheyWereGiven},
        {"aCategoryReadsBackAsItWasGiven", aCategoryReadsBackAsItWasGiven},
        {"everyBalanceFormatsAsTheLibraryWritesIt", everyBalanceFormatsAsTheLibraryWritesIt},
        {"destroyingNullDoesNothing", destroyingNullDoesNothing},
        {"aGroupHoldsItsDetailsAndChangesThem", aGroupHoldsItsDetailsAndChangesThem},
        {"entriesComeAndGoThroughTheGroup", entriesComeAndGoThroughTheGroup},
        {"anIdentifierThatIsNotOneIsRefusedByEveryLookup", anIdentifierThatIsNotOneIsRefusedByEveryLookup},
        {"theParticipantsOfASplitAndOfAnExpenseAreTheirMembers", theParticipantsOfASplitAndOfAnExpenseAreTheirMembers},
        {"anEntryTheGroupDoesNotHoldIsNeitherReplacedNorRemoved", anEntryTheGroupDoesNotHoldIsNeitherReplacedNorRemoved},
        {"theSummaryAndTheSharesAddUp", theSummaryAndTheSharesAddUp},
        {"eachStrategySettlesEveryBalance", eachStrategySettlesEveryBalance},
        {"anExactPlanTooLargeToComputeIsRefused", anExactPlanTooLargeToComputeIsRefused},
        {"aChangeOfCurrencyConvertsEveryEntry", aChangeOfCurrencyConvertsEveryEntry},
        {"aSavedGroupLoadsAsItWasSaved", aSavedGroupLoadsAsItWasSaved},
        {"everyErrorHoldsWhatItsFailureHolds", everyErrorHoldsWhatItsFailureHolds},
        {"everyAsynchronousCallReportsWhatItsSynchronousFormReturns", everyAsynchronousCallReportsWhatItsSynchronousFormReturns},
        {"aReaderReadsTheGroupUntilItIsReleased", aReaderReadsTheGroupUntilItIsReleased},
        {"everyReadOfAReleasedReaderFails", everyReadOfAReleasedReaderFails},
        {"aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas", aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas},
    };
    size_t const count = sizeof tests / sizeof tests[0];
    size_t failed = 0;
    for (size_t index = 0; index < count; ++index) {
        Check check = {.test = tests[index].name, .failures = 0};
        tests[index].body(&check);
        failed += 0 == check.failures ? 0U : 1U;
    }
    (void)printf("%zu of %zu tests passed\n", count - failed, count);
    return 0 == failed ? EXIT_SUCCESS : EXIT_FAILURE;
}
