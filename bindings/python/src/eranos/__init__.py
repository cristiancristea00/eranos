"""
Split and settle shared expenses: the eranos library for Python.

A group holds its members, categories, expenses, and transfers, and adds them up into balances and the payments that
settle them. Every amount is a whole number of minor units, and every division adds up exactly::

    from eranos import CalendarDate, CurrencyCode, EqualSplit, Expense, ExpenseGroup, LocalCost, Member, Money, WholeExpenseSplit

    euro = CurrencyCode('EUR')
    group = ExpenseGroup.create('Flat', euro)
    ana = group.add_member(Member.create(first_name='Ana'))
    bob = group.add_member(Member.create(first_name='Bob'))
    group.add_expense(
        Expense.create(
            name='Dinner',
            cost=LocalCost(Money(3000, euro)),
            payers=EqualSplit((ana,)),
            split=WholeExpenseSplit(EqualSplit((ana, bob))),
            date=CalendarDate(2026, 9, 23),
        )
    )

Every entry and every group has its identifier from the moment it is made: ``create`` generates it, and the class, or
``ExpenseGroup.create_with_id``, takes the app's own. Every call of a group comes in two forms: the one named for the call
blocks while another call holds the group's lock, and its twin, whose name ends in ``_async``, is a coroutine. What the
library refuses raises an ``EranosError``.

"""

from eranos._calendar import CalendarDate, TimeOfDay
from eranos._contact import Email, Iban, Language, PhoneNumber, Username
from eranos._currency_code import CurrencyCode
from eranos import _entries
from eranos._entries import (
    PaymentAccount,
    PaymentAccountDetails,
    AmountsSplit,
    Category,
    ConvertedCost,
    Cost,
    EqualSplit,
    Expense,
    ExpenseSplit,
    FixedTip,
    IbanPaymentAccountDetails,
    ExpenseItem,
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
    TipRule,
    TipDistribution,
    TipValue,
    Transfer,
    UsernamePaymentAccountDetails,
    WholeExpenseSplit,
)
from eranos._errors import (
    PaymentAccountError,
    PaymentAccountFailure,
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
from eranos._identifiers import (
    generate_category_id,
    generate_expense_group_id,
    generate_expense_id,
    generate_member_id,
    generate_transfer_id,
    parse_category_id,
    parse_expense_group_id,
    parse_expense_id,
    parse_member_id,
    parse_transfer_id,
)
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
    SettlementPlan,
    SettledBalance,
    SettlementStrategy,
    Summary,
)


# ruff: disable[non-empty-init-module]
# Autodoc documents a name only where the module it documents gives the name's docstring.
ExpenseGroupId = _identifiers.ExpenseGroupId
"""The identifier of a group: the text the app gives it, or one ``generate_expense_group_id`` returns, a UUID v7 in lowercase hyphenated form."""

MemberId = _identifiers.MemberId
"""The identifier of a member: the text the app gives them, or one ``generate_member_id`` returns, a UUID v7 in lowercase hyphenated form."""

CategoryId = _identifiers.CategoryId
"""
The identifier of a category: the text the app gives it, or one ``generate_category_id`` returns, a UUID v7 in lowercase hyphenated form.

The built-in category's, ``UNCATEGORISED``, is the nil UUID in that form, and the group lists that category before every other,
whatever their identifiers.
"""

UNCATEGORISED = _identifiers.UNCATEGORISED
"""
The identifier of the built-in category, Uncategorised, which every group has and every expense is in until it is given another.

The library fixes it: the nil UUID in lowercase hyphenated form, ``00000000-0000-0000-0000-000000000000``.
"""

ExpenseId = _identifiers.ExpenseId
"""The identifier of an expense: the text the app gives it, or one ``generate_expense_id`` returns, a UUID v7 in lowercase hyphenated form."""

TransferId = _identifiers.TransferId
"""The identifier of a transfer: the text the app gives it, or one ``generate_transfer_id`` returns, a UUID v7 in lowercase hyphenated form."""

NO_TIP = _entries.NO_TIP
"""
The no-tip rule, which an expense without a tip has.

It is 0 % of the expense's amount, shared in proportion to each participant's share. An expense's tip starts at it; a 0 % tip
shared equally is another rule, which the expense keeps.
"""
# ruff: enable[non-empty-init-module]


__all__ = [
    'NO_TIP',
    'UNCATEGORISED',
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
    'CurrencyCode',
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
    'ExpenseItem',
    'ExpenseShares',
    'ExpenseSplit',
    'FixedTip',
    'GetsBackBalance',
    'Iban',
    'IbanPaymentAccountDetails',
    'IbanReason',
    'IdError',
    'IdFailure',
    'InvalidJsonError',
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
    'PaymentAccount',
    'PaymentAccountDetails',
    'PaymentAccountError',
    'PaymentAccountFailure',
    'Percentage',
    'PercentageTip',
    'PercentagesSplit',
    'PhoneNumber',
    'SettledBalance',
    'SettlementError',
    'SettlementFailure',
    'SettlementPlan',
    'SettlementStrategy',
    'Split',
    'Summary',
    'TimeOfDay',
    'TipDistribution',
    'TipRule',
    'TipValue',
    'Transfer',
    'TransferError',
    'TransferFailure',
    'TransferId',
    'Username',
    'UsernamePaymentAccountDetails',
    'WholeExpenseSplit',
    'generate_category_id',
    'generate_expense_group_id',
    'generate_expense_id',
    'generate_member_id',
    'generate_transfer_id',
    'parse_category_id',
    'parse_expense_group_id',
    'parse_expense_id',
    'parse_member_id',
    'parse_transfer_id',
]
