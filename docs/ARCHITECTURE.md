# Architecture

How eranos is built and why. [README.md](../README.md) says what it does,
[AGENTS.md](../AGENTS.md) holds the conventions, and
[TESTING.md](TESTING.md) shows how each rule here is checked.

## Contents

1. [Goals and constraints](#1-goals-and-constraints)
2. [Crate map](#2-crate-map)
3. [Data model](#3-data-model)
4. [Money and rounding](#4-money-and-rounding)
5. [Resolving an entry](#5-resolving-an-entry)
6. [The ledger and its caches](#6-the-ledger-and-its-caches)
7. [Settlement strategies](#7-settlement-strategies)
8. [Validation and errors](#8-validation-and-errors)
9. [The serialised form](#9-the-serialised-form)
10. [Account details](#10-account-details)
11. [Future bindings](#11-future-bindings)
12. [Key numbers](#12-key-numbers)

## 1. Goals and constraints

- **One core for every app.** iOS, Android, and the web share the rules, so
  a group computes the same totals and the same cents everywhere. The core
  is pure synchronous Rust: no I/O, no async runtime, no clock except for
  generating identifiers, and no global state except the lock that
  generation reads the clock under, so the identifiers of concurrent threads
  stay in creation order (or, with the `sequential-ids` feature for
  examples and tests, the counter that replaces the clock), and the
  phone-number metadata, which
  rlibphonenumber decodes once, on first use, into a table nothing writes
  to.
- **Exact money.** A split that loses or invents a cent is a bug users see.
  Every amount is an integer; every division is exact by construction.
- **Changes are cheap and atomic.** Adding, editing, or removing an entry
  costs time proportional to the entry, not the group, and a change that
  fails leaves nothing half-applied.
- **Deterministic everywhere.** The same entries give the same shares, the
  same totals, and the same settlement on every device and in any order of
  arrival; this is what future sync relies on.
- **Dependencies for the rules others maintain.** `icu_casemap` and
  `icu_normalizer` (Unicode caseless matching of category names),
  `icu_locale_core` (UTS #35 language tags), `icu_properties` (the Unicode
  categories of email characters), `iso4217-static` (the ISO 4217 list,
  pinned exactly), `jiff` (calendar dates, times of day, and their ISO 8601
  parser, without its clock or time zones), and `rlibphonenumber`
  (libphonenumber's numbering plans, pinned exactly); besides them
  `thiserror` (derive only), `uuid` (v7 identifiers), and optionally
  `serde`. Everything else, including IBAN validation and the email rules,
  is in the crates. The licences the apps' notices must list are Apache-2.0
  (iso4217-static, and rlibphonenumber with libphonenumber's metadata),
  Unicode-3.0 (ICU4X), and Unlicense or MIT (jiff), besides those of the
  crates they bring; `deny.toml` does not check licences.

## 2. Crate map

The library is thirteen crates. Callers depend on `eranos`, the facade,
which re-exports the types they use from twelve internal crates; each
internal crate owns one part of the domain, holds its own error type, and
depends only on crates below it. Every crate depends on `eranos-core`, and
the facade on all twelve; the graph draws an edge to `eranos-core` only
from the crates with no other dependency in the workspace, and of the
facade's edges only the one to the group.

```mermaid
flowchart TD
    FACADE["eranos<br/>re-exports; tests, examples, benchmarks"]
    GROUP["eranos-group<br/>Group, GroupBuilder, Language, SavedGroup"]
    EXPENSE["eranos-expense<br/>Expense, Split, Item, Tip<br/>(+ resolve_expense)"]
    TRANSFER["eranos-transfer<br/>Transfer (+ resolve_transfer)"]
    MEMBER["eranos-member<br/>Member"]
    CATEGORY["eranos-category<br/>Category"]
    SETTLEMENT["eranos-settlement<br/>SettlementStrategy, Settlement, Payment<br/>(+ Settlements)"]
    ACCOUNT["eranos-account<br/>Account, Iban, Username"]
    CONTACT["eranos-contact<br/>Email, PhoneNumber"]
    LEDGER["eranos-ledger<br/>Summary, Balance, ExpenseShares<br/>(+ Ledger, Delta)"]
    CALENDAR["eranos-calendar<br/>CalendarDate, TimeOfDay"]
    MONEY["eranos-money<br/>Money, Currency, Cost, rates, ratios<br/>(+ allocation)"]
    CORE["eranos-core<br/>identifiers, cooperative futures<br/>(+ text rules, pacer)"]
    FACADE --> GROUP
    GROUP --> EXPENSE
    GROUP --> TRANSFER
    GROUP --> MEMBER
    GROUP --> CATEGORY
    GROUP --> SETTLEMENT
    GROUP --> LEDGER
    GROUP --> MONEY
    EXPENSE --> LEDGER
    EXPENSE --> CALENDAR
    EXPENSE --> MONEY
    TRANSFER --> SETTLEMENT
    TRANSFER --> CALENDAR
    TRANSFER --> MONEY
    MEMBER --> ACCOUNT
    MEMBER --> CONTACT
    SETTLEMENT --> LEDGER
    SETTLEMENT --> MONEY
    LEDGER --> MONEY
    MONEY --> CORE
    CALENDAR --> CORE
    ACCOUNT --> CORE
    CONTACT --> CORE
    CATEGORY --> CORE
```

<details>
<summary>ASCII version (for renderers without Mermaid)</summary>

```text
 ┌──────────────────────────────────────────────────────────────────────────┐
 │ eranos             the facade: re-exports; tests, examples, benchmarks   │
 ├──────────────────────────────────────────────────────────────────────────┤
 │ eranos-group       Group, GroupBuilder, Language, SavedGroup             │
 ├──────────────────────────────────────────────────────────────────────────┤
 │ eranos-expense     Expense, Split, Item, Tip (+ resolve_expense)         │
 │ eranos-transfer    Transfer (+ resolve_transfer)                         │
 │ eranos-member      Member                                                │
 │ eranos-category    Category                                              │
 ├──────────────────────────────────────────────────────────────────────────┤
 │ eranos-settlement  SettlementStrategy, Settlement, Payment               │
 │                    (+ Settlements)                                       │
 │ eranos-account     Account, Iban, Username                               │
 │ eranos-contact     Email, PhoneNumber                                    │
 ├──────────────────────────────────────────────────────────────────────────┤
 │ eranos-ledger      Summary, Balance, ExpenseShares (+ Ledger, Delta)     │
 │ eranos-calendar    CalendarDate, TimeOfDay                               │
 ├──────────────────────────────────────────────────────────────────────────┤
 │ eranos-money       Money, Currency, Cost, rates, ratios (+ allocation)   │
 ├──────────────────────────────────────────────────────────────────────────┤
 │ eranos-core        identifiers and cooperative futures (+ text, pacer)   │
 └──────────────────────────────────────────────────────────────────────────┘
   A crate depends only on crates in the bands below its own; the table
   lists each crate's direct dependencies.
```

</details>

| Crate               | Holds                                                                                                                                                               | Depends on                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `eranos`            | the re-exports callers use; the integration tests, examples, and benchmarks                                                                                         | the twelve crates below                                                               |
| `eranos-group`      | `Group`, `GroupBuilder`, `Language`, `SavedGroup`, `GroupError`, `LanguageError`                                                                                    | category, core, expense, ledger, member, money, settlement, transfer; icu_locale_core |
| `eranos-expense`    | `Expense`, `ExpenseBuilder`, `ExpenseSplit`, `Split` and its shares, `Item`, `ItemBuilder`, `Tip`, `TipValue`, `TipDistribution`, `ExpenseError`; `resolve_expense` | calendar, core, ledger, money                                                         |
| `eranos-transfer`   | `Transfer`, `TransferBuilder`, `TransferError`; `resolve_transfer`                                                                                                  | calendar, core, money, settlement                                                     |
| `eranos-member`     | `Member`, `MemberBuilder`, `MemberError`                                                                                                                            | account, contact, core                                                                |
| `eranos-category`   | `Category`, `CategoryError`                                                                                                                                         | core; icu_casemap, icu_normalizer                                                     |
| `eranos-settlement` | `SettlementStrategy`, `Settlement`, `Payment`, `SettlementError`; `Settlements`, `settle`                                                                           | core, ledger, money                                                                   |
| `eranos-account`    | `Account`, `AccountDetails`, `AccountKind`, `Iban`, `Username`, `AccountError`, `IbanError`                                                                         | core                                                                                  |
| `eranos-contact`    | `Email`, `PhoneNumber`, `ContactError`                                                                                                                              | core; icu_properties, rlibphonenumber                                                 |
| `eranos-ledger`     | `Summary`, `MemberSummary`, `Balance`, `CategoryTotal`, `ExpenseShares`, `MemberShare`; `Ledger`, `Delta`, `MemberPair`, `Debt`, `ResolvedExpense`                  | core, money                                                                           |
| `eranos-calendar`   | `CalendarDate`, `TimeOfDay`, `CalendarError`                                                                                                                        | core; jiff                                                                            |
| `eranos-money`      | `Money`, `Currency`, `Cost`, `ExchangeRate`, `CurrencyRate`, `Percentage`, `Parts`, `MoneyError`; `allocation`                                                      | core; iso4217-static                                                                  |
| `eranos-core`       | the five identifiers, `IdError`, `Cooperative`, `block_on`; `text`, `serde_text`, `yield_now`, `Pacer`                                                              | uuid                                                                                  |

The facade's modules (`eranos::money`, `eranos::member`, …) and its root
re-export what the table lists before a semicolon, and nothing else: what
follows one is machinery the crates above use, free to change. Every
internal crate but `eranos-ledger`, which has no error of its own, depends
on `thiserror`, and every crate but `eranos-ledger` and `eranos-settlement`
has a `serde` feature, which the facade's enables.

## 3. Data model

A `Group` has an ID, a name (the only required text), a description, a
**required** currency (every total is in it), and a language tag for the
apps. It owns four maps keyed by the UUID v7 identifiers of their entries,
so iteration order is the order the entries were built in, on every
device:

| Map        | Values                                        | Notes                                                             |
| ---------- | --------------------------------------------- | ----------------------------------------------------------------- |
| members    | `Member` + archived flag                      | first name required; contacts and accounts optional               |
| categories | `Category`                                    | names unique by Unicode caseless match; Uncategorised is built in |
| expenses   | `Expense` + its shares and pair debts         | the resolution is kept for exact removal                          |
| transfers  | `Transfer` + its amount in the group currency | the same, for transfers                                           |

Entities are immutable values built by builders (`Member::builder`,
`Expense::builder`, …) and edited through `to_builder`; the group validates
every value it receives. A builder's constructor takes every required value:
an expense has four, for one payer or several and for a whole split or
items, and an item two, for an equal split or any other. Every member,
category, expense, and transfer carries its identifier, generated when it
is built and kept by `to_builder`, so an edit stays the same entry: `add_*`
refuses an identifier the group already holds, and `replace_*` finds the
entry to replace by it.

An **expense** has a name, a cost (an amount in its own currency, local or
converted into the group currency at a fixed rate), a date and an optional
time, an optional description, a category (Uncategorised unless another is
set), a tip (none unless set), the payers (a `Split` of the total), and how
the cost is shared: an `ExpenseSplit::Whole(Split)` or
`ExpenseSplit::Itemised(Vec<Item>)`. A split is `Equal`, `Percentages`
(basis points summing to exactly 10 000), `Parts` (hundredths, so 0.5 for a
child), or `Amounts` (summing exactly to what is divided). An **item** has a
unit price, a quantity, and its own split; the line totals must add up to
the amount. A **tip** is fixed or a percentage of the amount, added on top,
and shared `Proportional`ly (by consumption) or `Equal`ly. A **transfer** is
money one member paid another, with a cost like an expense's.

**Archived members** keep their history and balance and may send and
receive transfers, but cannot join a new expense; a replacement may keep an
archived member the original already had. A member referenced by any entry
cannot be removed.

## 4. Money and rounding

- `Money` is `i64` minor units plus a `Currency`. `Currency` covers every
  active ISO 4217 code with a minor unit (165 codes, from the SIX "List One"
  published 2026-01-01, which `iso4217-static` 0.2.4 is generated from),
  with the ISO number of decimal places: 0 for JPY, 2 for EUR, 3 for BHD, 4
  for CLF. Precious metals, drawing rights, and test codes have no minor
  unit and are not currencies here. `Currency` holds the crate's value and
  its minor unit, read once when a code is looked up, so no crate type is
  part of the API; it also gives each currency's ISO English name and
  numeric code.
- `ExchangeRate` is an exact positive decimal (`mantissa / 10^scale`, up to
  18 decimal places) meaning "one unit of the entry's currency is this many
  units of the group's". Conversion multiplies in checked `i128` and rounds
  **half away from zero**, the rule of EU Regulation 1103/97 art. 5.
- `Cost` is an entry's amount with its conversion: `Local(Money)` is in the
  group currency and converts at 1; `Converted { amount, rate }`, made by
  `Money::at`, is in another currency. A rate of 1 does not make a cost
  local, because a currency pegged at par (BMD to USD) converts at exactly
  1. Only the group knows its currency, so it checks the kind of cost when
  an entry is added: `MissingExchangeRate` for a local cost in another
  currency, `UnexpectedExchangeRate` for a converted one in its own.
- `Percentage` is basis points (33.33 % = 3 333) and `Parts` hundredths
  (1.5 = 150): both exact integers.
- **Allocation.** Every division is `allocate(total, weights, rotation)`:
  each part is `total × weight / Σ weights` rounded down, and the leftover
  units go one each to the largest remainders (the Hamilton method), so the
  parts add up exactly and each is its exact share rounded down or up.
  Weights that are all zero count as equal, so a division never loses
  money. Members are sorted by ID first, so listing order never matters.
- **Ties rotate.** Equal remainders, as in every even split, are ordered
  from a start position given by FNV-1a over the entry's ID and a pass
  number. The extra cent of 10.00 / 3 therefore lands on a different member
  from one expense to the next, while the same expense divides identically
  everywhere. There is no stored cursor and no randomness.
- **Overflow is an error.** Parsing, conversions, tips, and every total are
  checked; the ledger keeps values within ±`i64::MAX` so any value can be
  negated. A change that would leave the range is refused whole.

## 5. Resolving an entry

Every figure a user enters is in the entry's own currency and is validated
there (items add up to the amount, percentages to 100 %, amounts to what
they divide). The group then checks that the entry's cost suits the group
currency, a local cost in it and a converted one in any other, so
resolution fails only when a conversion leaves the range of an amount.
Resolution produces the entry's shares in the group currency:

```mermaid
flowchart TD
    BASE["1. Base shares, entry currency<br/>whole: one allocation<br/>itemised: one per line, summed per member"]
    TIP["2. Tip<br/>fixed, or amount × rate rounded half up;<br/>allocated by base share or equally"]
    PAYERS["3. Payers<br/>the payer split divides amount + tip"]
    CONVERT{"4. Entry currency<br/>= group currency?"}
    REALLOCATE["Convert the total once;<br/>re-allocate shares and payments<br/>from the converted total"]
    DEBTS["5. Pair debts<br/>each payer's amount over what<br/>participants have left to cover"]
    BASE --> TIP --> PAYERS --> CONVERT
    CONVERT -- no --> REALLOCATE --> DEBTS
    CONVERT -- yes --> DEBTS
```

<details>
<summary>ASCII version (for renderers without Mermaid)</summary>

```text
 1. Base shares (entry currency)   whole: one allocation
                                   itemised: one per line, summed per member
                 │
                 ▼
 2. Tip           fixed, or amount × rate (half up);
                  allocated by base share, or equally
                 │
                 ▼
 3. Payers        the payer split divides amount + tip
                 │
                 ▼
 4. Foreign?  ── yes ──► convert the total once; re-allocate shares
     │                   and payments from the converted total ──┐
     no                                                          │
     ▼                                                           │
 5. Pair debts  ◄────────────────────────────────────────────────┘
    each payer's amount over what participants have left to cover
```

</details>

Converting the total once and re-dividing it keeps both sides exact: the
converted shares and the converted payments each add up to the converted
total, which a per-share conversion would not guarantee.

The **pair debts** feed the direct settlement. Payers are taken in ID
order; each payer's amount is allocated over the participants in proportion
to what each still has to cover, which is then reduced. The last payer
receives exactly what is left, so every participant's debts add up to their
share and every payer's credits to what they paid, exactly; debts of a
member to themselves are dropped. For a single payer this is simply "every
other participant owes the payer their share".

## 6. The ledger and its caches

The `Ledger` holds the `Summary` (total spent; per member paid, consumed,
sent, received, and net; per category; per member per category) and the
netted debt of every pair of members, keyed by a `MemberPair` whose `from`
is the lower member.

A change becomes a `Delta` in exact 128-bit arithmetic: adding an entry
adds its resolution, removing subtracts it, replacing does both. Applying a
delta computes and range-checks every new value first and only then writes
them, removing rows that reach zero, so a summary compares equal to a
rebuilt one however it was reached. Net is kept as its own accumulator:
computing it from the other four could overflow when each of them fits.

| Change                                                   | Cost                             | Settlement caches |
| -------------------------------------------------------- | -------------------------------- | ----------------- |
| add, replace, remove an expense                          | O(k log m), k = members involved | cleared           |
| add, replace, remove a transfer                          | O(log m)                         | cleared           |
| add, replace, archive a member; add or rename a category | O(log m)                         | kept              |
| remove a category: its totals move to Uncategorised      | O(expenses + category totals)    | kept              |
| change the group currency                                | O(entries), full rebuild         | cleared           |
| read the summary                                         | O(1) or O(log m)                 | —                 |

Settlements are computed on the first request per strategy and cached in
`OnceLock`s, which keep `Group` `Send + Sync`; the cache belongs to the
settlement module and reads the balances from the summary, and every
balance-changing mutation clears it. A plan is kept once it is computed,
never while it is: no request waits for another's computation, and an
asynchronous request dropped before it completes keeps nothing.
`change_currency` re-resolves every entry against the new rates into a
scratch ledger and swaps it in only if everything succeeded: entries in any
other currency become converted costs at the supplied rate (including
entries in the old group currency, which were local), and entries in the new
currency become local. Its asynchronous form yields once per 1 024 entries
and swaps only after its last yield, so a call dropped at a yield changes
nothing.

## 7. Settlement strategies

Every strategy pays out exactly the net balances: each debtor pays what
they owe and each creditor receives what they are owed, so nobody pays more
than their debt. Payments are sorted by payer, then recipient. The caller
always names the strategy; there is no default and no automatic fallback.

| Strategy           | Algorithm                                                                                                                      | Cost                   | Payments                                      |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------ | ---------------------- | --------------------------------------------- |
| `MinimumTransfers` | pair exact opposites; subset dynamic program for the largest partition into zero-sum groups; settle each group in member order | O(2ⁿ·n) time, 2ⁿ bytes | the minimum: n − (number of groups)           |
| `Greedy`           | largest debtor pays largest creditor, ties to the lower ID                                                                     | O(n log n)             | at most n − 1                                 |
| `Direct`           | every netted pair debt becomes a payment                                                                                       | O(pairs)               | the most; each traces back to shared expenses |

Minimising the number of payments is NP-hard (Verhoeff, "Settling multiple
debts efficiently", 2004): n balances need n − g payments, where g is the
largest number of disjoint groups that each sum to zero. The solver:

1. **Pairs exact opposites** (x owed, x owing) first. Taking such a pair as
   its own group is always optimal: if an optimal partition put the two in
   groups G and H, then {x, −x} and (G ∪ H) ∖ {x, −x} is a partition with as
   many groups.
2. **Runs `table[S] = max over i in S of table[S ∖ {i}] + [ΣS = 0]`** over
   every subset S of the remaining n balances, one byte per subset. Walking
   back from the full set along the maximum orders the members so that the
   prefix sums hit zero exactly at the group boundaries.
3. **Settles each group** with the northwest-corner rule: debtors and
   creditors in member order, each payment the overlap of the current debt
   and credit. A group of a maximum partition has no zero-sum subgroup, so
   it takes exactly one payment fewer than its size.

There is no size limit. The table must fit in the address space (n ≤ 63 on
64-bit targets, about 30 on wasm32); beyond that, or when the allocator
refuses, the solver returns `SettlementError::OutOfMemory`. Operating systems
that overcommit memory (Linux) or end memory-hungry apps (iOS) may stop the
process before an allocation fails, which is why the documentation of
`MinimumTransfers` recommends `Greedy` for large groups.

## 8. Validation and errors

Validation happens in two stages. A builder's `build()` checks the entry on
its own (names, positive amounts, sums, currencies inside the entry, tips).
The group's `add_*`/`replace_*` check the entry against the
group (members exist and are not archived, the category exists, and the
cost is converted exactly when the entry is foreign) and against the
ledger's range.

| Error             | Returned by                                                                                                            |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `IdError`         | reading an identifier from text                                                                                        |
| `CalendarError`   | building and reading dates and times of day                                                                            |
| `MoneyError`      | building, reading, and combining amounts, currencies, rates, percentages, and parts                                    |
| `AccountError`    | `Account::new`, `Iban::parse` (with the `IbanError` reason), `Username::new`                                           |
| `ContactError`    | `Email::parse`, `PhoneNumber::parse`                                                                                   |
| `MemberError`     | `MemberBuilder::build`                                                                                                 |
| `CategoryError`   | `Category::new`, `Category::renamed`                                                                                   |
| `LanguageError`   | `Language::parse`                                                                                                      |
| `ExpenseError`    | `ExpenseBuilder::build`, `ItemBuilder::build`                                                                          |
| `TransferError`   | `TransferBuilder::build`                                                                                               |
| `GroupError`      | `GroupBuilder::build`, `SavedGroup::into_group`, and every group mutation; the group is unchanged when one is returned |
| `SettlementError` | `Group::settlement` (only `MinimumTransfers` can fail)                                                                 |

Each module has one error enum, `#[non_exhaustive]` and built with
`thiserror`, and none wraps another: a function reports the failure of its
own module, so a caller that mixes modules uses its own error type or
`Box<dyn Error>`. Messages are lowercase without a trailing full stop and
never repeat the rejected text, because inputs include account numbers and
contact details.

## 9. The serialised form

With the `serde` feature, `Group` and every type it stores serialise and
deserialise: its entries, members, and categories, the identifiers, and the
values inside them, such as `Money`, `Currency`, `ExchangeRate`,
`CalendarDate`, and `Language`. `SavedGroup` only deserialises. What a group
derives (`Summary`, `Settlement`, `ExpenseShares`, and the types they hold),
the builders, the errors, `CurrencyRate`, and `SettlementStrategy` do not
serialise. A group serialises only its inputs, never the derived totals:

```text
{
  "format": 2,
  "id": "…",
  "categories": [{"id": "…", "name": "Food"}],
  "currency": "EUR",
  "expenses":   [{"id": "…",
                  "amount": {"currency": "EUR", "minor_units": 9000},
                  "date": "2026-09-23", "name": "Dinner", "payers": {"equal": ["…"]},
                  "split": {"whole": {"percentages": [["…", 5000], …]}},
                  "tip": {"value": {"percentage": 1000}, "distribution": "proportional"}}],
  "members":    [{"id": "…", "first_name": "Ana", …}],
  "name": "Lisbon",
  "transfers":  [{"id": "…", "amount": …, "date": "…", "from": "…", "to": "…"}],
  "archived":   ["…"],
  "language": "pt-PT"
}
```

- Identifiers are hyphenated UUIDs, currencies their code, rates decimal
  strings, dates `YYYY-MM-DD` and times `hh:mm` (read back by the ISO 8601
  parser behind `CalendarDate::parse` and `TimeOfDay::parse`), percentages
  basis points, parts hundredths, and each share of a split the two-element
  array `[member, value]`. Absent optional fields are omitted, and so are the
  built-in category, an expense's category when it is the built-in one
  (every group has it again when it loads), and a tip equal to
  `Tip::default()`, no tip at all. Keys follow the field order of
  [AGENTS.md](../AGENTS.md): `format` and `id` first, then the required
  fields, then the optional ones, each alphabetically.
- Loading replays the entries through the `add_*` methods: members (all
  unarchived first), categories, expenses, transfers, then the `archived`
  list, because an expense may reference a member archived after it was
  added. A hand-edited or corrupted file is refused, never loaded
  inconsistently. Deserialising a `Group` reports the refusal as the
  deserialiser's error; deserialising a `SavedGroup` and calling
  `into_group` reports it as a `GroupError`, so an app can tell a newer
  file from a damaged one. An entry that could never have been built fails
  the read in both cases, since every entity deserialises through its
  validating constructor.
- `format` is checked: a newer one is refused rather than read with fields
  silently dropped (`GroupError::UnsupportedFormat`). New optional fields
  get `#[serde(default)]`, so files written by older releases keep
  loading.

## 10. Account details

IBANs and usernames are validated values, held, printed, and serialised in
clear text like every other value: `as_str()` returns an IBAN in electronic
format and a username as entered, trimmed. IBANs are validated against the
SWIFT IBAN Registry (release 103): the country code,
the country's length and BBAN structure (which positions hold digits and
which letters), and the ISO 7064 mod-97 check digits, which must lie
between 02 and 98 and catch every single-character typo.

rlibphonenumber writes national numbers to the `log` facade at `trace`
level while it parses them, and a library cannot turn a dependency's logging
off without global logger state; a binding or an app that installs a logger
keeps the `rlibphonenumber` target below `trace`, so members' numbers stay
out of its logs.

The serialised form contains the details in clear text. Encryption at rest
belongs to each platform's secure storage (Keychain, Keystore, Web Crypto),
which holds keys the library never sees.

## 11. Future bindings

The core is shaped for binding crates added as workspace siblings:

- **`eranos-ffi` (UniFFI for Swift and Kotlin)**, following matrix-rust-sdk:
  a `#[derive(uniffi::Object)]` wrapper holding `Mutex<eranos::Group>`
  (UniFFI objects cannot take `&mut self`), mirror records with `From`
  conversions, one flat error enum, and identifiers as strings (UniFFI has no
  128-bit integers). The release profile keeps `panic = "unwind"` so the
  scaffolding can catch panics at the boundary.
- **`eranos-wasm` (browser)**, through wasm-bindgen or
  uniffi-bindgen-react-native. It must enable `uuid`'s `js` feature: UUID v7
  generation reads the clock and the random source, which
  `wasm32-unknown-unknown` provides only through JavaScript.
- Public types have no lifetimes or generic parameters on their data, IDs
  are `Copy`, and every public type is `Send + Sync`, which `tests/api.rs`
  asserts.

## 12. Key numbers

Release build on an Apple M4 Pro (`cargo bench -p eranos --features serde --bench '*'`):

| Operation                               | Size                                            | Time                              |
| --------------------------------------- | ----------------------------------------------- | --------------------------------- |
| `MinimumTransfers`                      | 12 / 16 / 20 / 24 open balances                 | 18 µs / 0.36 ms / 7.2 ms / 0.14 s |
| `Greedy`                                | 100 / 1 000 members                             | 3.5 µs / 55 µs                    |
| `Direct`                                | 100 members sharing every expense (4 950 pairs) | 110 µs                            |
| add or remove an expense of six members | group of 10 000 expenses                        | ≈ 1 µs                            |
| read a member's totals                  | any                                             | ≈ 3 ns                            |
| change the currency                     | group of 10 000 expenses                        | 9.8 ms                            |
| load a saved group from JSON            | group of 10 000 expenses                        | 20 ms                             |
