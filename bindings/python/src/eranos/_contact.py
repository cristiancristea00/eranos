"""Hold the ways to reach and pay a member, and the language a group is shown in, which the library checks."""

from typing import final, override

from eranos._checked import CheckedText
from eranos._errors import library_errors
from eranos._internal import eranos as generated


@final
class Email(CheckedText):
    """An email address, checked by the library's rules, as entered but without surrounding whitespace."""

    __slots__ = ()

    def __init__(self, text: str) -> None:
        """
        Check the address written text.

        The library refuses text that breaks the rules of RFC 5321 and 5322 with ``ContactError.InvalidEmail``, in a
        ``ContactError``.

        Args:
            text: The address's text: ``"ana@example.com"``.

        """
        with library_errors():
            super().__init__(generated.email_parse(text))


@final
class PhoneNumber(CheckedText):
    """
    A phone number, valid in its country's numbering plan, kept in E.164 form, such as ``"+40712345678"``.

    It holds its international form as the library wrote it, when it checked the number or gave it with a member, so reading
    the form calls nothing; ``str`` writes the international form, and ``e164`` the stored one. Two numbers are equal when
    both their forms are.
    """

    __slots__ = ('_international',)

    def __init__(self, text: str) -> None:
        """
        Check the number written text, starting with ``+`` and the country code.

        The library refuses a number its country's plan does not have with ``ContactError.InvalidPhoneNumber``, in a
        ``ContactError``.

        Args:
            text: The number's text: ``"+40 712 345 678"``.

        """
        with library_errors():
            number = generated.phone_number_parse(text)
        super().__init__(number.e164)
        self._international = number.international

    @classmethod
    def _of(cls, e164: str, international: str) -> PhoneNumber:
        """
        Make the number the library gave in its two forms, without parsing it.

        Args:
            e164: The number in E.164 form.
            international: The number in international form.

        Returns:
            The number.

        """
        phone: PhoneNumber = object.__new__(cls)
        phone._text = e164
        phone._international = international
        return phone

    @property
    def e164(self) -> str:
        """The number in E.164 form, ``+`` followed by digits only, such as ``"+40712345678"``: the form to dial and store."""
        return self._text

    @property
    def international(self) -> str:
        """The number in its international form, grouped as its country groups it, as the library wrote it, such as ``"+40 712 345 678"``."""
        return self._international

    @override
    def __str__(self) -> str:
        """
        Write the number in its international form, as the library did.

        Returns:
            The international form.

        """
        return self._international

    @override
    def __eq__(self, other: object) -> bool:
        """
        Tell whether other is a phone number of the same E.164 form and the same international form.

        Args:
            other: The object to compare.

        Returns:
            Whether the two are equal.

        """
        return isinstance(other, PhoneNumber) and other.e164 == self._text and other.international == self._international

    @override
    def __hash__(self) -> int:
        """
        Hash the kind and both forms.

        Returns:
            The hash.

        """
        return hash((PhoneNumber, self._text, self._international))


@final
class Iban(CheckedText):
    """
    An IBAN, checked against the SWIFT registry and kept in its electronic form, such as ``"RO49AAAA1B31007593840000"``.

    It holds its country code as the library read it, when it checked the IBAN or gave it with an account, so reading the
    code calls nothing. Two IBANs are equal when their text and their country code are.
    """

    __slots__ = ('_country_code',)

    def __init__(self, text: str) -> None:
        """
        Check the IBAN written text, with or without its spaces.

        The library refuses an IBAN that fails a check with ``PaymentAccountError.InvalidIban``, whose reason names the check, in a
        ``PaymentAccountError``.

        Args:
            text: The IBAN's text: ``"RO49 AAAA 1B31 0075 9384 0000"``.

        """
        with library_errors():
            iban = generated.iban_parse(text)
        super().__init__(iban.iban)
        self._country_code = iban.country_code

    @classmethod
    def _of(cls, iban: str, country_code: str) -> Iban:
        """
        Make the IBAN the library gave with its country code, without parsing it.

        Args:
            iban: The IBAN in its electronic form.
            country_code: Its country code.

        Returns:
            The IBAN.

        """
        account: Iban = object.__new__(cls)
        account._text = iban
        account._country_code = country_code
        return account

    @property
    def country_code(self) -> str:
        """The two letters of the IBAN's country, as the library read them, such as ``"RO"``."""
        return self._country_code

    @override
    def __eq__(self, other: object) -> bool:
        """
        Tell whether other is an IBAN of the same text and the same country code.

        Args:
            other: The object to compare.

        Returns:
            Whether the two are equal.

        """
        return isinstance(other, Iban) and str(other) == str(self) and other.country_code == self._country_code

    @override
    def __hash__(self) -> int:
        """
        Hash the kind, the text, and the country code.

        Returns:
            The hash.

        """
        return hash((Iban, str(self), self._country_code))


@final
class Username(CheckedText):
    """The username of an account in a payment app, as entered but without surrounding whitespace."""

    __slots__ = ()

    def __init__(self, text: str) -> None:
        """
        Check the username written text.

        The library refuses blank text with ``PaymentAccountError.EmptyUsername``, in a ``PaymentAccountError``.

        Args:
            text: The username's text: ``"ana.pays"``.

        """
        with library_errors():
            super().__init__(generated.username_parse(text))


@final
class Language(CheckedText):
    """A language as a BCP 47 tag, in its canonical form, such as ``"pt-PT"``."""

    __slots__ = ()

    def __init__(self, text: str) -> None:
        """
        Check the language tag written text.

        The library refuses text that is not a tag with ``LanguageError.InvalidLanguage``, in a ``LanguageError``.

        Args:
            text: The tag's text: ``"pt-pt"``.

        """
        with library_errors():
            super().__init__(generated.language_parse(text))


def phone_number_of(number: generated.PhoneNumber) -> PhoneNumber:
    """
    Convert the phone number the generated module holds as number, without parsing it.

    Args:
        number: The generated number.

    Returns:
        The number.

    """
    # The class's own constructor parses text, for an app's input; the library's answers already carry both forms.
    return PhoneNumber._of(number.e164, number.international)  # ruff: ignore[private-member-access]


def generated_phone_number(phone: PhoneNumber) -> generated.PhoneNumber:
    """
    Convert phone into the phone number the generated module holds.

    Args:
        phone: The number.

    Returns:
        The generated number.

    """
    return generated.PhoneNumber(e164=phone.e164, international=phone.international)


def iban_of(iban: str, country_code: str) -> Iban:
    """
    Convert the IBAN and the country code the generated module holds, without parsing the IBAN.

    Args:
        iban: The IBAN in its electronic form.
        country_code: Its country code.

    Returns:
        The IBAN.

    """
    # The class's own constructor parses text, for an app's input; the library's answers already carry the country code.
    return Iban._of(iban, country_code)  # ruff: ignore[private-member-access]
