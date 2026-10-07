/// @file eranos/expense.h
/// Expenses: what was bought, what it cost, who paid, and how it divides, with their costs, items, and tips.

#ifndef ERANOS_EXPENSE_H
#define ERANOS_EXPENSE_H

#include "EranosCost.h"
#include "EranosCostKind.h"
#include "EranosExpense.h"
#include "EranosExpenseItem.h"
#include "EranosExpenseItemList.h"
#include "EranosExpenseList.h"
#include "EranosExpenseSplit.h"
#include "EranosExpenseSplitKind.h"
#include "EranosTipDistribution.h"
#include "EranosTipRule.h"
#include "EranosTipValue.h"
#include "EranosTipValueKind.h"
#include "eranos/calendar.h"
#include "eranos/detail.h"
#include "eranos/identifiers.h"
#include "eranos/money.h"
#include "eranos/split.h"
#include "eranos/text.h"

#include <stddef.h>
#include <stdint.h>

/// The kind of a tip value. The values of EranosTipValueKind, by the names of the C layer.
typedef enum EranosTipValueKind EranosTipValueKind;

/// A percentage of the expense's amount.
static constexpr EranosTipValueKind ERANOS_TIP_VALUE_KIND_PERCENTAGE = EranosTipValueKind_Percentage;
/// A fixed amount in the expense's currency.
static constexpr EranosTipValueKind ERANOS_TIP_VALUE_KIND_FIXED = EranosTipValueKind_Fixed;

/// How a tip is shared among the participants. The values of EranosTipDistribution, by the names of the C layer.
typedef enum EranosTipDistribution EranosTipDistribution;

/// In proportion to each participant's share of the amount.
static constexpr EranosTipDistribution ERANOS_TIP_DISTRIBUTION_PROPORTIONAL = EranosTipDistribution_Proportional;
/// The same for every participant.
static constexpr EranosTipDistribution ERANOS_TIP_DISTRIBUTION_EQUAL = EranosTipDistribution_Equal;

/// The kind of a expense split. The values of EranosExpenseSplitKind, by the names of the C layer.
typedef enum EranosExpenseSplitKind EranosExpenseSplitKind;

/// The whole amount is divided by one split.
static constexpr EranosExpenseSplitKind ERANOS_EXPENSE_SPLIT_KIND_WHOLE = EranosExpenseSplitKind_Whole;
/// Every line is divided by its own split; the lines add up to the amount.
static constexpr EranosExpenseSplitKind ERANOS_EXPENSE_SPLIT_KIND_ITEMISED = EranosExpenseSplitKind_Itemised;

/// The kind of a cost. The values of EranosCostKind, by the names of the C layer.
typedef enum EranosCostKind EranosCostKind;

/// An amount in the group currency.
static constexpr EranosCostKind ERANOS_COST_KIND_LOCAL = EranosCostKind_Local;
/// An amount in another currency than the group's, converted at `rate`.
static constexpr EranosCostKind ERANOS_COST_KIND_CONVERTED = EranosCostKind_Converted;

/// What an entry cost: an amount in the group currency, or one in another, converted at a rate.
typedef struct EranosCost EranosCost;

/// An amount in the group currency.
///
/// @param amount The amount, in its own currency
/// @return The cost, which the caller destroys
[[nodiscard]] static inline EranosCost *eranosCostCreateLocal(EranosMoney const *const amount) { return EranosCost_create_local(amount); }

/// An amount in another currency than the group's, converted at `rate`.
///
/// @param amount The amount, in its own currency
/// @param rate The rate into the group currency, as a decimal, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cost Receives the cost on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCostCreateConverted(EranosMoney const *const amount, char const *const rate, EranosCost **const cost) {
    EranosCost_create_converted_result const result = EranosCost_create_converted(amount, eranosDetailView(rate));
    *cost = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Which kind of cost it is.
///
/// @param cost The cost
/// @return The result
[[nodiscard]] static inline EranosCostKind eranosCostKind(EranosCost const *const cost) { return EranosCost_kind(cost); }

/// The amount, in its own currency.
///
/// @param cost The cost
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosMoney *eranosCostAmount(EranosCost const *const cost) { return EranosCost_amount(cost); }

/// The rate into the group currency, as a decimal. Present in the kind ERANOS_ERROR_KIND_CONVERTED alone. Writes it, if present; absent otherwise.
///
/// @param cost The cost
/// @param rate Receives the rate into the group currency, as a decimal, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosCostRate(EranosCost const *const cost, EranosText *const rate) {
    eranosDetailClear(rate);
    bool const present = EranosCost_rate(cost, rate).is_ok;
    eranosDetailFinish(rate);
    return present;
}

/// Destroys @p cost; destroying nullptr does nothing, as free() does.
///
/// @param cost The cost, which no call uses after this one, or nullptr
static inline void eranosCostDestroy(EranosCost *const cost) {
    if (nullptr != cost) {
        EranosCost_destroy(cost);
    }
}

/// An expense: what was bought, what it cost, who paid, and how it divides.
typedef struct EranosExpense EranosExpense;

/// A new expense of its required fields; every other field starts absent, empty, or at its default.
///
/// @param name What was bought, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cost What it cost
/// @param payers Who paid the total, amount and tip, and how much each
/// @param split How the amount is shared
/// @param date The day it was bought
/// @param expense Receives the expense on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseCreate(char const *const name, EranosCost const *const cost, EranosSplit const *const payers,
                                                             EranosExpenseSplit const *const split, EranosCalendarDate const date,
                                                             EranosExpense **const expense) {
    EranosExpense_create_result const result = EranosExpense_create(eranosDetailView(name), cost, payers, split, date);
    *expense = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The identifier: the app's own for an expense to add, or absent for the library to generate one, and present for one the group holds. Writes it, if
/// present; absent otherwise.
///
/// @param expense The expense
/// @param id Receives the identifier: the app's own for an expense to add, or absent for the library to generate one, and present for one the group
///   holds, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosExpenseId(EranosExpense const *const expense, EranosText *const id) {
    eranosDetailClear(id);
    bool const present = EranosExpense_id(expense, id).is_ok;
    eranosDetailFinish(id);
    return present;
}

/// Sets the identifier: the app's own for an expense to add, or absent for the library to generate one, and present for one the group holds.
///
/// @param expense The expense
/// @param id The identifier: the app's own for an expense to add, or absent for the library to generate one, and present for one the group holds, as
///   UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseSetId(EranosExpense *const expense, char const *const id) {
    EranosExpense_set_id_result const result = EranosExpense_set_id(expense, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the identifier: the app's own for an expense to add, or absent for the library to generate one, and present for one the group holds.
///
/// @param expense The expense
static inline void eranosExpenseClearId(EranosExpense *const expense) { EranosExpense_clear_id(expense); }

/// What was bought.
///
/// @param expense The expense
/// @param name Receives what was bought, replacing what it held
static inline void eranosExpenseName(EranosExpense const *const expense, EranosText *const name) {
    eranosDetailClear(name);
    EranosExpense_name(expense, name);
    eranosDetailFinish(name);
}

/// Sets what was bought.
///
/// @param expense The expense
/// @param name What was bought, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseSetName(EranosExpense *const expense, char const *const name) {
    EranosExpense_set_name_result const result = EranosExpense_set_name(expense, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// What it cost.
///
/// @param expense The expense
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosCost *eranosExpenseCost(EranosExpense const *const expense) { return EranosExpense_cost(expense); }

/// Sets what it cost.
///
/// @param expense The expense
/// @param cost What it cost
static inline void eranosExpenseSetCost(EranosExpense *const expense, EranosCost const *const cost) { EranosExpense_set_cost(expense, cost); }

/// Who paid the total, amount and tip, and how much each.
///
/// @param expense The expense
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosSplit *eranosExpensePayers(EranosExpense const *const expense) { return EranosExpense_payers(expense); }

/// Sets who paid the total, amount and tip, and how much each.
///
/// @param expense The expense
/// @param payers Who paid the total, amount and tip, and how much each
static inline void eranosExpenseSetPayers(EranosExpense *const expense, EranosSplit const *const payers) {
    EranosExpense_set_payers(expense, payers);
}

/// How the amount is shared.
///
/// @param expense The expense
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosExpenseSplit *eranosExpenseSplit(EranosExpense const *const expense) { return EranosExpense_split(expense); }

/// Sets how the amount is shared.
///
/// @param expense The expense
/// @param split How the amount is shared
static inline void eranosExpenseSetSplit(EranosExpense *const expense, EranosExpenseSplit const *const split) {
    EranosExpense_set_split(expense, split);
}

/// The day it was bought.
///
/// @param expense The expense
/// @return The day it was bought
[[nodiscard]] static inline EranosCalendarDate eranosExpenseDate(EranosExpense const *const expense) { return EranosExpense_date(expense); }

/// Sets the day it was bought.
///
/// @param expense The expense
/// @param date The day it was bought
static inline void eranosExpenseSetDate(EranosExpense *const expense, EranosCalendarDate const date) { EranosExpense_set_date(expense, date); }

/// The tip on top of the amount.
///
/// @param expense The expense
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosTipRule *eranosExpenseTip(EranosExpense const *const expense) { return EranosExpense_tip(expense); }

/// Sets the tip on top of the amount.
///
/// @param expense The expense
/// @param tip The tip on top of the amount
static inline void eranosExpenseSetTip(EranosExpense *const expense, EranosTipRule const *const tip) { EranosExpense_set_tip(expense, tip); }

/// Clears the tip on top of the amount.
///
/// @param expense The expense
static inline void eranosExpenseClearTip(EranosExpense *const expense) { EranosExpense_clear_tip(expense); }

/// The time of day it was bought.
///
/// @param expense The expense
/// @param time Receives the time, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosExpenseTime(EranosExpense const *const expense, EranosTimeOfDay *const time) {
    EranosExpense_time_result const result = EranosExpense_time(expense);
    return eranosDetailPresentTimeOfDay(result.is_ok, &result.ok, time);
}

/// Sets the time of day it was bought.
///
/// @param expense The expense
/// @param time The time of day it was bought
static inline void eranosExpenseSetTime(EranosExpense *const expense, EranosTimeOfDay const time) { EranosExpense_set_time(expense, time); }

/// Clears the time of day it was bought.
///
/// @param expense The expense
static inline void eranosExpenseClearTime(EranosExpense *const expense) { EranosExpense_clear_time(expense); }

/// The category's identifier; absent for the built-in one. Writes it, if present; absent otherwise.
///
/// @param expense The expense
/// @param category Receives the category's identifier; absent for the built-in one, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosExpenseCategory(EranosExpense const *const expense, EranosText *const category) {
    eranosDetailClear(category);
    bool const present = EranosExpense_category(expense, category).is_ok;
    eranosDetailFinish(category);
    return present;
}

/// Sets the category's identifier; absent for the built-in one.
///
/// @param expense The expense
/// @param category The category's identifier; absent for the built-in one, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseSetCategory(EranosExpense *const expense, char const *const category) {
    EranosExpense_set_category_result const result = EranosExpense_set_category(expense, eranosDetailView(category));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the category's identifier; absent for the built-in one.
///
/// @param expense The expense
static inline void eranosExpenseClearCategory(EranosExpense *const expense) { EranosExpense_clear_category(expense); }

/// A longer description. Writes it, if present; absent otherwise.
///
/// @param expense The expense
/// @param description Receives a longer description, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosExpenseDescription(EranosExpense const *const expense, EranosText *const description) {
    eranosDetailClear(description);
    bool const present = EranosExpense_description(expense, description).is_ok;
    eranosDetailFinish(description);
    return present;
}

/// Sets a longer description.
///
/// @param expense The expense
/// @param description A longer description, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseSetDescription(EranosExpense *const expense, char const *const description) {
    EranosExpense_set_description_result const result = EranosExpense_set_description(expense, eranosDetailView(description));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears a longer description.
///
/// @param expense The expense
static inline void eranosExpenseClearDescription(EranosExpense *const expense) { EranosExpense_clear_description(expense); }

/// The tip in the expense's currency, zero without one.
///
/// @param expense The expense
/// @param tipAmount Receives the tip amount on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_EXPENSE for a tip
///   the expense cannot have, and of the kind ERANOS_ERROR_KIND_MONEY for an amount out of range
[[nodiscard]] static inline EranosError *eranosExpenseTipAmount(EranosExpense const *const expense, EranosMoney **const tipAmount) {
    EranosExpense_tip_amount_result const result = EranosExpense_tip_amount(expense);
    *tipAmount = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The amount with the tip, what the payers paid.
///
/// @param expense The expense
/// @param total Receives the total on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_EXPENSE for a tip
///   the expense cannot have, and of the kind ERANOS_ERROR_KIND_MONEY for an amount out of range
[[nodiscard]] static inline EranosError *eranosExpenseTotal(EranosExpense const *const expense, EranosMoney **const total) {
    EranosExpense_total_result const result = EranosExpense_total(expense);
    *total = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The identifiers of every member the expense names, payers and participants, each once and in the order of their identifiers.
///
/// @param expense The expense
/// @param participants Receives the participants on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_ID for an
/// identifier
///   that is not one
[[nodiscard]] static inline EranosError *eranosExpenseParticipants(EranosExpense const *const expense, EranosTextList **const participants) {
    EranosExpense_participants_result const result = EranosExpense_participants(expense);
    *participants = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p expense; destroying nullptr does nothing, as free() does.
///
/// @param expense The expense, which no call uses after this one, or nullptr
static inline void eranosExpenseDestroy(EranosExpense *const expense) {
    if (nullptr != expense) {
        EranosExpense_destroy(expense);
    }
}

/// A list of expense records.
typedef struct EranosExpenseList EranosExpenseList;

/// How many records the list holds.
///
/// @param list The expense list
/// @return The result
[[nodiscard]] static inline size_t eranosExpenseListLength(EranosExpenseList const *const list) { return EranosExpenseList_len(list); }

/// Whether the list holds no record.
///
/// @param list The expense list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosExpenseListIsEmpty(EranosExpenseList const *const list) { return EranosExpenseList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The expense list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosExpense *eranosExpenseListGet(EranosExpenseList const *const list, size_t const index) {
    return EranosExpenseList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The expense list, which no call uses after this one, or nullptr
static inline void eranosExpenseListDestroy(EranosExpenseList *const list) {
    if (nullptr != list) {
        EranosExpenseList_destroy(list);
    }
}

/// How an expense's amount is shared.
typedef struct EranosExpenseSplit EranosExpenseSplit;

/// The whole amount is divided by one split.
///
/// @param split How the amount divides
/// @return The expense split, which the caller destroys
[[nodiscard]] static inline EranosExpenseSplit *eranosExpenseSplitCreateWhole(EranosSplit const *const split) {
    return EranosExpenseSplit_create_whole(split);
}

/// Every line is divided by its own split; the lines add up to the amount.
///
/// @param items The lines of the bill
/// @return The expense split, which the caller destroys
[[nodiscard]] static inline EranosExpenseSplit *eranosExpenseSplitCreateItemised(EranosExpenseItemList const *const items) {
    return EranosExpenseSplit_create_itemised(items);
}

/// Which kind of expense split it is.
///
/// @param expenseSplit The expense split
/// @return The result
[[nodiscard]] static inline EranosExpenseSplitKind eranosExpenseSplitKind(EranosExpenseSplit const *const expenseSplit) {
    return EranosExpenseSplit_kind(expenseSplit);
}

/// The lines of the bill. Present in the kind ERANOS_ERROR_KIND_ITEMISED alone.
///
/// @param expenseSplit The expense split
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosExpenseItemList *eranosExpenseSplitItems(EranosExpenseSplit const *const expenseSplit) {
    return EranosExpenseSplit_items(expenseSplit);
}

/// How the amount divides. Present in the kind ERANOS_ERROR_KIND_WHOLE alone.
///
/// @param expenseSplit The expense split
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosSplit *eranosExpenseSplitSplit(EranosExpenseSplit const *const expenseSplit) {
    return EranosExpenseSplit_split(expenseSplit);
}

/// Destroys @p expenseSplit; destroying nullptr does nothing, as free() does.
///
/// @param expenseSplit The expense split, which no call uses after this one, or nullptr
static inline void eranosExpenseSplitDestroy(EranosExpenseSplit *const expenseSplit) {
    if (nullptr != expenseSplit) {
        EranosExpenseSplit_destroy(expenseSplit);
    }
}

/// A line of an itemised bill: `quantity` units of `name` at `unit_price`, which `split` divides.
typedef struct EranosExpenseItem EranosExpenseItem;

/// A new item of its required fields; every other field starts absent, empty, or at its default.
///
/// @param name What was bought, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param unitPrice The price of one unit, in the expense's currency
/// @param split How the line's total divides
/// @param item Receives the item on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseItemCreate(char const *const name, EranosMoney const *const unitPrice,
                                                                 EranosSplit const *const split, EranosExpenseItem **const item) {
    EranosExpenseItem_create_result const result = EranosExpenseItem_create(eranosDetailView(name), unitPrice, split);
    *item = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// What was bought.
///
/// @param item The item
/// @param name Receives what was bought, replacing what it held
static inline void eranosExpenseItemName(EranosExpenseItem const *const item, EranosText *const name) {
    eranosDetailClear(name);
    EranosExpenseItem_name(item, name);
    eranosDetailFinish(name);
}

/// Sets what was bought.
///
/// @param item The item
/// @param name What was bought, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseItemSetName(EranosExpenseItem *const item, char const *const name) {
    EranosExpenseItem_set_name_result const result = EranosExpenseItem_set_name(item, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The price of one unit, in the expense's currency.
///
/// @param item The item
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosMoney *eranosExpenseItemUnitPrice(EranosExpenseItem const *const item) {
    return EranosExpenseItem_unit_price(item);
}

/// Sets the price of one unit, in the expense's currency.
///
/// @param item The item
/// @param unitPrice The price of one unit, in the expense's currency
static inline void eranosExpenseItemSetUnitPrice(EranosExpenseItem *const item, EranosMoney const *const unitPrice) {
    EranosExpenseItem_set_unit_price(item, unitPrice);
}

/// How the line's total divides.
///
/// @param item The item
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosSplit *eranosExpenseItemSplit(EranosExpenseItem const *const item) { return EranosExpenseItem_split(item); }

/// Sets how the line's total divides.
///
/// @param item The item
/// @param split How the line's total divides
static inline void eranosExpenseItemSetSplit(EranosExpenseItem *const item, EranosSplit const *const split) {
    EranosExpenseItem_set_split(item, split);
}

/// How many units, from 1.
///
/// @param item The item
/// @return How many units, from 1
[[nodiscard]] static inline int32_t eranosExpenseItemQuantity(EranosExpenseItem const *const item) { return EranosExpenseItem_quantity(item); }

/// Sets how many units, from 1.
///
/// @param item The item
/// @param quantity How many units, from 1
static inline void eranosExpenseItemSetQuantity(EranosExpenseItem *const item, int32_t const quantity) {
    EranosExpenseItem_set_quantity(item, quantity);
}

/// Destroys @p item; destroying nullptr does nothing, as free() does.
///
/// @param item The item, which no call uses after this one, or nullptr
static inline void eranosExpenseItemDestroy(EranosExpenseItem *const item) {
    if (nullptr != item) {
        EranosExpenseItem_destroy(item);
    }
}

/// A list of item records.
typedef struct EranosExpenseItemList EranosExpenseItemList;

/// An empty list.
///
/// @return The item list, which the caller destroys
[[nodiscard]] static inline EranosExpenseItemList *eranosExpenseItemListCreate(void) { return EranosExpenseItemList_create(); }

/// Adds a copy of `element` at the end.
///
/// @param list The item list
/// @param element The element
static inline void eranosExpenseItemListPush(EranosExpenseItemList *const list, EranosExpenseItem const *const element) {
    EranosExpenseItemList_push(list, element);
}

/// How many records the list holds.
///
/// @param list The item list
/// @return The result
[[nodiscard]] static inline size_t eranosExpenseItemListLength(EranosExpenseItemList const *const list) { return EranosExpenseItemList_len(list); }

/// Whether the list holds no record.
///
/// @param list The item list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosExpenseItemListIsEmpty(EranosExpenseItemList const *const list) {
    return EranosExpenseItemList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The item list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosExpenseItem *eranosExpenseItemListGet(EranosExpenseItemList const *const list, size_t const index) {
    return EranosExpenseItemList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The item list, which no call uses after this one, or nullptr
static inline void eranosExpenseItemListDestroy(EranosExpenseItemList *const list) {
    if (nullptr != list) {
        EranosExpenseItemList_destroy(list);
    }
}

/// A tip on top of an expense's amount.
typedef struct EranosTipRule EranosTipRule;

/// A new tip of its required fields; every other field starts absent, empty, or at its default.
///
/// @param value How the tip is expressed
/// @param distribution How the tip is shared among the participants
/// @return The tip, which the caller destroys
[[nodiscard]] static inline EranosTipRule *eranosTipRuleCreate(EranosTipValue const *const value, EranosTipDistribution const distribution) {
    return EranosTipRule_create(value, distribution);
}

/// How the tip is expressed.
///
/// @param tip The tip
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosTipValue *eranosTipRuleValue(EranosTipRule const *const tip) { return EranosTipRule_value(tip); }

/// Sets how the tip is expressed.
///
/// @param tip The tip
/// @param value How the tip is expressed
static inline void eranosTipRuleSetValue(EranosTipRule *const tip, EranosTipValue const *const value) { EranosTipRule_set_value(tip, value); }

/// How the tip is shared among the participants.
///
/// @param tip The tip
/// @return How the tip is shared among the participants
[[nodiscard]] static inline EranosTipDistribution eranosTipRuleDistribution(EranosTipRule const *const tip) {
    return EranosTipRule_distribution(tip);
}

/// Sets how the tip is shared among the participants.
///
/// @param tip The tip
/// @param distribution How the tip is shared among the participants
static inline void eranosTipRuleSetDistribution(EranosTipRule *const tip, EranosTipDistribution const distribution) {
    EranosTipRule_set_distribution(tip, distribution);
}

/// Destroys @p tip; destroying nullptr does nothing, as free() does.
///
/// @param tip The tip, which no call uses after this one, or nullptr
static inline void eranosTipRuleDestroy(EranosTipRule *const tip) {
    if (nullptr != tip) {
        EranosTipRule_destroy(tip);
    }
}

/// How a tip is expressed.
typedef struct EranosTipValue EranosTipValue;

/// A percentage of the expense's amount.
///
/// @param basisPoints The tip's share of the amount, in basis points
/// @return The tip value, which the caller destroys
[[nodiscard]] static inline EranosTipValue *eranosTipValueCreatePercentage(int32_t const basisPoints) {
    return EranosTipValue_create_percentage(basisPoints);
}

/// A fixed amount in the expense's currency.
///
/// @param amount The tip
/// @return The tip value, which the caller destroys
[[nodiscard]] static inline EranosTipValue *eranosTipValueCreateFixed(EranosMoney const *const amount) { return EranosTipValue_create_fixed(amount); }

/// Which kind of tip value it is.
///
/// @param tipValue The tip value
/// @return The result
[[nodiscard]] static inline EranosTipValueKind eranosTipValueKind(EranosTipValue const *const tipValue) { return EranosTipValue_kind(tipValue); }

/// The tip. Present in the kind ERANOS_ERROR_KIND_FIXED alone.
///
/// @param tipValue The tip value
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosMoney *eranosTipValueAmount(EranosTipValue const *const tipValue) { return EranosTipValue_amount(tipValue); }

/// The tip's share of the amount, in basis points. Present in the kind ERANOS_ERROR_KIND_PERCENTAGE alone.
///
/// @param tipValue The tip value
/// @param basisPoints Receives the basis points, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosTipValueBasisPoints(EranosTipValue const *const tipValue, int32_t *const basisPoints) {
    EranosTipValue_basis_points_result const result = EranosTipValue_basis_points(tipValue);
    return eranosDetailPresentInt32(result.is_ok, &result.ok, basisPoints);
}

/// Destroys @p tipValue; destroying nullptr does nothing, as free() does.
///
/// @param tipValue The tip value, which no call uses after this one, or nullptr
static inline void eranosTipValueDestroy(EranosTipValue *const tipValue) {
    if (nullptr != tipValue) {
        EranosTipValue_destroy(tipValue);
    }
}

#endif // ERANOS_EXPENSE_H
