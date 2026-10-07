"""Hold the ways to reach and pay a member, and the language a group is shown in, which the library checks."""

from typing import final

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
    """A phone number, valid in its country's numbering plan, kept in E.164 form, such as ``"+40712345678"``."""

    __slots__ = ()

    def __init__(self, text: str) -> None:
        """
        Check the number written text, starting with ``+`` and the country code.

        The library refuses a number its country's plan does not have with ``ContactError.InvalidPhoneNumber``, in a
        ``ContactError``.

        Args:
            text: The number's text: ``"+40 712 345 678"``.

        """
        with library_errors():
            super().__init__(generated.phone_number_parse(text))

    def international(self) -> str:
        """
        Write the number in its international form, grouped as its country writes it.

        Returns:
            The text, such as ``"+40 712 345 678"``.

        """
        return generated.phone_number_international(str(self))


@final
class Iban(CheckedText):
    """An IBAN, checked against the SWIFT registry and kept in its electronic form, such as ``"RO49AAAA1B31007593840000"``."""

    __slots__ = ()

    def __init__(self, text: str) -> None:
        """
        Check the IBAN written text, with or without its spaces.

        The library refuses an IBAN that fails a check with ``PaymentAccountError.InvalidIban``, whose reason names the check, in an
        ``PaymentAccountError``.

        Args:
            text: The IBAN's text: ``"RO49 AAAA 1B31 0075 9384 0000"``.

        """
        with library_errors():
            super().__init__(generated.iban_parse(text))

    def country_code(self) -> str:
        """
        Read the IBAN's country code.

        Returns:
            The two letters, such as ``"RO"``.

        """
        return generated.iban_country_code(str(self))


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
