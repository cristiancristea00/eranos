"""
Print every way to divide an expense: equally, by percentage, by parts (including half a part for a child), by exact amounts, and with several payers.

The port of the Rust example ``split_methods``, which prints the same report.

"""

import dataclasses
from collections.abc import Mapping
from typing import ClassVar, final, override

from eranos import (
    AmountsSplit,
    CalendarDate,
    CurrencyCode,
    EqualSplit,
    Expense,
    ExpenseId,
    ExpenseGroup,
    LocalCost,
    Member,
    MemberAmount,
    MemberId,
    MemberParts,
    MemberPercentage,
    Money,
    Parts,
    PartsSplit,
    Percentage,
    PercentagesSplit,
    WholeExpenseSplit,
)
from examples.report import Row, names, print_lines, print_table, refusal, section, title


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ShareRow(Row):
    """A row of the table of how one expense divides: a member's rule, what they paid, and what they consumed."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Rule', 'Paid', 'Consumed')

    name: str
    """The member's first name."""

    rule: str
    """The rule the member's share follows."""

    paid: Money
    """What the member paid."""

    consumed: Money
    """What the member consumed."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.rule, str(self.paid), str(self.consumed)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class TotalRow(Row):
    """A row of the table of the holiday in total: what a member paid and consumed, and where they stand."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Paid', 'Consumed', 'Balance')

    name: str
    """The member's first name."""

    paid: Money
    """What the member paid."""

    consumed: Money
    """What the member consumed."""

    balance: str
    """Where the member stands."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, str(self.paid), str(self.consumed), self.balance


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Family:
    """The family and the day every expense happened."""

    ana: MemberId
    """Ana, a parent."""

    bob: MemberId
    """Bob, a parent."""

    mia: MemberId
    """Mia, their daughter, who counts as half an adult."""

    day: CalendarDate
    """The day every expense happened."""


def euros(text: str) -> Money:
    """
    Parse the amount written text, in euros.

    Args:
        text: The amount's text, such as ``"10.00"``.

    Returns:
        The amount.

    """
    return Money.parse(text, CurrencyCode('EUR'))


def report(group: ExpenseGroup, expense_id: ExpenseId, rules: Mapping[MemberId, str]) -> None:
    """
    Print how one expense divides: every involved member's rule, what they paid, and what they consumed.

    Args:
        group: The group that holds the expense.
        expense_id: The expense's identifier.
        rules: The rule each member's share follows.

    """
    expense = group.expense(expense_id)
    shares = group.expense_shares(expense_id)
    if expense is None or shares is None:
        return
    print_lines(f'{expense.name}, {shares.total}:')
    print_table(
        ShareRow,
        [
            ShareRow(name=names(group, (share.member,)), rule=rules.get(share.member, '(payer only)'), paid=share.paid, consumed=share.share)
            for share in shares.members
        ],
        2,
    )


def thirds(family: Family) -> dict[MemberId, str]:
    """
    Name the rule of each member of an expense the family shares equally.

    Args:
        family: The family.

    Returns:
        The rule of each member.

    """
    return dict.fromkeys((family.ana, family.bob, family.mia), 'an equal third')


def equally(group: ExpenseGroup, family: Family) -> None:
    """
    Print an expense the family shares equally, whose leftover cent one member pays.

    Args:
        group: The group.
        family: The family.

    """
    section('1. Equally')
    print_lines(
        'Everyone pays the same. 10.00 does not divide by three, so one member pays the',
        'leftover cent; which member it is rotates from expense to expense, so over a trip',
        'nobody always pays it.',
    )
    taxi = group.add_expense(
        Expense.create(
            name='Taxi',
            cost=LocalCost(euros('10.00')),
            payers=EqualSplit((family.ana,)),
            split=WholeExpenseSplit(EqualSplit((family.ana, family.bob, family.mia))),
            date=family.day,
        )
    )
    report(group, taxi, thirds(family))


def by_percentage(group: ExpenseGroup, family: Family) -> None:
    """
    Print an expense the family divides by percentage.

    Args:
        group: The group.
        family: The family.

    """
    section('2. By percentage')
    print_lines(
        'Everyone pays a percentage of the amount; the percentages must add up to exactly',
        "100 %. Mia's room is smaller, so she takes 20 %.",
    )
    rent = group.add_expense(
        Expense.create(
            name='Flat',
            cost=LocalCost(euros('500.00')),
            payers=EqualSplit((family.bob,)),
            split=WholeExpenseSplit(
                PercentagesSplit(
                    (
                        MemberPercentage(family.ana, Percentage.parse('40')),
                        MemberPercentage(family.bob, Percentage.parse('40')),
                        MemberPercentage(family.mia, Percentage.parse('20')),
                    )
                )
            ),
            date=family.day,
        )
    )
    report(group, rent, {family.ana: '40%', family.bob: '40%', family.mia: '20%'})


def by_parts(group: ExpenseGroup, family: Family) -> ExpenseId:
    """
    Print an expense the family divides by parts, half a part for the child.

    Args:
        group: The group.
        family: The family.

    Returns:
        The expense's identifier.

    """
    section('3. By parts')
    print_lines(
        'Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:',
        '2.5 parts in total, so one part is 125.00 / 2.5 = 50.00.',
    )
    food = group.add_expense(
        Expense.create(
            name='Food',
            cost=LocalCost(euros('125.00')),
            payers=EqualSplit((family.ana,)),
            split=WholeExpenseSplit(
                PartsSplit(
                    (
                        MemberParts(family.ana, Parts.parse('1')),
                        MemberParts(family.bob, Parts.parse('1')),
                        MemberParts(family.mia, Parts.parse('0.5')),
                    )
                )
            ),
            date=family.day,
        )
    )
    report(group, food, {family.ana: '1 part', family.bob: '1 part', family.mia: '0.5 parts'})
    return food


def by_amounts(group: ExpenseGroup, family: Family) -> None:
    """
    Print an expense the family divides by exact amounts.

    Args:
        group: The group.
        family: The family.

    """
    section('4. By exact amounts')
    print_lines('Everyone pays a fixed amount, such as a ticket price; the amounts must add up to', 'exactly the expense amount.')
    tickets = group.add_expense(
        Expense.create(
            name='Museum',
            cost=LocalCost(euros('35.00')),
            payers=EqualSplit((family.bob,)),
            split=WholeExpenseSplit(
                AmountsSplit(
                    (
                        MemberAmount(family.ana, euros('15.00')),
                        MemberAmount(family.bob, euros('15.00')),
                        MemberAmount(family.mia, euros('5.00')),
                    )
                )
            ),
            date=family.day,
        )
    )
    report(group, tickets, {family.ana: '15.00 EUR', family.bob: '15.00 EUR', family.mia: '5.00 EUR'})


def several_payers(group: ExpenseGroup, family: Family) -> None:
    """
    Print an expense that two members paid, each an exact amount, and the family shares equally.

    Args:
        group: The group.
        family: The family.

    """
    section('5. Several payers')
    print_lines(
        'The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are',
        'divided with the same methods as the participants.',
    )
    dinner = group.add_expense(
        Expense.create(
            name='Dinner',
            cost=LocalCost(euros('90.00')),
            payers=AmountsSplit((MemberAmount(family.ana, euros('50.00')), MemberAmount(family.bob, euros('40.00')))),
            split=WholeExpenseSplit(EqualSplit((family.ana, family.bob, family.mia))),
            date=family.day,
        )
    )
    report(group, dinner, thirds(family))


def mistakes(group: ExpenseGroup, family: Family) -> None:
    """
    Print how the library refuses a split whose percentages add up to 90 %.

    The layer builds an expense only when a group adds it, so the group's add_expense refuses it where Rust's build does.

    Args:
        group: The group.
        family: The family.

    """
    section('6. Mistakes are refused')
    print_lines('A percentage split that adds up to 90 % is refused when the expense is built:')
    invalid = Expense.create(
        name='Invalid',
        cost=LocalCost(euros('10.00')),
        payers=EqualSplit((family.ana,)),
        split=WholeExpenseSplit(
            PercentagesSplit((MemberPercentage(family.ana, Percentage.parse('60')), MemberPercentage(family.bob, Percentage.parse('30'))))
        ),
        date=family.day,
    )
    refused = refusal(lambda: group.add_expense(invalid))
    assert refused is not None, 'the split must be refused'
    print_lines(f'  error: {refused}')


def print_totals(group: ExpenseGroup) -> None:
    """
    Print what each member paid and consumed over the holiday, and where they stand.

    Args:
        group: The group.

    """
    section('7. The holiday in total')
    summary = group.summary()
    print_lines(f'{summary.total_spent} spent over five expenses:')
    print_table(
        TotalRow,
        [
            TotalRow(name=member.first_name, paid=found.paid, consumed=found.share, balance=str(found.balance))
            for member in group.members()
            for found in (summary.member(member.id),)
        ],
        1,
    )


def main() -> None:
    """Print the report, checking its key numbers."""
    group = ExpenseGroup.create('Family holiday', CurrencyCode('EUR'))
    family = Family(
        ana=group.add_member(Member.create(first_name='Ana')),
        bob=group.add_member(Member.create(first_name='Bob')),
        mia=group.add_member(Member.create(first_name='Mia')),
        day=CalendarDate(2026, 7, 14),
    )

    title('Family holiday: every way to split an expense')
    print_lines(
        'Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a',
        'different split method; whatever the method, the shares add up to exactly the',
        'amount, to the cent.',
    )
    equally(group, family)
    by_percentage(group, family)
    food = by_parts(group, family)
    by_amounts(group, family)
    several_payers(group, family)
    mistakes(group, family)
    print_totals(group)

    # Parts: 125.00 over 2.5 parts is 50.00 per part.
    shares = group.expense_shares(food)
    assert shares is not None, 'food was added'
    assert next((share.share for share in shares.members if share.member == family.mia), None) == euros('25.00')


if __name__ == '__main__':
    main()
