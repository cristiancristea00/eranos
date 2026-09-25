/// @file eranos/totals.h
/// The group's totals: the summary, the members' balances, an expense's shares, and the settlements.

#ifndef ERANOS_TOTALS_H
#define ERANOS_TOTALS_H

#include "EranosBalance.h"
#include "EranosBalanceKind.h"
#include "EranosCategoryTotal.h"
#include "EranosCategoryTotalList.h"
#include "EranosExpenseShares.h"
#include "EranosMemberCategoryTotals.h"
#include "EranosMemberCategoryTotalsList.h"
#include "EranosMemberShare.h"
#include "EranosMemberShareList.h"
#include "EranosMemberSummary.h"
#include "EranosMemberSummaryList.h"
#include "EranosPayment.h"
#include "EranosPaymentList.h"
#include "EranosSettlement.h"
#include "EranosSettlementStrategy.h"
#include "EranosSummary.h"
#include "eranos/detail.h"
#include "eranos/money.h"
#include "eranos/text.h"

#include <stdbool.h>
#include <stddef.h>

/// How a settlement plans its payments. The values of EranosSettlementStrategy, by the names of the C layer.
typedef enum EranosSettlementStrategy EranosSettlementStrategy;

/// The values of EranosSettlementStrategy.
enum {
    /// Every debt between two members, unsimplified.
    ERANOS_SETTLEMENT_STRATEGY_DIRECT = EranosSettlementStrategy_Direct,
    /// A fast plan that pairs the largest debtor with the largest creditor.
    ERANOS_SETTLEMENT_STRATEGY_GREEDY = EranosSettlementStrategy_Greedy,
    /// The fewest payments possible.
    ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS = EranosSettlementStrategy_MinimumTransfers,
};

/// The kind of a balance. The values of EranosBalanceKind, by the names of the C layer.
typedef enum EranosBalanceKind EranosBalanceKind;

/// The values of EranosBalanceKind.
enum {
    /// The others owe the member `amount`.
    ERANOS_BALANCE_KIND_GETS_BACK = EranosBalanceKind_GetsBack,
    /// The member owes the others `amount`.
    ERANOS_BALANCE_KIND_OWES = EranosBalanceKind_Owes,
    /// The member owes nothing and is owed nothing.
    ERANOS_BALANCE_KIND_SETTLED = EranosBalanceKind_Settled,
};

/// Where a member stands.
typedef struct EranosBalance EranosBalance;

/// The others owe the member `amount`.
///
/// @param amount What the others owe the member
/// @return The balance, which the caller destroys
static inline EranosBalance *eranosBalanceCreateGetsBack(EranosMoney const *const amount) { return EranosBalance_create_gets_back(amount); }

/// The member owes the others `amount`.
///
/// @param amount What the others owe the member
/// @return The balance, which the caller destroys
static inline EranosBalance *eranosBalanceCreateOwes(EranosMoney const *const amount) { return EranosBalance_create_owes(amount); }

/// The member owes nothing and is owed nothing.
///
/// @return The balance, which the caller destroys
static inline EranosBalance *eranosBalanceCreateSettled(void) { return EranosBalance_create_settled(); }

/// Which kind of balance it is.
///
/// @param balance The balance
/// @return The result
static inline EranosBalanceKind eranosBalanceKind(EranosBalance const *const balance) { return EranosBalance_kind(balance); }

/// What the others owe the member. Present in the kinds `GetsBack`, `Owes` alone.
///
/// @param balance The balance
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosBalanceAmount(EranosBalance const *const balance) { return EranosBalance_amount(balance); }

/// Writes the balance as the library does: "gets back 12.00 EUR", "owes 12.00 EUR", or "settled".
///
/// @param balance The balance
/// @param text Receives the text, replacing what it held
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_MONEY for a currency
///   the library does not list
static inline EranosError *eranosBalanceFormat(EranosBalance const *const balance, EranosText *const text) {
    eranosDetailClear(text);
    EranosBalance_format_result const result = EranosBalance_format(balance, text);
    eranosDetailFinish(text);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p balance; destroying NULL does nothing, as free() does.
///
/// @param balance The balance, which no call uses after this one, or NULL
static inline void eranosBalanceDestroy(EranosBalance *const balance) {
    if (NULL != balance) {
        EranosBalance_destroy(balance);
    }
}

/// What the expenses of one category add up to.
typedef struct EranosCategoryTotal EranosCategoryTotal;

/// The category's identifier.
///
/// @param categoryTotal The category total
/// @param category Receives the category's identifier, replacing what it held
static inline void eranosCategoryTotalCategory(EranosCategoryTotal const *const categoryTotal, EranosText *const category) {
    eranosDetailClear(category);
    EranosCategoryTotal_category(categoryTotal, category);
    eranosDetailFinish(category);
}

/// The total.
///
/// @param categoryTotal The category total
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosCategoryTotalTotal(EranosCategoryTotal const *const categoryTotal) {
    return EranosCategoryTotal_total(categoryTotal);
}

/// Destroys @p categoryTotal; destroying NULL does nothing, as free() does.
///
/// @param categoryTotal The category total, which no call uses after this one, or NULL
static inline void eranosCategoryTotalDestroy(EranosCategoryTotal *const categoryTotal) {
    if (NULL != categoryTotal) {
        EranosCategoryTotal_destroy(categoryTotal);
    }
}

/// A list of category total records.
typedef struct EranosCategoryTotalList EranosCategoryTotalList;

/// How many records the list holds.
///
/// @param list The category total list
/// @return The result
static inline size_t eranosCategoryTotalListLength(EranosCategoryTotalList const *const list) { return EranosCategoryTotalList_len(list); }

/// Whether the list holds no record.
///
/// @param list The category total list
/// @return Whether it holds none
static inline bool eranosCategoryTotalListIsEmpty(EranosCategoryTotalList const *const list) { return EranosCategoryTotalList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The category total list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosCategoryTotal *eranosCategoryTotalListGet(EranosCategoryTotalList const *const list, size_t const index) {
    return EranosCategoryTotalList_get(list, index);
}

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The category total list, which no call uses after this one, or NULL
static inline void eranosCategoryTotalListDestroy(EranosCategoryTotalList *const list) {
    if (NULL != list) {
        EranosCategoryTotalList_destroy(list);
    }
}

/// An expense's shares, in the group currency.
typedef struct EranosExpenseShares EranosExpenseShares;

/// Every member's part.
///
/// @param expenseShares The expense shares
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMemberShareList *eranosExpenseSharesMembers(EranosExpenseShares const *const expenseShares) {
    return EranosExpenseShares_members(expenseShares);
}

/// The tip.
///
/// @param expenseShares The expense shares
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosExpenseSharesTip(EranosExpenseShares const *const expenseShares) { return EranosExpenseShares_tip(expenseShares); }

/// The amount with the tip.
///
/// @param expenseShares The expense shares
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosExpenseSharesTotal(EranosExpenseShares const *const expenseShares) {
    return EranosExpenseShares_total(expenseShares);
}

/// Destroys @p expenseShares; destroying NULL does nothing, as free() does.
///
/// @param expenseShares The expense shares, which no call uses after this one, or NULL
static inline void eranosExpenseSharesDestroy(EranosExpenseShares *const expenseShares) {
    if (NULL != expenseShares) {
        EranosExpenseShares_destroy(expenseShares);
    }
}

/// A member's share of each category.
typedef struct EranosMemberCategoryTotals EranosMemberCategoryTotals;

/// The member's identifier.
///
/// @param memberCategoryTotals The member category totals
/// @param member Receives the member's identifier, replacing what it held
static inline void eranosMemberCategoryTotalsMember(EranosMemberCategoryTotals const *const memberCategoryTotals, EranosText *const member) {
    eranosDetailClear(member);
    EranosMemberCategoryTotals_member(memberCategoryTotals, member);
    eranosDetailFinish(member);
}

/// The member's total in every category they have a share in.
///
/// @param memberCategoryTotals The member category totals
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosCategoryTotalList *eranosMemberCategoryTotalsTotals(EranosMemberCategoryTotals const *const memberCategoryTotals) {
    return EranosMemberCategoryTotals_totals(memberCategoryTotals);
}

/// Destroys @p memberCategoryTotals; destroying NULL does nothing, as free() does.
///
/// @param memberCategoryTotals The member category totals, which no call uses after this one, or NULL
static inline void eranosMemberCategoryTotalsDestroy(EranosMemberCategoryTotals *const memberCategoryTotals) {
    if (NULL != memberCategoryTotals) {
        EranosMemberCategoryTotals_destroy(memberCategoryTotals);
    }
}

/// A list of member category totals records.
typedef struct EranosMemberCategoryTotalsList EranosMemberCategoryTotalsList;

/// How many records the list holds.
///
/// @param list The member category totals list
/// @return The result
static inline size_t eranosMemberCategoryTotalsListLength(EranosMemberCategoryTotalsList const *const list) {
    return EranosMemberCategoryTotalsList_len(list);
}

/// Whether the list holds no record.
///
/// @param list The member category totals list
/// @return Whether it holds none
static inline bool eranosMemberCategoryTotalsListIsEmpty(EranosMemberCategoryTotalsList const *const list) {
    return EranosMemberCategoryTotalsList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member category totals list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosMemberCategoryTotals *eranosMemberCategoryTotalsListGet(EranosMemberCategoryTotalsList const *const list, size_t const index) {
    return EranosMemberCategoryTotalsList_get(list, index);
}

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The member category totals list, which no call uses after this one, or NULL
static inline void eranosMemberCategoryTotalsListDestroy(EranosMemberCategoryTotalsList *const list) {
    if (NULL != list) {
        EranosMemberCategoryTotalsList_destroy(list);
    }
}

/// A member's part of an expense.
typedef struct EranosMemberShare EranosMemberShare;

/// The member's identifier.
///
/// @param memberShare The member share
/// @param member Receives the member's identifier, replacing what it held
static inline void eranosMemberShareMember(EranosMemberShare const *const memberShare, EranosText *const member) {
    eranosDetailClear(member);
    EranosMemberShare_member(memberShare, member);
    eranosDetailFinish(member);
}

/// What the member paid of it.
///
/// @param memberShare The member share
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosMemberSharePaid(EranosMemberShare const *const memberShare) { return EranosMemberShare_paid(memberShare); }

/// The member's share of it.
///
/// @param memberShare The member share
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosMemberShareShare(EranosMemberShare const *const memberShare) { return EranosMemberShare_share(memberShare); }

/// Destroys @p memberShare; destroying NULL does nothing, as free() does.
///
/// @param memberShare The member share, which no call uses after this one, or NULL
static inline void eranosMemberShareDestroy(EranosMemberShare *const memberShare) {
    if (NULL != memberShare) {
        EranosMemberShare_destroy(memberShare);
    }
}

/// A list of member share records.
typedef struct EranosMemberShareList EranosMemberShareList;

/// How many records the list holds.
///
/// @param list The member share list
/// @return The result
static inline size_t eranosMemberShareListLength(EranosMemberShareList const *const list) { return EranosMemberShareList_len(list); }

/// Whether the list holds no record.
///
/// @param list The member share list
/// @return Whether it holds none
static inline bool eranosMemberShareListIsEmpty(EranosMemberShareList const *const list) { return EranosMemberShareList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member share list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosMemberShare *eranosMemberShareListGet(EranosMemberShareList const *const list, size_t const index) {
    return EranosMemberShareList_get(list, index);
}

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The member share list, which no call uses after this one, or NULL
static inline void eranosMemberShareListDestroy(EranosMemberShareList *const list) {
    if (NULL != list) {
        EranosMemberShareList_destroy(list);
    }
}

/// Where a member stands: what they paid, consumed, sent, and received.
typedef struct EranosMemberSummary EranosMemberSummary;

/// Where the member stands.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosBalance *eranosMemberSummaryBalance(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_balance(memberSummary);
}

/// The member's identifier.
///
/// @param memberSummary The member summary
/// @param member Receives the member's identifier, replacing what it held
static inline void eranosMemberSummaryMember(EranosMemberSummary const *const memberSummary, EranosText *const member) {
    eranosDetailClear(member);
    EranosMemberSummary_member(memberSummary, member);
    eranosDetailFinish(member);
}

/// What the member is owed, less what they owe: positive when they get money back.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosMemberSummaryNet(EranosMemberSummary const *const memberSummary) { return EranosMemberSummary_net(memberSummary); }

/// What the member paid for expenses.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosMemberSummaryPaid(EranosMemberSummary const *const memberSummary) { return EranosMemberSummary_paid(memberSummary); }

/// What the member received in transfers.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosMemberSummaryReceived(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_received(memberSummary);
}

/// What the member sent in transfers.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosMemberSummarySent(EranosMemberSummary const *const memberSummary) { return EranosMemberSummary_sent(memberSummary); }

/// The member's share of the expenses: what they consumed.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosMemberSummaryShare(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_share(memberSummary);
}

/// Destroys @p memberSummary; destroying NULL does nothing, as free() does.
///
/// @param memberSummary The member summary, which no call uses after this one, or NULL
static inline void eranosMemberSummaryDestroy(EranosMemberSummary *const memberSummary) {
    if (NULL != memberSummary) {
        EranosMemberSummary_destroy(memberSummary);
    }
}

/// A list of member summary records.
typedef struct EranosMemberSummaryList EranosMemberSummaryList;

/// How many records the list holds.
///
/// @param list The member summary list
/// @return The result
static inline size_t eranosMemberSummaryListLength(EranosMemberSummaryList const *const list) { return EranosMemberSummaryList_len(list); }

/// Whether the list holds no record.
///
/// @param list The member summary list
/// @return Whether it holds none
static inline bool eranosMemberSummaryListIsEmpty(EranosMemberSummaryList const *const list) { return EranosMemberSummaryList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member summary list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosMemberSummary *eranosMemberSummaryListGet(EranosMemberSummaryList const *const list, size_t const index) {
    return EranosMemberSummaryList_get(list, index);
}

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The member summary list, which no call uses after this one, or NULL
static inline void eranosMemberSummaryListDestroy(EranosMemberSummaryList *const list) {
    if (NULL != list) {
        EranosMemberSummaryList_destroy(list);
    }
}

/// A payment of a settlement.
typedef struct EranosPayment EranosPayment;

/// What is paid.
///
/// @param payment The payment
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosPaymentAmount(EranosPayment const *const payment) { return EranosPayment_amount(payment); }

/// The identifier of the member who pays.
///
/// @param payment The payment
/// @param from Receives the identifier of the member who pays, replacing what it held
static inline void eranosPaymentFrom(EranosPayment const *const payment, EranosText *const from) {
    eranosDetailClear(from);
    EranosPayment_from(payment, from);
    eranosDetailFinish(from);
}

/// The identifier of the member who is paid.
///
/// @param payment The payment
/// @param to Receives the identifier of the member who is paid, replacing what it held
static inline void eranosPaymentTo(EranosPayment const *const payment, EranosText *const to) {
    eranosDetailClear(to);
    EranosPayment_to(payment, to);
    eranosDetailFinish(to);
}

/// Destroys @p payment; destroying NULL does nothing, as free() does.
///
/// @param payment The payment, which no call uses after this one, or NULL
static inline void eranosPaymentDestroy(EranosPayment *const payment) {
    if (NULL != payment) {
        EranosPayment_destroy(payment);
    }
}

/// A list of payment records.
typedef struct EranosPaymentList EranosPaymentList;

/// How many records the list holds.
///
/// @param list The payment list
/// @return The result
static inline size_t eranosPaymentListLength(EranosPaymentList const *const list) { return EranosPaymentList_len(list); }

/// Whether the list holds no record.
///
/// @param list The payment list
/// @return Whether it holds none
static inline bool eranosPaymentListIsEmpty(EranosPaymentList const *const list) { return EranosPaymentList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The payment list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosPayment *eranosPaymentListGet(EranosPaymentList const *const list, size_t const index) {
    return EranosPaymentList_get(list, index);
}

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The payment list, which no call uses after this one, or NULL
static inline void eranosPaymentListDestroy(EranosPaymentList *const list) {
    if (NULL != list) {
        EranosPaymentList_destroy(list);
    }
}

/// A plan of payments that settles every balance.
typedef struct EranosSettlement EranosSettlement;

/// The payments.
///
/// @param settlement The settlement
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosPaymentList *eranosSettlementPayments(EranosSettlement const *const settlement) { return EranosSettlement_payments(settlement); }

/// The strategy that made the plan.
///
/// @param settlement The settlement
/// @return The strategy that made the plan
static inline EranosSettlementStrategy eranosSettlementStrategy(EranosSettlement const *const settlement) {
    return EranosSettlement_strategy(settlement);
}

/// Destroys @p settlement; destroying NULL does nothing, as free() does.
///
/// @param settlement The settlement, which no call uses after this one, or NULL
static inline void eranosSettlementDestroy(EranosSettlement *const settlement) {
    if (NULL != settlement) {
        EranosSettlement_destroy(settlement);
    }
}

/// The group's totals.
typedef struct EranosSummary EranosSummary;

/// The total of every category.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosCategoryTotalList *eranosSummaryCategoryTotals(EranosSummary const *const summary) {
    return EranosSummary_category_totals(summary);
}

/// The totals per category of every member.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMemberCategoryTotalsList *eranosSummaryMemberCategoryTotals(EranosSummary const *const summary) {
    return EranosSummary_member_category_totals(summary);
}

/// Where every member stands.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMemberSummaryList *eranosSummaryMembers(EranosSummary const *const summary) { return EranosSummary_members(summary); }

/// What the expenses add up to.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosMoney *eranosSummaryTotalSpent(EranosSummary const *const summary) { return EranosSummary_total_spent(summary); }

/// Destroys @p summary; destroying NULL does nothing, as free() does.
///
/// @param summary The summary, which no call uses after this one, or NULL
static inline void eranosSummaryDestroy(EranosSummary *const summary) {
    if (NULL != summary) {
        EranosSummary_destroy(summary);
    }
}

#endif // ERANOS_TOTALS_H
