"""
Print section 14 of the tour: the group shared between tasks, and a change that waits for a report's read and is cancelled.

The section runs in an event loop and makes every call through its coroutine, the form whose name ends in ``_async``.

"""

import asyncio
import contextlib

from eranos import (
    Currency,
    CurrencyRate,
    EqualSplit,
    ExchangeRate,
    Expense,
    ExpenseGroup,
    ExpenseGroupReader,
    LocalCost,
    MemberId,
    SettlementStrategy,
    WholeExpenseSplit,
)
from examples.grand_tour.tour import PaymentRow, People, october, ron
from examples.report import print_lines, print_table, section


def _first_name(reader: ExpenseGroupReader, member: MemberId) -> str:
    """
    Read the first name of member through reader.

    Args:
        reader: The read handle.
        member: The member.

    Returns:
        The first name, or ``?`` for a member the group does not hold.

    """
    found = reader.member(member)
    return '?' if found is None else found.first_name


async def sharing(group: ExpenseGroup, people: People) -> None:
    """
    Print section 14: share the group between tasks, and cancel a change that waits for a report's read.

    Args:
        group: The group.
        people: The members of the trip.

    """
    section('14. Sharing the group')
    # Another handle on the same group, as another task holds: a group is shared as it is.
    other = group
    everyone = (people.ana, people.bob, people.carol, people.dan, people.eva)
    await other.add_expense_async(
        Expense(
            name='Airport coffee',
            cost=LocalCost(ron('75.00')),
            payers=EqualSplit((people.bob,)),
            split=WholeExpenseSplit(EqualSplit(everyone)),
            date=october(8),
        )
    )
    plan = await group.settlement_async(SettlementStrategy.MINIMUM_TRANSFERS)
    print_lines(
        'An app shares the group between its tasks and threads through a',
        "SharedExpenseGroup, whose calls are futures any executor runs; here the library's",
        'block_on runs each one on this thread. Bob adds the airport coffee through another',
        'handle, and the plan that settles it comes through the first:',
    )
    with await group.read_async() as reader:
        print_table(
            PaymentRow,
            [
                PaymentRow(from_=_first_name(reader, payment.sender), to=_first_name(reader, payment.receiver), amount=payment.amount)
                for payment in plan.payments
            ],
            2,
        )

    before = await group.summary_async()
    rates = (CurrencyRate(Currency('RON'), ExchangeRate('0.2')),)
    # A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
    # behind the change, so the report reads through its own handle.
    report = await group.read_async()
    change = asyncio.create_task(group.change_currency_async(Currency('EUR'), rates), eager_start=True)
    await asyncio.sleep(0)
    waited = not change.done()
    reported = report.summary().total_spent
    # Cancelling the task cancels the waiting change, which leaves the group as it was.
    change.cancel()
    with contextlib.suppress(asyncio.CancelledError):
        await change
    report.release()
    after = await group.summary_async()
    currency = await group.currency_async()
    print_lines(
        'While a report reads the group, a change of its currency to EUR waits for the',
        'report. The app drops the waiting change, which cancels it: the group still',
        f'keeps its books in {currency}, and the trip cost {after.total_spent}, as the report read.',
    )
    returned = await group.expenses_async()
    print_lines(f'With the other handle gone, the app takes the group back: {len(returned)} expenses.')

    assert len(plan.payments) == 4
    for payment in plan.payments:
        assert payment.receiver == people.bob
        assert payment.amount == ron('15.00')
    assert waited
    assert change.cancelled()
    assert after == before
    assert reported == after.total_spent
    assert currency == Currency('RON')
