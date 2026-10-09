"""Hold what a group adds up: its summary, each expense's shares, and the payments that settle it."""

import dataclasses
import enum
from collections.abc import Callable
from typing import Any, assert_never, final, override

from eranos._identifiers import CategoryId, MemberId
from eranos._internal import eranos as generated
from eranos._money import Money, generated_money, money_of
from eranos._crossing import variant


@final
@dataclasses.dataclass(frozen=True, slots=True)
class OwesBalance:
    """A member who owes the others amount."""

    amount: Money
    """What the member owes."""

    @override
    def __str__(self) -> str:
        """
        Write the balance as the library does.

        Returns:
            The text, such as ``"owes 12.00 EUR"``.

        """
        return balance_text(self)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class SettledBalance:
    """A member who owes nothing and is owed nothing."""

    @override
    def __str__(self) -> str:
        """
        Write the balance as the library does.

        Returns:
            The text, such as ``"settled"``.

        """
        return balance_text(self)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class GetsBackBalance:
    """A member whom the others owe amount."""

    amount: Money
    """What the others owe the member."""

    @override
    def __str__(self) -> str:
        """
        Write the balance as the library does.

        Returns:
            The text, such as ``"gets back 12.00 EUR"``.

        """
        return balance_text(self)


type Balance = OwesBalance | SettledBalance | GetsBackBalance
"""Where a member stands: owing, settled, or owed."""


def balance_text(balance: Balance) -> str:
    """
    Write balance as the library does.

    Args:
        balance: The balance.

    Returns:
        The text: ``"owes 12.00 EUR"``, ``"settled"``, or ``"gets back 12.00 EUR"``.

    """
    match balance:
        case OwesBalance(amount=amount):
            value = variant(generated.Balance, generated.Balance.OWES(generated_money(amount)))
        case SettledBalance():
            value = variant(generated.Balance, generated.Balance.SETTLED())
        case GetsBackBalance(amount=amount):
            value = variant(generated.Balance, generated.Balance.GETS_BACK(generated_money(amount)))
        case _:
            assert_never(balance)
    return generated.balance_format(value)


_BALANCES: dict[type[object], Callable[[Any], Balance]] = {
    generated.Balance.OWES: lambda value: OwesBalance(money_of(value.amount)),
    generated.Balance.SETTLED: lambda _: SettledBalance(),
    generated.Balance.GETS_BACK: lambda value: GetsBackBalance(money_of(value.amount)),
}
"""The balance of each case of the generated balance, by its class."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class MemberSummary:
    """What one member paid, consumed, sent, and received, and where they stand."""

    member: MemberId
    """The member."""

    paid: Money
    """What they paid for expenses."""

    share: Money
    """What they consumed, their shares of the expenses."""

    sent: Money
    """What they sent in transfers."""

    received: Money
    """What they received in transfers."""

    net: Money
    """What they paid and sent, less what they consumed and received."""

    balance: Balance
    """Where the member stands."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class CategoryTotal:
    """What a group spent in one category."""

    category: CategoryId
    """The category."""

    total: Money
    """What was spent in it."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class MemberCategoryTotals:
    """What one member consumed in each category."""

    member: MemberId
    """The member."""

    totals: tuple[CategoryTotal, ...]
    """What they consumed in each category."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Summary:
    """What a group adds up to: what it spent, by each member, and in each category."""

    total_spent: Money
    """What the group spent in all."""

    members: tuple[MemberSummary, ...]
    """The totals of each member."""

    category_totals: tuple[CategoryTotal, ...]
    """What the group spent in each category."""

    member_category_totals: tuple[MemberCategoryTotals, ...]
    """What each member consumed in each category."""

    def member(self, member_id: MemberId) -> MemberSummary:
        """
        Find the totals of the member with the identifier member_id: all zero, and settled, for a member without entries.

        Args:
            member_id: The member's identifier.

        Returns:
            The member's totals.

        """
        zero = Money.zero(self.total_spent.currency)
        absent = MemberSummary(member=member_id, paid=zero, share=zero, sent=zero, received=zero, net=zero, balance=SettledBalance())
        return next((totals for totals in self.members if totals.member == member_id), absent)

    def category_total(self, category_id: CategoryId) -> Money:
        """
        Find the spending in the category with the identifier category_id: zero for a category without expenses.

        Args:
            category_id: The category's identifier.

        Returns:
            The spending.

        """
        zero = Money.zero(self.total_spent.currency)
        return next((total.total for total in self.category_totals if total.category == category_id), zero)

    def member_category_total(self, member_id: MemberId, category_id: CategoryId) -> Money:
        """
        Find the share of the member with the identifier member_id of the spending in the category category_id.

        The share is zero when they have none there.

        Args:
            member_id: The member's identifier.
            category_id: The category's identifier.

        Returns:
            The member's share of the spending.

        """
        zero = Money.zero(self.total_spent.currency)
        totals = next((entry.totals for entry in self.member_category_totals if entry.member == member_id), ())
        return next((total.total for total in totals if total.category == category_id), zero)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class MemberShare:
    """What one member paid of an expense and what they consumed of it."""

    member: MemberId
    """The member."""

    paid: Money
    """What they paid of it."""

    share: Money
    """What they consumed of it."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ExpenseShares:
    """How an expense divides, in the group currency."""

    members: tuple[MemberShare, ...]
    """What each member paid and consumed."""

    total: Money
    """The amount with its tip."""

    tip: Money
    """The tip."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Payment:
    """A payment of a settlement: amount from sender to receiver."""

    sender: MemberId
    """The member who pays it."""

    receiver: MemberId
    """The member who receives it."""

    amount: Money
    """What to pay."""


@final
class SettlementStrategy(enum.Enum):
    """How balances turn into payments."""

    ADAPTIVE = enum.auto()
    """
    The fewest payments while at most 20 balances are left after exact opposites pay each other, the greedy plan beyond.

    The settlement names the strategy whose plan it is.
    """

    MINIMUM_TRANSFERS = enum.auto()
    """The fewest payments possible."""

    GREEDY = enum.auto()
    """The largest debtor pays the largest creditor, repeatedly, after exact opposites pay each other where that saves a payment."""

    DIRECT = enum.auto()
    """Everyone pays back the people who paid for them."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class SettlementPlan:
    """The payments that settle a group, and the strategy that planned them."""

    payments: tuple[Payment, ...]
    """The payments."""

    strategy: SettlementStrategy
    """The strategy that planned them, which is the one asked for or, for the adaptive strategy, the one whose plan it chose."""


def summary_of(value: generated.Summary) -> Summary:
    """
    Convert the summary the generated module holds as value.

    Args:
        value: The generated summary.

    Returns:
        The summary.

    """
    return Summary(
        total_spent=money_of(value.total_spent),
        members=tuple(
            MemberSummary(
                member=MemberId(member.member),
                paid=money_of(member.paid),
                share=money_of(member.share),
                sent=money_of(member.sent),
                received=money_of(member.received),
                net=money_of(member.net),
                balance=_BALANCES[type(member.balance)](member.balance),
            )
            for member in value.members
        ),
        category_totals=tuple(category_total_of(total) for total in value.category_totals),
        member_category_totals=tuple(
            MemberCategoryTotals(MemberId(totals.member), tuple(category_total_of(total) for total in totals.totals))
            for totals in value.member_category_totals
        ),
    )


def category_total_of(value: generated.CategoryTotal) -> CategoryTotal:
    """
    Convert the category's total the generated module holds as value.

    Args:
        value: The generated total.

    Returns:
        The total.

    """
    return CategoryTotal(CategoryId(value.category), money_of(value.total))


def expense_shares_of(value: generated.ExpenseShares) -> ExpenseShares:
    """
    Convert the expense's shares the generated module holds as value.

    Args:
        value: The generated shares.

    Returns:
        The shares.

    """
    return ExpenseShares(
        members=tuple(MemberShare(MemberId(share.member), money_of(share.paid), money_of(share.share)) for share in value.members),
        total=money_of(value.total),
        tip=money_of(value.tip),
    )


def settlement_of(value: generated.SettlementPlan) -> SettlementPlan:
    """
    Convert the settlement the generated module holds as value.

    Args:
        value: The generated settlement.

    Returns:
        The settlement.

    """
    return SettlementPlan(
        tuple(
            Payment(
                sender=MemberId(payment.sender),
                receiver=MemberId(payment.receiver),
                amount=money_of(payment.amount),
            )
            for payment in value.payments
        ),
        SettlementStrategy[value.strategy.name],
    )


def generated_strategy(strategy: SettlementStrategy) -> generated.SettlementStrategy:
    """
    Convert strategy into the strategy the generated module holds.

    Args:
        strategy: The strategy.

    Returns:
        The generated strategy.

    """
    return generated.SettlementStrategy[strategy.name]
