"""
Print a weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom.

The port of the Rust example ``quickstart``, which prints the same report.

"""

import dataclasses
from typing import ClassVar, final, override

from eranos import (
    Balance,
    CalendarDate,
    CurrencyCode,
    EqualSplit,
    Expense,
    GetsBackBalance,
    ExpenseGroup,
    LocalCost,
    Member,
    Money,
    SettlementStrategy,
    WholeExpenseSplit,
)
from examples.report import Row, names, print_lines, print_table, section, title


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ExpenseRow(Row):
    """A row of the table of expenses."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Expense', 'Paid by', 'Shared by', 'Amount')

    name: str
    """What the expense was."""

    payers: str
    """Who paid it."""

    participants: str
    """Who share it."""

    amount: Money
    """What the expense cost."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.payers, self.participants, str(self.amount)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class MemberRow(Row):
    """A row of the table of members' totals."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Paid', 'Consumed', 'Balance')

    name: str
    """The member's first name."""

    paid: Money
    """What the member paid."""

    consumed: Money
    """What the member consumed."""

    balance: Balance
    """Where the member stands."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, str(self.paid), str(self.consumed), str(self.balance)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class PaymentRow(Row):
    """A row of the table of payments that settle the group."""

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


def main() -> None:
    """Print the report, checking its key numbers."""
    euro = CurrencyCode('EUR')
    group = ExpenseGroup.create('Weekend in Sinaia', euro)
    title(group.name())
    print_lines(
        f'Three friends rent a cabin and buy groceries. The group keeps its books in {group.currency()}, so',
        f'every total, balance, and payment below is in {group.currency()}.',
    )

    section('1. Members')
    ana = group.add_member(Member.create(first_name='Ana'))
    bob = group.add_member(Member.create(first_name='Bob'))
    carol = group.add_member(Member.create(first_name='Carol'))
    print_lines(f'{len(group.members())} people joined the group: {names(group, (ana, bob, carol))}.')

    section('2. Expenses')
    print_lines('Each expense records who paid and who shares it. Here everyone shares everything', 'equally.')
    friday = CalendarDate(2026, 9, 25)
    everyone = WholeExpenseSplit(EqualSplit((ana, bob, carol)))
    group.add_expense(
        Expense.create(name='Cabin', cost=LocalCost(Money.parse('240.00', euro)), payers=EqualSplit((ana,)), split=everyone, date=friday)
    )
    group.add_expense(
        Expense.create(name='Groceries', cost=LocalCost(Money.parse('60.00', euro)), payers=EqualSplit((bob,)), split=everyone, date=friday)
    )
    print_table(
        ExpenseRow,
        [
            ExpenseRow(
                name=expense.name,
                payers=names(group, expense.payers.members),
                participants=names(group, expense.split.participants()),
                amount=expense.total(),
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
            MemberRow(name=member.first_name, paid=found.paid, consumed=found.share, balance=found.balance)
            for member in group.members()
            for found in (summary.member(member.id),)
        ],
        1,
    )
    ana_totals = summary.member(ana)
    print_lines(f'Ana paid {ana_totals.paid} but consumed only {ana_totals.share}, so the others owe her the difference.')

    section('4. Settling up')
    settlement = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS)
    print_lines(f'The fewest payments that bring every balance to zero ({len(settlement.payments)} in total):')
    print_table(
        PaymentRow,
        [
            PaymentRow(sender=names(group, (payment.sender,)), receiver=names(group, (payment.receiver,)), amount=payment.amount)
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
