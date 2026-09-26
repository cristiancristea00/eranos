"""
Hold what the sections of the tour share: the members and categories they name, and the amounts and dates of the trip.

It also holds the tables that several sections print: the balances, the spending per category, and the entries.

"""

import dataclasses
from collections.abc import Callable, Iterable
from typing import assert_never, final

from eranos import (
    CalendarDate,
    CategoryId,
    ConvertedCost,
    Cost,
    Currency,
    ExchangeRate,
    Expense,
    Group,
    LocalCost,
    Member,
    MemberId,
    Money,
    SettledBalance,
    Summary,
    Transfer,
)
from examples.report import names, print_table, totals


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class People:
    """The members of the trip."""

    ana: MemberId
    bob: MemberId
    carol: MemberId
    dan: MemberId
    eva: MemberId
    filip: MemberId


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Categories:
    """The categories the group adds to the built-in one."""

    food: CategoryId
    lodging: CategoryId
    transport: CategoryId


def eur(text: str) -> Money:
    """
    Parse an amount in euros.

    Args:
        text: The amount's text, such as ``"12.50"``.

    Returns:
        The amount.

    """
    return Money.parse(text, Currency('EUR'))


def ron(text: str) -> Money:
    """
    Parse an amount in lei.

    Args:
        text: The amount's text, such as ``"12.50"``.

    Returns:
        The amount.

    """
    return Money.parse(text, Currency('RON'))


def october(day: int) -> CalendarDate:
    """
    Make a date of the trip, in October 2026.

    Args:
        day: The day of the month.

    Returns:
        The date.

    """
    return CalendarDate(day, 10, 2026)


def name(group: Group, member: MemberId) -> str:
    """
    Find the first name of member in group.

    Args:
        group: The group.
        member: The member.

    Returns:
        The first name, or ``?`` for a member the group does not hold.

    """
    return names(group, (member,))


def cell(lines: Iterable[str]) -> str:
    """
    Put lines one below the other in a table cell.

    Args:
        lines: The lines.

    Returns:
        The cell's text.

    """
    return '\n'.join(lines)


def built_in_category(group: Group) -> CategoryId:
    """
    Find the identifier of the built-in category, which the group lists first.

    Args:
        group: The group.

    Returns:
        The identifier.

    """
    built_in = group.categories()[0].id
    assert built_in is not None, 'every category a group holds has an identifier'
    return built_in


def _balance_row(member: Member, summary: Summary, zero: str) -> tuple[str, ...]:
    """
    Write the row of member among the balances.

    Args:
        member: The member.
        summary: The group's summary.
        zero: Zero in the group currency, as the library writes it.

    Returns:
        The member's first name, what they paid, consumed, sent, and received, and their balance.

    """
    found = None if member.id is None else totals(member.id, summary)
    if found is None:
        # The summary lists the members with a total; every total of the others is zero.
        return member.first_name, zero, zero, zero, zero, str(SettledBalance())
    return member.first_name, str(found.paid), str(found.share), str(found.sent), str(found.received), str(found.balance)


def print_balances(group: Group) -> None:
    """
    Print what each member paid, consumed, sent, and received, and their balance.

    Args:
        group: The group.

    """
    summary = group.summary()
    zero = str(Money(group.currency(), 0))
    print_table(('Member', 'Paid', 'Consumed', 'Sent', 'Received', 'Balance'), [_balance_row(member, summary, zero) for member in group.members()], 1)


def print_categories(group: Group) -> None:
    """
    Print what the group spent in each category, and on how many expenses.

    Args:
        group: The group.

    """
    expenses = group.expenses()
    built_in = built_in_category(group)
    print_table(
        ('Category', 'Expenses', 'Total'),
        [
            (
                '?' if (category := group.category(total.category)) is None else category.name,
                # An expense in the built-in category names none.
                str(sum(1 for expense in expenses if (expense.category or built_in) == total.category)),
                str(total.total),
            )
            for total in group.summary().category_totals
        ],
        1,
    )


def _rate(cost: Cost) -> ExchangeRate:
    """
    Find the rate cost was entered at: 1 for a cost in the group currency.

    Args:
        cost: The cost of an entry.

    Returns:
        The rate into the group currency.

    """
    match cost:
        case ConvertedCost(rate=rate):
            return rate
        case LocalCost():
            return ExchangeRate('1')
        case _:
            assert_never(cost)


def _expense_row(group: Group, expense: Expense) -> tuple[str, ...]:
    """
    Write the row of expense among the entries.

    Args:
        group: The group.
        expense: An expense of the group.

    Returns:
        The expense's name, who paid it, its total as entered, its rate, and its total in the group currency.

    """
    shares = None if expense.id is None else group.expense_shares(expense.id)
    converted = Money(group.currency(), 0) if shares is None else shares.total
    return expense.name, names(group, expense.payers.members), str(expense.total()), str(_rate(expense.cost)), str(converted)


def _transfer_row(group: Group, transfer: Transfer) -> tuple[str, ...]:
    """
    Write the row of transfer among the entries.

    Args:
        group: The group.
        transfer: A transfer of the group.

    Returns:
        Whom the transfer went to, who sent it, its amount as entered, its rate, and its amount in the group currency.

    """
    amount = None if transfer.id is None else group.transfer_amount(transfer.id)
    converted = Money(group.currency(), 0) if amount is None else amount
    return (
        f'Transfer to {name(group, transfer.receiver)}',
        name(group, transfer.sender),
        str(transfer.cost.amount),
        str(_rate(transfer.cost)),
        str(converted),
    )


def print_entries(group: Group, shown: Callable[[Currency], bool]) -> None:
    """
    Print every expense and transfer in a currency shown accepts, as entered and in the group currency.

    Args:
        group: The group.
        shown: Whether to print the entries in a currency.

    """
    print_table(
        ('Entry', 'Paid by', 'Entered as', 'Rate', 'In the group currency'),
        [
            *(_expense_row(group, expense) for expense in group.expenses() if shown(expense.cost.amount.currency)),
            *(_transfer_row(group, transfer) for transfer in group.transfers() if shown(transfer.cost.amount.currency)),
        ],
        2,
    )
