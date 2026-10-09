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
#include "EranosSettlementPlan.h"
#include "EranosSettlementStrategy.h"
#include "EranosSummary.h"
#include "eranos/detail.h"
#include "eranos/money.h"
#include "eranos/text.h"

#include <stddef.h>

/// How a settlement plans its payments. The values of EranosSettlementStrategy, by the names of the C layer.
typedef enum EranosSettlementStrategy EranosSettlementStrategy;

/// The fewest payments while at most 20 balances are left after exact opposites pay each other, the greedy plan beyond; the settlement names the
/// strategy whose plan it is.
static constexpr EranosSettlementStrategy ERANOS_SETTLEMENT_STRATEGY_ADAPTIVE = EranosSettlementStrategy_Adaptive;
/// The fewest payments possible.
static constexpr EranosSettlementStrategy ERANOS_SETTLEMENT_STRATEGY_MINIMUM_TRANSFERS = EranosSettlementStrategy_MinimumTransfers;
/// A fast plan that pairs the largest debtor with the largest creditor, after exact opposites where that saves a payment.
static constexpr EranosSettlementStrategy ERANOS_SETTLEMENT_STRATEGY_GREEDY = EranosSettlementStrategy_Greedy;
/// Every debt between two members, unsimplified.
static constexpr EranosSettlementStrategy ERANOS_SETTLEMENT_STRATEGY_DIRECT = EranosSettlementStrategy_Direct;

/// The kind of a balance. The values of EranosBalanceKind, by the names of the C layer.
typedef enum EranosBalanceKind EranosBalanceKind;

/// The member owes the others `amount`.
static constexpr EranosBalanceKind ERANOS_BALANCE_KIND_OWES = EranosBalanceKind_Owes;
/// The member owes nothing and is owed nothing.
static constexpr EranosBalanceKind ERANOS_BALANCE_KIND_SETTLED = EranosBalanceKind_Settled;
/// The others owe the member `amount`.
static constexpr EranosBalanceKind ERANOS_BALANCE_KIND_GETS_BACK = EranosBalanceKind_GetsBack;

/// Where a member stands.
typedef struct EranosBalance EranosBalance;

/// The member owes the others `amount`.
///
/// @param amount What the member owes the others
/// @return The balance, which the caller destroys
[[nodiscard]] static inline EranosBalance *eranosBalanceCreateOwes(EranosMoney const *const amount) { return EranosBalance_create_owes(amount); }

/// The member owes nothing and is owed nothing.
///
/// @return The balance, which the caller destroys
[[nodiscard]] static inline EranosBalance *eranosBalanceCreateSettled(void) { return EranosBalance_create_settled(); }

/// The others owe the member `amount`.
///
/// @param amount What the others owe the member
/// @return The balance, which the caller destroys
[[nodiscard]] static inline EranosBalance *eranosBalanceCreateGetsBack(EranosMoney const *const amount) {
    return EranosBalance_create_gets_back(amount);
}

/// Which kind of balance it is.
///
/// @param balance The balance
/// @return The result
[[nodiscard]] static inline EranosBalanceKind eranosBalanceKind(EranosBalance const *const balance) { return EranosBalance_kind(balance); }

/// What the member owes the others in the kind ERANOS_BALANCE_KIND_OWES, and what the others owe the member in the kind
/// ERANOS_BALANCE_KIND_GETS_BACK. Present in the kinds ERANOS_BALANCE_KIND_OWES and ERANOS_BALANCE_KIND_GETS_BACK alone.
///
/// @param balance The balance
/// @return A copy of the value, which the caller destroys, or nullptr in every other kind
[[nodiscard]] static inline EranosMoney *eranosBalanceAmount(EranosBalance const *const balance) { return EranosBalance_amount(balance); }

/// Writes the balance as the library does: "owes 12.00 EUR", "settled", or "gets back 12.00 EUR".
///
/// @param balance The balance
/// @param text Receives the text, replacing what it held
static inline void eranosBalanceFormat(EranosBalance const *const balance, EranosText *const text) {
    eranosDetailClear(text);
    EranosBalance_format(balance, text);
    eranosDetailFinish(text);
}

/// Destroys @p balance; destroying nullptr does nothing, as free() does.
///
/// @param balance The balance, which no call uses after this one, or nullptr
static inline void eranosBalanceDestroy(EranosBalance *const balance) {
    if (nullptr != balance) {
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
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosCategoryTotalTotal(EranosCategoryTotal const *const categoryTotal) {
    return EranosCategoryTotal_total(categoryTotal);
}

/// Destroys @p categoryTotal; destroying nullptr does nothing, as free() does.
///
/// @param categoryTotal The category total, which no call uses after this one, or nullptr
static inline void eranosCategoryTotalDestroy(EranosCategoryTotal *const categoryTotal) {
    if (nullptr != categoryTotal) {
        EranosCategoryTotal_destroy(categoryTotal);
    }
}

/// A list of category total records.
typedef struct EranosCategoryTotalList EranosCategoryTotalList;

/// How many records the list holds.
///
/// @param list The category total list
/// @return The result
[[nodiscard]] static inline size_t eranosCategoryTotalListLength(EranosCategoryTotalList const *const list) {
    return EranosCategoryTotalList_len(list);
}

/// Whether the list holds no record.
///
/// @param list The category total list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosCategoryTotalListIsEmpty(EranosCategoryTotalList const *const list) {
    return EranosCategoryTotalList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The category total list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosCategoryTotal *eranosCategoryTotalListGet(EranosCategoryTotalList const *const list, size_t const index) {
    return EranosCategoryTotalList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The category total list, which no call uses after this one, or nullptr
static inline void eranosCategoryTotalListDestroy(EranosCategoryTotalList *const list) {
    if (nullptr != list) {
        EranosCategoryTotalList_destroy(list);
    }
}

/// An expense's shares, in the group currency.
typedef struct EranosExpenseShares EranosExpenseShares;

/// Every member's part.
///
/// @param expenseShares The expense shares
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMemberShareList *eranosExpenseSharesMembers(EranosExpenseShares const *const expenseShares) {
    return EranosExpenseShares_members(expenseShares);
}

/// The amount with the tip.
///
/// @param expenseShares The expense shares
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosExpenseSharesTotal(EranosExpenseShares const *const expenseShares) {
    return EranosExpenseShares_total(expenseShares);
}

/// The tip.
///
/// @param expenseShares The expense shares
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosExpenseSharesTip(EranosExpenseShares const *const expenseShares) {
    return EranosExpenseShares_tip(expenseShares);
}

/// Destroys @p expenseShares; destroying nullptr does nothing, as free() does.
///
/// @param expenseShares The expense shares, which no call uses after this one, or nullptr
static inline void eranosExpenseSharesDestroy(EranosExpenseShares *const expenseShares) {
    if (nullptr != expenseShares) {
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
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosCategoryTotalList *eranosMemberCategoryTotalsTotals(EranosMemberCategoryTotals const *const memberCategoryTotals) {
    return EranosMemberCategoryTotals_totals(memberCategoryTotals);
}

/// Destroys @p memberCategoryTotals; destroying nullptr does nothing, as free() does.
///
/// @param memberCategoryTotals The member category totals, which no call uses after this one, or nullptr
static inline void eranosMemberCategoryTotalsDestroy(EranosMemberCategoryTotals *const memberCategoryTotals) {
    if (nullptr != memberCategoryTotals) {
        EranosMemberCategoryTotals_destroy(memberCategoryTotals);
    }
}

/// A list of member category totals records.
typedef struct EranosMemberCategoryTotalsList EranosMemberCategoryTotalsList;

/// How many records the list holds.
///
/// @param list The member category totals list
/// @return The result
[[nodiscard]] static inline size_t eranosMemberCategoryTotalsListLength(EranosMemberCategoryTotalsList const *const list) {
    return EranosMemberCategoryTotalsList_len(list);
}

/// Whether the list holds no record.
///
/// @param list The member category totals list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosMemberCategoryTotalsListIsEmpty(EranosMemberCategoryTotalsList const *const list) {
    return EranosMemberCategoryTotalsList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member category totals list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosMemberCategoryTotals *eranosMemberCategoryTotalsListGet(EranosMemberCategoryTotalsList const *const list,
                                                                                          size_t const index) {
    return EranosMemberCategoryTotalsList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The member category totals list, which no call uses after this one, or nullptr
static inline void eranosMemberCategoryTotalsListDestroy(EranosMemberCategoryTotalsList *const list) {
    if (nullptr != list) {
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
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMemberSharePaid(EranosMemberShare const *const memberShare) {
    return EranosMemberShare_paid(memberShare);
}

/// The member's share of it.
///
/// @param memberShare The member share
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMemberShareShare(EranosMemberShare const *const memberShare) {
    return EranosMemberShare_share(memberShare);
}

/// Destroys @p memberShare; destroying nullptr does nothing, as free() does.
///
/// @param memberShare The member share, which no call uses after this one, or nullptr
static inline void eranosMemberShareDestroy(EranosMemberShare *const memberShare) {
    if (nullptr != memberShare) {
        EranosMemberShare_destroy(memberShare);
    }
}

/// A list of member share records.
typedef struct EranosMemberShareList EranosMemberShareList;

/// How many records the list holds.
///
/// @param list The member share list
/// @return The result
[[nodiscard]] static inline size_t eranosMemberShareListLength(EranosMemberShareList const *const list) { return EranosMemberShareList_len(list); }

/// Whether the list holds no record.
///
/// @param list The member share list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosMemberShareListIsEmpty(EranosMemberShareList const *const list) {
    return EranosMemberShareList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member share list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosMemberShare *eranosMemberShareListGet(EranosMemberShareList const *const list, size_t const index) {
    return EranosMemberShareList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The member share list, which no call uses after this one, or nullptr
static inline void eranosMemberShareListDestroy(EranosMemberShareList *const list) {
    if (nullptr != list) {
        EranosMemberShareList_destroy(list);
    }
}

/// Where a member stands: what they paid, consumed, sent, and received.
typedef struct EranosMemberSummary EranosMemberSummary;

/// The member's identifier.
///
/// @param memberSummary The member summary
/// @param member Receives the member's identifier, replacing what it held
static inline void eranosMemberSummaryMember(EranosMemberSummary const *const memberSummary, EranosText *const member) {
    eranosDetailClear(member);
    EranosMemberSummary_member(memberSummary, member);
    eranosDetailFinish(member);
}

/// What the member paid for expenses.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMemberSummaryPaid(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_paid(memberSummary);
}

/// The member's share of the expenses: what they consumed.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMemberSummaryShare(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_share(memberSummary);
}

/// What the member sent in transfers.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMemberSummarySent(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_sent(memberSummary);
}

/// What the member received in transfers.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMemberSummaryReceived(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_received(memberSummary);
}

/// What the member is owed, less what they owe: positive when they get money back.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosMemberSummaryNet(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_net(memberSummary);
}

/// Where the member stands.
///
/// @param memberSummary The member summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosBalance *eranosMemberSummaryBalance(EranosMemberSummary const *const memberSummary) {
    return EranosMemberSummary_balance(memberSummary);
}

/// Destroys @p memberSummary; destroying nullptr does nothing, as free() does.
///
/// @param memberSummary The member summary, which no call uses after this one, or nullptr
static inline void eranosMemberSummaryDestroy(EranosMemberSummary *const memberSummary) {
    if (nullptr != memberSummary) {
        EranosMemberSummary_destroy(memberSummary);
    }
}

/// A list of member summary records.
typedef struct EranosMemberSummaryList EranosMemberSummaryList;

/// How many records the list holds.
///
/// @param list The member summary list
/// @return The result
[[nodiscard]] static inline size_t eranosMemberSummaryListLength(EranosMemberSummaryList const *const list) {
    return EranosMemberSummaryList_len(list);
}

/// Whether the list holds no record.
///
/// @param list The member summary list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosMemberSummaryListIsEmpty(EranosMemberSummaryList const *const list) {
    return EranosMemberSummaryList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member summary list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosMemberSummary *eranosMemberSummaryListGet(EranosMemberSummaryList const *const list, size_t const index) {
    return EranosMemberSummaryList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The member summary list, which no call uses after this one, or nullptr
static inline void eranosMemberSummaryListDestroy(EranosMemberSummaryList *const list) {
    if (nullptr != list) {
        EranosMemberSummaryList_destroy(list);
    }
}

/// A payment of a settlement.
typedef struct EranosPayment EranosPayment;

/// The identifier of the member who pays.
///
/// @param payment The payment
/// @param sender Receives the identifier of the member who pays, replacing what it held
static inline void eranosPaymentSender(EranosPayment const *const payment, EranosText *const sender) {
    eranosDetailClear(sender);
    EranosPayment_sender(payment, sender);
    eranosDetailFinish(sender);
}

/// The identifier of the member who is paid.
///
/// @param payment The payment
/// @param receiver Receives the identifier of the member who is paid, replacing what it held
static inline void eranosPaymentReceiver(EranosPayment const *const payment, EranosText *const receiver) {
    eranosDetailClear(receiver);
    EranosPayment_receiver(payment, receiver);
    eranosDetailFinish(receiver);
}

/// What is paid.
///
/// @param payment The payment
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosPaymentAmount(EranosPayment const *const payment) { return EranosPayment_amount(payment); }

/// Destroys @p payment; destroying nullptr does nothing, as free() does.
///
/// @param payment The payment, which no call uses after this one, or nullptr
static inline void eranosPaymentDestroy(EranosPayment *const payment) {
    if (nullptr != payment) {
        EranosPayment_destroy(payment);
    }
}

/// A list of payment records.
typedef struct EranosPaymentList EranosPaymentList;

/// How many records the list holds.
///
/// @param list The payment list
/// @return The result
[[nodiscard]] static inline size_t eranosPaymentListLength(EranosPaymentList const *const list) { return EranosPaymentList_len(list); }

/// Whether the list holds no record.
///
/// @param list The payment list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosPaymentListIsEmpty(EranosPaymentList const *const list) { return EranosPaymentList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The payment list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosPayment *eranosPaymentListGet(EranosPaymentList const *const list, size_t const index) {
    return EranosPaymentList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The payment list, which no call uses after this one, or nullptr
static inline void eranosPaymentListDestroy(EranosPaymentList *const list) {
    if (nullptr != list) {
        EranosPaymentList_destroy(list);
    }
}

/// A plan of payments that settles every balance.
typedef struct EranosSettlementPlan EranosSettlementPlan;

/// The payments.
///
/// @param settlement The settlement
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosPaymentList *eranosSettlementPlanPayments(EranosSettlementPlan const *const settlement) {
    return EranosSettlementPlan_payments(settlement);
}

/// The strategy that made the plan: the one asked for, or, for the adaptive strategy, the one whose plan it chose.
///
/// @param settlement The settlement
/// @return The strategy that made the plan
[[nodiscard]] static inline EranosSettlementStrategy eranosSettlementPlanStrategy(EranosSettlementPlan const *const settlement) {
    return EranosSettlementPlan_strategy(settlement);
}

/// Destroys @p settlement; destroying nullptr does nothing, as free() does.
///
/// @param settlement The settlement, which no call uses after this one, or nullptr
static inline void eranosSettlementPlanDestroy(EranosSettlementPlan *const settlement) {
    if (nullptr != settlement) {
        EranosSettlementPlan_destroy(settlement);
    }
}

/// The group's totals.
typedef struct EranosSummary EranosSummary;

/// What the expenses add up to.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMoney *eranosSummaryTotalSpent(EranosSummary const *const summary) { return EranosSummary_total_spent(summary); }

/// Where every member stands.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMemberSummaryList *eranosSummaryMembers(EranosSummary const *const summary) {
    return EranosSummary_members(summary);
}

/// The total of every category.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosCategoryTotalList *eranosSummaryCategoryTotals(EranosSummary const *const summary) {
    return EranosSummary_category_totals(summary);
}

/// The totals per category of every member.
///
/// @param summary The summary
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosMemberCategoryTotalsList *eranosSummaryMemberCategoryTotals(EranosSummary const *const summary) {
    return EranosSummary_member_category_totals(summary);
}

/// The totals of the member with the identifier `member`: all zero, and settled, for a member without entries.
///
/// @param summary The summary
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param memberSummary Receives the member's totals on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosSummaryMember(EranosSummary const *const summary, char const *const member,
                                                             EranosMemberSummary **const memberSummary) {
    EranosSummary_member_result const result = EranosSummary_member(summary, eranosDetailView(member));
    *memberSummary = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The spending in the category with the identifier `category`: zero for a category without expenses.
///
/// @param summary The summary
/// @param category The category's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param total Receives the spending on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosSummaryCategoryTotal(EranosSummary const *const summary, char const *const category,
                                                                    EranosMoney **const total) {
    EranosSummary_category_total_result const result = EranosSummary_category_total(summary, eranosDetailView(category));
    *total = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The share of the member with the identifier `member` of the spending in the category with the identifier `category`: zero when they have none
/// there.
///
/// @param summary The summary
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param category The category's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param total Receives the member's share of the spending on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosSummaryMemberCategoryTotal(EranosSummary const *const summary, char const *const member,
                                                                          char const *const category, EranosMoney **const total) {
    EranosSummary_member_category_total_result const result =
        EranosSummary_member_category_total(summary, eranosDetailView(member), eranosDetailView(category));
    *total = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p summary; destroying nullptr does nothing, as free() does.
///
/// @param summary The summary, which no call uses after this one, or nullptr
static inline void eranosSummaryDestroy(EranosSummary *const summary) {
    if (nullptr != summary) {
        EranosSummary_destroy(summary);
    }
}

#endif // ERANOS_TOTALS_H
