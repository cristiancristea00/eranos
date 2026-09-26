"""Share the group the tests read and change: Ana and Bob, who share a flat, its food, a dinner, and a repayment."""

import dataclasses
from collections.abc import Iterator
from typing import final

import pytest

from eranos import (
    CalendarDate,
    Category,
    CategoryId,
    Currency,
    EqualSplit,
    Expense,
    ExpenseId,
    Group,
    Language,
    LocalCost,
    Member,
    MemberId,
    Money,
    Transfer,
    TransferId,
    WholeExpenseSplit,
)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Flat:
    """A group of two, Ana and Bob, with a category, an expense, and a transfer, and the identifiers the group gave them."""

    euro: Currency
    """The group currency."""

    day: CalendarDate
    """The day of every entry."""

    group: Group
    """The group."""

    ana: MemberId
    """Ana, who paid the dinner and received the repayment."""

    bob: MemberId
    """Bob, who shared the dinner and sent the repayment."""

    food: CategoryId
    """The food category, which the dinner is in."""

    dinner: ExpenseId
    """The dinner of 30.00 EUR that Ana paid for the two of them."""

    repayment: TransferId
    """The 10.00 EUR that Bob sent Ana."""

    def euros(self, minor_units: int) -> Money:
        """
        Make an amount of euros.

        Args:
            minor_units: The cents.

        Returns:
            The amount.

        """
        return Money(self.euro, minor_units)

    def expense(self, name: str, payer: MemberId, members: tuple[MemberId, ...]) -> Expense:
        """
        Make an expense of 30.00 EUR, called name, that payer paid and members share equally.

        Args:
            name: What it was.
            payer: Who paid it.
            members: Who share it.

        Returns:
            The expense, to add.

        """
        return Expense(
            cost=LocalCost(self.euros(3000)), date=self.day, name=name, payers=EqualSplit((payer,)), split=WholeExpenseSplit(EqualSplit(members))
        )


@pytest.fixture
def flat() -> Iterator[Flat]:
    """
    Make the flat: its group, members, category, dinner, and repayment.

    Yields:
        The flat, whose group is freed after the test.

    """
    euro = Currency('EUR')
    day = CalendarDate(23, 9, 2026)
    with Group.create(euro, 'Flat', description='Rent', language=Language('pt-pt')) as group:
        ana = group.add_member(Member(first_name='Ana'))
        bob = group.add_member(Member(first_name='Bob'))
        food = group.add_category(Category(name='Food'))
        dinner = group.add_expense(
            Expense(
                cost=LocalCost(Money(euro, 3000)),
                date=day,
                name='Dinner',
                payers=EqualSplit((ana,)),
                split=WholeExpenseSplit(EqualSplit((ana, bob))),
                category=food,
            )
        )
        repayment = group.add_transfer(Transfer(cost=LocalCost(Money(euro, 1000)), date=day, receiver=ana, sender=bob))
        yield Flat(euro=euro, day=day, group=group, ana=ana, bob=bob, food=food, dinner=dinner, repayment=repayment)
