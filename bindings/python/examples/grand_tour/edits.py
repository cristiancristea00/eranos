"""Print sections 5 to 7 of the tour: entries in another currency, edits and refused changes, and members who come and go."""

import copy
import dataclasses
from typing import ClassVar, final, override

from eranos import (
    Balance,
    ConvertedCost,
    CurrencyCode,
    EqualSplit,
    ExchangeRate,
    Expense,
    ExpenseError,
    ExpenseId,
    ExpenseGroup,
    ExpenseGroupError,
    LocalCost,
    MemberPercentage,
    Money,
    Percentage,
    PercentagesSplit,
    TimeOfDay,
    Transfer,
    TransferError,
    WholeExpenseSplit,
)
from examples.grand_tour.tour import Categories, People, RefusalRow, eur, name, october, print_balances, print_entries, ron
from examples.report import Row, message, print_lines, print_table, refusal, section, status


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class EditRow(Row):
    """A row of the table of edits: an entry, and what changed."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Entry', 'Edit')

    entry: str
    """The entry edited or removed."""

    edit: str
    """What changed."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.entry, self.edit


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class StatusRow(Row):
    """A row of the table of members' statuses: whether a member is active or archived, and where they stand."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Status', 'Balance')

    name: str
    """The member's first name."""

    status: str
    """Whether the member is active or archived."""

    balance: Balance
    """Where the member stands."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.status, str(self.balance)


def another_currency(group: ExpenseGroup, people: People, categories: Categories) -> None:
    """
    Print section 5: add an expense and a transfer in lei, each with its rate into the group currency.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.

    """
    section('5. Another currency')
    euro = CurrencyCode('EUR')
    rate = ExchangeRate('0.2')
    parking = group.add_expense(
        Expense.create(
            name='Airport parking',
            cost=ConvertedCost(ron('250.00'), rate),
            payers=EqualSplit((people.ana,)),
            split=WholeExpenseSplit(EqualSplit((people.ana, people.bob))),
            date=october(6),
            category=categories.transport,
        )
    )
    cash = group.add_transfer(
        Transfer.create(
            sender=people.bob,
            receiver=people.ana,
            cost=ConvertedCost(ron('100.00'), rate),
            date=october(6),
            time=TimeOfDay(7, 40),
            note='Cash for the parking',
        )
    )
    print_lines(
        'Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in',
        'cash. An entry in another currency carries the rate into the group currency, how',
        'many EUR one RON is worth, and is converted once, when it is entered:',
    )
    print_entries(group, lambda currency: currency != euro)
    transfer = group.transfer(cash)
    assert transfer is not None, 'the cash was recorded'
    counted = group.transfer_amount(cash) or Money.zero(euro)
    converted = ExchangeRate('0.2011').convert(ron('250.00'), euro)
    print_lines(
        f'Bob\'s transfer, at {transfer.time or ""} ("{transfer.note or ""}"), counts as {counted}.',
        'A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,',
        f'which becomes {converted}.',
    )

    assert converted == eur('50.28')
    shares = group.expense_shares(parking)
    assert shares is not None
    assert shares.total == eur('50.00')
    assert group.transfer_amount(cash) == eur('20.00')
    assert isinstance(transfer.cost, ConvertedCost)


def _replace_and_remove(group: ExpenseGroup, people: People, categories: Categories, seafood: ExpenseId) -> list[EditRow]:
    """
    Replace an expense and a transfer, and remove one of each.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.
        seafood: The seafood dinner, which the edit corrects.

    Returns:
        What changed: a row per entry, with the edit.

    """
    # The dinner's receipt said 99.00.
    dinner = group.expense(seafood)
    assert dinner is not None, 'the dinner was added'
    previous = group.replace_expense(copy.replace(dinner, cost=LocalCost(eur('99.00')), note='Cervejaria Ramiro'))

    # Bob entered the coffee twice, so each entry has an identifier of its own.
    def coffee() -> Expense:
        return Expense.create(
            name='Coffee',
            cost=LocalCost(eur('6.00')),
            payers=EqualSplit((people.bob,)),
            split=WholeExpenseSplit(EqualSplit((people.ana, people.bob, people.carol))),
            date=october(5),
            category=categories.food,
        )

    group.add_expense(coffee())
    duplicate = group.add_expense(coffee())
    removed = group.remove_expense(duplicate)
    # Dan's part of the apartment was 120.00, not 100.00.
    first = group.add_transfer(Transfer.create(sender=people.dan, receiver=people.ana, cost=LocalCost(eur('100.00')), date=october(5)))
    stored = group.transfer(first)
    assert stored is not None, 'the transfer was added'
    replaced = group.replace_transfer(copy.replace(stored, cost=LocalCost(eur('120.00')), note='Apartment'))
    # Carol sent 10.00 to the wrong member.
    mistake = group.add_transfer(Transfer.create(sender=people.carol, receiver=people.bob, cost=LocalCost(eur('10.00')), date=october(5)))
    undone = group.remove_transfer(mistake)

    assert previous.cost.amount == eur('90.00')
    assert group.expense(duplicate) is None
    assert group.transfer_amount(first) == eur('120.00')
    assert group.transfer(mistake) is None
    corrected = group.expense(seafood)
    assert corrected is not None
    apartment = group.transfer(first)
    assert apartment is not None
    return [
        EditRow(entry=previous.name, edit=f'{previous.cost.amount} became {corrected.cost.amount}, at {corrected.note or ""}'),
        EditRow(entry=removed.name, edit=f'entered twice; the copy of {removed.cost.amount} was removed'),
        EditRow(
            entry=f'{name(group, people.dan)} to {name(group, people.ana)}',
            edit=f'{replaced.cost.amount} became {apartment.cost.amount}, noted {apartment.note or ""}',
        ),
        EditRow(entry=f'{name(group, people.carol)} to {name(group, people.bob)}', edit=f'{undone.cost.amount} sent to the wrong member; removed'),
    ]


def edits(group: ExpenseGroup, people: People, categories: Categories, seafood: ExpenseId) -> None:
    """
    Print section 6: edit and remove entries, then show the changes the group refuses, which leave it as it was.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.
        seafood: The seafood dinner, which the edit corrects.

    """
    section('6. Edits')
    changes = _replace_and_remove(group, people, categories, seafood)
    print_lines(
        'Entries are values: an edit starts from a copy made with to_builder, keeps the',
        'identifier, and replaces the entry; a removal takes it out with its totals.',
    )
    print_table(EditRow, changes, 2)
    print_lines('The balances after the edits:')
    print_balances(group)

    before = group.summary()
    taxi = Expense.create(
        name='Taxi',
        cost=LocalCost(eur('30.00')),
        payers=EqualSplit((people.ana,)),
        split=WholeExpenseSplit(
            PercentagesSplit((MemberPercentage(people.ana, Percentage.parse('50')), MemberPercentage(people.bob, Percentage.parse('49.99'))))
        ),
        date=october(6),
    )
    unbalanced = refusal(lambda: group.add_expense(taxi))
    to_herself = refusal(
        lambda: group.add_transfer(Transfer.create(sender=people.ana, receiver=people.ana, cost=LocalCost(eur('5.00')), date=october(6)))
    )
    souvenirs = Expense.create(
        name='Souvenirs',
        cost=LocalCost(ron('40.00')),
        payers=EqualSplit((people.ana,)),
        split=WholeExpenseSplit(EqualSplit((people.ana,))),
        date=october(6),
    )
    unrated = refusal(lambda: group.add_expense(souvenirs))
    print_lines('A change that does not fit is refused, and the group stays as it was:')
    print_table(
        RefusalRow,
        [
            RefusalRow(change='A taxi split 50% and 49.99%', error=message(unbalanced)),
            RefusalRow(change='Ana paying herself back', error=message(to_herself)),
            RefusalRow(change='Souvenirs in RON, without a rate', error=message(unrated)),
        ],
        2,
    )
    print_lines('The balances again, unchanged:')
    print_balances(group)

    assert isinstance(unbalanced, ExpenseError)
    assert unbalanced.failure == ExpenseError.PercentagesDoNotAddUp(total_basis_points=9_999)
    assert isinstance(to_herself, TransferError)
    assert to_herself.failure == TransferError.SameMember()
    assert isinstance(unrated, ExpenseGroupError)
    assert unrated.failure == ExpenseGroupError.MissingExchangeRate(currency=CurrencyCode('RON'))
    assert group.summary() == before


def comings_and_goings(group: ExpenseGroup, people: People) -> None:
    """
    Print section 7: remove a member, archive another, and show what an archived member cannot do.

    Args:
        group: The group.
        people: The members of the trip.

    """
    section('7. Members come and go')
    removed = group.remove_member(people.filip)
    group.archive_member(people.dan)
    dinner = Expense.create(
        name='Farewell dinner',
        cost=LocalCost(eur('40.00')),
        payers=EqualSplit((people.ana,)),
        split=WholeExpenseSplit(EqualSplit((people.ana, people.bob, people.carol, people.dan))),
        date=october(6),
    )
    farewell = refusal(lambda: group.add_expense(dinner))
    in_use = refusal(lambda: group.remove_member(people.dan))
    print_lines(
        f'{removed.first_name} was never part of an entry, so removing him leaves no trace. Dan flew home',
        'early and was archived: his history and his balance stay, but a new expense',
        'cannot include him, and he cannot be removed while entries reference him.',
    )
    summary = group.summary()
    print_table(
        StatusRow,
        [
            StatusRow(name=member.first_name, status=status(group.is_archived(found.member)), balance=found.balance)
            for member in group.members()
            for found in (summary.member(member.id),)
        ],
        2,
    )
    print_table(
        RefusalRow,
        [
            RefusalRow(change='A farewell dinner with Dan', error=message(farewell)),
            RefusalRow(change='Removing Dan', error=message(in_use)),
        ],
        2,
    )
    group.unarchive_member(people.dan)
    print_lines('When Dan joins the next trip, unarchiving him makes him active again.')

    assert removed.first_name == 'Filip'
    assert group.member(people.filip) is None
    assert isinstance(farewell, ExpenseGroupError)
    assert farewell.failure == ExpenseGroupError.ArchivedMember(member=people.dan)
    assert isinstance(in_use, ExpenseGroupError)
    assert isinstance(in_use.failure, ExpenseGroupError.MemberInUse)
    assert group.is_archived(people.dan) is False
