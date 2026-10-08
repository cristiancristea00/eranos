"""Check that every entry and every variant goes through the group and comes back as it went in, and what the group refuses."""

import copy

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
    Email,
    EqualSplit,
    ExchangeRate,
    Expense,
    ExpenseId,
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
    PercentageTip,
    PhoneNumber,
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
    pizza = ExpenseItem(name='Pizza', unit_price=flat.euros(1200), split=EqualSplit((flat.ana, flat.bob)), quantity=2)
    wine = ExpenseItem(name='Wine', unit_price=flat.euros(600), split=PercentagesSplit((MemberPercentage(flat.ana, Percentage(10000)),)))
    receipt = Expense.create(
        name='Receipt',
        cost=LocalCost(flat.euros(3000)),
        payers=AmountsSplit((MemberAmount(flat.bob, flat.euros(3300)),)),
        split=ItemisedExpenseSplit((pizza, wine)),
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
    assert flat.group.is_archived(flat.bob)
    assert not flat.group.is_archived(flat.ana)
    with pytest.raises(ExpenseGroupError) as refused:
        flat.group.add_expense(flat.expense('Snack', flat.ana, (flat.ana, flat.bob)))
    assert refused.value.failure == ExpenseGroupError.ArchivedMember(member=flat.bob)
    flat.group.unarchive_member(flat.bob)
    assert not flat.group.is_archived(flat.bob)
    nobody = MemberId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(ExpenseGroupError) as unknown:
        flat.group.archive_member(nobody)
    assert unknown.value.failure == ExpenseGroupError.UnknownMember(member=nobody)


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


def test_a_tip_without_a_distribution_is_shared_in_proportion() -> None:
    """A tip whose sharing defaulted otherwise than the library's would divide the same bill differently in a Python app."""
    assert TipRule(PercentageTip(Percentage(1000))).distribution is TipDistribution.PROPORTIONAL


def test_the_no_tip_rule_is_zero_per_cent_shared_in_proportion() -> None:
    """An app that left an expense without a tip would give it a tip, or one shared otherwise than the library's own expense without one."""
    assert TipRule(PercentageTip(Percentage(0)), TipDistribution.PROPORTIONAL) == NO_TIP
