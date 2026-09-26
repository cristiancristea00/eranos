// What every port prints its report with: the headings and the tables of the Rust examples.
package eranos.examples

import eranos.ExpenseSplit
import eranos.Group
import eranos.MemberId
import eranos.MemberSummary
import eranos.Summary

/** Prints [title] with a double line under it, as an example opens. */
fun title(title: String) {
    println(title)
    println("═".repeat(width(title)))
}

/** Prints a section heading with a line under it, after a blank line. */
fun section(title: String) {
    println("\n$title\n${"─".repeat(width(title))}")
}

/**
 * Prints [rows] under [headers] with rounded borders, as the Rust examples' tables are printed, right-aligning the
 * columns from [numbersFrom] on. A cell of several lines makes its row as tall as its tallest cell.
 */
fun printTable(
    headers: List<String>,
    rows: List<List<String>>,
    numbersFrom: Int,
) {
    val table = (listOf(headers) + rows).map { row -> row.map { cell -> cell.split("\n") } }
    val widths = headers.indices.map { column -> table.maxOf { row -> row[column].maxOf(::width) } }
    val border = { left: String, middle: String, right: String ->
        widths.joinToString(middle, left, right) { width -> "─".repeat(width + 2) }
    }
    val lines = { row: List<List<String>> ->
        (0 until row.maxOf { cell -> cell.size }).map { index ->
            row.indices.joinToString("│", "│", "│") { column ->
                val text = row[column].getOrElse(index) { "" }
                val padding = " ".repeat(widths[column] - width(text))
                " " + (if (numbersFrom <= column) padding + text else text + padding) + " "
            }
        }
    }
    val output =
        listOf(border("╭", "┬", "╮")) + lines(table.first()) + listOf(border("├", "┼", "┤")) +
            table.drop(1).flatMap(lines) + listOf(border("╰", "┴", "╯"))
    println(output.joinToString("\n"))
}

/** The number of characters of [text], each one column wide in the reports. */
private fun width(text: String): Int = text.codePointCount(0, text.length)

/** The first names of [members] in [group], joined by commas, or `?` for one the group does not hold. */
fun names(
    group: Group,
    members: List<MemberId>,
): String = members.joinToString(", ") { member -> group.memberBlocking(member)?.firstName ?: "?" }

/** Every member who shares the cost of [split], once each, in the order of their identifiers. */
fun participants(split: ExpenseSplit): List<MemberId> {
    val members =
        when (split) {
            is ExpenseSplit.Itemised -> split.items.flatMap { item -> item.split.members }
            is ExpenseSplit.Whole -> split.split.members
        }
    return members.distinct().sortedBy { member -> member.value }
}

/** The totals of [member] in [summary]. */
fun totals(
    member: MemberId,
    summary: Summary,
): MemberSummary? = summary.members.firstOrNull { totals -> totals.member == member }
