"""Print section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed."""

import dataclasses
from typing import ClassVar, assert_never, final, override

from eranos import PaymentAccount, ExpenseGroup, ExpenseGroupError, IbanPaymentAccountDetails, UsernamePaymentAccountDetails
from examples.grand_tour.tour import Categories, People
from examples.report import Row, print_lines, print_table, refusal, section


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Tampering:
    """A change made to a saved group, and the text it leaves."""

    change: str
    """What was changed, as the report names it."""

    text: str
    """The saved group with the change."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class LoadRow(Row):
    """A row of the table of changed files: the change, and the error the library refuses the file with."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Change to the file', 'Message')

    change: str
    """What was changed."""

    message: ExpenseGroupError
    """The library's error."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.change, str(self.message)


def _details(account: PaymentAccount) -> str:
    """
    Write the details of account: its IBAN or its username.

    Args:
        account: The account.

    Returns:
        The details, as the library holds them.

    """
    details = account.details
    match details:
        case IbanPaymentAccountDetails(iban=iban):
            return str(iban)
        case UsernamePaymentAccountDetails(username=username):
            return str(username)
        case _:
            assert_never(details)


def _refused(text: str) -> ExpenseGroupError:
    """
    Load text, a saved group changed so that the library refuses it.

    Args:
        text: The changed saved group.

    Returns:
        The library's error.

    """
    refused = refusal(lambda: ExpenseGroup.load(text))
    assert isinstance(refused, ExpenseGroupError), 'the change is refused'
    return refused


def saving_and_loading(group: ExpenseGroup, people: People, categories: Categories) -> None:
    """
    Print section 12: save the group, load it again, and show what the library refuses in a changed file.

    Args:
        group: The group.
        people: The members of the trip.
        categories: The categories the group adds.

    """
    section('12. Saving and loading')
    saved = group.save()
    print_lines(
        f'The group saves to {len(saved.encode())} bytes of JSON: its details and entries, never the totals.',
        "Ana's accounts are saved in clear text, as the library holds them:",
    )
    ana = group.member(people.ana)
    assert ana is not None, 'Ana is a member'
    for account in ana.accounts:
        details = _details(account)
        assert details in saved, f'{details} is saved as it is'
        print_lines(f'  {account.name}: {details}')
    with ExpenseGroup.load(saved) as loaded:
        print_lines(
            'Loading replays every entry through the same checks as adding it and rebuilds the',
            'totals: the loaded group equals the saved one. Reading a SavedExpenseGroup and',
            'loading it with into_expense_group reports a refusal as a typed error, not as text:',
        )
        tampered = (
            Tampering(change='"format": 4', text=saved.replace('"format": 3', '"format": 4', 1)),
            Tampering(change='a blank group name', text=saved.replace('"name": "Lisbon and Porto"', '"name": " "', 1)),
            Tampering(change='Lodging renamed FOOD', text=saved.replace('"name": "Lodging"', '"name": "FOOD"', 1)),
        )
        rows: list[LoadRow] = []
        for tampering in tampered:
            assert tampering.text != saved, f'{tampering.change} changes the file'
            rows.append(LoadRow(change=tampering.change, message=_refused(tampering.text)))
        print_table(LoadRow, rows, 2)

        assert loaded.save() == saved
        assert loaded.summary() == group.summary()
        assert [row.message.failure for row in rows] == [
            ExpenseGroupError.UnsupportedFormat(format=4),
            ExpenseGroupError.EmptyName(),
            ExpenseGroupError.DuplicateCategoryName(existing=categories.food),
        ]
