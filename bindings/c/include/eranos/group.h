/// @file eranos/group.h
/// The group, the details it starts with, the rates of a currency change, the cancellation of asynchronous calls, and the callbacks that report their
/// outcomes.

#ifndef ERANOS_GROUP_H
#define ERANOS_GROUP_H

#include "EranosCancellation.h"
#include "EranosCategoryListOutcome.h"
#include "EranosCategoryOutcome.h"
#include "EranosCurrencyRate.h"
#include "EranosCurrencyRateList.h"
#include "EranosExpenseListOutcome.h"
#include "EranosExpenseOutcome.h"
#include "EranosExpenseSharesOutcome.h"
#include "EranosFlagOutcome.h"
#include "EranosGroup.h"
#include "EranosGroupOutcome.h"
#include "EranosGroupReaderOutcome.h"
#include "EranosMemberListOutcome.h"
#include "EranosMemberOutcome.h"
#include "EranosMoneyOutcome.h"
#include "EranosNewGroup.h"
#include "EranosOptionalTextOutcome.h"
#include "EranosSettlementOutcome.h"
#include "EranosSummaryOutcome.h"
#include "EranosTextOutcome.h"
#include "EranosTransferListOutcome.h"
#include "EranosTransferOutcome.h"
#include "EranosUnitOutcome.h"
#include "eranos/category.h"
#include "eranos/detail.h"
#include "eranos/expense.h"
#include "eranos/group_reader.h"
#include "eranos/members.h"
#include "eranos/money.h"
#include "eranos/text.h"
#include "eranos/totals.h"
#include "eranos/transfer.h"

#include <stddef.h>

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the categories, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosCategoryListCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosCategoryList *categories);
} EranosCategoryListCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunCategoryListCallback(void const *const data, EranosCategoryListOutcome *const outcome) {
    EranosCategoryListCallback const *const callback = data;
    EranosError *const error = EranosCategoryListOutcome_error(outcome);
    EranosCategoryList *const categories = EranosCategoryListOutcome_value(outcome);
    EranosCategoryListOutcome_destroy(outcome);
    callback->run(callback->context, error, categories);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the category, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosCategoryCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosCategory *category);
} EranosCategoryCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunCategoryCallback(void const *const data, EranosCategoryOutcome *const outcome) {
    EranosCategoryCallback const *const callback = data;
    EranosError *const error = EranosCategoryOutcome_error(outcome);
    EranosCategory *const category = EranosCategoryOutcome_value(outcome);
    EranosCategoryOutcome_destroy(outcome);
    callback->run(callback->context, error, category);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the expenses, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosExpenseListCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosExpenseList *expenses);
} EranosExpenseListCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunExpenseListCallback(void const *const data, EranosExpenseListOutcome *const outcome) {
    EranosExpenseListCallback const *const callback = data;
    EranosError *const error = EranosExpenseListOutcome_error(outcome);
    EranosExpenseList *const expenses = EranosExpenseListOutcome_value(outcome);
    EranosExpenseListOutcome_destroy(outcome);
    callback->run(callback->context, error, expenses);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the expense, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosExpenseCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosExpense *expense);
} EranosExpenseCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunExpenseCallback(void const *const data, EranosExpenseOutcome *const outcome) {
    EranosExpenseCallback const *const callback = data;
    EranosError *const error = EranosExpenseOutcome_error(outcome);
    EranosExpense *const expense = EranosExpenseOutcome_value(outcome);
    EranosExpenseOutcome_destroy(outcome);
    callback->run(callback->context, error, expense);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the expense shares, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosExpenseSharesCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosExpenseShares *expenseShares);
} EranosExpenseSharesCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunExpenseSharesCallback(void const *const data, EranosExpenseSharesOutcome *const outcome) {
    EranosExpenseSharesCallback const *const callback = data;
    EranosError *const error = EranosExpenseSharesOutcome_error(outcome);
    EranosExpenseShares *const expenseShares = EranosExpenseSharesOutcome_value(outcome);
    EranosExpenseSharesOutcome_destroy(outcome);
    callback->run(callback->context, error, expenseShares);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the value, false when the call failed. The
/// callback destroys each object it receives.
typedef struct EranosFlagCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, bool value);
} EranosFlagCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFlagCallback(void const *const data, EranosFlagOutcome *const outcome) {
    EranosFlagCallback const *const callback = data;
    EranosError *const error = EranosFlagOutcome_error(outcome);
    bool const value = EranosFlagOutcome_value(outcome);
    EranosFlagOutcome_destroy(outcome);
    callback->run(callback->context, error, value);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the group, or nullptr when the call failed.
/// The callback destroys each object it receives.
typedef struct EranosGroupCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosGroup *group);
} EranosGroupCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunGroupCallback(void const *const data, EranosGroupOutcome *const outcome) {
    EranosGroupCallback const *const callback = data;
    EranosError *const error = EranosGroupOutcome_error(outcome);
    EranosGroup *const group = EranosGroupOutcome_take(outcome);
    EranosGroupOutcome_destroy(outcome);
    callback->run(callback->context, error, group);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the reader, or nullptr when the call failed.
/// The callback destroys each object it receives.
typedef struct EranosGroupReaderCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosGroupReader *reader);
} EranosGroupReaderCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunGroupReaderCallback(void const *const data, EranosGroupReaderOutcome *const outcome) {
    EranosGroupReaderCallback const *const callback = data;
    EranosError *const error = EranosGroupReaderOutcome_error(outcome);
    EranosGroupReader *const reader = EranosGroupReaderOutcome_take(outcome);
    EranosGroupReaderOutcome_destroy(outcome);
    callback->run(callback->context, error, reader);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the members, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosMemberListCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosMemberList *members);
} EranosMemberListCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunMemberListCallback(void const *const data, EranosMemberListOutcome *const outcome) {
    EranosMemberListCallback const *const callback = data;
    EranosError *const error = EranosMemberListOutcome_error(outcome);
    EranosMemberList *const members = EranosMemberListOutcome_value(outcome);
    EranosMemberListOutcome_destroy(outcome);
    callback->run(callback->context, error, members);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the member, or nullptr when the call failed.
/// The callback destroys each object it receives.
typedef struct EranosMemberCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosMember *member);
} EranosMemberCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunMemberCallback(void const *const data, EranosMemberOutcome *const outcome) {
    EranosMemberCallback const *const callback = data;
    EranosError *const error = EranosMemberOutcome_error(outcome);
    EranosMember *const member = EranosMemberOutcome_value(outcome);
    EranosMemberOutcome_destroy(outcome);
    callback->run(callback->context, error, member);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the money, or nullptr when the call failed.
/// The callback destroys each object it receives.
typedef struct EranosMoneyCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosMoney *money);
} EranosMoneyCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunMoneyCallback(void const *const data, EranosMoneyOutcome *const outcome) {
    EranosMoneyCallback const *const callback = data;
    EranosError *const error = EranosMoneyOutcome_error(outcome);
    EranosMoney *const money = EranosMoneyOutcome_value(outcome);
    EranosMoneyOutcome_destroy(outcome);
    callback->run(callback->context, error, money);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the text, or nullptr when the call failed or
/// found none. The callback destroys each object it receives.
typedef struct EranosOptionalTextCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosText *text);
} EranosOptionalTextCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunOptionalTextCallback(void const *const data, EranosOptionalTextOutcome *const outcome) {
    EranosOptionalTextCallback const *const callback = data;
    EranosError *const error = EranosOptionalTextOutcome_error(outcome);
    EranosText *text = nullptr;
    if (nullptr == error) {
        text = eranosTextCreate();
        if (!EranosOptionalTextOutcome_value(outcome, text).is_ok) {
            eranosTextDestroy(text);
            text = nullptr;
        } else {
            eranosDetailFinish(text);
        }
    }
    EranosOptionalTextOutcome_destroy(outcome);
    callback->run(callback->context, error, text);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the settlement, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosSettlementCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosSettlement *settlement);
} EranosSettlementCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunSettlementCallback(void const *const data, EranosSettlementOutcome *const outcome) {
    EranosSettlementCallback const *const callback = data;
    EranosError *const error = EranosSettlementOutcome_error(outcome);
    EranosSettlement *const settlement = EranosSettlementOutcome_value(outcome);
    EranosSettlementOutcome_destroy(outcome);
    callback->run(callback->context, error, settlement);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the summary, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosSummaryCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosSummary *summary);
} EranosSummaryCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunSummaryCallback(void const *const data, EranosSummaryOutcome *const outcome) {
    EranosSummaryCallback const *const callback = data;
    EranosError *const error = EranosSummaryOutcome_error(outcome);
    EranosSummary *const summary = EranosSummaryOutcome_value(outcome);
    EranosSummaryOutcome_destroy(outcome);
    callback->run(callback->context, error, summary);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the text, or nullptr when the call failed.
/// The callback destroys each object it receives.
typedef struct EranosTextCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosText *text);
} EranosTextCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunTextCallback(void const *const data, EranosTextOutcome *const outcome) {
    EranosTextCallback const *const callback = data;
    EranosError *const error = EranosTextOutcome_error(outcome);
    EranosText *text = nullptr;
    if (nullptr == error) {
        text = eranosTextCreate();
        (void)EranosTextOutcome_value(outcome, text);
        eranosDetailFinish(text);
    }
    EranosTextOutcome_destroy(outcome);
    callback->run(callback->context, error, text);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the transfers, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosTransferListCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosTransferList *transfers);
} EranosTransferListCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunTransferListCallback(void const *const data, EranosTransferListOutcome *const outcome) {
    EranosTransferListCallback const *const callback = data;
    EranosError *const error = EranosTransferListOutcome_error(outcome);
    EranosTransferList *const transfers = EranosTransferListOutcome_value(outcome);
    EranosTransferListOutcome_destroy(outcome);
    callback->run(callback->context, error, transfers);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the transfer, or nullptr when the call
/// failed. The callback destroys each object it receives.
typedef struct EranosTransferCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosTransfer *transfer);
} EranosTransferCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunTransferCallback(void const *const data, EranosTransferOutcome *const outcome) {
    EranosTransferCallback const *const callback = data;
    EranosError *const error = EranosTransferOutcome_error(outcome);
    EranosTransfer *const transfer = EranosTransferOutcome_value(outcome);
    EranosTransferOutcome_destroy(outcome);
    callback->run(callback->context, error, transfer);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded. The callback destroys each object it receives.
typedef struct EranosUnitCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error);
} EranosUnitCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunUnitCallback(void const *const data, EranosUnitOutcome *const outcome) {
    EranosUnitCallback const *const callback = data;
    EranosError *const error = EranosUnitOutcome_error(outcome);
    EranosUnitOutcome_destroy(outcome);
    callback->run(callback->context, error);
}

/// A cancellation of asynchronous calls, which one cancellation can be given to several of.  Cancelling it stops the calls it was given at their next
/// yield, or at once while they wait for the lock, and they report the kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
typedef struct EranosCancellation EranosCancellation;

/// A cancellation not yet cancelled.
///
/// @return The cancellation, which the caller destroys
[[nodiscard]] static inline EranosCancellation *eranosCancellationCreate(void) { return EranosCancellation_create(); }

/// Cancels every call it was given.
///
/// @param cancellation The cancellation
static inline void eranosCancellationCancel(EranosCancellation *const cancellation) { EranosCancellation_cancel(cancellation); }

/// Whether it was cancelled.
///
/// @param cancellation The cancellation
/// @return Whether it is so
[[nodiscard]] static inline bool eranosCancellationIsCancelled(EranosCancellation const *const cancellation) {
    return EranosCancellation_is_cancelled(cancellation);
}

/// Destroys @p cancellation; destroying nullptr does nothing, as free() does.
///
/// @param cancellation The cancellation, which no call uses after this one, or nullptr
static inline void eranosCancellationDestroy(EranosCancellation *const cancellation) {
    if (nullptr != cancellation) {
        EranosCancellation_destroy(cancellation);
    }
}

/// The rate into a new group currency of an amount in another.
typedef struct EranosCurrencyRate EranosCurrencyRate;

/// A new currency rate of its required fields; every other field starts absent, empty, or at its default.
///
/// @param currency The ISO 4217 code of the currency the rate converts from, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param rate The rate, as a decimal, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param currencyRate Receives the currency rate on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCurrencyRateCreate(char const *const currency, char const *const rate,
                                                                  EranosCurrencyRate **const currencyRate) {
    EranosCurrencyRate_create_result const result = EranosCurrencyRate_create(eranosDetailView(currency), eranosDetailView(rate));
    *currencyRate = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The ISO 4217 code of the currency the rate converts from.
///
/// @param currencyRate The currency rate
/// @param currency Receives the ISO 4217 code of the currency the rate converts from, replacing what it held
static inline void eranosCurrencyRateCurrency(EranosCurrencyRate const *const currencyRate, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosCurrencyRate_currency(currencyRate, currency);
    eranosDetailFinish(currency);
}

/// Sets the ISO 4217 code of the currency the rate converts from.
///
/// @param currencyRate The currency rate
/// @param currency The ISO 4217 code of the currency the rate converts from, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCurrencyRateSetCurrency(EranosCurrencyRate *const currencyRate, char const *const currency) {
    EranosCurrencyRate_set_currency_result const result = EranosCurrencyRate_set_currency(currencyRate, eranosDetailView(currency));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The rate, as a decimal.
///
/// @param currencyRate The currency rate
/// @param rate Receives the rate, as a decimal, replacing what it held
static inline void eranosCurrencyRateRate(EranosCurrencyRate const *const currencyRate, EranosText *const rate) {
    eranosDetailClear(rate);
    EranosCurrencyRate_rate(currencyRate, rate);
    eranosDetailFinish(rate);
}

/// Sets the rate, as a decimal.
///
/// @param currencyRate The currency rate
/// @param rate The rate, as a decimal, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCurrencyRateSetRate(EranosCurrencyRate *const currencyRate, char const *const rate) {
    EranosCurrencyRate_set_rate_result const result = EranosCurrencyRate_set_rate(currencyRate, eranosDetailView(rate));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p currencyRate; destroying nullptr does nothing, as free() does.
///
/// @param currencyRate The currency rate, which no call uses after this one, or nullptr
static inline void eranosCurrencyRateDestroy(EranosCurrencyRate *const currencyRate) {
    if (nullptr != currencyRate) {
        EranosCurrencyRate_destroy(currencyRate);
    }
}

/// A list of currency rate records.
typedef struct EranosCurrencyRateList EranosCurrencyRateList;

/// An empty list.
///
/// @return The currency rate list, which the caller destroys
[[nodiscard]] static inline EranosCurrencyRateList *eranosCurrencyRateListCreate(void) { return EranosCurrencyRateList_create(); }

/// Adds a copy of `element` at the end.
///
/// @param list The currency rate list
/// @param element The element
static inline void eranosCurrencyRateListPush(EranosCurrencyRateList *const list, EranosCurrencyRate const *const element) {
    EranosCurrencyRateList_push(list, element);
}

/// How many records the list holds.
///
/// @param list The currency rate list
/// @return The result
[[nodiscard]] static inline size_t eranosCurrencyRateListLength(EranosCurrencyRateList const *const list) { return EranosCurrencyRateList_len(list); }

/// Whether the list holds no record.
///
/// @param list The currency rate list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosCurrencyRateListIsEmpty(EranosCurrencyRateList const *const list) {
    return EranosCurrencyRateList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The currency rate list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosCurrencyRate *eranosCurrencyRateListGet(EranosCurrencyRateList const *const list, size_t const index) {
    return EranosCurrencyRateList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The currency rate list, which no call uses after this one, or nullptr
static inline void eranosCurrencyRateListDestroy(EranosCurrencyRateList *const list) {
    if (nullptr != list) {
        EranosCurrencyRateList_destroy(list);
    }
}

/// A group: its details, members, categories, expenses, transfers, totals, and settlements.  Every call has a synchronous form, which blocks the
/// calling thread while another call holds the lock, and an asynchronous one, which runs on the group's worker thread, started by the first, in the
/// order of the calls, and reports to its callback there, so a callback must be safe to call from another thread. Destroying the group lets the
/// queued calls finish and their callbacks run.
typedef struct EranosGroup EranosGroup;

/// A new group, with the details `details` holds.
///
/// @param details The details
/// @param group Receives the group on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a
/// currency
///   the library does not list, of the kind ERANOS_ERROR_KIND_GROUP for a blank name, and of the kind ERANOS_ERROR_KIND_LANGUAGE for a language that
///   is not a tag
[[nodiscard]] static inline EranosError *eranosGroupCreate(EranosNewGroup const *const details, EranosGroup **const group) {
    EranosGroup_create_result const result = EranosGroup_create(details);
    *group = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group saved as `json`, which `save` wrote.
///
/// @param json The json, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param group Receives the group on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_JSON for
///   text that is not a saved group, of the kind ERANOS_ERROR_KIND_GROUP for a format this release cannot read, of the module whose rule an entry
///   breaks, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupLoad(char const *const json, EranosGroup **const group) {
    EranosGroup_load_result const result = EranosGroup_load(eranosDetailView(json));
    *group = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `load`: it runs on a thread of its own and reports its outcome to `callback` there.
///
/// @param json The json, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupLoadAsync(char const *const json, EranosCancellation const *const cancellation,
                                        EranosGroupCallback const *const callback) {
    DiplomatCallback_EranosGroup_load_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunGroupCallback,
        .destructor = nullptr,
    };
    EranosGroup_load_async(eranosDetailView(json), cancellation, wrapped);
}

/// The group's identifier. Writes it into @p id.
///
/// @param group The group
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupId(EranosGroup const *const group, EranosText *const id) {
    eranosDetailClear(id);
    EranosGroup_id_result const result = EranosGroup_id(group, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `id`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupIdAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                      EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_id_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_id_async(group, cancellation, wrapped);
}

/// Every category, the built-in one first.
///
/// @param group The group
/// @param categories Receives the categories on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupCategories(EranosGroup const *const group, EranosCategoryList **const categories) {
    EranosGroup_categories_result const result = EranosGroup_categories(group);
    *categories = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `categories`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupCategoriesAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                              EranosCategoryListCallback const *const callback) {
    DiplomatCallback_EranosGroup_categories_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunCategoryListCallback,
        .destructor = nullptr,
    };
    EranosGroup_categories_async(group, cancellation, wrapped);
}

/// The category with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param category Receives the category on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupCategory(EranosGroup const *const group, char const *const id, EranosCategory **const category) {
    EranosGroup_category_result const result = EranosGroup_category(group, eranosDetailView(id));
    *category = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `category`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupCategoryAsync(EranosGroup const *const group, char const *const id, EranosCancellation const *const cancellation,
                                            EranosCategoryCallback const *const callback) {
    DiplomatCallback_EranosGroup_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunCategoryCallback,
        .destructor = nullptr,
    };
    EranosGroup_category_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `category`, which has no identifier yet, and returns the one it gets. Writes it into @p id.
///
/// @param group The group
/// @param category The category
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupAddCategory(EranosGroup *const group, EranosCategory const *const category,
                                                                EranosText *const id) {
    eranosDetailClear(id);
    EranosGroup_add_category_result const result = EranosGroup_add_category(group, category, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `add_category`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param category The category
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupAddCategoryAsync(EranosGroup *const group, EranosCategory const *const category,
                                               EranosCancellation const *const cancellation, EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_add_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_add_category_async(group, category, cancellation, wrapped);
}

/// Renames the category that `category` names by its identifier, and returns it as it was.
///
/// @param group The group
/// @param category The category
/// @param previous Receives the previous on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupReplaceCategory(EranosGroup *const group, EranosCategory const *const category,
                                                                    EranosCategory **const previous) {
    EranosGroup_replace_category_result const result = EranosGroup_replace_category(group, category);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `replace_category`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param category The category
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupReplaceCategoryAsync(EranosGroup *const group, EranosCategory const *const category,
                                                   EranosCancellation const *const cancellation, EranosCategoryCallback const *const callback) {
    DiplomatCallback_EranosGroup_replace_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunCategoryCallback,
        .destructor = nullptr,
    };
    EranosGroup_replace_category_async(group, category, cancellation, wrapped);
}

/// Removes the category with the identifier `id` and returns it; its expenses move to the built-in one.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the removed on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupRemoveCategory(EranosGroup *const group, char const *const id, EranosCategory **const removed) {
    EranosGroup_remove_category_result const result = EranosGroup_remove_category(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `remove_category`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupRemoveCategoryAsync(EranosGroup *const group, char const *const id, EranosCancellation const *const cancellation,
                                                  EranosCategoryCallback const *const callback) {
    DiplomatCallback_EranosGroup_remove_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunCategoryCallback,
        .destructor = nullptr,
    };
    EranosGroup_remove_category_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The ISO 4217 code of the group currency. Writes it into @p currency.
///
/// @param group The group
/// @param currency Receives the currency, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupCurrency(EranosGroup const *const group, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosGroup_currency_result const result = EranosGroup_currency(group, currency);
    eranosDetailFinish(currency);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `currency`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupCurrencyAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                            EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_currency_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_currency_async(group, cancellation, wrapped);
}

/// Moves the group to `currency`, with a rate into it for every other currency its entries use.
///
/// @param group The group
/// @param currency The currency, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param rates The rates
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupChangeCurrency(EranosGroup *const group, char const *const currency,
                                                                   EranosCurrencyRateList const *const rates) {
    EranosGroup_change_currency_result const result = EranosGroup_change_currency(group, eranosDetailView(currency), rates);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `change_currency`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param currency The currency, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param rates The rates
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupChangeCurrencyAsync(EranosGroup *const group, char const *const currency, EranosCurrencyRateList const *const rates,
                                                  EranosCancellation const *const cancellation, EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosGroup_change_currency_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosGroup_change_currency_async(group, eranosDetailView(currency), rates, cancellation, wrapped);
}

/// Every expense, in the order they were created.
///
/// @param group The group
/// @param expenses Receives the expenses on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupExpenses(EranosGroup const *const group, EranosExpenseList **const expenses) {
    EranosGroup_expenses_result const result = EranosGroup_expenses(group);
    *expenses = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `expenses`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupExpensesAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                            EranosExpenseListCallback const *const callback) {
    DiplomatCallback_EranosGroup_expenses_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseListCallback,
        .destructor = nullptr,
    };
    EranosGroup_expenses_async(group, cancellation, wrapped);
}

/// The expense with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expense Receives the expense on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupExpense(EranosGroup const *const group, char const *const id, EranosExpense **const expense) {
    EranosGroup_expense_result const result = EranosGroup_expense(group, eranosDetailView(id));
    *expense = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `expense`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupExpenseAsync(EranosGroup const *const group, char const *const id, EranosCancellation const *const cancellation,
                                           EranosExpenseCallback const *const callback) {
    DiplomatCallback_EranosGroup_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseCallback,
        .destructor = nullptr,
    };
    EranosGroup_expense_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// How the expense with the identifier `id` divides, in the group currency.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expenseShares Receives the expense shares on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupExpenseShares(EranosGroup const *const group, char const *const id,
                                                                  EranosExpenseShares **const expenseShares) {
    EranosGroup_expense_shares_result const result = EranosGroup_expense_shares(group, eranosDetailView(id));
    *expenseShares = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `expense_shares`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupExpenseSharesAsync(EranosGroup const *const group, char const *const id, EranosCancellation const *const cancellation,
                                                 EranosExpenseSharesCallback const *const callback) {
    DiplomatCallback_EranosGroup_expense_shares_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseSharesCallback,
        .destructor = nullptr,
    };
    EranosGroup_expense_shares_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `expense`, which has no identifier yet, and returns the one it gets. Writes it into @p id.
///
/// @param group The group
/// @param expense The expense
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupAddExpense(EranosGroup *const group, EranosExpense const *const expense, EranosText *const id) {
    eranosDetailClear(id);
    EranosGroup_add_expense_result const result = EranosGroup_add_expense(group, expense, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `add_expense`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param expense The expense
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupAddExpenseAsync(EranosGroup *const group, EranosExpense const *const expense,
                                              EranosCancellation const *const cancellation, EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_add_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_add_expense_async(group, expense, cancellation, wrapped);
}

/// Replaces the expense that `expense` names by its identifier, and returns it as it was.
///
/// @param group The group
/// @param expense The expense
/// @param previous Receives the previous on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupReplaceExpense(EranosGroup *const group, EranosExpense const *const expense,
                                                                   EranosExpense **const previous) {
    EranosGroup_replace_expense_result const result = EranosGroup_replace_expense(group, expense);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `replace_expense`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param expense The expense
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupReplaceExpenseAsync(EranosGroup *const group, EranosExpense const *const expense,
                                                  EranosCancellation const *const cancellation, EranosExpenseCallback const *const callback) {
    DiplomatCallback_EranosGroup_replace_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseCallback,
        .destructor = nullptr,
    };
    EranosGroup_replace_expense_async(group, expense, cancellation, wrapped);
}

/// Removes the expense with the identifier `id` and returns it.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the removed on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupRemoveExpense(EranosGroup *const group, char const *const id, EranosExpense **const removed) {
    EranosGroup_remove_expense_result const result = EranosGroup_remove_expense(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `remove_expense`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupRemoveExpenseAsync(EranosGroup *const group, char const *const id, EranosCancellation const *const cancellation,
                                                 EranosExpenseCallback const *const callback) {
    DiplomatCallback_EranosGroup_remove_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseCallback,
        .destructor = nullptr,
    };
    EranosGroup_remove_expense_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The group's totals.
///
/// @param group The group
/// @param summary Receives the summary on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupSummary(EranosGroup const *const group, EranosSummary **const summary) {
    EranosGroup_summary_result const result = EranosGroup_summary(group);
    *summary = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `summary`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupSummaryAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                           EranosSummaryCallback const *const callback) {
    DiplomatCallback_EranosGroup_summary_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunSummaryCallback,
        .destructor = nullptr,
    };
    EranosGroup_summary_async(group, cancellation, wrapped);
}

/// Every member, archived ones included, in the order they were created.
///
/// @param group The group
/// @param members Receives the members on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupMembers(EranosGroup const *const group, EranosMemberList **const members) {
    EranosGroup_members_result const result = EranosGroup_members(group);
    *members = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `members`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupMembersAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                           EranosMemberListCallback const *const callback) {
    DiplomatCallback_EranosGroup_members_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMemberListCallback,
        .destructor = nullptr,
    };
    EranosGroup_members_async(group, cancellation, wrapped);
}

/// The member with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param member Receives the member on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupMember(EranosGroup const *const group, char const *const id, EranosMember **const member) {
    EranosGroup_member_result const result = EranosGroup_member(group, eranosDetailView(id));
    *member = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `member`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupMemberAsync(EranosGroup const *const group, char const *const id, EranosCancellation const *const cancellation,
                                          EranosMemberCallback const *const callback) {
    DiplomatCallback_EranosGroup_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMemberCallback,
        .destructor = nullptr,
    };
    EranosGroup_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Whether the member with the identifier `id` is archived; `false` for an unknown member.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param archived Receives the archived on success
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupIsArchived(EranosGroup const *const group, char const *const id, bool *const archived) {
    EranosGroup_is_archived_result const result = EranosGroup_is_archived(group, eranosDetailView(id));
    *archived = false;
    if (result.is_ok) {
        *archived = result.ok;
    }
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `is_archived`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupIsArchivedAsync(EranosGroup const *const group, char const *const id, EranosCancellation const *const cancellation,
                                              EranosFlagCallback const *const callback) {
    DiplomatCallback_EranosGroup_is_archived_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFlagCallback,
        .destructor = nullptr,
    };
    EranosGroup_is_archived_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `member`, who has no identifier yet, and returns the one they get. Writes it into @p id.
///
/// @param group The group
/// @param member The member
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupAddMember(EranosGroup *const group, EranosMember const *const member, EranosText *const id) {
    eranosDetailClear(id);
    EranosGroup_add_member_result const result = EranosGroup_add_member(group, member, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `add_member`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param member The member
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupAddMemberAsync(EranosGroup *const group, EranosMember const *const member, EranosCancellation const *const cancellation,
                                             EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_add_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_add_member_async(group, member, cancellation, wrapped);
}

/// Replaces the member that `member` names by their identifier, and returns them as they were.
///
/// @param group The group
/// @param member The member
/// @param previous Receives the previous on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupReplaceMember(EranosGroup *const group, EranosMember const *const member,
                                                                  EranosMember **const previous) {
    EranosGroup_replace_member_result const result = EranosGroup_replace_member(group, member);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `replace_member`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param member The member
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupReplaceMemberAsync(EranosGroup *const group, EranosMember const *const member,
                                                 EranosCancellation const *const cancellation, EranosMemberCallback const *const callback) {
    DiplomatCallback_EranosGroup_replace_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMemberCallback,
        .destructor = nullptr,
    };
    EranosGroup_replace_member_async(group, member, cancellation, wrapped);
}

/// Archives the member with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupArchiveMember(EranosGroup *const group, char const *const id) {
    EranosGroup_archive_member_result const result = EranosGroup_archive_member(group, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `archive_member`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupArchiveMemberAsync(EranosGroup *const group, char const *const id, EranosCancellation const *const cancellation,
                                                 EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosGroup_archive_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosGroup_archive_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Unarchives the member with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupUnarchiveMember(EranosGroup *const group, char const *const id) {
    EranosGroup_unarchive_member_result const result = EranosGroup_unarchive_member(group, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `unarchive_member`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupUnarchiveMemberAsync(EranosGroup *const group, char const *const id, EranosCancellation const *const cancellation,
                                                   EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosGroup_unarchive_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosGroup_unarchive_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Removes the member with the identifier `id`, whom no entry references, and returns them.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the removed on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupRemoveMember(EranosGroup *const group, char const *const id, EranosMember **const removed) {
    EranosGroup_remove_member_result const result = EranosGroup_remove_member(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `remove_member`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupRemoveMemberAsync(EranosGroup *const group, char const *const id, EranosCancellation const *const cancellation,
                                                EranosMemberCallback const *const callback) {
    DiplomatCallback_EranosGroup_remove_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMemberCallback,
        .destructor = nullptr,
    };
    EranosGroup_remove_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The group's name. Writes it into @p name.
///
/// @param group The group
/// @param name Receives the name, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupName(EranosGroup const *const group, EranosText *const name) {
    eranosDetailClear(name);
    EranosGroup_name_result const result = EranosGroup_name(group, name);
    eranosDetailFinish(name);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `name`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupNameAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                        EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_name_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_name_async(group, cancellation, wrapped);
}

/// Renames the group.
///
/// @param group The group
/// @param name The name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupSetName(EranosGroup *const group, char const *const name) {
    EranosGroup_set_name_result const result = EranosGroup_set_name(group, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `set_name`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param name The name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupSetNameAsync(EranosGroup *const group, char const *const name, EranosCancellation const *const cancellation,
                                           EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosGroup_set_name_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosGroup_set_name_async(group, eranosDetailView(name), cancellation, wrapped);
}

/// The payments that settle every balance by `strategy`.
///
/// @param group The group
/// @param strategy The strategy
/// @param settlement Receives the settlement on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupSettlement(EranosGroup const *const group, EranosSettlementStrategy const strategy,
                                                               EranosSettlement **const settlement) {
    EranosGroup_settlement_result const result = EranosGroup_settlement(group, strategy);
    *settlement = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `settlement`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param strategy The strategy
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupSettlementAsync(EranosGroup const *const group, EranosSettlementStrategy const strategy,
                                              EranosCancellation const *const cancellation, EranosSettlementCallback const *const callback) {
    DiplomatCallback_EranosGroup_settlement_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunSettlementCallback,
        .destructor = nullptr,
    };
    EranosGroup_settlement_async(group, strategy, cancellation, wrapped);
}

/// Every transfer, in the order they were created.
///
/// @param group The group
/// @param transfers Receives the transfers on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupTransfers(EranosGroup const *const group, EranosTransferList **const transfers) {
    EranosGroup_transfers_result const result = EranosGroup_transfers(group);
    *transfers = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `transfers`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupTransfersAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                             EranosTransferListCallback const *const callback) {
    DiplomatCallback_EranosGroup_transfers_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTransferListCallback,
        .destructor = nullptr,
    };
    EranosGroup_transfers_async(group, cancellation, wrapped);
}

/// The transfer with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transfer Receives the transfer on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupTransfer(EranosGroup const *const group, char const *const id, EranosTransfer **const transfer) {
    EranosGroup_transfer_result const result = EranosGroup_transfer(group, eranosDetailView(id));
    *transfer = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `transfer`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupTransferAsync(EranosGroup const *const group, char const *const id, EranosCancellation const *const cancellation,
                                            EranosTransferCallback const *const callback) {
    DiplomatCallback_EranosGroup_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTransferCallback,
        .destructor = nullptr,
    };
    EranosGroup_transfer_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The amount in the group currency of the transfer with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transferAmount Receives the transfer amount on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupTransferAmount(EranosGroup const *const group, char const *const id,
                                                                   EranosMoney **const transferAmount) {
    EranosGroup_transfer_amount_result const result = EranosGroup_transfer_amount(group, eranosDetailView(id));
    *transferAmount = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `transfer_amount`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupTransferAmountAsync(EranosGroup const *const group, char const *const id, EranosCancellation const *const cancellation,
                                                  EranosMoneyCallback const *const callback) {
    DiplomatCallback_EranosGroup_transfer_amount_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMoneyCallback,
        .destructor = nullptr,
    };
    EranosGroup_transfer_amount_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `transfer`, which has no identifier yet, and returns the one it gets. Writes it into @p id.
///
/// @param group The group
/// @param transfer The transfer
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupAddTransfer(EranosGroup *const group, EranosTransfer const *const transfer,
                                                                EranosText *const id) {
    eranosDetailClear(id);
    EranosGroup_add_transfer_result const result = EranosGroup_add_transfer(group, transfer, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `add_transfer`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param transfer The transfer
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupAddTransferAsync(EranosGroup *const group, EranosTransfer const *const transfer,
                                               EranosCancellation const *const cancellation, EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_add_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_add_transfer_async(group, transfer, cancellation, wrapped);
}

/// Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
///
/// @param group The group
/// @param transfer The transfer
/// @param previous Receives the previous on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupReplaceTransfer(EranosGroup *const group, EranosTransfer const *const transfer,
                                                                    EranosTransfer **const previous) {
    EranosGroup_replace_transfer_result const result = EranosGroup_replace_transfer(group, transfer);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `replace_transfer`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param transfer The transfer
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupReplaceTransferAsync(EranosGroup *const group, EranosTransfer const *const transfer,
                                                   EranosCancellation const *const cancellation, EranosTransferCallback const *const callback) {
    DiplomatCallback_EranosGroup_replace_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTransferCallback,
        .destructor = nullptr,
    };
    EranosGroup_replace_transfer_async(group, transfer, cancellation, wrapped);
}

/// Removes the transfer with the identifier `id` and returns it.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the removed on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_GROUP with the failure `GroupBusy` on wasm while another call holds the lock; the error of the kind
///   ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupRemoveTransfer(EranosGroup *const group, char const *const id, EranosTransfer **const removed) {
    EranosGroup_remove_transfer_result const result = EranosGroup_remove_transfer(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `remove_transfer`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupRemoveTransferAsync(EranosGroup *const group, char const *const id, EranosCancellation const *const cancellation,
                                                  EranosTransferCallback const *const callback) {
    DiplomatCallback_EranosGroup_remove_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTransferCallback,
        .destructor = nullptr,
    };
    EranosGroup_remove_transfer_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The group's description. Writes it into @p description, if the group has one, and nothing otherwise: the group holds no blank text there, since
/// blank text clears it.
///
/// @param group The group
/// @param description Receives the description, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupDescription(EranosGroup const *const group, EranosText *const description) {
    eranosDetailClear(description);
    EranosGroup_description_result const result = EranosGroup_description(group, description);
    eranosDetailFinish(description);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `description`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupDescriptionAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                               EranosOptionalTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_description_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunOptionalTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_description_async(group, cancellation, wrapped);
}

/// Sets the description; blank text clears it.
///
/// @param group The group
/// @param description The description, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupSetDescription(EranosGroup *const group, char const *const description) {
    EranosGroup_set_description_result const result = EranosGroup_set_description(group, eranosDetailView(description));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `set_description`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback`
/// there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param description The description, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupSetDescriptionAsync(EranosGroup *const group, char const *const description,
                                                  EranosCancellation const *const cancellation, EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosGroup_set_description_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosGroup_set_description_async(group, eranosDetailView(description), cancellation, wrapped);
}

/// The language the apps show the group in, as a BCP 47 tag. Writes it into @p language, if the group has one, and nothing otherwise: the group holds
/// no blank text there, since blank text clears it.
///
/// @param group The group
/// @param language Receives the language, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while another call holds the lock
[[nodiscard]] static inline EranosError *eranosGroupLanguage(EranosGroup const *const group, EranosText *const language) {
    eranosDetailClear(language);
    EranosGroup_language_result const result = EranosGroup_language(group, language);
    eranosDetailFinish(language);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `language`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupLanguageAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                            EranosOptionalTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_language_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunOptionalTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_language_async(group, cancellation, wrapped);
}

/// Sets the language the apps show the group in, a BCP 47 tag.
///
/// @param group The group
/// @param language The language, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_LANGUAGE for text
///   that is not a language tag, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosGroupSetLanguage(EranosGroup *const group, char const *const language) {
    EranosGroup_set_language_result const result = EranosGroup_set_language(group, eranosDetailView(language));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the language the apps show the group in.
///
/// @param group The group
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: None but the group's own, which the core reports
[[nodiscard]] static inline EranosError *eranosGroupClearLanguage(EranosGroup *const group) {
    EranosGroup_clear_language_result const result = EranosGroup_clear_language(group);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `set_language`, reported to `callback` on the group's worker thread.
///
/// @param group The group
/// @param language The language, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupSetLanguageAsync(EranosGroup *const group, char const *const language, EranosCancellation const *const cancellation,
                                               EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosGroup_set_language_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosGroup_set_language_async(group, eranosDetailView(language), cancellation, wrapped);
}

/// The asynchronous form of `clear_language`, reported to `callback` on the group's worker thread.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupClearLanguageAsync(EranosGroup *const group, EranosCancellation const *const cancellation,
                                                 EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosGroup_clear_language_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosGroup_clear_language_async(group, cancellation, wrapped);
}

/// The group in its saved form, as JSON. Writes it into @p json.
///
/// @param group The group
/// @param json Receives the json, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while a change holds the lock
[[nodiscard]] static inline EranosError *eranosGroupSave(EranosGroup const *const group, EranosText *const json) {
    eranosDetailClear(json);
    EranosGroup_save_result const result = EranosGroup_save(group, json);
    eranosDetailFinish(json);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `save`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupSaveAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                        EranosTextCallback const *const callback) {
    DiplomatCallback_EranosGroup_save_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosGroup_save_async(group, cancellation, wrapped);
}

/// A read handle, which holds the shared lock until it is released or dropped, so that its reads see no change in between.
///
/// @param group The group
/// @param reader Receives the reader on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_GROUP with the
///   failure `GroupBusy` on wasm while a change holds the lock
[[nodiscard]] static inline EranosError *eranosGroupRead(EranosGroup const *const group, EranosGroupReader **const reader) {
    EranosGroup_read_result const result = EranosGroup_read(group);
    *reader = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of `read`: it runs on the group's worker thread, in the order of the calls, and reports its outcome to `callback` there.
/// Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosGroupReadAsync(EranosGroup const *const group, EranosCancellation const *const cancellation,
                                        EranosGroupReaderCallback const *const callback) {
    DiplomatCallback_EranosGroup_read_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunGroupReaderCallback,
        .destructor = nullptr,
    };
    EranosGroup_read_async(group, cancellation, wrapped);
}

/// Destroys @p group; destroying nullptr does nothing, as free() does.
///
/// @param group The group, which no call uses after this one, or nullptr
static inline void eranosGroupDestroy(EranosGroup *const group) {
    if (nullptr != group) {
        EranosGroup_destroy(group);
    }
}

/// What a new group starts with.
typedef struct EranosNewGroup EranosNewGroup;

/// A new new group of its required fields; every other field starts absent, empty, or at its default.
///
/// @param name The group's name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param currency The ISO 4217 code of the group currency, which every total is in, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param newGroup Receives the new group on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewGroupCreate(char const *const name, char const *const currency, EranosNewGroup **const newGroup) {
    EranosNewGroup_create_result const result = EranosNewGroup_create(eranosDetailView(name), eranosDetailView(currency));
    *newGroup = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's name.
///
/// @param newGroup The new group
/// @param name Receives the group's name, replacing what it held
static inline void eranosNewGroupName(EranosNewGroup const *const newGroup, EranosText *const name) {
    eranosDetailClear(name);
    EranosNewGroup_name(newGroup, name);
    eranosDetailFinish(name);
}

/// Sets the group's name.
///
/// @param newGroup The new group
/// @param name The group's name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewGroupSetName(EranosNewGroup *const newGroup, char const *const name) {
    EranosNewGroup_set_name_result const result = EranosNewGroup_set_name(newGroup, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The ISO 4217 code of the group currency, which every total is in.
///
/// @param newGroup The new group
/// @param currency Receives the ISO 4217 code of the group currency, which every total is in, replacing what it held
static inline void eranosNewGroupCurrency(EranosNewGroup const *const newGroup, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosNewGroup_currency(newGroup, currency);
    eranosDetailFinish(currency);
}

/// Sets the ISO 4217 code of the group currency, which every total is in.
///
/// @param newGroup The new group
/// @param currency The ISO 4217 code of the group currency, which every total is in, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewGroupSetCurrency(EranosNewGroup *const newGroup, char const *const currency) {
    EranosNewGroup_set_currency_result const result = EranosNewGroup_set_currency(newGroup, eranosDetailView(currency));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// A longer description. Writes it, if present; absent otherwise.
///
/// @param newGroup The new group
/// @param description Receives a longer description, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosNewGroupDescription(EranosNewGroup const *const newGroup, EranosText *const description) {
    eranosDetailClear(description);
    bool const present = EranosNewGroup_description(newGroup, description).is_ok;
    eranosDetailFinish(description);
    return present;
}

/// Sets a longer description.
///
/// @param newGroup The new group
/// @param description A longer description, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewGroupSetDescription(EranosNewGroup *const newGroup, char const *const description) {
    EranosNewGroup_set_description_result const result = EranosNewGroup_set_description(newGroup, eranosDetailView(description));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears a longer description.
///
/// @param newGroup The new group
static inline void eranosNewGroupClearDescription(EranosNewGroup *const newGroup) { EranosNewGroup_clear_description(newGroup); }

/// The language the apps show the group in, as a BCP 47 tag. Writes it, if present; absent otherwise.
///
/// @param newGroup The new group
/// @param language Receives the language the apps show the group in, as a BCP 47 tag, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosNewGroupLanguage(EranosNewGroup const *const newGroup, EranosText *const language) {
    eranosDetailClear(language);
    bool const present = EranosNewGroup_language(newGroup, language).is_ok;
    eranosDetailFinish(language);
    return present;
}

/// Sets the language the apps show the group in, as a BCP 47 tag.
///
/// @param newGroup The new group
/// @param language The language the apps show the group in, as a BCP 47 tag, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewGroupSetLanguage(EranosNewGroup *const newGroup, char const *const language) {
    EranosNewGroup_set_language_result const result = EranosNewGroup_set_language(newGroup, eranosDetailView(language));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the language the apps show the group in, as a BCP 47 tag.
///
/// @param newGroup The new group
static inline void eranosNewGroupClearLanguage(EranosNewGroup *const newGroup) { EranosNewGroup_clear_language(newGroup); }

/// Destroys @p newGroup; destroying nullptr does nothing, as free() does.
///
/// @param newGroup The new group, which no call uses after this one, or nullptr
static inline void eranosNewGroupDestroy(EranosNewGroup *const newGroup) {
    if (nullptr != newGroup) {
        EranosNewGroup_destroy(newGroup);
    }
}

#endif // ERANOS_GROUP_H
