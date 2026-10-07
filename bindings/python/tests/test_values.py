"""Check the values the library checks and writes: currencies, amounts, ratios, dates, times, contacts, and accounts."""

import pytest

from eranos import (
    PaymentAccountError,
    CalendarDate,
    CalendarError,
    CurrencyCode,
    Email,
    ExchangeRate,
    Iban,
    Language,
    LanguageError,
    Money,
    MoneyError,
    Parts,
    Percentage,
    PhoneNumber,
    TimeOfDay,
    Username,
    ContactError,
)


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
    """A list that missed a currency or came out of order would hide one from the currency pickers of an app."""
    listed = CurrencyCode.all()
    assert list(listed) == sorted(listed)
    assert CurrencyCode('EUR') in listed


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
    assert phone.international() == '+40 712 345 678'
    with pytest.raises(ContactError) as local:
        PhoneNumber('0712345678')
    assert local.value.failure == ContactError.InvalidPhoneNumber()
    iban = Iban('ro49 aaaa 1b31 0075 9384 0000')
    assert str(iban) == 'RO49AAAA1B31007593840000'
    assert iban.country_code() == 'RO'
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
