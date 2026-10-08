"""
Hold the entries of a group: its members, categories, expenses, and transfers, and what they hold.

Every entry is a frozen dataclass that carries its identifier from the moment it is made: the class takes the app's own
or a generated one, and the classmethod ``create`` generates it when it is called. ``copy.replace`` edits an entry and
keeps its identifier. A variant, such as a split or a cost, is a union of dataclasses, which a ``match`` tells apart.

"""

import dataclasses
import enum
from collections.abc import Callable
from typing import Any, Final, Self, assert_never, final

from eranos._calendar import CalendarDate, TimeOfDay, date_of, generated_date, generated_time, time_of
from eranos._contact import Email, Iban, PhoneNumber, Username, generated_phone_number, iban_of, phone_number_of
from eranos._errors import library_errors
from eranos._identifiers import (
    UNCATEGORISED,
    CategoryId,
    ExpenseId,
    MemberId,
    TransferId,
    generate_category_id,
    generate_expense_id,
    generate_member_id,
    generate_transfer_id,
)
from eranos._internal import eranos as generated
from eranos._money import Money, generated_money, money_of
from eranos._quantity import Quantity
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
@dataclasses.dataclass(frozen=True, slots=True)
class PercentageDiscount:
    """A discount of a percentage of what it reduces, at most 100 %."""

    percentage: Percentage
    """The percentage."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class FixedDiscount:
    """A discount of a fixed amount, in the item's currency, at most what it reduces."""

    amount: Money
    """The amount."""


type DiscountValue = PercentageDiscount | FixedDiscount
"""How a discount is expressed: a percentage of what it reduces, or a fixed amount."""


@final
class DiscountScope(enum.Enum):
    """What a discount is taken off."""

    LINE = enum.auto()
    """The line total, the unit price times the quantity."""

    UNIT = enum.auto()
    """The price of each unit, before it is multiplied by the quantity."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Discount:
    """
    A discount off a line of an itemised bill: what it is worth and what it is taken off.

    Off the line, a percentage is taken of the line total before the discount and rounded half away from zero to the minor
    unit. Off each unit, a percentage of the unit price is rounded to the minor unit first, and the reduced price is then
    multiplied by the quantity, so three units at 3.33 with 50 % off cost 4.98 with the discount off each unit and 4.99 with
    it off the line. A discount of the whole price, 100 % or a fixed amount equal to what it reduces, makes a free line,
    whose total is zero.
    """

    value: DiscountValue
    """How the discount is expressed."""

    scope: DiscountScope = DiscountScope.LINE
    """What the discount is taken off, the line total unless given."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ExpenseItem:
    """A line of an itemised bill: quantity units of name at unit_price, less its discount, which split divides."""

    name: str
    """What was bought."""

    unit_price: Money
    """The price of one unit, in the expense's currency."""

    split: Split
    """How the line's total divides."""

    quantity: Quantity = Quantity.ONE
    """How many units, ``Quantity.ONE`` unless given; a fractional quantity, such as 0.452 kg, has at most four decimal places."""

    discount: Discount | None = None
    """The discount off the line or off each unit, or none for a line at full price."""

    def line_total_before_discount(self) -> Money:
        """
        Work out the line's total before its discount as the library does: the unit price times the quantity, rounded half away from zero.

        A line whose product is below half a minor unit totals zero. The library refuses an item it cannot hold as
        ``line_total`` does.

        Returns:
            The line total before the discount, in the unit price's currency.

        """
        with library_errors():
            return money_of(generated.expense_item_line_total_before_discount(generated_item(self)))

    def discount_amount(self) -> Money:
        """
        Work out what the discount takes off the line as the library does: the line total before the discount minus the line total.

        A line without a discount has a discount of zero. The library refuses an item it cannot hold as ``line_total`` does.

        Returns:
            The discount, in the unit price's currency.

        """
        with library_errors():
            return money_of(generated.expense_item_discount_amount(generated_item(self)))

    def line_total(self) -> Money:
        """
        Work out the line's total as the library does: the unit price times the quantity, rounded half away from zero, less the discount.

        A line whose product is below half a minor unit, or whose discount takes the whole price, totals zero. The library
        refuses an item it cannot hold, with a blank name, a unit price that is not positive, a split that does not divide the
        line total, a line total out of range, or a discount the line cannot take, with an ``ExpenseError``: a negative fixed
        discount with ``ExpenseError.NegativeDiscount``, a percentage above 100 % with
        ``ExpenseError.DiscountAboveHundredPercent``, a fixed discount larger than what it reduces with
        ``ExpenseError.DiscountAbovePrice``, and one in another currency than the unit price's with
        ``ExpenseError.CurrencyMismatch``. It refuses one that names a member by text that is not an identifier with an
        ``IdError``.

        Returns:
            The line total, in the unit price's currency.

        """
        with library_errors():
            return money_of(generated.expense_item_line_total(generated_item(self)))


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
    """The same for every participant with a share of the amount above zero."""


@final
@dataclasses.dataclass(frozen=True, slots=True)
class TipRule:
    """A tip on an expense: what it is worth and how it divides."""

    value: TipValue
    """What the tip is worth."""

    distribution: TipDistribution = TipDistribution.PROPORTIONAL
    """How the tip divides, in proportion to each participant's share unless given."""


# An expense's tip defaults to the core's no-tip rule, which the module converts when it loads, so the tip's conversions
# come before the expense.
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


NO_TIP: Final[TipRule] = tip_of(generated.tip_rule_none())


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
    """
    An expense: what it cost, who paid, how it divides, and when.

    The class takes the expense's identifier, the app's own or one ``generate_expense_id`` returns, and ``create`` generates it.
    """

    id: ExpenseId
    """The identifier, the app's own or a generated one."""

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

    tip: TipRule = NO_TIP
    """Its tip, ``NO_TIP`` for an expense without one."""

    time: TimeOfDay | None = None
    """The time of day it happened, or none."""

    category: CategoryId = UNCATEGORISED
    """Its category, the built-in one's identifier, ``UNCATEGORISED``, until it is given another."""

    description: str | None = None
    """A longer description, or none."""

    # One parameter per field of the expense but its identifier.
    @classmethod
    def create(  # ruff: ignore[too-many-arguments]
        cls,
        *,
        name: str,
        cost: Cost,
        payers: Split,
        split: ExpenseSplit,
        date: CalendarDate,
        tip: TipRule = NO_TIP,
        time: TimeOfDay | None = None,
        category: CategoryId = UNCATEGORISED,
        description: str | None = None,
    ) -> Self:
        """
        Make an expense under an identifier that ``generate_expense_id`` generates when this is called.

        An expense the group then refuses has still used up its identifier. The class itself takes the identifier instead,
        the app's own or a generated one.

        Args:
            name: What it was.
            cost: What it cost.
            payers: Who paid, and how much each.
            split: How it divides among the members who share it.
            date: The day it happened.
            tip: Its tip, ``NO_TIP`` for an expense without one.
            time: The time of day it happened, or none.
            category: Its category, the built-in one's identifier, ``UNCATEGORISED``, until it is given another.
            description: A longer description, or none.

        Returns:
            The expense.

        """
        return cls(
            id=generate_expense_id(),
            name=name,
            cost=cost,
            payers=payers,
            split=split,
            date=date,
            tip=tip,
            time=time,
            category=category,
            description=description,
        )

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
        Work out the tip in the expense's currency, zero for ``NO_TIP``.

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
    """
    A transfer of money between two members, such as a repayment.

    The class takes the transfer's identifier, the app's own or one ``generate_transfer_id`` returns, and ``create`` generates it.
    """

    id: TransferId
    """The identifier, the app's own or a generated one."""

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

    # One parameter per field of the transfer but its identifier.
    @classmethod
    def create(  # ruff: ignore[too-many-arguments]
        cls,
        *,
        sender: MemberId,
        receiver: MemberId,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay | None = None,
        note: str | None = None,
    ) -> Self:
        """
        Make a transfer under an identifier that ``generate_transfer_id`` generates when this is called.

        A transfer the group then refuses has still used up its identifier. The class itself takes the identifier instead,
        the app's own or a generated one.

        Args:
            sender: The member who sent the money.
            receiver: The member who received the money.
            cost: What was sent.
            date: The day it happened.
            time: The time of day it happened, or none.
            note: A note, or none.

        Returns:
            The transfer.

        """
        return cls(id=generate_transfer_id(), sender=sender, receiver=receiver, cost=cost, date=date, time=time, note=note)


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
    """
    A member of a group, with the details that reach and pay them.

    The class takes the member's identifier, the app's own or one ``generate_member_id`` returns, and ``create`` generates it.
    """

    id: MemberId
    """The identifier, the app's own or a generated one."""

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

    @classmethod
    def create(
        cls,
        *,
        first_name: str,
        last_name: str | None = None,
        email: Email | None = None,
        phone: PhoneNumber | None = None,
        accounts: tuple[PaymentAccount, ...] = (),
    ) -> Self:
        """
        Make a member under an identifier that ``generate_member_id`` generates when this is called.

        A member the group then refuses has still used up their identifier. The class itself takes the identifier instead,
        the app's own or a generated one.

        Args:
            first_name: The first name.
            last_name: The last name, or none.
            email: Their email address, or none.
            phone: Their phone number, or none.
            accounts: The accounts they are paid into.

        Returns:
            The member.

        """
        return cls(id=generate_member_id(), first_name=first_name, last_name=last_name, email=email, phone=phone, accounts=accounts)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Category:
    """
    A category of expenses.

    The class takes the category's identifier, the app's own or one ``generate_category_id`` returns, and ``create`` generates it.
    """

    id: CategoryId
    """The identifier, the app's own or a generated one."""

    name: str
    """The name."""

    @classmethod
    def create(cls, *, name: str) -> Self:
        """
        Make a category under an identifier that ``generate_category_id`` generates when this is called.

        A category the group then refuses has still used up its identifier. The class itself takes the identifier instead,
        the app's own or a generated one.

        Args:
            name: The name.

        Returns:
            The category.

        """
        return cls(id=generate_category_id(), name=name)


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


def generated_discount(discount: Discount) -> generated.Discount:
    """
    Convert discount into the discount the generated module holds.

    Args:
        discount: The discount.

    Returns:
        The generated discount.

    """
    match discount.value:
        case PercentageDiscount(percentage=percentage):
            value = variant(generated.DiscountValue, generated.DiscountValue.PERCENTAGE(percentage.basis_points))
        case FixedDiscount(amount=amount):
            value = variant(generated.DiscountValue, generated.DiscountValue.FIXED(generated_money(amount)))
        case _:
            assert_never(discount.value)
    return generated.Discount(value=value, scope=generated.DiscountScope[discount.scope.name])


_DISCOUNT_VALUES: dict[type[object], Callable[[Any], DiscountValue]] = {
    generated.DiscountValue.PERCENTAGE: lambda value: PercentageDiscount(Percentage(value.basis_points)),
    generated.DiscountValue.FIXED: lambda value: FixedDiscount(money_of(value.amount)),
}
"""The discount's value of each case of the generated discount's value, by its class."""


def discount_of(value: generated.Discount) -> Discount:
    """
    Convert the discount the generated module holds as value.

    Args:
        value: The generated discount.

    Returns:
        The discount.

    """
    return Discount(_DISCOUNT_VALUES[type(value.value)](value.value), DiscountScope[value.scope.name])


def generated_item(item: ExpenseItem) -> generated.ExpenseItem:
    """
    Convert item into the item the generated module holds.

    Args:
        item: The item.

    Returns:
        The generated item.

    """
    return generated.ExpenseItem(
        name=item.name,
        unit_price=generated_money(item.unit_price),
        split=generated_split(item.split),
        quantity=item.quantity.ten_thousandths,
        discount=None if item.discount is None else generated_discount(item.discount),
    )


def item_of(value: generated.ExpenseItem) -> ExpenseItem:
    """
    Convert the item the generated module holds as value.

    Args:
        value: The generated item.

    Returns:
        The item.

    """
    return ExpenseItem(
        name=value.name,
        unit_price=money_of(value.unit_price),
        split=split_of(value.split),
        quantity=Quantity(value.quantity),
        discount=None if value.discount is None else discount_of(value.discount),
    )


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
        tip=generated_tip(expense.tip),
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
        id=ExpenseId(value.id),
        name=value.name,
        cost=cost_of(value.cost),
        payers=split_of(value.payers),
        split=_EXPENSE_SPLITS[type(value.split)](value.split),
        date=date_of(value.date),
        tip=tip_of(value.tip),
        time=None if value.time is None else time_of(value.time),
        category=CategoryId(value.category),
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
        id=TransferId(value.id),
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
            details = variant(generated.PaymentAccountDetails, generated.PaymentAccountDetails.IBAN(str(iban), iban.country_code))
        case UsernamePaymentAccountDetails(username=username):
            details = variant(generated.PaymentAccountDetails, generated.PaymentAccountDetails.USERNAME(str(username)))
        case _:
            assert_never(account.details)
    return generated.PaymentAccount(name=account.name, details=details)


_PAYMENT_ACCOUNT_DETAILS: dict[type[object], Callable[[Any], PaymentAccountDetails]] = {
    generated.PaymentAccountDetails.IBAN: lambda value: IbanPaymentAccountDetails(iban_of(value.iban, value.country_code)),
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
        phone=None if member.phone is None else generated_phone_number(member.phone),
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
        id=MemberId(value.id),
        first_name=value.first_name,
        last_name=value.last_name,
        email=None if value.email is None else Email(value.email),
        phone=None if value.phone is None else phone_number_of(value.phone),
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
    return Category(id=CategoryId(value.id), name=value.name)
