"""Print what every port prints its report with: the headings and the tables of the Rust examples, and the helpers the ports share."""

from collections.abc import Callable, Iterable, Sequence
from typing import assert_never

from eranos import EranosError, ExpenseSplit, Group, ItemisedExpenseSplit, MemberId, MemberSummary, Summary, WholeExpenseSplit


def print_lines(*lines: str) -> None:
    """
    Print lines, one below the other.

    Args:
        *lines: The lines.

    """
    print('\n'.join(lines))


def title(text: str) -> None:
    """
    Print text with a double line under it, as an example opens.

    Args:
        text: The title.

    """
    print_lines(text, '═' * len(text))


def section(text: str) -> None:
    """
    Print a section heading with a line under it, after a blank line.

    Args:
        text: The heading.

    """
    print_lines('', text, '─' * len(text))


def print_table(headers: Sequence[str], rows: Iterable[Sequence[str]], numbers_from: int) -> None:
    """
    Print rows under headers with rounded borders, as the Rust examples' tables are printed.

    The columns from numbers_from on are right-aligned, and a cell of several lines makes its row as tall as its tallest
    cell. A width counts code points, since the reports keep to characters one column wide.

    Args:
        headers: The heading of each column.
        rows: The rows, a text per column.
        numbers_from: The first column whose cells are right-aligned.

    """
    table = [[cell.split('\n') for cell in row] for row in (headers, *rows)]
    widths = [max(len(line) for row in table for line in row[column]) for column in range(len(headers))]

    def border(left: str, middle: str, right: str) -> str:
        return left + middle.join('─' * (width + 2) for width in widths) + right

    def aligned(column: int, text: str) -> str:
        return text.rjust(widths[column]) if numbers_from <= column else text.ljust(widths[column])

    def lines(row: list[list[str]]) -> list[str]:
        height = max(len(cell) for cell in row)
        padded = [cell + [''] * (height - len(cell)) for cell in row]
        return ['│' + ''.join(f' {aligned(column, cell[index])} │' for column, cell in enumerate(padded)) for index in range(height)]

    header, *body = table
    print_lines(border('╭', '┬', '╮'), *lines(header), border('├', '┼', '┤'), *(line for row in body for line in lines(row)), border('╰', '┴', '╯'))


def names(group: Group, members: Iterable[MemberId]) -> str:
    """
    Join the first names of members in group with commas, writing ``?`` for one the group does not hold.

    Args:
        group: The group.
        members: The members.

    Returns:
        The names, such as ``"Ana, Bob"``.

    """
    return ', '.join(member.first_name if (member := group.member(identifier)) is not None else '?' for identifier in members)


def participants(split: ExpenseSplit) -> list[MemberId]:
    """
    List every member who shares the cost of split, once each, in the order of their identifiers.

    Args:
        split: How an expense is divided.

    Returns:
        The members.

    """
    match split:
        case ItemisedExpenseSplit(items=items):
            members = [member for item in items for member in item.split.members]
        case WholeExpenseSplit(split=whole):
            members = list(whole.members)
        case _:
            assert_never(split)
    return sorted(set(members))


def refusal(change: Callable[[], object]) -> EranosError | None:
    """
    Tell what the library refuses change with.

    Args:
        change: The call to make.

    Returns:
        The library's error, or none for a change it accepts.

    """
    try:
        change()
    except EranosError as error:
        return error
    return None


def message(refused: EranosError | None) -> str:
    """
    Write the message of refused.

    Args:
        refused: What the library refused a change with, or none.

    Returns:
        The library's message, or ``"accepted"``.

    """
    return 'accepted' if refused is None else str(refused)


def totals(member: MemberId, summary: Summary) -> MemberSummary | None:
    """
    Find the totals of member in summary.

    Args:
        member: The member.
        summary: The group's summary.

    Returns:
        The member's totals, or none for a member the summary leaves out.

    """
    return next((found for found in summary.members if found.member == member), None)
