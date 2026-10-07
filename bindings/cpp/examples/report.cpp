// What the ports share: the headings and the tables of the Rust examples, which they print, the members of splits, the
// totals of a member, and the refusals of the library.

#include "report.hpp"

#include "eranos.hpp"

#include <algorithm>
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <expected>
#include <format>
#include <optional>
#include <print>
#include <span>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// The bits of a byte that tell whether it continues a character in UTF-8.
constexpr unsigned CONTINUATION_MASK = 0xC0U;

/// Those bits in a byte that continues a character.
constexpr unsigned CONTINUATION = 0x80U;

/// How many characters @p text holds: the reports' characters are each one column wide.
[[nodiscard]] inline constexpr std::size_t widthOf(std::string_view const text) noexcept {
    return static_cast<std::size_t>(std::count_if(text.begin(), text.end(), [](char const byte) noexcept {
        return CONTINUATION != (static_cast<unsigned>(static_cast<unsigned char>(byte)) & CONTINUATION_MASK);
    }));
}

/// @p piece, @p count times over.
[[nodiscard]] inline constexpr std::string repeated(std::string_view const piece, std::size_t const count) noexcept {
    std::string text;
    for (std::size_t index = 0; count > index; ++index) {
        text += piece;
    }
    return text;
}

/// The lines of @p text, an empty text holding one empty line.
[[nodiscard]] inline constexpr std::vector<std::string> linesOf(std::string_view const text) noexcept {
    std::vector<std::string> lines;
    std::size_t start = 0;
    for (std::size_t end = text.find('\n'); std::string_view::npos != end; end = text.find('\n', start)) {
        lines.emplace_back(text.substr(start, end - start));
        start = end + 1;
    }
    lines.emplace_back(text.substr(start));
    return lines;
}

/// Ends the port with a failure, printing @p reason after the report it printed so far.
[[noreturn]] void stop(std::string_view const reason) noexcept {
    static_cast<void>(std::fflush(stdout));
    std::println(stderr, "the port stopped: {}", reason);
    // NOLINTNEXTLINE(concurrency-mt-unsafe): a port stops on its main thread, and no thread of its own calls exit().
    std::exit(EXIT_FAILURE);
}

/// The corners and the joints of a table's border.
struct Border final {
    /// Where it starts.
    std::string_view left;
    /// Where it crosses a line between two columns.
    std::string_view middle;
    /// Where it ends.
    std::string_view right;
};

/// The border @p border over columns of @p widths.
[[nodiscard]] inline constexpr std::string borderOf(std::vector<std::size_t> const &widths, Border const border) noexcept {
    std::string line(border.left);
    for (std::size_t column = 0; widths.size() > column; ++column) {
        line += repeated("─", widths.at(column) + 2);
        line += widths.size() == column + 1 ? border.right : border.middle;
    }
    return line;
}

/// The lines of @p row, each of its cells' lines on a line of its own, over columns of @p widths.
[[nodiscard]] std::vector<std::string> rowLines(std::vector<std::vector<std::string>> const &row, std::vector<std::size_t> const &widths,
                                                std::size_t const numbersFrom) noexcept {
    std::size_t height = 1;
    for (auto const &lines : row) {
        height = std::max(height, lines.size());
    }
    std::vector<std::string> output;
    for (std::size_t index = 0; height > index; ++index) {
        std::string line = "│";
        for (std::size_t column = 0; row.size() > column; ++column) {
            std::string const text = row.at(column).size() > index ? row.at(column).at(index) : "";
            std::string const padding(widths.at(column) - widthOf(text), ' ');
            line += std::format(" {} │", (numbersFrom <= column ? padding + text : text + padding));
        }
        output.push_back(line);
    }
    return output;
}

} // namespace

void stopWith(Error const &error) noexcept { stop(std::format("the library refused a call the example makes: {}", error.message())); }

void must(std::expected<void, Error> const &result) noexcept {
    if (!result) {
        stopWith(result.error());
    }
}

void check(bool const holds, std::string_view const claim) noexcept {
    if (!holds) {
        stop(std::format("the report is not the example's: {}", claim));
    }
}

void printLine(std::string_view const text) noexcept { std::println("{}", text); }

void title(std::string_view const text) noexcept { std::println("{}\n{}", text, repeated("═", widthOf(text))); }

void section(std::string_view const text) noexcept { std::println("\n{}\n{}", text, repeated("─", widthOf(text))); }

void printCells(std::span<std::string_view const> const headers, std::vector<std::vector<std::string_view>> const &rows,
                std::size_t const numbersFrom) noexcept {
    auto const cellsOf = [](std::span<std::string_view const> const row) noexcept {
        std::vector<std::vector<std::string>> cells;
        cells.reserve(row.size());
        for (auto const text : row) {
            cells.push_back(linesOf(text));
        }
        return cells;
    };
    std::vector<std::vector<std::vector<std::string>>> table{cellsOf(headers)};
    table.reserve(rows.size() + 1);
    for (auto const &row : rows) {
        table.push_back(cellsOf(row));
    }
    std::vector<std::size_t> widths(headers.size(), 0);
    for (auto const &row : table) {
        for (std::size_t column = 0; row.size() > column; ++column) {
            for (auto const &line : row.at(column)) {
                widths.at(column) = std::max(widths.at(column), widthOf(line));
            }
        }
    }
    std::vector<std::string> output{borderOf(widths, {.left = "╭", .middle = "┬", .right = "╮"})};
    for (std::size_t row = 0; table.size() > row; ++row) {
        for (auto &line : rowLines(table.at(row), widths, numbersFrom)) {
            output.push_back(std::move(line));
        }
        if (0 == row) {
            output.push_back(borderOf(widths, {.left = "├", .middle = "┼", .right = "┤"}));
        }
    }
    output.push_back(borderOf(widths, {.left = "╰", .middle = "┴", .right = "╯"}));
    printLine(cell(output));
}

std::string firstName(ExpenseGroup const &group, MemberId const &member) noexcept {
    auto const found = must(group.member(member));
    return found.has_value() ? found->firstName : "?";
}

std::string names(ExpenseGroup const &group, std::vector<MemberId> const &members) noexcept {
    std::string text;
    for (std::size_t index = 0; members.size() > index; ++index) {
        text += (0 == index ? "" : ", ") + firstName(group, members.at(index));
    }
    return text;
}

std::vector<MemberId> sharedBy(ExpenseSplit const &split) noexcept {
    std::vector<MemberId> members = std::visit(
        [](auto const &alternative) noexcept {
            using Alternative = std::decay_t<decltype(alternative)>;
            if constexpr (std::is_same_v<Alternative, ItemisedExpenseSplit>) {
                std::vector<MemberId> all;
                for (auto const &item : alternative.items) {
                    auto const own = must(participants(item.split));
                    all.insert(all.end(), own.begin(), own.end());
                }
                return all;
            } else {
                return must(participants(alternative.split));
            }
        },
        split);
    std::ranges::sort(members);
    members.erase(std::ranges::unique(members).begin(), members.end());
    return members;
}

std::string message(std::optional<Error> const &refused) noexcept { return refused.has_value() ? refused->message() : "accepted"; }

} // namespace eranos::examples
