// What the ports share: the entries they build, the text of the library's values, the members of splits, and the
// headings and the tables of the Rust examples, which they print.

#include "report.h"

#include "eranos.h"

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/// The bits of a byte that tell whether it continues a character in UTF-8.
static unsigned const CONTINUATION_MASK = 0xC0U;

/// Those bits in a byte that continues a character.
static unsigned const CONTINUATION = 0x80U;

/// How many rows a table has room for before it grows.
enum { FIRST_ROWS = 8 };

/// Ends the port with a failure, after the report it printed so far.
static void stop(void) {
    (void)fflush(stdout);
    // NOLINTNEXTLINE(concurrency-mt-unsafe): a port stops on its main thread, and no thread of its own calls exit().
    exit(EXIT_FAILURE);
}

void must(EranosError *const error) {
    if (NULL == error) {
        return;
    }
    EranosText *const message = eranosTextCreate();
    eranosErrorMessage(error, message);
    (void)fprintf(stderr, "the library refused a call the example makes: %s\n", eranosTextData(message));
    eranosTextDestroy(message);
    eranosErrorDestroy(error);
    stop();
}

void check(bool const holds, char const *const claim) {
    if (!holds) {
        (void)fprintf(stderr, "the report is not the example's: %s\n", claim);
        stop();
    }
}

bool isFailure(EranosError const *const error, EranosErrorKind const kind, EranosFailure const failure) {
    EranosFailure found = ERANOS_FAILURE_ACCOUNT_EMPTY_NAME;
    return NULL != error && kind == eranosErrorKind(error) && eranosErrorFailure(error, &found) && failure == found;
}

/// The copy of @p text in a line, which stops the port when it does not fit.
static Line lineFrom(char const *const text) {
    Line line = {{0}};
    size_t const length = strlen(text);
    check(LINE_SIZE > length, "a line of the report fits its buffer");
    memcpy(line.text, text, length + 1);
    return line;
}

Line lineOf(char const *const format, ...) {
    Line line = {{0}};
    va_list arguments;
    va_start(arguments, format);
    int const length = vsnprintf(line.text, sizeof line.text, format, arguments);
    va_end(arguments);
    check(0 <= length && LINE_SIZE > length, "a line of the report fits its buffer");
    return line;
}

void cellAdd(Cell *const cell, char const *const text) {
    size_t const length = strlen(cell->text.text);
    int const written = snprintf(cell->text.text + length, sizeof cell->text.text - length, "%s%s", 0 == cell->lines ? "" : "\n", text);
    check(0 <= written && sizeof cell->text.text - length > (size_t)written, "the lines fit a cell");
    ++cell->lines;
}

Line textLine(EranosText const *const text) { return lineFrom(eranosTextData(text)); }

Line moneyLine(EranosMoney *const money) {
    EranosText *const text = eranosTextCreate();
    must(eranosMoneyFormat(money, text));
    Line const line = textLine(text);
    eranosTextDestroy(text);
    eranosMoneyDestroy(money);
    return line;
}

Line balanceLine(EranosBalance *const balance) {
    EranosText *const text = eranosTextCreate();
    must(eranosBalanceFormat(balance, text));
    Line const line = textLine(text);
    eranosTextDestroy(text);
    eranosBalanceDestroy(balance);
    return line;
}

Line messageLine(EranosError *const error) {
    if (NULL == error) {
        return lineFrom("accepted");
    }
    EranosText *const message = eranosTextCreate();
    eranosErrorMessage(error, message);
    Line const line = textLine(message);
    eranosTextDestroy(message);
    eranosErrorDestroy(error);
    return line;
}

Id idOf(EranosText const *const text) {
    Id id = {{0}};
    check(sizeof id.text > eranosTextLength(text), "an identifier fits its buffer");
    memcpy(id.text, eranosTextData(text), eranosTextLength(text) + 1);
    return id;
}

Id addMember(EranosGroup *const group, char const *const firstName) {
    EranosMember *member = NULL;
    must(eranosMemberCreate(firstName, &member));
    EranosText *const text = eranosTextCreate();
    must(eranosGroupAddMember(group, member, text));
    Id const id = idOf(text);
    eranosTextDestroy(text);
    eranosMemberDestroy(member);
    return id;
}

EranosMoney *amountOf(char const *const text, char const *const currency) {
    EranosMoney *money = NULL;
    must(eranosMoneyParse(text, currency, &money));
    return money;
}

EranosSplit *equalSplit(Members const *const members) {
    EranosTextList *const list = eranosTextListCreate();
    for (size_t index = 0; members->count > index; ++index) {
        must(eranosTextListPush(list, members->ids[index].text));
    }
    EranosSplit *const split = eranosSplitCreateEqual(list);
    eranosTextListDestroy(list);
    return split;
}

EranosSplit *soleSplit(Id const member) {
    Members const members = {.count = 1, .ids = {member}};
    return equalSplit(&members);
}

EranosSplit *amountsSplit(char const *const currency, Share const *const shares, size_t const count) {
    EranosMemberAmountList *const list = eranosMemberAmountListCreate();
    for (size_t index = 0; count > index; ++index) {
        EranosMoney *const amount = amountOf(shares[index].value, currency);
        EranosMemberAmount *share = NULL;
        must(eranosMemberAmountCreate(amount, shares[index].member.text, &share));
        eranosMemberAmountListPush(list, share);
        eranosMemberAmountDestroy(share);
        eranosMoneyDestroy(amount);
    }
    EranosSplit *const split = eranosSplitCreateAmounts(list);
    eranosMemberAmountListDestroy(list);
    return split;
}

EranosSplit *partsSplit(Share const *const shares, size_t const count) {
    EranosMemberPartsList *const list = eranosMemberPartsListCreate();
    for (size_t index = 0; count > index; ++index) {
        int32_t hundredths = 0;
        must(eranosPartsParse(shares[index].value, &hundredths));
        EranosMemberParts *share = NULL;
        must(eranosMemberPartsCreate(hundredths, shares[index].member.text, &share));
        eranosMemberPartsListPush(list, share);
        eranosMemberPartsDestroy(share);
    }
    EranosSplit *const split = eranosSplitCreateParts(list);
    eranosMemberPartsListDestroy(list);
    return split;
}

EranosSplit *percentagesSplit(Share const *const shares, size_t const count) {
    EranosMemberPercentageList *const list = eranosMemberPercentageListCreate();
    for (size_t index = 0; count > index; ++index) {
        int32_t basisPoints = 0;
        must(eranosPercentageParse(shares[index].value, &basisPoints));
        EranosMemberPercentage *share = NULL;
        must(eranosMemberPercentageCreate(basisPoints, shares[index].member.text, &share));
        eranosMemberPercentageListPush(list, share);
        eranosMemberPercentageDestroy(share);
    }
    EranosSplit *const split = eranosSplitCreatePercentages(list);
    eranosMemberPercentageListDestroy(list);
    return split;
}

EranosCost *localCost(EranosMoney *const amount) {
    EranosCost *const cost = eranosCostCreateLocal(amount);
    eranosMoneyDestroy(amount);
    return cost;
}

EranosCost *convertedCost(EranosMoney *const amount, char const *const rate) {
    EranosCost *cost = NULL;
    must(eranosCostCreateConverted(amount, rate, &cost));
    eranosMoneyDestroy(amount);
    return cost;
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the payers, then the split, in the order the library's constructor reads them.
EranosExpense *expenseOf(EranosCost *const cost, EranosCalendarDate const date, char const *const name, EranosSplit *const payers,
                         EranosSplit *const split) {
    EranosExpenseSplit *const whole = eranosExpenseSplitCreateWhole(split);
    EranosExpense *expense = NULL;
    must(eranosExpenseCreate(cost, date, name, payers, whole, &expense));
    eranosExpenseSplitDestroy(whole);
    eranosSplitDestroy(split);
    eranosSplitDestroy(payers);
    eranosCostDestroy(cost);
    return expense;
}

Id addExpense(EranosGroup *const group, EranosExpense *const expense) {
    EranosText *const text = eranosTextCreate();
    must(eranosGroupAddExpense(group, expense, text));
    Id const id = idOf(text);
    eranosTextDestroy(text);
    eranosExpenseDestroy(expense);
    return id;
}

EranosError *expenseRefusal(EranosGroup *const group, EranosExpense *const expense) {
    EranosText *const text = eranosTextCreate();
    EranosError *const refusal = eranosGroupAddExpense(group, expense, text);
    eranosTextDestroy(text);
    eranosExpenseDestroy(expense);
    return refusal;
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the sender, then the receiver, in the order the library's constructor reads them.
EranosTransfer *transferOf(EranosCost *const cost, EranosCalendarDate const date, Id const from, Id const to) {
    EranosTransfer *transfer = NULL;
    must(eranosTransferCreate(cost, date, from.text, to.text, &transfer));
    eranosCostDestroy(cost);
    return transfer;
}

Id addTransfer(EranosGroup *const group, EranosTransfer *const transfer) {
    EranosText *const text = eranosTextCreate();
    must(eranosGroupAddTransfer(group, transfer, text));
    Id const id = idOf(text);
    eranosTextDestroy(text);
    eranosTransferDestroy(transfer);
    return id;
}

Line firstName(EranosGroup const *const group, char const *const member) {
    EranosMember *found = NULL;
    must(eranosGroupMember(group, member, &found));
    if (NULL == found) {
        return lineFrom("?");
    }
    EranosText *const text = eranosTextCreate();
    eranosMemberFirstName(found, text);
    Line const line = textLine(text);
    eranosTextDestroy(text);
    eranosMemberDestroy(found);
    return line;
}

Line names(EranosGroup const *const group, Members const *const members) {
    Line line = {{0}};
    size_t length = 0;
    for (size_t index = 0; members->count > index; ++index) {
        Line const name = firstName(group, members->ids[index].text);
        int const written = snprintf(line.text + length, sizeof line.text - length, "%s%s", 0 == index ? "" : ", ", name.text);
        check(0 <= written && sizeof line.text - length > (size_t)written, "the names fit a line");
        length += (size_t)written;
    }
    return line;
}

/// Adds each identifier of @p list that @p members lacks to @p members, at the end.
static void addAll(Members *const members, EranosTextList const *const list) {
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosTextListLength(list) > index; ++index) {
        (void)eranosTextListGet(list, index, text);
        Id const id = idOf(text);
        bool known = false;
        for (size_t other = 0; members->count > other; ++other) {
            known = known || 0 == strcmp(members->ids[other].text, id.text);
        }
        if (!known) {
            check(MEMBERS_SIZE > members->count, "a split names no more members than the ports hold");
            members->ids[members->count] = id;
            ++members->count;
        }
    }
    eranosTextDestroy(text);
}

Members splitMembers(EranosSplit const *const split) {
    Members members = {.count = 0};
    EranosTextList *list = NULL;
    must(eranosSplitParticipants(split, &list));
    addAll(&members, list);
    eranosTextListDestroy(list);
    return members;
}

/// Orders two identifiers by their text, which orders them as the library does.
static int compareIds(void const *const left, void const *const right) { return strcmp(((Id const *)left)->text, ((Id const *)right)->text); }

Members participants(EranosExpense const *const expense) {
    Members members = {.count = 0};
    EranosExpenseSplit *const split = eranosExpenseSplit(expense);
    EranosItemList *const items = eranosExpenseSplitItems(split);
    if (NULL == items) {
        EranosSplit *const whole = eranosExpenseSplitSplit(split);
        members = splitMembers(whole);
        eranosSplitDestroy(whole);
    } else {
        for (size_t index = 0; eranosItemListLength(items) > index; ++index) {
            EranosItem *const item = eranosItemListGet(items, index);
            EranosSplit *const itemSplit = eranosItemSplit(item);
            EranosTextList *list = NULL;
            must(eranosSplitParticipants(itemSplit, &list));
            addAll(&members, list);
            eranosTextListDestroy(list);
            eranosSplitDestroy(itemSplit);
            eranosItemDestroy(item);
        }
    }
    eranosItemListDestroy(items);
    eranosExpenseSplitDestroy(split);
    qsort(members.ids, members.count, sizeof members.ids[0], compareIds);
    return members;
}

EranosMemberSummary *totalsOf(EranosSummary const *const summary, char const *const member) {
    EranosMemberSummaryList *const list = eranosSummaryMembers(summary);
    EranosText *const text = eranosTextCreate();
    EranosMemberSummary *found = NULL;
    for (size_t index = 0; NULL == found && eranosMemberSummaryListLength(list) > index; ++index) {
        EranosMemberSummary *const totals = eranosMemberSummaryListGet(list, index);
        eranosMemberSummaryMember(totals, text);
        if (0 == strcmp(eranosTextData(text), member)) {
            found = totals;
        } else {
            eranosMemberSummaryDestroy(totals);
        }
    }
    eranosTextDestroy(text);
    eranosMemberSummaryListDestroy(list);
    return found;
}

// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the text, then what to find in it and what replaces it, as a search reads.
char *replaceFirst(char const *const text, char const *const pattern, char const *const replacement) {
    char const *const found = strstr(text, pattern);
    check(NULL != found, "the text holds what the example replaces");
    size_t const before = (size_t)(found - text);
    size_t const patternLength = strlen(pattern);
    size_t const replacementLength = strlen(replacement);
    size_t const after = strlen(found + patternLength);
    char *const copy = malloc(before + replacementLength + after + 1);
    check(NULL != copy, "the copy has room");
    memcpy(copy, text, before);
    // With its terminator, which the tail's copy overwrites: clang-tidy's bugprone-not-null-terminated-result refuses a
    // copy of a string's length alone wherever memcpy is glibc's plain function.
    memcpy(copy + before, replacement, replacementLength + 1);
    memcpy(copy + before + replacementLength, found + patternLength, after + 1);
    return copy;
}

void printLine(char const *const text) { (void)puts(text); }

void printFormatted(char const *const format, ...) {
    va_list arguments;
    va_start(arguments, format);
    (void)vprintf(format, arguments);
    va_end(arguments);
    (void)putchar('\n');
}

/// How many characters @p text holds, up to @p length bytes: the reports' characters are each one column wide.
static size_t widthOf(char const *const text, size_t const length) {
    size_t width = 0;
    for (size_t index = 0; length > index; ++index) {
        if (CONTINUATION != ((unsigned)(unsigned char)text[index] & CONTINUATION_MASK)) {
            ++width;
        }
    }
    return width;
}

/// Prints @p piece @p count times.
static void repeat(char const *const piece, size_t const count) {
    for (size_t index = 0; count > index; ++index) {
        (void)fputs(piece, stdout);
    }
}

/// Prints @p text with a line of @p piece under it, as long as the text.
static void underline(char const *const text, char const *const piece) {
    printLine(text);
    repeat(piece, widthOf(text, strlen(text)));
    (void)putchar('\n');
}

void title(char const *const text) { underline(text, "═"); }

void section(char const *const text) {
    (void)putchar('\n');
    underline(text, "─");
}

struct Table {
    /// How many rows its cells have room for.
    size_t capacity;
    /// The cells, row by row, the header's first.
    Line *cells;
    /// How many columns it has.
    size_t columns;
    /// The first column aligned to the right.
    size_t numbersFrom;
    /// How many rows it holds, the header's included.
    size_t rows;
};

/// The table's cell in @p row and @p column.
static Line *cellOf(Table const *const table, size_t const row, size_t const column) { return &table->cells[(row * table->columns) + column]; }

/// Adds a row of @p cells to @p table, making room for it when the table is full.
static void addRow(Table *const table, Line const *const cells) {
    if (table->capacity == table->rows) {
        size_t const capacity = 2 * table->capacity;
        Line *const grown = realloc(table->cells, capacity * table->columns * sizeof(Line));
        check(NULL != grown, "the table has room for its rows");
        table->cells = grown;
        table->capacity = capacity;
    }
    memcpy(cellOf(table, table->rows, 0), cells, table->columns * sizeof(Line));
    ++table->rows;
}

Table *tableCreate(size_t const columns, char const *const *const headers, size_t const numbersFrom) {
    Table *const table = malloc(sizeof(Table));
    check(NULL != table, "the table has room");
    table->capacity = FIRST_ROWS;
    table->cells = malloc(table->capacity * columns * sizeof(Line));
    table->columns = columns;
    table->numbersFrom = numbersFrom;
    table->rows = 0;
    check(NULL != table->cells, "the table has room for its rows");
    Line *const header = malloc(columns * sizeof(Line));
    check(NULL != header, "the table has room for its header");
    for (size_t column = 0; columns > column; ++column) {
        header[column] = lineFrom(headers[column]);
    }
    addRow(table, header);
    free(header);
    return table;
}

void tableAdd(Table *const table, Line const *const cells) { addRow(table, cells); }

/// How many lines @p text holds.
static size_t linesOf(char const *const text) {
    size_t lines = 1;
    for (char const *next = strchr(text, '\n'); NULL != next; next = strchr(next + 1, '\n')) {
        ++lines;
    }
    return lines;
}

/// The line @p index of @p text, as where it starts and how many bytes it holds; past its last line, an empty one.
static char const *lineAt(char const *const text, size_t const index, size_t *const length) {
    char const *start = text;
    for (size_t skipped = 0; index > skipped; ++skipped) {
        char const *const next = strchr(start, '\n');
        if (NULL == next) {
            *length = 0;
            return "";
        }
        start = next + 1;
    }
    char const *const end = strchr(start, '\n');
    *length = NULL == end ? strlen(start) : (size_t)(end - start);
    return start;
}

/// The corners and the joints of a table's border.
typedef struct Border {
    /// Where it starts.
    char const *left;
    /// Where it crosses a line between two columns.
    char const *middle;
    /// Where it ends.
    char const *right;
} Border;

/// The border above the header.
static Border const TOP = {.left = "╭", .middle = "┬", .right = "╮"};

/// The border between the header and the first row.
static Border const RULE = {.left = "├", .middle = "┼", .right = "┤"};

/// The border below the last row.
static Border const BOTTOM = {.left = "╰", .middle = "┴", .right = "╯"};

/// Prints @p border of @p table over @p widths.
static void printBorder(Table const *const table, size_t const *const widths, Border const border) {
    (void)fputs(border.left, stdout);
    for (size_t column = 0; table->columns > column; ++column) {
        repeat("─", widths[column] + 2);
        (void)fputs(table->columns == column + 1 ? border.right : border.middle, stdout);
    }
    (void)putchar('\n');
}

/// Prints the row @p row of @p table, each of its cells' lines on a line of its own, over @p widths.
static void printRow(Table const *const table, size_t const *const widths, size_t const row) {
    size_t height = 1;
    for (size_t column = 0; table->columns > column; ++column) {
        size_t const lines = linesOf(cellOf(table, row, column)->text);
        height = lines > height ? lines : height;
    }
    for (size_t index = 0; height > index; ++index) {
        (void)fputs("│", stdout);
        for (size_t column = 0; table->columns > column; ++column) {
            size_t length = 0;
            char const *const text = lineAt(cellOf(table, row, column)->text, index, &length);
            size_t const padding = widths[column] - widthOf(text, length);
            (void)putchar(' ');
            if (table->numbersFrom <= column) {
                repeat(" ", padding);
            }
            (void)fwrite(text, 1, length, stdout);
            if (table->numbersFrom > column) {
                repeat(" ", padding);
            }
            (void)fputs(" │", stdout);
        }
        (void)putchar('\n');
    }
}

void tablePrint(Table *const table) {
    size_t *const widths = calloc(table->columns, sizeof(size_t));
    check(NULL != widths, "the table has room for its widths");
    for (size_t row = 0; table->rows > row; ++row) {
        for (size_t column = 0; table->columns > column; ++column) {
            char const *const text = cellOf(table, row, column)->text;
            for (size_t index = 0; linesOf(text) > index; ++index) {
                size_t length = 0;
                size_t const width = widthOf(lineAt(text, index, &length), length);
                widths[column] = width > widths[column] ? width : widths[column];
            }
        }
    }
    printBorder(table, widths, TOP);
    printRow(table, widths, 0);
    printBorder(table, widths, RULE);
    for (size_t row = 1; table->rows > row; ++row) {
        printRow(table, widths, row);
    }
    printBorder(table, widths, BOTTOM);
    free(widths);
    free(table->cells);
    free(table);
}
