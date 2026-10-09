"""
Hold what the sections of the tour share: the members and categories they name, and the amounts and dates of the trip.

It also holds what several sections print: the rows of the tables they share, and the tables of the balances, the spending
per category, and the entries.

"""

import dataclasses
from collections.abc import Callable, Iterable
from typing import ClassVar, assert_never, final, override

from eranos import (
    Balance,
    CalendarDate,
    CategoryId,
    ConvertedCost,
    Cost,
    CurrencyCode,
    ExchangeRate,
    Expense,
    ExpenseGroup,
    LocalCost,
    Member,
    MemberId,
    Money,
    Summary,
    Transfer,
)
from examples.report import Row, names, print_table


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


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class RefusalRow(Row):
    """A row of a table of refused changes: the change, and the library's message."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Change', 'Refused with')

    change: str
    """The change the group refuses."""

    error: str
    """The library's message."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.change, self.error


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class EntryRow(Row):
    """A row of the table of entries: an expense or a transfer, as entered and in the group currency."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Entry', 'Paid by', 'Entered as', 'Rate', 'In the group currency')

    name: str
    """What the entry was."""

    payer: str
    """Who paid it."""

    original: Money
    """The entry's amount as entered."""

    rate: ExchangeRate
    """The rate the entry was entered at."""

    converted: Money
    """The entry's amount in the group currency."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.payer, str(self.original), str(self.rate), str(self.converted)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class BalanceRow(Row):
    """A row of the table of balances: what a member paid, consumed, sent, and received, and where they stand."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Paid', 'Consumed', 'Sent', 'Received', 'Balance')

    name: str
    """The member's first name."""

    paid: Money
    """What the member paid."""

    consumed: Money
    """What the member consumed."""

    sent: Money
    """What the member sent in transfers."""

    received: Money
    """What the member received in transfers."""

    balance: Balance
    """Where the member stands."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, str(self.paid), str(self.consumed), str(self.sent), str(self.received), str(self.balance)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class CategoryRow(Row):
    """A row of the table of spending per category."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Category', 'Expenses', 'Total')

    name: str
    """The category's name."""

    expenses: int
    """The number of expenses in the category."""

    total: Money
    """What the group spent in the category."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, str(self.expenses), str(self.total)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class PaymentRow(Row):
    """A row of the table of a plan's payments."""

    HEADERS: ClassVar[tuple[str, ...]] = ('From', 'To', 'Amount')

    sender: str
    """Who pays."""

    receiver: str
    """Who is paid."""

    amount: Money
    """What is paid."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.sender, self.receiver, str(self.amount)


def eur(text: str) -> Money:
    """
    Parse an amount in euros.

    Args:
        text: The amount's text, such as ``"12.50"``.

    Returns:
        The amount.

    """
    return Money.parse(text, CurrencyCode('EUR'))


def ron(text: str) -> Money:
    """
    Parse an amount in lei.

    Args:
        text: The amount's text, such as ``"12.50"``.

    Returns:
        The amount.

    """
    return Money.parse(text, CurrencyCode('RON'))


def october(day: int) -> CalendarDate:
    """
    Make a date of the trip, in October 2026.

    Args:
        day: The day of the month.

    Returns:
        The date.

    """
    return CalendarDate(2026, 10, day)


def name(group: ExpenseGroup, member: MemberId) -> str:
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


def _balance_row(member: Member, summary: Summary) -> BalanceRow:
    """
    Write the row of member among the balances.

    Args:
        member: The member.
        summary: The group's summary.

    Returns:
        The row.

    """
    found = summary.member(member.id)
    return BalanceRow(name=member.first_name, paid=found.paid, consumed=found.share, sent=found.sent, received=found.received, balance=found.balance)


def print_balances(group: ExpenseGroup) -> None:
    """
    Print what each member paid, consumed, sent, and received, and their balance.

    Args:
        group: The group.

    """
    summary = group.summary()
    print_table(BalanceRow, [_balance_row(member, summary) for member in group.members()], 1)


def print_categories(group: ExpenseGroup) -> None:
    """
    Print what the group spent in each category, and on how many expenses.

    Args:
        group: The group.

    """
    expenses = group.expenses()
    print_table(
        CategoryRow,
        [
            CategoryRow(
                name='?' if (category := group.category(total.category)) is None else category.name,
                expenses=sum(1 for expense in expenses if expense.category == total.category),
                total=total.total,
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
        case LocalCost():
            return ExchangeRate('1')
        case ConvertedCost(rate=rate):
            return rate
        case _:
            assert_never(cost)


def _expense_row(group: ExpenseGroup, expense: Expense) -> EntryRow:
    """
    Write the row of expense among the entries.

    Args:
        group: The group.
        expense: An expense of the group.

    Returns:
        The row: the expense's name, who paid it, its total as entered, its rate, and its total in the group currency.

    """
    shares = group.expense_shares(expense.id)
    return EntryRow(
        name=expense.name,
        payer=names(group, expense.payers.members),
        original=expense.total(),
        rate=_rate(expense.cost),
        converted=Money.zero(group.currency()) if shares is None else shares.total,
    )


def _transfer_row(group: ExpenseGroup, transfer: Transfer) -> EntryRow:
    """
    Write the row of transfer among the entries.

    Args:
        group: The group.
        transfer: A transfer of the group.

    Returns:
        The row: whom the transfer went to, who sent it, its amount as entered, its rate, and its amount in the group
        currency.

    """
    amount = group.transfer_amount(transfer.id)
    return EntryRow(
        name=f'Transfer to {name(group, transfer.receiver)}',
        payer=name(group, transfer.sender),
        original=transfer.cost.amount,
        rate=_rate(transfer.cost),
        converted=Money.zero(group.currency()) if amount is None else amount,
    )


def print_entries(group: ExpenseGroup, shown: Callable[[CurrencyCode], bool]) -> None:
    """
    Print every expense and transfer in a currency shown accepts, as entered and in the group currency.

    Args:
        group: The group.
        shown: Whether to print the entries in a currency.

    """
    print_table(
        EntryRow,
        [
            *(_expense_row(group, expense) for expense in group.expenses() if shown(expense.cost.amount.currency)),
            *(_transfer_row(group, transfer) for transfer in group.transfers() if shown(transfer.cost.amount.currency)),
        ],
        2,
    )
