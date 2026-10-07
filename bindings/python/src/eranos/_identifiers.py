"""
Name the entities by their identifiers, which an app gives them or the library generates, check an identifier's text, and generate one.

Each identifier is text, kept as it is given: at most 64 bytes of UTF-8, not empty, without control characters, and
without whitespace at either end. Two identifiers are equal when their bytes are, so case matters, and they order by their
bytes; one the library generates is a UUID v7 written as lowercase hyphenated text, so generated identifiers order as
they were created. Each is typed apart from other text and from the other entities' identifiers, so that one entity's
identifier cannot stand where another's is expected; it converts to its text and back, for storing it, and the library
checks the text when a call carries it. ``parse_member_id`` and its siblings check text by the same rules before an app
stores it, and ``generate_member_id`` and its siblings generate one, which every kind draws from the library's one
generator. The package documents each type where it exports it.

"""

from typing import NewType

# The module of errors, not its function: that module imports the types here before it defines its function, which
# this module reads only when a check runs.
from eranos import _errors
from eranos._internal import eranos as generated


ExpenseGroupId = NewType('ExpenseGroupId', str)

MemberId = NewType('MemberId', str)

CategoryId = NewType('CategoryId', str)

ExpenseId = NewType('ExpenseId', str)

TransferId = NewType('TransferId', str)


def _parsed(text: str) -> str:
    """
    Check the identifier written text by the rules every kind of identifier shares.

    Args:
        text: The identifier's text.

    Returns:
        The text, as it is.

    """
    with _errors.library_errors():
        return generated.identifier_parse(text)


def parse_expense_group_id(text: str) -> ExpenseGroupId:
    """
    Check the group identifier written text, as it is: not trimmed, and in its case.

    The library refuses text that breaks a rule with an ``IdError`` that names the first one: ``IdError.Empty``,
    ``IdError.TooLong`` for more than 64 bytes of UTF-8, ``IdError.ControlCharacter``, and ``IdError.SurroundingWhitespace``.

    Args:
        text: The identifier's text: ``"flat-12"``.

    Returns:
        The identifier.

    """
    return ExpenseGroupId(_parsed(text))


def generate_expense_group_id() -> ExpenseGroupId:
    """
    Generate a new group identifier, a UUID v7 as lowercase hyphenated text, which orders after every identifier generated before it.

    Every kind of identifier draws from the library's one generator. ``ExpenseGroup.create`` generates the group's through it, and
    ``ExpenseGroup.create_with_id`` takes one instead.

    Returns:
        The identifier.

    """
    return ExpenseGroupId(generated.identifier_generate())


def parse_member_id(text: str) -> MemberId:
    """
    Check the member identifier written text, as it is: not trimmed, and in its case.

    The library refuses text that breaks a rule with an ``IdError`` that names the first one: ``IdError.Empty``,
    ``IdError.TooLong`` for more than 64 bytes of UTF-8, ``IdError.ControlCharacter``, and ``IdError.SurroundingWhitespace``.

    Args:
        text: The identifier's text: ``"ana"``.

    Returns:
        The identifier.

    """
    return MemberId(_parsed(text))


def generate_member_id() -> MemberId:
    """
    Generate a new member identifier, a UUID v7 as lowercase hyphenated text, which orders after every identifier generated before it.

    Every kind of identifier draws from the library's one generator. ``Member.create`` generates the member's through it, and the class
    itself takes one instead.

    Returns:
        The identifier.

    """
    return MemberId(generated.identifier_generate())


def parse_category_id(text: str) -> CategoryId:
    """
    Check the category identifier written text, as it is: not trimmed, and in its case.

    The library refuses text that breaks a rule with an ``IdError`` that names the first one: ``IdError.Empty``,
    ``IdError.TooLong`` for more than 64 bytes of UTF-8, ``IdError.ControlCharacter``, and ``IdError.SurroundingWhitespace``.

    Args:
        text: The identifier's text: ``"lodging"``.

    Returns:
        The identifier.

    """
    return CategoryId(_parsed(text))


def generate_category_id() -> CategoryId:
    """
    Generate a new category identifier, a UUID v7 as lowercase hyphenated text, which orders after every identifier generated before it.

    Every kind of identifier draws from the library's one generator. ``Category.create`` generates the category's through it, and the class
    itself takes one instead.

    Returns:
        The identifier.

    """
    return CategoryId(generated.identifier_generate())


def parse_expense_id(text: str) -> ExpenseId:
    """
    Check the expense identifier written text, as it is: not trimmed, and in its case.

    The library refuses text that breaks a rule with an ``IdError`` that names the first one: ``IdError.Empty``,
    ``IdError.TooLong`` for more than 64 bytes of UTF-8, ``IdError.ControlCharacter``, and ``IdError.SurroundingWhitespace``.

    Args:
        text: The identifier's text: ``"receipt-0042"``.

    Returns:
        The identifier.

    """
    return ExpenseId(_parsed(text))


def generate_expense_id() -> ExpenseId:
    """
    Generate a new expense identifier, a UUID v7 as lowercase hyphenated text, which orders after every identifier generated before it.

    Every kind of identifier draws from the library's one generator. ``Expense.create`` generates the expense's through it, and the class
    itself takes one instead.

    Returns:
        The identifier.

    """
    return ExpenseId(generated.identifier_generate())


def parse_transfer_id(text: str) -> TransferId:
    """
    Check the transfer identifier written text, as it is: not trimmed, and in its case.

    The library refuses text that breaks a rule with an ``IdError`` that names the first one: ``IdError.Empty``,
    ``IdError.TooLong`` for more than 64 bytes of UTF-8, ``IdError.ControlCharacter``, and ``IdError.SurroundingWhitespace``.

    Args:
        text: The identifier's text: ``"payment-7"``.

    Returns:
        The identifier.

    """
    return TransferId(_parsed(text))


def generate_transfer_id() -> TransferId:
    """
    Generate a new transfer identifier, a UUID v7 as lowercase hyphenated text, which orders after every identifier generated before it.

    Every kind of identifier draws from the library's one generator. ``Transfer.create`` generates the transfer's through it, and the class
    itself takes one instead.

    Returns:
        The identifier.

    """
    return TransferId(generated.identifier_generate())
