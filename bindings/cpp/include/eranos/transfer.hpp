/// @file eranos/transfer.hpp
/// Transfers: money one member paid another.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_TRANSFER_HPP
#define ERANOS_TRANSFER_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/calendar.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/expense.hpp"
#include "eranos/identifiers.hpp"

#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace eranos {

/// A transfer: money one member paid another.
struct Transfer final {
    /// The record of its required fields; the others start absent, empty, or at their default.
    ///
    /// @param cost What was paid.
    /// @param date The day it was paid.
    /// @param from The identifier of the member who paid.
    /// @param to The identifier of the member who was paid.
    Transfer(Cost cost, CalendarDate const date, MemberId from, MemberId to) noexcept
        : cost(std::move(cost)), date(date), from(std::move(from)), to(std::move(to)) {}

    /// The identifier: absent for a transfer to add, present for one the group holds.
    std::optional<TransferId> id;

    /// What was paid.
    Cost cost;

    /// The day it was paid.
    CalendarDate date;

    /// The identifier of the member who paid.
    MemberId from;

    /// The identifier of the member who was paid.
    MemberId to;

    /// A note on the transfer.
    std::optional<std::string> note;

    /// The time of day it was paid.
    std::optional<TimeOfDay> time;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Transfer const &left, Transfer const &right) noexcept {
        return left.id == right.id && left.cost == right.cost && left.date == right.date && left.from == right.from && left.to == right.to &&
               left.note == right.note && left.time == right.time;
    }

    /// Whether a field of two records differs.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(Transfer const &left, Transfer const &right) noexcept { return !(left == right); }
};

namespace detail {

[[nodiscard]] inline Transfer fromDetail(EranosTransfer const &value);
[[nodiscard]] inline std::unique_ptr<EranosTransfer> toDetail(Transfer const &value);
[[nodiscard]] inline std::vector<Transfer> fromDetail(EranosTransferList const &list);

} // namespace detail

} // namespace eranos

#endif // ERANOS_TRANSFER_HPP
