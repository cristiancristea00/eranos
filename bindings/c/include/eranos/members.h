/// @file eranos/members.h
/// The members of a group and the accounts they can be paid into.

#ifndef ERANOS_MEMBERS_H
#define ERANOS_MEMBERS_H

#include "EranosAccount.h"
#include "EranosAccountDetails.h"
#include "EranosAccountDetailsKind.h"
#include "EranosAccountList.h"
#include "EranosMember.h"
#include "EranosMemberList.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stdbool.h>
#include <stddef.h>

/// The kind of a account details. The values of EranosAccountDetailsKind, by the names of the C layer.
typedef enum EranosAccountDetailsKind EranosAccountDetailsKind;

/// The values of EranosAccountDetailsKind.
enum {
    /// A bank account.
    ERANOS_ACCOUNT_DETAILS_KIND_IBAN = EranosAccountDetailsKind_Iban,
    /// A username in a payment app.
    ERANOS_ACCOUNT_DETAILS_KIND_USERNAME = EranosAccountDetailsKind_Username,
};

/// An account a member can be paid into.
typedef struct EranosAccount EranosAccount;

/// A new account of its required fields; every other field starts absent, empty, or at its default.
///
/// @param details The IBAN or the username
/// @param name The label, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param account Receives the account on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosAccountCreate(EranosAccountDetails const *const details, char const *const name, EranosAccount **const account) {
    EranosAccount_create_result const result = EranosAccount_create(details, eranosDetailView(name));
    *account = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The IBAN or the username.
///
/// @param account The account
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosAccountDetails *eranosAccountDetails(EranosAccount const *const account) { return EranosAccount_details(account); }

/// Sets the IBAN or the username.
///
/// @param account The account
/// @param details The IBAN or the username
static inline void eranosAccountSetDetails(EranosAccount *const account, EranosAccountDetails const *const details) {
    EranosAccount_set_details(account, details);
}

/// The label.
///
/// @param account The account
/// @param name Receives the label, replacing what it held
static inline void eranosAccountName(EranosAccount const *const account, EranosText *const name) {
    eranosDetailClear(name);
    EranosAccount_name(account, name);
    eranosDetailFinish(name);
}

/// Sets the label.
///
/// @param account The account
/// @param name The label, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosAccountSetName(EranosAccount *const account, char const *const name) {
    EranosAccount_set_name_result const result = EranosAccount_set_name(account, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p account; destroying NULL does nothing, as free() does.
///
/// @param account The account, which no call uses after this one, or NULL
static inline void eranosAccountDestroy(EranosAccount *const account) {
    if (NULL != account) {
        EranosAccount_destroy(account);
    }
}

/// Where an account is: a bank account or a username in a payment app.
typedef struct EranosAccountDetails EranosAccountDetails;

/// A bank account.
///
/// @param iban The account's IBAN, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param accountDetails Receives the account details on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosAccountDetailsCreateIban(char const *const iban, EranosAccountDetails **const accountDetails) {
    EranosAccountDetails_create_iban_result const result = EranosAccountDetails_create_iban(eranosDetailView(iban));
    *accountDetails = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// A username in a payment app.
///
/// @param username The username, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param accountDetails Receives the account details on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosAccountDetailsCreateUsername(char const *const username, EranosAccountDetails **const accountDetails) {
    EranosAccountDetails_create_username_result const result = EranosAccountDetails_create_username(eranosDetailView(username));
    *accountDetails = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Which kind of account details it is.
///
/// @param accountDetails The account details
/// @return The result
static inline EranosAccountDetailsKind eranosAccountDetailsKind(EranosAccountDetails const *const accountDetails) {
    return EranosAccountDetails_kind(accountDetails);
}

/// The account's IBAN. Present in the kind ERANOS_ERROR_KIND_IBAN alone. Writes it, if present; absent otherwise.
///
/// @param accountDetails The account details
/// @param iban Receives the account's IBAN, replacing what it held
/// @return Whether there is one
static inline bool eranosAccountDetailsIban(EranosAccountDetails const *const accountDetails, EranosText *const iban) {
    eranosDetailClear(iban);
    bool const present = EranosAccountDetails_iban(accountDetails, iban).is_ok;
    eranosDetailFinish(iban);
    return present;
}

/// The username. Present in the kind ERANOS_ERROR_KIND_USERNAME alone. Writes it, if present; absent otherwise.
///
/// @param accountDetails The account details
/// @param username Receives the username, replacing what it held
/// @return Whether there is one
static inline bool eranosAccountDetailsUsername(EranosAccountDetails const *const accountDetails, EranosText *const username) {
    eranosDetailClear(username);
    bool const present = EranosAccountDetails_username(accountDetails, username).is_ok;
    eranosDetailFinish(username);
    return present;
}

/// Destroys @p accountDetails; destroying NULL does nothing, as free() does.
///
/// @param accountDetails The account details, which no call uses after this one, or NULL
static inline void eranosAccountDetailsDestroy(EranosAccountDetails *const accountDetails) {
    if (NULL != accountDetails) {
        EranosAccountDetails_destroy(accountDetails);
    }
}

/// A list of account records.
typedef struct EranosAccountList EranosAccountList;

/// An empty list.
///
/// @return The account list, which the caller destroys
static inline EranosAccountList *eranosAccountListCreate(void) { return EranosAccountList_create(); }

/// Adds a copy of `element` at the end.
///
/// @param list The account list
/// @param element The element
static inline void eranosAccountListPush(EranosAccountList *const list, EranosAccount const *const element) { EranosAccountList_push(list, element); }

/// How many records the list holds.
///
/// @param list The account list
/// @return The result
static inline size_t eranosAccountListLength(EranosAccountList const *const list) { return EranosAccountList_len(list); }

/// Whether the list holds no record.
///
/// @param list The account list
/// @return Whether it holds none
static inline bool eranosAccountListIsEmpty(EranosAccountList const *const list) { return EranosAccountList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The account list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosAccount *eranosAccountListGet(EranosAccountList const *const list, size_t const index) {
    return EranosAccountList_get(list, index);
}

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The account list, which no call uses after this one, or NULL
static inline void eranosAccountListDestroy(EranosAccountList *const list) {
    if (NULL != list) {
        EranosAccountList_destroy(list);
    }
}

/// A member of the group.
typedef struct EranosMember EranosMember;

/// A new member of its required fields; every other field starts absent, empty, or at its default.
///
/// @param firstName The first name, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param member Receives the member on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosMemberCreate(char const *const firstName, EranosMember **const member) {
    EranosMember_create_result const result = EranosMember_create(eranosDetailView(firstName));
    *member = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The identifier: absent for a member to add, present for one the group holds. Writes it, if present; absent otherwise.
///
/// @param member The member
/// @param id Receives the identifier: absent for a member to add, present for one the group holds, replacing what it held
/// @return Whether there is one
static inline bool eranosMemberId(EranosMember const *const member, EranosText *const id) {
    eranosDetailClear(id);
    bool const present = EranosMember_id(member, id).is_ok;
    eranosDetailFinish(id);
    return present;
}

/// Sets the identifier: absent for a member to add, present for one the group holds.
///
/// @param member The member
/// @param id The identifier: absent for a member to add, present for one the group holds, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosMemberSetId(EranosMember *const member, char const *const id) {
    EranosMember_set_id_result const result = EranosMember_set_id(member, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the identifier: absent for a member to add, present for one the group holds.
///
/// @param member The member
static inline void eranosMemberClearId(EranosMember *const member) { EranosMember_clear_id(member); }

/// The first name.
///
/// @param member The member
/// @param firstName Receives the first name, replacing what it held
static inline void eranosMemberFirstName(EranosMember const *const member, EranosText *const firstName) {
    eranosDetailClear(firstName);
    EranosMember_first_name(member, firstName);
    eranosDetailFinish(firstName);
}

/// Sets the first name.
///
/// @param member The member
/// @param firstName The first name, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosMemberSetFirstName(EranosMember *const member, char const *const firstName) {
    EranosMember_set_first_name_result const result = EranosMember_set_first_name(member, eranosDetailView(firstName));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Where the member can be paid, in the order they were given.
///
/// @param member The member
/// @return A copy of the value, which the caller destroys, or NULL when there is none
static inline EranosAccountList *eranosMemberAccounts(EranosMember const *const member) { return EranosMember_accounts(member); }

/// Sets where the member can be paid, in the order they were given.
///
/// @param member The member
/// @param accounts Where the member can be paid, in the order they were given
static inline void eranosMemberSetAccounts(EranosMember *const member, EranosAccountList const *const accounts) {
    EranosMember_set_accounts(member, accounts);
}

/// The email address. Writes it, if present; absent otherwise.
///
/// @param member The member
/// @param email Receives the email address, replacing what it held
/// @return Whether there is one
static inline bool eranosMemberEmail(EranosMember const *const member, EranosText *const email) {
    eranosDetailClear(email);
    bool const present = EranosMember_email(member, email).is_ok;
    eranosDetailFinish(email);
    return present;
}

/// Sets the email address.
///
/// @param member The member
/// @param email The email address, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosMemberSetEmail(EranosMember *const member, char const *const email) {
    EranosMember_set_email_result const result = EranosMember_set_email(member, eranosDetailView(email));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the email address.
///
/// @param member The member
static inline void eranosMemberClearEmail(EranosMember *const member) { EranosMember_clear_email(member); }

/// The last name. Writes it, if present; absent otherwise.
///
/// @param member The member
/// @param lastName Receives the last name, replacing what it held
/// @return Whether there is one
static inline bool eranosMemberLastName(EranosMember const *const member, EranosText *const lastName) {
    eranosDetailClear(lastName);
    bool const present = EranosMember_last_name(member, lastName).is_ok;
    eranosDetailFinish(lastName);
    return present;
}

/// Sets the last name.
///
/// @param member The member
/// @param lastName The last name, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosMemberSetLastName(EranosMember *const member, char const *const lastName) {
    EranosMember_set_last_name_result const result = EranosMember_set_last_name(member, eranosDetailView(lastName));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the last name.
///
/// @param member The member
static inline void eranosMemberClearLastName(EranosMember *const member) { EranosMember_clear_last_name(member); }

/// The phone number. Writes it, if present; absent otherwise.
///
/// @param member The member
/// @param phone Receives the phone number, replacing what it held
/// @return Whether there is one
static inline bool eranosMemberPhone(EranosMember const *const member, EranosText *const phone) {
    eranosDetailClear(phone);
    bool const present = EranosMember_phone(member, phone).is_ok;
    eranosDetailFinish(phone);
    return present;
}

/// Sets the phone number.
///
/// @param member The member
/// @param phone The phone number, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosMemberSetPhone(EranosMember *const member, char const *const phone) {
    EranosMember_set_phone_result const result = EranosMember_set_phone(member, eranosDetailView(phone));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the phone number.
///
/// @param member The member
static inline void eranosMemberClearPhone(EranosMember *const member) { EranosMember_clear_phone(member); }

/// Destroys @p member; destroying NULL does nothing, as free() does.
///
/// @param member The member, which no call uses after this one, or NULL
static inline void eranosMemberDestroy(EranosMember *const member) {
    if (NULL != member) {
        EranosMember_destroy(member);
    }
}

/// A list of member records.
typedef struct EranosMemberList EranosMemberList;

/// How many records the list holds.
///
/// @param list The member list
/// @return The result
static inline size_t eranosMemberListLength(EranosMemberList const *const list) { return EranosMemberList_len(list); }

/// Whether the list holds no record.
///
/// @param list The member list
/// @return Whether it holds none
static inline bool eranosMemberListIsEmpty(EranosMemberList const *const list) { return EranosMemberList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosMember *eranosMemberListGet(EranosMemberList const *const list, size_t const index) { return EranosMemberList_get(list, index); }

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The member list, which no call uses after this one, or NULL
static inline void eranosMemberListDestroy(EranosMemberList *const list) {
    if (NULL != list) {
        EranosMemberList_destroy(list);
    }
}

#endif // ERANOS_MEMBERS_H
