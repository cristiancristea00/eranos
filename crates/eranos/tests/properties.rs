//! Properties of random groups through the public API: every split method,
//! itemised bills, both tip distributions, several payers, foreign
//! currencies, and transfers. Whatever the entries, the books must balance
//! (paid, consumed, and spent agree; balances sum to zero; categories add up),
//! every strategy's plan must clear every balance, the fewest-payments plan
//! must beat or match greedy, a plan awaited in an executor must equal the
//! one computed on the caller's thread, and a group must not depend on the
//! order its entries arrived in (two devices syncing the same entries must
//! agree).
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "a failing test helper panics to report the failure, as clippy.toml allows inside test functions"
)]

use eranos::cooperative::block_on;
use eranos::{
    CalendarDate, CategoryTotal, Cost, Currency, CurrencyRate, ExchangeRate, Expense, ExpenseId,
    Group, Item, Member, MemberAmount, MemberId, MemberParts, MemberPercentage, Money, Parts,
    Percentage, SettlementStrategy, Split, Tip, TipDistribution, Transfer, error::GroupError,
};
use proptest::prelude::*;

const MEMBERS: usize = 6;

fn day() -> CalendarDate {
    CalendarDate::new(23, 9, 2026).unwrap()
}

/// How one amount is divided: a method and weighted members.
#[derive(Debug, Clone)]
struct SplitRecipe {
    members: Vec<(usize, u32)>,
    method: u8,
}

fn split_recipe() -> impl Strategy<Value = SplitRecipe> {
    (
        0_u8..4,
        proptest::collection::vec((0..MEMBERS, 1_u32..1_000), 1..5),
    )
        .prop_map(|(method, members)| SplitRecipe { members, method })
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
    parts.iter().all(|&part| part > 0).then_some(parts)
}

impl SplitRecipe {
    /// The split for dividing `total`, falling back to an equal split when
    /// the drawn method cannot give everyone a positive share.
    fn build(&self, members: &[MemberId], total: Money) -> Split {
        let mut chosen: Vec<(MemberId, u32)> = Vec::new();
        for &(index, weight) in &self.members {
            if chosen.iter().all(|&(member, _)| member != members[index]) {
                chosen.push((members[index], weight));
            }
        }
        let weights: Vec<u32> = chosen.iter().map(|&(_, weight)| weight).collect();
        let equal = Split::equal(chosen.iter().map(|&(member, _)| member));
        match self.method {
            0 => equal,
            1 => Split::parts(chosen.iter().map(|&(member, weight)| {
                MemberParts::new(member, Parts::from_hundredths(weight).unwrap())
            })),
            2 => positive_parts(10_000, &weights).map_or(equal, |points| {
                Split::percentages(chosen.iter().zip(points).map(|(&(member, _), points)| {
                    MemberPercentage::new(
                        member,
                        Percentage::from_basis_points(u32::try_from(points).unwrap()),
                    )
                }))
            }),
            _ => positive_parts(total.minor_units(), &weights).map_or(equal, |amounts| {
                Split::amounts(chosen.iter().zip(amounts).map(|(&(member, _), amount)| {
                    MemberAmount::new(Money::from_minor_units(total.currency(), amount), member)
                }))
            }),
        }
    }
}

#[derive(Debug, Clone)]
struct ExpenseRecipe {
    amount: i64,
    payers: SplitRecipe,
    split: SplitRecipe,
    foreign: Option<(u8, u64, u8)>,
    items: Option<Vec<(i64, u32, SplitRecipe)>>,
    tip: Option<(bool, u32, bool)>,
}

fn expense_recipe() -> impl Strategy<Value = ExpenseRecipe> {
    (
        1_i64..500_000,
        proptest::option::weighted(
            0.3,
            proptest::collection::vec((1_i64..20_000, 1_u32..4, split_recipe()), 1..4),
        ),
        split_recipe(),
        split_recipe(),
        proptest::option::of((any::<bool>(), 0_u32..3_000, any::<bool>())),
        proptest::option::weighted(0.3, (0_u8..3, 1_u64..100_000, 0_u8..5)),
    )
        .prop_map(
            |(amount, items, split, payers, tip, foreign)| ExpenseRecipe {
                amount,
                payers,
                split,
                foreign,
                items,
                tip,
            },
        )
}

impl ExpenseRecipe {
    fn build(&self, members: &[MemberId]) -> Expense {
        let (currency, rate) = match self.foreign {
            None => (Currency::EUR, None),
            Some((kind, mantissa, scale)) => {
                let currency = [Currency::JPY, Currency::BHD, Currency::RON][usize::from(kind)];
                (
                    currency,
                    Some(ExchangeRate::from_decimal(mantissa, scale).unwrap()),
                )
            }
        };
        let money = |minor_units| Money::from_minor_units(currency, minor_units);
        let cost = |amount: Money| rate.map_or_else(|| Cost::from(amount), |rate| amount.at(rate));
        let whole = || {
            let amount = money(self.amount);
            let split = self.split.build(members, amount);
            Expense::builder(cost(amount), day(), "Whole", members[0], split)
        };
        let itemised = |items: &Vec<(i64, u32, SplitRecipe)>| {
            let items: Vec<Item> = items
                .iter()
                .map(|(price, quantity, split)| {
                    let line = money(price * i64::from(*quantity));
                    Item::builder_with_split("Item", split.build(members, line), money(*price))
                        .quantity(*quantity)
                        .build()
                        .unwrap()
                })
                .collect();
            let amount: i64 = items
                .iter()
                .map(|item| item.line_total().minor_units())
                .sum();
            Expense::itemised_builder(cost(money(amount)), day(), items, "Itemised", members[0])
        };
        let mut builder = self.items.as_ref().map_or_else(whole, itemised);
        if let Some((fixed, value, equal)) = self.tip {
            let tip = if fixed {
                Tip::fixed(money(i64::from(value)))
            } else {
                Tip::percentage(Percentage::from_basis_points(value))
            };
            let distribution = if equal {
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

#[derive(Debug, Clone)]
struct GroupRecipe {
    expenses: Vec<ExpenseRecipe>,
    transfers: Vec<(usize, usize, i64)>,
}

fn group_recipe() -> impl Strategy<Value = GroupRecipe> {
    (
        proptest::collection::vec(expense_recipe(), 0..12),
        proptest::collection::vec((0..MEMBERS, 0..MEMBERS, 1_i64..100_000), 0..6),
    )
        .prop_map(|(expenses, transfers)| GroupRecipe {
            expenses,
            transfers,
        })
}

/// The members of a recipe, and its entries built once, so the same
/// entries, with the same identifiers, can go into several groups.
type Entries = (Vec<Member>, Vec<Expense>, Vec<Transfer>);

fn entries(recipe: &GroupRecipe) -> Entries {
    let members: Vec<Member> =
        std::iter::repeat_with(|| Member::builder("Member").build().unwrap())
            .take(MEMBERS)
            .collect();
    let ids: Vec<MemberId> = members.iter().map(Member::id).collect();
    let expenses = recipe
        .expenses
        .iter()
        .map(|expense| expense.build(&ids))
        .collect();
    let transfers = recipe
        .transfers
        .iter()
        .filter(|&&(from, to, _)| from != to)
        .map(|&(from, to, amount)| {
            Transfer::builder(
                Money::from_minor_units(Currency::EUR, amount),
                day(),
                ids[from],
                ids[to],
            )
            .build()
            .unwrap()
        })
        .collect();
    (members, expenses, transfers)
}

/// A group of `members` without entries; clones of it share its identifier.
fn base(members: &[Member]) -> Group {
    let mut group = Group::builder(Currency::EUR, "Random").build().unwrap();
    for member in members {
        group.add_member(member.clone()).unwrap();
    }
    group
}

/// `base` with the entries added in the order given.
fn group(base: &Group, expenses: &[Expense], transfers: &[Transfer]) -> Group {
    let mut group = base.clone();
    for expense in expenses {
        // A conversion far out of range is refused; nothing else may be.
        match group.add_expense(expense.clone()) {
            Ok(_) | Err(GroupError::Overflow) => {}
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
        let (members, expenses, transfers) = entries(&recipe);
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

    /// Every strategy's plan clears every balance when recorded, and the
    /// fewest-payments plan never needs more payments than greedy, which
    /// never needs more than one fewer than the open balances.
    #[test]
    fn every_plan_clears_every_balance(recipe in group_recipe()) {
        let (members, expenses, transfers) = entries(&recipe);
        let group = group(&base(&members), &expenses, &transfers);
        let open = group.summary().members().filter(|member| !member.net().is_zero()).count();
        let minimum = group.settlement(SettlementStrategy::MinimumTransfers).unwrap().len();
        let greedy = group.settlement(SettlementStrategy::Greedy).unwrap().len();
        prop_assert!(minimum <= greedy);
        prop_assert!(greedy <= open.saturating_sub(1));
        for strategy in SettlementStrategy::all() {
            let mut settled = group.clone();
            let plan = settled.settlement(strategy).unwrap().clone();
            for payment in plan.payments() {
                prop_assert!(payment.amount().is_positive());
                settled.add_transfer(Transfer::from_payment(payment, day())).unwrap();
            }
            prop_assert!(settled.summary().members().all(|member| member.net().is_zero()));
            prop_assert!(settled.settlement(strategy).unwrap().is_empty());
        }
    }

    /// A plan awaited in an executor would differ from the one computed on
    /// the caller's thread.
    #[test]
    fn an_awaited_plan_equals_a_computed_one(recipe in group_recipe()) {
        let (members, expenses, transfers) = entries(&recipe);
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
        let (members, expenses, transfers) = entries(&recipe);
        let mut computed = group(&base(&members), &expenses, &transfers);
        let mut awaited = computed.clone();
        let rate = |text| ExchangeRate::parse(text).unwrap();
        let rates = [
            CurrencyRate::new(Currency::BHD, rate("2.65")),
            CurrencyRate::new(Currency::EUR, rate("1.1")),
            CurrencyRate::new(Currency::JPY, rate("0.0067")),
            CurrencyRate::new(Currency::RON, rate("0.22")),
        ];
        prop_assert_eq!(
            block_on(awaited.change_currency_async(Currency::USD, &rates)),
            computed.change_currency(Currency::USD, &rates)
        );
        prop_assert_eq!(&awaited, &computed);
        prop_assert_eq!(awaited.summary(), computed.summary());
    }

    /// The same entries in the opposite order give an equal group, totals,
    /// and plans, so two devices that received them differently agree.
    #[test]
    fn the_order_of_entries_does_not_matter(recipe in group_recipe()) {
        let (members, mut expenses, mut transfers) = entries(&recipe);
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
        let (members, expenses, transfers) = entries(&recipe);
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
