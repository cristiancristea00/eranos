"""Check the asynchronous form of every call, the read handle, and the cancellation of a call through its task."""

import asyncio
import copy

import pytest

from conftest import Flat
from eranos import (
    Category,
    CurrencyCode,
    CurrencyRate,
    ExchangeRate,
    ExpenseGroup,
    ExpenseGroupError,
    InvalidJsonError,
    Language,
    LocalCost,
    Member,
    MemberId,
    SettlementStrategy,
    Transfer,
)


def test_every_asynchronous_read_returns_what_its_synchronous_form_returns(flat: Flat) -> None:
    """An asynchronous read that returned otherwise than its synchronous form would give an app two answers to one question."""

    async def reads() -> None:
        group = flat.group
        assert await group.id_async() == group.id()
        assert await group.name_async() == group.name()
        assert await group.currency_async() == group.currency()
        assert await group.note_async() == group.note()
        assert await group.language_async() == group.language()
        assert await group.save_async() == group.save()
        assert await group.categories_async() == group.categories()
        assert await group.category_async(flat.food) == group.category(flat.food)
        assert await group.expenses_async() == group.expenses()
        assert await group.expense_async(flat.dinner) == group.expense(flat.dinner)
        assert await group.expense_shares_async(flat.dinner) == group.expense_shares(flat.dinner)
        assert await group.summary_async() == group.summary()
        assert await group.members_async() == group.members()
        assert await group.member_async(flat.ana) == group.member(flat.ana)
        assert await group.is_archived_async(flat.ana) == group.is_archived(flat.ana)
        assert await group.settlement_async(SettlementStrategy.GREEDY) == group.settlement(SettlementStrategy.GREEDY)
        assert await group.transfers_async() == group.transfers()
        assert await group.transfer_async(flat.repayment) == group.transfer(flat.repayment)
        assert await group.transfer_amount_async(flat.repayment) == group.transfer_amount(flat.repayment)

    asyncio.run(reads())


def test_every_asynchronous_change_of_an_entry_does_what_its_synchronous_form_does(flat: Flat) -> None:
    """
    An asynchronous change that did otherwise than its synchronous form would leave an app's entries in another state.

    A member it removed who still read as archived or active would stay on the app's list of members.
    """

    async def changes() -> None:
        group = flat.group
        lunch = await group.add_expense_async(flat.expense('Lunch', flat.bob, (flat.ana,)))
        added = group.expense(lunch)
        assert added is not None
        brunch = copy.replace(added, name='Brunch')
        assert await group.replace_expense_async(brunch) == added
        assert await group.remove_expense_async(lunch) == brunch
        cash = await group.add_transfer_async(Transfer.create(sender=flat.ana, receiver=flat.bob, cost=LocalCost(flat.euros(100)), date=flat.day))
        sent = group.transfer(cash)
        assert sent is not None
        coins = copy.replace(sent, note='Coins')
        assert await group.replace_transfer_async(coins) == sent
        assert await group.remove_transfer_async(cash) == coins
        carla = await group.add_member_async(Member.create(first_name='Carla'))
        joined = group.member(carla)
        assert joined is not None
        maria = copy.replace(joined, first_name='Carla Maria')
        assert await group.replace_member_async(maria) == joined
        await group.archive_member_async(carla)
        assert group.is_archived(carla) is True
        assert await group.is_archived_async(carla) is True
        await group.unarchive_member_async(carla)
        assert group.is_archived(carla) is False
        assert await group.is_archived_async(carla) is False
        assert await group.remove_member_async(carla) == maria
        assert group.is_archived(carla) is None
        assert await group.is_archived_async(carla) is None
        drinks = await group.add_category_async(Category.create(name='Drinks'))
        coffee = Category(id=drinks, name='Coffee')
        assert await group.replace_category_async(coffee) == Category(id=drinks, name='Drinks')
        assert await group.remove_category_async(drinks) == coffee

    asyncio.run(changes())


def test_every_asynchronous_change_of_the_details_does_what_its_synchronous_form_does(flat: Flat) -> None:
    """An asynchronous change of the group's details that did otherwise than its synchronous form would show an app another group."""

    async def changes() -> None:
        group = flat.group
        await group.set_name_async('Home')
        await group.set_note_async('Bills')
        await group.set_language_async(Language('ro-RO'))
        assert group.name() == 'Home'
        assert group.note() == 'Bills'
        assert group.language() == Language('ro-RO')
        await group.set_note_async(None)
        await group.set_language_async(None)
        assert group.note() is None
        assert group.language() is None
        await group.set_note_async(' Rent\n')
        assert group.note() == 'Rent'
        await group.set_note_async(' ')
        assert group.note() is None
        await group.change_currency_async(CurrencyCode('RON'), [CurrencyRate(flat.euro, ExchangeRate('5'))])
        assert group.currency() == CurrencyCode('RON')
        with await ExpenseGroup.load_async(group.save()) as loaded:
            assert loaded.save() == group.save()

    asyncio.run(changes())


def test_an_asynchronous_call_reports_the_error_its_synchronous_form_raises(flat: Flat) -> None:
    """An asynchronous call that swallowed its failure would leave an app waiting for a change that never happened."""
    nobody = MemberId('0199a000-0000-7000-8000-00000000ffff')

    async def calls() -> None:
        with pytest.raises(ExpenseGroupError) as removing:
            await flat.group.remove_member_async(nobody)
        assert removing.value.failure == ExpenseGroupError.UnknownMember(member=nobody)
        with pytest.raises(ExpenseGroupError) as archiving:
            await flat.group.archive_member_async(nobody)
        assert archiving.value.failure == ExpenseGroupError.UnknownMember(member=nobody)
        with pytest.raises(InvalidJsonError):
            await ExpenseGroup.load_async('{')

    asyncio.run(calls())


def test_a_reader_reads_the_group_until_it_is_released(flat: Flat) -> None:
    """
    A read handle that let a change in, or read after its release, would show a report two states of one group.

    One that read a member the group does not hold as active would list in the report a member the group refuses.
    """
    group = flat.group
    group.archive_member(flat.bob)
    with group.read() as reader:
        assert reader.id() == group.id()
        assert reader.name() == 'Flat'
        assert reader.currency() == flat.euro
        assert reader.note() == 'Rent'
        assert reader.language() == Language('pt-PT')
        assert reader.save() == group.save()
        assert reader.categories() == group.categories()
        assert reader.category(flat.food) == group.category(flat.food)
        assert reader.expenses() == group.expenses()
        assert reader.expense(flat.dinner) == group.expense(flat.dinner)
        assert reader.expense_shares(flat.dinner) == group.expense_shares(flat.dinner)
        assert reader.summary() == group.summary()
        assert reader.members() == group.members()
        assert reader.member(flat.ana) == group.member(flat.ana)
        assert reader.is_archived(flat.ana) is False
        assert reader.is_archived(flat.bob) is True
        assert reader.is_archived(MemberId('0199a000-0000-7000-8000-00000000ffff')) is None
        assert reader.settlement(SettlementStrategy.DIRECT) == group.settlement(SettlementStrategy.DIRECT)
        assert reader.transfers() == group.transfers()
        assert reader.transfer(flat.repayment) == group.transfer(flat.repayment)
        assert reader.transfer_amount(flat.repayment) == flat.euros(1000)
    with pytest.raises(RuntimeError, match='released'):
        reader.name()

    async def read() -> str:
        with await group.read_async() as asynchronous:
            return asynchronous.name()

    assert asyncio.run(read()) == 'Flat'


def test_a_change_waiting_behind_a_reader_is_cancelled_and_leaves_the_group_as_it_was(flat: Flat) -> None:
    """
    A cancelled change that still ran would change the group after the app gave it up.

    One that kept waiting would hold the lock's queue, and every later read would wait behind it.
    """

    async def change() -> None:
        reader = await flat.group.read_async()
        changing = asyncio.create_task(
            flat.group.change_currency_async(CurrencyCode('RON'), [CurrencyRate(flat.euro, ExchangeRate('5'))]), eager_start=True
        )
        await asyncio.sleep(0)
        assert not changing.done()
        changing.cancel()
        with pytest.raises(asyncio.CancelledError):
            await changing
        reader.release()

    asyncio.run(change())
    assert flat.group.currency() == flat.euro
