"""
Print a weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom.

The port of the Rust example ``quickstart``, which prints the same report.

"""

import dataclasses
from typing import ClassVar, final, override

from eranos import (
    Balance,
    CalendarDate,
    Currency,
    EqualSplit,
    Expense,
    GetsBackBalance,
    Group,
    LocalCost,
    Member,
    Money,
    SettlementStrategy,
    WholeExpenseSplit,
)
from examples.report import Row, names, participants, print_lines, print_table, section, title, totals


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ExpenseRow(Row):
    """A row of the table of expenses."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Expense', 'Paid by', 'Shared by', 'Amount')

    amount: Money
    """What the expense cost."""

    name: str
    """What the expense was."""

    participants: str
    """Who share it."""

    payers: str
    """Who paid it."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.payers, self.participants, str(self.amount)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class MemberRow(Row):
    """A row of the table of members' totals."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Paid', 'Consumed', 'Balance')

    balance: Balance
    """Where the member stands."""

    consumed: Money
    """What the member consumed."""

    name: str
    """The member's first name."""

    paid: Money
    """What the member paid."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, str(self.paid), str(self.consumed), str(self.balance)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class PaymentRow(Row):
    """A row of the table of payments that settle the group."""

    HEADERS: ClassVar[tuple[str, ...]] = ('From', 'To', 'Amount')

    amount: Money
    """What is paid."""

    from_: str
    """Who pays."""

    to: str
    """Who is paid."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.from_, self.to, str(self.amount)


def main() -> None:
    """Print the report, checking its key numbers."""
    euro = Currency('EUR')
    group = Group.create(euro, 'Weekend in Sinaia')
    title(group.name())
    print_lines(
        f'Three friends rent a cabin and buy groceries. The group keeps its books in {group.currency()}, so',
        f'every total, balance, and payment below is in {group.currency()}.',
    )

    section('1. Members')
    ana = group.add_member(Member(first_name='Ana'))
    bob = group.add_member(Member(first_name='Bob'))
    carol = group.add_member(Member(first_name='Carol'))
    print_lines(f'{len(group.members())} people joined the group: {names(group, (ana, bob, carol))}.')

    section('2. Expenses')
    print_lines('Each expense records who paid and who shares it. Here everyone shares everything', 'equally.')
    friday = CalendarDate(25, 9, 2026)
    everyone = WholeExpenseSplit(EqualSplit((ana, bob, carol)))
    group.add_expense(Expense(cost=LocalCost(Money.parse('240.00', euro)), date=friday, name='Cabin', payers=EqualSplit((ana,)), split=everyone))
    group.add_expense(Expense(cost=LocalCost(Money.parse('60.00', euro)), date=friday, name='Groceries', payers=EqualSplit((bob,)), split=everyone))
    print_table(
        ExpenseRow,
        [
            ExpenseRow(
                amount=expense.total(),
                name=expense.name,
                participants=names(group, participants(expense.split)),
                payers=names(group, expense.payers.members),
            )
            for expense in group.expenses()
        ],
        3,
    )

    section('3. Who paid and who consumed')
    summary = group.summary()
    print_lines(
        f"The group spent {summary.total_spent} in total. A member's share of an expense is what they consumed,",
        'and their balance is what they paid minus what they consumed.',
    )
    print_table(
        MemberRow,
        [
            MemberRow(balance=found.balance, consumed=found.share, name=member.first_name, paid=found.paid)
            for member in group.members()
            if member.id is not None and (found := totals(member.id, summary)) is not None
        ],
        1,
    )
    ana_totals = totals(ana, summary)
    assert ana_totals is not None
    print_lines(f'Ana paid {ana_totals.paid} but consumed only {ana_totals.share}, so the others owe her the difference.')

    section('4. Settling up')
    settlement = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS)
    print_lines(f'The fewest payments that bring every balance to zero ({len(settlement.payments)} in total):')
    print_table(
        PaymentRow,
        [
            PaymentRow(amount=payment.amount, from_=names(group, (payment.sender,)), to=names(group, (payment.receiver,)))
            for payment in settlement.payments
        ],
        2,
    )
    print_lines('Once these payments are made, nobody owes anything.')

    # Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
    assert summary.total_spent == Money.parse('300.00', euro)
    assert ana_totals.balance == GetsBackBalance(Money.parse('140.00', euro))
    assert len(settlement.payments) == 2


if __name__ == '__main__':
    main()
