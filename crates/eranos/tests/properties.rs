//! Properties of random groups through the public API: every split method,
//! itemised bills, both tip distributions, several payers, foreign
//! currencies, and transfers. Whatever the entries, the books must balance
//! (paid, consumed, and spent agree; balances sum to zero; categories add up),
//! every strategy's plan must clear every balance, the fewest-payments plan
//! must beat or match greedy, a plan awaited in an executor must equal the
//! one computed on the caller's thread, and a group must not depend on the
//! order its entries arrived in (two devices syncing the same entries must
//! agree). Entries draw identifiers an app gives as well as generated ones.
#![forbid(unsafe_code)]
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a failing test helper panics to report the failure, as clippy.toml allows inside test functions"
)]

use eranos::cooperative::block_on;
use eranos::{
    CalendarDate, CategoryTotal, Cost, CurrencyCode, CurrencyRate, ExchangeRate, Expense,
    ExpenseGroup, ExpenseId, ExpenseItem, Member, MemberAmount, MemberId, MemberParts,
    MemberPercentage, Money, Parts, Percentage, SettlementStrategy, Split, TipDistribution,
    TipRule, Transfer, TransferId, error::ExpenseGroupError,
};
use proptest::prelude::*;

const MEMBERS: usize = 6;

/// The starts of the identifiers an app might give an entry, outside the
/// shape of a generated one: before the nil UUID text, after every
/// generated text, and beyond ASCII. Each entry adds its position, so no
/// two of one kind share an identifier.
const GIVEN: [&str; 4] = ["-food", "!a", "zz", "Ωmega · "];

/// The identifier an app gives the entry at `position`, starting with the
/// text `GIVEN` holds at `start`.
fn given<Id: std::str::FromStr<Err = eranos::error::IdError>>(start: usize, position: usize) -> Id {
    format!("{}{position}", GIVEN[start]).parse().unwrap()
}

fn day() -> CalendarDate {
    CalendarDate::new(2026, 9, 23).unwrap()
}

/// A member of a split, by its index among the group's members, and the
/// weight it takes.
#[derive(Debug, Clone, Copy)]
struct MemberRecipe {
    index: usize,
    weight: u32,
}

/// How one amount is divided: a method and weighted members.
#[derive(Debug, Clone)]
struct SplitRecipe {
    method: u8,
    members: Vec<MemberRecipe>,
}

fn split_recipe() -> impl Strategy<Value = SplitRecipe> {
    (
        0_u8..4,
        proptest::collection::vec(
            (0..MEMBERS, 1_u32..1_000).prop_map(|(index, weight)| MemberRecipe { index, weight }),
            1..5,
        ),
    )
        .prop_map(|(method, members)| SplitRecipe { method, members })
}

/// A member of a split and the weight it takes.
#[derive(Debug, Clone, Copy)]
struct WeightedMember {
    member: MemberId,
    weight: u32,
}

/// Divides `total` minor units by `weights` into positive integers adding
/// up to `total`; `None` when some part would be zero.
fn positive_parts(total: i64, weights: &[u32]) -> Option<Vec<i64>> {
    let sum: i64 = weights.iter().map(|&weight| i64::from(weight)).sum();
    let mut parts: Vec<i64> = weights
        .iter()
        .map(|&weight| total * i64::from(weight) / sum)
        .collect();
    let leftover = total - parts.iter().sum::<i64>();
    for part in parts.iter_mut().take(usize::try_from(leftover).ok()?) {
        *part += 1;
    }
    parts.iter().all(|&part| 0 < part).then_some(parts)
}

impl SplitRecipe {
    /// The split for dividing `total`, falling back to an equal split when
    /// the drawn method cannot give everyone a positive share.
    fn build(&self, members: &[MemberId], total: Money) -> Split {
        let mut chosen: Vec<WeightedMember> = Vec::new();
        for recipe in &self.members {
            let member = members[recipe.index];
            if chosen.iter().all(|weighted| weighted.member != member) {
                chosen.push(WeightedMember {
                    member,
                    weight: recipe.weight,
                });
            }
        }
        let weights: Vec<u32> = chosen.iter().map(|weighted| weighted.weight).collect();
        let equal = Split::equal(chosen.iter().map(|weighted| weighted.member));
        match self.method {
            0 => equal,
            1 => Split::parts(chosen.iter().map(|weighted| {
                MemberParts::new(
                    weighted.member,
                    Parts::from_hundredths(weighted.weight).unwrap(),
                )
            })),
            2 => positive_parts(10_000, &weights).map_or(equal, |points| {
                Split::percentages(chosen.iter().zip(points).map(|(weighted, points)| {
                    MemberPercentage::new(
                        weighted.member,
                        Percentage::from_basis_points(u32::try_from(points).unwrap()),
                    )
                }))
            }),
            _ => positive_parts(total.minor_units(), &weights).map_or(equal, |amounts| {
                Split::amounts(chosen.iter().zip(amounts).map(|(weighted, amount)| {
                    MemberAmount::new(
                        weighted.member,
                        Money::from_minor_units(amount, total.currency()),
                    )
                }))
            }),
        }
    }
}

/// Another currency than the group's, and the rate into it, as a decimal
/// mantissa and scale.
#[derive(Debug, Clone, Copy)]
struct ForeignRecipe {
    currency: CurrencyCode,
    mantissa: u64,
    scale: u8,
}

impl ForeignRecipe {
    fn rate(self) -> ExchangeRate {
        ExchangeRate::from_decimal(self.mantissa, self.scale).unwrap()
    }
}

/// A line of an itemised bill: its unit price in minor units, its
/// quantity, and how it is divided.
#[derive(Debug, Clone)]
struct ExpenseItemRecipe {
    price: i64,
    split: SplitRecipe,
    quantity: u32,
}

/// A tip: fixed in minor units or a percentage in basis points, and shared
/// equally or in proportion.
#[derive(Debug, Clone, Copy)]
struct TipRuleRecipe {
    fixed: bool,
    value: u32,
    equal: bool,
}

#[derive(Debug, Clone)]
struct ExpenseRecipe {
    /// Where in `GIVEN` the identifier an app gives it starts, or `None`
    /// for a generated one.
    id: Option<usize>,
    amount: i64,
    payers: SplitRecipe,
    split: SplitRecipe,
    items: Option<Vec<ExpenseItemRecipe>>,
    tip: Option<TipRuleRecipe>,
    foreign: Option<ForeignRecipe>,
}

fn expense_recipe() -> impl Strategy<Value = ExpenseRecipe> {
    (
        proptest::option::of(0..GIVEN.len()),
        1_i64..500_000,
        proptest::option::weighted(
            0.3,
            proptest::collection::vec(
                (1_i64..20_000, 1_u32..4, split_recipe()).prop_map(|(price, quantity, split)| {
                    ExpenseItemRecipe {
                        price,
                        split,
                        quantity,
                    }
                }),
                1..4,
            ),
        ),
        split_recipe(),
        split_recipe(),
        proptest::option::of((any::<bool>(), 0_u32..3_000, any::<bool>()).prop_map(
            |(fixed, value, equal)| TipRuleRecipe {
                fixed,
                value,
                equal,
            },
        )),
        proptest::option::weighted(
            0.3,
            (0_u8..3, 1_u64..100_000, 0_u8..5).prop_map(|(currency, mantissa, scale)| {
                ForeignRecipe {
                    currency: [CurrencyCode::JPY, CurrencyCode::BHD, CurrencyCode::RON]
                        [usize::from(currency)],
                    mantissa,
                    scale,
                }
            }),
        ),
    )
        .prop_map(
            |(id, amount, items, split, payers, tip, foreign)| ExpenseRecipe {
                id,
                amount,
                payers,
                split,
                items,
                tip,
                foreign,
            },
        )
}

impl ExpenseRecipe {
    /// The expense at `position` among the recipe's, among `members`.
    fn build(&self, position: usize, members: &[MemberId]) -> Expense {
        let currency = self
            .foreign
            .map_or(CurrencyCode::EUR, |foreign| foreign.currency);
        let rate = self.foreign.map(ForeignRecipe::rate);
        let money = |minor_units| Money::from_minor_units(minor_units, currency);
        let cost = |amount: Money| rate.map_or_else(|| Cost::from(amount), |rate| amount.at(rate));
        let whole = || {
            let amount = money(self.amount);
            let split = self.split.build(members, amount);
            Expense::builder("Whole", cost(amount), members[0], split, day())
        };
        let itemised = |items: &Vec<ExpenseItemRecipe>| {
            let items: Vec<ExpenseItem> = items
                .iter()
                .map(|item| {
                    let line = money(item.price * i64::from(item.quantity));
                    ExpenseItem::builder_with_split(
                        "Item",
                        money(item.price),
                        item.split.build(members, line),
                    )
                    .quantity(item.quantity)
                    .build()
                    .unwrap()
                })
                .collect();
            let amount: i64 = items
                .iter()
                .map(|item| item.line_total().minor_units())
                .sum();
            Expense::itemised_builder("Itemised", cost(money(amount)), members[0], items, day())
        };
        let mut builder = self
            .items
            .as_ref()
            .map_or_else(whole, itemised)
            .id(self.id.map(|start| given::<ExpenseId>(start, position)));
        if let Some(recipe) = self.tip {
            let tip = if recipe.fixed {
                TipRule::fixed(money(i64::from(recipe.value)))
            } else {
                TipRule::percentage(Percentage::from_basis_points(recipe.value))
            };
            let distribution = if recipe.equal {
                TipDistribution::Equal
            } else {
                TipDistribution::Proportional
            };
            builder = builder.tip(tip.distributed(distribution));
        }
        // The payers divide the total, known only once the tip is resolved;
        // the draft, paid by the first member, resolves it.
        let draft = builder.clone().build().unwrap();
        builder
            .payers(self.payers.build(members, draft.total()))
            .build()
            .unwrap()
    }
}

/// A transfer of an amount in minor units between two members, by their
/// indices among the group's members, and where in `GIVEN` the identifier
/// an app gives it starts, or `None` for a generated one.
#[derive(Debug, Clone, Copy)]
struct TransferRecipe {
    id: Option<usize>,
    from: usize,
    to: usize,
    amount: i64,
}

#[derive(Debug, Clone)]
struct ExpenseGroupRecipe {
    expenses: Vec<ExpenseRecipe>,
    transfers: Vec<TransferRecipe>,
}

fn group_recipe() -> impl Strategy<Value = ExpenseGroupRecipe> {
    (
        proptest::collection::vec(expense_recipe(), 0..12),
        proptest::collection::vec(
            (
                proptest::option::of(0..GIVEN.len()),
                0..MEMBERS,
                0..MEMBERS,
                1_i64..100_000,
            )
                .prop_map(|(id, from, to, amount)| TransferRecipe {
                    id,
                    from,
                    to,
                    amount,
                }),
            0..6,
        ),
    )
        .prop_map(|(expenses, transfers)| ExpenseGroupRecipe {
            expenses,
            transfers,
        })
}

/// The members of a recipe, and its entries built once, so the same
/// entries, with the same identifiers, can go into several groups.
struct Entries {
    members: Vec<Member>,
    expenses: Vec<Expense>,
    transfers: Vec<Transfer>,
}

fn entries(recipe: &ExpenseGroupRecipe) -> Entries {
    // Every other member under an identifier an app gives, the others under
    // generated ones.
    let members: Vec<Member> = (0..MEMBERS)
        .map(|position| {
            let id: Option<MemberId> =
                (0 == position % 2).then(|| given(position % GIVEN.len(), position));
            Member::builder("Member").id(id).build().unwrap()
        })
        .collect();
    let ids: Vec<MemberId> = members.iter().map(Member::id).collect();
    let expenses = recipe
        .expenses
        .iter()
        .enumerate()
        .map(|(position, expense)| expense.build(position, &ids))
        .collect();
    let transfers = recipe
        .transfers
        .iter()
        .enumerate()
        .filter(|(_, transfer)| transfer.from != transfer.to)
        .map(|(position, transfer)| {
            let id: Option<TransferId> = transfer.id.map(|start| given(start, position));
            Transfer::builder(
                ids[transfer.from],
                ids[transfer.to],
                Money::from_minor_units(transfer.amount, CurrencyCode::EUR),
                day(),
            )
            .id(id)
            .build()
            .unwrap()
        })
        .collect();
    Entries {
        members,
        expenses,
        transfers,
    }
}

/// A group of `members` without entries; clones of it share its identifier.
fn base(members: &[Member]) -> ExpenseGroup {
    let mut group = ExpenseGroup::builder("Random", CurrencyCode::EUR)
        .build()
        .unwrap();
    for member in members {
        group.add_member(member.clone()).unwrap();
    }
    group
}

/// `base` with the entries added in the order given.
fn group(base: &ExpenseGroup, expenses: &[Expense], transfers: &[Transfer]) -> ExpenseGroup {
    let mut group = base.clone();
    for expense in expenses {
        // A conversion far out of range is refused; nothing else may be.
        match group.add_expense(expense.clone()) {
            Ok(_) | Err(ExpenseGroupError::Overflow) => {}
            Err(error) => panic!("a valid expense was refused: {error}"),
        }
    }
    for transfer in transfers {
        group.add_transfer(transfer.clone()).unwrap();
    }
    group
}

proptest! {
    /// The books balance: paid, consumed, and spent agree, balances sum to
    /// zero, and category totals add up to spending.
    #[test]
    fn the_books_balance(recipe in group_recipe()) {
        let Entries { members, expenses, transfers } = entries(&recipe);
        let group = group(&base(&members), &expenses, &transfers);
        let summary = group.summary();
        let total = |values: Vec<Money>| values.iter().map(|value| i128::from(value.minor_units())).sum::<i128>();
        let spent = i128::from(summary.total_spent().minor_units());
        prop_assert_eq!(total(summary.members().map(|member| member.paid()).collect()), spent);
        prop_assert_eq!(total(summary.members().map(|member| member.share()).collect()), spent);
        prop_assert_eq!(total(summary.members().map(|member| member.net()).collect()), 0);
        prop_assert_eq!(total(summary.category_totals().map(CategoryTotal::total).collect()), spent);
        for totals in summary.members() {
            prop_assert_eq!(total(summary.member_category_totals(totals.member()).map(CategoryTotal::total).collect()), i128::from(totals.share().minor_units()));
        }
    }

    /// Every strategy's plan clears every balance when recorded, the
    /// fewest-payments plan never needs more payments than greedy, which
    /// never needs more than one fewer than the open balances, and the
    /// adaptive plan of these six members is the fewest-payments plan.
    #[test]
    fn every_plan_clears_every_balance(recipe in group_recipe()) {
        let Entries { members, expenses, transfers } = entries(&recipe);
        let group = group(&base(&members), &expenses, &transfers);
        let open = group.summary().members().filter(|member| !member.net().is_zero()).count();
        let minimum = group.settlement(SettlementStrategy::MinimumTransfers).unwrap();
        let greedy = group.settlement(SettlementStrategy::Greedy).unwrap().len();
        prop_assert!(minimum.len() <= greedy);
        prop_assert!(greedy <= open.saturating_sub(1));
        prop_assert_eq!(group.settlement(SettlementStrategy::Adaptive).unwrap(), minimum);
        for strategy in SettlementStrategy::all() {
            let mut settled = group.clone();
            let plan = settled.settlement(strategy).unwrap().clone();
            for payment in plan.payments() {
                prop_assert!(payment.amount().is_positive());
                settled.add_transfer(Transfer::builder_from_payment(payment, day()).build().unwrap()).unwrap();
            }
            prop_assert!(settled.summary().members().all(|member| member.net().is_zero()));
            prop_assert!(settled.settlement(strategy).unwrap().is_empty());
        }
    }

    /// A plan awaited in an executor would differ from the one computed on
    /// the caller's thread.
    #[test]
    fn an_awaited_plan_equals_a_computed_one(recipe in group_recipe()) {
        let Entries { members, expenses, transfers } = entries(&recipe);
        let group = group(&base(&members), &expenses, &transfers);
        for strategy in SettlementStrategy::all() {
            // A copy, so each call computes its plan instead of reading the other's.
            let awaited = group.clone();
            prop_assert_eq!(block_on(awaited.settlement_async(strategy)), group.settlement(strategy));
        }
    }

    /// A currency change awaited in an executor would leave another group
    /// than the one made on the caller's thread.
    #[test]
    fn an_awaited_currency_change_equals_a_computed_one(recipe in group_recipe()) {
        let Entries { members, expenses, transfers } = entries(&recipe);
        let mut computed = group(&base(&members), &expenses, &transfers);
        let mut awaited = computed.clone();
        let rate = |text| ExchangeRate::parse(text).unwrap();
        let rates = [
            CurrencyRate::new(CurrencyCode::BHD, rate("2.65")),
            CurrencyRate::new(CurrencyCode::EUR, rate("1.1")),
            CurrencyRate::new(CurrencyCode::JPY, rate("0.0067")),
            CurrencyRate::new(CurrencyCode::RON, rate("0.22")),
        ];
        prop_assert_eq!(
            block_on(awaited.change_currency_async(CurrencyCode::USD, &rates)),
            computed.change_currency(CurrencyCode::USD, &rates)
        );
        prop_assert_eq!(&awaited, &computed);
        prop_assert_eq!(awaited.summary(), computed.summary());
    }

    /// The same entries in the opposite order give an equal group, totals,
    /// and plans, so two devices that received them differently agree.
    #[test]
    fn the_order_of_entries_does_not_matter(recipe in group_recipe()) {
        let Entries { members, mut expenses, mut transfers } = entries(&recipe);
        let base = base(&members);
        let forward = group(&base, &expenses, &transfers);
        expenses.reverse();
        transfers.reverse();
        let backward = group(&base, &expenses, &transfers);
        prop_assert_eq!(&forward, &backward);
        prop_assert_eq!(forward.summary(), backward.summary());
        for strategy in SettlementStrategy::all() {
            prop_assert_eq!(forward.settlement(strategy).unwrap(), backward.settlement(strategy).unwrap());
        }
    }

    /// Removing an expense and adding it back, under the identifier it
    /// carries, restores the exact same totals.
    #[test]
    fn removing_and_adding_back_restores_the_totals(recipe in group_recipe()) {
        let Entries { members, expenses, transfers } = entries(&recipe);
        let mut group = group(&base(&members), &expenses, &transfers);
        let summary = group.summary().clone();
        let ids: Vec<ExpenseId> = group.expenses().map(Expense::id).collect();
        for id in ids {
            let expense = group.remove_expense(id).unwrap();
            prop_assert_eq!(group.add_expense(expense), Ok(id));
            prop_assert_eq!(group.summary(), &summary);
        }
    }
}
