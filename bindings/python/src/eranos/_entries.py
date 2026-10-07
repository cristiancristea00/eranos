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
class MemberAmount:
    """The exact amount one member pays of a split."""

    member: MemberId
    """The member."""

    amount: Money
    """The amount."""


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


type Split = EqualSplit | PartsSplit | PercentagesSplit | AmountsSplit
"""
How an amount divides among members: equally, by parts, by percentages, or by exact amounts.

The library divides every amount so that the shares add up to it exactly, and gives any minor unit the rounding leaves
over to a member it picks from the expense's identifier.
"""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class LocalCost:
    """An amount in the group currency."""

    amount: Money
    """The amount."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class ConvertedCost:
    """An amount in another currency than the group's, with the rate that converts it into the group currency."""

    amount: Money
    """The amount, in its own currency."""

    rate: ExchangeRate
    """The rate into the group currency."""


type Cost = LocalCost | ConvertedCost
"""What an entry cost: an amount in the group currency, or one in another with the rate into the group currency."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ExpenseItem:
    """A line of an itemised bill: quantity units of name at unit_price, which split divides."""

    name: str
    """What was bought."""

    unit_price: Money
    """The price of one unit, in the expense's currency."""

    split: Split
    """How the line's total divides."""

    quantity: int = 1
    """How many units, from 1."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class PercentageTip:
    """A tip of a percentage of the expense's amount."""

    percentage: Percentage
    """The percentage."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class FixedTip:
    """A tip of a fixed amount, in the expense's currency."""

    amount: Money
    """The amount."""


type TipValue = PercentageTip | FixedTip
"""What a tip is worth: a percentage of the expense's amount, or a fixed amount."""


@final
class TipDistribution(enum.Enum):
    """How a tip is shared among an expense's participants."""

    PROPORTIONAL = enum.auto()
    """In proportion to each participant's share of the amount."""

    EQUAL = enum.auto()
    """The same for every participant."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class TipRule:
    """A tip on an expense: what it is worth and how it divides."""

    value: TipValue
    """What the tip is worth."""

    distribution: TipDistribution = TipDistribution.PROPORTIONAL
    """How the tip divides, in proportion to each participant's share unless given."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class WholeExpenseSplit:
    """An expense divided as a whole, by one split."""

    split: Split
    """How the amount divides."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class ItemisedExpenseSplit:
    """An expense divided line by line, each item by its own split."""

    items: tuple[ExpenseItem, ...]
    """The lines of the bill."""


type ExpenseSplit = WholeExpenseSplit | ItemisedExpenseSplit
"""How an expense divides: as a whole, by one split, or line by line."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Expense:
    """An expense: what it cost, who paid, how it divides, and when."""

    id: ExpenseId | None = None
    """The identifier the group gave it, or none for an expense to add."""

    name: str
    """What it was."""

    cost: Cost
    """What it cost."""

    payers: Split
    """Who paid, and how much each."""

    split: ExpenseSplit
    """How it divides among the members who share it."""

    date: CalendarDate
    """The day it happened."""

    tip: TipRule | None = None
    """Its tip, or none."""

    time: TimeOfDay | None = None
    """The time of day it happened, or none."""

    category: CategoryId | None = None
    """Its category, or none for the built-in one."""

    description: str | None = None
    """A longer description, or none."""

    def total(self) -> Money:
        """
        Work out the amount with its tip: what the payers paid.

        The library refuses a tip the expense cannot have with an ``ExpenseError``.

        Returns:
            The total.

        """
        with library_errors():
            return money_of(generated.expense_total(generated_expense(self)))

    def tip_amount(self) -> Money:
        """
        Work out the tip in the expense's currency, zero without one.

        The library refuses a tip the expense cannot have with an ``ExpenseError``.

        Returns:
            The tip.

        """
        with library_errors():
            return money_of(generated.expense_tip_amount(generated_expense(self)))

    def members(self) -> tuple[MemberId, ...]:
        """
        List every member the expense names, payers and participants, each once and in the order of their identifiers.

        The library refuses text that is not an identifier with an ``IdError``.

        Returns:
            The members.

        """
        with library_errors():
            return tuple(MemberId(member) for member in generated.expense_members(generated_expense(self)))


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Transfer:
    """A transfer of money between two members, such as a repayment."""

    id: TransferId | None = None
    """The identifier the group gave it, or none for a transfer to add."""

    sender: MemberId
    """The member who sent the money."""

    receiver: MemberId
    """The member who received the money."""

    cost: Cost
    """What was sent."""

    date: CalendarDate
    """The day it happened."""

    time: TimeOfDay | None = None
    """The time of day it happened, or none."""

    note: str | None = None
    """A note, or none."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class IbanPaymentAccountDetails:
    """A bank account, by its IBAN."""

    iban: Iban
    """The IBAN."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class UsernamePaymentAccountDetails:
    """An account in a payment app, by its username."""

    username: Username
    """The username."""


type PaymentAccountDetails = IbanPaymentAccountDetails | UsernamePaymentAccountDetails
"""How a member is paid: a bank account by its IBAN, or a payment app by its username."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class PaymentAccount:
    """An account a member is paid into, with its label."""

    name: str
    """The account's label, such as ``"Bank"``."""

    details: PaymentAccountDetails
    """How the member is paid."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Member:
    """A member of a group, with the details that reach and pay them."""

    id: MemberId | None = None
    """The identifier the group gave them, or none for a member to add."""

    first_name: str
    """The first name."""

    last_name: str | None = None
    """The last name, or none."""

    email: Email | None = None
    """Their email address, or none."""

    phone: PhoneNumber | None = None
    """Their phone number, or none."""

    accounts: tuple[PaymentAccount, ...] = ()
    """The accounts they are paid into."""


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
        case EqualSplit(members=members):
            return variant(generated.Split, generated.Split.EQUAL(list(members)))
        case PartsSplit(shares=shares):
            return variant(
                generated.Split,
                generated.Split.PARTS([generated.MemberParts(member=share.member, hundredths=share.parts.hundredths) for share in shares]),
            )
        case PercentagesSplit(shares=shares):
            return variant(
                generated.Split,
                generated.Split.PERCENTAGES(
                    [generated.MemberPercentage(member=share.member, basis_points=share.percentage.basis_points) for share in shares]
                ),
            )
        case AmountsSplit(shares=shares):
            return variant(
                generated.Split,
                generated.Split.AMOUNTS([generated.MemberAmount(member=share.member, amount=generated_money(share.amount)) for share in shares]),
            )
        case _:
            assert_never(split)


_SPLITS: dict[type[object], Callable[[Any], Split]] = {
    generated.Split.EQUAL: lambda value: EqualSplit(tuple(MemberId(member) for member in value.members)),
    generated.Split.PARTS: lambda value: PartsSplit(tuple(MemberParts(MemberId(share.member), Parts(share.hundredths)) for share in value.shares)),
    generated.Split.PERCENTAGES: lambda value: PercentagesSplit(
        tuple(MemberPercentage(MemberId(share.member), Percentage(share.basis_points)) for share in value.shares)
    ),
    generated.Split.AMOUNTS: lambda value: AmountsSplit(
        tuple(MemberAmount(MemberId(share.member), money_of(share.amount)) for share in value.shares)
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
        case LocalCost(amount=amount):
            return variant(generated.Cost, generated.Cost.LOCAL(generated_money(amount)))
        case ConvertedCost(amount=amount, rate=rate):
            return variant(generated.Cost, generated.Cost.CONVERTED(generated_money(amount), str(rate)))
        case _:
            assert_never(cost)


_COSTS: dict[type[object], Callable[[Any], Cost]] = {
    generated.Cost.LOCAL: lambda value: LocalCost(money_of(value.amount)),
    generated.Cost.CONVERTED: lambda value: ConvertedCost(money_of(value.amount), ExchangeRate(value.rate)),
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


def generated_item(item: ExpenseItem) -> generated.ExpenseItem:
    """
    Convert item into the item the generated module holds.

    Args:
        item: The item.

    Returns:
        The generated item.

    """
    return generated.ExpenseItem(
        name=item.name, unit_price=generated_money(item.unit_price), split=generated_split(item.split), quantity=item.quantity
    )


def item_of(value: generated.ExpenseItem) -> ExpenseItem:
    """
    Convert the item the generated module holds as value.

    Args:
        value: The generated item.

    Returns:
        The item.

    """
    return ExpenseItem(name=value.name, unit_price=money_of(value.unit_price), split=split_of(value.split), quantity=value.quantity)


def generated_tip(tip: TipRule) -> generated.TipRule:
    """
    Convert tip into the tip the generated module holds.

    Args:
        tip: The tip.

    Returns:
        The generated tip.

    """
    match tip.value:
        case PercentageTip(percentage=percentage):
            value = variant(generated.TipValue, generated.TipValue.PERCENTAGE(percentage.basis_points))
        case FixedTip(amount=amount):
            value = variant(generated.TipValue, generated.TipValue.FIXED(generated_money(amount)))
        case _:
            assert_never(tip.value)
    return generated.TipRule(value=value, distribution=generated.TipDistribution[tip.distribution.name])


_TIP_VALUES: dict[type[object], Callable[[Any], TipValue]] = {
    generated.TipValue.PERCENTAGE: lambda value: PercentageTip(Percentage(value.basis_points)),
    generated.TipValue.FIXED: lambda value: FixedTip(money_of(value.amount)),
}
"""The tip's value of each case of the generated tip's value, by its class."""


def tip_of(value: generated.TipRule) -> TipRule:
    """
    Convert the tip the generated module holds as value.

    Args:
        value: The generated tip.

    Returns:
        The tip.

    """
    return TipRule(_TIP_VALUES[type(value.value)](value.value), TipDistribution[value.distribution.name])


def generated_expense_split(split: ExpenseSplit) -> generated.ExpenseSplit:
    """
    Convert split into the expense split the generated module holds.

    Args:
        split: The expense split.

    Returns:
        The generated expense split.

    """
    match split:
        case WholeExpenseSplit(split=whole):
            return variant(generated.ExpenseSplit, generated.ExpenseSplit.WHOLE(generated_split(whole)))
        case ItemisedExpenseSplit(items=items):
            return variant(generated.ExpenseSplit, generated.ExpenseSplit.ITEMISED([generated_item(item) for item in items]))
        case _:
            assert_never(split)


_EXPENSE_SPLITS: dict[type[object], Callable[[Any], ExpenseSplit]] = {
    generated.ExpenseSplit.WHOLE: lambda value: WholeExpenseSplit(split_of(value.split)),
    generated.ExpenseSplit.ITEMISED: lambda value: ItemisedExpenseSplit(tuple(item_of(item) for item in value.items)),
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
        name=expense.name,
        cost=generated_cost(expense.cost),
        payers=generated_split(expense.payers),
        split=generated_expense_split(expense.split),
        date=generated_date(expense.date),
        tip=None if expense.tip is None else generated_tip(expense.tip),
        time=None if expense.time is None else generated_time(expense.time),
        category=expense.category,
        description=expense.description,
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
        name=value.name,
        cost=cost_of(value.cost),
        payers=split_of(value.payers),
        split=_EXPENSE_SPLITS[type(value.split)](value.split),
        date=date_of(value.date),
        tip=None if value.tip is None else tip_of(value.tip),
        time=None if value.time is None else time_of(value.time),
        category=None if value.category is None else CategoryId(value.category),
        description=value.description,
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
        _from=transfer.sender,
        to=transfer.receiver,
        cost=generated_cost(transfer.cost),
        date=generated_date(transfer.date),
        time=None if transfer.time is None else generated_time(transfer.time),
        note=transfer.note,
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
        # The generator names the field after the keyword it would otherwise be.
        sender=MemberId(value._from),  # ruff: ignore[private-member-access]
        receiver=MemberId(value.to),
        cost=cost_of(value.cost),
        date=date_of(value.date),
        time=None if value.time is None else time_of(value.time),
        note=value.note,
    )


def generated_account(account: PaymentAccount) -> generated.PaymentAccount:
    """
    Convert account into the account the generated module holds.

    Args:
        account: The account.

    Returns:
        The generated account.

    """
    match account.details:
        case IbanPaymentAccountDetails(iban=iban):
            details = variant(generated.PaymentAccountDetails, generated.PaymentAccountDetails.IBAN(str(iban)))
        case UsernamePaymentAccountDetails(username=username):
            details = variant(generated.PaymentAccountDetails, generated.PaymentAccountDetails.USERNAME(str(username)))
        case _:
            assert_never(account.details)
    return generated.PaymentAccount(name=account.name, details=details)


_PAYMENT_ACCOUNT_DETAILS: dict[type[object], Callable[[Any], PaymentAccountDetails]] = {
    generated.PaymentAccountDetails.IBAN: lambda value: IbanPaymentAccountDetails(Iban(value.iban)),
    generated.PaymentAccountDetails.USERNAME: lambda value: UsernamePaymentAccountDetails(Username(value.username)),
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
        last_name=member.last_name,
        email=None if member.email is None else str(member.email),
        phone=None if member.phone is None else str(member.phone),
        accounts=[generated_account(account) for account in member.accounts],
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
        last_name=value.last_name,
        email=None if value.email is None else Email(value.email),
        phone=None if value.phone is None else PhoneNumber(value.phone),
        accounts=tuple(PaymentAccount(account.name, _PAYMENT_ACCOUNT_DETAILS[type(account.details)](account.details)) for account in value.accounts),
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
