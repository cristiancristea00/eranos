"""Share the group the tests read and change: Ana and Bob, who share a flat, its food, a dinner, and a repayment."""

import dataclasses
from collections.abc import Iterator
from typing import final

import pytest

from eranos import (
    CalendarDate,
    Category,
    CategoryId,
    CurrencyCode,
    EqualSplit,
    Expense,
    ExpenseId,
    ExpenseGroup,
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

    group: ExpenseGroup
    """The group."""

    euro: CurrencyCode
    """The group currency."""

    day: CalendarDate
    """The day of every entry."""

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
        return Money(minor_units, self.euro)

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
            name=name, cost=LocalCost(self.euros(3000)), payers=EqualSplit((payer,)), split=WholeExpenseSplit(EqualSplit(members)), date=self.day
        )


@pytest.fixture
def flat() -> Iterator[Flat]:
    """
    Make the flat: its group, members, category, dinner, and repayment.

    Yields:
        The flat, whose group is freed after the test.

    """
    euro = CurrencyCode('EUR')
    day = CalendarDate(2026, 9, 23)
    with ExpenseGroup.create('Flat', euro, description='Rent', language=Language('pt-pt')) as group:
        ana = group.add_member(Member(first_name='Ana'))
        bob = group.add_member(Member(first_name='Bob'))
        food = group.add_category(Category(name='Food'))
        dinner = group.add_expense(
            Expense(
                name='Dinner',
                cost=LocalCost(Money(3000, euro)),
                payers=EqualSplit((ana,)),
                split=WholeExpenseSplit(EqualSplit((ana, bob))),
                date=day,
                category=food,
            )
        )
        repayment = group.add_transfer(Transfer(sender=bob, receiver=ana, cost=LocalCost(Money(1000, euro)), date=day))
        yield Flat(group=group, euro=euro, day=day, ana=ana, bob=bob, food=food, dinner=dinner, repayment=repayment)
