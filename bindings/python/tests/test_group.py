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
    Currency,
    CurrencyRate,
    ExchangeRate,
    ExpenseError,
    ExpenseId,
    GetsBackBalance,
    Group,
    GroupError,
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
)


def test_a_group_holds_its_details_and_changes_them(flat: Flat) -> None:
    """A group whose details did not change as asked would show an app a name or a language the user replaced."""
    assert flat.group.id()
    assert flat.group.name() == 'Flat'
    assert flat.group.currency() == flat.euro
    assert flat.group.description() == 'Rent'
    assert flat.group.language() == Language('pt-PT')
    flat.group.set_name('Home')
    flat.group.set_description('Rent and bills')
    flat.group.set_language(Language('ro-RO'))
    assert flat.group.name() == 'Home'
    assert flat.group.description() == 'Rent and bills'
    assert flat.group.language() == Language('ro-RO')
    flat.group.set_description('')
    flat.group.set_language(None)
    assert flat.group.description() is None
    assert flat.group.language() is None
    with pytest.raises(GroupError) as blank:
        flat.group.set_name(' ')
    assert blank.value.failure == GroupError.EmptyName()
    with Group.create(flat.euro, 'Trip') as trip:
        assert trip.description() is None
        assert trip.language() is None
    with pytest.raises(GroupError) as nameless:
        Group.create(flat.euro, ' ')
    assert nameless.value.failure == GroupError.EmptyName()


def test_the_summary_and_the_shares_add_up(flat: Flat) -> None:
    """Totals that did not add up would tell a member they owe what they do not."""
    summary = flat.group.summary()
    assert summary.total_spent == flat.euros(3000)
    assert summary.category_totals == (CategoryTotal(flat.food, flat.euros(3000)),)
    ana = next(member for member in summary.members if member.member == flat.ana)
    assert ana.paid == flat.euros(3000)
    assert ana.share == flat.euros(1500)
    assert ana.received == flat.euros(1000)
    assert ana.sent == flat.euros(0)
    assert ana.net == flat.euros(500)
    assert ana.balance == GetsBackBalance(flat.euros(500))
    assert len(summary.member_category_totals) == 2
    assert summary.member_category_totals[0].totals[0].category == flat.food
    shares = flat.group.expense_shares(flat.dinner)
    assert shares is not None
    assert shares.total == flat.euros(3000)
    assert shares.tip == flat.euros(0)
    assert len(shares.members) == 2
    assert flat.group.expense_shares(ExpenseId('0199a000-0000-7000-8000-00000000ffff')) is None


def test_each_strategy_settles_every_balance(flat: Flat) -> None:
    """A strategy that settled otherwise than the library would ask a member to pay twice."""
    for strategy in SettlementStrategy:
        settlement = flat.group.settlement(strategy)
        # The flat's two open balances are well within the adaptive strategy's limit, so it settles them exactly.
        assert settlement.strategy == (SettlementStrategy.MINIMUM_TRANSFERS if strategy is SettlementStrategy.ADAPTIVE else strategy)
        assert settlement.payments == (Payment(amount=flat.euros(500), receiver=flat.ana, sender=flat.bob),)
    flat.group.add_transfer(Transfer(cost=LocalCost(flat.euros(500)), date=flat.day, receiver=flat.ana, sender=flat.bob))
    settled: Balance = SettledBalance()
    assert all(member.balance == settled for member in flat.group.summary().members)


def test_a_change_of_currency_converts_every_entry(flat: Flat) -> None:
    """A change of currency that left an entry unconverted, or took a rate it was not given, would mix currencies in the totals."""
    lei = Currency('RON')
    with pytest.raises(GroupError) as missing:
        flat.group.change_currency(lei, [])
    assert missing.value.failure == GroupError.MissingExchangeRate(currency=flat.euro)
    rate = CurrencyRate(flat.euro, ExchangeRate('5'))
    with pytest.raises(GroupError) as twice:
        flat.group.change_currency(lei, [rate, rate])
    assert twice.value.failure == GroupError.DuplicateExchangeRate(currency=flat.euro)
    converted = copy.replace(flat.expense('Taxi', flat.ana, (flat.ana,)), cost=ConvertedCost(flat.euros(1000), ExchangeRate('1')))
    with pytest.raises(GroupError) as unexpected:
        flat.group.add_expense(converted)
    assert unexpected.value.failure == GroupError.UnexpectedExchangeRate()
    with pytest.raises(GroupError) as unconverted:
        flat.group.add_expense(copy.replace(converted, cost=LocalCost(Money(lei, 1000))))
    assert unconverted.value.failure == GroupError.MissingExchangeRate(currency=lei)
    flat.group.change_currency(lei, (rate,))
    assert flat.group.currency() == lei
    assert flat.group.summary().total_spent == Money(lei, 15000)


def test_a_saved_group_loads_as_it_was_saved(flat: Flat) -> None:
    """A saved group that loaded otherwise than it was saved would lose entries between two runs of an app."""
    saved = flat.group.save()
    with Group.load(saved) as loaded:
        assert loaded.save() == saved
    with pytest.raises(InvalidJsonError) as broken:
        Group.load('{')
    assert broken.value.line == 1
    assert broken.value.column == 1
    newer = json.loads(saved) | {'format': 26}
    with pytest.raises(GroupError) as unsupported:
        Group.load(json.dumps(newer))
    assert unsupported.value.failure == GroupError.UnsupportedFormat(format=26)


def test_every_error_holds_what_its_failure_holds(flat: Flat) -> None:
    """An error that lost what it holds would leave an app unable to say which entry, amount, or category the library refused."""
    uneven = copy.replace(flat.expense('Uneven', flat.ana, ()), split=WholeExpenseSplit(AmountsSplit((MemberAmount(flat.euros(1000), flat.ana),))))
    with pytest.raises(ExpenseError) as amounts:
        flat.group.add_expense(uneven)
    assert amounts.value.failure == ExpenseError.AmountsDoNotAddUp(actual=flat.euros(1000), expected=flat.euros(3000))
    foreign = copy.replace(uneven, split=WholeExpenseSplit(AmountsSplit((MemberAmount(Money(Currency('RON'), 3000), flat.ana),))))
    with pytest.raises(ExpenseError) as currencies:
        flat.group.add_expense(foreign)
    assert currencies.value.failure == ExpenseError.CurrencyMismatch(expected=flat.euro, found=Currency('RON'))
    halves = copy.replace(uneven, split=WholeExpenseSplit(PercentagesSplit((MemberPercentage(flat.ana, Percentage(5000)),))))
    with pytest.raises(ExpenseError) as percentages:
        flat.group.add_expense(halves)
    assert percentages.value.failure == ExpenseError.PercentagesDoNotAddUp(total_basis_points=5000)
    with pytest.raises(GroupError) as duplicate:
        flat.group.add_category(Category(name='food'))
    assert duplicate.value.failure == GroupError.DuplicateCategoryName(existing=flat.food)
    absent = CategoryId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(GroupError) as category:
        flat.group.add_expense(copy.replace(flat.expense('Taxi', flat.ana, (flat.ana,)), category=absent))
    assert category.value.failure == GroupError.UnknownCategory(category=absent)
    nothing = ExpenseId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(GroupError) as expense:
        flat.group.remove_expense(nothing)
    assert expense.value.failure == GroupError.UnknownExpense(expense=nothing)
    nowhere = TransferId('0199a000-0000-7000-8000-00000000ffff')
    with pytest.raises(GroupError) as transfer:
        flat.group.remove_transfer(nowhere)
    assert transfer.value.failure == GroupError.UnknownTransfer(transfer=nowhere)
    with pytest.raises(IdError) as identifier:
        flat.group.member(MemberId('not an identifier'))
    assert identifier.value.failure == IdError.InvalidId()


def test_a_freed_group_refuses_every_call(flat: Flat) -> None:
    """A group that answered after it was freed would read the library's memory of another."""
    with Group.create(flat.euro, 'Trip') as trip:
        assert trip.name() == 'Trip'
    with pytest.raises(RuntimeError, match='freed'):
        trip.name()
