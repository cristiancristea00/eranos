"""Check that the layer converts every case the generator wrote."""

import asyncio
import enum
import inspect
import re

import pytest

from eranos import EranosError, InvalidJsonError, _entries, _errors, _totals
from eranos._errors import library_errors
from eranos._internal import eranos as generated


FAILURES = {
    'Account': (generated.AccountFailure, _errors._ACCOUNT_FAILURES, _errors.AccountError),
    'Calendar': (generated.CalendarFailure, _errors._CALENDAR_FAILURES, _errors.CalendarError),
    'Category': (generated.CategoryFailure, _errors._CATEGORY_FAILURES, _errors.CategoryError),
    'Contact': (generated.ContactFailure, _errors._CONTACT_FAILURES, _errors.ContactError),
    'Expense': (generated.ExpenseFailure, _errors._EXPENSE_FAILURES, _errors.ExpenseError),
    'Group': (generated.GroupFailure, _errors._GROUP_FAILURES, _errors.GroupError),
    'Id': (generated.IdFailure, _errors._ID_FAILURES, _errors.IdError),
    'Language': (generated.LanguageFailure, _errors._LANGUAGE_FAILURES, _errors.LanguageError),
    'Member': (generated.MemberFailure, _errors._MEMBER_FAILURES, _errors.MemberError),
    'Money': (generated.MoneyFailure, _errors._MONEY_FAILURES, _errors.MoneyError),
    'Settlement': (generated.SettlementFailure, _errors._SETTLEMENT_FAILURES, _errors.SettlementError),
    'Transfer': (generated.TransferFailure, _errors._TRANSFER_FAILURES, _errors.TransferError),
}
"""Each module's generated failure, its table, and the layer's error that holds it, by the module's name."""


def cases(kind: type) -> list[type]:
    """
    List the cases of a generated data enum, each a nested class named in capitals.

    Args:
        kind: The generated data enum.

    Returns:
        Its cases.

    """
    return [value for name, value in vars(kind).items() if isinstance(value, type) and name.isupper()]


def instance(case: type) -> object:
    """
    Make a case of a generated data enum, each field an example of its type: a currency's code for text.

    Args:
        case: The case.

    Returns:
        The instance.

    """
    examples = {
        'str': 'EUR',
        'int': 1,
        'Money': generated.Money(currency='EUR', minor_units=1),
        'typing.List[str]': ['EUR'],
        'IbanReason': generated.IbanReason.INVALID_FORMAT(),
    }
    parameters = list(inspect.signature(case.__init__).parameters.values())[1:]
    return case(*(examples[parameter.annotation] for parameter in parameters))


def converted(error: BaseException) -> BaseException:
    """
    Raise error inside library_errors, as a generated call would.

    Args:
        error: The generated error.

    Returns:
        What the layer raised in its place.

    """
    with pytest.raises(BaseException) as caught, library_errors():
        raise error
    return caught.value


def title(name: str) -> str:
    """
    Name a case as the layer does: ``MEMBER_IN_USE`` is ``MemberInUse``.

    Args:
        name: The generated case's name.

    Returns:
        The layer's name.

    """
    return ''.join(word.title() for word in name.split('_'))


def test_every_table_converts_every_case_the_generator_wrote() -> None:
    """A case the generator wrote and no table holds would reach an app as a KeyError; one left over would name no case at all."""
    for kind, table in [
        (generated.Split, _entries._SPLITS),
        (generated.Cost, _entries._COSTS),
        (generated.TipValue, _entries._TIP_VALUES),
        (generated.ExpenseSplit, _entries._EXPENSE_SPLITS),
        (generated.AccountDetails, _entries._ACCOUNT_DETAILS),
        (generated.Balance, _totals._BALANCES),
        (generated.IbanReason, _errors._IBAN_REASONS),
    ]:
        assert set(table) == set(cases(kind)), kind
    for kind, table, _ in FAILURES.values():
        expected = set(kind) if isinstance(kind, type) and issubclass(kind, enum.Enum) else set(cases(kind))
        assert set(table) == expected, kind
    assert set(_errors._ERRORS) == {value for value in vars(generated.EranosError).values() if isinstance(value, type)}
    assert [member.name for member in _entries.TipDistribution] == [member.name for member in generated.TipDistribution]
    assert [member.name for member in _totals.SettlementStrategy] == [member.name for member in generated.SettlementStrategy]


def test_every_failure_the_library_reports_converts_to_the_layers_case_of_its_name() -> None:
    """A failure that converted to another case, or lost its module, would send an app's handling of it to the wrong branch."""
    for module, (kind, _, error) in FAILURES.items():
        failures = list(kind) if isinstance(kind, type) and issubclass(kind, enum.Enum) else [instance(case) for case in cases(kind)]
        for failure in failures:
            raised = converted(getattr(generated.EranosError, module)('the message', failure))
            assert isinstance(raised, error), failure
            assert isinstance(raised, EranosError), failure
            assert str(raised) == 'the message'
            name = failure.name if isinstance(failure, enum.Enum) else re.sub(r'.*\.', '', type(failure).__name__)
            assert type(raised.failure).__name__ == title(name), failure
    for case in cases(generated.IbanReason):
        reason = instance(case)
        assert isinstance(reason, generated.IbanReason)
        raised = converted(generated.EranosError.Account('the message', generated.AccountFailure.INVALID_IBAN(reason)))
        assert isinstance(raised, _errors.AccountError)
        assert isinstance(raised.failure, _errors.AccountError.InvalidIban)
        assert type(raised.failure.reason).__name__ == title(re.sub(r'.*\.', '', case.__name__))


def test_the_librarys_own_errors_convert_to_the_standard_errors_they_stand_for() -> None:
    """An error a program should not have made that converted to a library error would read as a refusal of valid data."""
    assert isinstance(converted(generated.EranosError.Cancelled('cancelled')), asyncio.CancelledError)
    assert isinstance(converted(generated.EranosError.IdentifierGiven('given')), ValueError)
    assert isinstance(converted(generated.EranosError.IdentifierMissing('missing')), ValueError)
    assert isinstance(converted(generated.EranosError.Released('released')), RuntimeError)
    invalid = converted(generated.EranosError.InvalidJson(3, 'invalid', 2))
    assert isinstance(invalid, InvalidJsonError)
    assert (invalid.column, invalid.line, str(invalid)) == (3, 2, 'invalid')
