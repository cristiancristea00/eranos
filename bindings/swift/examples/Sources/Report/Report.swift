// What every example prints its report with: the headings and the tables of the Rust examples.

import Eranos

/// Prints `title` with a double line under it, as an example opens.
public func title(_ title: String) {
    print(title)
    print(String(repeating: "═", count: title.count))
}

/// Prints a section heading with a line under it, after a blank line.
public func section(_ title: String) {
    print("\n\(title)\n\(String(repeating: "─", count: title.count))")
}

/// A row of a report's table, as a Rust example's `#[derive(Tabled)]` struct describes one: the headings of the
/// table's columns and the row's cells, both in the order the table prints its columns.
public protocol TableRow {
    /// The headings of the columns, in the order the table prints them.
    static var headers: [String] { get }

    /// The row's cells, in the order of ``headers``.
    var cells: [String] { get }
}

/// Prints `rows` under their type's headings with rounded borders, as the Rust examples' tables are printed,
/// right-aligning the columns from `numbersFrom` on. A cell of several lines makes its row as tall as its tallest cell.
public func printTable<Row: TableRow>(_ rows: [Row], numbersFrom: Int) {
    let split = { (row: [String]) in row.map { cell in cell.split(separator: "\n", omittingEmptySubsequences: false).map(String.init) } }
    let headings = split(Row.headers)
    let body = rows.map { row in split(row.cells) }
    let widths = Row.headers.indices.map { column in ([headings] + body).map { row in row[column].map(\.count).max() ?? 0 }.max() ?? 0 }
    let border = { (left: String, middle: String, right: String) in
        left + widths.map { width in String(repeating: "─", count: width + 2) }.joined(separator: middle) + right
    }
    let lines = { (row: [[String]]) in
        (0..<(row.map(\.count).max() ?? 1)).map { index in
            "│" + row.indices.map { column in
                let text = index < row[column].count ? row[column][index] : ""
                let padding = String(repeating: " ", count: widths[column] - text.count)
                return " " + (numbersFrom <= column ? padding + text : text + padding) + " "
            }
            .joined(separator: "│") + "│"
        }
    }
    var output = [border("╭", "┬", "╮")] + lines(headings) + [border("├", "┼", "┤")]
    for row in body {
        output += lines(row)
    }
    output.append(border("╰", "┴", "╯"))
    print(output.joined(separator: "\n"))
}

/// The first names of `members` in `group`, joined by commas, or `?` for one the group does not hold.
public func names(_ group: Group, _ members: [Member.ID]) throws(EranosError) -> String {
    try members.map { id throws(EranosError) in try group.member(withID: id)?.firstName ?? "?" }.joined(separator: ", ")
}

/// Every member who shares the cost of `split`, once each, in the order of their identifiers.
public func participants(_ split: ExpenseSplit) -> [Member.ID] {
    let members = switch split {
    case let .itemised(items):
        items.flatMap(\.split.members)
    case let .whole(split):
        split.members
    }
    return Set(members).sorted { first, second in first.rawValue < second.rawValue }
}

/// The totals of `member` in `summary`.
public func totals(of member: Member.ID, in summary: Summary) -> MemberSummary? {
    summary.members.first { totals in totals.member == member }
}
