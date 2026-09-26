"""Print section 4 of the tour: an expense for every way of dividing one, with each member's payment and consumption."""

from collections.abc import Sequence
from typing import assert_never

from eranos import (
    AmountsSplit,
    EqualSplit,
    Expense,
    ExpenseId,
    FixedTip,
    Group,
    Item,
    ItemisedExpenseSplit,
    LocalCost,
    MemberAmount,
    MemberId,
    MemberParts,
    MemberPercentage,
    MemberShare,
    Money,
    Parts,
    PartsSplit,
    Percentage,
    PercentagesSplit,
    PercentageTip,
    Split,
    TimeOfDay,
    Tip,
    TipDistribution,
    TipValue,
    WholeExpenseSplit,
)
from examples.grand_tour.tour import Categories, People, cell, eur, name, october
from examples.report import names, print_lines, print_table, section


def _divided(split: Split) -> str:
    """
    Tell how a whole expense's amount divides, in a word or two.

    Args:
        split: How the amount divides.

    Returns:
        The words, such as ``"by parts"``.

    """
    match split:
        case AmountsSplit():
            return 'by amounts'
        case EqualSplit():
            return 'equally'
        case PartsSplit():
            return 'by parts'
        case PercentagesSplit():
            return 'by percentage'
        case _:
            assert_never(split)


def _worth(value: TipValue) -> str:
    """
    Write what a tip is worth: its amount or its percentage.

    Args:
        value: What the tip is worth.

    Returns:
        The text, such as ``"10%"``.

    """
    match value:
        case FixedTip(amount=amount):
            return str(amount)
        case PercentageTip(percentage=percentage):
            return str(percentage)
        case _:
            assert_never(value)


def _shared(distribution: TipDistribution) -> str:
    """
    Tell how a tip is shared among an expense's participants.

    Args:
        distribution: How the tip divides.

    Returns:
        The words, such as ``"equally"``.

    """
    match distribution:
        case TipDistribution.EQUAL:
            return 'equally'
        case TipDistribution.PROPORTIONAL:
            return 'in proportion to what each member ordered'
        case _:
            assert_never(distribution)


def _how(expense: Expense) -> str:
    """
    Tell how expense divides, in a few words.

    Args:
        expense: The expense.

    Returns:
        The words, such as ``"equally, 2 payers"``.

    """
    split = expense.split
    match split:
        case ItemisedExpenseSplit(items=items):
            divided = f'{len(items)} items'
        case WholeExpenseSplit(split=whole):
            divided = _divided(whole)
        case _:
            assert_never(split)
    count = len(expense.payers.members)
    payers = '' if count == 1 else f', {count} payers'
    tip = expense.tip
    worth = '' if tip is None or expense.tip_amount().minor_units == 0 else f', {_worth(tip.value)} tip'
    return divided + payers + worth


def _share_row(group: Group, expense_id: ExpenseId) -> tuple[str, ...] | None:
    """
    Write the row of an expense among the shares: every involved member's payment and consumption.

    Args:
        group: The group.
        expense_id: The expense's identifier.

    Returns:
        The expense's name, how it divides, and its members, their payments, and their consumptions, a line each; or none
        for an expense the group does not hold.

    """
    expense = group.expense(expense_id)
    shares = group.expense_shares(expense_id)
    if expense is None or shares is None:
        return None
    return (
        expense.name,
        _how(expense),
        cell(name(group, share.member) for share in shares.members),
        cell('' if share.paid.minor_units == 0 else str(share.paid) for share in shares.members),
        cell(str(share.share) for share in shares.members),
    )


def _print_shares(group: Group, expenses: Sequence[ExpenseId]) -> None:
    """
    Print each of expenses, with every involved member's payment and consumption.

    Args:
        group: The group.
        expenses: The expenses' identifiers.

    """
    print_table(
        ('Expense', 'Split', 'Member', 'Paid', 'Consumed'),
        [row for expense_id in expenses if (row := _share_row(group, expense_id)) is not None],
        3,
    )


def _line_total(item: Item) -> Money:
    """
    Work out an item's line total, its unit price times its quantity, which the layer does not offer.

    Args:
        item: The line of the bill.

    Returns:
        The line total, in the expense's currency.

    """
    return Money(item.unit_price.currency, item.unit_price.minor_units * item.quantity)


def _print_items(group: Group, expense: Expense) -> None:
    """
    Print the lines of an itemised expense, and nothing for another.

    Args:
        group: The group.
        expense: The expense.

    """
    split = expense.split
    if not isinstance(split, ItemisedExpenseSplit):
        return
    print_table(
        ('Item', 'Shared by', 'Quantity', 'Unit price', 'Line total'),
        [(item.name, names(group, item.split.members), str(item.quantity), str(item.unit_price), str(_line_total(item))) for item in split.items],
        2,
    )


def _fado_items(people: People) -> tuple[Item, ...]:
    """
    Write the receipt of the fado night: every line with its own split.

    Args:
        people: The members of the trip.

    Returns:
        The lines of the bill.

    """
    one = Parts.parse('1')
    return (
        Item(name='Vinho verde', split=EqualSplit((people.ana, people.bob, people.dan)), unit_price=eur('12.00'), quantity=2),
        Item(name='Bacalhau', split=EqualSplit((people.carol,)), unit_price=eur('18.00')),
        Item(
            name='Custard tarts',
            split=PartsSplit(
                (
                    MemberParts(people.ana, one),
                    MemberParts(people.bob, one),
                    MemberParts(people.carol, one),
                    MemberParts(people.dan, one),
                    MemberParts(people.eva, Parts.parse('2')),
                )
            ),
            unit_price=eur('1.50'),
            quantity=6,
        ),
        Item(name='Juice', split=EqualSplit((people.eva,)), unit_price=eur('3.00')),
    )


def _every_way(people: People, categories: Categories) -> tuple[Expense, ...]:
    """
    Write one expense for every way of dividing one, in order.

    Args:
        people: The members of the trip.
        categories: The categories the group adds.

    Returns:
        The expenses, to add.

    """
    ana, bob, carol, dan, eva = people.ana, people.bob, people.carol, people.dan, people.eva
    one = Parts.parse('1')
    return (
        Expense(
            cost=LocalCost(eur('10.00')),
            date=october(1),
            name='Groceries',
            payers=EqualSplit((ana,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob, carol))),
            category=categories.food,
        ),
        Expense(
            cost=LocalCost(eur('800.00')),
            date=october(1),
            name='Apartment',
            payers=EqualSplit((ana,)),
            split=WholeExpenseSplit(
                PercentagesSplit(
                    (
                        MemberPercentage(ana, Percentage.parse('25')),
                        MemberPercentage(bob, Percentage.parse('25')),
                        MemberPercentage(carol, Percentage.parse('35')),
                        MemberPercentage(dan, Percentage.parse('15')),
                    )
                )
            ),
            category=categories.lodging,
        ),
        Expense(
            cost=LocalCost(eur('90.00')),
            date=october(2),
            name='Seafood dinner',
            payers=EqualSplit((carol,)),
            split=WholeExpenseSplit(
                PartsSplit(
                    (
                        MemberParts(ana, one),
                        MemberParts(bob, one),
                        MemberParts(carol, one),
                        MemberParts(dan, one),
                        MemberParts(eva, Parts.parse('0.5')),
                    )
                )
            ),
            category=categories.food,
        ),
        Expense(
            cost=LocalCost(eur('88.50')),
            date=october(3),
            name='Train to Porto',
            payers=EqualSplit((bob,)),
            split=WholeExpenseSplit(
                AmountsSplit(
                    (
                        MemberAmount(eur('24.50'), ana),
                        MemberAmount(eur('24.50'), bob),
                        MemberAmount(eur('24.50'), carol),
                        MemberAmount(eur('15.00'), eva),
                    )
                )
            ),
            category=categories.transport,
        ),
        Expense(
            cost=LocalCost(eur('60.00')),
            date=october(3),
            name='Museum',
            payers=EqualSplit((ana, bob)),
            split=WholeExpenseSplit(EqualSplit((ana, bob, carol, dan))),
        ),
        Expense(
            cost=LocalCost(eur('54.00')),
            date=october(4),
            name='Fado night',
            payers=AmountsSplit((MemberAmount(eur('39.40'), carol), MemberAmount(eur('20.00'), dan))),
            split=ItemisedExpenseSplit(_fado_items(people)),
            category=categories.food,
            description='Tasca do Chico',
            time=TimeOfDay(22, 15),
            tip=Tip(TipDistribution.PROPORTIONAL, PercentageTip(Percentage.parse('10'))),
        ),
        Expense(
            cost=LocalCost(eur('45.00')),
            date=october(5),
            name='Tuk-tuk tour',
            payers=EqualSplit((dan,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob, carol, dan, eva))),
            category=categories.transport,
            tip=Tip(TipDistribution.EQUAL, FixedTip(eur('5.00'))),
        ),
    )


def _share(group: Group, expense_id: ExpenseId, member: MemberId) -> MemberShare | None:
    """
    Find what member paid and consumed of the expense expense_id.

    Args:
        group: The group.
        expense_id: The expense's identifier.
        member: The member.

    Returns:
        The member's share, or none for an expense the group does not hold or a member it does not involve.

    """
    shares = group.expense_shares(expense_id)
    return None if shares is None else next((share for share in shares.members if share.member == member), None)


def _consumed(group: Group, expense_id: ExpenseId, member: MemberId) -> Money | None:
    """
    Find what member consumed of the expense expense_id.

    Args:
        group: The group.
        expense_id: The expense's identifier.
        member: The member.

    Returns:
        The member's consumption, or none for an expense the group does not hold or a member it does not involve.

    """
    share = _share(group, expense_id, member)
    return None if share is None else share.share


def _print_fado(group: Group, fado: Expense) -> None:
    """
    Print how the fado night's tip was shared, and its receipt.

    Args:
        group: The group.
        fado: The fado night.

    """
    tip = fado.tip
    assert tip is not None, 'the fado night has a tip'
    print_lines(
        "The groceries' 10.00 EUR did not divide into thirds, so one member consumed the",
        "leftover cent; the expense's identifier picks which, so it moves from expense to",
        f'expense. The fado night, at {fado.time or ""} on {fado.date} at {fado.description or ""}, was itemised; its',
        f'{_worth(tip.value)} tip of {fado.tip_amount()} was shared {_shared(tip.distribution)}:',
    )
    _print_items(group, fado)
    print_lines("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.")


def expenses(group: Group, people: People, categories: Categories) -> ExpenseId:
    """
    Print section 4: add an expense for every way of dividing one, and show each member's payment and consumption.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.

    Returns:
        The seafood dinner, which section 6 corrects.

    """
    section('4. Expenses, one for every way of dividing')
    ids = [group.add_expense(expense) for expense in _every_way(people, categories)]
    groceries, apartment, seafood, train, museum, fado, tour = ids
    print_lines(
        'Seven expenses cover every way of dividing one: equally, by percentage, by parts',
        '(Eva counts as half), by exact amounts, with two payers, item by item, and with a',
        "tip. Every member's consumption adds up to the expense's total, to the cent:",
    )
    _print_shares(group, ids)
    fado_expense = group.expense(fado)
    assert fado_expense is not None, 'the fado night was added'
    _print_fado(group, fado_expense)

    thirds = sorted(
        share.minor_units for member in (people.ana, people.bob, people.carol) if (share := _consumed(group, groceries, member)) is not None
    )
    assert thirds == [333, 333, 334]
    assert _consumed(group, apartment, people.carol) == eur('280.00')
    assert _consumed(group, seafood, people.eva) == eur('10.00')
    assert _consumed(group, train, people.eva) == eur('15.00')
    bob_at_the_museum = _share(group, museum, people.bob)
    assert bob_at_the_museum is not None
    assert bob_at_the_museum.paid == eur('30.00')
    assert _consumed(group, museum, people.dan) == eur('15.00')
    museum_expense = group.expense(museum)
    assert museum_expense is not None
    assert len(museum_expense.members()) == 4
    assert fado_expense.total() == eur('59.40')
    assert fado_expense.tip_amount() == eur('5.40')
    assert _consumed(group, fado, people.carol) == eur('21.45')
    assert _consumed(group, fado, people.eva) == eur('6.60')
    assert _consumed(group, tour, people.eva) == eur('10.00')
    return seafood
