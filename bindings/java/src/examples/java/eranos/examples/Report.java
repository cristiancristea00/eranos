package eranos.examples;

import eranos.EranosException;
import eranos.ExpenseGroup;
import eranos.Member;
import eranos.MemberId;

import java.util.ArrayList;
import java.util.List;

/** What every port prints its report with: the headings and the tables of the Rust examples. */
public final class Report {
    private Report() {}

    /**
     * Prints {@code title} with a double line under it, as an example opens.
     *
     * @param title the title
     */
    public static void title(String title) {
        System.out.println(title);
        System.out.println("═".repeat(width(title)));
    }

    /**
     * Prints a section heading with a line under it, after a blank line.
     *
     * @param title the heading
     */
    public static void section(String title) {
        System.out.println();
        System.out.println(title);
        System.out.println("─".repeat(width(title)));
    }

    /**
     * A row of a report's table, as the Rust examples' row structs are: a record whose components
     * are in the order of its columns, with a constant {@code HEADERS} that lists its columns'
     * headers in the order the report prints them.
     */
    public interface Row {
        /**
         * The text of every cell, in the order of the row's {@code HEADERS}.
         *
         * @return the cells
         */
        List<String> cells();
    }

    /**
     * Prints {@code rows} under {@code headers} with rounded borders, as the Rust examples' tables
     * are printed, right-aligning the columns from {@code numbersFrom} on. A cell of several lines
     * makes its row as tall as its tallest cell.
     *
     * @param headers the columns' headers, the {@code HEADERS} of the rows' type
     * @param rows the rows
     * @param numbersFrom the first column aligned to the right
     */
    public static void printTable(List<String> headers, List<? extends Row> rows, int numbersFrom) {
        List<List<String>> heading = lines(headers);
        List<List<List<String>>> body = rows.stream().map(row -> lines(row.cells())).toList();
        int[] widths = new int[headers.size()];
        widen(widths, heading);
        for (List<List<String>> row : body) {
            widen(widths, row);
        }
        StringBuilder output = new StringBuilder();
        output.append(border(widths, "╭", "┬", "╮"));
        append(output, heading, widths, numbersFrom);
        output.append(border(widths, "├", "┼", "┤"));
        for (List<List<String>> row : body) {
            append(output, row, widths, numbersFrom);
        }
        output.append(border(widths, "╰", "┴", "╯"));
        System.out.print(output);
    }

    /** The lines of every cell of {@code cells}. */
    private static List<List<String>> lines(List<String> cells) {
        return cells.stream().map(cell -> List.of(cell.split("\n", -1))).toList();
    }

    /** Widens each of {@code widths} to the widest line of its column in {@code row}. */
    private static void widen(int[] widths, List<List<String>> row) {
        for (int column = 0; column < widths.length; column++) {
            for (String line : row.get(column)) {
                widths[column] = Math.max(widths[column], width(line));
            }
        }
    }

    /** A border line of the table whose columns are {@code widths} wide. */
    private static String border(int[] widths, String left, String middle, String right) {
        List<String> lines = new ArrayList<>();
        for (int width : widths) {
            lines.add("─".repeat(width + 2));
        }
        return left + String.join(middle, lines) + right + "\n";
    }

    /** Appends the lines of {@code row} to {@code output}. */
    private static void append(
            StringBuilder output, List<List<String>> row, int[] widths, int numbersFrom) {
        int height = row.stream().mapToInt(List::size).max().orElse(1);
        for (int index = 0; index < height; index++) {
            output.append('│');
            for (int column = 0; column < widths.length; column++) {
                List<String> lines = row.get(column);
                String text = index < lines.size() ? lines.get(index) : "";
                String padding = " ".repeat(widths[column] - width(text));
                output.append(' ')
                        .append(numbersFrom <= column ? padding + text : text + padding)
                        .append(" │");
            }
            output.append('\n');
        }
    }

    /** The number of characters of {@code text}, each one column wide in the reports. */
    private static int width(String text) {
        return text.codePointCount(0, text.length());
    }

    /**
     * The first names of {@code members} in {@code group}, joined by commas, or {@code ?} for one
     * the group does not hold.
     *
     * @param group the group
     * @param members the members
     * @return the names
     * @throws EranosException for an identifier that is not one
     */
    public static String names(ExpenseGroup group, List<MemberId> members) throws EranosException {
        List<String> names = new ArrayList<>();
        for (MemberId member : members) {
            names.add(group.member(member).map(Member::firstName).orElse("?"));
        }
        return String.join(", ", names);
    }

    /**
     * Stops the example unless {@code holds}: a number of the report is not the example's.
     *
     * @param holds whether the number is the example's
     */
    public static void check(boolean holds) {
        if (!holds) {
            throw new IllegalStateException("a number of the report is not the example's");
        }
    }
}
