"""
Name the entities by the identifiers the library gives them, which the package documents where it exports them.

Each identifier is a hyphenated UUID, typed apart from other text and from the other entities' identifiers, so that one
entity's identifier cannot stand where another's is expected; it converts to its text and back, for storing it, and the
library checks the text when a call carries it.

"""

from typing import NewType


ExpenseGroupId = NewType('ExpenseGroupId', str)

MemberId = NewType('MemberId', str)

CategoryId = NewType('CategoryId', str)

ExpenseId = NewType('ExpenseId', str)

TransferId = NewType('TransferId', str)
