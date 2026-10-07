"""Print section 4 of the tour: an expense for every way of dividing one, with each member's payment and consumption."""

import dataclasses
from collections.abc import Sequence
from typing import ClassVar, assert_never, final, override

from eranos import (
    AmountsSplit,
    EqualSplit,
    Expense,
    ExpenseId,
    FixedTip,
    ExpenseGroup,
    ExpenseItem,
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
from examples.report import Row, names, print_lines, print_table, section


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Expenses:
    """The seven expenses, one for every way of dividing one."""

    apartment: ExpenseId
    """The apartment, divided by percentage."""

    fado: ExpenseId
    """The fado night, item by item, with two payers and a tip."""

    groceries: ExpenseId
    """The groceries, divided equally."""

    museum: ExpenseId
    """The museum, with two payers."""

    seafood: ExpenseId
    """The seafood dinner, divided by parts."""

    tour: ExpenseId
    """The tuk-tuk tour, with a fixed tip."""

    train: ExpenseId
    """The train to Porto, divided by exact amounts."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ShareRow(Row):
    """A row of the table of shares: an expense, how it divides, and every involved member's payment and consumption."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Expense', 'Split', 'Member', 'Paid', 'Consumed')

    expense: str
    """The expense's name."""

    how: str
    """How the expense divides, in a few words."""

    members: str
    """The involved members' first names, a line each."""

    paid: str
    """What each member paid, a line each, blank for a member who paid nothing."""

    consumed: str
    """What each member consumed, a line each."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.expense, self.how, self.members, self.paid, self.consumed


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ItemRow(Row):
    """A row of the table of an itemised expense's lines."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Item', 'Shared by', 'Quantity', 'Unit price', 'Line total')

    name: str
    """What the line is."""

    shared_by: str
    """Who had it."""

    quantity: int
    """How many were ordered."""

    unit_price: Money
    """What one costs."""

    line_total: Money
    """The unit price times the quantity."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.shared_by, str(self.quantity), str(self.unit_price), str(self.line_total)


def _divided(split: Split) -> str:
    """
    Tell how a whole expense's amount divides, in a word or two.

    Args:
        split: How the amount divides.

    Returns:
        The words, such as ``"by parts"``.

    """
    match split:
        case EqualSplit():
            return 'equally'
        case PartsSplit():
            return 'by parts'
        case PercentagesSplit():
            return 'by percentage'
        case AmountsSplit():
            return 'by amounts'
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
        case PercentageTip(percentage=percentage):
            return str(percentage)
        case FixedTip(amount=amount):
            return str(amount)
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
        case TipDistribution.PROPORTIONAL:
            return 'in proportion to what each member ordered'
        case TipDistribution.EQUAL:
            return 'equally'
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
        case WholeExpenseSplit(split=whole):
            divided = _divided(whole)
        case ItemisedExpenseSplit(items=items):
            divided = f'{len(items)} items'
        case _:
            assert_never(split)
    count = len(expense.payers.members)
    payers = '' if count == 1 else f', {count} payers'
    tip = expense.tip
    worth = '' if tip is None or expense.tip_amount().minor_units == 0 else f', {_worth(tip.value)} tip'
    return divided + payers + worth


def _share_row(group: ExpenseGroup, expense_id: ExpenseId) -> ShareRow | None:
    """
    Write the row of an expense among the shares: every involved member's payment and consumption.

    Args:
        group: The group.
        expense_id: The expense's identifier.

    Returns:
        The row, or none for an expense the group does not hold.

    """
    expense = group.expense(expense_id)
    shares = group.expense_shares(expense_id)
    if expense is None or shares is None:
        return None
    return ShareRow(
        expense=expense.name,
        how=_how(expense),
        members=cell(name(group, share.member) for share in shares.members),
        paid=cell('' if share.paid.minor_units == 0 else str(share.paid) for share in shares.members),
        consumed=cell(str(share.share) for share in shares.members),
    )


def _print_shares(group: ExpenseGroup, expenses: Sequence[ExpenseId]) -> None:
    """
    Print each of expenses, with every involved member's payment and consumption.

    Args:
        group: The group.
        expenses: The expenses' identifiers.

    """
    print_table(
        ShareRow,
        [row for expense_id in expenses if (row := _share_row(group, expense_id)) is not None],
        3,
    )


def _line_total(item: ExpenseItem) -> Money:
    """
    Work out an item's line total, its unit price times its quantity, which the layer does not offer.

    Args:
        item: The line of the bill.

    Returns:
        The line total, in the expense's currency.

    """
    return Money(item.unit_price.minor_units * item.quantity, item.unit_price.currency)


def _print_items(group: ExpenseGroup, expense: Expense) -> None:
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
        ItemRow,
        [
            ItemRow(
                name=item.name,
                shared_by=names(group, item.split.members),
                quantity=item.quantity,
                unit_price=item.unit_price,
                line_total=_line_total(item),
            )
            for item in split.items
        ],
        2,
    )


def _fado_items(people: People) -> tuple[ExpenseItem, ...]:
    """
    Write the receipt of the fado night: every line with its own split.

    Args:
        people: The members of the trip.

    Returns:
        The lines of the bill.

    """
    one = Parts.parse('1')
    return (
        ExpenseItem(name='Vinho verde', unit_price=eur('12.00'), split=EqualSplit((people.ana, people.bob, people.dan)), quantity=2),
        ExpenseItem(name='Bacalhau', unit_price=eur('18.00'), split=EqualSplit((people.carol,))),
        ExpenseItem(
            name='Custard tarts',
            unit_price=eur('1.50'),
            split=PartsSplit(
                (
                    MemberParts(people.ana, one),
                    MemberParts(people.bob, one),
                    MemberParts(people.carol, one),
                    MemberParts(people.dan, one),
                    MemberParts(people.eva, Parts.parse('2')),
                )
            ),
            quantity=6,
        ),
        ExpenseItem(name='Juice', unit_price=eur('3.00'), split=EqualSplit((people.eva,))),
    )


def _add_expenses(group: ExpenseGroup, people: People, categories: Categories) -> Expenses:
    """
    Add one expense for every way of dividing one.

    The expenses are added in the order the report lists them, which gives each its identifier.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.

    Returns:
        The expenses' identifiers.

    """
    ana, bob, carol, dan, eva = people.ana, people.bob, people.carol, people.dan, people.eva
    one = Parts.parse('1')
    groceries = group.add_expense(
        Expense(
            name='Groceries',
            cost=LocalCost(eur('10.00')),
            payers=EqualSplit((ana,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob, carol))),
            date=october(1),
            category=categories.food,
        )
    )
    apartment = group.add_expense(
        Expense(
            name='Apartment',
            cost=LocalCost(eur('800.00')),
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
            date=october(1),
            category=categories.lodging,
        )
    )
    seafood = group.add_expense(
        Expense(
            name='Seafood dinner',
            cost=LocalCost(eur('90.00')),
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
            date=october(2),
            category=categories.food,
        )
    )
    train = group.add_expense(
        Expense(
            name='Train to Porto',
            cost=LocalCost(eur('88.50')),
            payers=EqualSplit((bob,)),
            split=WholeExpenseSplit(
                AmountsSplit(
                    (
                        MemberAmount(ana, eur('24.50')),
                        MemberAmount(bob, eur('24.50')),
                        MemberAmount(carol, eur('24.50')),
                        MemberAmount(eva, eur('15.00')),
                    )
                )
            ),
            date=october(3),
            category=categories.transport,
        )
    )
    museum = group.add_expense(
        Expense(
            name='Museum',
            cost=LocalCost(eur('60.00')),
            payers=EqualSplit((ana, bob)),
            split=WholeExpenseSplit(EqualSplit((ana, bob, carol, dan))),
            date=october(3),
        )
    )
    fado = group.add_expense(
        Expense(
            name='Fado night',
            cost=LocalCost(eur('54.00')),
            payers=AmountsSplit((MemberAmount(carol, eur('39.40')), MemberAmount(dan, eur('20.00')))),
            split=ItemisedExpenseSplit(_fado_items(people)),
            date=october(4),
            tip=Tip(PercentageTip(Percentage.parse('10'))),
            time=TimeOfDay(22, 15),
            category=categories.food,
            description='Tasca do Chico',
        )
    )
    tour = group.add_expense(
        Expense(
            name='Tuk-tuk tour',
            cost=LocalCost(eur('45.00')),
            payers=EqualSplit((dan,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob, carol, dan, eva))),
            date=october(5),
            tip=Tip(FixedTip(eur('5.00')), TipDistribution.EQUAL),
            category=categories.transport,
        )
    )
    return Expenses(apartment=apartment, fado=fado, groceries=groceries, museum=museum, seafood=seafood, tour=tour, train=train)


def _share(group: ExpenseGroup, expense_id: ExpenseId, member: MemberId) -> MemberShare | None:
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


def _consumed(group: ExpenseGroup, expense_id: ExpenseId, member: MemberId) -> Money | None:
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


def _print_fado(group: ExpenseGroup, fado: Expense) -> None:
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


def expenses(group: ExpenseGroup, people: People, categories: Categories) -> ExpenseId:
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
    added = _add_expenses(group, people, categories)
    print_lines(
        'Seven expenses cover every way of dividing one: equally, by percentage, by parts',
        '(Eva counts as half), by exact amounts, with two payers, item by item, and with a',
        "tip. Every member's consumption adds up to the expense's total, to the cent:",
    )
    _print_shares(group, (added.groceries, added.apartment, added.seafood, added.train, added.museum, added.fado, added.tour))
    fado_expense = group.expense(added.fado)
    assert fado_expense is not None, 'the fado night was added'
    _print_fado(group, fado_expense)

    thirds = sorted(
        share.minor_units for member in (people.ana, people.bob, people.carol) if (share := _consumed(group, added.groceries, member)) is not None
    )
    assert thirds == [333, 333, 334]
    assert _consumed(group, added.apartment, people.carol) == eur('280.00')
    assert _consumed(group, added.seafood, people.eva) == eur('10.00')
    assert _consumed(group, added.train, people.eva) == eur('15.00')
    bob_at_the_museum = _share(group, added.museum, people.bob)
    assert bob_at_the_museum is not None
    assert bob_at_the_museum.paid == eur('30.00')
    assert _consumed(group, added.museum, people.dan) == eur('15.00')
    museum_expense = group.expense(added.museum)
    assert museum_expense is not None
    assert len(museum_expense.members()) == 4
    assert fado_expense.total() == eur('59.40')
    assert fado_expense.tip_amount() == eur('5.40')
    assert _consumed(group, added.fado, people.carol) == eur('21.45')
    assert _consumed(group, added.fado, people.eva) == eur('6.60')
    assert _consumed(group, added.tour, people.eva) == eur('10.00')
    return added.seafood
