/// @file eranos/expense_group_writer.h
/// The write handle, which holds a group's exclusive lock for the reads and changes of one piece of work.

#ifndef ERANOS_EXPENSE_GROUP_WRITER_H
#define ERANOS_EXPENSE_GROUP_WRITER_H

#include "EranosExpenseGroupWriter.h"
#include "eranos/category.h"
#include "eranos/currency_code.h"
#include "eranos/detail.h"
#include "eranos/expense.h"
#include "eranos/members.h"
#include "eranos/money.h"
#include "eranos/text.h"
#include "eranos/totals.h"
#include "eranos/transfer.h"

/// A write handle: it holds the group's exclusive lock until it is released or destroyed, so its reads and changes run with no other call in
/// between; every other call waits for it.
typedef struct EranosExpenseGroupWriter EranosExpenseGroupWriter;

/// Releases the lock at once; later calls report the kind ERANOS_ERROR_KIND_RELEASED.
///
/// @param writer The group writer
static inline void eranosExpenseGroupWriterRelease(EranosExpenseGroupWriter *const writer) { EranosExpenseGroupWriter_release(writer); }

/// The group's identifier. Writes it into @p id.
///
/// @param writer The group writer
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterId(EranosExpenseGroupWriter const *const writer, EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroupWriter_id_result const result = EranosExpenseGroupWriter_id(writer, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers.
///
/// @param writer The group writer
/// @param categories Receives the categories on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterCategories(EranosExpenseGroupWriter const *const writer,
                                                                            EranosCategoryList **const categories) {
    EranosExpenseGroupWriter_categories_result const result = EranosExpenseGroupWriter_categories(writer);
    *categories = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The category with the identifier `id`.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param category Receives the category, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no category has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterCategory(EranosExpenseGroupWriter const *const writer, char const *const id,
                                                                          EranosCategory **const category) {
    EranosExpenseGroupWriter_category_result const result = EranosExpenseGroupWriter_category(writer, eranosDetailView(id));
    *category = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group currency, with what ISO 4217 says of it.
///
/// @param writer The group writer
/// @param currency Receives the currency details on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterCurrency(EranosExpenseGroupWriter const *const writer,
                                                                          EranosCurrencyDetails **const currency) {
    EranosExpenseGroupWriter_currency_result const result = EranosExpenseGroupWriter_currency(writer);
    *currency = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every expense, in identifier order, which is creation order for generated identifiers.
///
/// @param writer The group writer
/// @param expenses Receives the expenses on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterExpenses(EranosExpenseGroupWriter const *const writer,
                                                                          EranosExpenseList **const expenses) {
    EranosExpenseGroupWriter_expenses_result const result = EranosExpenseGroupWriter_expenses(writer);
    *expenses = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The expense with the identifier `id`.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expense Receives the expense, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no expense has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterExpense(EranosExpenseGroupWriter const *const writer, char const *const id,
                                                                         EranosExpense **const expense) {
    EranosExpenseGroupWriter_expense_result const result = EranosExpenseGroupWriter_expense(writer, eranosDetailView(id));
    *expense = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// How the expense with the identifier `id` divides, in the group currency.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param expenseShares Receives the expense shares, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error
///   means that no expense has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterExpenseShares(EranosExpenseGroupWriter const *const writer, char const *const id,
                                                                               EranosExpenseShares **const expenseShares) {
    EranosExpenseGroupWriter_expense_shares_result const result = EranosExpenseGroupWriter_expense_shares(writer, eranosDetailView(id));
    *expenseShares = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's totals.
///
/// @param writer The group writer
/// @param summary Receives the summary on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterSummary(EranosExpenseGroupWriter const *const writer,
                                                                         EranosSummary **const summary) {
    EranosExpenseGroupWriter_summary_result const result = EranosExpenseGroupWriter_summary(writer);
    *summary = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
///
/// @param writer The group writer
/// @param members Receives the members on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterMembers(EranosExpenseGroupWriter const *const writer,
                                                                         EranosMemberList **const members) {
    EranosExpenseGroupWriter_members_result const result = EranosExpenseGroupWriter_members(writer);
    *members = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The member with the identifier `id`.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param member Receives the member, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no member has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterMember(EranosExpenseGroupWriter const *const writer, char const *const id,
                                                                        EranosMember **const member) {
    EranosExpenseGroupWriter_member_result const result = EranosExpenseGroupWriter_member(writer, eranosDetailView(id));
    *member = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Whether the member with the identifier `id` is archived. Writes it into @p archived when the group holds the member, and nothing otherwise:
/// @p found false with no error means that no member has the identifier.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param found Receives whether the group holds the member, which is false when the call fails
/// @param archived Receives whether the member is archived, when the group holds them
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterIsArchived(EranosExpenseGroupWriter const *const writer, char const *const id,
                                                                            bool *const found, bool *const archived) {
    EranosExpenseGroupWriter_is_archived_result const result = EranosExpenseGroupWriter_is_archived(writer, eranosDetailView(id));
    *found = result.is_ok && eranosDetailPresentBool(result.ok.is_ok, &result.ok.ok, archived);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's name. Writes it into @p name.
///
/// @param writer The group writer
/// @param name Receives the name, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterName(EranosExpenseGroupWriter const *const writer, EranosText *const name) {
    eranosDetailClear(name);
    EranosExpenseGroupWriter_name_result const result = EranosExpenseGroupWriter_name(writer, name);
    eranosDetailFinish(name);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The payments that settle every balance by `strategy`.
///
/// @param writer The group writer
/// @param strategy The strategy
/// @param settlement Receives the settlement on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterSettlement(EranosExpenseGroupWriter const *const writer,
                                                                            EranosSettlementStrategy const strategy,
                                                                            EranosSettlementPlan **const settlement) {
    EranosExpenseGroupWriter_settlement_result const result = EranosExpenseGroupWriter_settlement(writer, strategy);
    *settlement = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Every transfer, in identifier order, which is creation order for generated identifiers.
///
/// @param writer The group writer
/// @param transfers Receives the transfers on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterTransfers(EranosExpenseGroupWriter const *const writer,
                                                                           EranosTransferList **const transfers) {
    EranosExpenseGroupWriter_transfers_result const result = EranosExpenseGroupWriter_transfers(writer);
    *transfers = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The transfer with the identifier `id`.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transfer Receives the transfer, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error means that
///   no transfer has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterTransfer(EranosExpenseGroupWriter const *const writer, char const *const id,
                                                                          EranosTransfer **const transfer) {
    EranosExpenseGroupWriter_transfer_result const result = EranosExpenseGroupWriter_transfer(writer, eranosDetailView(id));
    *transfer = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The amount in the group currency of the transfer with the identifier `id`.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param transferAmount Receives the transfer amount, which the caller destroys, or nullptr when the call fails or finds none: nullptr with no error
///   means that no transfer has the identifier
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, such as one of the kind
///   ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterTransferAmount(EranosExpenseGroupWriter const *const writer, char const *const id,
                                                                                EranosMoney **const transferAmount) {
    EranosExpenseGroupWriter_transfer_amount_result const result = EranosExpenseGroupWriter_transfer_amount(writer, eranosDetailView(id));
    *transferAmount = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group's note. Writes it into @p note, if the group has one, and nothing otherwise: the group holds no blank text there, since
/// blank text clears it.
///
/// @param writer The group writer
/// @param note Receives the note, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterNote(EranosExpenseGroupWriter const *const writer, EranosText *const note) {
    eranosDetailClear(note);
    EranosExpenseGroupWriter_note_result const result = EranosExpenseGroupWriter_note(writer, note);
    eranosDetailFinish(note);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The language the apps show the group in, as a BCP 47 tag. Writes it into @p language, if the group has one, and nothing otherwise: the group holds
/// no blank text there, since blank text clears it.
///
/// @param writer The group writer
/// @param language Receives the language, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterLanguage(EranosExpenseGroupWriter const *const writer, EranosText *const language) {
    eranosDetailClear(language);
    EranosExpenseGroupWriter_language_result const result = EranosExpenseGroupWriter_language(writer, language);
    eranosDetailFinish(language);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The group in its saved form, as JSON. Writes it into @p json.
///
/// @param writer The group writer
/// @param json Receives the json, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
///   eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterSave(EranosExpenseGroupWriter const *const writer, EranosText *const json) {
    eranosDetailClear(json);
    EranosExpenseGroupWriter_save_result const result = EranosExpenseGroupWriter_save(writer, json);
    eranosDetailFinish(json);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Renames the group.
///
/// @param writer The group writer
/// @param name The name, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterSetName(EranosExpenseGroupWriter *const writer, char const *const name) {
    EranosExpenseGroupWriter_set_name_result const result = EranosExpenseGroupWriter_set_name(writer, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Moves the group to `currency`, with a rate into it for every other currency its entries use.
///
/// @param writer The group writer
/// @param currency The currency, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param rates The rates
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterChangeCurrency(EranosExpenseGroupWriter *const writer, char const *const currency,
                                                                                EranosCurrencyRateList const *const rates) {
    EranosExpenseGroupWriter_change_currency_result const result =
        EranosExpenseGroupWriter_change_currency(writer, eranosDetailView(currency), rates);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Sets the note, trimmed, or clears it when the text is blank.
///
/// @param writer The group writer
/// @param note The note, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
/// eranosExpenseGroupWriterRelease(), or one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterSetNote(EranosExpenseGroupWriter *const writer, char const *const note) {
    EranosExpenseGroupWriter_set_note_result const result = EranosExpenseGroupWriter_set_note(writer, eranosDetailView(note));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the group's note.
///
/// @param writer The group writer
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
/// eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterClearNote(EranosExpenseGroupWriter *const writer) {
    EranosExpenseGroupWriter_clear_note_result const result = EranosExpenseGroupWriter_clear_note(writer);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Sets the language the apps show the group in, a BCP 47 tag, or clears it when the text is blank.
///
/// @param writer The group writer
/// @param language The language, as UTF-8 ended by a NUL; nullptr or blank text clears it
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
/// eranosExpenseGroupWriterRelease(), one of the kind ERANOS_ERROR_KIND_LANGUAGE for text that is not a language tag, or of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterSetLanguage(EranosExpenseGroupWriter *const writer, char const *const language) {
    EranosExpenseGroupWriter_set_language_result const result = EranosExpenseGroupWriter_set_language(writer, eranosDetailView(language));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the language the apps show the group in.
///
/// @param writer The group writer
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_RELEASED after
/// eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterClearLanguage(EranosExpenseGroupWriter *const writer) {
    EranosExpenseGroupWriter_clear_language_result const result = EranosExpenseGroupWriter_clear_language(writer);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Adds `member` under their identifier, and returns the identifier. Writes it into @p id.
///
/// @param writer The group writer
/// @param member The member
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterAddMember(EranosExpenseGroupWriter *const writer, EranosMember const *const member,
                                                                           EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroupWriter_add_member_result const result = EranosExpenseGroupWriter_add_member(writer, member, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Replaces the member that `member` names by their identifier, and returns them as they were.
///
/// @param writer The group writer
/// @param member The member
/// @param previous Receives the member as they were before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterReplaceMember(EranosExpenseGroupWriter *const writer,
                                                                               EranosMember const *const member, EranosMember **const previous) {
    EranosExpenseGroupWriter_replace_member_result const result = EranosExpenseGroupWriter_replace_member(writer, member);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Archives the member with the identifier `id`.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterArchiveMember(EranosExpenseGroupWriter *const writer, char const *const id) {
    EranosExpenseGroupWriter_archive_member_result const result = EranosExpenseGroupWriter_archive_member(writer, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Unarchives the member with the identifier `id`.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterUnarchiveMember(EranosExpenseGroupWriter *const writer, char const *const id) {
    EranosExpenseGroupWriter_unarchive_member_result const result = EranosExpenseGroupWriter_unarchive_member(writer, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Removes the member with the identifier `id`, whom no entry references, and returns them.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the member the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterRemoveMember(EranosExpenseGroupWriter *const writer, char const *const id,
                                                                              EranosMember **const removed) {
    EranosExpenseGroupWriter_remove_member_result const result = EranosExpenseGroupWriter_remove_member(writer, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Adds `category` under its identifier, and returns the identifier. Writes it into @p id.
///
/// @param writer The group writer
/// @param category The category
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterAddCategory(EranosExpenseGroupWriter *const writer,
                                                                             EranosCategory const *const category, EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroupWriter_add_category_result const result = EranosExpenseGroupWriter_add_category(writer, category, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Renames the category that `category` names by its identifier, and returns it as it was.
///
/// @param writer The group writer
/// @param category The category
/// @param previous Receives the category as it was before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterReplaceCategory(EranosExpenseGroupWriter *const writer,
                                                                                 EranosCategory const *const category,
                                                                                 EranosCategory **const previous) {
    EranosExpenseGroupWriter_replace_category_result const result = EranosExpenseGroupWriter_replace_category(writer, category);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Removes the category with the identifier `id` and returns it; its expenses move to the built-in one.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the category the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterRemoveCategory(EranosExpenseGroupWriter *const writer, char const *const id,
                                                                                EranosCategory **const removed) {
    EranosExpenseGroupWriter_remove_category_result const result = EranosExpenseGroupWriter_remove_category(writer, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Adds `expense` under its identifier, and returns the identifier. Writes it into @p id.
///
/// @param writer The group writer
/// @param expense The expense
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterAddExpense(EranosExpenseGroupWriter *const writer,
                                                                            EranosExpense const *const expense, EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroupWriter_add_expense_result const result = EranosExpenseGroupWriter_add_expense(writer, expense, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Replaces the expense that `expense` names by its identifier, and returns it as it was.
///
/// @param writer The group writer
/// @param expense The expense
/// @param previous Receives the expense as it was before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterReplaceExpense(EranosExpenseGroupWriter *const writer,
                                                                                EranosExpense const *const expense, EranosExpense **const previous) {
    EranosExpenseGroupWriter_replace_expense_result const result = EranosExpenseGroupWriter_replace_expense(writer, expense);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Removes the expense with the identifier `id` and returns it.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the expense the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterRemoveExpense(EranosExpenseGroupWriter *const writer, char const *const id,
                                                                               EranosExpense **const removed) {
    EranosExpenseGroupWriter_remove_expense_result const result = EranosExpenseGroupWriter_remove_expense(writer, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Adds `transfer` under its identifier, and returns the identifier. Writes it into @p id.
///
/// @param writer The group writer
/// @param transfer The transfer
/// @param id Receives the id, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterAddTransfer(EranosExpenseGroupWriter *const writer,
                                                                             EranosTransfer const *const transfer, EranosText *const id) {
    eranosDetailClear(id);
    EranosExpenseGroupWriter_add_transfer_result const result = EranosExpenseGroupWriter_add_transfer(writer, transfer, id);
    eranosDetailFinish(id);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
///
/// @param writer The group writer
/// @param transfer The transfer
/// @param previous Receives the transfer as it was before the call, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, or one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease()
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterReplaceTransfer(EranosExpenseGroupWriter *const writer,
                                                                                 EranosTransfer const *const transfer,
                                                                                 EranosTransfer **const previous) {
    EranosExpenseGroupWriter_replace_transfer_result const result = EranosExpenseGroupWriter_replace_transfer(writer, transfer);
    *previous = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Removes the transfer with the identifier `id` and returns it.
///
/// @param writer The group writer
/// @param id The id, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param removed Receives the transfer the call removed, on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: the library's refusal, whose kind names the module
/// that refused the call, one of the kind ERANOS_ERROR_KIND_RELEASED after eranosExpenseGroupWriterRelease(), or one of the kind
/// ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosExpenseGroupWriterRemoveTransfer(EranosExpenseGroupWriter *const writer, char const *const id,
                                                                                EranosTransfer **const removed) {
    EranosExpenseGroupWriter_remove_transfer_result const result = EranosExpenseGroupWriter_remove_transfer(writer, eranosDetailView(id));
    *removed = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p writer; destroying nullptr does nothing, as free() does.
///
/// @param writer The group writer, which no call uses after this one, or nullptr
static inline void eranosExpenseGroupWriterDestroy(EranosExpenseGroupWriter *const writer) {
    if (nullptr != writer) {
        EranosExpenseGroupWriter_destroy(writer);
    }
}

#endif // ERANOS_EXPENSE_GROUP_WRITER_H
