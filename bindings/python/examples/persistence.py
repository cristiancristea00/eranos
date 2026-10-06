"""
Save a group as JSON and load it back.

The saved form holds the entries only; loading replays them through the group's checks and recomputes every total.

The port of the Rust example ``persistence``, which prints the same report.

"""

import dataclasses
from typing import ClassVar, final, override

from eranos import (
    CalendarDate,
    Currency,
    EqualSplit,
    Expense,
    Group,
    LocalCost,
    Member,
    MemberId,
    Money,
    Percentage,
    PercentageTip,
    SettlementStrategy,
    Summary,
    Tip,
    TipDistribution,
    WholeExpenseSplit,
)
from examples.report import Row, names, print_lines, print_table, refusal, section, title, totals


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ComparisonRow(Row):
    """A row of the table that compares a member's balance before saving and after loading."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Balance before saving', 'Balance after loading')

    after: str
    """The member's balance in the loaded group."""

    before: str
    """The member's balance in the saved group."""

    name: str
    """The member's first name."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.before, self.after


def balance(member: MemberId, summary: Summary) -> str:
    """
    Write the balance of member in summary as the library does.

    Args:
        member: The member.
        summary: The group's summary.

    Returns:
        The balance, such as ``"owes 18.50 EUR"``, or ``?`` for a member the summary leaves out.

    """
    found = totals(member, summary)
    return '?' if found is None else str(found.balance)


def main() -> None:
    """Print the report, checking its key numbers."""
    euro = Currency('EUR')
    day = CalendarDate(23, 9, 2026)
    group = Group.create(euro, 'Road trip')
    ana = group.add_member(Member(first_name='Ana'))
    bob = group.add_member(Member(first_name='Bob'))
    group.add_expense(
        Expense(
            cost=LocalCost(Money.parse('70.00', euro)),
            date=day,
            name='Fuel',
            payers=EqualSplit((ana,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob))),
        )
    )
    group.add_expense(
        Expense(
            cost=LocalCost(Money.parse('30.00', euro)),
            date=day,
            name='Lunch',
            payers=EqualSplit((bob,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob))),
            tip=Tip(distribution=TipDistribution.PROPORTIONAL, value=PercentageTip(Percentage.parse('10'))),
        )
    )

    title('Road trip: saving and loading a group')
    print_lines(
        'With the `serde` feature a group serialises, and so does every type it stores.',
        'It saves only what was entered: its details, members, categories, expenses,',
        'and transfers. The totals, balances, and settlements are recomputed when it is',
        'loaded.',
    )

    section('1. The saved form')
    # Account details would be in clear text: store this in the platform's secure storage (Keychain, Keystore, or an encrypted database).
    saved = group.save()
    print_lines(saved)
    print_lines(
        'Amounts are minor units with their currency, dates are YYYY-MM-DD, and the',
        '`format` number lets a future release refuse a file it cannot read instead of',
        'dropping fields.',
    )

    section('2. Loading it back')
    loaded = Group.load(saved)
    print_lines(
        'Loading replays every entry through the same checks as adding it, then rebuilds',
        'the totals. The loaded group equals the saved one, and so do its balances:',
    )
    before = group.summary()
    after = loaded.summary()
    print_table(
        ComparisonRow,
        [ComparisonRow(after=balance(member, after), before=balance(member, before), name=names(group, (member,))) for member in (ana, bob)],
        1,
    )
    plan = loaded.settlement(SettlementStrategy.GREEDY)
    print_lines(f'Payments needed to settle the loaded group: {len(plan.payments)}.')

    section('3. A corrupted file is refused')
    print_lines(
        "Here the fuel's payer was edited to an identifier no member has. Reading the file",
        'as a SavedGroup and loading it with into_group replays every entry through the',
        'same checks as adding it, and refuses the file instead of producing balances that',
        'do not add up:',
    )
    # The fuel is the first entry that names Ana: the expenses come before the members in the saved form.
    corrupted = saved.replace(ana, '0199a000-0000-7000-8000-00000000ffff', count=1)
    refused = refusal(lambda: Group.load(corrupted))
    assert refused is not None, 'the file must be refused'
    print_lines(f'  error: {refused}')

    # A group is a handle, which compares by identity: the loaded one equals the saved one when it saves the same text.
    assert loaded.save() == saved
    assert after == before
    assert len(plan.payments) == 1


if __name__ == '__main__':
    main()
