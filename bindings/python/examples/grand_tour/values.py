"""Print section 13 of the tour: currencies, amounts, percentages, parts, rates, dates, times, identifiers, and language tags, on their own."""

import copy
import dataclasses
from typing import ClassVar, final, override

from eranos import (
    CalendarDate,
    ConvertedCost,
    CurrencyCode,
    ExchangeRate,
    ExpenseGroup,
    IdError,
    Language,
    MemberId,
    Money,
    Parts,
    Percentage,
    TimeOfDay,
)
from examples.grand_tour.tour import People, eur, ron
from examples.report import Row, message, print_lines, print_table, refusal, section


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class CurrencyRow(Row):
    """A row of the table of currencies: what the library knows of a currency."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Code', 'Name', 'Numeric code', 'Decimal places')

    code: str
    """The ISO 4217 code."""

    name: str
    """The currency's English name."""

    numeric_code: int
    """The ISO 4217 number."""

    decimal_places: int
    """The digits of the minor unit."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.code, self.name, str(self.numeric_code), str(self.decimal_places)


@final
@dataclasses.dataclass(frozen=True, slots=True, kw_only=True)
class ValueRow(Row):
    """A row of a table of values: an input, and what the library makes of it."""

    HEADERS: ClassVar[tuple[str, ...]] = ('Input', 'Result')

    input: str
    """What the library reads."""

    result: str
    """The value as the library writes it, or the library's message."""

    @override
    def cells(self) -> tuple[str, ...]:
        return self.input, self.result


def _truth(value: bool) -> str:
    """
    Write value as the report writes a truth value.

    Args:
        value: The truth value.

    Returns:
        The text, ``"true"`` or ``"false"``.

    """
    return 'true' if value else 'false'


def _currencies() -> None:
    """Print a few currencies with what the library knows of them, and the code it refuses."""
    listed = [CurrencyCode(code) for code in ('EUR', 'RON', 'JPY', 'BHD')]
    unknown = refusal(lambda: CurrencyCode('XYZ'))
    print_lines(
        f'Currencies are the {len(CurrencyCode.all())} active ISO 4217 codes with a minor unit; an unknown code is',
        f'refused ({message(unknown)}):',
    )
    print_table(
        CurrencyRow,
        [
            CurrencyRow(code=currency.code, name=currency.name, numeric_code=currency.numeric_code, decimal_places=currency.decimal_places)
            for currency in listed
        ],
        2,
    )
    assert CurrencyCode('BHD').decimal_places == 3


def _amounts() -> None:
    """Print amounts added, subtracted, parsed, made from minor units, and entered at a rate, and what the library refuses."""
    total = eur('12.34').plus(eur('0.66'))
    change = eur('5.00').minus(eur('7.50'))
    mixed = refusal(lambda: eur('1.00').plus(ron('1.00')))
    yen = Money.parse('1500', CurrencyCode('JPY'))
    dinars = Money(1_234, CurrencyCode('BHD'))
    nothing = Money(0, CurrencyCode('RON'))
    cost = ConvertedCost(eur('10.00'), ExchangeRate('4.9765'))
    doubled = copy.replace(cost, amount=eur('20.00'))
    print_lines('Amounts are exact minor units in their currency:')
    print_table(
        ValueRow,
        [
            ValueRow(input='12.34 EUR + 0.66 EUR', result=str(total)),
            ValueRow(input='5.00 EUR \N{MINUS SIGN} 7.50 EUR', result=f'{change}, negative: {_truth(change.minor_units < 0)}'),
            ValueRow(input='1.00 EUR + 1.00 RON', result=message(mixed)),
            ValueRow(input='1.005 EUR', result=message(refusal(lambda: eur('1.005')))),
            ValueRow(input='1500 JPY', result=f'{yen}, {yen.minor_units} minor units'),
            ValueRow(input='1234 minor units of BHD', result=str(dinars)),
            ValueRow(input='zero RON', result=f'{nothing}, zero: {_truth(nothing.minor_units == 0)}'),
            ValueRow(input='10.00 EUR at 4.9765', result=f'{cost.amount} at {cost.rate}, then {doubled.amount} at the same rate'),
        ],
        2,
    )
    assert total == eur('13.00')
    assert change.minor_units == -250
    assert mixed is not None
    assert str(dinars) == '1.234 BHD'


def _ratios() -> None:
    """Print percentages, parts, and rates as the library reads and writes them, and what it refuses."""
    percentage = Percentage.parse('12.5')
    parts = Parts.parse('1.5')
    rate = ExchangeRate('4.97650')
    print_lines('Percentages, parts, and rates are exact decimals:')
    print_table(
        ValueRow,
        [
            ValueRow(input='12.5%', result=f'{percentage}, {percentage.basis_points} basis points'),
            ValueRow(input='333 basis points', result=str(Percentage(333))),
            ValueRow(input='33.333%', result=message(refusal(lambda: Percentage.parse('33.333')))),
            ValueRow(input='1.5 parts', result=f'{parts}, {parts.hundredths} hundredths'),
            ValueRow(input='3 whole parts', result=str(Parts(300))),
            ValueRow(input='25 hundredths of a part', result=str(Parts(25))),
            ValueRow(input='0 parts', result=message(refusal(lambda: Parts.parse('0')))),
            ValueRow(input='a rate of 4.97650', result=str(rate)),
            ValueRow(input='a rate of 0', result=message(refusal(lambda: ExchangeRate('0')))),
        ],
        2,
    )
    assert percentage.basis_points == 1_250
    assert parts.hundredths == 150
    assert Parts.parse('1').hundredths == 100
    # A rate keeps no trailing zero.
    assert str(rate) == '4.9765'


def _dates_and_identifiers(group: ExpenseGroup, people: People) -> None:
    """
    Print dates, times, an identifier, and language tags as the library reads and writes them, and what it refuses.

    Args:
        group: The group, which checks an identifier when a call carries it.
        people: The members of the trip.

    """
    date = CalendarDate.parse('2026-10-01')
    time = TimeOfDay.parse('22:15')
    parsed = MemberId(str(people.ana))
    found = group.member(parsed)
    empty = refusal(lambda: group.member(MemberId('')))
    after_a_space = refusal(lambda: group.member(MemberId(' ana')))
    with_a_tab = refusal(lambda: group.member(MemberId('a\tb')))
    too_long = refusal(lambda: group.member(MemberId('a' * 65)))
    language = Language('pt-pt')
    print_lines('Dates, times, identifiers, and language tags read and print the ISO forms:')
    print_table(
        ValueRow,
        [
            ValueRow(input='2026-10-01', result=f'{date}: day {date.day}, month {date.month}, year {date.year}'),
            ValueRow(input='29 February 2027', result=message(refusal(lambda: CalendarDate(2027, 2, 29)))),
            ValueRow(input='22:15', result=f'{time}: hour {time.hour}, minute {time.minute}'),
            ValueRow(input='24:00', result=message(refusal(lambda: TimeOfDay(24, 0)))),
            ValueRow(
                input="Ana's identifier", result=f'{parsed}, the same after a round trip: {_truth(found is not None and found.id == people.ana)}'
            ),
            ValueRow(input='empty text', result=message(empty)),
            ValueRow(input='ana after a space', result=message(after_a_space)),
            ValueRow(input='a tab between a and b', result=message(with_a_tab)),
            ValueRow(input='65 letters a', result=message(too_long)),
            ValueRow(input='pt-pt', result=str(language)),
            ValueRow(input='pt_PT', result=message(refusal(lambda: Language('pt_PT')))),
        ],
        2,
    )
    assert date.day == 1
    assert date.month == 10
    assert date.year == 2026
    assert time.hour == 22
    assert time.minute == 15
    assert parsed == people.ana
    refused = {
        IdError.Empty(): empty,
        IdError.SurroundingWhitespace(): after_a_space,
        IdError.ControlCharacter(): with_a_tab,
        IdError.TooLong(): too_long,
    }
    for failure, error in refused.items():
        assert isinstance(error, IdError), f'the identifier is refused with {failure}'
        assert error.failure == failure


def values(group: ExpenseGroup, people: People) -> None:
    """
    Print section 13: the library's values on their own.

    Args:
        group: The group, which checks an identifier when a call carries it.
        people: The members of the trip.

    """
    section('13. Values on their own')
    _currencies()
    _amounts()
    _ratios()
    _dates_and_identifiers(group, people)
