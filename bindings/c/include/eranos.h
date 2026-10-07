/// @file eranos.h
/// The eranos library for C23: groups that share expenses, their members, entries, totals, and settlements.
///
/// Every function is a static inline function over the C API that Diplomat generates from the library, whose names, which start with a type's name
/// and an underscore, are not part of this layer. A call the library can refuse returns the error, or nullptr on success, and gives its value through
/// its last parameter; text comes back in an EranosText, ended by a NUL, and text goes in as UTF-8 ended by a NUL, nullptr reading as empty text.
/// Every object a call returns is the caller's to destroy. Every group call has a synchronous form, which blocks the calling thread while another
/// call holds the group's lock, and an asynchronous one, whose name ends in Async and which reports its outcome to a callback on the group's worker
/// thread, in the order of the calls.
///
/// The layer is split into parts by what they hold, under eranos/; this header includes every part.

#ifndef ERANOS_H
#define ERANOS_H

#include "eranos/calendar.h"             // IWYU pragma: export
#include "eranos/category.h"             // IWYU pragma: export
#include "eranos/contact.h"              // IWYU pragma: export
#include "eranos/currency.h"             // IWYU pragma: export
#include "eranos/detail.h"               // IWYU pragma: export
#include "eranos/error.h"                // IWYU pragma: export
#include "eranos/expense.h"              // IWYU pragma: export
#include "eranos/expense_group.h"        // IWYU pragma: export
#include "eranos/expense_group_reader.h" // IWYU pragma: export
#include "eranos/failures.h"             // IWYU pragma: export
#include "eranos/identifiers.h"          // IWYU pragma: export
#include "eranos/members.h"              // IWYU pragma: export
#include "eranos/money.h"                // IWYU pragma: export
#include "eranos/ratios.h"               // IWYU pragma: export
#include "eranos/split.h"                // IWYU pragma: export
#include "eranos/text.h"                 // IWYU pragma: export
#include "eranos/totals.h"               // IWYU pragma: export
#include "eranos/transfer.h"             // IWYU pragma: export

#endif // ERANOS_H
