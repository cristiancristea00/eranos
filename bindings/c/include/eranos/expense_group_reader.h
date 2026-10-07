/// @file eranos/expense_group_reader.h
/// The read handle, which holds a group's shared lock for the reads of one report.

#ifndef ERANOS_EXPENSE_GROUP_READER_H
#define ERANOS_EXPENSE_GROUP_READER_H

#include "EranosExpenseGroupReader.h"
#include "eranos/category.h"
#include "eranos/detail.h"
#include "eranos/expense.h"
#include "eranos/members.h"
#include "eranos/money.h"
#include "eranos/text.h"
#include "eranos/totals.h"
#include "eranos/transfer.h"

/// A read handle: it holds the group's shared lock until it is released or destroyed, so the reads of one report see no change in between; a change
/// waits for it.
typedef struct EranosExpenseGroupReader EranosExpenseGroupReader;

/// Releases the lock at once; later reads report the kind ERANOS_ERROR_KIND_RELEASED.
///
/// @param reader The group reader
static inline void eranosExpenseGroupReaderRelease(EranosExpenseGroupReader *const reader) { EranosExpenseGroupReader_release(reader); }

/// The group's identifier. Writes it into @p id.
///
/// @param reader The group reader
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderId(EranosExpenseGroupReader const *const reader, EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroupReader_id_result const result = EranosExpenseGroupReader_id(reader, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers.
///
/// @param reader The group reader
/// @param categories Receives the categories on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderCategories(EranosExpenseGroupReader const *const reader,
                                                                            EranosCategoryList **const categories) {
    EranosExpenseGroupReader_categories_result const result = EranosExpenseGroupReader_categories(reader);
    *categories = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The category with the identifier `id`.
///
/// @param reader The group reader
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param category Receives the category on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderCategory(EranosExpenseGroupReader const *const reader, char const *const id,
                                                                          EranosCategory **const category) {
    EranosExpenseGroupReader_category_result const result = EranosExpenseGroupReader_category(reader, eranosDetailView(id));
    *category = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The ISO 4217 code of the group currency. Writes it into @p currency.
///
/// @param reader The group reader
/// @param currency Receives the currency, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderCurrency(EranosExpenseGroupReader const *const reader, EranosText *const currency) {
    eranosDetailClear(currency);
    EranosExpenseGroupReader_currency_result const result = EranosExpenseGroupReader_currency(reader, currency);
    eranosDetailFinish(currency);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every expense, in identifier order, which is creation order for generated identifiers.
///
/// @param reader The group reader
/// @param expenses Receives the expenses on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderExpenses(EranosExpenseGroupReader const *const reader,
                                                                          EranosExpenseList **const expenses) {
    EranosExpenseGroupReader_expenses_result const result = EranosExpenseGroupReader_expenses(reader);
    *expenses = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The expense with the identifier `id`.
///
/// @param reader The group reader
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expense Receives the expense on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderExpense(EranosExpenseGroupReader const *const reader, char const *const id,
                                                                         EranosExpense **const expense) {
    EranosExpenseGroupReader_expense_result const result = EranosExpenseGroupReader_expense(reader, eranosDetailView(id));
    *expense = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// How the expense with the identifier `id` divides, in the group currency.
///
/// @param reader The group reader
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expenseShares Receives the expense shares on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderExpenseShares(EranosExpenseGroupReader const *const reader, char const *const id,
                                                                               EranosExpenseShares **const expenseShares) {
    EranosExpenseGroupReader_expense_shares_result const result = EranosExpenseGroupReader_expense_shares(reader, eranosDetailView(id));
    *expenseShares = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's totals.
///
/// @param reader The group reader
/// @param summary Receives the summary on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderSummary(EranosExpenseGroupReader const *const reader,
                                                                         EranosSummary **const summary) {
    EranosExpenseGroupReader_summary_result const result = EranosExpenseGroupReader_summary(reader);
    *summary = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
///
/// @param reader The group reader
/// @param members Receives the members on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderMembers(EranosExpenseGroupReader const *const reader,
                                                                         EranosMemberList **const members) {
    EranosExpenseGroupReader_members_result const result = EranosExpenseGroupReader_members(reader);
    *members = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The member with the identifier `id`.
///
/// @param reader The group reader
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param member Receives the member on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderMember(EranosExpenseGroupReader const *const reader, char const *const id,
                                                                        EranosMember **const member) {
    EranosExpenseGroupReader_member_result const result = EranosExpenseGroupReader_member(reader, eranosDetailView(id));
    *member = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Whether the member with the identifier `id` is archived; `false` for an unknown member.
///
/// @param reader The group reader
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param archived Receives the archived on success
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderIsArchived(EranosExpenseGroupReader const *const reader, char const *const id,
                                                                            bool *const archived) {
    EranosExpenseGroupReader_is_archived_result const result = EranosExpenseGroupReader_is_archived(reader, eranosDetailView(id));
    *archived = false;
    if (result.is_ok) {
        *archived = result.ok;
    }
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's name. Writes it into @p name.
///
/// @param reader The group reader
/// @param name Receives the name, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderName(EranosExpenseGroupReader const *const reader, EranosText *const name) {
    eranosDetailClear(name);
    EranosExpenseGroupReader_name_result const result = EranosExpenseGroupReader_name(reader, name);
    eranosDetailFinish(name);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The payments that settle every balance by `strategy`.
///
/// @param reader The group reader
/// @param strategy The strategy
/// @param settlement Receives the settlement on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderSettlement(EranosExpenseGroupReader const *const reader,
                                                                            EranosSettlementStrategy const strategy,
                                                                            EranosSettlementPlan **const settlement) {
    EranosExpenseGroupReader_settlement_result const result = EranosExpenseGroupReader_settlement(reader, strategy);
    *settlement = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every transfer, in identifier order, which is creation order for generated identifiers.
///
/// @param reader The group reader
/// @param transfers Receives the transfers on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderTransfers(EranosExpenseGroupReader const *const reader,
                                                                           EranosTransferList **const transfers) {
    EranosExpenseGroupReader_transfers_result const result = EranosExpenseGroupReader_transfers(reader);
    *transfers = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The transfer with the identifier `id`.
///
/// @param reader The group reader
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transfer Receives the transfer on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderTransfer(EranosExpenseGroupReader const *const reader, char const *const id,
                                                                          EranosTransfer **const transfer) {
    EranosExpenseGroupReader_transfer_result const result = EranosExpenseGroupReader_transfer(reader, eranosDetailView(id));
    *transfer = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The amount in the group currency of the transfer with the identifier `id`.
///
/// @param reader The group reader
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transferAmount Receives the transfer amount on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: The library's refusal, and the error of the kind
///   ERANOS_ERROR_KIND_RELEASED after `release`; the error of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderTransferAmount(EranosExpenseGroupReader const *const reader, char const *const id,
                                                                                EranosMoney **const transferAmount) {
    EranosExpenseGroupReader_transfer_amount_result const result = EranosExpenseGroupReader_transfer_amount(reader, eranosDetailView(id));
    *transferAmount = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's description. Writes it into @p description, if the group has one, and nothing otherwise: the group holds no blank text there, since
/// blank text clears it.
///
/// @param reader The group reader
/// @param description Receives the description, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderDescription(EranosExpenseGroupReader const *const reader,
                                                                             EranosText *const description) {
    eranosDetailClear(description);
    EranosExpenseGroupReader_description_result const result = EranosExpenseGroupReader_description(reader, description);
    eranosDetailFinish(description);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The language the apps show the group in, as a BCP 47 tag. Writes it into @p language, if the group has one, and nothing otherwise: the group holds
/// no blank text there, since blank text clears it.
///
/// @param reader The group reader
/// @param language Receives the language, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderLanguage(EranosExpenseGroupReader const *const reader, EranosText *const language) {
    eranosDetailClear(language);
    EranosExpenseGroupReader_language_result const result = EranosExpenseGroupReader_language(reader, language);
    eranosDetailFinish(language);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group in its saved form, as JSON. Writes it into @p json.
///
/// @param reader The group reader
/// @param json Receives the json, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   `release`
[[nodiscard]] static inline EranosError *eranosExpenseGroupReaderSave(EranosExpenseGroupReader const *const reader, EranosText *const json) {
    eranosDetailClear(json);
    EranosExpenseGroupReader_save_result const result = EranosExpenseGroupReader_save(reader, json);
    eranosDetailFinish(json);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p reader; destroying nullptr does nothing, as free() does.
///
/// @param reader The group reader, which no call uses after this one, or nullptr
static inline void eranosExpenseGroupReaderDestroy(EranosExpenseGroupReader *const reader) {
    if (nullptr != reader) {
        EranosExpenseGroupReader_destroy(reader);
    }
}

#endif // ERANOS_EXPENSE_GROUP_READER_H
