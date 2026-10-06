"""
Settle up with each strategy on a group where they differ, then record the chosen plan as transfers until everyone is settled.

The port of the Rust example ``settling_up``, which prints the same report.

"""

import dataclasses
from typing import ClassVar, final, override

from eranos import (
    Balance,
    CalendarDate,
    Currency,
    EqualSplit,
    Expense,
    Group,
    LocalCost,
    Member,
    MemberId,
    Money,
    SettledBalance,
    Settlement,
    SettlementStrategy,
    Transfer,
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

    payer: str
    """Who paid it."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.payer, self.participants, str(self.amount)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class BalanceRow(Row):
    """A row of the table of balances: what a member paid and consumed, and where they stand."""

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
    """A row of the table of a plan's payments."""

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


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class StrategyRow(Row):
    """A row of the table of the strategies side by side."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Strategy', 'Guarantee', 'Cost', 'Payments here')

    cost: str
    """How the strategy's time grows with the open balances."""

    guarantee: str
    """What the strategy promises."""

    payments: int
    """The number of payments of the strategy's plan for the group."""

    strategy: str
    """The strategy's name."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.strategy, self.guarantee, self.cost, str(self.payments)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Counts:
    """The number of payments of each strategy's plan."""

    direct: int
    """The payments of the direct plan."""

    greedy: int
    """The payments of the greedy plan."""

    minimum: int
    """The payments of the minimum plan."""


def spend(group: Group, name: str, amount: str, payer: MemberId, members: tuple[MemberId, ...]) -> None:
    """
    Record an expense of amount EUR in group, called name, that payer paid and members share equally.

    Args:
        group: The group.
        name: What it was.
        amount: The amount's text, such as ``"12.00"``.
        payer: Who paid it.
        members: Who share it.

    """
    group.add_expense(
        Expense(
            cost=LocalCost(Money.parse(amount, Currency('EUR'))),
            date=CalendarDate(20, 9, 2026),
            name=name,
            payers=EqualSplit((payer,)),
            split=WholeExpenseSplit(EqualSplit(members)),
        )
    )


def print_balances(group: Group) -> None:
    """
    Print what each member of group paid and consumed, and their balance.

    Args:
        group: The group.

    """
    summary = group.summary()
    print_table(
        BalanceRow,
        [
            BalanceRow(balance=found.balance, consumed=found.share, name=member.first_name, paid=found.paid)
            for member in group.members()
            if member.id is not None and (found := totals(member.id, summary)) is not None
        ],
        1,
    )


def print_payments(group: Group, settlement: Settlement) -> None:
    """
    Print who pays whom in settlement, and how much, by the first names of the members of group.

    Args:
        group: The group that settlement settles.
        settlement: The settlement.

    """
    print_table(
        PaymentRow,
        [
            PaymentRow(amount=payment.amount, from_=names(group, (payment.sender,)), to=names(group, (payment.receiver,)))
            for payment in settlement.payments
        ],
        2,
    )


def strategies(group: Group) -> Counts:
    """
    Print each strategy's plan for group with what it promises.

    Args:
        group: The group.

    Returns:
        The number of payments of each plan.

    """
    section('3. Minimum transfers')
    minimum = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS)
    print_lines(
        'The fewest payments possible. Finding them is NP-hard; the solver does it exactly by',
        'splitting the balances into as many groups that sum to zero as it can: here',
        '{Ana, Dan, Eva} and {Bob, Carol, Filip}, each settled with one payment fewer than',
        f'its size. {len(minimum.payments)} payments:',
    )
    print_payments(group, minimum)

    section('4. Greedy')
    greedy = group.settlement(SettlementStrategy.GREEDY)
    print_lines(
        'The largest debtor pays the largest creditor, again and again: fast, at most one',
        f'payment fewer than the members with a balance, but not always the minimum. {len(greedy.payments)}',
        'payments here:',
    )
    print_payments(group, greedy)

    section('5. Direct')
    direct = group.settlement(SettlementStrategy.DIRECT)
    print_lines(
        'No simplification: everyone pays back the people who paid for them, with the debts',
        f'between each pair of members netted. The most payments ({len(direct.payments)}), but each one traces',
        'back to shared expenses:',
    )
    print_payments(group, direct)

    section('6. The strategies side by side')
    print_table(
        StrategyRow,
        [
            StrategyRow(
                cost='O(2ⁿ·n) for n open balances',
                guarantee='the fewest payments',
                payments=len(minimum.payments),
                strategy='MinimumTransfers',
            ),
            StrategyRow(
                cost='O(n log n)',
                # The minus sign goes by its name: Ruff refuses the character itself, which reads as a hyphen.
                guarantee='at most n \N{MINUS SIGN} 1 payments',
                payments=len(greedy.payments),
                strategy='Greedy',
            ),
            StrategyRow(
                cost='O(pairs)',
                guarantee='one payment per pair in debt',
                payments=len(direct.payments),
                strategy='Direct',
            ),
        ],
        3,
    )
    return Counts(direct=len(direct.payments), greedy=len(greedy.payments), minimum=len(minimum.payments))


def main() -> None:
    """Print the report, checking its key numbers."""
    group = Group.create(Currency('EUR'), 'Ski week')
    ana = group.add_member(Member(first_name='Ana'))
    bob = group.add_member(Member(first_name='Bob'))
    carol = group.add_member(Member(first_name='Carol'))
    dan = group.add_member(Member(first_name='Dan'))
    eva = group.add_member(Member(first_name='Eva'))
    filip = group.add_member(Member(first_name='Filip'))

    title('Ski week: three ways to settle up')
    print_lines(
        'Six friends spent a week in the mountains. The library can turn their balances',
        'into payments in three ways; the app picks one, and they differ here.',
    )

    section('1. What was spent')
    spend(group, 'Cabin', '12.00', dan, (dan, eva, filip))
    spend(group, 'Lift passes', '9.00', bob, (bob, carol, filip))
    spend(group, 'Groceries', '6.00', carol, (bob, dan))
    spend(group, 'Coffee', '2.00', ana, (dan, eva))
    spend(group, 'Snacks', '1.00', filip, (ana,))
    print_table(
        ExpenseRow,
        [
            ExpenseRow(
                amount=expense.total(),
                name=expense.name,
                participants=names(group, participants(expense.split)),
                payer=names(group, expense.payers.members),
            )
            for expense in group.expenses()
        ],
        3,
    )

    section('2. Balances')
    print_lines('Four members are owed money and two owe it:')
    print_balances(group)

    counts = strategies(group)

    section('7. Settling up for real')
    plan = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS)
    for payment in plan.payments:
        # A settlement's payments are in the group currency, so the transfer that records one has a local cost.
        group.add_transfer(Transfer(cost=LocalCost(payment.amount), date=CalendarDate(27, 9, 2026), receiver=payment.receiver, sender=payment.sender))
    print_lines(
        'Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards',
        'every balance is zero and a new settlement has nothing left to pay:',
    )
    print_balances(group)
    print_lines(f'Payments still needed: {len(group.settlement(SettlementStrategy.MINIMUM_TRANSFERS).payments)}.')

    # {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice; greedy pays the largest amounts first and needs
    # five; the unsimplified debts between pairs need seven.
    assert counts == Counts(direct=7, greedy=5, minimum=4)
    summary = group.summary()
    for member in (ana, bob, carol, dan, eva, filip):
        found = totals(member, summary)
        assert found is not None
        assert found.balance == SettledBalance()


if __name__ == '__main__':
    main()
