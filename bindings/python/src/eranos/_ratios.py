"""Hold the ratios that convert and divide amounts: exchange rates, with the currency each converts, percentages, and parts."""

import dataclasses
from collections.abc import Iterable
from typing import final, override

from eranos._checked import CheckedText
from eranos._currency_code import CurrencyCode
from eranos._errors import library_errors
from eranos._internal import eranos as generated
from eranos._money import Money, generated_money, money_of


@final
class ExchangeRate(CheckedText):
    """The rate that converts an amount in one currency into another: a positive decimal, as the library writes it."""

    __slots__ = ()

    def __init__(self, text: str) -> None:
        """
        Check the rate written text: ``"4.97"`` or ``"0.2012"``.

        The library refuses text that is not a positive decimal with at most 18 decimal places with
        ``MoneyError.InvalidExchangeRate``, in a ``MoneyError``.

        Args:
            text: The rate's text.

        """
        with library_errors():
            super().__init__(generated.exchange_rate_parse(text))

    @classmethod
    def from_float(cls, value: float) -> ExchangeRate:
        """
        Convert the float value into an exchange rate, for an app that holds rates as floats.

        The library writes the float in its shortest decimal form, rounds it half away from zero to 18 decimal places, and
        parses it, as ``Money.from_float`` converts an amount, so the rate keeps every digit of that form up to the 18th
        place: ``25.5`` is ``"25.5"``, and ``0.1 + 0.2`` is ``"0.30000000000000004"``. The library refuses NaN and the
        infinities with ``MoneyError.NotFinite``, and a value that is not positive at 18 decimal places or has more
        significant digits than fit in 64 bits with ``MoneyError.InvalidExchangeRate``, in a ``MoneyError``.

        Args:
            value: The rate, as a float.

        Returns:
            The rate.

        """
        with library_errors():
            return cls(generated.exchange_rate_from_f64(value))

    def convert(self, amount: Money, currency: CurrencyCode) -> Money:
        """
        Convert amount at this rate into currency, rounded to its minor unit, halves away from zero.

        The library refuses a result out of range with ``MoneyError.Overflow``, in a ``MoneyError``.

        Args:
            amount: The amount to convert.
            currency: The currency to convert it into.

        Returns:
            The converted amount.

        """
        with library_errors():
            return money_of(generated.exchange_rate_convert(str(self), generated_money(amount), currency.code))


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Percentage:
    """
    A share of a whole, to a hundredth of a per cent: 12.5 % is 1250 basis points.

    The library checks the percentage when it is made, and refuses a negative one with ``MoneyError.InvalidPercentage``, in
    a ``MoneyError``.
    """

    basis_points: int
    """The percentage in basis points, 1250 for 12.5 %."""

    def __post_init__(self) -> None:
        """Check the percentage with the library."""
        with library_errors():
            generated.percentage_check(self.basis_points)

    @classmethod
    def parse(cls, text: str) -> Percentage:
        """
        Parse the percentage written text: ``"12.5"``, ``"12.5%"``, or ``"100"``.

        The library refuses text that is not a non-negative decimal with at most two decimal places with
        ``MoneyError.InvalidPercentage``, in a ``MoneyError``.

        Args:
            text: The percentage's text.

        Returns:
            The percentage.

        """
        with library_errors():
            return cls(generated.percentage_parse(text))

    @classmethod
    def from_float(cls, value: float) -> Percentage:
        """
        Convert the float value, a number of per cent, into a percentage, for an app that holds percentages as floats.

        The library writes the float in its shortest decimal form, rounds it half away from zero to two decimal places, and
        parses it, as ``Money.from_float`` converts an amount: ``12.345`` is 12.35 %, and ``0.1 + 0.2``, which is
        0.30000000000000004 as a float, is 0.3 %. The library refuses NaN and the infinities with ``MoneyError.NotFinite``,
        and a negative value or one whose basis points do not fit in 32 bits with ``MoneyError.InvalidPercentage``, in a
        ``MoneyError``.

        Args:
            value: The percentage, as a float number of per cent.

        Returns:
            The percentage.

        """
        with library_errors():
            return cls(generated.percentage_from_f64(value))

    @override
    def __str__(self) -> str:
        """
        Write the percentage as the library does.

        Returns:
            The text, such as ``"12.5%"``.

        """
        return generated.percentage_format(self.basis_points)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Parts:
    """
    A number of parts, to a hundredth: a member with half a part pays half of what a member with one part pays.

    The library checks the parts when they are made, and refuses a number that is not positive with
    ``MoneyError.InvalidParts``, in a ``MoneyError``.
    """

    hundredths: int
    """The parts in hundredths, 50 for half a part."""

    def __post_init__(self) -> None:
        """Check the parts with the library."""
        with library_errors():
            generated.parts_check(self.hundredths)

    @classmethod
    def parse(cls, text: str) -> Parts:
        """
        Parse the parts written text: ``"1"``, ``"0.5"``, or ``"2.25"``.

        The library refuses text that is not a positive decimal with at most two decimal places with
        ``MoneyError.InvalidParts``, in a ``MoneyError``.

        Args:
            text: The parts' text.

        Returns:
            The parts.

        """
        with library_errors():
            return cls(generated.parts_parse(text))

    @classmethod
    def from_float(cls, value: float) -> Parts:
        """
        Convert the float value into parts, for an app that holds parts as floats.

        The library writes the float in its shortest decimal form, rounds it half away from zero to two decimal places, and
        parses it, as ``Money.from_float`` converts an amount: ``1.005`` is 1.01 parts. The library refuses NaN and the
        infinities with ``MoneyError.NotFinite``, and a value that rounds to zero or below, as ``0.004`` does, or whose
        hundredths do not fit in 32 bits with ``MoneyError.InvalidParts``, in a ``MoneyError``.

        Args:
            value: The parts, as a float.

        Returns:
            The parts.

        """
        with library_errors():
            return cls(generated.parts_from_f64(value))

    @override
    def __str__(self) -> str:
        """
        Write the parts as the library does.

        Returns:
            The text, such as ``"0.5"``.

        """
        return generated.parts_format(self.hundredths)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class CurrencyRate:
    """The rate into a new group currency of an amount in currency."""

    currency: CurrencyCode
    """The currency the rate converts from."""

    rate: ExchangeRate
    """The rate."""


def generated_rates(rates: Iterable[CurrencyRate]) -> list[generated.CurrencyRate]:
    """
    Convert rates into the rates the generated module holds.

    Args:
        rates: The rates.

    Returns:
        The generated rates.

    """
    return [generated.CurrencyRate(currency=rate.currency.code, rate=str(rate.rate)) for rate in rates]
