//! The C bridge's files, as the generator writes them from a small core that
//! holds every kind of field, variant, parameter, and answer the bridge
//! carries. A file missing an import, importing what it defines, or naming a
//! type after another rule than the C API's would fail to build, or build a
//! C API other than the one the C and C++ packages call.

use quote::{ToTokens, quote};

use super::*;

/// The records and enums of a core that holds every kind of field and
/// variant the bridge carries, and the names the templates and the group's
/// constructor need.
pub(crate) const RECORDS: &str = "
/// An amount.
#[mirrored]
pub struct Money {
    /// The currency.
    pub currency: String,
    /// The minor units.
    pub minor_units: i64,
}

/// A day.
#[mirrored]
pub struct CalendarDate {
    /// The day of the month.
    pub day: i32,
}

/// A time of day.
#[mirrored]
pub struct TimeOfDay {
    /// The hour.
    pub hour: i32,
}

/// How a tip is shared.
#[mirrored]
pub enum TipDistribution {
    /// The same for everyone.
    Equal,
    /// In proportion to each share.
    Proportional,
}

/// A line of a bill.
#[mirrored]
pub struct ExpenseItem {
    /// What was bought.
    pub name: String,
    /// How many units.
    pub quantity: i32,
}

/// A tip.
#[mirrored]
pub struct Tip {
    /// How it is shared.
    pub distribution: TipDistribution,
    /// The amount.
    pub value: Money,
}

/// How an amount divides.
#[mirrored]
pub enum Split {
    /// Exact amounts.
    Amounts {
        /// The amounts.
        shares: Vec<Money>,
    },
    /// The same for everyone.
    Equal {
        /// The members.
        members: Vec<String>,
    },
    /// Items.
    Itemised {
        /// The items.
        shares: Vec<ExpenseItem>,
    },
}

/// What an entry cost.
#[mirrored]
pub enum Cost {
    /// In another currency.
    Converted {
        /// The amount.
        amount: Money,
        /// The day.
        date: CalendarDate,
        /// The items.
        items: Vec<ExpenseItem>,
        /// The decimal places.
        places: i32,
        /// The rate.
        rate: String,
    },
    /// In the group currency.
    Local {
        /// The amount.
        amount: Money,
        /// The day.
        date: CalendarDate,
        /// The items.
        items: Vec<ExpenseItem>,
        /// The decimal places.
        places: i32,
        /// The rate.
        rate: String,
    },
}

/// Where a member stands.
#[mirrored]
pub enum Balance {
    /// The others owe the member.
    GetsBack {
        /// What the member gets back.
        amount: Money,
        /// A note.
        note: String,
        /// Since when.
        since: CalendarDate,
    },
    /// The member owes the others.
    Owes {
        /// How many.
        count: i32,
        /// How it is shared.
        distribution: TipDistribution,
        /// What the member owes.
        amount: Money,
    },
    /// The member owes nothing.
    Settled,
    /// The member waits.
    Waiting {
        /// What the member waits for.
        amount: Money,
        /// A note.
        note: String,
    },
}

/// An expense.
#[mirrored]
pub struct Expense {
    /// The identifier.
    pub id: Option<String>,
    /// What it cost.
    pub cost: Cost,
    /// How many.
    pub count: i32,
    /// The day.
    pub date: CalendarDate,
    /// How it is shared.
    pub distribution: TipDistribution,
    /// The name.
    pub name: String,
    /// The tip.
    pub tip: Tip,
    /// The fallback.
    pub fallback: Option<TipDistribution>,
    /// The items.
    pub items: Vec<ExpenseItem>,
    /// The limit.
    pub limit: Option<i64>,
    /// The members.
    pub members: Vec<String>,
    /// The split.
    pub split: Option<Split>,
    /// The time.
    pub time: Option<TimeOfDay>,
}

/// A currency.
#[mirrored]
pub struct CurrencyDetails {
    /// The code.
    pub code: String,
}

/// What a new group starts with.
#[mirrored]
pub struct NewExpenseGroup {
    /// The name.
    pub name: String,
}

/// The totals.
#[mirrored]
pub struct Summary {
    /// Where every member stands.
    pub balances: Vec<Balance>,
    /// What the group spent.
    pub total: Money,
}

/// Not mirrored, so the bridge holds nothing of it.
pub struct Handle(u8);
";

/// The trait of calls of the same core: a read of every kind of answer and
/// a change of every kind of parameter.
pub(crate) const CALLS: &str = "
#[eranos_glue_macros::calls]
trait ExpenseGroupCalls {
    /// The expense with the identifier `id`.
    #[read]
    fn expense(&self, id: &str) -> Result<Option<Expense>, EranosError>;

    /// Every expense.
    #[read]
    #[infallible]
    fn expenses(&self) -> Result<Vec<Expense>, EranosError>;

    /// The group's name.
    ///
    /// It is never blank.
    #[read]
    #[infallible]
    fn name(&self) -> Result<String, EranosError>;

    /// The description.
    #[read]
    #[infallible]
    fn description(&self) -> Result<Option<String>, EranosError>;

    /// Whether the member with the identifier `id` is archived.
    #[read]
    fn is_archived(&self, id: &str) -> Result<bool, EranosError>;

    /// The totals.
    #[read]
    #[infallible]
    fn summary(&self) -> Result<Summary, EranosError>;

    /// Adds `expense` and returns its identifier.
    #[write]
    fn add_expense(&self, expense: &Expense) -> Result<String, EranosError>;

    /// Sets the members.
    #[write]
    fn set_members(&self, members: &[String], items: &[ExpenseItem]) -> Result<(), EranosError>;

    /// Sets the language, or clears it when there is none.
    #[write]
    fn set_language(&self, language: Option<&str>, count: i32) -> Result<(), EranosError>;

    /// Moves the day and the sharing.
    #[write]
    fn move_to(&self, date: CalendarDate, distribution: TipDistribution) -> Result<(), EranosError>;
}
";

/// The sources of the core: its records, then its calls.
pub(crate) fn sources() -> Vec<String> {
    vec![RECORDS.to_owned(), CALLS.to_owned()]
}

/// The file at `path` among `files`.
fn at<'files>(files: &'files [Generated], path: &str) -> &'files Generated {
    files
        .iter()
        .find(|file| file.path() == path)
        .unwrap_or_else(|| panic!("no file {path}"))
}

/// The imports of `generated`'s module, as they print.
fn imports(generated: &Generated) -> Vec<String> {
    let Some(syn::Item::Mod(module)) = generated.file.items.first() else {
        panic!("one module");
    };
    let (_, content) = module.content.as_ref().unwrap();
    content
        .iter()
        .filter_map(|item| {
            if let syn::Item::Use(import) = item {
                Some(import.to_token_stream().to_string())
            } else {
                None
            }
        })
        .collect()
}

/// The bridge is two files, the group's and the records', each a module
/// that Diplomat reads as C API and imports exactly the types it names from
/// the other modules, in rustfmt's order, and each file's text holds its
/// code, opening with the notice not to edit it.
#[test]
fn the_bridge_is_two_modules_that_import_what_they_name() {
    let files = bridge(&sources()).unwrap();
    assert_eq!(
        files.iter().map(Generated::path).collect::<Vec<_>>(),
        ["generated/expense_group.rs", "generated/records.rs"]
    );
    for generated in &files {
        let text = generated.text();
        assert!(text.starts_with("// Written by `cargo xtask generate`"));
        assert!(generated.is_written_in(&text).unwrap());
        assert!(format!("{generated:?}").contains(generated.path()));
    }
    let expected = |imports: TokenStream| {
        syn::parse2::<syn::File>(imports)
            .unwrap()
            .items
            .iter()
            .map(|item| item.to_token_stream().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        imports(at(&files, "generated/expense_group.rs")),
        expected(quote! {
            use crate::error::ffi::EranosError;
            use crate::generated::records::ffi::{
                EranosExpense, EranosExpenseList, EranosItemList, EranosNewExpenseGroup, EranosSummary,
                EranosTipDistribution
            };
            use crate::values::ffi::{EranosCalendarDate, EranosTextList};
            use diplomat_runtime::{DiplomatStr, DiplomatWrite};
            use std::sync::{Arc, OnceLock, mpsc};
        })
    );
    assert_eq!(
        imports(at(&files, "generated/records.rs")),
        expected(quote! {
            use crate::error::ffi::EranosError;
            use crate::values::ffi::{EranosCalendarDate, EranosMoney, EranosTextList, EranosTimeOfDay};
            use diplomat_runtime::{DiplomatStr, DiplomatWrite};
        })
    );
}

/// An import of one type is written without braces, as rustfmt writes it,
/// and a module that names nothing of another imports nothing of it.
#[test]
fn a_single_import_has_no_braces() {
    let file = module(
        vec![Named {
            name: "EranosWord".to_owned(),
            tokens: quote! {
                pub struct EranosWord(pub(crate) EranosMoney);
            },
        }],
        &BTreeSet::new(),
    );
    let generated = Generated {
        path: "generated/word.rs",
        file,
    };
    assert_eq!(
        imports(&generated),
        [quote!(
            use crate::values::ffi::EranosMoney;
        )
        .to_string()]
    );
}

/// Names follow the C API's rules: the core's type after `Eranos`, a list
/// after its element, and prose in words with their article.
#[test]
fn names_and_prose_follow_the_c_api() {
    let name = format_ident!("MemberAmount");
    assert_eq!(eranos(&name), "EranosMemberAmount");
    assert_eq!(list(&name), "EranosMemberAmountList");
    assert_eq!(words(&name), "member amount");
    assert_eq!(article("member amount"), "a member amount");
    assert_eq!(article("expense"), "an expense");
    assert_eq!(inside("The amount."), "the amount.");
    assert_eq!(inside("IBAN of the account."), "IBAN of the account.");
    assert_eq!(inside("A"), "A");
    assert_eq!(inside(""), "");
    assert_eq!(capitalised("in `A`, x"), "In `A`, x");
    assert_eq!(capitalised(""), "");
    assert_eq!(
        doc(&["One.", "Two."]).to_string(),
        quote!(#[doc = "One.\n\nTwo."]).to_string()
    );
    assert_eq!(
        doc_with_errors("One.", "Two.").to_string(),
        quote!(#[doc = "One.\n\n# Errors\n\nTwo."]).to_string()
    );
}

/// Every identifier of a stream counts, those inside its groups too, so an
/// import is never missing for a type named deep in a method.
#[test]
fn every_identifier_of_a_stream_counts() {
    let mut found = BTreeSet::new();
    names(
        &quote!(
            fn outer() {
                inner(Box::new(Deep))
            }
        ),
        &mut found,
    );
    assert!(
        ["outer", "inner", "Box", "new", "Deep", "fn"]
            .iter()
            .all(|name| found.contains(*name))
    );
}

/// Sources that are not Rust, or that the rules refuse, give no bridge:
/// a type the bridge cannot carry, an optional field of an enum's variant,
/// and optional text a call takes twice.
#[test]
fn sources_the_rules_refuse_give_no_bridge() {
    assert!(bridge(&["fn (".to_owned()]).is_err());
    assert!(bridge(&[RECORDS.to_owned()]).is_err());
    let refused = CALLS.replace(
        "-> Result<bool, EranosError>",
        "-> Result<i32, EranosError>",
    );
    assert!(bridge(&[RECORDS.to_owned(), refused]).is_err());
    let unsupported = RECORDS.replace("pub code: String,", "pub code: f64,");
    assert!(bridge(&[unsupported, CALLS.to_owned()]).is_err());
    let optional = RECORDS.replace(
        "        note: String,\n        /// Since",
        "        note: Option<String>,\n        /// Since",
    );
    assert_ne!(optional, RECORDS);
    assert!(bridge(&[optional, CALLS.to_owned()]).is_err());
    let twice = CALLS.replace(
        "count: i32) -> Result<(), EranosError>",
        "count: Option<&str>) -> Result<(), EranosError>",
    );
    assert_ne!(twice, CALLS);
    assert!(bridge(&[RECORDS.to_owned(), twice]).is_err());
}

/// Tokens that are not Rust stay in their file as they are, where rustfmt
/// and the compiler refuse them, rather than be dropped.
#[test]
fn tokens_that_are_not_rust_stay_verbatim() {
    let tokens = quote!(fn broken;);
    assert_eq!(
        file(tokens.clone()).to_token_stream().to_string(),
        tokens.to_string()
    );
}

/// The crate's entry point writes the same bridge, and refuses sources
/// without a trait of calls.
#[test]
fn the_entry_point_writes_the_bridge() {
    let files = crate::capi_bridge(&sources()).unwrap();
    assert_eq!(files.len(), 2);
    assert!(crate::capi_bridge(&[]).is_err());
}
