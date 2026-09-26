/// @file eranos/detail.h
/// The helpers that the other parts' functions call, which are not part of the layer.

#ifndef ERANOS_DETAIL_H
#define ERANOS_DETAIL_H

#include "EranosError.d.h"
#include "EranosFailure.d.h"
#include "EranosIbanReason.d.h"
#include "EranosTimeOfDay.d.h"
#include "diplomat_runtime.h"
#include "eranos/text.h"

#include <stdint.h>
#include <string.h>

/// The view of @p text that the generated functions read: nullptr reads as empty text.
[[nodiscard]] static inline DiplomatStringView eranosDetailView(char const *const text) {
    char const *const bytes = nullptr == text ? "" : text;
    DiplomatStringView const view = {bytes, strlen(bytes)};
    return view;
}

/// Empties @p text before a call writes into it.
static inline void eranosDetailClear(EranosText *const text) { text->len = 0; }

/// Ends @p text with a NUL after a call wrote into it. A buffer of the generated API grows or ends the program, as the library's own
/// allocations do, so its growth never fails.
static inline void eranosDetailFinish(EranosText *const text) {
    (void)text->grow(text, text->len + 1);
    text->buf[text->len] = '\0';
}

/// The error @p error points to when @p isOk says the call failed, and nullptr when it succeeded: the error is read only when present.
[[nodiscard]] static inline EranosError *eranosDetailErrorOf(bool const isOk, EranosError *const *const error) { return isOk ? nullptr : *error; }

/// Gives @p out the value @p value points to when @p isOk says there is one, reading it only then.
[[nodiscard]] static inline bool eranosDetailPresentFailure(bool const isOk, EranosFailure const *const value, EranosFailure *const out) {
    if (isOk) {
        *out = *value;
    }
    return isOk;
}

/// Gives @p out the value @p value points to when @p isOk says there is one, reading it only then.
[[nodiscard]] static inline bool eranosDetailPresentIbanReason(bool const isOk, EranosIbanReason const *const value, EranosIbanReason *const out) {
    if (isOk) {
        *out = *value;
    }
    return isOk;
}

/// Gives @p out the value @p value points to when @p isOk says there is one, reading it only then.
[[nodiscard]] static inline bool eranosDetailPresentTimeOfDay(bool const isOk, EranosTimeOfDay const *const value, EranosTimeOfDay *const out) {
    if (isOk) {
        *out = *value;
    }
    return isOk;
}

/// Gives @p out the value @p value points to when @p isOk says there is one, reading it only then.
[[nodiscard]] static inline bool eranosDetailPresentInt32(bool const isOk, int32_t const *const value, int32_t *const out) {
    if (isOk) {
        *out = *value;
    }
    return isOk;
}

/// Gives @p out the value @p value points to when @p isOk says there is one, reading it only then.
[[nodiscard]] static inline bool eranosDetailPresentInt64(bool const isOk, int64_t const *const value, int64_t *const out) {
    if (isOk) {
        *out = *value;
    }
    return isOk;
}

#endif // ERANOS_DETAIL_H
