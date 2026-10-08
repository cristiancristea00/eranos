"""Report why a call failed: an error per module of the library, each holding what the module reports."""

import asyncio
import contextlib
import dataclasses
from collections.abc import Callable, Iterator
from typing import Any, final

# The modules of currencies and amounts, not their classes: each calls into the library through library_errors, so
# whichever of this module and theirs is imported first finds the other partly run.
from eranos import _currency_code, _money
from eranos._identifiers import CategoryId, ExpenseId, MemberId, TransferId
from eranos._internal import eranos as generated


class EranosError(Exception):
    """
    Why a call failed: the base of an error per module of the library, each holding what the module reports.

    Every call that the library refuses raises a subclass, whose message is the library's, so an ``except`` takes a
    module's failures together, and a ``match`` over its ``failure`` takes one of them::

        try:
            group.remove_member(member)
        except ExpenseGroupError as error:
            match error.failure:
                case ExpenseGroupError.MemberInUse():
                    group.archive_member(member)  # The member still has entries; archive them instead.
                case _:
                    raise

    A call the program should not make raises the standard error instead: ``RuntimeError`` for a read through an
    ``ExpenseGroupReader`` after its release. A call whose task is cancelled raises ``asyncio.CancelledError``.
    """


@final
class IdError(EranosError):
    """The library's ``IdError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class Empty:
        """An identifier that is empty."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class TooLong:
        """An identifier longer than 64 bytes of UTF-8."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class ControlCharacter:
        """An identifier that holds a control character, such as a tab or a line break."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class SurroundingWhitespace:
        """An identifier that starts or ends with whitespace."""

    def __init__(self, failure: IdFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: IdFailure = failure
        """What the module reports."""


type IdFailure = IdError.Empty | IdError.TooLong | IdError.ControlCharacter | IdError.SurroundingWhitespace
"""What ``IdError`` can report: a class per failure, with what it holds."""


@final
class PaymentAccountError(EranosError):
    """The library's ``PaymentAccountError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptyName:
        """An account whose name is blank."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidIban:
        """An IBAN that failed validation, and why."""

        @final
        @dataclasses.dataclass(frozen=True, slots=True)
        class InvalidCharacter:
            """A character other than an ASCII letter, a digit, or whitespace (which is dropped)."""

        @final
        @dataclasses.dataclass(frozen=True, slots=True)
        class InvalidFormat:
            """Not two letters followed by two check digits."""

        @final
        @dataclasses.dataclass(frozen=True, slots=True)
        class UnknownCountry:
            """A country code without an entry in the SWIFT IBAN registry."""

        @final
        @dataclasses.dataclass(frozen=True, slots=True)
        class WrongLength:
            """The wrong number of characters for the country."""

            expected: int
            """How many characters the country's IBANs have."""

            found: int
            """How many characters the IBAN has."""

        @final
        @dataclasses.dataclass(frozen=True, slots=True)
        class InvalidStructure:
            """
            A letter where the country's account number has a digit, or a digit where it has a letter.

            The account number's structure is the BBAN structure of the SWIFT IBAN registry.
            """

        @final
        @dataclasses.dataclass(frozen=True, slots=True)
        class InvalidChecksum:
            """The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97 check failed: the number contains a typo."""

        reason: IbanReason
        """The check the IBAN failed."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptyUsername:
        """A username that is blank."""

    def __init__(self, failure: PaymentAccountFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: PaymentAccountFailure = failure
        """What the module reports."""


type PaymentAccountFailure = PaymentAccountError.EmptyName | PaymentAccountError.InvalidIban | PaymentAccountError.EmptyUsername
"""What ``PaymentAccountError`` can report: a class per failure, with what it holds."""


type IbanReason = (
    PaymentAccountError.InvalidIban.InvalidCharacter
    | PaymentAccountError.InvalidIban.InvalidFormat
    | PaymentAccountError.InvalidIban.UnknownCountry
    | PaymentAccountError.InvalidIban.WrongLength
    | PaymentAccountError.InvalidIban.InvalidStructure
    | PaymentAccountError.InvalidIban.InvalidChecksum
)
"""Why the library refused an IBAN: a class per check it failed, with what it holds."""


@final
class CalendarError(EranosError):
    """The library's ``CalendarError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidDate:
        """A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidTime:
        """A time of day that does not exist, has seconds, or is not ISO 8601 text."""

    def __init__(self, failure: CalendarFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: CalendarFailure = failure
        """What the module reports."""


type CalendarFailure = CalendarError.InvalidDate | CalendarError.InvalidTime
"""What ``CalendarError`` can report: a class per failure, with what it holds."""


@final
class CategoryError(EranosError):
    """The library's ``CategoryError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptyName:
        """A name that is blank."""

    def __init__(self, failure: CategoryFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: CategoryFailure = failure
        """What the module reports."""


type CategoryFailure = CategoryError.EmptyName
"""What ``CategoryError`` can report: a class per failure, with what it holds."""


@final
class ContactError(EranosError):
    """The library's ``ContactError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidEmail:
        """
        An email address that breaks the RFC 5321 and 5322 rules.

        It has no local part or domain around an ``@``, a malformed one, or one too long, or it holds whitespace or an invisible character.
        """

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidPhoneNumber:
        """A phone number that does not start with ``+``, holds a letter or an extension, or is not valid in its country's numbering plan."""

    def __init__(self, failure: ContactFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: ContactFailure = failure
        """What the module reports."""


type ContactFailure = ContactError.InvalidEmail | ContactError.InvalidPhoneNumber
"""What ``ContactError`` can report: a class per failure, with what it holds."""


@final
class MoneyError(EranosError):
    """The library's ``MoneyError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidAmount:
        """An amount that is not a decimal number."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class TooManyDecimals:
        """An amount with more decimal places than its currency has."""

        currency: _currency_code.CurrencyCode
        """The currency."""

        allowed: int
        """How many decimal places the currency has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class CurrencyMismatch:
        """Arithmetic between amounts of two different currencies."""

        left: _currency_code.CurrencyCode
        """The currency of the first amount."""

        right: _currency_code.CurrencyCode
        """The currency of the second amount."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class UnknownCurrency:
        """A currency code that is not an active ISO 4217 code."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidExchangeRate:
        """An exchange rate that is not a positive decimal number with at most 18 decimal places."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidParts:
        """A number of parts that is not a positive decimal number with at most two decimal places."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidPercentage:
        """A percentage that is not a decimal number with at most two decimal places."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidQuantity:
        """A quantity that is not a positive decimal number with at most four decimal places, or whose ten-thousandths do not fit in 64 bits."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class NotFinite:
        """A float that is NaN or infinite, which no amount, percentage, number of parts, quantity, or exchange rate is."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class Overflow:
        """A number or a result that does not fit its range."""

    def __init__(self, failure: MoneyFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: MoneyFailure = failure
        """What the module reports."""


type MoneyFailure = (
    MoneyError.InvalidAmount
    | MoneyError.TooManyDecimals
    | MoneyError.CurrencyMismatch
    | MoneyError.UnknownCurrency
    | MoneyError.InvalidExchangeRate
    | MoneyError.InvalidParts
    | MoneyError.InvalidPercentage
    | MoneyError.InvalidQuantity
    | MoneyError.NotFinite
    | MoneyError.Overflow
)
"""What ``MoneyError`` can report: a class per failure, with what it holds."""


@final
class MemberError(EranosError):
    """The library's ``MemberError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptyFirstName:
        """A first name that is blank."""

    def __init__(self, failure: MemberFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: MemberFailure = failure
        """What the module reports."""


type MemberFailure = MemberError.EmptyFirstName
"""What ``MemberError`` can report: a class per failure, with what it holds."""


@final
class ExpenseError(EranosError):
    """The library's ``ExpenseError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptyName:
        """An expense whose name is blank."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class NonPositiveAmount:
        """The amount or unit price is zero or negative."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class CurrencyMismatch:
        """An amount inside the expense is in a different currency."""

        expected: _currency_code.CurrencyCode
        """The expense's currency."""

        found: _currency_code.CurrencyCode
        """The currency of the amount inside it."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptySplit:
        """A split lists no member."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DuplicateMember:
        """A member appears more than once in one split."""

        member: MemberId
        """The member who appears more than once."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class NonPositiveShare:
        """A member's percentage or amount is zero or negative."""

        member: MemberId
        """The member whose share is zero or negative."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class AmountsDoNotAddUp:
        """The amounts of a split do not add up to the amount being split."""

        expected: _money.Money
        """The amount being split."""

        actual: _money.Money
        """What the amounts add up to."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class PercentagesDoNotAddUp:
        """The percentages of a split do not add up to exactly 100 %."""

        total_basis_points: int
        """What the percentages add up to, in basis points."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class NoItems:
        """An itemised expense without items."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptyItemName:
        """An item whose name is blank."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class ItemsDoNotAddUp:
        """The item line totals do not add up to the expense amount."""

        expected: _money.Money
        """The expense amount."""

        actual: _money.Money
        """What the line totals add up to."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class NegativeDiscount:
        """A fixed discount below zero."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DiscountAboveHundredPercent:
        """A percentage discount above 100 %."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DiscountAbovePrice:
        """A fixed discount larger than what it reduces: the line total before the discount, or the unit price for a discount off each unit."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class NegativeTip:
        """A fixed tip below zero."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class Overflow:
        """A total that does not fit the range of an amount."""

    def __init__(self, failure: ExpenseFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: ExpenseFailure = failure
        """What the module reports."""


type ExpenseFailure = (
    ExpenseError.EmptyName
    | ExpenseError.NonPositiveAmount
    | ExpenseError.CurrencyMismatch
    | ExpenseError.EmptySplit
    | ExpenseError.DuplicateMember
    | ExpenseError.NonPositiveShare
    | ExpenseError.AmountsDoNotAddUp
    | ExpenseError.PercentagesDoNotAddUp
    | ExpenseError.NoItems
    | ExpenseError.EmptyItemName
    | ExpenseError.ItemsDoNotAddUp
    | ExpenseError.NegativeDiscount
    | ExpenseError.DiscountAboveHundredPercent
    | ExpenseError.DiscountAbovePrice
    | ExpenseError.NegativeTip
    | ExpenseError.Overflow
)
"""What ``ExpenseError`` can report: a class per failure, with what it holds."""


@final
class SettlementError(EranosError):
    """The library's ``SettlementError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class OutOfMemory:
        """The exact solver needs 2ⁿ bytes for ``n`` open balances, and that table does not fit in the address space or the allocator refused it."""

        balances: int
        """How many open balances the plan has."""

    def __init__(self, failure: SettlementFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: SettlementFailure = failure
        """What the module reports."""


type SettlementFailure = SettlementError.OutOfMemory
"""What ``SettlementError`` can report: a class per failure, with what it holds."""


@final
class TransferError(EranosError):
    """The library's ``TransferError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class SameMember:
        """The sender and the recipient are the same member."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class NonPositiveAmount:
        """The amount is zero or negative."""

    def __init__(self, failure: TransferFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: TransferFailure = failure
        """What the module reports."""


type TransferFailure = TransferError.SameMember | TransferError.NonPositiveAmount
"""What ``TransferError`` can report: a class per failure, with what it holds."""


@final
class ExpenseGroupError(EranosError):
    """The library's ``ExpenseGroupError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class UnsupportedFormat:
        """A saved group in a format this version does not read: a newer one, whose fields it would drop, or an earlier one."""

        format: int
        """The format the saved group names."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class EmptyName:
        """A group name that is blank."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class MissingExchangeRate:
        """
        An entry in another currency than the group's has no exchange rate into the group currency.

        Its cost is local, or a currency change gave no rate for its currency.
        """

        currency: _currency_code.CurrencyCode
        """The currency without a rate."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class UnexpectedExchangeRate:
        """An entry in the group currency has a converted cost; only a local cost is in the group currency."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DuplicateExchangeRate:
        """The same currency appears twice among the rates of a currency change."""

        currency: _currency_code.CurrencyCode
        """The currency that appears twice."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DuplicateMember:
        """A member with this ID already exists."""

        member: MemberId
        """The identifier another member has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class UnknownMember:
        """No member with this ID."""

        member: MemberId
        """The identifier no member has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class ArchivedMember:
        """An archived member cannot join a new expense."""

        member: MemberId
        """The archived member."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class MemberInUse:
        """The member is still referenced and cannot be removed; archive it instead."""

        member: MemberId
        """The member."""

        expenses: tuple[ExpenseId, ...]
        """The expenses that name the member."""

        transfers: tuple[TransferId, ...]
        """The transfers that name the member."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DuplicateCategory:
        """A category with this ID already exists."""

        category: CategoryId
        """The identifier another category has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DuplicateCategoryName:
        """Another category already has this name, compared by Unicode canonical caseless matching."""

        existing: CategoryId
        """The category that has the name."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class UnknownCategory:
        """No category with this ID."""

        category: CategoryId
        """The identifier no category has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class BuiltInCategory:
        """The built-in Uncategorised category cannot be renamed or removed."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DuplicateExpense:
        """An expense with this ID already exists."""

        expense: ExpenseId
        """The identifier another expense has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class UnknownExpense:
        """No expense with this ID."""

        expense: ExpenseId
        """The identifier no expense has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class DuplicateTransfer:
        """A transfer with this ID already exists."""

        transfer: TransferId
        """The identifier another transfer has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class UnknownTransfer:
        """No transfer with this ID."""

        transfer: TransferId
        """The identifier no transfer has."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class Busy:
        """A shared group's lock was asked for without waiting, while another call held it."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class Overflow:
        """A total would leave the range of an amount."""

    def __init__(self, failure: ExpenseGroupFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: ExpenseGroupFailure = failure
        """What the module reports."""


type ExpenseGroupFailure = (
    ExpenseGroupError.UnsupportedFormat
    | ExpenseGroupError.EmptyName
    | ExpenseGroupError.MissingExchangeRate
    | ExpenseGroupError.UnexpectedExchangeRate
    | ExpenseGroupError.DuplicateExchangeRate
    | ExpenseGroupError.DuplicateMember
    | ExpenseGroupError.UnknownMember
    | ExpenseGroupError.ArchivedMember
    | ExpenseGroupError.MemberInUse
    | ExpenseGroupError.DuplicateCategory
    | ExpenseGroupError.DuplicateCategoryName
    | ExpenseGroupError.UnknownCategory
    | ExpenseGroupError.BuiltInCategory
    | ExpenseGroupError.DuplicateExpense
    | ExpenseGroupError.UnknownExpense
    | ExpenseGroupError.DuplicateTransfer
    | ExpenseGroupError.UnknownTransfer
    | ExpenseGroupError.Busy
    | ExpenseGroupError.Overflow
)
"""What ``ExpenseGroupError`` can report: a class per failure, with what it holds."""


@final
class LanguageError(EranosError):
    """The library's ``LanguageError``, with the message it words: what the module reports, a class per failure."""

    @final
    @dataclasses.dataclass(frozen=True, slots=True)
    class InvalidLanguage:
        """A language tag that is not a Unicode BCP 47 locale identifier (UTS #35)."""

    def __init__(self, failure: LanguageFailure, message: str) -> None:
        """
        Make the error of failure, which the library words as message.

        Args:
            failure: What the module reports.
            message: The library's message.

        """
        super().__init__(message)
        self.failure: LanguageFailure = failure
        """What the module reports."""


type LanguageFailure = LanguageError.InvalidLanguage
"""What ``LanguageError`` can report: a class per failure, with what it holds."""


@final
class InvalidJsonError(EranosError):
    """
    A saved group that is not JSON of the saved form, at its line and column.

    The message names the fault, but never the text there, which can hold an account number.
    """

    def __init__(self, message: str, *, line: int, column: int) -> None:
        """
        Make the error of a saved group that stops being one at line and column, which the library words as message.

        Args:
            message: The library's message.
            line: The line where the text stops being a saved group, from 1.
            column: The column where the text stops being a saved group, from 1.

        """
        super().__init__(message)
        self.line = line
        """The line where the text stops being a saved group, from 1."""
        self.column = column
        """The column where the text stops being a saved group, from 1."""


# What the generated module reports, converted by its class: a table per module's failures, and one of the errors, whose keys a
# test checks against every class the generator wrote.

_ID_FAILURES: dict[generated.IdFailure, IdFailure] = {
    generated.IdFailure.EMPTY: IdError.Empty(),
    generated.IdFailure.TOO_LONG: IdError.TooLong(),
    generated.IdFailure.CONTROL_CHARACTER: IdError.ControlCharacter(),
    generated.IdFailure.SURROUNDING_WHITESPACE: IdError.SurroundingWhitespace(),
}

_IBAN_REASONS: dict[type[object], Callable[[Any], IbanReason]] = {
    generated.IbanReason.INVALID_CHARACTER: lambda _: PaymentAccountError.InvalidIban.InvalidCharacter(),
    generated.IbanReason.INVALID_FORMAT: lambda _: PaymentAccountError.InvalidIban.InvalidFormat(),
    generated.IbanReason.UNKNOWN_COUNTRY: lambda _: PaymentAccountError.InvalidIban.UnknownCountry(),
    generated.IbanReason.WRONG_LENGTH: lambda failure: PaymentAccountError.InvalidIban.WrongLength(expected=failure.expected, found=failure.found),
    generated.IbanReason.INVALID_STRUCTURE: lambda _: PaymentAccountError.InvalidIban.InvalidStructure(),
    generated.IbanReason.INVALID_CHECKSUM: lambda _: PaymentAccountError.InvalidIban.InvalidChecksum(),
}

_PAYMENT_ACCOUNT_FAILURES: dict[type[object], Callable[[Any], PaymentAccountFailure]] = {
    generated.PaymentAccountFailure.EMPTY_NAME: lambda _: PaymentAccountError.EmptyName(),
    generated.PaymentAccountFailure.INVALID_IBAN: lambda failure: PaymentAccountError.InvalidIban(
        reason=_IBAN_REASONS[type(failure.reason)](failure.reason)
    ),
    generated.PaymentAccountFailure.EMPTY_USERNAME: lambda _: PaymentAccountError.EmptyUsername(),
}

_CALENDAR_FAILURES: dict[generated.CalendarFailure, CalendarFailure] = {
    generated.CalendarFailure.INVALID_DATE: CalendarError.InvalidDate(),
    generated.CalendarFailure.INVALID_TIME: CalendarError.InvalidTime(),
}

_CATEGORY_FAILURES: dict[generated.CategoryFailure, CategoryFailure] = {
    generated.CategoryFailure.EMPTY_NAME: CategoryError.EmptyName(),
}

_CONTACT_FAILURES: dict[generated.ContactFailure, ContactFailure] = {
    generated.ContactFailure.INVALID_EMAIL: ContactError.InvalidEmail(),
    generated.ContactFailure.INVALID_PHONE_NUMBER: ContactError.InvalidPhoneNumber(),
}

_MONEY_FAILURES: dict[type[object], Callable[[Any], MoneyFailure]] = {
    generated.MoneyFailure.INVALID_AMOUNT: lambda _: MoneyError.InvalidAmount(),
    generated.MoneyFailure.TOO_MANY_DECIMALS: lambda failure: MoneyError.TooManyDecimals(
        currency=_currency_code.currency_of(failure.currency), allowed=failure.allowed
    ),
    generated.MoneyFailure.CURRENCY_MISMATCH: lambda failure: MoneyError.CurrencyMismatch(
        left=_currency_code.currency_of(failure.left), right=_currency_code.currency_of(failure.right)
    ),
    generated.MoneyFailure.UNKNOWN_CURRENCY: lambda _: MoneyError.UnknownCurrency(),
    generated.MoneyFailure.INVALID_EXCHANGE_RATE: lambda _: MoneyError.InvalidExchangeRate(),
    generated.MoneyFailure.INVALID_PARTS: lambda _: MoneyError.InvalidParts(),
    generated.MoneyFailure.INVALID_PERCENTAGE: lambda _: MoneyError.InvalidPercentage(),
    generated.MoneyFailure.INVALID_QUANTITY: lambda _: MoneyError.InvalidQuantity(),
    generated.MoneyFailure.NOT_FINITE: lambda _: MoneyError.NotFinite(),
    generated.MoneyFailure.OVERFLOW: lambda _: MoneyError.Overflow(),
}

_MEMBER_FAILURES: dict[generated.MemberFailure, MemberFailure] = {
    generated.MemberFailure.EMPTY_FIRST_NAME: MemberError.EmptyFirstName(),
}

_EXPENSE_FAILURES: dict[type[object], Callable[[Any], ExpenseFailure]] = {
    generated.ExpenseFailure.EMPTY_NAME: lambda _: ExpenseError.EmptyName(),
    generated.ExpenseFailure.NON_POSITIVE_AMOUNT: lambda _: ExpenseError.NonPositiveAmount(),
    generated.ExpenseFailure.CURRENCY_MISMATCH: lambda failure: ExpenseError.CurrencyMismatch(
        expected=_currency_code.currency_of(failure.expected), found=_currency_code.currency_of(failure.found)
    ),
    generated.ExpenseFailure.EMPTY_SPLIT: lambda _: ExpenseError.EmptySplit(),
    generated.ExpenseFailure.DUPLICATE_MEMBER: lambda failure: ExpenseError.DuplicateMember(member=MemberId(failure.member)),
    generated.ExpenseFailure.NON_POSITIVE_SHARE: lambda failure: ExpenseError.NonPositiveShare(member=MemberId(failure.member)),
    generated.ExpenseFailure.AMOUNTS_DO_NOT_ADD_UP: lambda failure: ExpenseError.AmountsDoNotAddUp(
        expected=_money.money_of(failure.expected), actual=_money.money_of(failure.actual)
    ),
    generated.ExpenseFailure.PERCENTAGES_DO_NOT_ADD_UP: lambda failure: ExpenseError.PercentagesDoNotAddUp(
        total_basis_points=failure.total_basis_points
    ),
    generated.ExpenseFailure.NO_ITEMS: lambda _: ExpenseError.NoItems(),
    generated.ExpenseFailure.EMPTY_ITEM_NAME: lambda _: ExpenseError.EmptyItemName(),
    generated.ExpenseFailure.ITEMS_DO_NOT_ADD_UP: lambda failure: ExpenseError.ItemsDoNotAddUp(
        expected=_money.money_of(failure.expected), actual=_money.money_of(failure.actual)
    ),
    generated.ExpenseFailure.NEGATIVE_DISCOUNT: lambda _: ExpenseError.NegativeDiscount(),
    generated.ExpenseFailure.DISCOUNT_ABOVE_HUNDRED_PERCENT: lambda _: ExpenseError.DiscountAboveHundredPercent(),
    generated.ExpenseFailure.DISCOUNT_ABOVE_PRICE: lambda _: ExpenseError.DiscountAbovePrice(),
    generated.ExpenseFailure.NEGATIVE_TIP: lambda _: ExpenseError.NegativeTip(),
    generated.ExpenseFailure.OVERFLOW: lambda _: ExpenseError.Overflow(),
}

_SETTLEMENT_FAILURES: dict[type[object], Callable[[Any], SettlementFailure]] = {
    generated.SettlementFailure.OUT_OF_MEMORY: lambda failure: SettlementError.OutOfMemory(balances=failure.balances),
}

_TRANSFER_FAILURES: dict[generated.TransferFailure, TransferFailure] = {
    generated.TransferFailure.SAME_MEMBER: TransferError.SameMember(),
    generated.TransferFailure.NON_POSITIVE_AMOUNT: TransferError.NonPositiveAmount(),
}

_EXPENSE_GROUP_FAILURES: dict[type[object], Callable[[Any], ExpenseGroupFailure]] = {
    generated.ExpenseGroupFailure.UNSUPPORTED_FORMAT: lambda failure: ExpenseGroupError.UnsupportedFormat(format=failure.format),
    generated.ExpenseGroupFailure.EMPTY_NAME: lambda _: ExpenseGroupError.EmptyName(),
    generated.ExpenseGroupFailure.MISSING_EXCHANGE_RATE: lambda failure: ExpenseGroupError.MissingExchangeRate(
        currency=_currency_code.currency_of(failure.currency)
    ),
    generated.ExpenseGroupFailure.UNEXPECTED_EXCHANGE_RATE: lambda _: ExpenseGroupError.UnexpectedExchangeRate(),
    generated.ExpenseGroupFailure.DUPLICATE_EXCHANGE_RATE: lambda failure: ExpenseGroupError.DuplicateExchangeRate(
        currency=_currency_code.currency_of(failure.currency)
    ),
    generated.ExpenseGroupFailure.DUPLICATE_MEMBER: lambda failure: ExpenseGroupError.DuplicateMember(member=MemberId(failure.member)),
    generated.ExpenseGroupFailure.UNKNOWN_MEMBER: lambda failure: ExpenseGroupError.UnknownMember(member=MemberId(failure.member)),
    generated.ExpenseGroupFailure.ARCHIVED_MEMBER: lambda failure: ExpenseGroupError.ArchivedMember(member=MemberId(failure.member)),
    generated.ExpenseGroupFailure.MEMBER_IN_USE: lambda failure: ExpenseGroupError.MemberInUse(
        member=MemberId(failure.member),
        expenses=tuple(ExpenseId(each) for each in failure.expenses),
        transfers=tuple(TransferId(each) for each in failure.transfers),
    ),
    generated.ExpenseGroupFailure.DUPLICATE_CATEGORY: lambda failure: ExpenseGroupError.DuplicateCategory(category=CategoryId(failure.category)),
    generated.ExpenseGroupFailure.DUPLICATE_CATEGORY_NAME: lambda failure: ExpenseGroupError.DuplicateCategoryName(
        existing=CategoryId(failure.existing)
    ),
    generated.ExpenseGroupFailure.UNKNOWN_CATEGORY: lambda failure: ExpenseGroupError.UnknownCategory(category=CategoryId(failure.category)),
    generated.ExpenseGroupFailure.BUILT_IN_CATEGORY: lambda _: ExpenseGroupError.BuiltInCategory(),
    generated.ExpenseGroupFailure.DUPLICATE_EXPENSE: lambda failure: ExpenseGroupError.DuplicateExpense(expense=ExpenseId(failure.expense)),
    generated.ExpenseGroupFailure.UNKNOWN_EXPENSE: lambda failure: ExpenseGroupError.UnknownExpense(expense=ExpenseId(failure.expense)),
    generated.ExpenseGroupFailure.DUPLICATE_TRANSFER: lambda failure: ExpenseGroupError.DuplicateTransfer(transfer=TransferId(failure.transfer)),
    generated.ExpenseGroupFailure.UNKNOWN_TRANSFER: lambda failure: ExpenseGroupError.UnknownTransfer(transfer=TransferId(failure.transfer)),
    generated.ExpenseGroupFailure.BUSY: lambda _: ExpenseGroupError.Busy(),
    generated.ExpenseGroupFailure.OVERFLOW: lambda _: ExpenseGroupError.Overflow(),
}

_LANGUAGE_FAILURES: dict[generated.LanguageFailure, LanguageFailure] = {
    generated.LanguageFailure.INVALID_LANGUAGE: LanguageError.InvalidLanguage(),
}

_ERRORS: dict[type[object], Callable[[Any], BaseException]] = {
    generated.EranosError.Id: lambda error: IdError(_ID_FAILURES[error.failure], error.detail),
    generated.EranosError.PaymentAccount: lambda error: PaymentAccountError(
        _PAYMENT_ACCOUNT_FAILURES[type(error.failure)](error.failure), error.detail
    ),
    generated.EranosError.Calendar: lambda error: CalendarError(_CALENDAR_FAILURES[error.failure], error.detail),
    generated.EranosError.Category: lambda error: CategoryError(_CATEGORY_FAILURES[error.failure], error.detail),
    generated.EranosError.Contact: lambda error: ContactError(_CONTACT_FAILURES[error.failure], error.detail),
    generated.EranosError.Money: lambda error: MoneyError(_MONEY_FAILURES[type(error.failure)](error.failure), error.detail),
    generated.EranosError.Member: lambda error: MemberError(_MEMBER_FAILURES[error.failure], error.detail),
    generated.EranosError.Expense: lambda error: ExpenseError(_EXPENSE_FAILURES[type(error.failure)](error.failure), error.detail),
    generated.EranosError.Settlement: lambda error: SettlementError(_SETTLEMENT_FAILURES[type(error.failure)](error.failure), error.detail),
    generated.EranosError.Transfer: lambda error: TransferError(_TRANSFER_FAILURES[error.failure], error.detail),
    generated.EranosError.ExpenseGroup: lambda error: ExpenseGroupError(_EXPENSE_GROUP_FAILURES[type(error.failure)](error.failure), error.detail),
    generated.EranosError.Language: lambda error: LanguageError(_LANGUAGE_FAILURES[error.failure], error.detail),
    generated.EranosError.InvalidJson: lambda error: InvalidJsonError(error.detail, line=error.line, column=error.column),
    generated.EranosError.Cancelled: lambda error: asyncio.CancelledError(error.detail),
    generated.EranosError.Released: lambda error: RuntimeError(error.detail),
}


@contextlib.contextmanager
def library_errors() -> Iterator[None]:
    """
    Raise what the library reports inside the block as this package's error.

    Yields:
        Nothing: the block runs inside it.

    """
    try:
        yield
    # ty reads the generated module's second definition of the name, the container its cases are made in, and not the
    # exception the module binds the name to afterwards.
    except generated.EranosError as error:  # ty: ignore[invalid-exception-caught]
        raise _ERRORS[type(error)](error) from None
