"""Check the values the library checks and writes: currencies, amounts, ratios, dates, times, contacts, and accounts."""

import pytest

from eranos import (
    AccountError,
    CalendarDate,
    CalendarError,
    Currency,
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
    euro = Currency('EUR')
    assert (euro.code, euro.decimal_places, euro.name, euro.numeric_code) == ('EUR', 2, 'Euro', 978)
    assert Currency('JPY').decimal_places == 0
    assert euro == Currency('EUR')
    assert euro != Currency('JPY')
    assert euro != 'EUR'
    assert hash(euro) == hash(Currency('EUR'))
    assert euro < Currency('JPY')
    assert repr(euro) == "Currency('EUR')"
    assert str(euro) == 'EUR'


def test_every_currency_the_library_lists_is_listed_by_code() -> None:
    """A list that missed a currency or came out of order would hide one from the currency pickers of an app."""
    listed = Currency.all()
    assert list(listed) == sorted(listed)
    assert Currency('EUR') in listed


def test_a_code_the_library_does_not_list_is_refused() -> None:
    """A code the library does not list, taken, would make an amount no call can format."""
    with pytest.raises(MoneyError) as refused:
        Currency('XYZ')
    assert refused.value.failure == MoneyError.UnknownCurrency()
    assert str(refused.value) == 'unknown currency code'


def test_text_that_is_not_utf8_is_refused_before_the_library_sees_it() -> None:
    """Text with a lone surrogate, passed on, would reach the library as text it cannot hold."""
    with pytest.raises(UnicodeEncodeError):
        Currency('\ud800')


def test_an_amount_parses_and_formats_as_the_library_writes_it() -> None:
    """An amount that parsed or formatted otherwise than the library does would show one number in an app and store another."""
    euro = Currency('EUR')
    price = Money.parse('12.5', euro)
    assert price == Money(euro, 1250)
    assert str(price) == '12.50 EUR'
    with pytest.raises(MoneyError) as word:
        Money.parse('twelve', euro)
    assert word.value.failure == MoneyError.InvalidAmount()
    with pytest.raises(MoneyError) as precise:
        Money.parse('1.234', euro)
    assert precise.value.failure == MoneyError.TooManyDecimals(allowed=2, currency=euro)


def test_amounts_in_one_currency_add_and_subtract() -> None:
    """Arithmetic that mixed currencies or wrapped around would put a wrong total in a report."""
    euro = Currency('EUR')
    lei = Currency('RON')
    assert Money(euro, 1000).plus(Money(euro, 250)) == Money(euro, 1250)
    assert Money(euro, 1000).minus(Money(euro, 250)) == Money(euro, 750)
    with pytest.raises(MoneyError) as mixed:
        Money(euro, 1).plus(Money(lei, 1))
    assert mixed.value.failure == MoneyError.CurrencyMismatch(left=euro, right=lei)
    with pytest.raises(MoneyError) as largest:
        Money(euro, 2**63 - 1).plus(Money(euro, 1))
    assert largest.value.failure == MoneyError.Overflow()
    with pytest.raises(MoneyError) as smallest:
        Money(euro, -(2**63)).minus(Money(euro, 1))
    assert smallest.value.failure == MoneyError.Overflow()


def test_a_rate_parses_and_converts_as_the_library_does() -> None:
    """A rate that parsed or converted otherwise than the library does would change the share of every converted expense."""
    euro = Currency('EUR')
    lei = Currency('RON')
    rate = ExchangeRate('4.970')
    assert str(rate) == '4.97'
    assert rate == ExchangeRate('4.97')
    assert rate != ExchangeRate('5')
    assert rate != Percentage(497)
    assert hash(rate) == hash(ExchangeRate('4.97'))
    assert repr(rate) == "ExchangeRate('4.97')"
    assert ExchangeRate('0.25').convert(Money(lei, 2), euro) == Money(euro, 1)
    with pytest.raises(MoneyError) as zero:
        ExchangeRate('0')
    assert zero.value.failure == MoneyError.InvalidExchangeRate()
    with pytest.raises(MoneyError) as overflow:
        ExchangeRate('2').convert(Money(lei, 2**63 - 1), euro)
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
    date = CalendarDate(23, 9, 2026)
    assert (date.day, date.month, date.year, str(date)) == (23, 9, 2026, '2026-09-23')
    assert CalendarDate.parse('2026-09-23') == date
    assert CalendarDate(24, 8, 2026) < date < CalendarDate(1, 1, 2027)
    with pytest.raises(CalendarError) as february:
        CalendarDate(30, 2, 2026)
    assert february.value.failure == CalendarError.InvalidDate()
    with pytest.raises(CalendarError) as parsed:
        CalendarDate.parse('2026-02-30')
    assert parsed.value.failure == CalendarError.InvalidDate()
    time = TimeOfDay(18, 30)
    assert (time.hour, time.minute, str(time)) == (18, 30, '18:30')
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
    assert (str(phone), phone.international()) == ('+40712345678', '+40 712 345 678')
    with pytest.raises(ContactError) as local:
        PhoneNumber('0712345678')
    assert local.value.failure == ContactError.InvalidPhoneNumber()
    iban = Iban('ro49 aaaa 1b31 0075 9384 0000')
    assert (str(iban), iban.country_code()) == ('RO49AAAA1B31007593840000', 'RO')
    with pytest.raises(AccountError) as checksum:
        Iban('RO49AAAA1B31007593840001')
    assert checksum.value.failure == AccountError.InvalidIban(AccountError.InvalidIban.InvalidChecksum())
    assert str(checksum.value) == 'invalid IBAN: the check digits do not match'
    with pytest.raises(AccountError) as length:
        Iban('RO49AAAA1B3100759384000')
    assert length.value.failure == AccountError.InvalidIban(AccountError.InvalidIban.WrongLength(expected=24, found=23))
    assert str(Username(' ana.pays ')) == 'ana.pays'
    with pytest.raises(AccountError) as blank:
        Username(' ')
    assert blank.value.failure == AccountError.EmptyUsername()
    assert Language('pt-pt') == Language('pt-PT')
    assert str(Language('pt-pt')) == 'pt-PT'
    with pytest.raises(LanguageError) as tag:
        Language('not a tag')
    assert tag.value.failure == LanguageError.InvalidLanguage()
