/// @file eranos/category.h
/// The categories of a group's expenses.

#ifndef ERANOS_CATEGORY_H
#define ERANOS_CATEGORY_H

#include "EranosCategory.h"
#include "EranosCategoryList.h"
#include "eranos/detail.h"
#include "eranos/identifiers.h"
#include "eranos/text.h"

#include <stddef.h>

/// A category of expenses.
typedef struct EranosCategory EranosCategory;

/// A new category of its required fields under the identifier @p id, such as the key an app's own store gives it; every other field starts absent,
/// empty, or at its default. eranosCategoryCreate() generates the identifier instead.
///
/// @param id The identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes, as UTF-8 ended by a NUL; nullptr reads as empty
///   text
/// @param name The name, which no other category of the group shares, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param category Receives the category on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCategoryCreateWithId(char const *const id, char const *const name, EranosCategory **const category) {
    EranosCategory_create_result const result = EranosCategory_create(eranosDetailView(id), eranosDetailView(name));
    *category = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// A new category of its required fields under an identifier generated when it is called, which eranosCategoryCreateWithId() takes instead; every
/// other field starts absent, empty, or at its default.
///
/// @param name The name, which no other category of the group shares, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param category Receives the category on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCategoryCreate(char const *const name, EranosCategory **const category) {
    EranosText *const id = eranosTextCreate();
    eranosIdentifierGenerate(id);
    EranosError *const error = eranosCategoryCreateWithId(eranosTextData(id), name, category);
    eranosTextDestroy(id);
    return error;
}

/// The identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes.
///
/// @param category The category
/// @param id Receives the identifier, replacing what it held
static inline void eranosCategoryId(EranosCategory const *const category, EranosText *const id) {
    eranosDetailClear(id);
    EranosCategory_id(category, id);
    eranosDetailFinish(id);
}

/// Sets the identifier, which the app gives: its own, or one eranosIdentifierGenerate() writes.
///
/// @param category The category
/// @param id The identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCategorySetId(EranosCategory *const category, char const *const id) {
    EranosCategory_set_id_result const result = EranosCategory_set_id(category, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The name, which no other category of the group shares.
///
/// @param category The category
/// @param name Receives the name, which no other category of the group shares, replacing what it held
static inline void eranosCategoryName(EranosCategory const *const category, EranosText *const name) {
    eranosDetailClear(name);
    EranosCategory_name(category, name);
    eranosDetailFinish(name);
}

/// Sets the name, which no other category of the group shares.
///
/// @param category The category
/// @param name The name, which no other category of the group shares, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosCategorySetName(EranosCategory *const category, char const *const name) {
    EranosCategory_set_name_result const result = EranosCategory_set_name(category, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p category; destroying nullptr does nothing, as free() does.
///
/// @param category The category, which no call uses after this one, or nullptr
static inline void eranosCategoryDestroy(EranosCategory *const category) {
    if (nullptr != category) {
        EranosCategory_destroy(category);
    }
}

/// A list of category records.
typedef struct EranosCategoryList EranosCategoryList;

/// How many records the list holds.
///
/// @param list The category list
/// @return The result
[[nodiscard]] static inline size_t eranosCategoryListLength(EranosCategoryList const *const list) { return EranosCategoryList_len(list); }

/// Whether the list holds no record.
///
/// @param list The category list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosCategoryListIsEmpty(EranosCategoryList const *const list) { return EranosCategoryList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The category list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosCategory *eranosCategoryListGet(EranosCategoryList const *const list, size_t const index) {
    return EranosCategoryList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The category list, which no call uses after this one, or nullptr
static inline void eranosCategoryListDestroy(EranosCategoryList *const list) {
    if (nullptr != list) {
        EranosCategoryList_destroy(list);
    }
}

#endif // ERANOS_CATEGORY_H
