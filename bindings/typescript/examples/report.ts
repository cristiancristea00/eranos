// What every port prints its report with: the headings and the tables of the Rust examples, and the loading of the
// module the ports run in.

import { readFile } from 'node:fs/promises';

import {
    type ExpenseSplit,
    type ExpenseGroup,
    type MemberId,
    type MemberSummary,
    type Summary,
    EranosError,
    init,
    splitMembers,
} from '../src/index.js';

/** Loads the WebAssembly module beside the layer, which every other call runs in. */
export async function load(): Promise<void> {
    await init(await readFile(new URL('../src/internal/eranos_web.wasm', import.meta.url)));
}

/** Prints `lines`, one below the other. */
export function printLines(...lines: readonly string[]): void {
    console.log(lines.join('\n'));
}

/** Prints `text` with a double line under it, as an example opens. */
export function title(text: string): void {
    printLines(text, '═'.repeat(width(text)));
}

/** Prints a section heading with a line under it, after a blank line. */
export function section(text: string): void {
    printLines('', text, '─'.repeat(width(text)));
}

/** A column of a table of `Row`s: its header, and the text of a row's cell in it. */
export interface Column<Row> {
    readonly header: string;
    readonly text: (row: Row) => string;
}

/**
 * The columns of a table of `Row`s, in the order they are printed: the headers that the Rust example's
 * `#[tabled(rename = …)]` gives the fields of its row, in the order it declares them.
 */
export type Columns<Row> = ReadonlyArray<Column<Row>>;

/**
 * Prints `rows` in `columns` with rounded borders, as the Rust examples' tables are printed, right-aligning the columns
 * from `numbersFrom` on. A cell of several lines makes its row as tall as its tallest cell.
 */
export function printTable<Row>(columns: Columns<Row>, rows: readonly Row[], numbersFrom: number): void {
    const headers = columns.map((column) => column.header);
    const cells = rows.map((row) => columns.map((column) => column.text(row)));
    const table = [headers, ...cells].map((row) => row.map((cell) => cell.split('\n')));
    const widths = Array.from(headers.keys(), (column) => Math.max(...table.map((row) => Math.max(...(row[column] ?? []).map(width)))));
    const border = (left: string, middle: string, right: string): string => left + widths.map((size) => '─'.repeat(size + 2)).join(middle) + right;
    const lines = (row: ReadonlyArray<readonly string[]>): string[] =>
        Array.from(Array(Math.max(...row.map((cell) => cell.length))).keys(), (index) =>
            widths
                .map((size, column) => {
                    const text = row[column]?.[index] ?? '';
                    const padding = ' '.repeat(size - width(text));
                    return ` ${numbersFrom <= column ? padding + text : text + padding} `;
                })
                .reduce((line, cell) => `${line}${cell}│`, '│'),
        );
    const [header = [], ...body] = table;
    printLines(border('╭', '┬', '╮'), ...lines(header), border('├', '┼', '┤'), ...body.flatMap(lines), border('╰', '┴', '╯'));
}

/** The characters of the reports' text, each one column wide. */
const characters = new Intl.Segmenter();

/** The number of characters of `text`. */
function width(text: string): number {
    return Array.from(characters.segment(text)).length;
}

/** The first names of `members` in `group`, joined by commas, or `?` for one the group does not hold. */
export function names(group: ExpenseGroup, members: readonly MemberId[]): string {
    return members.map((member) => group.memberSync(member)?.firstName ?? '?').join(', ');
}

/** Every member who shares the cost of `split`, once each, in the order of their identifiers. */
export function participants(split: ExpenseSplit): MemberId[] {
    const members = 'itemised' === split.kind ? split.items.flatMap((item) => splitMembers(item.split)) : splitMembers(split.split);
    return [...new Set(members)].sort();
}

/** What the library refuses `change` with, or `undefined` for a change it accepts. */
export function refusal(change: () => unknown): EranosError | undefined {
    try {
        change();
        return undefined;
    } catch (error) {
        if (error instanceof EranosError) {
            return error;
        }
        throw error;
    }
}

/** The message of `refused`, or "accepted". */
export function message(refused: EranosError | undefined): string {
    return refused?.message ?? 'accepted';
}

/** The totals of `member` in `summary`. */
export function totals(member: MemberId, summary: Summary): MemberSummary | undefined {
    return summary.members.find((found) => found.member === member);
}
