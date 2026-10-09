// What the ports share: the headings and the tables of the Rust examples, which they print, the members of splits, the
// totals of a member, and the refusals of the library.

#ifndef REPORT_HPP
#define REPORT_HPP

#include "eranos.hpp"

#include <array>
#include <concepts>
#include <cstddef>
#include <expected>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

namespace eranos::examples {

/// Stops the port, printing @p error, which the library refused a call the example makes with.
///
/// @param error The refusal
[[noreturn]] void stopWith(Error const &error) noexcept;

/// The value of @p result, of a call the example makes, which stops the port when the library refused the call.
///
/// @param result The call's result
/// @return Its value
template <typename Value> [[nodiscard]] inline Value must(std::expected<Value, Error> result) noexcept {
    if (!result) {
        stopWith(result.error());
    }
    return std::move(*result);
}

/// Stops the port when the library refused @p result, of a call the example makes that returns nothing.
///
/// @param result The call's result
void must(std::expected<void, Error> const &result) noexcept;

/// Stops the port, naming @p claim, unless @p holds: a number of the report is not the example's.
///
/// @param holds Whether the number is the example's
/// @param claim What the number is
void check(bool const holds, std::string_view const claim) noexcept;

/// Prints @p text, then a line break.
///
/// @param text The text, which may hold line breaks of its own
void printLine(std::string_view const text) noexcept;

/// Prints @p text with a double line under it, as an example opens.
///
/// @param text The title
void title(std::string_view const text) noexcept;

/// Prints a section heading with a line under it, after a blank line.
///
/// @param text The heading
void section(std::string_view const text) noexcept;

/// The text of a table's columns, one per column: the headers of its columns, or the cells of one of its rows.
template <std::size_t Columns> using Cells = std::array<std::string_view, Columns>;

/// A row of a table: its type names the headers of the table's columns in HEADERS, in the order the table prints them,
/// and its cells() gives its cells in that order, as many as there are headers.
template <typename Row>
concept TableRow = requires(Row const &row) {
    { row.cells() } -> std::same_as<std::remove_cvref_t<decltype(Row::HEADERS)>>;
};

/// Prints @p rows under @p headers with rounded borders, as the Rust examples' tables are printed, right-aligning the
/// columns from @p numbersFrom on; a cell of several lines makes its row as tall as its tallest cell.
///
/// @param headers The columns' headers
/// @param rows The rows, a cell per column
/// @param numbersFrom The first column aligned to the right
void printCells(std::span<std::string_view const> const headers, std::vector<std::vector<std::string_view>> const &rows,
                std::size_t const numbersFrom) noexcept;

/// Prints @p rows under the headers of their type with rounded borders, as the Rust examples' tables are printed,
/// right-aligning the columns from @p numbersFrom on.
///
/// @param rows The rows
/// @param numbersFrom The first column aligned to the right
template <TableRow Row> inline void printTable(std::vector<Row> const &rows, std::size_t const numbersFrom) noexcept {
    std::vector<std::vector<std::string_view>> cells;
    cells.reserve(rows.size());
    for (auto const &row : rows) {
        auto const own = row.cells();
        cells.emplace_back(own.begin(), own.end());
    }
    printCells(Row::HEADERS, cells, numbersFrom);
}

/// @p lines, one below the other in a table cell.
///
/// @param lines The lines
/// @return The cell
[[nodiscard]] inline constexpr std::string cell(std::vector<std::string> const &lines) noexcept {
    std::string text;
    for (std::size_t index = 0; lines.size() > index; ++index) {
        text += (0 == index ? "" : "\n") + lines.at(index);
    }
    return text;
}

/// The first name of @p member in @p group, or ? for one the group does not hold.
///
/// @param group The group
/// @param member The member
/// @return The first name
[[nodiscard]] std::string firstName(ExpenseGroup const &group, MemberId const &member) noexcept;

/// The first names of @p members in @p group, joined by commas.
///
/// @param group The group
/// @param members The members
/// @return The names
[[nodiscard]] std::string names(ExpenseGroup const &group, std::vector<MemberId> const &members) noexcept;

/// What the library refused the call of @p result with, or none when it accepted it.
///
/// @param result The call's result
/// @return The refusal
template <typename Value> [[nodiscard]] inline std::optional<Error> refusal(std::expected<Value, Error> const &result) noexcept {
    if (result) {
        return std::nullopt;
    }
    return result.error();
}

/// The message of @p refused, or "accepted".
///
/// @param refused The refusal
/// @return The message
[[nodiscard]] std::string message(std::optional<Error> const &refused) noexcept;

} // namespace eranos::examples

#endif // REPORT_HPP
