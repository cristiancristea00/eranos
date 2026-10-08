"""
Settle up with each strategy on a group where they differ, then record the chosen plan as transfers until everyone is settled.

The port of the Rust example ``settling_up``, which prints the same report.

"""

import dataclasses
from typing import ClassVar, assert_never, final, override

from eranos import (
    Balance,
    CalendarDate,
    CurrencyCode,
    EqualSplit,
    Expense,
    ExpenseGroup,
    LocalCost,
    Member,
    MemberId,
    Money,
    SettledBalance,
    SettlementPlan,
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

    name: str
    """What the expense was."""

    payer: str
    """Who paid it."""

    participants: str
    """Who share it."""

    amount: Money
    """What the expense cost."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.payer, self.participants, str(self.amount)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class BalanceRow(Row):
    """A row of the table of balances: what a member paid and consumed, and where they stand."""

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


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class StrategyRow(Row):
    """A row of the table of the strategies side by side."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Strategy', 'Guarantee', 'Cost', 'Payments here')

    strategy: str
    """The strategy's name."""

    guarantee: str
    """What the strategy promises."""

    cost: str
    """How the strategy's time grows with the open balances."""

    payments: int
    """The number of payments of the strategy's plan for the group."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.strategy, self.guarantee, self.cost, str(self.payments)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Counts:
    """The number of payments of each strategy's plan."""

    adaptive: int
    """The payments of the adaptive plan."""

    minimum: int
    """The payments of the minimum plan."""

    greedy: int
    """The payments of the greedy plan."""

    direct: int
    """The payments of the direct plan."""


def strategy_name(strategy: SettlementStrategy) -> str:
    """
    Name strategy as the apps show it.

    Args:
        strategy: The strategy.

    Returns:
        The name, such as ``"MinimumTransfers"``.

    """
    match strategy:
        case SettlementStrategy.ADAPTIVE:
            return 'Adaptive'
        case SettlementStrategy.MINIMUM_TRANSFERS:
            return 'MinimumTransfers'
        case SettlementStrategy.GREEDY:
            return 'Greedy'
        case SettlementStrategy.DIRECT:
            return 'Direct'
        case _:
            assert_never(strategy)


def spend(group: ExpenseGroup, name: str, amount: str, payer: MemberId, members: tuple[MemberId, ...]) -> None:
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
        Expense.create(
            name=name,
            cost=LocalCost(Money.parse(amount, CurrencyCode('EUR'))),
            payers=EqualSplit((payer,)),
            split=WholeExpenseSplit(EqualSplit(members)),
            date=CalendarDate(2026, 9, 20),
        )
    )


def print_balances(group: ExpenseGroup) -> None:
    """
    Print what each member of group paid and consumed, and their balance.

    Args:
        group: The group.

    """
    summary = group.summary()
    print_table(
        BalanceRow,
        [
            BalanceRow(name=member.first_name, paid=found.paid, consumed=found.share, balance=found.balance)
            for member in group.members()
            if (found := totals(member.id, summary)) is not None
        ],
        1,
    )


def print_payments(group: ExpenseGroup, settlement: SettlementPlan) -> None:
    """
    Print who pays whom in settlement, and how much, by the first names of the members of group.

    Args:
        group: The group that settlement settles.
        settlement: The settlement.

    """
    print_table(
        PaymentRow,
        [
            PaymentRow(sender=names(group, (payment.sender,)), receiver=names(group, (payment.receiver,)), amount=payment.amount)
            for payment in settlement.payments
        ],
        2,
    )


def strategies(group: ExpenseGroup) -> Counts:
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

    section('6. Adaptive')
    adaptive = group.settlement(SettlementStrategy.ADAPTIVE)
    print_lines(
        "The fewest payments while they are quick to find, greedy's beyond: up to 20 open",
        'balances the minimum solver settles them, past that greedy does, and the plan',
        'names the strategy whose plan it is. Six balances are well within the limit, so',
        f'the plan is the one {strategy_name(adaptive.strategy)} gives, the same {len(adaptive.payments)} payments.',
    )

    section('7. The strategies side by side')
    print_table(
        StrategyRow,
        [
            StrategyRow(
                strategy='MinimumTransfers',
                guarantee='the fewest payments',
                cost='O(2ⁿ·n) for n open balances',
                payments=len(minimum.payments),
            ),
            StrategyRow(
                strategy='Greedy',
                # The minus sign goes by its name: Ruff refuses the character itself, which reads as a hyphen.
                guarantee='at most n \N{MINUS SIGN} 1 payments',
                cost='O(n log n)',
                payments=len(greedy.payments),
            ),
            StrategyRow(
                strategy='Direct',
                guarantee='one payment per pair in debt',
                cost='O(pairs)',
                payments=len(direct.payments),
            ),
            StrategyRow(
                strategy='Adaptive',
                guarantee='the fewest up to 20 balances',
                cost='as MinimumTransfers, then Greedy',
                payments=len(adaptive.payments),
            ),
        ],
        3,
    )
    return Counts(adaptive=len(adaptive.payments), minimum=len(minimum.payments), greedy=len(greedy.payments), direct=len(direct.payments))


def festival() -> int:
    """
    Settle a group of 21 friends, more open balances than the adaptive strategy settles exactly, and print which strategy's plan it took.

    Returns:
        The number of payments.

    """
    section('8. A larger group')
    euro = CurrencyCode('EUR')
    group = ExpenseGroup.create('Festival', euro)
    friends = tuple(group.add_member(Member.create(first_name=f'Friend {number}')) for number in range(1, 22))
    # Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's balance is 21·i² - 3311
    # cents: never zero, and never the opposite of another's.
    for number, payer in enumerate(friends, start=1):
        group.add_expense(
            Expense.create(
                name='Round',
                cost=LocalCost(Money(21 * number**2, euro)),
                payers=EqualSplit((payer,)),
                split=WholeExpenseSplit(EqualSplit(friends)),
                date=CalendarDate(2026, 9, 18),
            )
        )
    open_balances = sum(1 for member in group.summary().members if member.net.minor_units != 0)
    adaptive = group.settlement(SettlementStrategy.ADAPTIVE)
    greedy = group.settlement(SettlementStrategy.GREEDY)
    print_lines(
        'Twenty-one friends at a festival each bought one round for everyone, each round',
        f'at another price, so all {open_balances} end with a balance and no two of them cancel out.',
        'That is more than the 20 the adaptive strategy settles exactly, so it takes the',
        f'plan {strategy_name(adaptive.strategy)} gives and says so: the same {len(adaptive.payments)} payments.',
    )
    assert open_balances == 21
    assert adaptive.strategy is SettlementStrategy.GREEDY
    assert adaptive.payments == greedy.payments
    return len(adaptive.payments)


def main() -> None:
    """Print the report, checking its key numbers."""
    group = ExpenseGroup.create('Ski week', CurrencyCode('EUR'))
    ana = group.add_member(Member.create(first_name='Ana'))
    bob = group.add_member(Member.create(first_name='Bob'))
    carol = group.add_member(Member.create(first_name='Carol'))
    dan = group.add_member(Member.create(first_name='Dan'))
    eva = group.add_member(Member.create(first_name='Eva'))
    filip = group.add_member(Member.create(first_name='Filip'))

    title('Ski week: four ways to settle up')
    print_lines(
        'Six friends spent a week in the mountains. The library can turn their balances',
        'into payments in four ways, and the app picks one.',
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
                name=expense.name,
                payer=names(group, expense.payers.members),
                participants=names(group, participants(expense.split)),
                amount=expense.total(),
            )
            for expense in group.expenses()
        ],
        3,
    )

    section('2. Balances')
    print_lines('Four members are owed money and two owe it:')
    print_balances(group)

    counts = strategies(group)
    festival_payments = festival()

    section('9. Settling up for real')
    plan = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS)
    for payment in plan.payments:
        # A settlement's payments are in the group currency, so the transfer that records one has a local cost.
        group.add_transfer(
            Transfer.create(sender=payment.sender, receiver=payment.receiver, cost=LocalCost(payment.amount), date=CalendarDate(2026, 9, 27))
        )
    print_lines(
        'Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards',
        'every balance is zero and a new settlement has nothing left to pay:',
    )
    print_balances(group)
    print_lines(f'Payments still needed: {len(group.settlement(SettlementStrategy.MINIMUM_TRANSFERS).payments)}.')

    # {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice, and the adaptive plan is that one; greedy pays
    # the largest amounts first and needs five; the unsimplified debts between pairs need seven.
    assert counts == Counts(adaptive=4, minimum=4, greedy=5, direct=7)
    # Greedy settles the festival's 21 balances in 19 payments.
    assert festival_payments == 19
    summary = group.summary()
    for member in (ana, bob, carol, dan, eva, filip):
        found = totals(member, summary)
        assert found is not None
        assert found.balance == SettledBalance()


if __name__ == '__main__':
    main()
