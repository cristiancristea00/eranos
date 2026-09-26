// What the ports share: the headings and the tables of the Rust examples, which they print, the members of splits, the
// totals of a member, and the refusals of the library.

#ifndef REPORT_HPP
#define REPORT_HPP

#include "eranos.hpp"

#include <cstddef>
#include <expected>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos::examples {

/// Stops the port, printing @p error, which the library refused a call the example makes with.
///
/// @param error The refusal
[[noreturn]] void stopWith(Error const &error);

/// The value of @p result, of a call the example makes, which stops the port when the library refused the call.
///
/// @param result The call's result
/// @return Its value
template <typename Value> [[nodiscard]] Value must(std::expected<Value, Error> result) {
    if (!result) {
        stopWith(result.error());
    }
    return std::move(*result);
}

/// Stops the port when the library refused @p result, of a call the example makes that returns nothing.
///
/// @param result The call's result
void must(std::expected<void, Error> const &result);

/// Stops the port, naming @p claim, unless @p holds: a number of the report is not the example's.
///
/// @param holds Whether the number is the example's
/// @param claim What the number is
void check(bool const holds, std::string_view const claim);

/// Prints @p text, then a line break.
///
/// @param text The text, which may hold line breaks of its own
void printLine(std::string_view const text);

/// Prints @p text with a double line under it, as an example opens.
///
/// @param text The title
void title(std::string_view const text);

/// Prints a section heading with a line under it, after a blank line.
///
/// @param text The heading
void section(std::string_view const text);

/// The rows of a table, a cell per column; a cell of several lines makes its row as tall as its tallest cell.
using Rows = std::vector<std::vector<std::string>>;

/// Prints @p rows under @p headers with rounded borders, as the Rust examples' tables are printed, right-aligning the
/// columns from @p numbersFrom on.
///
/// @param headers The columns' headers
/// @param rows The rows
/// @param numbersFrom The first column aligned to the right
void printTable(std::vector<std::string> const &headers, Rows const &rows, std::size_t const numbersFrom);

/// @p lines, one below the other in a table cell.
///
/// @param lines The lines
/// @return The cell
[[nodiscard]] std::string cell(std::vector<std::string> const &lines);

/// The first name of @p member in @p group, or ? for one the group does not hold.
///
/// @param group The group
/// @param member The member
/// @return The first name
[[nodiscard]] std::string firstName(Group const &group, MemberId const &member);

/// The first names of @p members in @p group, joined by commas.
///
/// @param group The group
/// @param members The members
/// @return The names
[[nodiscard]] std::string names(Group const &group, std::vector<MemberId> const &members);

/// Every member who shares the cost of @p split, once each, in the order of their identifiers.
///
/// @param split The expense's split
/// @return The members
[[nodiscard]] std::vector<MemberId> sharedBy(ExpenseSplit const &split);

/// The totals of @p member in @p summary, absent when the summary lists none of theirs.
///
/// @param summary The summary
/// @param member The member
/// @return The totals
[[nodiscard]] std::optional<MemberSummary> totalsOf(Summary const &summary, MemberId const &member);

/// What the library refused the call of @p result with, or none when it accepted it.
///
/// @param result The call's result
/// @return The refusal
template <typename Value> [[nodiscard]] std::optional<Error> refusal(std::expected<Value, Error> const &result) {
    if (result) {
        return std::nullopt;
    }
    return result.error();
}

/// The message of @p refused, or "accepted".
///
/// @param refused The refusal
/// @return The message
[[nodiscard]] std::string message(std::optional<Error> const &refused);

} // namespace eranos::examples

#endif // REPORT_HPP
