"""Hold quantities: how many units a line of a bill counts, a whole number or a decimal with four places."""

import dataclasses
from typing import ClassVar, final, override

from eranos._errors import library_errors
from eranos._internal import eranos as generated


@final
@dataclasses.dataclass(frozen=True, slots=True)
class Quantity:
    """
    A positive quantity with four decimal places, held in ten-thousandths: 0.452 kg is 4520.

    The quantity of an item multiplies its unit price into its line total. The library checks the quantity when it is made,
    and refuses a number that is not positive with ``MoneyError.InvalidQuantity``, in a ``MoneyError``.
    """

    ten_thousandths: int
    """The quantity in ten-thousandths, 10000 for one unit and 4520 for 0.452."""

    ONE: ClassVar[Quantity]
    """One unit, 10000 ten-thousandths, which an item given no quantity has; the library makes it when the package loads."""

    def __post_init__(self) -> None:
        """Check the quantity with the library."""
        with library_errors():
            generated.quantity_check(self.ten_thousandths)

    @classmethod
    def whole(cls, units: int) -> Quantity:
        """
        Make the quantity of units whole units: ``3`` is 30000 ten-thousandths.

        The library refuses a number that is not positive, or whose ten-thousandths do not fit in 64 bits, with
        ``MoneyError.InvalidQuantity``, in a ``MoneyError``.

        Args:
            units: The number of whole units.

        Returns:
            The quantity.

        """
        with library_errors():
            return cls(generated.quantity_from_whole(units))

    @classmethod
    def parse(cls, text: str) -> Quantity:
        """
        Parse the quantity written text: ``"2"``, ``"1.5"``, or ``"0.452"``.

        The library refuses text that is not a positive decimal with at most four decimal places, without a sign or an
        exponent, or whose ten-thousandths do not fit in 64 bits, with ``MoneyError.InvalidQuantity``, in a ``MoneyError``.

        Args:
            text: The quantity's text.

        Returns:
            The quantity.

        """
        with library_errors():
            return cls(generated.quantity_parse(text))

    @classmethod
    def from_float(cls, value: float) -> Quantity:
        """
        Convert the float value into a quantity, for an app that holds quantities as floats.

        The library writes the float in its shortest decimal form, rounds it half away from zero to four decimal places, and
        parses it, as ``Money.from_float`` converts an amount: ``0.45205`` is 0.4521. The library refuses NaN and the
        infinities with ``MoneyError.NotFinite``, and a value that rounds to zero or below, as ``0.00004`` does, or whose
        ten-thousandths do not fit in 64 bits with ``MoneyError.InvalidQuantity``, in a ``MoneyError``.

        Args:
            value: The quantity, as a float.

        Returns:
            The quantity.

        """
        with library_errors():
            return cls(generated.quantity_from_f64(value))

    @override
    def __str__(self) -> str:
        """
        Write the quantity as the library does, without trailing zeros.

        Returns:
            The text, such as ``"0.452"`` or ``"3"``.

        """
        return generated.quantity_format(self.ten_thousandths)


# The class cannot hold an instance of itself until it is defined, so its one unit is made from the core once it is.
Quantity.ONE = Quantity.whole(1)
