"""Check the group's own calls: its details, totals, settlements, currency change, and saved form, and what its errors hold."""

import copy
import json

import pytest

from conftest import Flat
from eranos import (
    AmountsSplit,
    Balance,
    CategoryId,
    CategoryTotal,
    ConvertedCost,
    CurrencyCode,
    CurrencyRate,
    ExchangeRate,
    ExpenseError,
    ExpenseId,
    GetsBackBalance,
    ExpenseGroup,
    ExpenseGroupError,
    ExpenseGroupId,
    IdError,
    InvalidJsonError,
    Language,
    LocalCost,
    MemberAmount,
    MemberId,
    MemberPercentage,
    Money,
    Payment,
    Percentage,
    PercentagesSplit,
    SettledBalance,
    SettlementStrategy,
    Transfer,
    TransferId,
    WholeExpenseSplit,
    Category,
    parse_expense_group_id,
)


def test_a_group_holds_its_details_and_changes_them(flat: Flat) -> None:
    """
    A group whose details did not change as asked would show an app a name, a description, or a language the user replaced.

    A description is trimmed, and none or blank text clears it, so the group never holds an empty one.
    """
    assert flat.group.id()
    assert flat.group.name() == 'Flat'
    assert flat.group.currency() == flat.euro
    assert flat.group.description() == 'Rent'
    assert flat.group.language() == Language('pt-PT')
    flat.group.set_name('Home')
    flat.group.set_description(' Rent and bills\n')
    flat.group.set_language(Language('ro-RO'))
    assert flat.group.name() == 'Home'
    assert flat.group.description() == 'Rent and bills'
    assert flat.group.language() == Language('ro-RO')
    flat.group.set_description(None)
    flat.group.set_language(None)
    assert flat.group.description() is None
    assert flat.group.language() is None
    flat.group.set_description('Rent')
    flat.group.set_description(' ')
    assert flat.group.description() is None
    with pytest.raises(ExpenseGroupError) as blank:
        flat.group.set_name(' ')
    assert blank.value.failure == ExpenseGroupError.EmptyName()
    with ExpenseGroup.create('Trip', flat.euro) as trip, ExpenseGroup.create('Trip', flat.euro, description=' ') as blank_description:
        assert trip.description() is None
        assert trip.language() is None
        assert blank_description.description() is None
    with pytest.raises(ExpenseGroupError) as nameless:
        ExpenseGroup.create(' ', flat.euro)
    assert nameless.value.failure == ExpenseGroupError.EmptyName()


def test_a_group_keeps_the_identifier_its_app_gives_it(flat: Flat) -> None:
    """A group that lost the identifier its app gave it, or took one the library refuses, would be saved under another key than the app's."""
    trip_id = parse_expense_group_id('trip-12')
    with ExpenseGroup.create_with_id(trip_id, 'Trip', flat.euro, description='Porto', language=Language('pt-pt')) as trip:
        assert trip.id() == trip_id
        assert trip.description() == 'Porto'
        assert trip.language() == Language('pt-PT')
        with ExpenseGroup.load(trip.save()) as loaded:
            assert loaded.id() == trip_id
    assert flat.group.id() != trip_id
    with pytest.raises(IdError) as refused:
        ExpenseGroup.create_with_id(ExpenseGroupId('trip\n12'), 'Trip', flat.euro)
    assert refused.value.failure == IdError.ControlCharacter()


def test_create_makes_each_group_under_a_new_identifier(flat: Flat) -> None:
    """A create that reused an identifier would save a new group over another one the app keeps under that identifier."""
    with ExpenseGroup.create('Trip', flat.euro) as trip, ExpenseGroup.create('Trip', flat.euro) as again:
        assert flat.group.id() < trip.id() < again.id()
        assert parse_expense_group_id(trip.id()) == trip.id()


def test_the_summary_and_the_shares_add_up(flat: Flat) -> None:
    """Totals that did not add up would tell a member they owe what they do not."""
    summary = flat.group.summary()
    assert summary.total_spent == flat.euros(3000)
    ana = next(member for member in summary.members if member.member == flat.ana)
    assert ana.paid == flat.euros(3000)
    assert ana.share == flat.euros(1500)
    assert ana.sent == flat.euros(0)
    assert ana.received == flat.euros(1000)
    assert ana.net == flat.euros(500)
    assert ana.balance == GetsBackBalance(flat.euros(500))
    assert summary.category_totals == (CategoryTotal(flat.food, flat.euros(3000)),)
    assert len(summary.member_category_totals) == 2
    assert summary.member_category_totals[0].totals[0].category == flat.food
    shares = flat.group.expense_shares(flat.dinner)
    assert shares is not None
    assert len(shares.members) == 2
    assert shares.total == flat.euros(3000)
    assert shares.tip == flat.euros(0)
    assert flat.group.expense_shares(ExpenseId('0199a000-0000-7000-8000-00000000ffff')) is None


def test_each_strategy_settles_every_balance(flat: Flat) -> None:
    """A strategy that settled otherwise than the library would ask a member to pay twice."""
    for strategy in SettlementStrategy:
        settlement = flat.group.settlement(strategy)
        # The flat's two open balances are well within the adaptive strategy's limit, so it settles them exactly.
        assert settlement.strategy == (SettlementStrategy.MINIMUM_TRANSFERS if strategy is SettlementStrategy.ADAPTIVE else strategy)
        assert settlement.payments == (Payment(sender=flat.bob, receiver=flat.ana, amount=flat.euros(500)),)
    flat.group.add_transfer(Transfer.create(sender=flat.bob, receiver=flat.ana, cost=LocalCost(flat.euros(500)), date=flat.day))
    settled: Balance = SettledBalance()
    assert all(member.balance == settled for member in flat.group.summary().members)


def test_a_change_of_currency_converts_every_entry(flat: Flat) -> None:
    """A change of currency that left an entry unconverted, or took a rate it was not given, would mix currencies in the totals."""
    lei = CurrencyCode('RON')
    with pytest.raises(ExpenseGroupError) as missing:
        flat.group.change_currency(lei, [])
    assert missing.value.failure == ExpenseGroupError.MissingExchangeRate(currency=flat.euro)
    rate = CurrencyRate(flat.euro, ExchangeRate('5'))
    with pytest.raises(ExpenseGroupError) as twice:
        flat.group.change_currency(lei, [rate, rate])
    assert twice.value.failure == ExpenseGroupError.DuplicateExchangeRate(currency=flat.euro)
    converted = copy.replace(flat.expense('Taxi', flat.ana, (flat.ana,)), cost=ConvertedCost(flat.euros(1000), ExchangeRate('1')))
    with pytest.raises(ExpenseGroupError) as unexpected:
        flat.group.add_expense(converted)
    assert unexpected.value.failure == ExpenseGroupError.UnexpectedExchangeRate()
    with pytest.raises(ExpenseGroupError) as unconverted:
        flat.group.add_expense(copy.replace(converted, cost=LocalCost(Money(1000, lei))))
    assert unconverted.value.failure == ExpenseGroupError.MissingExchangeRate(currency=lei)
    flat.group.change_currency(lei, (rate,))
    assert flat.group.currency() == lei
    assert flat.group.summary().total_spent == Money(15000, lei)


def test_a_saved_group_loads_as_it_was_saved(flat: Flat) -> None:
    """A saved group that loaded otherwise than it was saved would lose entries between two runs of an app."""
    saved = flat.group.save()
    with ExpenseGroup.load(saved) as loaded:
        assert loaded.save() == saved
    with pytest.raises(InvalidJsonError) as broken:
        ExpenseGroup.load('{')
    assert broken.value.line == 1
    assert broken.value.column == 1
    newer = json.loads(saved) | {'format': 4}
    with pytest.raises(ExpenseGroupError) as unsupported:
        ExpenseGroup.load(json.dumps(newer))
    assert unsupported.value.failure == ExpenseGroupError.UnsupportedFormat(format=4)
    assert str(unsupported.value) == 'unsupported group format 4: this version reads format 3'


def test_every_error_holds_what_its_failure_holds(flat: Flat) -> None:
    """An error that lost what it holds would leave an app unable to say which entry, amount, or category the library refused."""
    uneven = copy.replace(flat.expense('Uneven', flat.ana, ()), split=WholeExpenseSplit(AmountsSplit((MemberAmount(flat.ana, flat.euros(1000)),))))
    foreign = copy.replace(uneven, split=WholeExpenseSplit(AmountsSplit((MemberAmount(flat.ana, Money(3000, CurrencyCode('RON'))),))))
    with pytest.raises(ExpenseError) as currencies:
        flat.group.add_expense(foreign)
    assert currencies.value.failure == ExpenseError.CurrencyMismatch(expected=flat.euro, found=CurrencyCode('RON'))
    with pytest.raises(ExpenseError) as amounts:
        flat.group.add_expense(uneven)
    assert amounts.value.failure == ExpenseError.AmountsDoNotAddUp(expected=flat.euros(3000), actual=flat.euros(1000))
    halves = copy.replace(uneven, split=WholeExpenseSplit(PercentagesSplit((MemberPercentage(flat.ana, Percentage(5000)),))))
    with pytest.raises(ExpenseError) as percentages:
        flat.group.add_expense(halves)
    assert percentages.value.failure == ExpenseError.PercentagesDoNotAddUp(total_basis_points=5000)
    with pytest.raises(ExpenseGroupError) as duplicate:
        flat.group.add_category(Category.create(name='food'))
    assert duplicate.value.failure == ExpenseGroupError.DuplicateCategoryName(existing=flat.food)
    absent = CategoryId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(ExpenseGroupError) as category:
        flat.group.add_expense(copy.replace(flat.expense('Taxi', flat.ana, (flat.ana,)), category=absent))
    assert category.value.failure == ExpenseGroupError.UnknownCategory(category=absent)
    nothing = ExpenseId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(ExpenseGroupError) as expense:
        flat.group.remove_expense(nothing)
    assert expense.value.failure == ExpenseGroupError.UnknownExpense(expense=nothing)
    nowhere = TransferId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(ExpenseGroupError) as transfer:
        flat.group.remove_transfer(nowhere)
    assert transfer.value.failure == ExpenseGroupError.UnknownTransfer(transfer=nowhere)
    refused = {'': IdError.Empty(), 'a' * 65: IdError.TooLong(), 'a\tb': IdError.ControlCharacter(), ' ana': IdError.SurroundingWhitespace()}
    for text, failure in refused.items():
        with pytest.raises(IdError) as identifier:
            flat.group.member(MemberId(text))
        assert identifier.value.failure == failure, text


def test_text_that_is_not_a_uuid_is_an_identifier_the_group_does_not_hold(flat: Flat) -> None:
    """A group that refused text that is not a UUID would raise where an app looks up an identifier it does not hold, instead of finding nothing."""
    for text in ('not a uuid', 'ană', 'Ana and Bob', 'a' * 64):
        assert flat.group.member(MemberId(text)) is None, text


def test_a_freed_group_refuses_every_call(flat: Flat) -> None:
    """A group that answered after it was freed would read the library's memory of another."""
    with ExpenseGroup.create('Trip', flat.euro) as trip:
        assert trip.name() == 'Trip'
    with pytest.raises(RuntimeError, match='freed'):
        trip.name()
