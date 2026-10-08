/// @file eranos/expense_group.h
/// The group, the details it starts with, the rates of a currency change, the cancellation of asynchronous calls, and the callbacks that report their
/// outcomes.

#ifndef ERANOS_EXPENSE_GROUP_H
#define ERANOS_EXPENSE_GROUP_H

#include "EranosCancellation.h"
#include "EranosCategoryListOutcome.h"
#include "EranosCategoryOutcome.h"
#include "EranosCurrencyRate.h"
#include "EranosCurrencyRateList.h"
#include "EranosExpenseGroup.h"
#include "EranosExpenseGroupOutcome.h"
#include "EranosExpenseGroupReaderOutcome.h"
#include "EranosExpenseListOutcome.h"
#include "EranosExpenseOutcome.h"
#include "EranosFoundCategoryOutcome.h"
#include "EranosFoundExpenseOutcome.h"
#include "EranosFoundExpenseSharesOutcome.h"
#include "EranosFoundFlagOutcome.h"
#include "EranosFoundMemberOutcome.h"
#include "EranosFoundMoneyOutcome.h"
#include "EranosFoundTransferOutcome.h"
#include "EranosMemberListOutcome.h"
#include "EranosMemberOutcome.h"
#include "EranosNewExpenseGroup.h"
#include "EranosOptionalTextOutcome.h"
#include "EranosSettlementPlanOutcome.h"
#include "EranosSummaryOutcome.h"
#include "EranosTextOutcome.h"
#include "EranosTransferListOutcome.h"
#include "EranosTransferOutcome.h"
#include "EranosUnitOutcome.h"
#include "eranos/category.h"
#include "eranos/detail.h"
#include "eranos/expense.h"
#include "eranos/expense_group_reader.h"
#include "eranos/identifiers.h"
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

/// What an asynchronous lookup reports to when it ends: the error, or nullptr when the call succeeded, and the category, or nullptr when the call
/// failed or found none: nullptr with no error means that no category has the identifier. The callback destroys each object it receives.
typedef struct EranosFoundCategoryCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosCategory *category);
} EranosFoundCategoryCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFoundCategoryCallback(void const *const data, EranosFoundCategoryOutcome *const outcome) {
    EranosFoundCategoryCallback const *const callback = data;
    EranosError *const error = EranosFoundCategoryOutcome_error(outcome);
    EranosCategory *const category = EranosFoundCategoryOutcome_value(outcome);
    EranosFoundCategoryOutcome_destroy(outcome);
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

/// What an asynchronous lookup reports to when it ends: the error, or nullptr when the call succeeded, and the expense, or nullptr when the call
/// failed or found none: nullptr with no error means that no expense has the identifier. The callback destroys each object it receives.
typedef struct EranosFoundExpenseCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosExpense *expense);
} EranosFoundExpenseCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFoundExpenseCallback(void const *const data, EranosFoundExpenseOutcome *const outcome) {
    EranosFoundExpenseCallback const *const callback = data;
    EranosError *const error = EranosFoundExpenseOutcome_error(outcome);
    EranosExpense *const expense = EranosFoundExpenseOutcome_value(outcome);
    EranosFoundExpenseOutcome_destroy(outcome);
    callback->run(callback->context, error, expense);
}

/// What an asynchronous lookup reports to when it ends: the error, or nullptr when the call succeeded, and the expense's shares, or nullptr when the
/// call failed or found none: nullptr with no error means that no expense has the identifier. The callback destroys each object it receives.
typedef struct EranosFoundExpenseSharesCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosExpenseShares *expenseShares);
} EranosFoundExpenseSharesCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFoundExpenseSharesCallback(void const *const data, EranosFoundExpenseSharesOutcome *const outcome) {
    EranosFoundExpenseSharesCallback const *const callback = data;
    EranosError *const error = EranosFoundExpenseSharesOutcome_error(outcome);
    EranosExpenseShares *const expenseShares = EranosFoundExpenseSharesOutcome_value(outcome);
    EranosFoundExpenseSharesOutcome_destroy(outcome);
    callback->run(callback->context, error, expenseShares);
}

/// What an asynchronous lookup reports to when it ends: the error, or nullptr when the call succeeded, and whether the member is archived, or nullptr
/// when the call failed or found none, which points to a value that stays valid until the function returns: nullptr with no error means that no
/// member has the identifier. The callback destroys each object it receives.
typedef struct EranosFoundFlagCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, bool const *archived);
} EranosFoundFlagCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFoundFlagCallback(void const *const data, EranosFoundFlagOutcome *const outcome) {
    EranosFoundFlagCallback const *const callback = data;
    EranosError *const error = EranosFoundFlagOutcome_error(outcome);
    EranosFoundFlagOutcome_value_result const archived = EranosFoundFlagOutcome_value(outcome);
    EranosFoundFlagOutcome_destroy(outcome);
    callback->run(callback->context, error, archived.is_ok ? &archived.ok : nullptr);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the group, or nullptr when the call failed.
/// The callback destroys each object it receives.
typedef struct EranosExpenseGroupCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosExpenseGroup *group);
} EranosExpenseGroupCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunExpenseGroupCallback(void const *const data, EranosExpenseGroupOutcome *const outcome) {
    EranosExpenseGroupCallback const *const callback = data;
    EranosError *const error = EranosExpenseGroupOutcome_error(outcome);
    EranosExpenseGroup *const group = EranosExpenseGroupOutcome_take(outcome);
    EranosExpenseGroupOutcome_destroy(outcome);
    callback->run(callback->context, error, group);
}

/// What an asynchronous call reports to when it ends: the error, or nullptr when the call succeeded, and the reader, or nullptr when the call failed.
/// The callback destroys each object it receives.
typedef struct EranosExpenseGroupReaderCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosExpenseGroupReader *reader);
} EranosExpenseGroupReaderCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunExpenseGroupReaderCallback(void const *const data, EranosExpenseGroupReaderOutcome *const outcome) {
    EranosExpenseGroupReaderCallback const *const callback = data;
    EranosError *const error = EranosExpenseGroupReaderOutcome_error(outcome);
    EranosExpenseGroupReader *const reader = EranosExpenseGroupReaderOutcome_take(outcome);
    EranosExpenseGroupReaderOutcome_destroy(outcome);
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

/// What an asynchronous lookup reports to when it ends: the error, or nullptr when the call succeeded, and the member, or nullptr when the call
/// failed or found none: nullptr with no error means that no member has the identifier. The callback destroys each object it receives.
typedef struct EranosFoundMemberCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosMember *member);
} EranosFoundMemberCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFoundMemberCallback(void const *const data, EranosFoundMemberOutcome *const outcome) {
    EranosFoundMemberCallback const *const callback = data;
    EranosError *const error = EranosFoundMemberOutcome_error(outcome);
    EranosMember *const member = EranosFoundMemberOutcome_value(outcome);
    EranosFoundMemberOutcome_destroy(outcome);
    callback->run(callback->context, error, member);
}

/// What an asynchronous lookup reports to when it ends: the error, or nullptr when the call succeeded, and the amount, or nullptr when the call
/// failed or found none: nullptr with no error means that no transfer has the identifier. The callback destroys each object it receives.
typedef struct EranosFoundMoneyCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosMoney *money);
} EranosFoundMoneyCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFoundMoneyCallback(void const *const data, EranosFoundMoneyOutcome *const outcome) {
    EranosFoundMoneyCallback const *const callback = data;
    EranosError *const error = EranosFoundMoneyOutcome_error(outcome);
    EranosMoney *const money = EranosFoundMoneyOutcome_value(outcome);
    EranosFoundMoneyOutcome_destroy(outcome);
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
typedef struct EranosSettlementPlanCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosSettlementPlan *settlement);
} EranosSettlementPlanCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunSettlementPlanCallback(void const *const data, EranosSettlementPlanOutcome *const outcome) {
    EranosSettlementPlanCallback const *const callback = data;
    EranosError *const error = EranosSettlementPlanOutcome_error(outcome);
    EranosSettlementPlan *const settlement = EranosSettlementPlanOutcome_value(outcome);
    EranosSettlementPlanOutcome_destroy(outcome);
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

/// What an asynchronous lookup reports to when it ends: the error, or nullptr when the call succeeded, and the transfer, or nullptr when the call
/// failed or found none: nullptr with no error means that no transfer has the identifier. The callback destroys each object it receives.
typedef struct EranosFoundTransferCallback {
    /// What the caller passes back to itself.
    void *context;
    /// Called once with @p context and the outcome.
    void (*run)(void *context, EranosError *error, EranosTransfer *transfer);
} EranosFoundTransferCallback;

/// Reports @p outcome to the callback @p data points to, and destroys it.
static inline void eranosDetailRunFoundTransferCallback(void const *const data, EranosFoundTransferOutcome *const outcome) {
    EranosFoundTransferCallback const *const callback = data;
    EranosError *const error = EranosFoundTransferOutcome_error(outcome);
    EranosTransfer *const transfer = EranosFoundTransferOutcome_value(outcome);
    EranosFoundTransferOutcome_destroy(outcome);
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
///
/// A synchronous call that takes nothing the library could refuse, such as eranosExpenseGroupName() or eranosExpenseGroupSummary(), returns what
/// it reads and reports no error: it waits for the lock, and only the WebAssembly build, for which the C API is not built, would fail instead.
typedef struct EranosExpenseGroup EranosExpenseGroup;

/// A new group, with the details `details` holds, under the identifier they give.
///
/// @param details The details
/// @param group Receives the group on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_ID for an
///   identifier the library refuses, of the kind ERANOS_ERROR_KIND_MONEY for a currency the library does not list, of the kind
///   ERANOS_ERROR_KIND_EXPENSE_GROUP for a blank name, and of the kind ERANOS_ERROR_KIND_LANGUAGE for a language that is not a tag
[[nodiscard]] static inline EranosError *eranosExpenseGroupCreate(EranosNewExpenseGroup const *const details, EranosExpenseGroup **const group) {
    EranosExpenseGroup_create_result const result = EranosExpenseGroup_create(details);
    *group = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group saved as `json`, which eranosExpenseGroupSave() or eranosExpenseGroupReaderSave() wrote.
///
/// @param json The json, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param group Receives the group on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_JSON for
///   text that is not a saved group, of the kind ERANOS_ERROR_KIND_EXPENSE_GROUP for a format this release cannot read, of the module whose rule an
///   entry breaks, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupLoad(char const *const json, EranosExpenseGroup **const group) {
    EranosExpenseGroup_load_result const result = EranosExpenseGroup_load(eranosDetailView(json));
    *group = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupLoad(): it runs on a thread of its own and reports its outcome to `callback` there.
///
/// @param json The json, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupLoadAsync(char const *const json, EranosCancellation const *const cancellation,
                                               EranosExpenseGroupCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_load_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseGroupCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_load_async(eranosDetailView(json), cancellation, wrapped);
}

/// The group's identifier. Writes it into @p id.
///
/// @param group The group
/// @param id Receives the id, replacing what it held
static inline void eranosExpenseGroupId(EranosExpenseGroup const *const group, EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroup_id(group, id);
    eranosDetailFinish(id);
}

/// The asynchronous form of eranosExpenseGroupId(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupIdAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                             EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_id_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_id_async(group, cancellation, wrapped);
}

/// Every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers.
///
/// @param group The group
/// @return The categories, which the caller destroys
[[nodiscard]] static inline EranosCategoryList *eranosExpenseGroupCategories(EranosExpenseGroup const *const group) {
    return EranosExpenseGroup_categories(group);
}

/// The asynchronous form of eranosExpenseGroupCategories(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupCategoriesAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                     EranosCategoryListCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_categories_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunCategoryListCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_categories_async(group, cancellation, wrapped);
}

/// The category with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param category Receives the category, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no category has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupCategory(EranosExpenseGroup const *const group, char const *const id,
                                                                    EranosCategory **const category) {
    EranosExpenseGroup_category_result const result = EranosExpenseGroup_category(group, eranosDetailView(id));
    *category = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupCategory(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupCategoryAsync(EranosExpenseGroup const *const group, char const *const id,
                                                   EranosCancellation const *const cancellation, EranosFoundCategoryCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFoundCategoryCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_category_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `category` under its identifier, and returns the identifier. Writes it into @p id.
///
/// @param group The group
/// @param category The category
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupAddCategory(EranosExpenseGroup *const group, EranosCategory const *const category,
                                                                       EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroup_add_category_result const result = EranosExpenseGroup_add_category(group, category, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupAddCategory(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param category The category
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupAddCategoryAsync(EranosExpenseGroup *const group, EranosCategory const *const category,
                                                      EranosCancellation const *const cancellation, EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_add_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_add_category_async(group, category, cancellation, wrapped);
}

/// Renames the category that `category` names by its identifier, and returns it as it was.
///
/// @param group The group
/// @param category The category
/// @param previous Receives the category as it was before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupReplaceCategory(EranosExpenseGroup *const group, EranosCategory const *const category,
                                                                           EranosCategory **const previous) {
    EranosExpenseGroup_replace_category_result const result = EranosExpenseGroup_replace_category(group, category);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupReplaceCategory(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param category The category
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupReplaceCategoryAsync(EranosExpenseGroup *const group, EranosCategory const *const category,
                                                          EranosCancellation const *const cancellation,
                                                          EranosCategoryCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_replace_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunCategoryCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_replace_category_async(group, category, cancellation, wrapped);
}

/// Removes the category with the identifier `id` and returns it; its expenses move to the built-in one.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the category the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupRemoveCategory(EranosExpenseGroup *const group, char const *const id,
                                                                          EranosCategory **const removed) {
    EranosExpenseGroup_remove_category_result const result = EranosExpenseGroup_remove_category(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupRemoveCategory(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupRemoveCategoryAsync(EranosExpenseGroup *const group, char const *const id,
                                                         EranosCancellation const *const cancellation, EranosCategoryCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_remove_category_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunCategoryCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_remove_category_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The ISO 4217 code of the group currency. Writes it into @p currency.
///
/// @param group The group
/// @param currency Receives the currency, replacing what it held
static inline void eranosExpenseGroupCurrency(EranosExpenseGroup const *const group, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosExpenseGroup_currency(group, currency);
    eranosDetailFinish(currency);
}

/// The asynchronous form of eranosExpenseGroupCurrency(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupCurrencyAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                   EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_currency_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_currency_async(group, cancellation, wrapped);
}

/// Moves the group to `currency`, with a rate into it for every other currency its entries use.
///
/// @param group The group
/// @param currency The currency, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param rates The rates
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupChangeCurrency(EranosExpenseGroup *const group, char const *const currency,
                                                                          EranosCurrencyRateList const *const rates) {
    EranosExpenseGroup_change_currency_result const result = EranosExpenseGroup_change_currency(group, eranosDetailView(currency), rates);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupChangeCurrency(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param currency The currency, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param rates The rates
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupChangeCurrencyAsync(EranosExpenseGroup *const group, char const *const currency,
                                                         EranosCurrencyRateList const *const rates, EranosCancellation const *const cancellation,
                                                         EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_change_currency_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_change_currency_async(group, eranosDetailView(currency), rates, cancellation, wrapped);
}

/// Every expense, in identifier order, which is creation order for generated identifiers.
///
/// @param group The group
/// @return The expenses, which the caller destroys
[[nodiscard]] static inline EranosExpenseList *eranosExpenseGroupExpenses(EranosExpenseGroup const *const group) {
    return EranosExpenseGroup_expenses(group);
}

/// The asynchronous form of eranosExpenseGroupExpenses(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupExpensesAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                   EranosExpenseListCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_expenses_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseListCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_expenses_async(group, cancellation, wrapped);
}

/// The expense with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expense Receives the expense, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no expense has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupExpense(EranosExpenseGroup const *const group, char const *const id,
                                                                   EranosExpense **const expense) {
    EranosExpenseGroup_expense_result const result = EranosExpenseGroup_expense(group, eranosDetailView(id));
    *expense = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupExpense(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupExpenseAsync(EranosExpenseGroup const *const group, char const *const id,
                                                  EranosCancellation const *const cancellation, EranosFoundExpenseCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFoundExpenseCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_expense_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// How the expense with the identifier `id` divides, in the group currency.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expenseShares Receives the expense shares, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error
///   means that no expense has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupExpenseShares(EranosExpenseGroup const *const group, char const *const id,
                                                                         EranosExpenseShares **const expenseShares) {
    EranosExpenseGroup_expense_shares_result const result = EranosExpenseGroup_expense_shares(group, eranosDetailView(id));
    *expenseShares = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupExpenseShares(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupExpenseSharesAsync(EranosExpenseGroup const *const group, char const *const id,
                                                        EranosCancellation const *const cancellation,
                                                        EranosFoundExpenseSharesCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_expense_shares_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFoundExpenseSharesCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_expense_shares_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `expense` under its identifier, and returns the identifier. Writes it into @p id.
///
/// @param group The group
/// @param expense The expense
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupAddExpense(EranosExpenseGroup *const group, EranosExpense const *const expense,
                                                                      EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroup_add_expense_result const result = EranosExpenseGroup_add_expense(group, expense, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupAddExpense(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param expense The expense
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupAddExpenseAsync(EranosExpenseGroup *const group, EranosExpense const *const expense,
                                                     EranosCancellation const *const cancellation, EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_add_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_add_expense_async(group, expense, cancellation, wrapped);
}

/// Replaces the expense that `expense` names by its identifier, and returns it as it was.
///
/// @param group The group
/// @param expense The expense
/// @param previous Receives the expense as it was before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupReplaceExpense(EranosExpenseGroup *const group, EranosExpense const *const expense,
                                                                          EranosExpense **const previous) {
    EranosExpenseGroup_replace_expense_result const result = EranosExpenseGroup_replace_expense(group, expense);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupReplaceExpense(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param expense The expense
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupReplaceExpenseAsync(EranosExpenseGroup *const group, EranosExpense const *const expense,
                                                         EranosCancellation const *const cancellation, EranosExpenseCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_replace_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_replace_expense_async(group, expense, cancellation, wrapped);
}

/// Removes the expense with the identifier `id` and returns it.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the expense the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupRemoveExpense(EranosExpenseGroup *const group, char const *const id,
                                                                         EranosExpense **const removed) {
    EranosExpenseGroup_remove_expense_result const result = EranosExpenseGroup_remove_expense(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupRemoveExpense(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupRemoveExpenseAsync(EranosExpenseGroup *const group, char const *const id,
                                                        EranosCancellation const *const cancellation, EranosExpenseCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_remove_expense_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_remove_expense_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The group's totals.
///
/// @param group The group
/// @return The summary, which the caller destroys
[[nodiscard]] static inline EranosSummary *eranosExpenseGroupSummary(EranosExpenseGroup const *const group) {
    return EranosExpenseGroup_summary(group);
}

/// The asynchronous form of eranosExpenseGroupSummary(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupSummaryAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                  EranosSummaryCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_summary_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunSummaryCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_summary_async(group, cancellation, wrapped);
}

/// Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
///
/// @param group The group
/// @return The members, which the caller destroys
[[nodiscard]] static inline EranosMemberList *eranosExpenseGroupMembers(EranosExpenseGroup const *const group) {
    return EranosExpenseGroup_members(group);
}

/// The asynchronous form of eranosExpenseGroupMembers(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupMembersAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                  EranosMemberListCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_members_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMemberListCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_members_async(group, cancellation, wrapped);
}

/// The member with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param member Receives the member, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no member has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupMember(EranosExpenseGroup const *const group, char const *const id,
                                                                  EranosMember **const member) {
    EranosExpenseGroup_member_result const result = EranosExpenseGroup_member(group, eranosDetailView(id));
    *member = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupMember(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupMemberAsync(EranosExpenseGroup const *const group, char const *const id,
                                                 EranosCancellation const *const cancellation, EranosFoundMemberCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFoundMemberCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Whether the member with the identifier `id` is archived. Writes it into @p archived when the group holds the member, and nothing otherwise:
/// @p found false with no error means that no member has the identifier.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param found Receives whether the group holds the member, which is false when the call fails
/// @param archived Receives whether the member is archived, when the group holds them
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupIsArchived(EranosExpenseGroup const *const group, char const *const id, bool *const found,
                                                                      bool *const archived) {
    EranosExpenseGroup_is_archived_result const result = EranosExpenseGroup_is_archived(group, eranosDetailView(id));
    *found = result.is_ok && eranosDetailPresentBool(result.ok.is_ok, &result.ok.ok, archived);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupIsArchived(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupIsArchivedAsync(EranosExpenseGroup const *const group, char const *const id,
                                                     EranosCancellation const *const cancellation, EranosFoundFlagCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_is_archived_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFoundFlagCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_is_archived_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `member` under their identifier, and returns the identifier. Writes it into @p id.
///
/// @param group The group
/// @param member The member
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupAddMember(EranosExpenseGroup *const group, EranosMember const *const member,
                                                                     EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroup_add_member_result const result = EranosExpenseGroup_add_member(group, member, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupAddMember(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param member The member
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupAddMemberAsync(EranosExpenseGroup *const group, EranosMember const *const member,
                                                    EranosCancellation const *const cancellation, EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_add_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_add_member_async(group, member, cancellation, wrapped);
}

/// Replaces the member that `member` names by their identifier, and returns them as they were.
///
/// @param group The group
/// @param member The member
/// @param previous Receives the member as they were before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupReplaceMember(EranosExpenseGroup *const group, EranosMember const *const member,
                                                                         EranosMember **const previous) {
    EranosExpenseGroup_replace_member_result const result = EranosExpenseGroup_replace_member(group, member);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupReplaceMember(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param member The member
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupReplaceMemberAsync(EranosExpenseGroup *const group, EranosMember const *const member,
                                                        EranosCancellation const *const cancellation, EranosMemberCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_replace_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMemberCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_replace_member_async(group, member, cancellation, wrapped);
}

/// Archives the member with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupArchiveMember(EranosExpenseGroup *const group, char const *const id) {
    EranosExpenseGroup_archive_member_result const result = EranosExpenseGroup_archive_member(group, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupArchiveMember(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupArchiveMemberAsync(EranosExpenseGroup *const group, char const *const id,
                                                        EranosCancellation const *const cancellation, EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_archive_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_archive_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Unarchives the member with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupUnarchiveMember(EranosExpenseGroup *const group, char const *const id) {
    EranosExpenseGroup_unarchive_member_result const result = EranosExpenseGroup_unarchive_member(group, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupUnarchiveMember(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupUnarchiveMemberAsync(EranosExpenseGroup *const group, char const *const id,
                                                          EranosCancellation const *const cancellation, EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_unarchive_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_unarchive_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Removes the member with the identifier `id`, whom no entry references, and returns them.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the member the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupRemoveMember(EranosExpenseGroup *const group, char const *const id,
                                                                        EranosMember **const removed) {
    EranosExpenseGroup_remove_member_result const result = EranosExpenseGroup_remove_member(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupRemoveMember(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupRemoveMemberAsync(EranosExpenseGroup *const group, char const *const id,
                                                       EranosCancellation const *const cancellation, EranosMemberCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_remove_member_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunMemberCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_remove_member_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The group's name. Writes it into @p name.
///
/// @param group The group
/// @param name Receives the name, replacing what it held
static inline void eranosExpenseGroupName(EranosExpenseGroup const *const group, EranosText *const name) {
    eranosDetailClear(name);
    EranosExpenseGroup_name(group, name);
    eranosDetailFinish(name);
}

/// The asynchronous form of eranosExpenseGroupName(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupNameAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                               EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_name_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_name_async(group, cancellation, wrapped);
}

/// Renames the group.
///
/// @param group The group
/// @param name The name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupSetName(EranosExpenseGroup *const group, char const *const name) {
    EranosExpenseGroup_set_name_result const result = EranosExpenseGroup_set_name(group, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupSetName(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param name The name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupSetNameAsync(EranosExpenseGroup *const group, char const *const name,
                                                  EranosCancellation const *const cancellation, EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_set_name_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_set_name_async(group, eranosDetailView(name), cancellation, wrapped);
}

/// The payments that settle every balance by `strategy`.
///
/// @param group The group
/// @param strategy The strategy
/// @param settlement Receives the settlement on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupSettlement(EranosExpenseGroup const *const group, EranosSettlementStrategy const strategy,
                                                                      EranosSettlementPlan **const settlement) {
    EranosExpenseGroup_settlement_result const result = EranosExpenseGroup_settlement(group, strategy);
    *settlement = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupSettlement(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param strategy The strategy
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupSettlementAsync(EranosExpenseGroup const *const group, EranosSettlementStrategy const strategy,
                                                     EranosCancellation const *const cancellation,
                                                     EranosSettlementPlanCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_settlement_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunSettlementPlanCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_settlement_async(group, strategy, cancellation, wrapped);
}

/// Every transfer, in identifier order, which is creation order for generated identifiers.
///
/// @param group The group
/// @return The transfers, which the caller destroys
[[nodiscard]] static inline EranosTransferList *eranosExpenseGroupTransfers(EranosExpenseGroup const *const group) {
    return EranosExpenseGroup_transfers(group);
}

/// The asynchronous form of eranosExpenseGroupTransfers(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupTransfersAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                    EranosTransferListCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_transfers_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTransferListCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_transfers_async(group, cancellation, wrapped);
}

/// The transfer with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transfer Receives the transfer, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no transfer has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupTransfer(EranosExpenseGroup const *const group, char const *const id,
                                                                    EranosTransfer **const transfer) {
    EranosExpenseGroup_transfer_result const result = EranosExpenseGroup_transfer(group, eranosDetailView(id));
    *transfer = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupTransfer(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupTransferAsync(EranosExpenseGroup const *const group, char const *const id,
                                                   EranosCancellation const *const cancellation, EranosFoundTransferCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFoundTransferCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_transfer_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The amount in the group currency of the transfer with the identifier `id`.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transferAmount Receives the transfer amount, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error
///   means that no transfer has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupTransferAmount(EranosExpenseGroup const *const group, char const *const id,
                                                                          EranosMoney **const transferAmount) {
    EranosExpenseGroup_transfer_amount_result const result = EranosExpenseGroup_transfer_amount(group, eranosDetailView(id));
    *transferAmount = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupTransferAmount(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupTransferAmountAsync(EranosExpenseGroup const *const group, char const *const id,
                                                         EranosCancellation const *const cancellation,
                                                         EranosFoundMoneyCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_transfer_amount_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunFoundMoneyCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_transfer_amount_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// Adds `transfer` under its identifier, and returns the identifier. Writes it into @p id.
///
/// @param group The group
/// @param transfer The transfer
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupAddTransfer(EranosExpenseGroup *const group, EranosTransfer const *const transfer,
                                                                       EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroup_add_transfer_result const result = EranosExpenseGroup_add_transfer(group, transfer, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupAddTransfer(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param transfer The transfer
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupAddTransferAsync(EranosExpenseGroup *const group, EranosTransfer const *const transfer,
                                                      EranosCancellation const *const cancellation, EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_add_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_add_transfer_async(group, transfer, cancellation, wrapped);
}

/// Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
///
/// @param group The group
/// @param transfer The transfer
/// @param previous Receives the transfer as it was before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call
[[nodiscard]] static inline EranosError *eranosExpenseGroupReplaceTransfer(EranosExpenseGroup *const group, EranosTransfer const *const transfer,
                                                                           EranosTransfer **const previous) {
    EranosExpenseGroup_replace_transfer_result const result = EranosExpenseGroup_replace_transfer(group, transfer);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupReplaceTransfer(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param transfer The transfer
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupReplaceTransferAsync(EranosExpenseGroup *const group, EranosTransfer const *const transfer,
                                                          EranosCancellation const *const cancellation,
                                                          EranosTransferCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_replace_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTransferCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_replace_transfer_async(group, transfer, cancellation, wrapped);
}

/// Removes the transfer with the identifier `id` and returns it.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the transfer the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
///   that refused the call, or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupRemoveTransfer(EranosExpenseGroup *const group, char const *const id,
                                                                          EranosTransfer **const removed) {
    EranosExpenseGroup_remove_transfer_result const result = EranosExpenseGroup_remove_transfer(group, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The asynchronous form of eranosExpenseGroupRemoveTransfer(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupRemoveTransferAsync(EranosExpenseGroup *const group, char const *const id,
                                                         EranosCancellation const *const cancellation, EranosTransferCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_remove_transfer_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTransferCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_remove_transfer_async(group, eranosDetailView(id), cancellation, wrapped);
}

/// The group's description. Writes it into @p description, if the group has one, and nothing otherwise: the group holds no blank text there, since
/// blank text clears it.
///
/// @param group The group
/// @param description Receives the description, replacing what it held
/// @return Whether the group has one
[[nodiscard]] static inline bool eranosExpenseGroupDescription(EranosExpenseGroup const *const group, EranosText *const description) {
    eranosDetailClear(description);
    bool const present = EranosExpenseGroup_description(group, description).is_ok;
    eranosDetailFinish(description);
    return present;
}

/// The asynchronous form of eranosExpenseGroupDescription(): it runs on the group's worker thread, in the order of the calls, and reports its outcome
/// to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupDescriptionAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                      EranosOptionalTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_description_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunOptionalTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_description_async(group, cancellation, wrapped);
}

/// Sets the description, trimmed, or clears it when the text is blank.
///
/// @param group The group
/// @param description The description, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupSetDescription(EranosExpenseGroup *const group, char const *const description) {
    EranosExpenseGroup_set_description_result const result = EranosExpenseGroup_set_description(group, eranosDetailView(description));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the group's description.
///
/// @param group The group
static inline void eranosExpenseGroupClearDescription(EranosExpenseGroup *const group) { EranosExpenseGroup_clear_description(group); }

/// The asynchronous form of eranosExpenseGroupSetDescription(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param description The description, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupSetDescriptionAsync(EranosExpenseGroup *const group, char const *const description,
                                                         EranosCancellation const *const cancellation, EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_set_description_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_set_description_async(group, eranosDetailView(description), cancellation, wrapped);
}

/// The asynchronous form of eranosExpenseGroupClearDescription(): it runs on the group's worker thread, in the order of the calls, and reports its
/// outcome to `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the
/// kind ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupClearDescriptionAsync(EranosExpenseGroup *const group, EranosCancellation const *const cancellation,
                                                           EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_clear_description_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_clear_description_async(group, cancellation, wrapped);
}

/// The language the apps show the group in, as a BCP 47 tag. Writes it into @p language, if the group has one, and nothing otherwise: the group holds
/// no blank text there, since blank text clears it.
///
/// @param group The group
/// @param language Receives the language, replacing what it held
/// @return Whether the group has one
[[nodiscard]] static inline bool eranosExpenseGroupLanguage(EranosExpenseGroup const *const group, EranosText *const language) {
    eranosDetailClear(language);
    bool const present = EranosExpenseGroup_language(group, language).is_ok;
    eranosDetailFinish(language);
    return present;
}

/// The asynchronous form of eranosExpenseGroupLanguage(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupLanguageAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                                   EranosOptionalTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_language_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunOptionalTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_language_async(group, cancellation, wrapped);
}

/// Sets the language the apps show the group in, a BCP 47 tag.
///
/// @param group The group
/// @param language The language, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_LANGUAGE for text
///   that is not a language tag, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupSetLanguage(EranosExpenseGroup *const group, char const *const language) {
    EranosExpenseGroup_set_language_result const result = EranosExpenseGroup_set_language(group, eranosDetailView(language));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the language the apps show the group in.
///
/// @param group The group
static inline void eranosExpenseGroupClearLanguage(EranosExpenseGroup *const group) { (void)EranosExpenseGroup_clear_language(group); }

/// The asynchronous form of eranosExpenseGroupSetLanguage(), reported to `callback` on the group's worker thread.
///
/// @param group The group
/// @param language The language, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupSetLanguageAsync(EranosExpenseGroup *const group, char const *const language,
                                                      EranosCancellation const *const cancellation, EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_set_language_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_set_language_async(group, eranosDetailView(language), cancellation, wrapped);
}

/// The asynchronous form of eranosExpenseGroupClearLanguage(), reported to `callback` on the group's worker thread.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupClearLanguageAsync(EranosExpenseGroup *const group, EranosCancellation const *const cancellation,
                                                        EranosUnitCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_clear_language_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunUnitCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_clear_language_async(group, cancellation, wrapped);
}

/// The group in its saved form, as JSON. Writes it into @p json.
///
/// @param group The group
/// @param json Receives the json, replacing what it held
static inline void eranosExpenseGroupSave(EranosExpenseGroup const *const group, EranosText *const json) {
    eranosDetailClear(json);
    EranosExpenseGroup_save(group, json);
    eranosDetailFinish(json);
}

/// The asynchronous form of eranosExpenseGroupSave(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupSaveAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                               EranosTextCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_save_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunTextCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_save_async(group, cancellation, wrapped);
}

/// A read handle, which holds the shared lock until it is released or destroyed, so that its reads see no change in between.
///
/// @param group The group
/// @return The reader, which the caller destroys
[[nodiscard]] static inline EranosExpenseGroupReader *eranosExpenseGroupRead(EranosExpenseGroup const *const group) {
    return EranosExpenseGroup_read(group);
}

/// The asynchronous form of eranosExpenseGroupRead(): it runs on the group's worker thread, in the order of the calls, and reports its outcome to
/// `callback` there. Cancelling `cancellation` stops the call at its next yield, or at once while it waits for the lock, and reports the kind
/// ERANOS_ERROR_KIND_CANCELLED, leaving the group as it was.
///
/// @param group The group
/// @param cancellation A cancellation for the call, or nullptr for none
/// @param callback What to call with the outcome, on the group's worker thread or, for a load, a thread of its own; it and its context stay valid
///   until it is called, which happens once
static inline void eranosExpenseGroupReadAsync(EranosExpenseGroup const *const group, EranosCancellation const *const cancellation,
                                               EranosExpenseGroupReaderCallback const *const callback) {
    DiplomatCallback_EranosExpenseGroup_read_async_callback const wrapped = {
        .data = callback,
        .run_callback = eranosDetailRunExpenseGroupReaderCallback,
        .destructor = nullptr,
    };
    EranosExpenseGroup_read_async(group, cancellation, wrapped);
}

/// Destroys @p group; destroying nullptr does nothing, as free() does.
///
/// @param group The group, which no call uses after this one, or nullptr
static inline void eranosExpenseGroupDestroy(EranosExpenseGroup *const group) {
    if (nullptr != group) {
        EranosExpenseGroup_destroy(group);
    }
}

/// What a new group starts with.
typedef struct EranosNewExpenseGroup EranosNewExpenseGroup;

/// The details of a new group, made of their required fields, under the identifier @p id, such as the key an app's own store gives the group; every
/// other field starts absent, empty, or at its default. eranosNewExpenseGroupCreate() generates the identifier instead.
///
/// @param id The identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes, as UTF-8 ended by a NUL; nullptr reads as empty
///   text
/// @param name The group's name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param currency The ISO 4217 code of the group currency, which every total is in, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param newGroup Receives the new group on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewExpenseGroupCreateWithId(char const *const id, char const *const name, char const *const currency,
                                                                           EranosNewExpenseGroup **const newGroup) {
    EranosNewExpenseGroup_create_result const result =
        EranosNewExpenseGroup_create(eranosDetailView(id), eranosDetailView(name), eranosDetailView(currency));
    *newGroup = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The details of a new group, made of their required fields, under an identifier generated when it is called, which
/// eranosNewExpenseGroupCreateWithId() takes instead; every other field starts absent, empty, or at its default.
///
/// @param name The group's name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param currency The ISO 4217 code of the group currency, which every total is in, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param newGroup Receives the new group on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewExpenseGroupCreate(char const *const name, char const *const currency,
                                                                     EranosNewExpenseGroup **const newGroup) {
    EranosText *const id = eranosTextCreate();
    eranosIdentifierGenerate(id);
    EranosError *const error = eranosNewExpenseGroupCreateWithId(eranosTextData(id), name, currency, newGroup);
    eranosTextDestroy(id);
    return error;
}

/// The identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes.
///
/// @param newGroup The new group
/// @param id Receives the identifier, replacing what it held
static inline void eranosNewExpenseGroupId(EranosNewExpenseGroup const *const newGroup, EranosText *const id) {
    eranosDetailClear(id);
    EranosNewExpenseGroup_id(newGroup, id);
    eranosDetailFinish(id);
}

/// Sets the identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes.
///
/// @param newGroup The new group
/// @param id The identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewExpenseGroupSetId(EranosNewExpenseGroup *const newGroup, char const *const id) {
    EranosNewExpenseGroup_set_id_result const result = EranosNewExpenseGroup_set_id(newGroup, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's name.
///
/// @param newGroup The new group
/// @param name Receives the group's name, replacing what it held
static inline void eranosNewExpenseGroupName(EranosNewExpenseGroup const *const newGroup, EranosText *const name) {
    eranosDetailClear(name);
    EranosNewExpenseGroup_name(newGroup, name);
    eranosDetailFinish(name);
}

/// Sets the group's name.
///
/// @param newGroup The new group
/// @param name The group's name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewExpenseGroupSetName(EranosNewExpenseGroup *const newGroup, char const *const name) {
    EranosNewExpenseGroup_set_name_result const result = EranosNewExpenseGroup_set_name(newGroup, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The ISO 4217 code of the group currency, which every total is in.
///
/// @param newGroup The new group
/// @param currency Receives the ISO 4217 code of the group currency, which every total is in, replacing what it held
static inline void eranosNewExpenseGroupCurrency(EranosNewExpenseGroup const *const newGroup, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosNewExpenseGroup_currency(newGroup, currency);
    eranosDetailFinish(currency);
}

/// Sets the ISO 4217 code of the group currency, which every total is in.
///
/// @param newGroup The new group
/// @param currency The ISO 4217 code of the group currency, which every total is in, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewExpenseGroupSetCurrency(EranosNewExpenseGroup *const newGroup, char const *const currency) {
    EranosNewExpenseGroup_set_currency_result const result = EranosNewExpenseGroup_set_currency(newGroup, eranosDetailView(currency));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// A longer description. Writes it, if present; absent otherwise.
///
/// @param newGroup The new group
/// @param description Receives a longer description, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosNewExpenseGroupDescription(EranosNewExpenseGroup const *const newGroup, EranosText *const description) {
    eranosDetailClear(description);
    bool const present = EranosNewExpenseGroup_description(newGroup, description).is_ok;
    eranosDetailFinish(description);
    return present;
}

/// Sets a longer description.
///
/// @param newGroup The new group
/// @param description A longer description, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewExpenseGroupSetDescription(EranosNewExpenseGroup *const newGroup, char const *const description) {
    EranosNewExpenseGroup_set_description_result const result = EranosNewExpenseGroup_set_description(newGroup, eranosDetailView(description));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears a longer description.
///
/// @param newGroup The new group
static inline void eranosNewExpenseGroupClearDescription(EranosNewExpenseGroup *const newGroup) { EranosNewExpenseGroup_clear_description(newGroup); }

/// The language the apps show the group in, as a BCP 47 tag. Writes it, if present; absent otherwise.
///
/// @param newGroup The new group
/// @param language Receives the language the apps show the group in, as a BCP 47 tag, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosNewExpenseGroupLanguage(EranosNewExpenseGroup const *const newGroup, EranosText *const language) {
    eranosDetailClear(language);
    bool const present = EranosNewExpenseGroup_language(newGroup, language).is_ok;
    eranosDetailFinish(language);
    return present;
}

/// Sets the language the apps show the group in, as a BCP 47 tag.
///
/// @param newGroup The new group
/// @param language The language the apps show the group in, as a BCP 47 tag, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosNewExpenseGroupSetLanguage(EranosNewExpenseGroup *const newGroup, char const *const language) {
    EranosNewExpenseGroup_set_language_result const result = EranosNewExpenseGroup_set_language(newGroup, eranosDetailView(language));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the language the apps show the group in, as a BCP 47 tag.
///
/// @param newGroup The new group
static inline void eranosNewExpenseGroupClearLanguage(EranosNewExpenseGroup *const newGroup) { EranosNewExpenseGroup_clear_language(newGroup); }

/// Destroys @p newGroup; destroying nullptr does nothing, as free() does.
///
/// @param newGroup The new group, which no call uses after this one, or nullptr
static inline void eranosNewExpenseGroupDestroy(EranosNewExpenseGroup *const newGroup) {
    if (nullptr != newGroup) {
        EranosNewExpenseGroup_destroy(newGroup);
    }
}

#endif // ERANOS_EXPENSE_GROUP_H
