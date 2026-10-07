/// @file eranos/identifiers.h
/// Identifiers: a new one, the check of one by the rules every kind shares, and lists of them, such as the members of an equal split and the
/// entries an error names.

#ifndef ERANOS_IDENTIFIERS_H
#define ERANOS_IDENTIFIERS_H

#include "EranosIdentifier.h"
#include "EranosTextList.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stddef.h>

/// Writes a new identifier, which any entry or group can take: a UUID v7 in lowercase hyphenated form. Every kind of identifier draws from the one
/// generator of the library, and identifiers generated one after another are in increasing order.
///
/// @param identifier Receives the identifier, replacing what it held
static inline void eranosIdentifierGenerate(EranosText *const identifier) {
    eranosDetailClear(identifier);
    EranosIdentifier_generate(identifier);
    eranosDetailFinish(identifier);
}

/// Writes the identifier written `text`, as it is: not trimmed, and in its case. Every kind of identifier, of a group, a member, a category, an
/// expense, or a transfer, follows the same rules.
///
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param identifier Receives the identifier, replacing what it held
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_ID for text that
///   is not an identifier, with the first rule it breaks, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosIdentifierParse(char const *const text, EranosText *const identifier) {
    eranosDetailClear(identifier);
    EranosIdentifier_parse_result const result = EranosIdentifier_parse(eranosDetailView(text), identifier);
    eranosDetailFinish(identifier);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// A list of text: identifiers, for the most part.
typedef struct EranosTextList EranosTextList;

/// An empty list.
///
/// @return The text list, which the caller destroys
[[nodiscard]] static inline EranosTextList *eranosTextListCreate(void) { return EranosTextList_create(); }

/// Adds `text` at the end.
///
/// @param list The text list
/// @param text The text, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosTextListPush(EranosTextList *const list, char const *const text) {
    EranosTextList_push_result const result = EranosTextList_push(list, eranosDetailView(text));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// How many texts the list holds.
///
/// @param list The text list
/// @return The result
[[nodiscard]] static inline size_t eranosTextListLength(EranosTextList const *const list) { return EranosTextList_len(list); }

/// Whether the list holds no text.
///
/// @param list The text list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosTextListIsEmpty(EranosTextList const *const list) { return EranosTextList_is_empty(list); }

/// Writes the text at `index`, if the list holds one there; absent past the end.
///
/// @param list The text list
/// @param index The index
/// @param text Receives the text, replacing what it held
/// @return Whether there is one
[[nodiscard]] static inline bool eranosTextListGet(EranosTextList const *const list, size_t const index, EranosText *const text) {
    eranosDetailClear(text);
    bool const present = EranosTextList_get(list, index, text).is_ok;
    eranosDetailFinish(text);
    return present;
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The text list, which no call uses after this one, or nullptr
static inline void eranosTextListDestroy(EranosTextList *const list) {
    if (nullptr != list) {
        EranosTextList_destroy(list);
    }
}

#endif // ERANOS_IDENTIFIERS_H
