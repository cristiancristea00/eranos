"""Print sections 8 to 11 of the tour: the summary, removing a category, a new group currency, and settling up."""

import dataclasses
import functools
from collections.abc import Iterable
from typing import ClassVar, assert_never, final, override

from eranos import (
    CategoryId,
    CategoryTotal,
    CurrencyCode,
    CurrencyRate,
    ExchangeRate,
    GetsBackBalance,
    ExpenseGroup,
    ExpenseGroupError,
    LocalCost,
    Member,
    MemberId,
    MemberSummary,
    Money,
    OwesBalance,
    SettledBalance,
    SettlementStrategy,
    Summary,
    Transfer,
)
from examples.grand_tour.tour import (
    Categories,
    PaymentRow,
    People,
    built_in_category,
    eur,
    name,
    october,
    print_balances,
    print_categories,
    print_entries,
    ron,
)
from examples.report import Row, message, print_lines, print_table, refusal, section, totals


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class MemberCategoryRow(Row):
    """A row of the table of each member's consumption per category."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Food', 'Lodging', 'Getting around', 'Uncategorised')

    name: str
    """The member's first name."""

    food: Money
    """What the member consumed in Food."""

    lodging: Money
    """What the member consumed in Lodging."""

    transport: Money
    """What the member consumed in Getting around, which the group added as Transport."""

    uncategorised: Money
    """What the member consumed in the built-in category."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, str(self.food), str(self.lodging), str(self.transport), str(self.uncategorised)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class StrategyRow(Row):
    """A row of the table of the strategies: a strategy, the strategy the settlement names, and the number of payments of its plan."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Strategy', 'Computed by', 'Payments')

    strategy: str
    """The name of the strategy asked for."""

    computed_by: str
    """The name of the strategy the settlement names: the one asked for, or the one whose plan the adaptive strategy chose."""

    payments: int
    """The number of payments of the settlement's plan."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.strategy, self.computed_by, str(self.payments)


def _add_up(currency: CurrencyCode, amounts: Iterable[Money]) -> Money:
    """
    Add up amounts, all in currency, with the library's checks.

    Args:
        currency: The currency of every amount.
        amounts: The amounts.

    Returns:
        The sum, zero for no amounts.

    """
    return functools.reduce(Money.plus, amounts, Money(0, currency))


def _member_totals(summary: Summary, member: MemberId) -> MemberSummary:
    """
    Find the totals of member, whom an entry names, in summary.

    Args:
        summary: The group's summary.
        member: The member.

    Returns:
        The member's totals.

    """
    found = totals(member, summary)
    assert found is not None, 'the summary lists every member an entry names'
    return found


def _by_category(summary: Summary, member: MemberId) -> tuple[CategoryTotal, ...]:
    """
    Find what member consumed in each category.

    Args:
        summary: The group's summary.
        member: The member.

    Returns:
        The member's totals per category, none for a member the summary leaves out.

    """
    return next((entry.totals for entry in summary.member_category_totals if entry.member == member), ())


def _total(category_totals: Iterable[CategoryTotal], category: CategoryId) -> Money | None:
    """
    Find the total of category among category_totals.

    Args:
        category_totals: The totals per category.
        category: The category.

    Returns:
        The category's total, or none for a category they leave out.

    """
    return next((entry.total for entry in category_totals if entry.category == category), None)


def _consumption_row(member: Member, summary: Summary, categories: Categories, built_in: CategoryId, zero: Money) -> MemberCategoryRow:
    """
    Write the row of member among the consumptions per category.

    Args:
        member: The member.
        summary: The group's summary.
        categories: The categories the group adds.
        built_in: The built-in category.
        zero: Zero in the group currency, for a category the member consumed nothing in.

    Returns:
        The row: the member's first name and what they consumed in each category.

    """
    own = () if member.id is None else _by_category(summary, member.id)
    return MemberCategoryRow(
        name=member.first_name,
        food=_total(own, categories.food) or zero,
        lodging=_total(own, categories.lodging) or zero,
        transport=_total(own, categories.transport) or zero,
        uncategorised=_total(own, built_in) or zero,
    )


def summary(group: ExpenseGroup, people: People, categories: Categories) -> None:
    """
    Print section 8: what the group spent, each member's totals, and the spending per category.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.

    """
    section('8. The summary')
    summary = group.summary()
    print_lines(
        f'The group spent {summary.total_spent} in all. For each member, what they paid, what they',
        'consumed, the transfers they sent and received, and what is left:',
    )
    print_balances(group)
    print_lines('The spending per category:')
    print_categories(group)
    print_lines("And each member's consumption per category:")
    built_in = built_in_category(group)
    zero = Money(0, group.currency())
    print_table(MemberCategoryRow, [_consumption_row(member, summary, categories, built_in, zero) for member in group.members()], 1)

    assert summary.total_spent == eur('1222.90')
    ana = _member_totals(summary, people.ana)
    assert ana.paid == eur('890.00')
    assert ana.received == eur('140.00')
    assert _member_totals(summary, people.bob).sent == eur('20.00')
    assert _member_totals(summary, people.dan).balance == GetsBackBalance(eur('12.55'))
    assert _member_totals(summary, people.eva).balance == OwesBalance(eur('42.60'))
    assert _total(summary.category_totals, categories.food) == eur('174.40')
    assert _total(summary.category_totals, categories.lodging) == eur('800.00')
    assert _total(summary.category_totals, categories.transport) == eur('188.50')
    assert _total(summary.category_totals, built_in) == eur('60.00')
    for found in summary.members:
        assert _add_up(zero.currency, (entry.total for entry in _by_category(summary, found.member))) == found.share
    assert _add_up(zero.currency, (found.net for found in summary.members)).minor_units == 0


def removing_a_category(group: ExpenseGroup, categories: Categories) -> None:
    """
    Print section 9: remove a category, whose expenses move to the built-in one, which cannot be removed.

    Args:
        group: The group.
        categories: The categories the group adds.

    """
    section('9. Removing a category')
    removed = group.remove_category(categories.transport)
    uncategorised = built_in_category(group)
    built_in = refusal(lambda: group.remove_category(uncategorised))
    print_lines(
        f'Removing {removed.name} moves its expenses to Uncategorised and the category',
        'totals with them; no balance changes. The built-in category cannot be removed:',
        f'  error: {message(built_in)}',
    )
    print_categories(group)

    assert isinstance(built_in, ExpenseGroupError)
    assert built_in.failure == ExpenseGroupError.BuiltInCategory()
    assert group.category(categories.transport) is None
    assert _total(group.summary().category_totals, uncategorised) == eur('248.50')
    assert all(expense.category != categories.transport for expense in group.expenses())


def new_currency(group: ExpenseGroup, people: People) -> None:
    """
    Print section 10: move the group to lei, with the rate of the euro, and show every entry converted again.

    Args:
        group: The group.
        people: The members of the trip.

    """
    section('10. A new currency')
    euro = CurrencyCode('EUR')
    leu = CurrencyCode('RON')
    missing = refusal(lambda: group.change_currency(leu, ()))
    group.change_currency(leu, (CurrencyRate(euro, ExchangeRate('5')),))
    print_lines(
        'Home again, the group moves its books to RON. The change needs a rate into RON',
        'for every other currency in use, here only EUR, or it is refused:',
        f'  error: {message(missing)}',
        'Entries in EUR take the new rate, and the RON parking and cash become local:',
    )
    print_entries(group, lambda _: True)
    print_lines(
        'Each entry keeps its currency and converts at the new rate. At 5 every share',
        'converts exactly, so each balance is five times what it was. The trip cost',
        f'{group.summary().total_spent}:',
    )
    print_balances(group)

    summary = group.summary()
    assert isinstance(missing, ExpenseGroupError)
    assert missing.failure == ExpenseGroupError.MissingExchangeRate(currency=euro)
    assert group.currency() == leu
    assert summary.total_spent == ron('6114.50')
    assert _member_totals(summary, people.dan).balance == GetsBackBalance(ron('62.75'))
    assert _member_totals(summary, people.eva).balance == OwesBalance(ron('213.00'))


def _strategy_name(strategy: SettlementStrategy) -> str:
    """
    Name strategy as the apps show it.

    Args:
        strategy: The strategy.

    Returns:
        The name, such as ``"MinimumTransfers"``.

    """
    match strategy:
        case SettlementStrategy.ADAPTIVE:
            return 'Adaptive'
        case SettlementStrategy.MINIMUM_TRANSFERS:
            return 'MinimumTransfers'
        case SettlementStrategy.GREEDY:
            return 'Greedy'
        case SettlementStrategy.DIRECT:
            return 'Direct'
        case _:
            assert_never(strategy)


def settling_up(group: ExpenseGroup) -> None:
    """
    Print section 11: the payments of each strategy, and the minimum plan, recorded as transfers.

    Args:
        group: The group.

    """
    section('11. Settling up')
    counts: list[StrategyRow] = []
    for strategy in SettlementStrategy:
        settlement = group.settlement(strategy)
        # The group's open balances are well within the adaptive strategy's limit, so it settles them exactly.
        computed_by = SettlementStrategy.MINIMUM_TRANSFERS if strategy is SettlementStrategy.ADAPTIVE else strategy
        assert settlement.strategy == computed_by
        counts.append(
            StrategyRow(
                strategy=_strategy_name(strategy),
                computed_by=_strategy_name(settlement.strategy),
                payments=len(settlement.payments),
            )
        )
    print_lines(
        'Each strategy turns the balances into payments: the direct one pays back every',
        "pair's own debts, greedy pays the largest amounts first, the minimum one finds",
        'the fewest payments, and the adaptive one finds the fewest while that is quick',
        "and greedy's beyond, naming the strategy whose plan it chose.",
    )
    print_table(StrategyRow, counts, 2)
    plan = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS)
    print_lines('The group follows the minimum plan:')
    print_table(
        PaymentRow,
        [PaymentRow(from_=name(group, payment.sender), to=name(group, payment.receiver), amount=payment.amount) for payment in plan.payments],
        2,
    )
    for payment in plan.payments:
        group.add_transfer(Transfer(sender=payment.sender, receiver=payment.receiver, cost=LocalCost(payment.amount), date=october(7)))
    print_lines('Each payment, once made, is recorded as a transfer, and every balance is settled:')
    print_balances(group)

    assert len(group.settlement(SettlementStrategy.GREEDY).payments) == 0
    assert len(plan.payments) == 4
    assert all(found.balance == SettledBalance() for found in group.summary().members)
