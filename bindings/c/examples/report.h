// What the ports share: the entries they build, the text of the library's values, the members of splits, and the
// headings and the tables of the Rust examples, which they print.

#ifndef REPORT_H
#define REPORT_H

#include "eranos.h"

#include <stddef.h>

/// The most bytes a line of a report holds, its NUL included: every value, message, and cell of the examples fits.
static constexpr size_t LINE_SIZE = 512;

/// The most members a split or an expense of the examples names, each once.
static constexpr size_t MEMBERS_SIZE = 8;

/// A line of text, ended by a NUL.
typedef struct Line {
    /// The text.
    char text[LINE_SIZE];
} Line;

/// How many bytes an identifier's text holds, its NUL included: the library refuses an identifier longer than 64 bytes of UTF-8.
static constexpr size_t ID_SIZE = 65;

/// An identifier the app gave or the library generated, as its text.
typedef struct Id {
    /// The identifier's text, ended by a NUL.
    char text[ID_SIZE];
} Id;

/// The members a split or an expense names.
typedef struct Members {
    /// How many there are.
    size_t count;
    /// Their identifiers, the first count of them in use.
    Id ids[MEMBERS_SIZE];
} Members;

/// A table cell of several lines, which a port fills line by line.
typedef struct Cell {
    /// How many lines it holds.
    size_t lines;
    /// Its text, the lines separated by line breaks.
    Line text;
} Cell;

/// Stops the port, printing the library's message, when @p error is an error the example does not expect.
///
/// @param error The outcome of a call, which this destroys
void must(EranosError *error);

/// Stops the port, naming @p claim, unless @p holds: a number of the report is not the example's.
///
/// @param holds Whether the number is the example's
/// @param claim What the number is
void check(bool holds, char const *claim);

/// Whether @p error is a failure of @p kind, and within it of @p failure.
///
/// @param error The error, or nullptr
/// @param kind The module that refused the call
/// @param failure The failure within the module
/// @return Whether it is
bool isFailure(EranosError const *error, EranosErrorKind kind, EranosFailure failure);

/// The text that @p format writes of the values after it, as printf() writes it.
///
/// @param format The format
/// @return The text
[[gnu::format(printf, 1, 2)]] Line lineOf(char const *format, ...);

/// Adds @p text to @p cell as its last line, which may be empty.
///
/// @param cell The cell
/// @param text The line
void cellAdd(Cell *cell, char const *text);

/// The text of @p text.
///
/// @param text The library's text
/// @return The text
Line textLine(EranosText const *text);

/// The text of @p money, as the library writes it; destroys @p money.
///
/// @param money The amount
/// @return The text
Line moneyLine(EranosMoney *money);

/// The text of @p balance, as the library writes it; destroys @p balance.
///
/// @param balance The balance
/// @return The text
Line balanceLine(EranosBalance *balance);

/// The message of @p error, a change the library refuses, or "accepted" for nullptr; destroys @p error.
///
/// @param error The outcome of the change
/// @return The message
Line messageLine(EranosError *error);

/// The identifier the library wrote in @p text.
///
/// @param text The identifier's text
/// @return The identifier
Id idOf(EranosText const *text);

/// Adds a member called @p firstName to @p group.
///
/// @param group The group
/// @param firstName The member's first name
/// @return The member's identifier
Id addMember(EranosExpenseGroup *group, char const *firstName);

/// A member's share of a split: an amount, a number of parts, or a percentage, as text.
typedef struct Share {
    /// The member.
    Id member;
    /// The share, as the library reads it: an amount such as 15.00, parts such as 0.5, or a percentage such as 40.
    char const *value;
} Share;

/// The amount @p text of @p currency.
///
/// @param text The amount, such as 12.50
/// @param currency The currency's code
/// @return The amount, which the caller destroys
EranosMoney *amountOf(char const *text, char const *currency);

/// The split that @p members share equally.
///
/// @param members The members
/// @return The split, which the caller destroys
EranosSplit *equalSplit(Members const *members);

/// The split of which @p member takes the whole.
///
/// @param member The member
/// @return The split, which the caller destroys
EranosSplit *soleSplit(Id member);

/// The split that gives each member of @p shares the amount of @p currency beside them.
///
/// @param currency The currency's code
/// @param shares The members and their amounts
/// @param count How many shares there are
/// @return The split, which the caller destroys
EranosSplit *amountsSplit(char const *currency, Share const *shares, size_t count);

/// The split that gives each member of @p shares the number of parts beside them.
///
/// @param shares The members and their parts
/// @param count How many shares there are
/// @return The split, which the caller destroys
EranosSplit *partsSplit(Share const *shares, size_t count);

/// The split that gives each member of @p shares the percentage beside them.
///
/// @param shares The members and their percentages
/// @param count How many shares there are
/// @return The split, which the caller destroys
EranosSplit *percentagesSplit(Share const *shares, size_t count);

/// The cost of @p amount in the group's currency; destroys @p amount.
///
/// @param amount The amount
/// @return The cost, which the caller destroys
EranosCost *localCost(EranosMoney *amount);

/// The cost of @p amount in another currency, entered at @p rate; destroys @p amount.
///
/// @param amount The amount
/// @param rate What one unit of its currency is worth in the group's, such as 0.2
/// @return The cost, which the caller destroys
EranosCost *convertedCost(EranosMoney *amount, char const *rate);

/// An expense of @p cost, which @p split shares whole; destroys @p cost, @p payers, and @p split.
///
/// @param name What was bought
/// @param cost What it cost
/// @param payers Who paid it
/// @param split Who shares it
/// @param date The day it was bought
/// @return The expense, which the caller destroys
EranosExpense *expenseOf(char const *name, EranosCost *cost, EranosSplit *payers, EranosSplit *split, EranosCalendarDate date);

/// Adds @p expense to @p group, and destroys it.
///
/// @param group The group
/// @param expense The expense
/// @return The identifier the group gave it
Id addExpense(EranosExpenseGroup *group, EranosExpense *expense);

/// What @p group refuses @p expense with, or nullptr when it adds it; destroys @p expense.
///
/// @param group The group
/// @param expense The expense
/// @return The refusal, which the caller destroys, or nullptr
EranosError *expenseRefusal(EranosExpenseGroup *group, EranosExpense *expense);

/// A transfer from @p from to @p to of @p cost; destroys @p cost.
///
/// @param from Who sent it
/// @param to Who received it
/// @param cost What was sent
/// @param date The day it was sent
/// @return The transfer, which the caller destroys
EranosTransfer *transferOf(Id from, Id to, EranosCost *cost, EranosCalendarDate date);

/// Adds @p transfer to @p group, and destroys it.
///
/// @param group The group
/// @param transfer The transfer
/// @return The identifier the group gave it
Id addTransfer(EranosExpenseGroup *group, EranosTransfer *transfer);

/// The first name of @p member in @p group, or ? for one the group does not hold.
///
/// @param group The group
/// @param member The member's identifier
/// @return The first name
Line firstName(EranosExpenseGroup const *group, char const *member);

/// The first names of @p members in @p group, joined by commas.
///
/// @param group The group
/// @param members The members
/// @return The names
Line names(EranosExpenseGroup const *group, Members const *members);

/// The members that @p split names, in its order.
///
/// @param split The split
/// @return The members
Members splitMembers(EranosSplit const *split);

/// Every member who shares the cost of @p expense, once each, in the order of their identifiers.
///
/// @param expense The expense
/// @return The members
Members participants(EranosExpense const *expense);

/// The totals of @p member in @p summary, which the caller destroys, or nullptr when the summary lists no totals of theirs.
///
/// @param summary The summary
/// @param member The member's identifier
/// @return The totals, or nullptr
EranosMemberSummary *totalsOf(EranosSummary const *summary, char const *member);

/// The copy of @p text with the first @p pattern in it replaced by @p replacement, which stops the port when @p text does
/// not hold @p pattern.
///
/// @param text The text
/// @param pattern What to replace
/// @param replacement What replaces it
/// @return The copy, which the caller frees
char *replaceFirst(char const *text, char const *pattern, char const *replacement);

/// Prints @p text, then a line break.
///
/// @param text The text, which may hold line breaks of its own
void printLine(char const *text);

/// Prints the text that @p format writes of the values after it, as printf() writes it, then a line break.
///
/// @param format The format
[[gnu::format(printf, 1, 2)]] void printFormatted(char const *format, ...);

/// Prints @p text with a double line under it, as an example opens.
///
/// @param text The title
void title(char const *text);

/// Prints a section heading with a line under it, after a blank line.
///
/// @param text The heading
void section(char const *text);

/// A table that a port fills row by row and prints once.
typedef struct Table Table;

/// A table of @p columns columns under @p headers, whose columns from @p numbersFrom on are aligned to the right.
///
/// @param columns How many columns it has
/// @param headers The columns' headers
/// @param numbersFrom The first column aligned to the right
/// @return The table, which tablePrint() prints and destroys
Table *tableCreate(size_t columns, char const *const *headers, size_t numbersFrom);

/// Adds a row of @p cells, one per column, which the table copies; a cell of several lines makes its row as tall as its
/// tallest cell.
///
/// @param table The table
/// @param cells The row's cells
void tableAdd(Table *table, Line const *cells);

/// Prints @p table with rounded borders, as the Rust examples' tables are printed, and destroys it.
///
/// @param table The table
void tablePrint(Table *table);

#endif // REPORT_H
