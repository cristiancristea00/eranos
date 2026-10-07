"""
Name the entities by the identifiers the library gives them, which the package documents where it exports them.

Each identifier is text, kept as it is given: at most 64 bytes of UTF-8, not empty, without control characters, and
without whitespace at either end. Two identifiers are equal when their bytes are, so case matters, and they order by their
bytes; one the library generates is a UUID v7 written as lowercase hyphenated text, so generated identifiers order as
they were created. Each is typed apart from other text and from the other entities' identifiers, so that one entity's
identifier cannot stand where another's is expected; it converts to its text and back, for storing it, and the library
checks the text when a call carries it.

"""

from typing import NewType


ExpenseGroupId = NewType('ExpenseGroupId', str)

MemberId = NewType('MemberId', str)

CategoryId = NewType('CategoryId', str)

ExpenseId = NewType('ExpenseId', str)

TransferId = NewType('TransferId', str)
