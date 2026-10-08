"""Check that every entry goes through the group and back as it went in, what the group refuses, and what an entry's helpers work out."""

import copy
import dataclasses
from typing import final

import pytest

from conftest import Flat
from eranos import (
    NO_TIP,
    UNCATEGORISED,
    PaymentAccount,
    AmountsSplit,
    Category,
    ConvertedCost,
    CurrencyCode,
    Discount,
    DiscountScope,
    Email,
    EqualSplit,
    ExchangeRate,
    Expense,
    ExpenseError,
    ExpenseFailure,
    ExpenseGroup,
    ExpenseId,
    FixedDiscount,
    FixedTip,
    GetsBackBalance,
    ExpenseGroupError,
    Iban,
    IbanPaymentAccountDetails,
    IdError,
    ExpenseItem,
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
    PercentageDiscount,
    PercentageTip,
    PhoneNumber,
    Quantity,
    SettledBalance,
    TimeOfDay,
    TipRule,
    TipDistribution,
    Transfer,
    TransferId,
    Username,
    UsernamePaymentAccountDetails,
    WholeExpenseSplit,
    parse_category_id,
    parse_expense_id,
    parse_member_id,
    parse_transfer_id,
)


def test_an_expense_reads_back_as_it_was_added(flat: Flat) -> None:
    """An expense that came back from the group otherwise than it went in would change what an app shows after a save."""
    lunch = Expense.create(
        name='Lunch',
        cost=ConvertedCost(Money(5000, CurrencyCode('RON')), ExchangeRate('0.2')),
        payers=PercentagesSplit((MemberPercentage(flat.ana, Percentage(5000)), MemberPercentage(flat.bob, Percentage(5000)))),
        split=WholeExpenseSplit(PartsSplit((MemberParts(flat.ana, Parts(100)), MemberParts(flat.bob, Parts(50))))),
        date=flat.day,
        tip=TipRule(PercentageTip(Percentage(1000))),
        time=TimeOfDay(13, 15),
        category=flat.food,
        description='By the sea',
    )
    assert flat.group.add_expense(lunch) == lunch.id
    assert flat.group.expense(lunch.id) == lunch
    pizza = ExpenseItem(name='Pizza', unit_price=flat.euros(1050), split=EqualSplit((flat.ana, flat.bob)), quantity=Quantity.whole(2))
    wine = ExpenseItem(
        name='Wine',
        unit_price=flat.euros(1200),
        split=PercentagesSplit((MemberPercentage(flat.ana, Percentage(10000)),)),
        quantity=Quantity.parse('0.5'),
    )
    bread = ExpenseItem(name='Bread', unit_price=flat.euros(300), split=EqualSplit((flat.bob,)))
    receipt = Expense.create(
        name='Receipt',
        cost=LocalCost(flat.euros(3000)),
        payers=AmountsSplit((MemberAmount(flat.bob, flat.euros(3300)),)),
        split=ItemisedExpenseSplit((pizza, wine, bread)),
        date=flat.day,
        tip=TipRule(FixedTip(flat.euros(300)), TipDistribution.EQUAL),
    )
    assert flat.group.add_expense(receipt) == receipt.id
    assert flat.group.expense(receipt.id) == receipt
    assert len(flat.group.expenses()) == 3
    assert flat.group.expense(ExpenseId('0199a000-0000-7000-8000-00000000ffff')) is None


def test_an_expense_starts_with_no_tip_in_the_built_in_category(flat: Flat) -> None:
    """An expense made without a tip or a category that started with either would add to a bill a tip or a category the app never chose."""
    cost, payers, split = LocalCost(flat.euros(1200)), EqualSplit((flat.ana,)), WholeExpenseSplit(EqualSplit((flat.ana, flat.bob)))
    created = Expense.create(name='Bus', cost=cost, payers=payers, split=split, date=flat.day)
    made = Expense(id=created.id, name='Bus', cost=cost, payers=payers, split=split, date=flat.day)
    assert made == created
    for expense in (created, made):
        assert expense.tip == NO_TIP
        assert expense.category == UNCATEGORISED
        assert expense.tip_amount() == flat.euros(0)
        assert expense.total() == flat.euros(1200)


def test_an_expense_without_a_tip_or_a_category_reads_back_with_no_tip_in_the_built_in_category(flat: Flat) -> None:
    """
    An expense left without a tip or a category that read back with another tip or category would show an app a value it never set.

    The group holds every expense with a tip rule and in a category: the no-tip rule, and the built-in category, which it lists first and
    which takes the expenses of a category it removes.
    """
    taxi = flat.expense('Taxi', flat.ana, (flat.ana, flat.bob))
    assert flat.group.add_expense(taxi) == taxi.id
    stored = flat.group.expense(taxi.id)
    assert stored is not None
    assert stored.tip == NO_TIP
    assert stored.category == UNCATEGORISED
    assert stored == taxi
    assert flat.group.categories()[0] == Category(id=UNCATEGORISED, name='Uncategorised')
    assert flat.group.remove_category(flat.food).id == flat.food
    dinner = flat.group.expense(flat.dinner)
    assert dinner is not None
    assert dinner.category == UNCATEGORISED


def test_a_zero_tip_reads_back_as_it_was_set(flat: Flat) -> None:
    """
    A 0 % tip set on purpose that read back as another rule would change the rule an app shows.

    Shared in proportion, it is the no-tip rule; shared equally, it is a rule of its own, which the group keeps.
    """
    proportional = copy.replace(flat.expense('Taxi', flat.ana, (flat.ana, flat.bob)), tip=TipRule(PercentageTip(Percentage(0))))
    equal = copy.replace(flat.expense('Bus', flat.ana, (flat.ana, flat.bob)), tip=TipRule(PercentageTip(Percentage(0)), TipDistribution.EQUAL))
    for expense in (proportional, equal):
        assert flat.group.add_expense(expense) == expense.id
        assert flat.group.expense(expense.id) == expense
    assert proportional.tip == NO_TIP
    assert equal.tip != NO_TIP


def test_a_transfer_and_a_member_read_back_as_they_were_added(flat: Flat) -> None:
    """A transfer or a member that came back otherwise than it went in would move money to the wrong account after a save."""
    cash = Transfer.create(
        sender=flat.ana,
        receiver=flat.bob,
        cost=ConvertedCost(Money(1000, CurrencyCode('RON')), ExchangeRate('0.2')),
        date=flat.day,
        time=TimeOfDay(9, 5),
        note='Cash',
    )
    assert flat.group.add_transfer(cash) == cash.id
    assert flat.group.transfer(cash.id) == cash
    assert flat.group.transfer_amount(cash.id) == flat.euros(200)
    assert len(flat.group.transfers()) == 2
    assert flat.group.transfer_amount(TransferId('0199a000-0000-7000-8000-00000000ffff')) is None
    carla = Member.create(
        first_name='Carla',
        last_name='Pop',
        email=Email('carla@example.com'),
        phone=PhoneNumber('+40712345678'),
        accounts=(
            PaymentAccount('Bank', IbanPaymentAccountDetails(Iban('RO49AAAA1B31007593840000'))),
            PaymentAccount('App', UsernamePaymentAccountDetails(Username('carla.pays'))),
        ),
    )
    assert flat.group.add_member(carla) == carla.id
    assert flat.group.member(carla.id) == carla
    assert len(flat.group.members()) == 3
    assert flat.group.category(flat.food) == Category(id=flat.food, name='Food')
    assert len(flat.group.categories()) == 2


def test_create_makes_every_entry_under_a_new_identifier_with_the_fields_it_is_given(flat: Flat) -> None:
    """
    A create that reused an identifier would make two entries collide in the group; one that dropped a field would lose what the app entered.

    Each create draws its identifier from the library's one generator when it is called, so the identifiers follow the order of the calls,
    whatever the kind, and the class, given that identifier and the same fields, makes the same entry.
    """
    email, phone, time = Email('carla@example.com'), PhoneNumber('+40712345678'), TimeOfDay(8, 30)
    accounts = (PaymentAccount('App', UsernamePaymentAccountDetails(Username('carla.pays'))),)
    cost, tip = LocalCost(flat.euros(300)), TipRule(FixedTip(flat.euros(30)))
    payers, split = EqualSplit((flat.ana,)), WholeExpenseSplit(EqualSplit((flat.ana, flat.bob)))
    carla = Member.create(first_name='Carla', last_name='Pop', email=email, phone=phone, accounts=accounts)
    assert carla == Member(id=carla.id, first_name='Carla', last_name='Pop', email=email, phone=phone, accounts=accounts)
    drinks = Category.create(name='Drinks')
    assert drinks == Category(id=drinks.id, name='Drinks')
    coffee = Expense.create(
        name='Coffee', cost=cost, payers=payers, split=split, date=flat.day, tip=tip, time=time, category=flat.food, description='Two cups'
    )
    assert coffee == Expense(
        id=coffee.id,
        name='Coffee',
        cost=cost,
        payers=payers,
        split=split,
        date=flat.day,
        tip=tip,
        time=time,
        category=flat.food,
        description='Two cups',
    )
    coins = Transfer.create(sender=flat.bob, receiver=flat.ana, cost=cost, date=flat.day, time=time, note='Coins')
    assert coins == Transfer(id=coins.id, sender=flat.bob, receiver=flat.ana, cost=cost, date=flat.day, time=time, note='Coins')
    twin = Member.create(first_name='Carla', last_name='Pop', email=email, phone=phone, accounts=accounts)
    assert twin != carla
    assert copy.replace(twin, id=carla.id) == carla
    identifiers = [carla.id, drinks.id, coffee.id, coins.id, twin.id]
    assert identifiers == sorted(identifiers)
    assert len(set(identifiers)) == len(identifiers)
    assert parse_member_id(carla.id) == carla.id
    assert parse_category_id(drinks.id) == drinks.id
    assert parse_expense_id(coffee.id) == coffee.id
    assert parse_transfer_id(coins.id) == coins.id


def test_every_entry_keeps_the_identifier_its_app_gives_it(flat: Flat) -> None:
    """An entry that lost the identifier its app gave it, or replaced one the group holds under it, would part the app's store and the group."""
    carla = parse_member_id('!a')
    member = Member(id=carla, first_name='Carla')
    assert flat.group.add_member(member) == carla
    assert flat.group.member(carla) == member
    drinks = parse_category_id('-food')
    category = Category(id=drinks, name='Drinks')
    assert flat.group.add_category(category) == drinks
    assert flat.group.category(drinks) == category
    omega = parse_expense_id('Ωmega')
    expense = copy.replace(flat.expense('Snack', flat.ana, (flat.ana, carla)), id=omega, category=drinks)
    assert flat.group.add_expense(expense) == omega
    assert flat.group.expense(omega) == expense
    coins = parse_transfer_id('zz')
    transfer = Transfer(id=coins, sender=carla, receiver=flat.ana, cost=LocalCost(flat.euros(100)), date=flat.day)
    assert flat.group.add_transfer(transfer) == coins
    assert flat.group.transfer(coins) == transfer
    with pytest.raises(ExpenseGroupError) as second_member:
        flat.group.add_member(copy.replace(member, first_name='Dana'))
    assert second_member.value.failure == ExpenseGroupError.DuplicateMember(member=carla)
    with pytest.raises(ExpenseGroupError) as second_category:
        flat.group.add_category(copy.replace(category, name='Sweets'))
    assert second_category.value.failure == ExpenseGroupError.DuplicateCategory(category=drinks)
    with pytest.raises(ExpenseGroupError) as second_expense:
        flat.group.add_expense(copy.replace(expense, name='Tea'))
    assert second_expense.value.failure == ExpenseGroupError.DuplicateExpense(expense=omega)
    with pytest.raises(ExpenseGroupError) as second_transfer:
        flat.group.add_transfer(copy.replace(transfer, note='Again'))
    assert second_transfer.value.failure == ExpenseGroupError.DuplicateTransfer(transfer=coins)
    assert flat.group.member(carla) == member
    assert flat.group.category(drinks) == category
    assert flat.group.expense(omega) == expense
    assert flat.group.transfer(coins) == transfer
    with pytest.raises(IdError) as padded:
        flat.group.add_member(Member(id=MemberId(' x'), first_name='Dana'))
    assert padded.value.failure == IdError.SurroundingWhitespace()
    assert len(flat.group.members()) == 3


def test_identifiers_order_in_python_as_the_group_lists_them(flat: Flat) -> None:
    """
    An app that sorted identifiers otherwise than the library, by UTF-16 code units, would list entries in another order than the group.

    The group orders identifiers by their bytes of UTF-8, as Python orders text by code points: U+FFFD comes before U+1F600 in both, and after it in
    UTF-16. Text that Unicode holds equivalent, a precomposed é and an e with a combining accent, stays two identifiers.
    """
    added = [
        flat.group.add_member(Member(id=parse_member_id(text), first_name='Carla'))
        for text in ('\U0001f600', '\ufffd', 'Z', 'a', '-food', '\u00e9', 'e\u0301')
    ]
    assert [member.id for member in flat.group.members()] == sorted([flat.ana, flat.bob, *added])


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
    with pytest.raises(ExpenseGroupError) as added:
        flat.group.add_expense(lunch)
    assert added.value.failure == ExpenseGroupError.DuplicateExpense(expense=flat.dinner)


def test_optional_text_reads_back_trimmed_and_absent_when_left_out_or_blank(flat: Flat) -> None:
    """
    Optional text read back padded, empty, or as it was before a replacement without it would show an app text the user never entered or removed.

    The optional text of an entry is an expense's description, a member's last name, and a transfer's note; the group trims each, and holds
    none for text left out or blank.
    """
    dinner, ana, repayment = flat.group.expense(flat.dinner), flat.group.member(flat.ana), flat.group.transfer(flat.repayment)
    assert dinner is not None
    assert ana is not None
    assert repayment is not None
    for text in (None, ' '):
        flat.group.add_expense(copy.replace(flat.expense('Lunch', flat.ana, (flat.ana,)), description=text))
        flat.group.add_member(Member.create(first_name='Carla', last_name=text))
        flat.group.add_transfer(Transfer.create(sender=flat.ana, receiver=flat.bob, cost=LocalCost(flat.euros(100)), date=flat.day, note=text))
        flat.group.replace_expense(copy.replace(dinner, description=' Soup\n'))
        flat.group.replace_member(copy.replace(ana, last_name=' Pop\n'))
        flat.group.replace_transfer(copy.replace(repayment, note=' Coins\n'))
        assert flat.group.replace_expense(copy.replace(dinner, description=text)).description == 'Soup'
        assert flat.group.replace_member(copy.replace(ana, last_name=text)).last_name == 'Pop'
        assert flat.group.replace_transfer(copy.replace(repayment, note=text)).note == 'Coins'
        assert {expense.description for expense in flat.group.expenses()} == {None}
        assert {member.last_name for member in flat.group.members()} == {None}
        assert {transfer.note for transfer in flat.group.transfers()} == {None}


def test_replacing_an_entry_the_group_does_not_hold_is_refused(flat: Flat) -> None:
    """A replace that added an entry the group does not hold would bring back, through an app's stale edit, an entry another device removed."""
    snack = flat.expense('Snack', flat.ana, (flat.ana,))
    with pytest.raises(ExpenseGroupError) as expense:
        flat.group.replace_expense(snack)
    assert expense.value.failure == ExpenseGroupError.UnknownExpense(expense=snack.id)
    coins = Transfer.create(sender=flat.ana, receiver=flat.bob, cost=LocalCost(flat.euros(100)), date=flat.day)
    with pytest.raises(ExpenseGroupError) as transfer:
        flat.group.replace_transfer(coins)
    assert transfer.value.failure == ExpenseGroupError.UnknownTransfer(transfer=coins.id)
    carla = Member.create(first_name='Carla')
    with pytest.raises(ExpenseGroupError) as member:
        flat.group.replace_member(carla)
    assert member.value.failure == ExpenseGroupError.UnknownMember(member=carla.id)
    drinks = Category.create(name='Drinks')
    with pytest.raises(ExpenseGroupError) as category:
        flat.group.replace_category(drinks)
    assert category.value.failure == ExpenseGroupError.UnknownCategory(category=drinks.id)
    assert flat.group.expense(snack.id) is None
    assert flat.group.transfer(coins.id) is None
    assert flat.group.member(carla.id) is None
    assert flat.group.category(drinks.id) is None


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
    with pytest.raises(ExpenseGroupError) as refused:
        flat.group.remove_member(flat.ana)
    assert refused.value.failure == ExpenseGroupError.MemberInUse(member=flat.ana, expenses=(flat.dinner,), transfers=(flat.repayment,))


def test_an_archived_member_joins_no_new_expense(flat: Flat) -> None:
    """An archived member who could join a new expense would owe money for a trip they left."""
    flat.group.archive_member(flat.bob)
    assert flat.group.is_archived(flat.bob) is True
    assert flat.group.is_archived(flat.ana) is False
    with pytest.raises(ExpenseGroupError) as refused:
        flat.group.add_expense(flat.expense('Snack', flat.ana, (flat.ana, flat.bob)))
    assert refused.value.failure == ExpenseGroupError.ArchivedMember(member=flat.bob)
    flat.group.unarchive_member(flat.bob)
    assert flat.group.is_archived(flat.bob) is False
    nobody = MemberId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(ExpenseGroupError) as unknown:
        flat.group.archive_member(nobody)
    assert unknown.value.failure == ExpenseGroupError.UnknownMember(member=nobody)


def test_a_member_the_group_does_not_hold_is_neither_archived_nor_active(flat: Flat) -> None:
    """A member the group does not hold who read as active would let an app offer, for a new expense, a member the group refuses."""
    assert flat.group.is_archived(MemberId('0199a000-0000-7000-8000-00000000ffff')) is None


def test_the_helpers_read_an_entry_as_the_library_does(flat: Flat) -> None:
    """A helper that read an entry otherwise than the library would show an app a total the group does not hold."""
    assert EqualSplit((flat.bob, flat.ana)).members == (flat.bob, flat.ana)
    assert PartsSplit((MemberParts(flat.ana, Parts(100)),)).members == (flat.ana,)
    assert PercentagesSplit((MemberPercentage(flat.bob, Percentage(10000)),)).members == (flat.bob,)
    assert AmountsSplit((MemberAmount(flat.bob, flat.euros(100)),)).members == (flat.bob,)
    dinner = copy.replace(flat.expense('Dinner', flat.ana, (flat.bob,)), tip=TipRule(PercentageTip(Percentage(1000)), TipDistribution.EQUAL))
    assert dinner.total() == flat.euros(3300)
    assert dinner.tip_amount() == flat.euros(300)
    assert dinner.members() == tuple(sorted((flat.ana, flat.bob)))
    assert str(OwesBalance(flat.euros(1200))) == 'owes 12.00 EUR'
    assert str(SettledBalance()) == 'settled'
    assert str(GetsBackBalance(flat.euros(1200))) == 'gets back 12.00 EUR'
    with pytest.raises(IdError) as unknown:
        flat.expense('Snack', MemberId(' ana'), (flat.ana,)).members()
    assert unknown.value.failure == IdError.SurroundingWhitespace()


def test_an_items_line_total_is_its_unit_price_times_its_quantity_rounded_half_away_from_zero(flat: Flat) -> None:
    """A line total worked out otherwise than the library's would show a receipt whose lines do not add up to what the group divides."""
    split = EqualSplit((flat.ana, flat.bob))
    cheese = ExpenseItem(name='Cheese', unit_price=flat.euros(4200), split=split, quantity=Quantity.parse('0.35'))
    assert cheese.line_total() == flat.euros(1470)
    # 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
    olives = ExpenseItem(name='Olives', unit_price=flat.euros(100), split=split, quantity=Quantity.parse('0.335'))
    assert olives.line_total() == flat.euros(34)
    beers = ExpenseItem(name='Beer', unit_price=flat.euros(450), split=split, quantity=Quantity.whole(3))
    assert beers.line_total() == flat.euros(1350)
    # 0.0049 at 1.00 EUR is under half a cent, a free line.
    sample = ExpenseItem(name='Sample', unit_price=flat.euros(100), split=split, quantity=Quantity.parse('0.0049'))
    assert sample.line_total() == flat.euros(0)


def test_an_item_given_no_quantity_counts_one_unit(flat: Flat) -> None:
    """An item whose quantity defaulted otherwise than the library's would charge a line given no quantity for another number of units."""
    bread = ExpenseItem(name='Bread', unit_price=flat.euros(250), split=EqualSplit((flat.ana,)))
    assert bread.quantity == Quantity.ONE
    assert bread.line_total() == flat.euros(250)


def test_a_line_total_of_an_item_the_library_refuses_is_refused(flat: Flat) -> None:
    """A line total worked out for an item the group refuses would show a receipt line that no expense can hold."""
    wine = ExpenseItem(name='Wine', unit_price=flat.euros(600), split=AmountsSplit((MemberAmount(flat.ana, flat.euros(100)),)))
    with pytest.raises(ExpenseError) as unbalanced:
        wine.line_total()
    assert isinstance(unbalanced.value.failure, ExpenseError.AmountsDoNotAddUp)
    gold = ExpenseItem(name='Gold', unit_price=flat.euros(2**62), split=EqualSplit((flat.ana,)), quantity=Quantity.whole(4))
    with pytest.raises(ExpenseError) as overflow:
        gold.line_total()
    assert overflow.value.failure == ExpenseError.Overflow()


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class DiscountRefusal:
    """A discount a line cannot take, and the failure the library refuses it with."""

    discount: Discount
    """The discount."""

    failure: ExpenseFailure
    """The failure."""


def test_a_percentage_off_each_unit_is_rounded_per_unit_and_one_off_the_line_once(flat: Flat) -> None:
    """
    A discount rounded otherwise than the library's would show a receipt whose lines do not add up to what the group divides.

    Three lemonades at 3.33 with half off cost 4.98 with the discount off each unit, whose 1.665 off rounds to 1.67, and 4.99 with it off the
    line, whose 4.995 off rounds to 5.00.
    """
    half = PercentageDiscount(Percentage(5000))
    lemonades = ExpenseItem(name='Lemonade', unit_price=flat.euros(333), split=EqualSplit((flat.ana, flat.bob)), quantity=Quantity.whole(3))
    off_each_unit = copy.replace(lemonades, discount=Discount(half, DiscountScope.UNIT))
    assert off_each_unit.line_total_before_discount() == flat.euros(999)
    assert off_each_unit.discount_amount() == flat.euros(501)
    assert off_each_unit.line_total() == flat.euros(498)
    off_the_line = copy.replace(lemonades, discount=Discount(half, DiscountScope.LINE))
    assert off_the_line.line_total_before_discount() == flat.euros(999)
    assert off_the_line.discount_amount() == flat.euros(500)
    assert off_the_line.line_total() == flat.euros(499)


def test_a_fixed_discount_comes_off_each_unit_or_once_off_the_line(flat: Flat) -> None:
    """A fixed discount taken once where the bill takes it off each unit, or the reverse, would charge a line another amount than the library."""
    lemonades = ExpenseItem(name='Lemonade', unit_price=flat.euros(300), split=EqualSplit((flat.ana, flat.bob)), quantity=Quantity.whole(3))
    off_each_unit = copy.replace(lemonades, discount=Discount(FixedDiscount(flat.euros(100)), DiscountScope.UNIT))
    assert off_each_unit.line_total_before_discount() == flat.euros(900)
    assert off_each_unit.discount_amount() == flat.euros(300)
    assert off_each_unit.line_total() == flat.euros(600)
    off_the_line = copy.replace(lemonades, discount=Discount(FixedDiscount(flat.euros(100)), DiscountScope.LINE))
    assert off_the_line.line_total_before_discount() == flat.euros(900)
    assert off_the_line.discount_amount() == flat.euros(100)
    assert off_the_line.line_total() == flat.euros(800)


def test_a_percentage_off_a_weighed_line_is_taken_of_its_rounded_total(flat: Flat) -> None:
    """
    A weighed line whose discount the layer worked out itself would show a receipt line the group does not divide.

    A scale reports 0.35 kg as the float just below it, which the quantity reads in its shortest form, 0.35: at 42.00 EUR, 14.70 before 10 % off,
    1.47 off, and 13.23 to pay.
    """
    telemea = ExpenseItem(
        name='Telemea',
        unit_price=flat.euros(4200),
        split=EqualSplit((flat.ana, flat.bob)),
        quantity=Quantity.from_float(0.35),
        discount=Discount(PercentageDiscount(Percentage.parse('10'))),
    )
    assert telemea.line_total_before_discount() == flat.euros(1470)
    assert telemea.discount_amount() == flat.euros(147)
    assert telemea.line_total() == flat.euros(1323)


def test_a_discount_of_the_whole_price_makes_a_free_line_that_divides_to_zeros(flat: Flat) -> None:
    """
    A free line the library refused, or charged anyone for, would stop an app from entering a dish on the house.

    Its members share nothing of it, and a split of amounts on it is refused, since no amount above zero adds up to a line of zero.
    """
    on_the_house = ExpenseItem(
        name='Papanași', unit_price=flat.euros(900), split=EqualSplit((flat.bob,)), discount=Discount(PercentageDiscount(Percentage(10000)))
    )
    assert on_the_house.line_total_before_discount() == flat.euros(900)
    assert on_the_house.discount_amount() == flat.euros(900)
    assert on_the_house.line_total() == flat.euros(0)
    free_units = ExpenseItem(
        name='Water',
        unit_price=flat.euros(200),
        split=EqualSplit((flat.ana,)),
        quantity=Quantity.whole(2),
        discount=Discount(FixedDiscount(flat.euros(200)), DiscountScope.UNIT),
    )
    assert free_units.discount_amount() == flat.euros(400)
    assert free_units.line_total() == flat.euros(0)
    soup = ExpenseItem(name='Ciorbă', unit_price=flat.euros(3000), split=EqualSplit((flat.ana,)))
    dinner = Expense.create(
        name='Dinner',
        cost=LocalCost(flat.euros(3000)),
        payers=EqualSplit((flat.ana,)),
        split=ItemisedExpenseSplit((soup, on_the_house, free_units)),
        date=flat.day,
    )
    shares = flat.group.expense_shares(flat.group.add_expense(dinner))
    assert shares is not None
    assert {share.member: share.share for share in shares.members} == {flat.ana: flat.euros(3000), flat.bob: flat.euros(0)}
    by_amounts = copy.replace(on_the_house, split=AmountsSplit((MemberAmount(flat.bob, flat.euros(100)),)))
    with pytest.raises(ExpenseError) as refused:
        by_amounts.line_total()
    assert refused.value.failure == ExpenseError.AmountsDoNotAddUp(expected=flat.euros(0), actual=flat.euros(100))


def test_a_discount_the_line_cannot_take_is_refused_with_every_total(flat: Flat) -> None:
    """
    A discount the library refuses that an app could still show totals for would put on a receipt a line no expense can hold.

    Two loaves at 2.50 take at most 2.50 off each and 5.00 off the line; a percentage cannot pass 100 %, a fixed discount cannot be negative, and
    it is in the item's currency.
    """
    bread = ExpenseItem(name='Bread', unit_price=flat.euros(250), split=EqualSplit((flat.ana,)), quantity=Quantity.whole(2))
    refusals = [
        DiscountRefusal(discount=Discount(FixedDiscount(flat.euros(-1))), failure=ExpenseError.NegativeDiscount()),
        DiscountRefusal(
            discount=Discount(PercentageDiscount(Percentage(10001)), DiscountScope.UNIT), failure=ExpenseError.DiscountAboveHundredPercent()
        ),
        DiscountRefusal(discount=Discount(FixedDiscount(flat.euros(251)), DiscountScope.UNIT), failure=ExpenseError.DiscountAbovePrice()),
        DiscountRefusal(discount=Discount(FixedDiscount(flat.euros(501)), DiscountScope.LINE), failure=ExpenseError.DiscountAbovePrice()),
        DiscountRefusal(
            discount=Discount(FixedDiscount(Money(100, CurrencyCode('RON')))),
            failure=ExpenseError.CurrencyMismatch(expected=flat.euro, found=CurrencyCode('RON')),
        ),
    ]
    for refusal in refusals:
        item = copy.replace(bread, discount=refusal.discount)
        for total in (item.line_total_before_discount, item.discount_amount, item.line_total):
            with pytest.raises(ExpenseError) as refused:
                total()
            assert refused.value.failure == refusal.failure


def test_an_items_discount_reads_back_as_it_was_added_and_saved(flat: Flat) -> None:
    """An item whose discount came back from the group or a saved group otherwise than it went in would charge its line another amount."""
    pizza = ExpenseItem(
        name='Pizza',
        unit_price=flat.euros(1050),
        split=EqualSplit((flat.ana, flat.bob)),
        quantity=Quantity.whole(2),
        discount=Discount(PercentageDiscount(Percentage(1000))),
    )
    wine = ExpenseItem(
        name='Wine',
        unit_price=flat.euros(1200),
        split=EqualSplit((flat.ana,)),
        quantity=Quantity.parse('0.5'),
        discount=Discount(FixedDiscount(flat.euros(100)), DiscountScope.UNIT),
    )
    bread = ExpenseItem(name='Bread', unit_price=flat.euros(300), split=EqualSplit((flat.bob,)))
    # 21.00 less 2.10, 0.5 of 11.00, and 3.00 at full price.
    receipt = Expense.create(
        name='Receipt',
        cost=LocalCost(flat.euros(2740)),
        payers=EqualSplit((flat.ana,)),
        split=ItemisedExpenseSplit((pizza, wine, bread)),
        date=flat.day,
    )
    assert flat.group.add_expense(receipt) == receipt.id
    assert flat.group.expense(receipt.id) == receipt
    with ExpenseGroup.load(flat.group.save()) as loaded:
        assert loaded.expense(receipt.id) == receipt


def test_an_item_given_no_discount_is_at_full_price(flat: Flat) -> None:
    """An item whose discount defaulted to anything but none would take money off a line given no discount."""
    bread = ExpenseItem(name='Bread', unit_price=flat.euros(250), split=EqualSplit((flat.ana,)), quantity=Quantity.whole(2))
    assert bread.discount is None
    assert bread.discount_amount() == flat.euros(0)
    assert bread.line_total_before_discount() == bread.line_total()


def test_a_discount_without_a_scope_comes_off_the_line() -> None:
    """A discount whose scope defaulted otherwise than the library's would take a fixed discount off every unit of a Python app's line."""
    assert Discount(PercentageDiscount(Percentage(1000))).scope is DiscountScope.LINE


def test_a_tip_without_a_distribution_is_shared_in_proportion() -> None:
    """A tip whose sharing defaulted otherwise than the library's would divide the same bill differently in a Python app."""
    assert TipRule(PercentageTip(Percentage(1000))).distribution is TipDistribution.PROPORTIONAL


def test_the_no_tip_rule_is_zero_per_cent_shared_in_proportion() -> None:
    """An app that left an expense without a tip would give it a tip, or one shared otherwise than the library's own expense without one."""
    assert TipRule(PercentageTip(Percentage(0)), TipDistribution.PROPORTIONAL) == NO_TIP
