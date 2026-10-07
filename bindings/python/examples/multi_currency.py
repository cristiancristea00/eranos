"""
Print a trip across currencies: a group in EUR, expenses and a cash repayment in RON and HUF at fixed rates, and its books moved to RON.

The port of the Rust example ``multi_currency``, which prints the same report.

"""

import dataclasses
from typing import ClassVar, assert_never, final, override

from eranos import (
    Balance,
    CalendarDate,
    ConvertedCost,
    Cost,
    CurrencyCode,
    CurrencyRate,
    EqualSplit,
    ExchangeRate,
    Expense,
    GetsBackBalance,
    ExpenseGroup,
    LocalCost,
    Member,
    MemberId,
    Money,
    SettledBalance,
    Transfer,
    WholeExpenseSplit,
)
from examples.report import Row, names, print_lines, print_table, section, title, totals


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


def rate(cost: Cost) -> ExchangeRate:
    """
    Find the rate cost was entered at: 1 for a cost in the group currency.

    Args:
        cost: What an entry cost.

    Returns:
        The rate into the group currency.

    """
    match cost:
        case LocalCost():
            return ExchangeRate('1')
        case ConvertedCost(rate=converted):
            return converted
        case _:
            assert_never(cost)


def balance(group: ExpenseGroup, member: MemberId) -> Balance | None:
    """
    Find where member stands in group.

    Args:
        group: The group.
        member: The member.

    Returns:
        The member's balance, or none for a member the summary leaves out.

    """
    found = totals(member, group.summary())
    return None if found is None else found.balance


def print_entries(group: ExpenseGroup) -> None:
    """
    Print every expense and transfer, as entered and in the group currency.

    Args:
        group: The group.

    """
    zero = Money(0, group.currency())
    expenses = [
        EntryRow(
            name=expense.name,
            payer=names(group, expense.payers.members),
            original=expense.total(),
            rate=rate(expense.cost),
            converted=shares.total if (shares := group.expense_shares(expense.id)) is not None else zero,
        )
        for expense in group.expenses()
    ]
    transfers = [
        EntryRow(
            name=f'Transfer to {names(group, (transfer.receiver,))}',
            payer=names(group, (transfer.sender,)),
            original=transfer.cost.amount,
            rate=rate(transfer.cost),
            converted=amount if (amount := group.transfer_amount(transfer.id)) is not None else zero,
        )
        for transfer in group.transfers()
    ]
    print_table(EntryRow, [*expenses, *transfers], 2)


def print_balances(group: ExpenseGroup) -> None:
    """
    Print what each member paid, consumed, sent, and received, and where they stand.

    Args:
        group: The group.

    """
    summary = group.summary()
    print_table(
        BalanceRow,
        [
            BalanceRow(
                name=member.first_name,
                paid=found.paid,
                consumed=found.share,
                sent=found.sent,
                received=found.received,
                balance=found.balance,
            )
            for member in group.members()
            if (found := totals(member.id, summary)) is not None
        ],
        1,
    )


def expenses(group: ExpenseGroup, ana: MemberId, bob: MemberId) -> None:
    """
    Add the fuel in RON, the dinner in HUF, and the ferry in EUR, and print them.

    Args:
        group: The group, whose books are in EUR.
        ana: The member who pays for the fuel.
        bob: The member who pays for the dinner and the ferry.

    """
    section('1. Expenses in three currencies')
    both = WholeExpenseSplit(EqualSplit((ana, bob)))
    group.add_expense(
        Expense.create(
            name='Fuel',
            cost=ConvertedCost(Money.parse('400.00', CurrencyCode('RON')), ExchangeRate('0.2')),
            payers=EqualSplit((ana,)),
            split=both,
            date=CalendarDate(2026, 8, 1),
        )
    )
    group.add_expense(
        Expense.create(
            name='Dinner',
            cost=ConvertedCost(Money.parse('24000.00', CurrencyCode('HUF')), ExchangeRate('0.0025')),
            payers=EqualSplit((bob,)),
            split=both,
            date=CalendarDate(2026, 8, 2),
        )
    )
    group.add_expense(
        Expense.create(
            name='Ferry',
            cost=LocalCost(Money.parse('60.00', CurrencyCode('EUR'))),
            payers=EqualSplit((bob,)),
            split=both,
            date=CalendarDate(2026, 8, 3),
        )
    )
    print_entries(group)
    print_lines(
        f"The rate says how many {group.currency()} one unit of the entry's currency is worth: 400.00 RON at",
        '0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.',
    )


def repayment(group: ExpenseGroup, ana: MemberId, bob: MemberId) -> None:
    """
    Add Ana's repayment to Bob, in cash and in RON, and print the balances it settles.

    Args:
        group: The group, whose books are in EUR.
        ana: The member who pays back.
        bob: The member who is paid back.

    """
    section('3. A repayment in cash, in another currency')
    cash = Transfer.create(
        sender=ana,
        receiver=bob,
        cost=ConvertedCost(Money.parse('100.00', CurrencyCode('RON')), ExchangeRate('0.2')),
        date=CalendarDate(2026, 8, 4),
        note='Cash at the border',
    )
    group.add_transfer(cash)
    print_lines(
        'Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.',
        f'Transfers move balances but are not spending, so the total stays at {group.summary().total_spent}:',
    )
    print_balances(group)


def new_currency(group: ExpenseGroup) -> None:
    """
    Move the group's books from EUR to RON, and print the entries and the balances again.

    Args:
        group: The group, whose books are in EUR.

    """
    section('4. Moving the books to another currency')
    print_lines(
        'Changing the group currency needs a rate into the new currency for every other',
        'currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies',
        'take the new rate; entries already in RON become local costs, at 1.',
    )
    group.change_currency(
        CurrencyCode('RON'),
        (CurrencyRate(CurrencyCode('EUR'), ExchangeRate('5')), CurrencyRate(CurrencyCode('HUF'), ExchangeRate('0.0125'))),
    )
    print_entries(group)
    print_lines(f'In {group.currency()}, the trip cost {group.summary().total_spent}, and everyone is still settled:')
    print_balances(group)


def main() -> None:
    """Print the report, checking its key numbers."""
    euro = CurrencyCode('EUR')
    group = ExpenseGroup.create('Danube road trip', euro)
    ana = group.add_member(Member.create(first_name='Ana'))
    bob = group.add_member(Member.create(first_name='Bob'))

    title(f'{group.name()}: expenses in several currencies')
    print_lines(
        f'Ana and Bob drive from Romania to Hungary. The group keeps its books in {group.currency()}; an',
        'expense in another currency carries the exchange rate used when it was entered,',
        'and is converted once, rounding half away from zero to the cent.',
    )
    expenses(group, ana, bob)

    section('2. Balances')
    print_lines(
        f'The group spent {group.summary().total_spent}. Bob paid 120.00 of it but consumed half, so he gets back the',
        'difference:',
    )
    print_balances(group)
    assert balance(group, bob) == GetsBackBalance(Money.parse('20.00', euro))

    repayment(group, ana, bob)
    assert balance(group, bob) == SettledBalance()

    new_currency(group)
    # 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
    assert group.summary().total_spent == Money.parse('1000.00', CurrencyCode('RON'))
    assert balance(group, bob) == SettledBalance()


if __name__ == '__main__':
    main()
