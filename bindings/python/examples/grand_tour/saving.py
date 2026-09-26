"""Print section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed."""

from typing import assert_never

from eranos import Account, Group, GroupError, IbanAccountDetails, UsernameAccountDetails
from examples.grand_tour.tour import Categories, People
from examples.report import message, print_lines, print_table, refusal, section


def _details(account: Account) -> str:
    """
    Write the details of account: its IBAN or its username.

    Args:
        account: The account.

    Returns:
        The details, as the library holds them.

    """
    details = account.details
    match details:
        case IbanAccountDetails(iban=iban):
            return str(iban)
        case UsernameAccountDetails(username=username):
            return str(username)
        case _:
            assert_never(details)


def _refused(text: str) -> GroupError:
    """
    Load text, a saved group changed so that the library refuses it.

    Args:
        text: The changed saved group.

    Returns:
        The library's error.

    """
    refused = refusal(lambda: Group.load(text))
    assert isinstance(refused, GroupError), 'the change is refused'
    return refused


def saving_and_loading(group: Group, people: People, categories: Categories) -> None:
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
    with Group.load(saved) as loaded:
        print_lines(
            'Loading replays every entry through the same checks as adding it and rebuilds the',
            'totals: the loaded group equals the saved one. Reading a SavedGroup and loading it',
            'with into_group reports a refusal as a typed error instead of a message:',
        )
        tampered = (
            ('"format": 3', saved.replace('"format": 2', '"format": 3', 1)),
            ('a blank group name', saved.replace('"name": "Lisbon and Porto"', '"name": " "', 1)),
            ('Lodging renamed FOOD', saved.replace('"name": "Lodging"', '"name": "FOOD"', 1)),
        )
        refusals: list[tuple[str, GroupError]] = []
        for change, text in tampered:
            assert text != saved, f'{change} changes the file'
            refusals.append((change, _refused(text)))
        print_table(('Change to the file', 'Message'), [(change, message(refused)) for change, refused in refusals], 2)

        assert loaded.save() == saved
        assert loaded.summary() == group.summary()
        assert [refused.failure for _, refused in refusals] == [
            GroupError.UnsupportedFormat(format=3),
            GroupError.EmptyName(),
            GroupError.DuplicateCategoryName(existing=categories.food),
        ]
