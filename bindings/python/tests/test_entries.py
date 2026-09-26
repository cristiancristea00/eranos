"""Check that every entry and every variant goes through the group and comes back as it went in, and what the group refuses."""

import copy

import pytest

from conftest import Flat
from eranos import (
    Account,
    AmountsSplit,
    Category,
    ConvertedCost,
    Currency,
    Email,
    EqualSplit,
    ExchangeRate,
    Expense,
    ExpenseId,
    FixedTip,
    GetsBackBalance,
    GroupError,
    Iban,
    IbanAccountDetails,
    IdError,
    Item,
    ItemisedExpenseSplit,
    LocalCost,
    Member,
    MemberAmount,
    MemberId,
    MemberParts,
    MemberPercentage,
    Money,
    OwesBalance,
    Parts,
    PartsSplit,
    Percentage,
    PercentagesSplit,
    PercentageTip,
    PhoneNumber,
    SettledBalance,
    TimeOfDay,
    Tip,
    TipDistribution,
    Transfer,
    TransferId,
    Username,
    UsernameAccountDetails,
    WholeExpenseSplit,
)


def test_an_expense_reads_back_as_it_was_added(flat: Flat) -> None:
    """An expense that came back from the group otherwise than it went in would change what an app shows after a save."""
    lunch = Expense(
        cost=ConvertedCost(Money(Currency('RON'), 5000), ExchangeRate('0.2')),
        date=flat.day,
        name='Lunch',
        payers=PercentagesSplit((MemberPercentage(flat.ana, Percentage(5000)), MemberPercentage(flat.bob, Percentage(5000)))),
        split=WholeExpenseSplit(PartsSplit((MemberParts(flat.ana, Parts(100)), MemberParts(flat.bob, Parts(50))))),
        category=flat.food,
        description='By the sea',
        time=TimeOfDay(13, 15),
        tip=Tip(TipDistribution.PROPORTIONAL, PercentageTip(Percentage(1000))),
    )
    lunch_id = flat.group.add_expense(lunch)
    assert flat.group.expense(lunch_id) == copy.replace(lunch, id=lunch_id)
    pizza = Item(name='Pizza', split=EqualSplit((flat.ana, flat.bob)), unit_price=flat.euros(1200), quantity=2)
    wine = Item(name='Wine', split=PercentagesSplit((MemberPercentage(flat.ana, Percentage(10000)),)), unit_price=flat.euros(600))
    receipt = Expense(
        cost=LocalCost(flat.euros(3000)),
        date=flat.day,
        name='Receipt',
        payers=AmountsSplit((MemberAmount(flat.euros(3300), flat.bob),)),
        split=ItemisedExpenseSplit((pizza, wine)),
        tip=Tip(TipDistribution.EQUAL, FixedTip(flat.euros(300))),
    )
    receipt_id = flat.group.add_expense(receipt)
    assert flat.group.expense(receipt_id) == copy.replace(receipt, id=receipt_id)
    assert len(flat.group.expenses()) == 3
    assert flat.group.expense(ExpenseId('0199a000-0000-7000-8000-00000000ffff')) is None


def test_a_transfer_and_a_member_read_back_as_they_were_added(flat: Flat) -> None:
    """A transfer or a member that came back otherwise than it went in would move money to the wrong account after a save."""
    cash = Transfer(
        cost=ConvertedCost(Money(Currency('RON'), 1000), ExchangeRate('0.2')),
        date=flat.day,
        receiver=flat.bob,
        sender=flat.ana,
        note='Cash',
        time=TimeOfDay(9, 5),
    )
    cash = copy.replace(cash, id=flat.group.add_transfer(cash))
    assert cash.id is not None
    assert flat.group.transfer(cash.id) == cash
    assert flat.group.transfer_amount(cash.id) == flat.euros(200)
    assert len(flat.group.transfers()) == 2
    assert flat.group.transfer_amount(TransferId('0199a000-0000-7000-8000-00000000ffff')) is None
    carla = Member(
        first_name='Carla',
        accounts=(
            Account(IbanAccountDetails(Iban('RO49AAAA1B31007593840000')), 'Bank'),
            Account(UsernameAccountDetails(Username('carla.pays')), 'App'),
        ),
        email=Email('carla@example.com'),
        last_name='Pop',
        phone=PhoneNumber('+40712345678'),
    )
    carla = copy.replace(carla, id=flat.group.add_member(carla))
    assert carla.id is not None
    assert flat.group.member(carla.id) == carla
    assert len(flat.group.members()) == 3
    assert flat.group.category(flat.food) == Category(id=flat.food, name='Food')
    assert len(flat.group.categories()) == 2


def test_replacing_an_entry_keeps_its_identifier_and_returns_it_as_it_was(flat: Flat) -> None:
    """An edit that lost an entry's identifier, or returned the entry as it became, would leave an app's undo pointing nowhere."""
    dinner = flat.group.expense(flat.dinner)
    assert dinner is not None
    lunch = copy.replace(dinner, name='Lunch')
    assert flat.group.replace_expense(lunch) == dinner
    assert flat.group.expense(flat.dinner) == lunch
    repayment = flat.group.transfer(flat.repayment)
    assert repayment is not None
    noted = copy.replace(repayment, note='Back')
    assert flat.group.replace_transfer(noted) == repayment
    assert flat.group.transfer(flat.repayment) == noted
    ana = flat.group.member(flat.ana)
    assert ana is not None
    renamed = copy.replace(ana, first_name='Ana Maria')
    assert flat.group.replace_member(renamed) == ana
    assert flat.group.member(flat.ana) == renamed
    food = flat.group.category(flat.food)
    assert food is not None
    meals = copy.replace(food, name='Meals')
    assert flat.group.replace_category(meals) == food
    assert flat.group.category(flat.food) == meals
    with pytest.raises(ValueError, match='identifier'):
        flat.group.replace_expense(flat.expense('Snack', flat.ana, (flat.ana,)))
    with pytest.raises(ValueError, match='identifier'):
        flat.group.add_expense(lunch)


def test_removing_an_entry_returns_it_and_forgets_it(flat: Flat) -> None:
    """A removal that returned something else, or left the entry behind, would let an app show a deleted entry."""
    dinner = flat.group.expense(flat.dinner)
    assert flat.group.remove_expense(flat.dinner) == dinner
    assert flat.group.expense(flat.dinner) is None
    repayment = flat.group.transfer(flat.repayment)
    assert flat.group.remove_transfer(flat.repayment) == repayment
    assert flat.group.transfer(flat.repayment) is None
    food = flat.group.category(flat.food)
    assert flat.group.remove_category(flat.food) == food
    assert flat.group.category(flat.food) is None
    bob = flat.group.member(flat.bob)
    assert flat.group.remove_member(flat.bob) == bob
    assert flat.group.member(flat.bob) is None


def test_a_member_in_use_is_not_removed(flat: Flat) -> None:
    """A member removed while entries name them would leave shares nobody owes."""
    with pytest.raises(GroupError) as refused:
        flat.group.remove_member(flat.ana)
    assert refused.value.failure == GroupError.MemberInUse(expenses=(flat.dinner,), member=flat.ana, transfers=(flat.repayment,))


def test_an_archived_member_joins_no_new_expense(flat: Flat) -> None:
    """An archived member who could join a new expense would owe money for a trip they left."""
    flat.group.archive_member(flat.bob)
    assert flat.group.is_archived(flat.bob)
    assert not flat.group.is_archived(flat.ana)
    with pytest.raises(GroupError) as refused:
        flat.group.add_expense(flat.expense('Snack', flat.ana, (flat.ana, flat.bob)))
    assert refused.value.failure == GroupError.ArchivedMember(member=flat.bob)
    flat.group.unarchive_member(flat.bob)
    assert not flat.group.is_archived(flat.bob)
    nobody = MemberId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(GroupError) as unknown:
        flat.group.archive_member(nobody)
    assert unknown.value.failure == GroupError.UnknownMember(member=nobody)


def test_the_helpers_read_an_entry_as_the_library_does(flat: Flat) -> None:
    """A helper that read an entry otherwise than the library would show an app a total the group does not hold."""
    assert EqualSplit((flat.bob, flat.ana)).members == (flat.bob, flat.ana)
    assert AmountsSplit((MemberAmount(flat.euros(100), flat.bob),)).members == (flat.bob,)
    assert PartsSplit((MemberParts(flat.ana, Parts(100)),)).members == (flat.ana,)
    assert PercentagesSplit((MemberPercentage(flat.bob, Percentage(10000)),)).members == (flat.bob,)
    dinner = copy.replace(flat.expense('Dinner', flat.ana, (flat.bob,)), tip=Tip(TipDistribution.EQUAL, PercentageTip(Percentage(1000))))
    assert dinner.members() == tuple(sorted((flat.ana, flat.bob)))
    assert dinner.tip_amount() == flat.euros(300)
    assert dinner.total() == flat.euros(3300)
    assert str(GetsBackBalance(flat.euros(1200))) == 'gets back 12.00 EUR'
    assert str(OwesBalance(flat.euros(1200))) == 'owes 12.00 EUR'
    assert str(SettledBalance()) == 'settled'
    with pytest.raises(IdError) as unknown:
        flat.expense('Snack', MemberId('not an identifier'), (flat.ana,)).members()
    assert unknown.value.failure == IdError.InvalidId()
