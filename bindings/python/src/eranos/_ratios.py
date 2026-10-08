"""Hold the ratios that convert and divide amounts: exchange rates, percentages, and parts."""

import dataclasses
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

    @override
    def __str__(self) -> str:
        """
        Write the parts as the library does.

        Returns:
            The text, such as ``"0.5"``.

        """
        return generated.parts_format(self.hundredths)
