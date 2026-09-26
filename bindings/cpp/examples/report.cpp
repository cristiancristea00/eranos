// What the ports share: the headings and the tables of the Rust examples, which they print, the members of splits, the
// totals of a member, and the refusals of the library.

#include "report.hpp"

#include "eranos.hpp"

#include <algorithm>
#include <cstddef>
#include <cstdlib>
#include <exception>
#include <iostream>
#include <optional>
#include <stdexcept>
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
[[nodiscard]] std::size_t widthOf(std::string_view const text) noexcept {
    return static_cast<std::size_t>(std::count_if(text.begin(), text.end(), [](char const byte) noexcept {
        return CONTINUATION != (static_cast<unsigned>(static_cast<unsigned char>(byte)) & CONTINUATION_MASK);
    }));
}

/// @p piece, @p count times over.
[[nodiscard]] std::string repeated(std::string_view const piece, std::size_t const count) {
    std::string text;
    for (std::size_t index = 0; count > index; ++index) {
        text += piece;
    }
    return text;
}

/// The lines of @p text, an empty text holding one empty line.
[[nodiscard]] std::vector<std::string> linesOf(std::string_view const text) {
    std::vector<std::string> lines;
    std::size_t start = 0;
    for (std::size_t end = text.find('\n'); std::string_view::npos != end; end = text.find('\n', start)) {
        lines.emplace_back(text.substr(start, end - start));
        start = end + 1;
    }
    lines.emplace_back(text.substr(start));
    return lines;
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
[[nodiscard]] std::string borderOf(std::vector<std::size_t> const &widths, Border const border) {
    std::string line(border.left);
    for (std::size_t column = 0; widths.size() > column; ++column) {
        line += repeated("─", widths.at(column) + 2);
        line += widths.size() == column + 1 ? border.right : border.middle;
    }
    return line;
}

/// The lines of @p row, each of its cells' lines on a line of its own, over columns of @p widths.
[[nodiscard]] std::vector<std::string> rowLines(std::vector<std::vector<std::string>> const &row, std::vector<std::size_t> const &widths,
                                                std::size_t const numbersFrom) {
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
            line += " " + (numbersFrom <= column ? padding + text : text + padding) + " │";
        }
        output.push_back(line);
    }
    return output;
}

} // namespace

int runPort(void (*const port)()) {
    try {
        port();
    } catch (std::exception const &error) {
        std::cout.flush();
        std::cerr << "the port stopped: " << error.what() << '\n';
        return EXIT_FAILURE;
    }
    return EXIT_SUCCESS;
}

void check(bool const holds, std::string_view const claim) {
    if (!holds) {
        throw std::logic_error("the report is not the example's: " + std::string(claim));
    }
}

void printLine(std::string_view const text) { std::cout << text << '\n'; }

void title(std::string_view const text) { printLine(std::string(text) + "\n" + repeated("═", widthOf(text))); }

void section(std::string_view const text) { printLine("\n" + std::string(text) + "\n" + repeated("─", widthOf(text))); }

void printTable(std::vector<std::string> const &headers, Rows const &rows, std::size_t const numbersFrom) {
    auto const cellsOf = [](std::vector<std::string> const &row) {
        std::vector<std::vector<std::string>> cells;
        cells.reserve(row.size());
        for (auto const &text : row) {
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
    std::vector<std::string> output{borderOf(widths, {"╭", "┬", "╮"})};
    for (std::size_t row = 0; table.size() > row; ++row) {
        for (auto &line : rowLines(table.at(row), widths, numbersFrom)) {
            output.push_back(std::move(line));
        }
        if (0 == row) {
            output.push_back(borderOf(widths, {"├", "┼", "┤"}));
        }
    }
    output.push_back(borderOf(widths, {"╰", "┴", "╯"}));
    printLine(cell(output));
}

std::string cell(std::vector<std::string> const &lines) {
    std::string text;
    for (std::size_t index = 0; lines.size() > index; ++index) {
        text += (0 == index ? "" : "\n") + lines.at(index);
    }
    return text;
}

std::string firstName(Group const &group, MemberId const &member) {
    auto const found = group.member(member);
    return found.has_value() ? found->firstName : "?";
}

std::string names(Group const &group, std::vector<MemberId> const &members) {
    std::string text;
    for (std::size_t index = 0; members.size() > index; ++index) {
        text += (0 == index ? "" : ", ") + firstName(group, members.at(index));
    }
    return text;
}

std::vector<MemberId> sharedBy(ExpenseSplit const &split) {
    std::vector<MemberId> members = std::visit(
        [](auto const &alternative) {
            using Alternative = std::decay_t<decltype(alternative)>;
            if constexpr (std::is_same_v<Alternative, ItemisedExpenseSplit>) {
                std::vector<MemberId> all;
                for (auto const &item : alternative.items) {
                    auto const own = participants(item.split);
                    all.insert(all.end(), own.begin(), own.end());
                }
                return all;
            } else {
                return participants(alternative.split);
            }
        },
        split);
    std::sort(members.begin(), members.end());
    members.erase(std::unique(members.begin(), members.end()), members.end());
    return members;
}

std::optional<MemberSummary> totalsOf(Summary const &summary, MemberId const &member) {
    auto const found =
        std::find_if(summary.members.begin(), summary.members.end(), [&member](MemberSummary const &totals) { return member == totals.member; });
    return summary.members.end() == found ? std::nullopt : std::optional<MemberSummary>(*found);
}

std::string message(std::optional<Error> const &refused) { return refused.has_value() ? refused->what() : "accepted"; }

} // namespace eranos::examples
