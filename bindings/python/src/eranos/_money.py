"""Hold amounts of money: whole numbers of minor units of a currency."""

import dataclasses
from typing import final, override

# The module of errors, not its function: its conversions make amounts, so whichever of this module and it is imported
# first finds the other partly run.
from eranos import _errors
from eranos._currency_code import CurrencyCode, currency_of, generated_currency
from eranos._internal import eranos as generated


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Money:
    """
    An amount: a whole number of minor units of a currency, cents for the euro.

    An amount prints as the library writes it, ``12.50 EUR``, with the decimal places its currency holds, and adds and
    subtracts with the library's checks: both amounts in one currency, and the result within the range of a 64-bit number
    of minor units.
    """

    minor_units: int
    """The amount in minor units, cents for the euro and none for the yen."""

    currency: CurrencyCode
    """The currency."""

    @classmethod
    def parse(cls, text: str, currency: CurrencyCode) -> Money:
        """
        Parse the amount written text in currency: ``"12.50"``, ``"12.5"``, or ``"-3"``.

        The library refuses text that is not a decimal number with ``MoneyError.InvalidAmount``, and more decimal places
        than the currency has with ``MoneyError.TooManyDecimals``, in a ``MoneyError``.

        Args:
            text: The amount's text.
            currency: The currency.

        Returns:
            The amount.

        """
        with _errors.library_errors():
            return money_of(generated.money_parse(text, currency.code))

    def plus(self, other: Money) -> Money:
        """
        Add other to this amount.

        The library refuses amounts in two currencies with ``MoneyError.CurrencyMismatch``, and a sum out of range with
        ``MoneyError.Overflow``, in a ``MoneyError``.

        Args:
            other: The amount to add.

        Returns:
            The sum.

        """
        with _errors.library_errors():
            return money_of(generated.money_add(generated_money(self), generated_money(other)))

    def minus(self, other: Money) -> Money:
        """
        Subtract other from this amount.

        The library refuses amounts in two currencies with ``MoneyError.CurrencyMismatch``, and a difference out of range
        with ``MoneyError.Overflow``, in a ``MoneyError``.

        Args:
            other: The amount to subtract.

        Returns:
            The difference.

        """
        with _errors.library_errors():
            return money_of(generated.money_subtract(generated_money(self), generated_money(other)))

    @override
    def __str__(self) -> str:
        """
        Write the amount as the library does.

        Returns:
            The text, such as ``"12.50 EUR"``.

        """
        return generated.money_format(generated_money(self))


def money_of(value: generated.Money) -> Money:
    """
    Convert the amount the generated module holds as value.

    Args:
        value: The generated amount.

    Returns:
        The amount.

    """
    return Money(value.minor_units, currency_of(value.currency))


def generated_money(money: Money) -> generated.Money:
    """
    Convert money into the amount the generated module holds.

    Args:
        money: The amount.

    Returns:
        The generated amount.

    """
    return generated.Money(minor_units=money.minor_units, currency=generated_currency(money.currency))
