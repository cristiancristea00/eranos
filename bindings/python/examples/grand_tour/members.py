"""Print sections 1 to 3 of the tour: the group and its details, the members with their contact details and accounts, and the categories."""

import copy
import dataclasses
from collections.abc import Callable
from typing import ClassVar, assert_never, final, override

from eranos import (
    Account,
    Category,
    Currency,
    Email,
    EranosError,
    Group,
    GroupError,
    Iban,
    IbanAccountDetails,
    Language,
    Member,
    PhoneNumber,
    Username,
    UsernameAccountDetails,
)
from examples.grand_tour.tour import Categories, People, RefusalRow, cell
from examples.report import Row, message, print_lines, print_table, refusal, section


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class DetailRow(Row):
    """A row of the table of the group's details."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Detail', 'Value')

    detail: str
    """Which detail it is."""

    value: str
    """The detail's value."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.detail, self.value


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
    """A row of the table of what the library checks in a member's details."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Input', 'Checked as', 'Result')

    input: str
    """The text checked, in quotes."""

    kind: str
    """What the text is checked as, such as ``"IBAN"``."""

    result: str
    """The value as the library writes it, or the library's message."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.input, self.kind, self.result


def the_group() -> Group:
    """
    Print section 1: make the group, change its details, and show them.

    Returns:
        The group.

    """
    section('1. The group')
    group = Group.create('Lisbon', Currency('EUR'), description='A week by the Atlantic', language=Language('pt-pt'))
    blank = refusal(lambda: group.set_name('   '))
    group.set_name('Lisbon and Porto')
    group.set_description('A week by the Atlantic, with a day in Porto')
    language = group.language()
    tag = '' if language is None else str(language)
    print_table(
        DetailRow,
        [
            DetailRow(detail='Name', value=group.name()),
            DetailRow(detail='Currency', value=str(group.currency())),
            DetailRow(detail='Language', value=tag),
            DetailRow(detail='Description', value=group.description() or ''),
        ],
        2,
    )
    print_lines(
        'The group started as Lisbon and was renamed when a day in Porto joined the plan.',
        f'Every total is kept in its currency, {group.currency()}. The language is a BCP 47 tag the apps',
        f'show the group in, stored in canonical form: pt-pt became {tag}. A blank name is',
        'refused:',
        f'  error: {message(blank)}',
    )

    assert isinstance(blank, GroupError)
    assert blank.failure == GroupError.EmptyName()
    assert group.name() == 'Lisbon and Porto'
    assert tag == 'pt-PT'
    return group


def _describe(account: Account) -> str:
    """
    Describe account: its label, its kind, and its details.

    Args:
        account: The account.

    Returns:
        The description, such as ``"Revolut, username @ana"``.

    """
    details = account.details
    match details:
        case IbanAccountDetails(iban=iban):
            return f'{account.name}, IBAN {iban}'
        case UsernameAccountDetails(username=username):
            return f'{account.name}, username {username}'
        case _:
            assert_never(details)


def _print_members(group: Group) -> None:
    """
    Print every member with their contact details and accounts.

    Args:
        group: The group.

    """
    print_table(
        MemberRow,
        [
            MemberRow(
                name=' '.join(part for part in (member.first_name, member.last_name) if part is not None),
                email='' if member.email is None else str(member.email),
                phone='' if member.phone is None else member.phone.international(),
                accounts=cell(_describe(account) for account in member.accounts),
            )
            for member in group.members()
        ],
        4,
    )


def _outcome[Value](make: Callable[[], Value], shown: Callable[[Value], str]) -> str:
    """
    Write ``valid:`` with what shown writes of the value make returns, or ``refused:`` with the library's message.

    Args:
        make: The call that checks the value.
        shown: What writes the value.

    Returns:
        The outcome, such as ``"valid: ana@example.com"``.

    """
    try:
        value = make()
    except EranosError as error:
        return f'refused: {error}'
    return f'valid: {shown(value)}'


def _iban(text: str) -> CheckRow:
    """
    Check text as an IBAN.

    Args:
        text: The IBAN as entered.

    Returns:
        The row of the check.

    """
    return CheckRow(input=f'"{text}"', kind='IBAN', result=_outcome(lambda: Iban(text), lambda iban: f'{iban}, country {iban.country_code()}'))


def _phone(text: str) -> CheckRow:
    """
    Check text as a phone number.

    Args:
        text: The number as entered.

    Returns:
        The row of the check.

    """
    return CheckRow(
        input=f'"{text}"', kind='phone', result=_outcome(lambda: PhoneNumber(text), lambda phone: f'{phone.international()}, stored as {phone}')
    )


def _validation(group: Group) -> None:
    """
    Print what the library checks in a member's details: IBANs, emails, phone numbers, and required text.

    Args:
        group: The group, which checks a member's first name when it adds them.

    """
    print_lines(
        'IBANs are checked against the SWIFT registry and their mod-97 check digits,',
        "emails against RFC 5321 and 5322, and phone numbers against their country's",
        'numbering plan. Required text must not be blank:',
    )
    print_table(
        CheckRow,
        [
            _iban('ro49 aaaa 1b31 0075 9384 0000'),
            _iban('RO49 AAAA 1B31 0075 9384 0001'),
            CheckRow(input='"ana..popescu@example.com"', kind='email', result=_outcome(lambda: Email('ana..popescu@example.com'), str)),
            _phone('+351 912 345 678'),
            _phone('+40 612 345 678'),
            CheckRow(input='"   "', kind='username', result=_outcome(lambda: Username('   '), str)),
            CheckRow(input='"  "', kind='first name', result=_outcome(lambda: group.add_member(Member(first_name='  ')), str)),
        ],
        3,
    )


def members(group: Group) -> People:
    """
    Print section 2: add the members, edit one, and show how the library checks their details.

    Args:
        group: The group.

    Returns:
        The members.

    """
    section('2. Members')
    ana = group.add_member(
        Member(
            first_name='Ana',
            last_name='Popescu',
            email=Email('ana@example.com'),
            phone=PhoneNumber('+40 712 345 678'),
            accounts=(
                Account('ING', IbanAccountDetails(Iban('RO49 AAAA 1B31 0075 9384 0000'))),
                Account('Revolut', UsernameAccountDetails(Username('@ana'))),
            ),
        )
    )
    bob = group.add_member(Member(first_name='Bob'))
    carol = group.add_member(Member(first_name='Carol'))
    dan = group.add_member(Member(first_name='Dan'))
    eva = group.add_member(Member(first_name='Eva'))
    filip = group.add_member(Member(first_name='Filip'))
    # An edit starts from a copy of the member, which keeps its identifier.
    stored = group.member(bob)
    assert stored is not None, 'Bob is a member'
    edited = copy.replace(stored, last_name='Ionescu', email=Email('bob@example.com'))
    before = group.replace_member(edited)
    _print_members(group)
    print_lines(
        'Only a first name is required. Ana gave every detail; Bob added his last name and',
        'email later, through an edit that kept his identifier:',
        f'  {bob}',
        "Eva is Carol's daughter, and Filip was added by mistake.",
    )
    _validation(group)

    assert before.last_name is None
    assert group.member(bob) == edited
    found = group.member(ana)
    assert found is not None
    assert len(found.accounts) == 2
    return People(ana=ana, bob=bob, carol=carol, dan=dan, eva=eva, filip=filip)


def add_categories(group: Group) -> Categories:
    """
    Print section 3: add the categories, rename one, and show what the group refuses.

    Args:
        group: The group.

    Returns:
        The categories the group adds.

    """
    section('3. Categories')
    food = group.add_category(Category(name='Food'))
    lodging = group.add_category(Category(name='Lodging'))
    transport = group.add_category(Category(name='Transport'))
    homonym = refusal(lambda: group.add_category(Category(name='FOOD')))
    stored = group.category(transport)
    assert stored is not None, 'Transport was added'
    group.replace_category(copy.replace(stored, name='Getting around'))
    uncategorised = group.categories()[0]
    built_in = refusal(lambda: group.replace_category(copy.replace(uncategorised, name='Other')))
    listed = ', '.join(category.name for category in group.categories())
    print_lines(
        'Every group has the built-in Uncategorised; this one adds Food, Lodging, and',
        'Transport, then renames Transport, which keeps its identifier. In identifier order:',
        f'  {listed}',
        'Names are compared caselessly, and the built-in category keeps its name:',
    )
    print_table(
        RefusalRow,
        [
            RefusalRow(change='Add a category named FOOD', error=message(homonym)),
            RefusalRow(change='Rename Uncategorised to Other', error=message(built_in)),
        ],
        2,
    )

    assert isinstance(homonym, GroupError)
    assert homonym.failure == GroupError.DuplicateCategoryName(existing=food)
    assert isinstance(built_in, GroupError)
    assert built_in.failure == GroupError.BuiltInCategory()
    renamed = group.category(transport)
    assert renamed is not None
    assert renamed.name == 'Getting around'
    return Categories(food=food, lodging=lodging, transport=transport)
