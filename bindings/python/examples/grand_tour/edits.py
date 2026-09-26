"""Print sections 5 to 7 of the tour: entries in another currency, edits and refused changes, and members who come and go."""

import copy

from eranos import (
    ConvertedCost,
    Currency,
    EqualSplit,
    ExchangeRate,
    Expense,
    ExpenseError,
    ExpenseId,
    Group,
    GroupError,
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
from examples.grand_tour.tour import Categories, People, eur, name, october, print_balances, print_entries, ron
from examples.report import message, print_lines, print_table, refusal, section, totals


def another_currency(group: Group, people: People, categories: Categories) -> None:
    """
    Print section 5: add an expense and a transfer in lei, each with its rate into the group currency.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.

    """
    section('5. Another currency')
    euro = Currency('EUR')
    rate = ExchangeRate('0.2')
    parking = group.add_expense(
        Expense(
            cost=ConvertedCost(ron('250.00'), rate),
            date=october(6),
            name='Airport parking',
            payers=EqualSplit((people.ana,)),
            split=WholeExpenseSplit(EqualSplit((people.ana, people.bob))),
            category=categories.transport,
        )
    )
    cash = group.add_transfer(
        Transfer(
            cost=ConvertedCost(ron('100.00'), rate),
            date=october(6),
            receiver=people.ana,
            sender=people.bob,
            note='Cash for the parking',
            time=TimeOfDay(7, 40),
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
    counted = group.transfer_amount(cash) or Money(euro, 0)
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


def _replace_and_remove(group: Group, people: People, categories: Categories, seafood: ExpenseId) -> list[tuple[str, str]]:
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
    previous = group.replace_expense(copy.replace(dinner, cost=LocalCost(eur('99.00')), description='Cervejaria Ramiro'))
    # Bob entered the coffee twice.
    coffee = Expense(
        cost=LocalCost(eur('6.00')),
        date=october(5),
        name='Coffee',
        payers=EqualSplit((people.bob,)),
        split=WholeExpenseSplit(EqualSplit((people.ana, people.bob, people.carol))),
        category=categories.food,
    )
    group.add_expense(coffee)
    duplicate = group.add_expense(coffee)
    removed = group.remove_expense(duplicate)
    # Dan's part of the apartment was 120.00, not 100.00.
    first = group.add_transfer(Transfer(cost=LocalCost(eur('100.00')), date=october(5), receiver=people.ana, sender=people.dan))
    stored = group.transfer(first)
    assert stored is not None, 'the transfer was added'
    replaced = group.replace_transfer(copy.replace(stored, cost=LocalCost(eur('120.00')), note='Apartment'))
    # Carol sent 10.00 to the wrong member.
    mistake = group.add_transfer(Transfer(cost=LocalCost(eur('10.00')), date=october(5), receiver=people.bob, sender=people.carol))
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
        (previous.name, f'{previous.cost.amount} became {corrected.cost.amount}, at {corrected.description or ""}'),
        (removed.name, f'entered twice; the copy of {removed.cost.amount} was removed'),
        (
            f'{name(group, people.dan)} to {name(group, people.ana)}',
            f'{replaced.cost.amount} became {apartment.cost.amount}, noted {apartment.note or ""}',
        ),
        (f'{name(group, people.carol)} to {name(group, people.bob)}', f'{undone.cost.amount} sent to the wrong member; removed'),
    ]


def edits(group: Group, people: People, categories: Categories, seafood: ExpenseId) -> None:
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
    print_table(('Entry', 'Edit'), changes, 2)
    print_lines('The balances after the edits:')
    print_balances(group)

    before = group.summary()
    taxi = Expense(
        cost=LocalCost(eur('30.00')),
        date=october(6),
        name='Taxi',
        payers=EqualSplit((people.ana,)),
        split=WholeExpenseSplit(
            PercentagesSplit((MemberPercentage(people.ana, Percentage.parse('50')), MemberPercentage(people.bob, Percentage.parse('49.99'))))
        ),
    )
    unbalanced = refusal(lambda: group.add_expense(taxi))
    to_herself = refusal(lambda: group.add_transfer(Transfer(cost=LocalCost(eur('5.00')), date=october(6), receiver=people.ana, sender=people.ana)))
    souvenirs = Expense(
        cost=LocalCost(ron('40.00')),
        date=october(6),
        name='Souvenirs',
        payers=EqualSplit((people.ana,)),
        split=WholeExpenseSplit(EqualSplit((people.ana,))),
    )
    unrated = refusal(lambda: group.add_expense(souvenirs))
    print_lines('A change that does not fit is refused, and the group stays as it was:')
    print_table(
        ('Change', 'Refused with'),
        [
            ('A taxi split 50% and 49.99%', message(unbalanced)),
            ('Ana paying herself back', message(to_herself)),
            ('Souvenirs in RON, without a rate', message(unrated)),
        ],
        2,
    )
    print_lines('The balances again, unchanged:')
    print_balances(group)

    assert isinstance(unbalanced, ExpenseError)
    assert unbalanced.failure == ExpenseError.PercentagesDoNotAddUp(total_basis_points=9_999)
    assert isinstance(to_herself, TransferError)
    assert to_herself.failure == TransferError.SameMember()
    assert isinstance(unrated, GroupError)
    assert unrated.failure == GroupError.MissingExchangeRate(currency=Currency('RON'))
    assert group.summary() == before


def comings_and_goings(group: Group, people: People) -> None:
    """
    Print section 7: remove a member, archive another, and show what an archived member cannot do.

    Args:
        group: The group.
        people: The members of the trip.

    """
    section('7. Members come and go')
    removed = group.remove_member(people.filip)
    group.archive_member(people.dan)
    dinner = Expense(
        cost=LocalCost(eur('40.00')),
        date=october(6),
        name='Farewell dinner',
        payers=EqualSplit((people.ana,)),
        split=WholeExpenseSplit(EqualSplit((people.ana, people.bob, people.carol, people.dan))),
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
        ('Member', 'Status', 'Balance'),
        [
            (member.first_name, 'archived' if group.is_archived(found.member) else 'active', str(found.balance))
            for member in group.members()
            if member.id is not None and (found := totals(member.id, summary)) is not None
        ],
        2,
    )
    print_table(
        ('Change', 'Refused with'),
        [
            ('A farewell dinner with Dan', message(farewell)),
            ('Removing Dan', message(in_use)),
        ],
        2,
    )
    group.unarchive_member(people.dan)
    print_lines('When Dan joins the next trip, unarchiving him makes him active again.')

    assert removed.first_name == 'Filip'
    assert group.member(people.filip) is None
    assert isinstance(farewell, GroupError)
    assert farewell.failure == GroupError.ArchivedMember(member=people.dan)
    assert isinstance(in_use, GroupError)
    assert isinstance(in_use.failure, GroupError.MemberInUse)
    assert not group.is_archived(people.dan)
