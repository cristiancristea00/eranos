//! The binding core's records and enums, mirrored for `uniffi`: each
//! definition repeats the core's type field for field, and `uniffi`
//! derives its conversions from the copy while the core keeps the type.

use eranos_bindings::{
    Account, AccountDetails, Balance, CalendarDate, Category, CategoryTotal, Cost, CurrencyDetails,
    CurrencyRate, Expense, ExpenseShares, ExpenseSplit, Item, Member, MemberAmount,
    MemberCategoryTotals, MemberParts, MemberPercentage, MemberShare, MemberSummary, Money,
    NewGroup, Payment, Settlement, SettlementStrategy, Split, Summary, TimeOfDay, Tip,
    TipDistribution, TipValue, Transfer,
};

#[uniffi::remote(Record)]
pub struct CalendarDate {
    pub day: i32,
    pub month: i32,
    pub year: i32,
}

#[uniffi::remote(Record)]
pub struct TimeOfDay {
    pub hour: i32,
    pub minute: i32,
}

#[uniffi::remote(Record)]
pub struct Money {
    pub currency: String,
    pub minor_units: i64,
}

#[uniffi::remote(Record)]
pub struct CurrencyRate {
    pub currency: String,
    pub rate: String,
}

#[uniffi::remote(Record)]
pub struct CurrencyDetails {
    pub code: String,
    pub decimal_places: i32,
    pub name: String,
    pub numeric_code: i32,
}

#[uniffi::remote(Enum)]
pub enum Cost {
    Converted { amount: Money, rate: String },
    Local { amount: Money },
}

#[uniffi::remote(Record)]
pub struct MemberAmount {
    pub amount: Money,
    pub member: String,
}

#[uniffi::remote(Record)]
pub struct MemberParts {
    pub hundredths: i32,
    pub member: String,
}

#[uniffi::remote(Record)]
pub struct MemberPercentage {
    pub basis_points: i32,
    pub member: String,
}

#[uniffi::remote(Record)]
pub struct Item {
    pub name: String,
    pub split: Split,
    pub unit_price: Money,
    pub quantity: i32,
}

#[uniffi::remote(Record)]
pub struct Tip {
    pub value: TipValue,
    pub distribution: TipDistribution,
}

#[uniffi::remote(Record)]
pub struct Expense {
    pub id: Option<String>,
    pub cost: Cost,
    pub date: CalendarDate,
    pub name: String,
    pub payers: Split,
    pub split: ExpenseSplit,
    pub category: Option<String>,
    pub description: Option<String>,
    pub time: Option<TimeOfDay>,
    pub tip: Option<Tip>,
}

#[uniffi::remote(Enum)]
pub enum Split {
    Amounts { shares: Vec<MemberAmount> },
    Equal { members: Vec<String> },
    Parts { shares: Vec<MemberParts> },
    Percentages { shares: Vec<MemberPercentage> },
}

#[uniffi::remote(Enum)]
pub enum ExpenseSplit {
    Itemised { items: Vec<Item> },
    Whole { split: Split },
}

#[uniffi::remote(Enum)]
pub enum TipValue {
    Fixed { amount: Money },
    Percentage { basis_points: i32 },
}

#[uniffi::remote(Enum)]
pub enum TipDistribution {
    Equal,
    Proportional,
}

#[uniffi::remote(Record)]
pub struct Transfer {
    pub id: Option<String>,
    pub cost: Cost,
    pub date: CalendarDate,
    pub from: String,
    pub to: String,
    pub note: Option<String>,
    pub time: Option<TimeOfDay>,
}

#[uniffi::remote(Record)]
pub struct Account {
    pub details: AccountDetails,
    pub name: String,
}

#[uniffi::remote(Record)]
pub struct Member {
    pub id: Option<String>,
    pub first_name: String,
    pub accounts: Vec<Account>,
    pub email: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

#[uniffi::remote(Record)]
pub struct Category {
    pub id: Option<String>,
    pub name: String,
}

#[uniffi::remote(Enum)]
pub enum AccountDetails {
    Iban { iban: String },
    Username { username: String },
}

#[uniffi::remote(Record)]
pub struct MemberSummary {
    pub balance: Balance,
    pub member: String,
    pub net: Money,
    pub paid: Money,
    pub received: Money,
    pub sent: Money,
    pub share: Money,
}

#[uniffi::remote(Record)]
pub struct CategoryTotal {
    pub category: String,
    pub total: Money,
}

#[uniffi::remote(Record)]
pub struct MemberCategoryTotals {
    pub member: String,
    pub totals: Vec<CategoryTotal>,
}

#[uniffi::remote(Record)]
pub struct Summary {
    pub category_totals: Vec<CategoryTotal>,
    pub member_category_totals: Vec<MemberCategoryTotals>,
    pub members: Vec<MemberSummary>,
    pub total_spent: Money,
}

#[uniffi::remote(Record)]
pub struct MemberShare {
    pub member: String,
    pub paid: Money,
    pub share: Money,
}

#[uniffi::remote(Record)]
pub struct ExpenseShares {
    pub members: Vec<MemberShare>,
    pub tip: Money,
    pub total: Money,
}

#[uniffi::remote(Enum)]
pub enum Balance {
    GetsBack { amount: Money },
    Owes { amount: Money },
    Settled,
}

#[uniffi::remote(Record)]
pub struct Payment {
    pub amount: Money,
    pub from: String,
    pub to: String,
}

#[uniffi::remote(Record)]
pub struct Settlement {
    pub payments: Vec<Payment>,
    pub strategy: SettlementStrategy,
}

#[uniffi::remote(Enum)]
pub enum SettlementStrategy {
    Direct,
    Greedy,
    MinimumTransfers,
}

#[uniffi::remote(Record)]
pub struct NewGroup {
    pub currency: String,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
}
