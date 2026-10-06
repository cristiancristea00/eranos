"""Check that the layer converts every case the generator wrote."""

import asyncio
import dataclasses
import enum
import inspect
import re
from collections.abc import Mapping
from typing import Any, final

import pytest

from eranos import EranosError, InvalidJsonError, _entries, _errors, _totals
from eranos._errors import library_errors
from eranos._internal import eranos as generated


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class CaseTable:
    """A data enum the generator wrote, and the layer's table that converts its cases."""

    kind: type
    """The generated data enum."""

    table: Mapping[Any, object]
    """The layer's table, keyed by the cases it converts."""


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class Conversion[Error: EranosError]:
    """A module's generated failure, the layer's table of its cases, and the layer's error that holds it."""

    error: type[Error]
    """The layer's error that holds the failure."""

    kind: type
    """The generated failure: a data enum, or an enum of cases without fields."""

    table: Mapping[Any, object]
    """The layer's table, keyed by the cases it converts."""


FAILURES = {
    'Account': Conversion(error=_errors.AccountError, kind=generated.AccountFailure, table=_errors._ACCOUNT_FAILURES),
    'Calendar': Conversion(error=_errors.CalendarError, kind=generated.CalendarFailure, table=_errors._CALENDAR_FAILURES),
    'Category': Conversion(error=_errors.CategoryError, kind=generated.CategoryFailure, table=_errors._CATEGORY_FAILURES),
    'Contact': Conversion(error=_errors.ContactError, kind=generated.ContactFailure, table=_errors._CONTACT_FAILURES),
    'Expense': Conversion(error=_errors.ExpenseError, kind=generated.ExpenseFailure, table=_errors._EXPENSE_FAILURES),
    'Group': Conversion(error=_errors.GroupError, kind=generated.GroupFailure, table=_errors._GROUP_FAILURES),
    'Id': Conversion(error=_errors.IdError, kind=generated.IdFailure, table=_errors._ID_FAILURES),
    'Language': Conversion(error=_errors.LanguageError, kind=generated.LanguageFailure, table=_errors._LANGUAGE_FAILURES),
    'Member': Conversion(error=_errors.MemberError, kind=generated.MemberFailure, table=_errors._MEMBER_FAILURES),
    'Money': Conversion(error=_errors.MoneyError, kind=generated.MoneyFailure, table=_errors._MONEY_FAILURES),
    'Settlement': Conversion(error=_errors.SettlementError, kind=generated.SettlementFailure, table=_errors._SETTLEMENT_FAILURES),
    'Transfer': Conversion(error=_errors.TransferError, kind=generated.TransferFailure, table=_errors._TRANSFER_FAILURES),
}
"""How each module's failure converts, by the module's name."""


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
    for case_table in [
        CaseTable(kind=generated.Split, table=_entries._SPLITS),
        CaseTable(kind=generated.Cost, table=_entries._COSTS),
        CaseTable(kind=generated.TipValue, table=_entries._TIP_VALUES),
        CaseTable(kind=generated.ExpenseSplit, table=_entries._EXPENSE_SPLITS),
        CaseTable(kind=generated.AccountDetails, table=_entries._ACCOUNT_DETAILS),
        CaseTable(kind=generated.Balance, table=_totals._BALANCES),
        CaseTable(kind=generated.IbanReason, table=_errors._IBAN_REASONS),
    ]:
        assert set(case_table.table) == set(cases(case_table.kind)), case_table.kind
    for conversion in FAILURES.values():
        expected = set(conversion.kind) if issubclass(conversion.kind, enum.Enum) else set(cases(conversion.kind))
        assert set(conversion.table) == expected, conversion.kind
    assert set(_errors._ERRORS) == {value for value in vars(generated.EranosError).values() if isinstance(value, type)}
    assert [member.name for member in _entries.TipDistribution] == [member.name for member in generated.TipDistribution]
    assert [member.name for member in _totals.SettlementStrategy] == [member.name for member in generated.SettlementStrategy]


def test_every_strategy_converts_into_the_generated_module_and_back() -> None:
    """A strategy that converted to another case would plan by, or name, one the app did not ask for; no group names the adaptive one."""
    for strategy in _totals.SettlementStrategy:
        assert _totals.settlement_of(generated.Settlement(payments=[], strategy=_totals.generated_strategy(strategy))).strategy is strategy


def test_every_failure_the_library_reports_converts_to_the_layers_case_of_its_name() -> None:
    """A failure that converted to another case, or lost its module, would send an app's handling of it to the wrong branch."""
    for module, conversion in FAILURES.items():
        failures = list(conversion.kind) if issubclass(conversion.kind, enum.Enum) else [instance(case) for case in cases(conversion.kind)]
        for failure in failures:
            raised = converted(getattr(generated.EranosError, module)('the message', failure))
            assert isinstance(raised, conversion.error), failure
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
    assert invalid.column == 3
    assert invalid.line == 2
    assert str(invalid) == 'invalid'
