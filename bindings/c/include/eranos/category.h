/// @file eranos/category.h
/// The categories of a group's expenses.

#ifndef ERANOS_CATEGORY_H
#define ERANOS_CATEGORY_H

#include "EranosCategory.h"
#include "EranosCategoryList.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stdbool.h>
#include <stddef.h>

/// A category of expenses.
typedef struct EranosCategory EranosCategory;

/// A new category of its required fields; every other field starts absent, empty, or at its default.
///
/// @param name The name, which no other category of the group shares, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param category Receives the category on success, which the caller destroys, and NULL on failure
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosCategoryCreate(char const *const name, EranosCategory **const category) {
    EranosCategory_create_result const result = EranosCategory_create(eranosDetailView(name));
    *category = result.is_ok ? result.ok : NULL;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The identifier: absent for a category to add, present for one the group holds. Writes it, if present; absent otherwise.
///
/// @param category The category
/// @param id Receives the identifier: absent for a category to add, present for one the group holds, replacing what it held
/// @return Whether there is one
static inline bool eranosCategoryId(EranosCategory const *const category, EranosText *const id) {
    eranosDetailClear(id);
    bool const present = EranosCategory_id(category, id).is_ok;
    eranosDetailFinish(id);
    return present;
}

/// Sets the identifier: absent for a category to add, present for one the group holds.
///
/// @param category The category
/// @param id The identifier: absent for a category to add, present for one the group holds, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosCategorySetId(EranosCategory *const category, char const *const id) {
    EranosCategory_set_id_result const result = EranosCategory_set_id(category, eranosDetailView(id));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Clears the identifier: absent for a category to add, present for one the group holds.
///
/// @param category The category
static inline void eranosCategoryClearId(EranosCategory *const category) { EranosCategory_clear_id(category); }

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
/// @param name The name, which no other category of the group shares, as UTF-8 ended by a NUL; NULL reads as empty text
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
static inline EranosError *eranosCategorySetName(EranosCategory *const category, char const *const name) {
    EranosCategory_set_name_result const result = EranosCategory_set_name(category, eranosDetailView(name));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p category; destroying NULL does nothing, as free() does.
///
/// @param category The category, which no call uses after this one, or NULL
static inline void eranosCategoryDestroy(EranosCategory *const category) {
    if (NULL != category) {
        EranosCategory_destroy(category);
    }
}

/// A list of category records.
typedef struct EranosCategoryList EranosCategoryList;

/// How many records the list holds.
///
/// @param list The category list
/// @return The result
static inline size_t eranosCategoryListLength(EranosCategoryList const *const list) { return EranosCategoryList_len(list); }

/// Whether the list holds no record.
///
/// @param list The category list
/// @return Whether it holds none
static inline bool eranosCategoryListIsEmpty(EranosCategoryList const *const list) { return EranosCategoryList_is_empty(list); }

/// A copy of the record at `index`, absent past the end.
///
/// @param list The category list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or NULL past the end
static inline EranosCategory *eranosCategoryListGet(EranosCategoryList const *const list, size_t const index) {
    return EranosCategoryList_get(list, index);
}

/// Destroys @p list; destroying NULL does nothing, as free() does.
///
/// @param list The category list, which no call uses after this one, or NULL
static inline void eranosCategoryListDestroy(EranosCategoryList *const list) {
    if (NULL != list) {
        EranosCategoryList_destroy(list);
    }
}

#endif // ERANOS_CATEGORY_H
