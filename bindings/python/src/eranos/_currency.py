"""Name the currencies an amount can be in: those of ISO 4217 with minor units, as the library lists them."""

import functools
from typing import final, override

# The module of errors, not its function: its conversions make currencies, so whichever of this module and it is
# imported first finds the other partly run.
from eranos import _errors
from eranos._internal import eranos as generated


@final
@functools.total_ordering
class Currency:
    """
    A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.

    A currency is made from its code, which the library looks up, so an amount in one always formats. Currencies compare,
    hash, and order by code.
    """

    __slots__ = ('_code', '_decimal_places', '_name', '_numeric_code')

    def __init__(self, code: str) -> None:
        """
        Look up the currency of the ISO 4217 code code.

        The library refuses a code that names no currency with minor units with ``MoneyError.UnknownCurrency``, in a
        ``MoneyError``.

        Args:
            code: The three-letter code: ``"EUR"``.

        """
        with _errors.library_errors():
            details = generated.currency_details(code)
        self._code = details.code
        self._name = details.name
        self._numeric_code = details.numeric_code
        self._decimal_places = details.decimal_places

    @classmethod
    def all(cls) -> tuple[Currency, ...]:
        """
        List every currency with minor units, by code.

        Returns:
            The currencies.

        """
        return tuple(cls(details.code) for details in generated.currencies())

    @property
    def code(self) -> str:
        """The three-letter code, such as ``"EUR"``."""
        return self._code

    @property
    def name(self) -> str:
        """The English name, such as ``"Euro"``."""
        return self._name

    @property
    def numeric_code(self) -> int:
        """The three-digit numeric code, 978 for the euro."""
        return self._numeric_code

    @property
    def decimal_places(self) -> int:
        """How many decimal places its minor unit has, 2 for the euro and 0 for the yen."""
        return self._decimal_places

    @override
    def __eq__(self, other: object) -> bool:
        """
        Tell whether other is the currency of the same code.

        Args:
            other: The object to compare.

        Returns:
            Whether the two are equal.

        """
        return isinstance(other, Currency) and self._code == other.code

    @override
    def __hash__(self) -> int:
        """
        Hash the code.

        Returns:
            The hash.

        """
        return hash(self._code)

    def __lt__(self, other: Currency) -> bool:
        """
        Tell whether this currency's code comes before other's.

        Args:
            other: The currency to compare.

        Returns:
            Whether this one orders first.

        """
        return self._code < other.code

    @override
    def __repr__(self) -> str:
        """
        Write the call that makes the currency.

        Returns:
            The text, such as ``"Currency('EUR')"``.

        """
        return f'Currency({self._code!r})'

    @override
    def __str__(self) -> str:
        """
        Write the code.

        Returns:
            The text, such as ``"EUR"``.

        """
        return self._code
