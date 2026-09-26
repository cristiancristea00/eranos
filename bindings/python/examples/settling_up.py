"""
Settle up with each strategy on a group where they differ, then record the chosen plan as transfers until everyone is settled.

The port of the Rust example ``settling_up``, which prints the same report.

"""

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
    SettledBalance,
    Settlement,
    SettlementStrategy,
    Transfer,
    WholeExpenseSplit,
)
from examples.report import names, participants, print_lines, print_table, section, title, totals


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
        ('Member', 'Paid', 'Consumed', 'Balance'),
        [
            (member.first_name, str(found.paid), str(found.share), str(found.balance))
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
        ('From', 'To', 'Amount'),
        [(names(group, (payment.sender,)), names(group, (payment.receiver,)), str(payment.amount)) for payment in settlement.payments],
        2,
    )


def strategies(group: Group) -> tuple[int, int, int]:
    """
    Print each strategy's plan for group with what it promises.

    Args:
        group: The group.

    Returns:
        The number of payments of each plan, in the order minimum, greedy, direct.

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
        ('Strategy', 'Guarantee', 'Cost', 'Payments here'),
        [
            ('MinimumTransfers', 'the fewest payments', 'O(2ⁿ·n) for n open balances', str(len(minimum.payments))),
            # The minus sign goes by its name: Ruff refuses the character itself, which reads as a hyphen.
            ('Greedy', 'at most n \N{MINUS SIGN} 1 payments', 'O(n log n)', str(len(greedy.payments))),
            ('Direct', 'one payment per pair in debt', 'O(pairs)', str(len(direct.payments))),
        ],
        3,
    )
    return len(minimum.payments), len(greedy.payments), len(direct.payments)


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
        ('Expense', 'Paid by', 'Shared by', 'Amount'),
        [
            (expense.name, names(group, expense.payers.members), names(group, participants(expense.split)), str(expense.total()))
            for expense in group.expenses()
        ],
        3,
    )

    section('2. Balances')
    print_lines('Four members are owed money and two owe it:')
    print_balances(group)

    payments = strategies(group)

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
    assert payments == (4, 5, 7)
    summary = group.summary()
    for member in (ana, bob, carol, dan, eva, filip):
        found = totals(member, summary)
        assert found is not None
        assert found.balance == SettledBalance()


if __name__ == '__main__':
    main()
