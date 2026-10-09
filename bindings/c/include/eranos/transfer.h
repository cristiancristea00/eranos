/// @file eranos/transfer.h
/// Transfers: money one member paid another.

#ifndef ERANOS_TRANSFER_H
#define ERANOS_TRANSFER_H

#include "EranosTransfer.h"
#include "EranosTransferList.h"
#include "eranos/calendar.h"
#include "eranos/detail.h"
#include "eranos/error.h"
#include "eranos/expense.h"
#include "eranos/identifiers.h"
#include "eranos/money.h"
#include "eranos/text.h"
#include "eranos/totals.h"

#include <stddef.h>

/// A transfer: money one member paid another.
typedef struct EranosTransfer EranosTransfer;

/// A new transfer of its required fields under the identifier @p id, such as the key an app's own store gives it; every other field starts absent,
/// empty, or at its default. eranosTransferCreate() generates the identifier instead.
///
/// @param id The identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes, as UTF-8 ended by a NUL; nullptr reads as empty
///   text
/// @param sender The identifier of the member who paid, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param receiver The identifier of the member who was paid, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cost What was paid
/// @param date The day it was paid
/// @param transfer Receives the transfer on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTransferCreateWithId(char const *const id, char const *const sender, char const *const receiver,
                                                                    EranosCost const *const cost, EranosCalendarDate const date,
                                                                    EranosTransfer **const transfer) {
    EranosTransfer_create_result const result =
        EranosTransfer_create(eranosDetailView(id), eranosDetailView(sender), eranosDetailView(receiver), cost, date);
    *transfer = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// A new transfer of its required fields under an identifier generated when it is called, which eranosTransferCreateWithId() takes instead; every
/// other field starts absent, empty, or at its default.
///
/// @param sender The identifier of the member who paid, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param receiver The identifier of the member who was paid, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param cost What was paid
/// @param date The day it was paid
/// @param transfer Receives the transfer on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTransferCreate(char const *const sender, char const *const receiver, EranosCost const *const cost,
                                                              EranosCalendarDate const date, EranosTransfer **const transfer) {
    EranosText *const id = eranosTextCreate();
    eranosIdentifierGenerate(id);
    EranosError *const error = eranosTransferCreateWithId(eranosTextData(id), sender, receiver, cost, date, transfer);
    eranosTextDestroy(id);
    return error;
}

/// The transfer that records @p payment of a settlement, made on @p date, under the identifier @p id: from its sender to its receiver, of its
/// amount in the group currency; eranosTransferCreateFromPayment() generates the identifier instead.
///
/// @param id The identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes, as UTF-8 ended by a NUL; nullptr reads as empty
///   text
/// @param payment The payment
/// @param date The day it was paid
/// @param transfer Receives the transfer on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTransferCreateFromPaymentWithId(char const *const id, EranosPayment const *const payment,
                                                                               EranosCalendarDate const date, EranosTransfer **const transfer) {
    EranosText *const sender = eranosTextCreate();
    EranosText *const receiver = eranosTextCreate();
    eranosPaymentSender(payment, sender);
    eranosPaymentReceiver(payment, receiver);
    EranosMoney *const amount = eranosPaymentAmount(payment);
    EranosCost *const cost = eranosCostCreateLocal(amount);
    EranosError *const error = eranosTransferCreateWithId(id, eranosTextData(sender), eranosTextData(receiver), cost, date, transfer);
    eranosCostDestroy(cost);
    eranosMoneyDestroy(amount);
    eranosTextDestroy(receiver);
    eranosTextDestroy(sender);
    return error;
}

/// The transfer that records @p payment of a settlement, made on @p date, under an identifier generated when it is called, which
/// eranosTransferCreateFromPaymentWithId() takes instead.
///
/// @param payment The payment
/// @param date The day it was paid
/// @return The transfer, which the caller destroys
[[nodiscard]] static inline EranosTransfer *eranosTransferCreateFromPayment(EranosPayment const *const payment, EranosCalendarDate const date) {
    EranosText *const id = eranosTextCreate();
    eranosIdentifierGenerate(id);
    EranosTransfer *transfer = nullptr;
    // A generated identifier and the payment's members are UTF-8, so the transfer is made.
    eranosErrorDestroy(eranosTransferCreateFromPaymentWithId(eranosTextData(id), payment, date, &transfer));
    eranosTextDestroy(id);
    return transfer;
}

/// The identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes.
///
/// @param transfer The transfer
/// @param id Receives the identifier, replacing what it held
static inline void eranosTransferId(EranosTransfer const *const transfer, EranosText *const id) {
    eranosDetailClear(id);
    EranosTransfer_id(transfer, id);
    eranosDetailFinish(id);
}

/// Sets the identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes.
///
/// @param transfer The transfer
/// @param id The identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTransferSetId(EranosTransfer *const transfer, char const *const id) {
    EranosTransfer_set_id_result const result = EranosTransfer_set_id(transfer, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The identifier of the member who paid.
///
/// @param transfer The transfer
/// @param sender Receives the identifier of the member who paid, replacing what it held
static inline void eranosTransferSender(EranosTransfer const *const transfer, EranosText *const sender) {
    eranosDetailClear(sender);
    EranosTransfer_sender(transfer, sender);
    eranosDetailFinish(sender);
}

/// Sets the identifier of the member who paid.
///
/// @param transfer The transfer
/// @param sender The identifier of the member who paid, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTransferSetSender(EranosTransfer *const transfer, char const *const sender) {
    EranosTransfer_set_sender_result const result = EranosTransfer_set_sender(transfer, eranosDetailView(sender));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The identifier of the member who was paid.
///
/// @param transfer The transfer
/// @param receiver Receives the identifier of the member who was paid, replacing what it held
static inline void eranosTransferReceiver(EranosTransfer const *const transfer, EranosText *const receiver) {
    eranosDetailClear(receiver);
    EranosTransfer_receiver(transfer, receiver);
    eranosDetailFinish(receiver);
}

/// Sets the identifier of the member who was paid.
///
/// @param transfer The transfer
/// @param receiver The identifier of the member who was paid, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTransferSetReceiver(EranosTransfer *const transfer, char const *const receiver) {
    EranosTransfer_set_receiver_result const result = EranosTransfer_set_receiver(transfer, eranosDetailView(receiver));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// What was paid.
///
/// @param transfer The transfer
/// @return A copy of the value, which the caller destroys
[[nodiscard]] static inline EranosCost *eranosTransferCost(EranosTransfer const *const transfer) { return EranosTransfer_cost(transfer); }

/// Sets what was paid.
///
/// @param transfer The transfer
/// @param cost What was paid
static inline void eranosTransferSetCost(EranosTransfer *const transfer, EranosCost const *const cost) { EranosTransfer_set_cost(transfer, cost); }

/// The day it was paid.
///
/// @param transfer The transfer
/// @return The day it was paid
[[nodiscard]] static inline EranosCalendarDate eranosTransferDate(EranosTransfer const *const transfer) { return EranosTransfer_date(transfer); }

/// Sets the day it was paid.
///
/// @param transfer The transfer
/// @param date The day it was paid
static inline void eranosTransferSetDate(EranosTransfer *const transfer, EranosCalendarDate const date) { EranosTransfer_set_date(transfer, date); }

/// The time of day it was paid.
///
/// @param transfer The transfer
/// @param time Receives the time, when there is one
/// @return Whether there is one
[[nodiscard]] static inline bool eranosTransferTime(EranosTransfer const *const transfer, EranosTimeOfDay *const time) {
    EranosTransfer_time_result const result = EranosTransfer_time(transfer);
    return eranosDetailPresentTimeOfDay(result.is_ok, &result.ok, time);
}

/// Sets the time of day it was paid.
///
/// @param transfer The transfer
/// @param time The time of day it was paid
static inline void eranosTransferSetTime(EranosTransfer *const transfer, EranosTimeOfDay const time) { EranosTransfer_set_time(transfer, time); }

/// Clears the time of day it was paid.
///
/// @param transfer The transfer
static inline void eranosTransferClearTime(EranosTransfer *const transfer) { EranosTransfer_clear_time(transfer); }

/// A note on the transfer. Writes it, if present; absent otherwise.
///
/// @param transfer The transfer
/// @param note Receives a note on the transfer, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosTransferNote(EranosTransfer const *const transfer, EranosText *const note) {
    eranosDetailClear(note);
    bool const present = EranosTransfer_note(transfer, note).is_ok;
    eranosDetailFinish(note);
    return present;
}

/// Sets a note on the transfer.
///
/// @param transfer The transfer
/// @param note A note on the transfer, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTransferSetNote(EranosTransfer *const transfer, char const *const note) {
    EranosTransfer_set_note_result const result = EranosTransfer_set_note(transfer, eranosDetailView(note));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears a note on the transfer.
///
/// @param transfer The transfer
static inline void eranosTransferClearNote(EranosTransfer *const transfer) { EranosTransfer_clear_note(transfer); }

/// Destroys @p transfer; destroying nullptr does nothing, as free() does.
///
/// @param transfer The transfer, which no call uses after this one, or nullptr
static inline void eranosTransferDestroy(EranosTransfer *const transfer) {
    if (nullptr != transfer) {
        EranosTransfer_destroy(transfer);
    }
}

/// A list of transfer records.
typedef struct EranosTransferList EranosTransferList;

/// How many records the list holds.
///
/// @param list The transfer list
/// @return The result
[[nodiscard]] static inline size_t eranosTransferListLength(EranosTransferList const *const list) { return EranosTransferList_len(list); }

/// Whether the list holds no record.
///
/// @param list The transfer list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosTransferListIsEmpty(EranosTransferList const *const list) { return EranosTransferList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The transfer list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosTransfer *eranosTransferListGet(EranosTransferList const *const list, size_t const index) {
    return EranosTransferList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The transfer list, which no call uses after this one, or nullptr
static inline void eranosTransferListDestroy(EranosTransferList *const list) {
    if (nullptr != list) {
        EranosTransferList_destroy(list);
    }
}

#endif // ERANOS_TRANSFER_H
