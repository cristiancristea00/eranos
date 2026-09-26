/// @file eranos/split.h
/// How an amount divides among members: equally, by exact amounts, by parts, or by percentages.

#ifndef ERANOS_SPLIT_H
#define ERANOS_SPLIT_H

#include "EranosMemberAmount.h"
#include "EranosMemberAmountList.h"
#include "EranosMemberParts.h"
#include "EranosMemberPartsList.h"
#include "EranosMemberPercentage.h"
#include "EranosMemberPercentageList.h"
#include "EranosSplit.h"
#include "EranosSplitKind.h"
#include "eranos/detail.h"
#include "eranos/identifiers.h"
#include "eranos/money.h"
#include "eranos/text.h"

#include <stddef.h>
#include <stdint.h>

/// The kind of a split. The values of EranosSplitKind, by the names of the C layer.
typedef enum EranosSplitKind EranosSplitKind;

/// Every member pays an exact amount; the amounts add up to what they divide.
static constexpr EranosSplitKind ERANOS_SPLIT_KIND_AMOUNTS = EranosSplitKind_Amounts;
/// Every member pays the same.
static constexpr EranosSplitKind ERANOS_SPLIT_KIND_EQUAL = EranosSplitKind_Equal;
/// Every member pays in proportion to their parts.
static constexpr EranosSplitKind ERANOS_SPLIT_KIND_PARTS = EranosSplitKind_Parts;
/// Every member pays a percentage; the percentages add up to 100 %.
static constexpr EranosSplitKind ERANOS_SPLIT_KIND_PERCENTAGES = EranosSplitKind_Percentages;

/// A member's exact amount in a split.
typedef struct EranosMemberAmount EranosMemberAmount;

/// A new member amount of its required fields; every other field starts absent, empty, or at its default.
///
/// @param amount The amount
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param memberAmount Receives the member amount on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMemberAmountCreate(EranosMoney const *const amount, char const *const member,
                                                                  EranosMemberAmount **const memberAmount) {
    EranosMemberAmount_create_result const result = EranosMemberAmount_create(amount, eranosDetailView(member));
    *memberAmount = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The amount.
///
/// @param memberAmount The member amount
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosMoney *eranosMemberAmountAmount(EranosMemberAmount const *const memberAmount) {
    return EranosMemberAmount_amount(memberAmount);
}

/// Sets the amount.
///
/// @param memberAmount The member amount
/// @param amount The amount
static inline void eranosMemberAmountSetAmount(EranosMemberAmount *const memberAmount, EranosMoney const *const amount) {
    EranosMemberAmount_set_amount(memberAmount, amount);
}

/// The member's identifier.
///
/// @param memberAmount The member amount
/// @param member Receives the member's identifier, replacing what it held
static inline void eranosMemberAmountMember(EranosMemberAmount const *const memberAmount, EranosText *const member) {
    eranosDetailClear(member);
    EranosMemberAmount_member(memberAmount, member);
    eranosDetailFinish(member);
}

/// Sets the member's identifier.
///
/// @param memberAmount The member amount
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMemberAmountSetMember(EranosMemberAmount *const memberAmount, char const *const member) {
    EranosMemberAmount_set_member_result const result = EranosMemberAmount_set_member(memberAmount, eranosDetailView(member));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p memberAmount; destroying nullptr does nothing, as free() does.
///
/// @param memberAmount The member amount, which no call uses after this one, or nullptr
static inline void eranosMemberAmountDestroy(EranosMemberAmount *const memberAmount) {
    if (nullptr != memberAmount) {
        EranosMemberAmount_destroy(memberAmount);
    }
}

/// A list of member amount records.
typedef struct EranosMemberAmountList EranosMemberAmountList;

/// An empty list.
///
/// @return The member amount list, which the caller destroys
[[nodiscard]] static inline EranosMemberAmountList *eranosMemberAmountListCreate(void) { return EranosMemberAmountList_create(); }

/// Adds a copy of `element` at the end.
///
/// @param list The member amount list
/// @param element The element
static inline void eranosMemberAmountListPush(EranosMemberAmountList *const list, EranosMemberAmount const *const element) {
    EranosMemberAmountList_push(list, element);
}

/// How many records the list holds.
///
/// @param list The member amount list
/// @return The result
[[nodiscard]] static inline size_t eranosMemberAmountListLength(EranosMemberAmountList const *const list) { return EranosMemberAmountList_len(list); }

/// Whether the list holds no record.
///
/// @param list The member amount list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosMemberAmountListIsEmpty(EranosMemberAmountList const *const list) {
    return EranosMemberAmountList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member amount list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosMemberAmount *eranosMemberAmountListGet(EranosMemberAmountList const *const list, size_t const index) {
    return EranosMemberAmountList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The member amount list, which no call uses after this one, or nullptr
static inline void eranosMemberAmountListDestroy(EranosMemberAmountList *const list) {
    if (nullptr != list) {
        EranosMemberAmountList_destroy(list);
    }
}

/// A member's parts in a split, in hundredths of a part.
typedef struct EranosMemberParts EranosMemberParts;

/// A new member parts of its required fields; every other field starts absent, empty, or at its default.
///
/// @param hundredths The parts, in hundredths: 100 for one part
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param memberParts Receives the member parts on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMemberPartsCreate(int32_t const hundredths, char const *const member,
                                                                 EranosMemberParts **const memberParts) {
    EranosMemberParts_create_result const result = EranosMemberParts_create(hundredths, eranosDetailView(member));
    *memberParts = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The parts, in hundredths: 100 for one part.
///
/// @param memberParts The member parts
/// @return The parts, in hundredths: 100 for one part
[[nodiscard]] static inline int32_t eranosMemberPartsHundredths(EranosMemberParts const *const memberParts) {
    return EranosMemberParts_hundredths(memberParts);
}

/// Sets the parts, in hundredths: 100 for one part.
///
/// @param memberParts The member parts
/// @param hundredths The parts, in hundredths: 100 for one part
static inline void eranosMemberPartsSetHundredths(EranosMemberParts *const memberParts, int32_t const hundredths) {
    EranosMemberParts_set_hundredths(memberParts, hundredths);
}

/// The member's identifier.
///
/// @param memberParts The member parts
/// @param member Receives the member's identifier, replacing what it held
static inline void eranosMemberPartsMember(EranosMemberParts const *const memberParts, EranosText *const member) {
    eranosDetailClear(member);
    EranosMemberParts_member(memberParts, member);
    eranosDetailFinish(member);
}

/// Sets the member's identifier.
///
/// @param memberParts The member parts
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMemberPartsSetMember(EranosMemberParts *const memberParts, char const *const member) {
    EranosMemberParts_set_member_result const result = EranosMemberParts_set_member(memberParts, eranosDetailView(member));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p memberParts; destroying nullptr does nothing, as free() does.
///
/// @param memberParts The member parts, which no call uses after this one, or nullptr
static inline void eranosMemberPartsDestroy(EranosMemberParts *const memberParts) {
    if (nullptr != memberParts) {
        EranosMemberParts_destroy(memberParts);
    }
}

/// A list of member parts records.
typedef struct EranosMemberPartsList EranosMemberPartsList;

/// An empty list.
///
/// @return The member parts list, which the caller destroys
[[nodiscard]] static inline EranosMemberPartsList *eranosMemberPartsListCreate(void) { return EranosMemberPartsList_create(); }

/// Adds a copy of `element` at the end.
///
/// @param list The member parts list
/// @param element The element
static inline void eranosMemberPartsListPush(EranosMemberPartsList *const list, EranosMemberParts const *const element) {
    EranosMemberPartsList_push(list, element);
}

/// How many records the list holds.
///
/// @param list The member parts list
/// @return The result
[[nodiscard]] static inline size_t eranosMemberPartsListLength(EranosMemberPartsList const *const list) { return EranosMemberPartsList_len(list); }

/// Whether the list holds no record.
///
/// @param list The member parts list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosMemberPartsListIsEmpty(EranosMemberPartsList const *const list) {
    return EranosMemberPartsList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member parts list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosMemberParts *eranosMemberPartsListGet(EranosMemberPartsList const *const list, size_t const index) {
    return EranosMemberPartsList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The member parts list, which no call uses after this one, or nullptr
static inline void eranosMemberPartsListDestroy(EranosMemberPartsList *const list) {
    if (nullptr != list) {
        EranosMemberPartsList_destroy(list);
    }
}

/// A member's percentage in a split, in basis points.
typedef struct EranosMemberPercentage EranosMemberPercentage;

/// A new member percentage of its required fields; every other field starts absent, empty, or at its default.
///
/// @param basisPoints The percentage, in basis points: 10000 for 100 %
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @param memberPercentage Receives the member percentage on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMemberPercentageCreate(int32_t const basisPoints, char const *const member,
                                                                      EranosMemberPercentage **const memberPercentage) {
    EranosMemberPercentage_create_result const result = EranosMemberPercentage_create(basisPoints, eranosDetailView(member));
    *memberPercentage = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The percentage, in basis points: 10000 for 100 %.
///
/// @param memberPercentage The member percentage
/// @return The percentage, in basis points: 10000 for 100 %
[[nodiscard]] static inline int32_t eranosMemberPercentageBasisPoints(EranosMemberPercentage const *const memberPercentage) {
    return EranosMemberPercentage_basis_points(memberPercentage);
}

/// Sets the percentage, in basis points: 10000 for 100 %.
///
/// @param memberPercentage The member percentage
/// @param basisPoints The percentage, in basis points: 10000 for 100 %
static inline void eranosMemberPercentageSetBasisPoints(EranosMemberPercentage *const memberPercentage, int32_t const basisPoints) {
    EranosMemberPercentage_set_basis_points(memberPercentage, basisPoints);
}

/// The member's identifier.
///
/// @param memberPercentage The member percentage
/// @param member Receives the member's identifier, replacing what it held
static inline void eranosMemberPercentageMember(EranosMemberPercentage const *const memberPercentage, EranosText *const member) {
    eranosDetailClear(member);
    EranosMemberPercentage_member(memberPercentage, member);
    eranosDetailFinish(member);
}

/// Sets the member's identifier.
///
/// @param memberPercentage The member percentage
/// @param member The member's identifier, as UTF-8 ended by a NUL; nullptr reads as empty text
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_INVALID_TEXT for
///   text that is not UTF-8
[[nodiscard]] static inline EranosError *eranosMemberPercentageSetMember(EranosMemberPercentage *const memberPercentage, char const *const member) {
    EranosMemberPercentage_set_member_result const result = EranosMemberPercentage_set_member(memberPercentage, eranosDetailView(member));
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p memberPercentage; destroying nullptr does nothing, as free() does.
///
/// @param memberPercentage The member percentage, which no call uses after this one, or nullptr
static inline void eranosMemberPercentageDestroy(EranosMemberPercentage *const memberPercentage) {
    if (nullptr != memberPercentage) {
        EranosMemberPercentage_destroy(memberPercentage);
    }
}

/// A list of member percentage records.
typedef struct EranosMemberPercentageList EranosMemberPercentageList;

/// An empty list.
///
/// @return The member percentage list, which the caller destroys
[[nodiscard]] static inline EranosMemberPercentageList *eranosMemberPercentageListCreate(void) { return EranosMemberPercentageList_create(); }

/// Adds a copy of `element` at the end.
///
/// @param list The member percentage list
/// @param element The element
static inline void eranosMemberPercentageListPush(EranosMemberPercentageList *const list, EranosMemberPercentage const *const element) {
    EranosMemberPercentageList_push(list, element);
}

/// How many records the list holds.
///
/// @param list The member percentage list
/// @return The result
[[nodiscard]] static inline size_t eranosMemberPercentageListLength(EranosMemberPercentageList const *const list) {
    return EranosMemberPercentageList_len(list);
}

/// Whether the list holds no record.
///
/// @param list The member percentage list
/// @return Whether it holds none
[[nodiscard]] static inline bool eranosMemberPercentageListIsEmpty(EranosMemberPercentageList const *const list) {
    return EranosMemberPercentageList_is_empty(list);
}

/// A copy of the record at `index`, absent past the end.
///
/// @param list The member percentage list
/// @param index The index
/// @return A copy of the element at @p index, which the caller destroys, or nullptr past the end
[[nodiscard]] static inline EranosMemberPercentage *eranosMemberPercentageListGet(EranosMemberPercentageList const *const list, size_t const index) {
    return EranosMemberPercentageList_get(list, index);
}

/// Destroys @p list; destroying nullptr does nothing, as free() does.
///
/// @param list The member percentage list, which no call uses after this one, or nullptr
static inline void eranosMemberPercentageListDestroy(EranosMemberPercentageList *const list) {
    if (nullptr != list) {
        EranosMemberPercentageList_destroy(list);
    }
}

/// How an amount is divided among members.
typedef struct EranosSplit EranosSplit;

/// Every member pays an exact amount; the amounts add up to what they divide.
///
/// @param shares Each member's amount
/// @return The split, which the caller destroys
[[nodiscard]] static inline EranosSplit *eranosSplitCreateAmounts(EranosMemberAmountList const *const shares) {
    return EranosSplit_create_amounts(shares);
}

/// Every member pays the same.
///
/// @param members The identifiers of the members who share the amount
/// @return The split, which the caller destroys
[[nodiscard]] static inline EranosSplit *eranosSplitCreateEqual(EranosTextList const *const members) { return EranosSplit_create_equal(members); }

/// Every member pays in proportion to their parts.
///
/// @param shares Each member's amount
/// @return The split, which the caller destroys
[[nodiscard]] static inline EranosSplit *eranosSplitCreateParts(EranosMemberPartsList const *const shares) {
    return EranosSplit_create_parts(shares);
}

/// Every member pays a percentage; the percentages add up to 100 %.
///
/// @param shares Each member's amount
/// @return The split, which the caller destroys
[[nodiscard]] static inline EranosSplit *eranosSplitCreatePercentages(EranosMemberPercentageList const *const shares) {
    return EranosSplit_create_percentages(shares);
}

/// Which kind of split it is.
///
/// @param split The split
/// @return The result
[[nodiscard]] static inline EranosSplitKind eranosSplitKind(EranosSplit const *const split) { return EranosSplit_kind(split); }

/// Each member's amount. Present in the kind ERANOS_ERROR_KIND_AMOUNTS alone.
///
/// @param split The split
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosMemberAmountList *eranosSplitAmountsShares(EranosSplit const *const split) {
    return EranosSplit_amounts_shares(split);
}

/// The identifiers of the members who share the amount. Present in the kind ERANOS_ERROR_KIND_EQUAL alone.
///
/// @param split The split
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosTextList *eranosSplitMembers(EranosSplit const *const split) { return EranosSplit_members(split); }

/// Each member's parts. Present in the kind ERANOS_ERROR_KIND_PARTS alone.
///
/// @param split The split
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosMemberPartsList *eranosSplitPartsShares(EranosSplit const *const split) { return EranosSplit_parts_shares(split); }

/// Each member's percentage. Present in the kind ERANOS_ERROR_KIND_PERCENTAGES alone.
///
/// @param split The split
/// @return A copy of the value, which the caller destroys, or nullptr when there is none
[[nodiscard]] static inline EranosMemberPercentageList *eranosSplitPercentagesShares(EranosSplit const *const split) {
    return EranosSplit_percentages_shares(split);
}

/// The identifiers of the members the split divides among, in the order it lists them.
///
/// @param split The split
/// @param participants Receives the participants on success, which the caller destroys, and nullptr on failure
/// @return nullptr on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_ID for an
/// identifier
///   that is not one
[[nodiscard]] static inline EranosError *eranosSplitParticipants(EranosSplit const *const split, EranosTextList **const participants) {
    EranosSplit_participants_result const result = EranosSplit_participants(split);
    *participants = result.is_ok ? result.ok : nullptr;
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Destroys @p split; destroying nullptr does nothing, as free() does.
///
/// @param split The split, which no call uses after this one, or nullptr
static inline void eranosSplitDestroy(EranosSplit *const split) {
    if (nullptr != split) {
        EranosSplit_destroy(split);
    }
}

#endif // ERANOS_SPLIT_H
