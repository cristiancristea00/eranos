/// @file eranos/text.h
/// Text the library writes and reads: the buffer every call that returns text writes into.

#ifndef ERANOS_TEXT_H
#define ERANOS_TEXT_H

#include "diplomat_runtime.h"

#include <stddef.h>

/// Text the library writes: UTF-8 ended by a NUL, which a call that writes into it replaces. It is a buffer of the generated
/// C API; read it through eranosTextData() and eranosTextLength() only.
typedef DiplomatWrite EranosText;

/// An empty text, which the calls that return text write into.
///
/// @return The text, which the caller destroys with eranosTextDestroy()
static inline EranosText *eranosTextCreate(void) {
    EranosText *const text = diplomat_buffer_write_create(1);
    text->buf[0] = '\0';
    return text;
}

/// The bytes of @p text, ended by a NUL, valid until a call writes into it again or it is destroyed.
///
/// @param text The text
/// @return The bytes
static inline char const *eranosTextData(EranosText const *const text) { return text->buf; }

/// How many bytes @p text holds, the NUL left out.
///
/// @param text The text
/// @return The number of bytes
static inline size_t eranosTextLength(EranosText const *const text) { return text->len; }

/// Destroys @p text; destroying NULL does nothing, as free() does.
///
/// @param text The text, which no call uses after this one, or NULL
static inline void eranosTextDestroy(EranosText *const text) {
    if (NULL != text) {
        diplomat_buffer_write_destroy(text);
    }
}

#endif // ERANOS_TEXT_H
