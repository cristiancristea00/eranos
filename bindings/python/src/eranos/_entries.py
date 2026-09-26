"""
Hold the entries of a group: its members, categories, expenses, and transfers, and what they hold.

Every entry is a frozen dataclass: one to add has no identifier, and one the group holds has the identifier it gave, and
``copy.replace`` edits one. A variant, such as a split or a cost, is a union of dataclasses, which a ``match`` tells apart.

"""

import dataclasses
import enum
from collections.abc import Callable
from typing import Any, assert_never, final

from eranos._calendar import CalendarDate, TimeOfDay, date_of, generated_date, generated_time, time_of
from eranos._contact import Email, Iban, PhoneNumber, Username
from eranos._errors import library_errors
from eranos._identifiers import CategoryId, ExpenseId, MemberId, TransferId
from eranos._internal import eranos as generated
from eranos._money import Money, generated_money, money_of
from eranos._ratios import ExchangeRate, Parts, Percentage
from eranos._crossing import variant


@final
@dataclasses.dataclass(frozen=True, slots=True)
class MemberAmount:
    """The exact amount one member pays of a split."""

    amount: Money
    """The amount."""

    member: MemberId
    """The member."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class MemberParts:
    """The parts one member pays in proportion to."""

    member: MemberId
    """The member."""

    parts: Parts
    """The member's parts."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class MemberPercentage:
    """The percentage one member pays."""

    member: MemberId
    """The member."""

    percentage: Percentage
    """The member's percentage."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class AmountsSplit:
    """Every member pays an exact amount; the amounts add up to what they divide."""

    shares: tuple[MemberAmount, ...]
    """Each member's amount."""

    @property
    def members(self) -> tuple[MemberId, ...]:
        """The members the split divides among, in the order it lists them."""
        return tuple(share.member for share in self.shares)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class EqualSplit:
    """Every member pays the same."""

    members: tuple[MemberId, ...]
    """The members the split divides among, in the order it lists them."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class PartsSplit:
    """Every member pays in proportion to their parts."""

    shares: tuple[MemberParts, ...]
    """Each member's parts."""

    @property
    def members(self) -> tuple[MemberId, ...]:
        """The members the split divides among, in the order it lists them."""
        return tuple(share.member for share in self.shares)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class PercentagesSplit:
    """Every member pays a percentage; the percentages add up to 100 %."""

    shares: tuple[MemberPercentage, ...]
    """Each member's percentage."""

    @property
    def members(self) -> tuple[MemberId, ...]:
        """The members the split divides among, in the order it lists them."""
        return tuple(share.member for share in self.shares)


type Split = AmountsSplit | EqualSplit | PartsSplit | PercentagesSplit
"""
How an amount divides among members: equally, by exact amounts, by parts, or by percentages.

The library divides every amount so that the shares add up to it exactly, and gives any minor unit the rounding leaves
over to a member it picks from the expense's identifier.
"""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class ConvertedCost:
    """An amount in another currency than the group's, with the rate that converts it into the group currency."""

    amount: Money
    """The amount, in its own currency."""

    rate: ExchangeRate
    """The rate into the group currency."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class LocalCost:
    """An amount in the group currency."""

    amount: Money
    """The amount."""


type Cost = ConvertedCost | LocalCost
"""What an entry cost: an amount in the group currency, or one in another with the rate into the group currency."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Item:
    """A line of an itemised bill: quantity units of name at unit_price, which split divides."""

    name: str
    """What was bought."""

    split: Split
    """How the line's total divides."""

    unit_price: Money
    """The price of one unit, in the expense's currency."""

    quantity: int = 1
    """How many units, from 1."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class FixedTip:
    """A tip of a fixed amount, in the expense's currency."""

    amount: Money
    """The amount."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class PercentageTip:
    """A tip of a percentage of the expense's amount."""

    percentage: Percentage
    """The percentage."""


type TipValue = FixedTip | PercentageTip
"""What a tip is worth: a fixed amount, or a percentage of the expense's amount."""


@final
class TipDistribution(enum.Enum):
    """How a tip is shared among an expense's participants."""

    EQUAL = enum.auto()
    """The same for every participant."""

    PROPORTIONAL = enum.auto()
    """In proportion to each participant's share of the amount."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Tip:
    """A tip on an expense: what it is worth and how it divides."""

    distribution: TipDistribution
    """How the tip divides."""

    value: TipValue
    """What the tip is worth."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class ItemisedExpenseSplit:
    """An expense divided line by line, each item by its own split."""

    items: tuple[Item, ...]
    """The lines of the bill."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class WholeExpenseSplit:
    """An expense divided as a whole, by one split."""

    split: Split
    """How the amount divides."""


type ExpenseSplit = ItemisedExpenseSplit | WholeExpenseSplit
"""How an expense divides: as a whole, by one split, or line by line."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Expense:
    """An expense: what it cost, when, who paid, and how it divides."""

    id: ExpenseId | None = None
    """The identifier the group gave it, or none for an expense to add."""

    cost: Cost
    """What it cost."""

    date: CalendarDate
    """The day it happened."""

    name: str
    """What it was."""

    payers: Split
    """Who paid, and how much each."""

    split: ExpenseSplit
    """How it divides among the members who share it."""

    category: CategoryId | None = None
    """Its category, or none for the built-in one."""

    description: str | None = None
    """A longer description, or none."""

    time: TimeOfDay | None = None
    """The time of day it happened, or none."""

    tip: Tip | None = None
    """Its tip, or none."""

    def members(self) -> tuple[MemberId, ...]:
        """
        List every member the expense names, payers and participants, each once and in the order of their identifiers.

        The library refuses text that is not an identifier with ``IdError.InvalidId``, in an ``IdError``.

        Returns:
            The members.

        """
        with library_errors():
            return tuple(MemberId(member) for member in generated.expense_members(generated_expense(self)))

    def tip_amount(self) -> Money:
        """
        Work out the tip in the expense's currency, zero without one.

        The library refuses a tip the expense cannot have with an ``ExpenseError``.

        Returns:
            The tip.

        """
        with library_errors():
            return money_of(generated.expense_tip_amount(generated_expense(self)))

    def total(self) -> Money:
        """
        Work out the amount with its tip: what the payers paid.

        The library refuses a tip the expense cannot have with an ``ExpenseError``.

        Returns:
            The total.

        """
        with library_errors():
            return money_of(generated.expense_total(generated_expense(self)))


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Transfer:
    """A transfer of money between two members, such as a repayment."""

    id: TransferId | None = None
    """The identifier the group gave it, or none for a transfer to add."""

    cost: Cost
    """What was sent."""

    date: CalendarDate
    """The day it happened."""

    receiver: MemberId
    """The member who received the money."""

    sender: MemberId
    """The member who sent the money."""

    note: str | None = None
    """A note, or none."""

    time: TimeOfDay | None = None
    """The time of day it happened, or none."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class IbanAccountDetails:
    """A bank account, by its IBAN."""

    iban: Iban
    """The IBAN."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class UsernameAccountDetails:
    """An account in a payment app, by its username."""

    username: Username
    """The username."""


type AccountDetails = IbanAccountDetails | UsernameAccountDetails
"""How a member is paid: a bank account by its IBAN, or a payment app by its username."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Account:
    """An account a member is paid into, with its label."""

    details: AccountDetails
    """How the member is paid."""

    name: str
    """The account's label, such as ``"Bank"``."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Member:
    """A member of a group, with the details that reach and pay them."""

    id: MemberId | None = None
    """The identifier the group gave them, or none for a member to add."""

    first_name: str
    """The first name."""

    accounts: tuple[Account, ...] = ()
    """The accounts they are paid into."""

    email: Email | None = None
    """Their email address, or none."""

    last_name: str | None = None
    """The last name, or none."""

    phone: PhoneNumber | None = None
    """Their phone number, or none."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Category:
    """A category of expenses."""

    id: CategoryId | None = None
    """The identifier the group gave it, or none for a category to add."""

    name: str
    """The name."""


def generated_split(split: Split) -> generated.Split:
    """
    Convert split into the split the generated module holds.

    Args:
        split: The split.

    Returns:
        The generated split.

    """
    match split:
        case AmountsSplit(shares=shares):
            return variant(
                generated.Split,
                generated.Split.AMOUNTS([generated.MemberAmount(amount=generated_money(share.amount), member=share.member) for share in shares]),
            )
        case EqualSplit(members=members):
            return variant(generated.Split, generated.Split.EQUAL(list(members)))
        case PartsSplit(shares=shares):
            return variant(
                generated.Split,
                generated.Split.PARTS([generated.MemberParts(hundredths=share.parts.hundredths, member=share.member) for share in shares]),
            )
        case PercentagesSplit(shares=shares):
            return variant(
                generated.Split,
                generated.Split.PERCENTAGES(
                    [generated.MemberPercentage(basis_points=share.percentage.basis_points, member=share.member) for share in shares]
                ),
            )
        case _:
            assert_never(split)


_SPLITS: dict[type[object], Callable[[Any], Split]] = {
    generated.Split.AMOUNTS: lambda value: AmountsSplit(
        tuple(MemberAmount(money_of(share.amount), MemberId(share.member)) for share in value.shares)
    ),
    generated.Split.EQUAL: lambda value: EqualSplit(tuple(MemberId(member) for member in value.members)),
    generated.Split.PARTS: lambda value: PartsSplit(tuple(MemberParts(MemberId(share.member), Parts(share.hundredths)) for share in value.shares)),
    generated.Split.PERCENTAGES: lambda value: PercentagesSplit(
        tuple(MemberPercentage(MemberId(share.member), Percentage(share.basis_points)) for share in value.shares)
    ),
}
"""The split of each case of the generated split, by its class."""


def split_of(value: generated.Split) -> Split:
    """
    Convert the split the generated module holds as value.

    Args:
        value: The generated split.

    Returns:
        The split.

    """
    return _SPLITS[type(value)](value)


def generated_cost(cost: Cost) -> generated.Cost:
    """
    Convert cost into the cost the generated module holds.

    Args:
        cost: The cost.

    Returns:
        The generated cost.

    """
    match cost:
        case ConvertedCost(amount=amount, rate=rate):
            return variant(generated.Cost, generated.Cost.CONVERTED(generated_money(amount), str(rate)))
        case LocalCost(amount=amount):
            return variant(generated.Cost, generated.Cost.LOCAL(generated_money(amount)))
        case _:
            assert_never(cost)


_COSTS: dict[type[object], Callable[[Any], Cost]] = {
    generated.Cost.CONVERTED: lambda value: ConvertedCost(money_of(value.amount), ExchangeRate(value.rate)),
    generated.Cost.LOCAL: lambda value: LocalCost(money_of(value.amount)),
}
"""The cost of each case of the generated cost, by its class."""


def cost_of(value: generated.Cost) -> Cost:
    """
    Convert the cost the generated module holds as value.

    Args:
        value: The generated cost.

    Returns:
        The cost.

    """
    return _COSTS[type(value)](value)


def generated_item(item: Item) -> generated.Item:
    """
    Convert item into the item the generated module holds.

    Args:
        item: The item.

    Returns:
        The generated item.

    """
    return generated.Item(name=item.name, split=generated_split(item.split), unit_price=generated_money(item.unit_price), quantity=item.quantity)


def item_of(value: generated.Item) -> Item:
    """
    Convert the item the generated module holds as value.

    Args:
        value: The generated item.

    Returns:
        The item.

    """
    return Item(name=value.name, split=split_of(value.split), unit_price=money_of(value.unit_price), quantity=value.quantity)


def generated_tip(tip: Tip) -> generated.Tip:
    """
    Convert tip into the tip the generated module holds.

    Args:
        tip: The tip.

    Returns:
        The generated tip.

    """
    match tip.value:
        case FixedTip(amount=amount):
            value = variant(generated.TipValue, generated.TipValue.FIXED(generated_money(amount)))
        case PercentageTip(percentage=percentage):
            value = variant(generated.TipValue, generated.TipValue.PERCENTAGE(percentage.basis_points))
        case _:
            assert_never(tip.value)
    return generated.Tip(value=value, distribution=generated.TipDistribution[tip.distribution.name])


_TIP_VALUES: dict[type[object], Callable[[Any], TipValue]] = {
    generated.TipValue.FIXED: lambda value: FixedTip(money_of(value.amount)),
    generated.TipValue.PERCENTAGE: lambda value: PercentageTip(Percentage(value.basis_points)),
}
"""The tip's value of each case of the generated tip's value, by its class."""


def tip_of(value: generated.Tip) -> Tip:
    """
    Convert the tip the generated module holds as value.

    Args:
        value: The generated tip.

    Returns:
        The tip.

    """
    return Tip(TipDistribution[value.distribution.name], _TIP_VALUES[type(value.value)](value.value))


def generated_expense_split(split: ExpenseSplit) -> generated.ExpenseSplit:
    """
    Convert split into the expense split the generated module holds.

    Args:
        split: The expense split.

    Returns:
        The generated expense split.

    """
    match split:
        case ItemisedExpenseSplit(items=items):
            return variant(generated.ExpenseSplit, generated.ExpenseSplit.ITEMISED([generated_item(item) for item in items]))
        case WholeExpenseSplit(split=whole):
            return variant(generated.ExpenseSplit, generated.ExpenseSplit.WHOLE(generated_split(whole)))
        case _:
            assert_never(split)


_EXPENSE_SPLITS: dict[type[object], Callable[[Any], ExpenseSplit]] = {
    generated.ExpenseSplit.ITEMISED: lambda value: ItemisedExpenseSplit(tuple(item_of(item) for item in value.items)),
    generated.ExpenseSplit.WHOLE: lambda value: WholeExpenseSplit(split_of(value.split)),
}
"""The expense split of each case of the generated expense split, by its class."""


def generated_expense(expense: Expense) -> generated.Expense:
    """
    Convert expense into the expense the generated module holds.

    Args:
        expense: The expense.

    Returns:
        The generated expense.

    """
    return generated.Expense(
        id=expense.id,
        cost=generated_cost(expense.cost),
        date=generated_date(expense.date),
        name=expense.name,
        payers=generated_split(expense.payers),
        split=generated_expense_split(expense.split),
        category=expense.category,
        description=expense.description,
        time=None if expense.time is None else generated_time(expense.time),
        tip=None if expense.tip is None else generated_tip(expense.tip),
    )


def expense_of(value: generated.Expense) -> Expense:
    """
    Convert the expense the generated module holds as value.

    Args:
        value: The generated expense.

    Returns:
        The expense.

    """
    return Expense(
        id=None if value.id is None else ExpenseId(value.id),
        cost=cost_of(value.cost),
        date=date_of(value.date),
        name=value.name,
        payers=split_of(value.payers),
        split=_EXPENSE_SPLITS[type(value.split)](value.split),
        category=None if value.category is None else CategoryId(value.category),
        description=value.description,
        time=None if value.time is None else time_of(value.time),
        tip=None if value.tip is None else tip_of(value.tip),
    )


def generated_transfer(transfer: Transfer) -> generated.Transfer:
    """
    Convert transfer into the transfer the generated module holds.

    Args:
        transfer: The transfer.

    Returns:
        The generated transfer.

    """
    return generated.Transfer(
        id=transfer.id,
        cost=generated_cost(transfer.cost),
        date=generated_date(transfer.date),
        _from=transfer.sender,
        to=transfer.receiver,
        note=transfer.note,
        time=None if transfer.time is None else generated_time(transfer.time),
    )


def transfer_of(value: generated.Transfer) -> Transfer:
    """
    Convert the transfer the generated module holds as value.

    Args:
        value: The generated transfer.

    Returns:
        The transfer.

    """
    return Transfer(
        id=None if value.id is None else TransferId(value.id),
        cost=cost_of(value.cost),
        date=date_of(value.date),
        receiver=MemberId(value.to),
        # The generator names the field after the keyword it would otherwise be.
        sender=MemberId(value._from),  # ruff: ignore[private-member-access]
        note=value.note,
        time=None if value.time is None else time_of(value.time),
    )


def generated_account(account: Account) -> generated.Account:
    """
    Convert account into the account the generated module holds.

    Args:
        account: The account.

    Returns:
        The generated account.

    """
    match account.details:
        case IbanAccountDetails(iban=iban):
            details = variant(generated.AccountDetails, generated.AccountDetails.IBAN(str(iban)))
        case UsernameAccountDetails(username=username):
            details = variant(generated.AccountDetails, generated.AccountDetails.USERNAME(str(username)))
        case _:
            assert_never(account.details)
    return generated.Account(details=details, name=account.name)


_ACCOUNT_DETAILS: dict[type[object], Callable[[Any], AccountDetails]] = {
    generated.AccountDetails.IBAN: lambda value: IbanAccountDetails(Iban(value.iban)),
    generated.AccountDetails.USERNAME: lambda value: UsernameAccountDetails(Username(value.username)),
}
"""The account details of each case of the generated account details, by its class."""


def generated_member(member: Member) -> generated.Member:
    """
    Convert member into the member the generated module holds.

    Args:
        member: The member.

    Returns:
        The generated member.

    """
    return generated.Member(
        id=member.id,
        first_name=member.first_name,
        accounts=[generated_account(account) for account in member.accounts],
        email=None if member.email is None else str(member.email),
        last_name=member.last_name,
        phone=None if member.phone is None else str(member.phone),
    )


def member_of(value: generated.Member) -> Member:
    """
    Convert the member the generated module holds as value.

    Args:
        value: The generated member.

    Returns:
        The member.

    """
    return Member(
        id=None if value.id is None else MemberId(value.id),
        first_name=value.first_name,
        accounts=tuple(Account(_ACCOUNT_DETAILS[type(account.details)](account.details), account.name) for account in value.accounts),
        email=None if value.email is None else Email(value.email),
        last_name=value.last_name,
        phone=None if value.phone is None else PhoneNumber(value.phone),
    )


def generated_category(category: Category) -> generated.Category:
    """
    Convert category into the category the generated module holds.

    Args:
        category: The category.

    Returns:
        The generated category.

    """
    return generated.Category(id=category.id, name=category.name)


def category_of(value: generated.Category) -> Category:
    """
    Convert the category the generated module holds as value.

    Args:
        value: The generated category.

    Returns:
        The category.

    """
    return Category(id=None if value.id is None else CategoryId(value.id), name=value.name)
