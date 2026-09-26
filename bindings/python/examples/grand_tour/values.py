"""Print section 13 of the tour: currencies, amounts, percentages, parts, rates, dates, times, identifiers, and language tags, on their own."""

import copy

from eranos import (
    CalendarDate,
    ConvertedCost,
    Currency,
    ExchangeRate,
    Group,
    Language,
    MemberId,
    Money,
    Parts,
    Percentage,
    TimeOfDay,
)
from examples.grand_tour.tour import People, eur, ron
from examples.report import message, print_lines, print_table, refusal, section


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
    listed = [Currency(code) for code in ('EUR', 'RON', 'JPY', 'BHD')]
    unknown = refusal(lambda: Currency('XYZ'))
    print_lines(
        f'Currencies are the {len(Currency.all())} active ISO 4217 codes with a minor unit; an unknown code is',
        f'refused ({message(unknown)}):',
    )
    print_table(
        ('Code', 'Name', 'Numeric code', 'Decimal places'),
        [(currency.code, currency.name, str(currency.numeric_code), str(currency.decimal_places)) for currency in listed],
        2,
    )
    assert Currency('BHD').decimal_places == 3


def _amounts() -> None:
    """Print amounts added, subtracted, parsed, made from minor units, and entered at a rate, and what the library refuses."""
    total = eur('12.34').plus(eur('0.66'))
    change = eur('5.00').minus(eur('7.50'))
    mixed = refusal(lambda: eur('1.00').plus(ron('1.00')))
    yen = Money.parse('1500', Currency('JPY'))
    dinars = Money(Currency('BHD'), 1_234)
    nothing = Money(Currency('RON'), 0)
    cost = ConvertedCost(eur('10.00'), ExchangeRate('4.9765'))
    doubled = copy.replace(cost, amount=eur('20.00'))
    print_lines('Amounts are exact minor units in their currency:')
    print_table(
        ('Input', 'Result'),
        [
            ('12.34 EUR + 0.66 EUR', str(total)),
            ('5.00 EUR \N{MINUS SIGN} 7.50 EUR', f'{change}, negative: {_truth(change.minor_units < 0)}'),
            ('1.00 EUR + 1.00 RON', message(mixed)),
            ('1.005 EUR', message(refusal(lambda: eur('1.005')))),
            ('1500 JPY', f'{yen}, {yen.minor_units} minor units'),
            ('1234 minor units of BHD', str(dinars)),
            ('zero RON', f'{nothing}, zero: {_truth(nothing.minor_units == 0)}'),
            ('10.00 EUR at 4.9765', f'{cost.amount} at {cost.rate}, then {doubled.amount} at the same rate'),
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
        ('Input', 'Result'),
        [
            ('12.5%', f'{percentage}, {percentage.basis_points} basis points'),
            ('333 basis points', str(Percentage(333))),
            ('33.333%', message(refusal(lambda: Percentage.parse('33.333')))),
            ('1.5 parts', f'{parts}, {parts.hundredths} hundredths'),
            ('3 whole parts', str(Parts(300))),
            ('25 hundredths of a part', str(Parts(25))),
            ('0 parts', message(refusal(lambda: Parts.parse('0')))),
            ('a rate of 4.97650', str(rate)),
            ('a rate of 0', message(refusal(lambda: ExchangeRate('0')))),
        ],
        2,
    )
    assert percentage.basis_points == 1_250
    assert parts.hundredths == 150
    assert Parts.parse('1').hundredths == 100
    # A rate keeps no trailing zero.
    assert str(rate) == '4.9765'


def _dates_and_identifiers(group: Group, people: People) -> None:
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
    language = Language('pt-pt')
    print_lines('Dates, times, identifiers, and language tags read and print the ISO forms:')
    print_table(
        ('Input', 'Result'),
        [
            ('2026-10-01', f'{date}: day {date.day}, month {date.month}, year {date.year}'),
            ('29 February 2027', message(refusal(lambda: CalendarDate(29, 2, 2027)))),
            ('22:15', f'{time}: hour {time.hour}, minute {time.minute}'),
            ('24:00', message(refusal(lambda: TimeOfDay(24, 0)))),
            ("Ana's identifier", f'{parsed}, the same after a round trip: {_truth(found is not None and found.id == people.ana)}'),
            ('not-a-uuid', message(refusal(lambda: group.member(MemberId('not-a-uuid'))))),
            ('pt-pt', str(language)),
            ('pt_PT', message(refusal(lambda: Language('pt_PT')))),
        ],
        2,
    )
    assert (date.day, date.month, date.year) == (1, 10, 2026)
    assert (time.hour, time.minute) == (22, 15)
    assert parsed == people.ana


def values(group: Group, people: People) -> None:
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
