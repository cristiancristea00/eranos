"""Check that the layer converts every case the generator wrote."""

import asyncio
import dataclasses
import enum
import inspect
import re
from collections.abc import Mapping
from typing import Any, final

import pytest

from conftest import Flat
from eranos import (
    NO_TIP,
    UNCATEGORISED,
    CurrencyCode,
    EranosError,
    ExpenseError,
    ExpenseGroupError,
    Iban,
    IbanPaymentAccountDetails,
    InvalidJsonError,
    Member,
    MismatchedField,
    MismatchedRecordError,
    Money,
    MoneyError,
    PaymentAccount,
    PhoneNumber,
    _entries,
    _errors,
    _totals,
)
from eranos._contact import iban_of, phone_number_of
from eranos._crossing import variant
from eranos._currency_code import currency_of, generated_currency
from eranos._errors import library_errors
from eranos._internal import eranos as generated
from eranos._money import money_of


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

    kind: type
    """The generated failure: a data enum, or an enum of cases without fields."""

    table: Mapping[Any, object]
    """The layer's table, keyed by the cases it converts."""

    error: type[Error]
    """The layer's error that holds the failure."""


FAILURES = {
    'Id': Conversion(kind=generated.IdFailure, table=_errors._ID_FAILURES, error=_errors.IdError),
    'PaymentAccount': Conversion(kind=generated.PaymentAccountFailure, table=_errors._PAYMENT_ACCOUNT_FAILURES, error=_errors.PaymentAccountError),
    'Calendar': Conversion(kind=generated.CalendarFailure, table=_errors._CALENDAR_FAILURES, error=_errors.CalendarError),
    'Category': Conversion(kind=generated.CategoryFailure, table=_errors._CATEGORY_FAILURES, error=_errors.CategoryError),
    'Contact': Conversion(kind=generated.ContactFailure, table=_errors._CONTACT_FAILURES, error=_errors.ContactError),
    'Money': Conversion(kind=generated.MoneyFailure, table=_errors._MONEY_FAILURES, error=_errors.MoneyError),
    'Member': Conversion(kind=generated.MemberFailure, table=_errors._MEMBER_FAILURES, error=_errors.MemberError),
    'Expense': Conversion(kind=generated.ExpenseFailure, table=_errors._EXPENSE_FAILURES, error=_errors.ExpenseError),
    'Settlement': Conversion(kind=generated.SettlementFailure, table=_errors._SETTLEMENT_FAILURES, error=_errors.SettlementError),
    'Transfer': Conversion(kind=generated.TransferFailure, table=_errors._TRANSFER_FAILURES, error=_errors.TransferError),
    'ExpenseGroup': Conversion(kind=generated.ExpenseGroupFailure, table=_errors._EXPENSE_GROUP_FAILURES, error=_errors.ExpenseGroupError),
    'Language': Conversion(kind=generated.LanguageFailure, table=_errors._LANGUAGE_FAILURES, error=_errors.LanguageError),
}
"""How each module's failure converts, by the module's name."""

UNLISTED = generated.CurrencyDetails(code='XTS', name='Testing', numeric_code=963, decimal_places=4)
"""The details of a currency the library does not list: a lookup of its code is refused."""

REGROUPED = generated.PhoneNumber(e164='+40712345678', international='+40-712-345-678')
"""A phone number with an international form the library does not write for it: it groups the number ``+40 712 345 678``."""

IBAN = 'RO49AAAA1B31007593840000'
"""A Romanian IBAN, whose country code is ``RO``."""


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
    Make a case of a generated data enum, each field an example of its type: a currency's code for text, and an unlisted currency's details.

    Args:
        case: The case.

    Returns:
        The instance.

    """
    examples = {
        'str': 'EUR',
        'int': 1,
        'CurrencyDetails': UNLISTED,
        'Money': generated.Money(minor_units=1, currency=UNLISTED),
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
        CaseTable(kind=generated.DiscountValue, table=_entries._DISCOUNT_VALUES),
        CaseTable(kind=generated.TipValue, table=_entries._TIP_VALUES),
        CaseTable(kind=generated.ExpenseSplit, table=_entries._EXPENSE_SPLITS),
        CaseTable(kind=generated.PaymentAccountDetails, table=_entries._PAYMENT_ACCOUNT_DETAILS),
        CaseTable(kind=generated.Balance, table=_totals._BALANCES),
        CaseTable(kind=generated.IbanReason, table=_errors._IBAN_REASONS),
    ]:
        assert set(case_table.table) == set(cases(case_table.kind)), case_table.kind
    assert set(_errors._MISMATCHED_FIELDS) == set(generated.MismatchedField)
    for conversion in FAILURES.values():
        expected = set(conversion.kind) if issubclass(conversion.kind, enum.Enum) else set(cases(conversion.kind))
        assert set(conversion.table) == expected, conversion.kind
    assert set(_errors._ERRORS) == {value for value in vars(generated.EranosError).values() if isinstance(value, type)}
    assert [member.name for member in _entries.DiscountScope] == [member.name for member in generated.DiscountScope]
    assert [member.name for member in _errors.MismatchedField] == [member.name for member in generated.MismatchedField]
    assert [member.name for member in _entries.TipDistribution] == [member.name for member in generated.TipDistribution]
    assert [member.name for member in _totals.SettlementStrategy] == [member.name for member in generated.SettlementStrategy]


def test_the_built_in_category_and_the_no_tip_rule_are_the_ones_the_core_gives() -> None:
    """A constant that differed from the core's would put an expense the app left alone in another category, or give it a tip."""
    assert generated.category_id_uncategorised() == UNCATEGORISED
    assert _entries.generated_tip(NO_TIP) == generated.tip_rule_none()


def test_every_strategy_converts_into_the_generated_module_and_back() -> None:
    """A strategy that converted to another case would plan by, or name, one the app did not ask for; no group names the adaptive one."""
    for strategy in _totals.SettlementStrategy:
        assert _totals.settlement_of(generated.SettlementPlan(payments=[], strategy=_totals.generated_strategy(strategy))).strategy is strategy


def test_every_failure_the_library_reports_converts_to_the_layers_case_of_its_name() -> None:
    """A failure that converted to another case, or lost its module, would send an app's handling of it to the wrong branch."""
    for module, conversion in FAILURES.items():
        failures = list(conversion.kind) if issubclass(conversion.kind, enum.Enum) else [instance(case) for case in cases(conversion.kind)]
        for failure in failures:
            raised = converted(getattr(generated.EranosError, module)(failure, 'the message'))
            assert isinstance(raised, conversion.error), failure
            assert isinstance(raised, EranosError), failure
            assert str(raised) == 'the message'
            name = failure.name if isinstance(failure, enum.Enum) else re.sub(r'.*\.', '', type(failure).__name__)
            assert type(raised.failure).__name__ == title(name), failure
    for case in cases(generated.IbanReason):
        reason = instance(case)
        assert isinstance(reason, generated.IbanReason)
        raised = converted(generated.EranosError.PaymentAccount(generated.PaymentAccountFailure.INVALID_IBAN(reason), 'the message'))
        assert isinstance(raised, _errors.PaymentAccountError)
        assert isinstance(raised.failure, _errors.PaymentAccountError.InvalidIban)
        assert type(raised.failure.reason).__name__ == title(re.sub(r'.*\.', '', case.__name__))


def test_the_librarys_own_errors_convert_to_the_standard_errors_they_stand_for() -> None:
    """
    An error a program should not have made that converted to a library error would read as a refusal of valid data.

    A mismatched record that named another field would send the app to check the wrong one.
    """
    invalid = converted(generated.EranosError.InvalidJson(2, 3, 'invalid'))
    assert isinstance(invalid, InvalidJsonError)
    assert invalid.line == 2
    assert invalid.column == 3
    assert str(invalid) == 'invalid'
    for field in generated.MismatchedField:
        mismatched = converted(generated.EranosError.MismatchedRecord(field, 'mismatched'))
        assert isinstance(mismatched, MismatchedRecordError)
        assert mismatched.field is MismatchedField[field.name]
        assert str(mismatched) == 'mismatched'
    assert isinstance(converted(generated.EranosError.Cancelled('cancelled')), asyncio.CancelledError)
    assert isinstance(converted(generated.EranosError.Released('released')), RuntimeError)


def test_an_amount_and_a_failure_hold_the_details_the_library_gives_without_a_lookup(monkeypatch: pytest.MonkeyPatch) -> None:
    """
    A currency looked up by its code for each amount and failure the library answers with would cost a call of the library each.

    The details are of a currency the library does not list, which no lookup returns, and a lookup fails the test.
    """

    def looked_up(code: str) -> generated.CurrencyDetails:
        raise AssertionError(code)

    monkeypatch.setattr(generated, 'currency_details', looked_up)
    amount = money_of(generated.Money(minor_units=-12345, currency=UNLISTED))
    assert generated_currency(amount.currency) == UNLISTED
    assert str(amount) == '-1.2345 XTS'
    decimals = converted(generated.EranosError.Money(generated.MoneyFailure.TOO_MANY_DECIMALS(UNLISTED, 4), 'the message'))
    assert isinstance(decimals, MoneyError)
    assert isinstance(decimals.failure, MoneyError.TooManyDecimals)
    assert generated_currency(decimals.failure.currency) == UNLISTED
    amounts = converted(generated.EranosError.Money(generated.MoneyFailure.CURRENCY_MISMATCH(UNLISTED, UNLISTED), 'the message'))
    assert isinstance(amounts, MoneyError)
    assert isinstance(amounts.failure, MoneyError.CurrencyMismatch)
    assert generated_currency(amounts.failure.left) == UNLISTED
    assert generated_currency(amounts.failure.right) == UNLISTED
    parts = converted(generated.EranosError.Expense(generated.ExpenseFailure.CURRENCY_MISMATCH(UNLISTED, UNLISTED), 'the message'))
    assert isinstance(parts, ExpenseError)
    assert isinstance(parts.failure, ExpenseError.CurrencyMismatch)
    assert generated_currency(parts.failure.expected) == UNLISTED
    assert generated_currency(parts.failure.found) == UNLISTED
    missing = converted(generated.EranosError.ExpenseGroup(generated.ExpenseGroupFailure.MISSING_EXCHANGE_RATE(UNLISTED), 'the message'))
    assert isinstance(missing, ExpenseGroupError)
    assert isinstance(missing.failure, ExpenseGroupError.MissingExchangeRate)
    assert generated_currency(missing.failure.currency) == UNLISTED
    twice = converted(generated.EranosError.ExpenseGroup(generated.ExpenseGroupFailure.DUPLICATE_EXCHANGE_RATE(UNLISTED), 'the message'))
    assert isinstance(twice, ExpenseGroupError)
    assert isinstance(twice.failure, ExpenseGroupError.DuplicateExchangeRate)
    assert generated_currency(twice.failure.currency) == UNLISTED


def test_an_amount_whose_currency_details_are_not_the_librarys_is_refused() -> None:
    """
    An amount whose currency held other details than the library's for its code, taken, would show other decimal places than the group holds.

    An app makes a currency only from its code, which the library looks up, so only the package's private conversion makes one.
    """
    euro = CurrencyCode('EUR')
    for details in (
        generated.CurrencyDetails(code='eur', name='Euro', numeric_code=978, decimal_places=2),
        generated.CurrencyDetails(code='EUR', name='Euros', numeric_code=978, decimal_places=2),
        generated.CurrencyDetails(code='EUR', name='Euro', numeric_code=840, decimal_places=2),
        generated.CurrencyDetails(code='EUR', name='Euro', numeric_code=978, decimal_places=3),
    ):
        with pytest.raises(MismatchedRecordError) as mismatched:
            Money(1, currency_of(details)).plus(Money(1, euro))
        assert mismatched.value.field is MismatchedField.CURRENCY_DETAILS, details
        assert str(mismatched.value) == "the currency's details are not the library's for its code"
    with pytest.raises(MoneyError) as refused:
        Money(1, currency_of(UNLISTED)).plus(Money(1, euro))
    assert refused.value.failure == MoneyError.UnknownCurrency()


def test_a_phone_number_and_an_iban_hold_the_text_the_library_gives_without_a_call(monkeypatch: pytest.MonkeyPatch) -> None:
    """
    A phone number's international form or an IBAN's country code worked out again for each member the library answers with would cost a call each.

    The forms are not the ones the library writes, which no call returns, and a parse fails the test; the member crosses back as it came.
    """

    def parsed(text: str) -> object:
        raise AssertionError(text)

    monkeypatch.setattr(generated, 'phone_number_parse', parsed)
    monkeypatch.setattr(generated, 'iban_parse', parsed)
    german = variant(generated.PaymentAccountDetails, generated.PaymentAccountDetails.IBAN(IBAN, 'DE'))
    record = generated.Member(
        id='carla',
        first_name='Carla',
        last_name=None,
        email=None,
        phone=REGROUPED,
        accounts=[generated.PaymentAccount(name='Bank', details=german)],
    )
    carla = _entries.member_of(record)
    assert carla.phone is not None
    assert carla.phone.e164 == '+40712345678'
    assert carla.phone.international == '+40-712-345-678'
    assert str(carla.phone) == '+40-712-345-678'
    details = carla.accounts[0].details
    assert isinstance(details, IbanPaymentAccountDetails)
    assert str(details.iban) == IBAN
    assert details.iban.country_code == 'DE'
    assert _entries.generated_member(carla) == record


def test_a_phone_number_or_an_iban_whose_derived_text_is_not_the_librarys_is_refused(flat: Flat) -> None:
    """
    A member whose phone number or IBAN carried other text than the library derives from it, taken, would show a form the group does not hold.

    An app makes a phone number and an IBAN only from their text, which the library parses, so only the package's private conversions make one.
    """
    with pytest.raises(MismatchedRecordError) as phone:
        flat.group.add_member(Member.create(first_name='Carla', phone=phone_number_of(REGROUPED)))
    assert phone.value.field is MismatchedField.PHONE_INTERNATIONAL
    assert str(phone.value) == "the phone number's international form is not the one the library writes for it"
    german = PaymentAccount('Bank', IbanPaymentAccountDetails(iban_of(IBAN, 'DE')))
    with pytest.raises(MismatchedRecordError) as iban:
        flat.group.add_member(Member.create(first_name='Carla', accounts=(german,)))
    assert iban.value.field is MismatchedField.IBAN_COUNTRY_CODE
    assert str(iban.value) == "the IBAN's country code is not the one the library derives from it"
    assert len(flat.group.members()) == 2


def test_a_phone_number_or_an_iban_equals_only_one_with_the_same_derived_text() -> None:
    """Two numbers or IBANs equal despite other derived text would let an app's set or cache keep a form the library does not write."""
    phone = PhoneNumber('+40 712 345 678')
    same = phone_number_of(generated.PhoneNumber(e164='+40712345678', international='+40 712 345 678'))
    assert phone == same
    assert hash(phone) == hash(same)
    assert phone != phone_number_of(REGROUPED)
    assert len({phone, phone_number_of(REGROUPED)}) == 2
    assert phone != '+40712345678'
    assert repr(phone) == "PhoneNumber('+40712345678')"
    iban = Iban(IBAN)
    assert iban == iban_of(IBAN, 'RO')
    assert hash(iban) == hash(iban_of(IBAN, 'RO'))
    assert iban != iban_of(IBAN, 'DE')
    assert len({iban, iban_of(IBAN, 'DE')}) == 2
    assert iban != IBAN
    assert repr(iban) == f'Iban({IBAN!r})'
