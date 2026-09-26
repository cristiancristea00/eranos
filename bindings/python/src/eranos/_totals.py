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


type Balance = GetsBackBalance | OwesBalance | SettledBalance
"""Where a member stands: owed, owing, or settled."""


def balance_text(balance: Balance) -> str:
    """
    Write balance as the library does.

    Args:
        balance: The balance.

    Returns:
        The text: ``"gets back 12.00 EUR"``, ``"owes 12.00 EUR"``, or ``"settled"``.

    """
    match balance:
        case GetsBackBalance(amount=amount):
            value = variant(generated.Balance, generated.Balance.GETS_BACK(generated_money(amount)))
        case OwesBalance(amount=amount):
            value = variant(generated.Balance, generated.Balance.OWES(generated_money(amount)))
        case SettledBalance():
            value = variant(generated.Balance, generated.Balance.SETTLED())
        case _:
            assert_never(balance)
    return generated.balance_format(value)


_BALANCES: dict[type[object], Callable[[Any], Balance]] = {
    generated.Balance.GETS_BACK: lambda value: GetsBackBalance(money_of(value.amount)),
    generated.Balance.OWES: lambda value: OwesBalance(money_of(value.amount)),
    generated.Balance.SETTLED: lambda _: SettledBalance(),
}
"""The balance of each case of the generated balance, by its class."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class MemberSummary:
    """What one member paid, consumed, sent, and received, and where they stand."""

    balance: Balance
    """Where the member stands."""

    member: MemberId
    """The member."""

    net: Money
    """What they paid and sent, less what they consumed and received."""

    paid: Money
    """What they paid for expenses."""

    received: Money
    """What they received in transfers."""

    sent: Money
    """What they sent in transfers."""

    share: Money
    """What they consumed, their shares of the expenses."""


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
    """What a group adds up to: what it spent, in each category, and by each member."""

    category_totals: tuple[CategoryTotal, ...]
    """What the group spent in each category."""

    member_category_totals: tuple[MemberCategoryTotals, ...]
    """What each member consumed in each category."""

    members: tuple[MemberSummary, ...]
    """The totals of each member."""

    total_spent: Money
    """What the group spent in all."""


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

    tip: Money
    """The tip."""

    total: Money
    """The amount with its tip."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Payment:
    """A payment of a settlement: amount from sender to receiver."""

    amount: Money
    """What to pay."""

    receiver: MemberId
    """The member who receives it."""

    sender: MemberId
    """The member who pays it."""


@final
class SettlementStrategy(enum.Enum):
    """How balances turn into payments."""

    DIRECT = enum.auto()
    """Everyone pays back the people who paid for them."""

    GREEDY = enum.auto()
    """The largest debtor pays the largest creditor, repeatedly."""

    MINIMUM_TRANSFERS = enum.auto()
    """The fewest payments possible."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Settlement:
    """The payments that settle a group, and the strategy that planned them."""

    payments: tuple[Payment, ...]
    """The payments."""

    strategy: SettlementStrategy
    """The strategy that planned them."""


def summary_of(value: generated.Summary) -> Summary:
    """
    Convert the summary the generated module holds as value.

    Args:
        value: The generated summary.

    Returns:
        The summary.

    """
    return Summary(
        category_totals=tuple(category_total_of(total) for total in value.category_totals),
        member_category_totals=tuple(
            MemberCategoryTotals(MemberId(totals.member), tuple(category_total_of(total) for total in totals.totals))
            for totals in value.member_category_totals
        ),
        members=tuple(
            MemberSummary(
                balance=_BALANCES[type(member.balance)](member.balance),
                member=MemberId(member.member),
                net=money_of(member.net),
                paid=money_of(member.paid),
                received=money_of(member.received),
                sent=money_of(member.sent),
                share=money_of(member.share),
            )
            for member in value.members
        ),
        total_spent=money_of(value.total_spent),
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
        tip=money_of(value.tip),
        total=money_of(value.total),
    )


def settlement_of(value: generated.Settlement) -> Settlement:
    """
    Convert the settlement the generated module holds as value.

    Args:
        value: The generated settlement.

    Returns:
        The settlement.

    """
    return Settlement(
        tuple(
            Payment(
                amount=money_of(payment.amount),
                receiver=MemberId(payment.to),
                # The generator names the field after the keyword it would otherwise be.
                sender=MemberId(payment._from),  # ruff: ignore[private-member-access]
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
