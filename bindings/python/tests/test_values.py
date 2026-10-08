"""Check the values the library checks and writes: identifiers, currencies, amounts, ratios, quantities, dates, times, contacts, and accounts."""

import functools
import math
import uuid
from collections.abc import Callable

import pytest

from eranos import (
    UNCATEGORISED,
    PaymentAccountError,
    CalendarDate,
    CalendarError,
    CurrencyCode,
    Email,
    ExchangeRate,
    GetsBackBalance,
    Iban,
    IdError,
    Language,
    LanguageError,
    Money,
    MoneyError,
    OwesBalance,
    Parts,
    Percentage,
    PhoneNumber,
    Quantity,
    SettledBalance,
    TimeOfDay,
    Username,
    ContactError,
    generate_category_id,
    generate_expense_group_id,
    generate_expense_id,
    generate_member_id,
    generate_transfer_id,
    parse_category_id,
    parse_expense_group_id,
    parse_expense_id,
    parse_member_id,
    parse_transfer_id,
)


def test_an_identifier_parses_as_the_library_reads_it() -> None:
    """An app would store an identifier the library refuses, see its text changed, or not learn which rule the text broke."""
    parsers: tuple[Callable[[str], str], ...] = (parse_expense_group_id, parse_member_id, parse_category_id, parse_expense_id, parse_transfer_id)
    # The limit counts bytes of UTF-8: 32 letters é fill it, and 33 pass it.
    refused = {
        '': IdError.Empty(),
        'x' * 65: IdError.TooLong(),
        'é' * 33: IdError.TooLong(),
        'a\tb': IdError.ControlCharacter(),
        ' x': IdError.SurroundingWhitespace(),
    }
    for parse in parsers:
        # Kept as it is, case and inner spaces included.
        for text in ('-food', '!a', 'zz', 'Lisboa · 2026', 'é' * 32):
            assert parse(text) == text, text
        for text, failure in refused.items():
            with pytest.raises(IdError) as refusal:
                parse(text)
            assert refusal.value.failure == failure, text


def test_a_generated_identifier_is_new_and_parses_as_the_library_reads_it() -> None:
    """
    A repeated generated identifier would collide with another entry, and one out of order would list entries out of creation order.

    Every kind draws from the library's one generator a UUID v7 in lowercase hyphenated form, which the parse of every kind accepts and which
    orders after every one generated before it.
    """
    generators: tuple[Callable[[], str], ...] = (
        generate_expense_group_id,
        generate_member_id,
        generate_category_id,
        generate_expense_id,
        generate_transfer_id,
    )
    parsers: tuple[Callable[[str], str], ...] = (parse_expense_group_id, parse_member_id, parse_category_id, parse_expense_id, parse_transfer_id)
    identifiers = [generate() for generate in generators + generators]
    for parse, identifier in zip(parsers + parsers, identifiers, strict=True):
        assert parse(identifier) == identifier
        assert str(uuid.UUID(identifier)) == identifier
        assert uuid.UUID(identifier).version == 7
    assert identifiers == sorted(identifiers)
    assert len(set(identifiers)) == len(identifiers)


def test_the_built_in_category_is_named_by_the_nil_uuid() -> None:
    """An app that left an expense in the built-in category would name another category than the one every group has, or one the rules refuse."""
    assert UNCATEGORISED == '00000000-0000-0000-0000-000000000000'
    assert parse_category_id(UNCATEGORISED) == UNCATEGORISED


def test_a_currency_holds_what_the_library_knows_of_it() -> None:
    """A currency whose details came out wrong would format every amount in it wrongly."""
    euro = CurrencyCode('EUR')
    assert euro.code == 'EUR'
    assert euro.name == 'Euro'
    assert euro.numeric_code == 978
    assert euro.decimal_places == 2
    assert CurrencyCode('JPY').decimal_places == 0
    assert euro == CurrencyCode('EUR')
    assert euro != CurrencyCode('JPY')
    assert euro != 'EUR'
    assert hash(euro) == hash(CurrencyCode('EUR'))
    assert euro < CurrencyCode('JPY')
    assert repr(euro) == "CurrencyCode('EUR')"
    assert str(euro) == 'EUR'


def test_every_currency_the_library_lists_is_listed_by_code() -> None:
    """
    A list that missed a currency or came out of order would hide one from the currency pickers of an app.

    A listed currency that held other details than its code's would format its amounts otherwise than the library does.
    """
    listed = CurrencyCode.all()
    assert list(listed) == sorted(listed)
    assert CurrencyCode('EUR') in listed
    for currency in listed:
        looked_up = CurrencyCode(currency.code)
        assert currency.name == looked_up.name, currency
        assert currency.numeric_code == looked_up.numeric_code, currency
        assert currency.decimal_places == looked_up.decimal_places, currency


def test_a_code_the_library_does_not_list_is_refused() -> None:
    """A code the library does not list, taken, would make an amount no call can format."""
    with pytest.raises(MoneyError) as refused:
        CurrencyCode('XYZ')
    assert refused.value.failure == MoneyError.UnknownCurrency()
    assert str(refused.value) == 'unknown currency code'


def test_text_that_is_not_utf8_is_refused_before_the_library_sees_it() -> None:
    """Text with a lone surrogate, passed on, would reach the library as text it cannot hold."""
    with pytest.raises(UnicodeEncodeError):
        CurrencyCode('\ud800')


def test_an_amount_parses_and_formats_as_the_library_writes_it() -> None:
    """An amount that parsed or formatted otherwise than the library does would show one number in an app and store another."""
    euro = CurrencyCode('EUR')
    price = Money.parse('12.5', euro)
    assert price == Money(1250, euro)
    assert str(price) == '12.50 EUR'
    with pytest.raises(MoneyError) as word:
        Money.parse('twelve', euro)
    assert word.value.failure == MoneyError.InvalidAmount()
    with pytest.raises(MoneyError) as precise:
        Money.parse('1.234', euro)
    assert precise.value.failure == MoneyError.TooManyDecimals(currency=euro, allowed=2)


def test_an_amount_and_a_balance_format_with_their_currencys_decimal_places() -> None:
    """An amount written with other decimal places than its currency has would show a price ten or a hundred times off."""
    assert str(Money(-12345, CurrencyCode('JPY'))) == '-12345 JPY'
    assert str(Money(-12345, CurrencyCode('EUR'))) == '-123.45 EUR'
    assert str(Money(-12345, CurrencyCode('BHD'))) == '-12.345 BHD'
    assert str(Money(-12345, CurrencyCode('CLF'))) == '-1.2345 CLF'
    assert str(Money(5, CurrencyCode('CLF'))) == '0.0005 CLF'
    assert str(OwesBalance(Money(1500, CurrencyCode('JPY')))) == 'owes 1500 JPY'
    assert str(SettledBalance()) == 'settled'
    assert str(GetsBackBalance(Money(1234, CurrencyCode('BHD')))) == 'gets back 1.234 BHD'


def test_amounts_in_one_currency_add_and_subtract() -> None:
    """Arithmetic that mixed currencies or wrapped around would put a wrong total in a report."""
    euro = CurrencyCode('EUR')
    lei = CurrencyCode('RON')
    assert Money(1000, euro).plus(Money(250, euro)) == Money(1250, euro)
    assert Money(1000, euro).minus(Money(250, euro)) == Money(750, euro)
    with pytest.raises(MoneyError) as mixed:
        Money(1, euro).plus(Money(1, lei))
    assert mixed.value.failure == MoneyError.CurrencyMismatch(left=euro, right=lei)
    with pytest.raises(MoneyError) as largest:
        Money(2**63 - 1, euro).plus(Money(1, euro))
    assert largest.value.failure == MoneyError.Overflow()
    with pytest.raises(MoneyError) as smallest:
        Money(-(2**63), euro).minus(Money(1, euro))
    assert smallest.value.failure == MoneyError.Overflow()


def test_a_rate_parses_and_converts_as_the_library_does() -> None:
    """A rate that parsed or converted otherwise than the library does would change the share of every converted expense."""
    euro = CurrencyCode('EUR')
    lei = CurrencyCode('RON')
    rate = ExchangeRate('4.970')
    assert str(rate) == '4.97'
    assert rate == ExchangeRate('4.97')
    assert rate != ExchangeRate('5')
    assert rate != Percentage(497)
    assert hash(rate) == hash(ExchangeRate('4.97'))
    assert repr(rate) == "ExchangeRate('4.97')"
    assert ExchangeRate('0.25').convert(Money(2, lei), euro) == Money(1, euro)
    with pytest.raises(MoneyError) as zero:
        ExchangeRate('0')
    assert zero.value.failure == MoneyError.InvalidExchangeRate()
    with pytest.raises(MoneyError) as overflow:
        ExchangeRate('2').convert(Money(2**63 - 1, lei), euro)
    assert overflow.value.failure == MoneyError.Overflow()


def test_percentages_and_parts_parse_and_format_as_the_library_does() -> None:
    """A percentage or a number of parts read otherwise than the library reads it would divide a split otherwise than it shows."""
    percentage = Percentage.parse('12.5 %')
    assert percentage == Percentage(1250)
    assert str(percentage) == '12.5%'
    with pytest.raises(MoneyError) as word:
        Percentage.parse('half')
    assert word.value.failure == MoneyError.InvalidPercentage()
    with pytest.raises(MoneyError) as negative:
        Percentage(-1)
    assert negative.value.failure == MoneyError.InvalidPercentage()
    parts = Parts.parse('0.5')
    assert parts == Parts(50)
    assert str(parts) == '0.5'
    with pytest.raises(MoneyError) as parts_word:
        Parts.parse('half')
    assert parts_word.value.failure == MoneyError.InvalidParts()
    with pytest.raises(MoneyError) as none:
        Parts(0)
    assert none.value.failure == MoneyError.InvalidParts()


def money_failure(make: Callable[[], object]) -> object:
    """
    Make a value that the library should refuse, and read what its money module reports.

    Args:
        make: What makes the value.

    Returns:
        The failure.

    """
    with pytest.raises(MoneyError) as refused:
        make()
    return refused.value.failure


def calendar_failure(make: Callable[[], object]) -> object:
    """
    Make a value that the library should refuse, and read what its calendar module reports.

    Args:
        make: What makes the value.

    Returns:
        The failure.

    """
    with pytest.raises(CalendarError) as refused:
        make()
    return refused.value.failure


def test_an_amount_converts_from_a_float_as_the_library_converts_it() -> None:
    """An app that holds amounts as floats would get other cents than the library rounds to, or an amount from NaN, an infinity, or past the range."""
    euro = CurrencyCode('EUR')
    lei = CurrencyCode('RON')
    yen = CurrencyCode('JPY')
    # The float nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
    price = Money.from_float(1.005, euro)
    assert price == Money(101, euro)
    assert str(price) == '1.01 EUR'
    assert Money.from_float(-2.675, lei) == Money(-268, lei)
    assert Money.from_float(1234.5, yen) == Money(1235, yen)
    # A float parameter also takes an int, as PEP 484 allows.
    assert Money.from_float(12, euro) == Money(1200, euro)
    assert money_failure(lambda: Money.from_float(math.nan, euro)) == MoneyError.NotFinite()
    assert money_failure(lambda: Money.from_float(math.inf, euro)) == MoneyError.NotFinite()
    assert money_failure(lambda: Money.from_float(-math.inf, euro)) == MoneyError.NotFinite()
    assert money_failure(lambda: Money.from_float(1e21, euro)) == MoneyError.Overflow()


def test_rates_percentages_and_parts_convert_from_floats_as_the_library_converts_them() -> None:
    """A rate, a percentage, or parts held as a float would get other digits than the library's, wrap past the core's integers, or come from NaN."""
    assert ExchangeRate.from_float(25.5) == ExchangeRate('25.5')
    assert str(ExchangeRate.from_float(0.1 + 0.2)) == '0.30000000000000004'
    assert money_failure(lambda: ExchangeRate.from_float(0.0)) == MoneyError.InvalidExchangeRate()
    assert Percentage.from_float(0.1 + 0.2) == Percentage(30)
    assert Percentage.from_float(12.345) == Percentage(1235)
    assert money_failure(lambda: Percentage.from_float(-1.0)) == MoneyError.InvalidPercentage()
    # Above 21 474 836.47 %, the basis points no longer fit in 32 bits.
    assert money_failure(lambda: Percentage.from_float(30_000_000.0)) == MoneyError.InvalidPercentage()
    assert Parts.from_float(1.005) == Parts(101)
    assert money_failure(lambda: Parts.from_float(0.004)) == MoneyError.InvalidParts()
    assert money_failure(lambda: Parts.from_float(30_000_000.0)) == MoneyError.InvalidParts()
    converters: tuple[Callable[[float], object], ...] = (ExchangeRate.from_float, Percentage.from_float, Parts.from_float)
    for convert in converters:
        for value in (math.nan, math.inf, -math.inf):
            assert money_failure(functools.partial(convert, value)) == MoneyError.NotFinite(), value


def test_percentages_parts_dates_and_times_print_as_the_library_writes_them() -> None:
    """A percentage, parts, a date, or a time printed otherwise than the library writes it would show an app one value and the group another."""
    percentages = {0: '0%', 1: '0.01%', 10: '0.1%', 105: '1.05%', 1250: '12.5%', 3333: '33.33%', 10000: '100%', 2**31 - 1: '21474836.47%'}
    for basis_points, text in percentages.items():
        assert str(Percentage(basis_points)) == text, basis_points
    parts = {1: '0.01', 10: '0.1', 50: '0.5', 100: '1', 150: '1.5', 225: '2.25', 2**31 - 1: '21474836.47'}
    for hundredths, text in parts.items():
        assert str(Parts(hundredths)) == text, hundredths
    assert str(CalendarDate(1, 1, 1)) == '0001-01-01'
    assert str(CalendarDate(2028, 2, 29)) == '2028-02-29'
    assert str(CalendarDate(9999, 12, 31)) == '9999-12-31'
    assert str(TimeOfDay(0, 0)) == '00:00'
    assert str(TimeOfDay(9, 5)) == '09:05'
    assert str(TimeOfDay(23, 59)) == '23:59'


def test_percentages_parts_dates_and_times_are_refused_as_the_library_refuses_them() -> None:
    """A value the layer took and the library refuses would fail only when an entry carries it; one it refused would keep an app from entering it."""
    assert Percentage(0).basis_points == 0
    assert Percentage(2**31 - 1).basis_points == 2**31 - 1
    assert money_failure(lambda: Percentage(-1)) == MoneyError.InvalidPercentage()
    assert money_failure(lambda: Percentage(-(2**31))) == MoneyError.InvalidPercentage()
    assert Parts(1).hundredths == 1
    assert Parts(2**31 - 1).hundredths == 2**31 - 1
    assert money_failure(lambda: Parts(0)) == MoneyError.InvalidParts()
    assert money_failure(lambda: Parts(-50)) == MoneyError.InvalidParts()
    assert money_failure(lambda: Parts(-(2**31))) == MoneyError.InvalidParts()
    # A day the month lacks, a year outside 1 to 9999, a month or a day out of range, and a month too large for the library's field.
    assert calendar_failure(lambda: CalendarDate(2026, 2, 29)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: CalendarDate(2026, 4, 31)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: CalendarDate(0, 1, 1)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: CalendarDate(-1, 1, 1)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: CalendarDate(10000, 1, 1)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: CalendarDate(2026, 13, 1)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: CalendarDate(2026, 256, 1)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: CalendarDate(2026, 1, 0)) == CalendarError.InvalidDate()
    assert calendar_failure(lambda: TimeOfDay(24, 0)) == CalendarError.InvalidTime()
    assert calendar_failure(lambda: TimeOfDay(0, 60)) == CalendarError.InvalidTime()
    assert calendar_failure(lambda: TimeOfDay(-1, 0)) == CalendarError.InvalidTime()
    assert calendar_failure(lambda: TimeOfDay(0, 256)) == CalendarError.InvalidTime()


def test_a_quantity_parses_and_prints_as_the_library_reads_and_writes_it() -> None:
    """A quantity read or printed otherwise than the library reads and writes it would multiply a price by another number than a receipt shows."""
    weight = Quantity.parse('0.452')
    assert weight == Quantity(4520)
    assert weight.ten_thousandths == 4520
    assert str(weight) == '0.452'
    assert Quantity.parse(' 1.5 ') == Quantity(15_000)
    assert Quantity.parse('0.0001') == Quantity(1)
    assert Quantity.whole(3) == Quantity(30_000)
    assert str(Quantity.whole(3)) == '3'
    assert Quantity(10_000) == Quantity.ONE
    assert Quantity.whole(1) == Quantity.ONE
    quantities = {1: '0.0001', 10: '0.001', 4520: '0.452', 10_000: '1', 15_000: '1.5', 30_000: '3', 2**63 - 1: '922337203685477.5807'}
    for ten_thousandths, text in quantities.items():
        assert str(Quantity(ten_thousandths)) == text, ten_thousandths
        assert Quantity.parse(text) == Quantity(ten_thousandths), text


def test_a_quantity_is_refused_as_the_library_refuses_it() -> None:
    """A quantity the layer took and the library refuses would fail only when an item holds it; one it refused would keep an app from entering it."""
    assert Quantity(1).ten_thousandths == 1
    assert Quantity(2**63 - 1).ten_thousandths == 2**63 - 1
    assert money_failure(lambda: Quantity(0)) == MoneyError.InvalidQuantity()
    assert money_failure(lambda: Quantity(-1)) == MoneyError.InvalidQuantity()
    assert money_failure(lambda: Quantity(-(2**63))) == MoneyError.InvalidQuantity()
    assert money_failure(lambda: Quantity.whole(0)) == MoneyError.InvalidQuantity()
    assert money_failure(lambda: Quantity.whole(-1)) == MoneyError.InvalidQuantity()
    # Past 922 337 203 685 477 units, the ten-thousandths no longer fit in 64 bits.
    assert Quantity.whole(922_337_203_685_477) == Quantity(9_223_372_036_854_770_000)
    assert money_failure(lambda: Quantity.whole(922_337_203_685_478)) == MoneyError.InvalidQuantity()
    for text in ('0', '0.0000', '0.00005', '-1', '+1', '1e3', 'half', '', '922337203685477.5808'):
        assert money_failure(functools.partial(Quantity.parse, text)) == MoneyError.InvalidQuantity(), text


def test_a_quantity_converts_from_a_float_as_the_library_converts_it() -> None:
    """A quantity held as a float would round otherwise than the library rounds it, or come from NaN, an infinity, or a number that rounds to zero."""
    assert Quantity.from_float(0.45205) == Quantity(4521)
    assert str(Quantity.from_float(1.5)) == '1.5'
    # A float parameter also takes an int, as PEP 484 allows.
    assert Quantity.from_float(2) == Quantity.whole(2)
    assert money_failure(lambda: Quantity.from_float(0.00004)) == MoneyError.InvalidQuantity()
    assert money_failure(lambda: Quantity.from_float(-1.0)) == MoneyError.InvalidQuantity()
    assert money_failure(lambda: Quantity.from_float(1e15)) == MoneyError.InvalidQuantity()
    for value in (math.nan, math.inf, -math.inf):
        assert money_failure(functools.partial(Quantity.from_float, value)) == MoneyError.NotFinite(), value


def test_dates_and_times_are_checked_when_made() -> None:
    """A date or a time the calendar does not have, taken, would date an expense on a day that never was."""
    date = CalendarDate(2026, 9, 23)
    assert date.year == 2026
    assert date.month == 9
    assert date.day == 23
    assert str(date) == '2026-09-23'
    assert CalendarDate.parse('2026-09-23') == date
    assert CalendarDate(2026, 8, 24) < date < CalendarDate(2027, 1, 1)
    with pytest.raises(CalendarError) as february:
        CalendarDate(2026, 2, 30)
    assert february.value.failure == CalendarError.InvalidDate()
    with pytest.raises(CalendarError) as parsed:
        CalendarDate.parse('2026-02-30')
    assert parsed.value.failure == CalendarError.InvalidDate()
    time = TimeOfDay(18, 30)
    assert time.hour == 18
    assert time.minute == 30
    assert str(time) == '18:30'
    assert TimeOfDay.parse('18:30') == time
    assert TimeOfDay(9, 45) < time < TimeOfDay(18, 31)
    with pytest.raises(CalendarError) as hour:
        TimeOfDay(25, 0)
    assert hour.value.failure == CalendarError.InvalidTime()
    with pytest.raises(CalendarError) as parsed_time:
        TimeOfDay.parse('25:00')
    assert parsed_time.value.failure == CalendarError.InvalidTime()


def test_contacts_and_accounts_are_checked_and_made_canonical() -> None:
    """Contact details and accounts kept as typed would differ from the same details typed otherwise, and a wrong one would pass."""
    assert str(Email(' ana@example.com ')) == 'ana@example.com'
    assert Email('ana@example.com') != Email('bob@example.com')
    with pytest.raises(ContactError) as email:
        Email('ana')
    assert email.value.failure == ContactError.InvalidEmail()
    phone = PhoneNumber('+40 712 345 678')
    assert str(phone) == '+40712345678'
    assert phone.international == '+40 712 345 678'
    with pytest.raises(ContactError) as local:
        PhoneNumber('0712345678')
    assert local.value.failure == ContactError.InvalidPhoneNumber()
    iban = Iban('ro49 aaaa 1b31 0075 9384 0000')
    assert str(iban) == 'RO49AAAA1B31007593840000'
    assert iban.country_code == 'RO'
    with pytest.raises(PaymentAccountError) as checksum:
        Iban('RO49AAAA1B31007593840001')
    assert checksum.value.failure == PaymentAccountError.InvalidIban(PaymentAccountError.InvalidIban.InvalidChecksum())
    assert str(checksum.value) == 'invalid IBAN: the check digits do not match'
    with pytest.raises(PaymentAccountError) as length:
        Iban('RO49AAAA1B3100759384000')
    assert length.value.failure == PaymentAccountError.InvalidIban(PaymentAccountError.InvalidIban.WrongLength(expected=24, found=23))
    assert str(Username(' ana.pays ')) == 'ana.pays'
    with pytest.raises(PaymentAccountError) as blank:
        Username(' ')
    assert blank.value.failure == PaymentAccountError.EmptyUsername()
    assert Language('pt-pt') == Language('pt-PT')
    assert str(Language('pt-pt')) == 'pt-PT'
    with pytest.raises(LanguageError) as tag:
        Language('not a tag')
    assert tag.value.failure == LanguageError.InvalidLanguage()
