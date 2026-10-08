"""
Print an itemised restaurant bill: every line is split among the people who had it, and quantities multiply the unit price.

The tip is shared either in proportion to what each person ordered or equally; then a line weighed on a scale, and discounts off
the line and off each unit.

The port of the Rust example ``restaurant_bill``, which prints the same report.

"""

import copy
import dataclasses
from typing import ClassVar, final, override

from eranos import (
    CalendarDate,
    CurrencyCode,
    Discount,
    DiscountScope,
    EqualSplit,
    Expense,
    ExpenseShares,
    ExpenseGroup,
    ExpenseItem,
    FixedDiscount,
    ItemisedExpenseSplit,
    LocalCost,
    Member,
    MemberId,
    MemberParts,
    Money,
    Parts,
    PartsSplit,
    Percentage,
    PercentageDiscount,
    PercentageTip,
    Quantity,
    TipRule,
    TipDistribution,
    generate_expense_id,
)
from examples.report import Row, names, print_lines, print_table, refusal, section, title


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class LineRow(Row):
    """A row of the table of the receipt: a line, who had it, and what it costs."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Item', 'Shared by', 'Quantity', 'Unit price', 'Line total')

    name: str
    """What the line is."""

    participants: str
    """Who had it."""

    quantity: Quantity
    """How many were ordered."""

    unit_price: Money
    """What one costs."""

    line_total: Money
    """The unit price times the quantity, as the library works it out."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.participants, str(self.quantity), str(self.unit_price), str(self.line_total)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class DiscountRow(Row):
    """A row of the table of the second round: a line, what it costs before its discount, the discount, and what is left."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Item', 'Quantity', 'Unit price', 'Before discount', 'Discount', 'Line total')

    name: str
    """What the line is."""

    quantity: Quantity
    """How many were ordered, or how much was weighed."""

    unit_price: Money
    """What one costs."""

    line_total_before_discount: Money
    """The unit price times the quantity, as the library works it out."""

    discount_amount: Money
    """What the discount takes off the line, as the library works it out."""

    line_total: Money
    """What is left to pay for the line, as the library works it out."""

    @override
    def cells(self) -> tuple[str, ...]:
        return (
            self.name,
            str(self.quantity),
            str(self.unit_price),
            str(self.line_total_before_discount),
            str(self.discount_amount),
            str(self.line_total),
        )


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class TipRow(Row):
    """A row of the table of the tip: what a member ordered, and the tip and the total of each way of sharing it."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Ordered', 'Tip by consumption', 'Pays', 'Tip shared equally', 'Pays')

    name: str
    """The member's first name."""

    subtotal: Money
    """What the member ordered."""

    proportional_tip: Money
    """The member's part of the tip, shared in proportion to what each person ordered."""

    proportional_total: Money
    """What the member pays with the tip shared in proportion to what each person ordered."""

    equal_tip: Money
    """The member's part of the tip, shared equally."""

    equal_total: Money
    """What the member pays with the tip shared equally."""

    @override
    def cells(self) -> tuple[str, ...]:
        return (
            self.name,
            str(self.subtotal),
            str(self.proportional_tip),
            str(self.proportional_total),
            str(self.equal_tip),
            str(self.equal_total),
        )


def euros(text: str) -> Money:
    """
    Parse the amount written text, in euros.

    Args:
        text: The amount's text, such as ``"8.00"``.

    Returns:
        The amount.

    """
    return Money.parse(text, CurrencyCode('EUR'))


def receipt(ana: MemberId, bob: MemberId, carol: MemberId) -> tuple[ExpenseItem, ...]:
    """
    List the lines of the receipt, each with the members who had it.

    Args:
        ana: Ana.
        bob: Bob.
        carol: Carol.

    Returns:
        The lines.

    """
    return (
        ExpenseItem(name='Ciorbă', unit_price=euros('8.00'), split=EqualSplit((ana,))),
        ExpenseItem(name='Sarmale', unit_price=euros('14.00'), split=EqualSplit((bob, carol)), quantity=Quantity.whole(2)),
        ExpenseItem(
            name='Papanași', unit_price=euros('9.00'), split=PartsSplit((MemberParts(ana, Parts.parse('1')), MemberParts(carol, Parts.parse('2'))))
        ),
        ExpenseItem(name='Lemonade', unit_price=euros('3.00'), split=EqualSplit((ana, bob, carol)), quantity=Quantity.whole(3)),
    )


def second_round(ana: MemberId, bob: MemberId, carol: MemberId) -> tuple[ExpenseItem, ...]:
    """
    List a second round: telemea weighed on a scale with 10 % off the line, three lemonades with 1.00 off each, and papanași on the house.

    Args:
        ana: Ana.
        bob: Bob.
        carol: Carol.

    Returns:
        The lines.

    """
    # The scale reports the weight as a float.
    weight = Quantity.from_float(0.35)
    return (
        ExpenseItem(
            name='Telemea',
            unit_price=euros('42.00'),
            split=EqualSplit((ana, bob)),
            quantity=weight,
            discount=Discount(PercentageDiscount(Percentage.parse('10'))),
        ),
        ExpenseItem(
            name='Lemonade',
            unit_price=euros('3.00'),
            split=EqualSplit((ana, bob, carol)),
            quantity=Quantity.whole(3),
            discount=Discount(FixedDiscount(euros('1.00')), DiscountScope.UNIT),
        ),
        ExpenseItem(
            name='Papanași', unit_price=euros('9.00'), split=EqualSplit((carol,)), discount=Discount(PercentageDiscount(Percentage.parse('100')))
        ),
    )


def preview(group: ExpenseGroup, expense: Expense) -> ExpenseShares:
    """
    Work out how expense would divide if it were added to group, on a copy, so the group itself is unchanged.

    The layer's group has no clone, so the copy is the group's saved form, loaded again.

    Args:
        group: The group.
        expense: The expense, under an identifier the group does not hold.

    Returns:
        How the expense divides.

    """
    with ExpenseGroup.load(group.save()) as loaded:
        shares = loaded.expense_shares(loaded.add_expense(expense))
    assert shares is not None, 'the expense was just added'
    return shares


def consumed(shares: ExpenseShares, member: MemberId) -> Money:
    """
    Find what member consumed of the expense that shares divides.

    Args:
        shares: How the expense divides.
        member: The member.

    Returns:
        The member's share.

    """
    found = next((share.share for share in shares.members if share.member == member), None)
    assert found is not None, 'every member ordered'
    return found


def bill(untipped: Expense, distribution: TipDistribution) -> Expense:
    """
    Make the dinner with its tip of 10 %, shared by distribution, under an identifier of its own, which it generates.

    Args:
        untipped: The dinner without a tip.
        distribution: How the tip is shared.

    Returns:
        The dinner with its tip.

    """
    return copy.replace(untipped, id=generate_expense_id(), tip=TipRule(PercentageTip(Percentage.parse('10')), distribution))


def tip_row(group: ExpenseGroup, member: MemberId, ordered: ExpenseShares, by_consumption: ExpenseShares, even: ExpenseShares) -> TipRow:
    """
    Write the row of member in the table of the tip: what they ordered, then the tip and the total of each way of sharing it.

    Args:
        group: The group, which names the member.
        member: The member.
        ordered: How the dinner divides without its tip.
        by_consumption: How it divides with the tip shared in proportion to what each person ordered.
        even: How it divides with the tip shared equally.

    Returns:
        The row.

    """
    subtotal = consumed(ordered, member)
    proportional_total = consumed(by_consumption, member)
    equal_total = consumed(even, member)
    return TipRow(
        name=names(group, (member,)),
        subtotal=subtotal,
        proportional_tip=proportional_total.minus(subtotal),
        proportional_total=proportional_total,
        equal_tip=equal_total.minus(subtotal),
        equal_total=equal_total,
    )


def weighed_lines_and_discounts(ana: MemberId, bob: MemberId, carol: MemberId) -> None:
    """
    Print section 4: the second round's lines, each with its totals before and after its discount, and a discount larger than its line, refused.

    Args:
        ana: Ana.
        bob: Bob.
        carol: Carol.

    """
    section('4. Weighed lines and discounts')
    telemea, lemonade, papanasi = second_round(ana, bob, carol)
    print_table(
        DiscountRow,
        [
            DiscountRow(
                name=item.name,
                quantity=item.quantity,
                unit_price=item.unit_price,
                line_total_before_discount=item.line_total_before_discount(),
                discount_amount=item.discount_amount(),
                line_total=item.line_total(),
            )
            for item in (telemea, lemonade, papanasi)
        ],
        1,
    )
    nine, ten = euros('9.00'), euros('10.00')
    # The layer checks an item only when the library reads it, so working out its line total refuses it where Rust's build does.
    refused = refusal(ExpenseItem(name='Papanași', unit_price=nine, split=EqualSplit((carol,)), discount=Discount(FixedDiscount(ten))).line_total)
    assert refused is not None, 'the discount must be refused'
    print_lines(
        f'The scale reports {telemea.quantity} kg as a double just below it; the quantity reads its shortest',
        f'form, {telemea.quantity}, so the telemea costs exactly {telemea.line_total_before_discount()} before its discount. Taking {ten}',
        f'off a {nine} line is refused when the item is built:',
        f'  error: {refused}',
    )

    # 10 % of 14.70 is 1.47; 1.00 off each of three lemonades is 3.00; the papanași on the house leave a free line.
    assert telemea.line_total_before_discount() == euros('14.70')
    assert telemea.discount_amount() == euros('1.47')
    assert telemea.line_total() == euros('13.23')
    assert lemonade.discount_amount() == euros('3.00')
    assert lemonade.line_total() == euros('6.00')
    assert papanasi.discount_amount() == euros('9.00')
    assert papanasi.line_total() == euros('0.00')


def main() -> None:
    """Print the report, checking its key numbers."""
    group = ExpenseGroup.create("Dinner at Caru' cu Bere", CurrencyCode('EUR'))
    ana = group.add_member(Member.create(first_name='Ana'))
    bob = group.add_member(Member.create(first_name='Bob'))
    carol = group.add_member(Member.create(first_name='Carol'))
    day = CalendarDate(2026, 9, 23)

    title(f'{group.name()}: an itemised bill')
    print_lines(
        'Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,',
        'every line of the receipt is split among the people who had it.',
    )

    section('1. The receipt')
    lines = receipt(ana, bob, carol)
    print_table(
        LineRow,
        [
            LineRow(
                name=item.name,
                participants=names(group, item.split.members),
                quantity=item.quantity,
                unit_price=item.unit_price,
                line_total=item.line_total(),
            )
            for item in lines
        ],
        2,
    )
    print_lines(
        'The papanași are split by parts: Carol ate two of the three. The line totals add up',
        "to 54.00 EUR, and the expense's amount must match them exactly.",
    )

    section('2. Who ordered what, and the tip')
    untipped = Expense.create(name='Dinner', cost=LocalCost(euros('54.00')), payers=EqualSplit((ana,)), split=ItemisedExpenseSplit(lines), date=day)
    ordered = preview(group, untipped)
    proportional = group.add_expense(bill(untipped, TipDistribution.PROPORTIONAL))
    by_consumption = group.expense_shares(proportional)
    assert by_consumption is not None, 'the bill was added'
    even = preview(group, bill(untipped, TipDistribution.EQUAL))
    print_lines(
        f'Ana paid the bill plus a 10 % tip ({by_consumption.tip}). A tip can be shared in proportion to what',
        'each person ordered, or equally; both are shown:',
    )
    print_table(
        TipRow,
        [tip_row(group, member, ordered, by_consumption, even) for member in (ana, bob, carol)],
        1,
    )
    print_lines('By consumption, Carol, who ordered the most, adds the largest part of the tip;', 'shared equally, everyone adds the same 1.80.')

    section('3. A total that does not match is refused')
    # The layer builds an expense only when a group adds it, so the group's add_expense refuses it where Rust's build does.
    mismatched = copy.replace(untipped, id=generate_expense_id(), cost=LocalCost(euros('55.00')))
    refused = refusal(lambda: group.add_expense(mismatched))
    assert refused is not None, 'the mismatch must be refused'
    print_lines('An amount of 55.00 EUR with these lines is refused when the expense is built:', f'  error: {refused}')

    weighed_lines_and_discounts(ana, bob, carol)

    # Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to each;
    # an equal one 1.80 to everyone.
    assert consumed(by_consumption, ana) == euros('15.40')
    assert consumed(by_consumption, bob) == euros('18.70')
    assert consumed(by_consumption, carol) == euros('25.30')
    assert consumed(even, carol) == euros('24.80')
    assert even.total == euros('59.40')


if __name__ == '__main__':
    main()
