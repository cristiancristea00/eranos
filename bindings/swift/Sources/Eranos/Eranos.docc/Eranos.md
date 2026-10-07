# ``Eranos``

Track what a group spends, divide every expense exactly, and settle up with the fewest payments.

## Overview

The package is the eranos library, written in Rust, for Swift. Every amount is a whole number of
minor units, every division adds up to the cent, and every text the package shows is the
library's. An ``ExpenseGroup`` holds its members, categories, expenses, and transfers; its ``Summary``
has the totals, and a ``SettlementPlan`` has the payments that clear every balance. Every call that
can fail throws ``EranosError``, grouped by the module of the library that refused it.

Every call of a group comes in a synchronous and an asynchronous form under one name. The
asynchronous form waits for the group's lock without blocking a thread, and cancelling its task
cancels the call, which leaves the group as it was.

## Topics

### The group

- ``ExpenseGroup``
- ``ExpenseGroupReader``

### Entries

- ``Expense``
- ``Member``
- ``Category``
- ``Transfer``
- ``Identifier``

### How an expense divides

- ``Cost``
- ``Split``
- ``ExpenseSplit``
- ``ExpenseItem``
- ``TipRule``

### Totals and settlements

- ``Summary``
- ``Balance``
- ``ExpenseShares``
- ``SettlementPlan``
- ``SettlementStrategy``

### Values

- ``CurrencyCode``
- ``Money``
- ``ExchangeRate``
- ``Percentage``
- ``Parts``
- ``CalendarDate``
- ``TimeOfDay``
- ``Email``
- ``PhoneNumber``
- ``IBAN``
- ``Username``
- ``Language``

### Errors

- ``EranosError``
