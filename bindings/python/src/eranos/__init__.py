"""
Split and settle shared expenses: the eranos library for Python.

A group holds its members, categories, expenses, and transfers, and adds them up into balances and the payments that
settle them. Every amount is a whole number of minor units, and every division adds up exactly::

    from eranos import CalendarDate, Currency, EqualSplit, Expense, ExpenseGroup, LocalCost, Member, Money, WholeExpenseSplit

    euro = Currency('EUR')
    group = ExpenseGroup.create('Flat', euro)
    ana = group.add_member(Member(first_name='Ana'))
    bob = group.add_member(Member(first_name='Bob'))
    group.add_expense(
        Expense(
            name='Dinner',
            cost=LocalCost(Money(3000, euro)),
            payers=EqualSplit((ana,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob))),
            date=CalendarDate(2026, 9, 23),
        )
    )

Every call of a group comes in two forms: the one named for the call blocks while another call holds the group's lock,
and its twin, whose name ends in ``_async``, is a coroutine. What the library refuses raises an ``EranosError``.

"""

from eranos._calendar import CalendarDate, TimeOfDay
from eranos._contact import Email, Iban, Language, PhoneNumber, Username
from eranos._currency import Currency
from eranos._entries import (
    Account,
    AccountDetails,
    AmountsSplit,
    Category,
    ConvertedCost,
    Cost,
    EqualSplit,
    Expense,
    ExpenseSplit,
    FixedTip,
    IbanAccountDetails,
    Item,
    ItemisedExpenseSplit,
    LocalCost,
    Member,
    MemberAmount,
    MemberParts,
    MemberPercentage,
    PartsSplit,
    PercentagesSplit,
    PercentageTip,
    Split,
    Tip,
    TipDistribution,
    TipValue,
    Transfer,
    UsernameAccountDetails,
    WholeExpenseSplit,
)
from eranos._errors import (
    AccountError,
    AccountFailure,
    CalendarError,
    CalendarFailure,
    CategoryError,
    CategoryFailure,
    ContactError,
    ContactFailure,
    EranosError,
    ExpenseError,
    ExpenseFailure,
    ExpenseGroupError,
    ExpenseGroupFailure,
    IbanReason,
    IdError,
    IdFailure,
    InvalidJsonError,
    LanguageError,
    LanguageFailure,
    MemberError,
    MemberFailure,
    MoneyError,
    MoneyFailure,
    SettlementError,
    SettlementFailure,
    TransferError,
    TransferFailure,
)
from eranos._expense_group import CurrencyRate, ExpenseGroup
from eranos._expense_group_reader import ExpenseGroupReader
from eranos import _identifiers
from eranos._money import Money
from eranos._ratios import ExchangeRate, Parts, Percentage
from eranos._totals import (
    Balance,
    CategoryTotal,
    ExpenseShares,
    GetsBackBalance,
    MemberCategoryTotals,
    MemberShare,
    MemberSummary,
    OwesBalance,
    Payment,
    Settlement,
    SettledBalance,
    SettlementStrategy,
    Summary,
)


# ruff: disable[non-empty-init-module]
# Autodoc documents an identifier only where the module it documents gives the identifier's docstring.
ExpenseGroupId = _identifiers.ExpenseGroupId
"""The identifier of a group: a hyphenated UUID the library gave it."""

MemberId = _identifiers.MemberId
"""The identifier of a member: a hyphenated UUID the group gave them."""

CategoryId = _identifiers.CategoryId
"""The identifier of a category: a hyphenated UUID the group gave it."""

ExpenseId = _identifiers.ExpenseId
"""The identifier of an expense: a hyphenated UUID the group gave it."""

TransferId = _identifiers.TransferId
"""The identifier of a transfer: a hyphenated UUID the group gave it."""
# ruff: enable[non-empty-init-module]


__all__ = [
    'Account',
    'AccountDetails',
    'AccountError',
    'AccountFailure',
    'AmountsSplit',
    'Balance',
    'CalendarDate',
    'CalendarError',
    'CalendarFailure',
    'Category',
    'CategoryError',
    'CategoryFailure',
    'CategoryId',
    'CategoryTotal',
    'ContactError',
    'ContactFailure',
    'ConvertedCost',
    'Cost',
    'Currency',
    'CurrencyRate',
    'Email',
    'EqualSplit',
    'EranosError',
    'ExchangeRate',
    'Expense',
    'ExpenseError',
    'ExpenseFailure',
    'ExpenseGroup',
    'ExpenseGroupError',
    'ExpenseGroupFailure',
    'ExpenseGroupId',
    'ExpenseGroupReader',
    'ExpenseId',
    'ExpenseShares',
    'ExpenseSplit',
    'FixedTip',
    'GetsBackBalance',
    'Iban',
    'IbanAccountDetails',
    'IbanReason',
    'IdError',
    'IdFailure',
    'InvalidJsonError',
    'Item',
    'ItemisedExpenseSplit',
    'Language',
    'LanguageError',
    'LanguageFailure',
    'LocalCost',
    'Member',
    'MemberAmount',
    'MemberCategoryTotals',
    'MemberError',
    'MemberFailure',
    'MemberId',
    'MemberParts',
    'MemberPercentage',
    'MemberShare',
    'MemberSummary',
    'Money',
    'MoneyError',
    'MoneyFailure',
    'OwesBalance',
    'Parts',
    'PartsSplit',
    'Payment',
    'Percentage',
    'PercentageTip',
    'PercentagesSplit',
    'PhoneNumber',
    'SettledBalance',
    'Settlement',
    'SettlementError',
    'SettlementFailure',
    'SettlementStrategy',
    'Split',
    'Summary',
    'TimeOfDay',
    'Tip',
    'TipDistribution',
    'TipValue',
    'Transfer',
    'TransferError',
    'TransferFailure',
    'TransferId',
    'Username',
    'UsernameAccountDetails',
    'WholeExpenseSplit',
]
