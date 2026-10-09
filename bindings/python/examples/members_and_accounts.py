"""
Print a flat's members with their contact details and accounts, what validation accepts of each, and a flatmate who moves out.

The port of the Rust example ``members_and_accounts``, which prints the same report.

"""

import dataclasses
from collections.abc import Callable
from typing import ClassVar, assert_never, final, override

from eranos import (
    PaymentAccount,
    Balance,
    CalendarDate,
    CurrencyCode,
    Email,
    EqualSplit,
    EranosError,
    Expense,
    ExpenseGroup,
    ExpenseGroupError,
    Iban,
    IbanPaymentAccountDetails,
    LocalCost,
    Member,
    MemberId,
    Money,
    OwesBalance,
    PhoneNumber,
    Username,
    UsernamePaymentAccountDetails,
    WholeExpenseSplit,
)
from examples.report import Row, print_lines, print_table, refusal, section, status, title, totals


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class MemberRow(Row):
    """A row of the table of members: a member's full name, contact details, and accounts."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Email', 'Phone', 'Accounts')

    name: str
    """The member's full name."""

    email: str
    """The member's email, or nothing."""

    phone: str
    """The member's phone number, grouped for reading, or nothing."""

    accounts: str
    """The member's accounts, a line each."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.email, self.phone, self.accounts


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class CheckRow(Row):
    """A row of the table of what validation accepts."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Input', 'Checked as', 'Result')

    input: str
    """The text checked."""

    kind: str
    """What the text is checked as, such as ``"IBAN"``."""

    result: str
    """The value as the library writes it, or the library's message."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.input, self.kind, self.result


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class BalanceRow(Row):
    """A row of the table of balances: whether a member is active, what they consumed, and where they stand."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Member', 'Status', 'Consumed', 'Balance')

    name: str
    """The member's first name."""

    status: str
    """Whether the member is active or archived."""

    consumed: Money
    """What the member consumed."""

    balance: Balance
    """Where the member stands."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.name, self.status, str(self.consumed), str(self.balance)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Flatmates:
    """The two flatmates: Ana, who gives every detail, and Bob, who moves out."""

    ana: MemberId
    """Ana, who pays the rent."""

    bob: MemberId
    """Bob, who moves out."""


def describe(account: PaymentAccount) -> str:
    """
    Write an account with its label and its details.

    Args:
        account: The account.

    Returns:
        The text, such as ``"ING: RO49AAAA1B31007593840000"``.

    """
    match account.details:
        case IbanPaymentAccountDetails(iban=iban):
            details = str(iban)
        case UsernamePaymentAccountDetails(username=username):
            details = str(username)
        case _:
            assert_never(account.details)
    return f'{account.name}: {details}'


def checked[Value](text: str, kind: str, parse: Callable[[str], Value], shown: Callable[[Value], str] = str) -> CheckRow:
    """
    Check text as kind with parse.

    Args:
        text: The text to check.
        kind: What the text is checked as, such as ``"IBAN"``.
        parse: What checks the text, such as ``Iban``.
        shown: What writes the value for the table: its text, unless the table shows another form.

    Returns:
        The row of the table of what validation accepts, whose result is ``valid:`` with the value as the library writes
        it, or ``refused:`` with the library's message.

    """
    try:
        value = parse(text)
    except EranosError as error:
        return CheckRow(input=text, kind=kind, result=f'refused: {error}')
    return CheckRow(input=text, kind=kind, result=f'valid: {shown(value)}')


def print_members(group: ExpenseGroup) -> None:
    """
    Print each member's full name, email, phone, and accounts, an account per line.

    Args:
        group: The group.

    """
    print_table(
        MemberRow,
        [
            MemberRow(
                name=' '.join(name for name in (member.first_name, member.last_name) if name is not None),
                email='' if member.email is None else str(member.email),
                phone='' if member.phone is None else member.phone.international,
                accounts='\n'.join(describe(account) for account in member.accounts),
            )
            for member in group.members()
        ],
        4,
    )


def print_balances(group: ExpenseGroup) -> None:
    """
    Print whether each member is active or archived, what they consumed, and where they stand.

    Args:
        group: The group.

    """
    summary = group.summary()
    print_table(
        BalanceRow,
        [
            BalanceRow(
                name=member.first_name,
                status=status(group.is_archived(member.id)),
                consumed=found.share,
                balance=found.balance,
            )
            for member in group.members()
            if (found := totals(member.id, summary)) is not None
        ],
        2,
    )


def members(group: ExpenseGroup) -> Flatmates:
    """
    Add Ana, with every detail a member can have, and Bob, with a first name only, and print them.

    Args:
        group: The group.

    Returns:
        The flatmates.

    """
    section('1. Members')
    ana = group.add_member(
        Member.create(
            first_name='Ana',
            last_name='Popescu',
            email=Email('ana@example.com'),
            phone=PhoneNumber('+40 712 345 678'),
            accounts=(
                PaymentAccount('ING', IbanPaymentAccountDetails(Iban('ro49 aaaa 1b31 0075 9384 0000'))),
                PaymentAccount('Revolut', UsernamePaymentAccountDetails(Username('@ana'))),
            ),
        )
    )
    bob = group.add_member(Member.create(first_name='Bob'))
    print_members(group)
    print_lines(
        'The phone number was stored in E.164 form, +40712345678, and the table shows it',
        'grouped for reading; the IBAN was stored in its electronic form, uppercase and',
        'without spaces, the way banking apps accept it.',
    )
    return Flatmates(ana=ana, bob=bob)


def validation() -> None:
    """Print what validation makes of IBANs, emails, and phone numbers, valid and mistyped."""
    section('2. What validation accepts')
    print_lines(
        'IBANs are checked against the SWIFT registry: the country, its length, which',
        'positions hold digits and which letters, and the mod-97 check digits, which catch',
        'any single mistyped character. Emails are checked against the rules of RFC 5321',
        'and 5322, which refuse the doubled dots and stray commas of hurried typing, and',
        "phone numbers against their country's numbering plan, stored in E.164 form.",
    )
    print_table(
        CheckRow,
        [
            checked('ro49 aaaa 1b31 0075 9384 0000', 'IBAN', Iban),
            checked('RO49 AAAA 1B31 0075 9384 0001', 'IBAN', Iban),
            checked('RO49 AAAA 1B31 0075', 'IBAN', Iban),
            checked('DE46 37O4 0044 0532 0130 00', 'IBAN', Iban),
            checked('ana@example.com', 'email', Email),
            checked('ana.example.com', 'email', Email),
            checked('ana..popescu@example.com', 'email', Email),
            checked('+40 (712) 345-678', 'phone', PhoneNumber, lambda phone: phone.e164),
            checked('0712 345 678', 'phone', PhoneNumber, lambda phone: phone.e164),
            checked('+40 612 345 678', 'phone', PhoneNumber, lambda phone: phone.e164),
            checked('+44 (0)20 7946 0958', 'phone', PhoneNumber, lambda phone: phone.e164),
        ],
        3,
    )


def moving_out(group: ExpenseGroup, flatmates: Flatmates) -> None:
    """
    Archive Bob after he shares the rent, print the balances, and print what the group refuses of him.

    Args:
        group: The group.
        flatmates: Ana, who pays the rent, and Bob, who moves out.

    """
    section('3. A flatmate moves out')
    ana, bob = flatmates.ana, flatmates.bob
    euro = CurrencyCode('EUR')
    both = WholeExpenseSplit(EqualSplit((ana, bob)))
    rent = group.add_expense(
        Expense.create(name='Rent', cost=LocalCost(Money.parse('900.00', euro)), payers=EqualSplit((ana,)), split=both, date=CalendarDate(2026, 9, 1))
    )
    group.archive_member(bob)
    print_lines(
        "Bob shared September's rent, then moved out. He is archived, not removed: his",
        'history and his balance stay, and he can still pay Ana back.',
    )
    print_balances(group)

    utilities = Expense.create(
        name='Utilities', cost=LocalCost(Money.parse('60.00', euro)), payers=EqualSplit((ana,)), split=both, date=CalendarDate(2026, 10, 1)
    )
    refused = refusal(lambda: group.add_expense(utilities))
    assert isinstance(refused, ExpenseGroupError), 'the archived member must be refused'
    print_lines("October's utilities cannot include him:", f'  error: {refused}')
    removal = refusal(lambda: group.remove_member(bob))
    assert isinstance(removal, ExpenseGroupError), 'the referenced member must stay'
    print_lines('Nor can he be removed while the rent references him:', f'  error: {removal}')

    assert refused.failure == ExpenseGroupError.ArchivedMember(bob)
    assert isinstance(removal.failure, ExpenseGroupError.MemberInUse)
    bob_totals = totals(bob, group.summary())
    assert bob_totals is not None
    assert bob_totals.balance == OwesBalance(Money.parse('450.00', euro))
    assert group.expense(rent) is not None


def main() -> None:
    """Print the report, checking its key numbers."""
    group = ExpenseGroup.create('Flat 4B', CurrencyCode('EUR'))
    title('Flat 4B: members and their accounts')
    print_lines(
        'Ana and Bob share a flat. A member needs only a first name; the last name,',
        'email, phone, and the accounts they can be paid to are optional.',
    )
    flatmates = members(group)
    validation()
    moving_out(group, flatmates)


if __name__ == '__main__':
    main()
