//! How each of the core's types crosses the C bridge, and which records,
//! lists, and enums the bridge writes. A type crossed another way would
//! build a C API whose getters and setters the C and C++ packages do not
//! call; a record a call takes left read-only, or a list written that
//! nothing reaches, would leave the C API without a constructor it needs or
//! with one no caller has.

use quote::quote;
use syn::Type;

use super::*;
use crate::capi::tests::{CALLS, RECORDS, sources};

/// How a crossing takes a parameter named `value`, and the value it owns
/// from it.
struct Case {
    /// The crossing.
    crossing: Crossing,
    /// The value it owns.
    owned: TokenStream,
    /// The parameter it takes.
    parameter: TokenStream,
}

/// A type the plan writes a list or an object of, by name, and whether a
/// call takes it.
struct Reached {
    /// The type's name.
    name: &'static str,
    /// Whether a call takes it.
    taken: bool,
}

/// The fixture's core.
fn core() -> Core {
    Core::parse(&sources()).unwrap()
}

/// The type written `text`.
fn written(text: &str) -> Type {
    syn::parse_str(text).unwrap()
}

/// Text, integers, plain structs, enums without data, records, enums with
/// data, and lists of text or of records each cross in their own way, and
/// any other type is refused.
#[test]
fn every_type_crosses_by_its_kind() {
    let core = core();
    let crossing = |text: &str| Crossing::of(&written(text), &core);
    assert_eq!(crossing("String").unwrap(), Crossing::Text);
    assert_eq!(
        crossing("i32").unwrap(),
        Crossing::Integer(format_ident!("i32"))
    );
    assert_eq!(
        crossing("i64").unwrap(),
        Crossing::Integer(format_ident!("i64"))
    );
    assert_eq!(
        crossing("CalendarDate").unwrap(),
        Crossing::Plain(format_ident!("CalendarDate"))
    );
    assert_eq!(
        crossing("TipDistribution").unwrap(),
        Crossing::Choice(format_ident!("TipDistribution"))
    );
    assert_eq!(
        crossing("Expense").unwrap(),
        Crossing::Object(format_ident!("Expense"))
    );
    assert_eq!(
        crossing("Cost").unwrap(),
        Crossing::Object(format_ident!("Cost"))
    );
    assert_eq!(crossing("Vec<String>").unwrap(), Crossing::TextList);
    assert_eq!(
        crossing("Vec<Item>").unwrap(),
        Crossing::List(format_ident!("Item"))
    );
    for refused in [
        "f64",
        "(i32, i32)",
        "Box<String>",
        "Vec<i32>",
        "Vec<Vec<String>>",
        "Vec<String, Global>",
        "Vec<f64>",
        "Vec<'static>",
        "Unknown",
        "Handle",
    ] {
        assert!(crossing(refused).is_err(), "{refused}");
    }
}

/// An `Option` makes a value optional, but a list is never optional: an
/// empty list says the same, and Diplomat has no optional list.
#[test]
fn an_option_is_optional_but_a_list_never_is() {
    let core = core();
    assert_eq!(
        Value::of(&written("Option<String>"), &core).unwrap(),
        Value {
            crossing: Crossing::Text,
            optional: true
        }
    );
    assert_eq!(
        Value::of(&written("String"), &core).unwrap(),
        Value {
            crossing: Crossing::Text,
            optional: false
        }
    );
    for refused in ["Option<Vec<String>>", "Option<Vec<Item>>", "Option<f64>"] {
        assert!(Value::of(&written(refused), &core).is_err(), "{refused}");
    }
}

/// A record, a list, and the list of text cross in an opaque object named
/// after them, taken by reference; text crosses as bytes; an integer as
/// itself; and a plain struct or a C-like enum by value, converted.
#[test]
fn every_crossing_has_its_parameter_and_its_owned_value() {
    let name = format_ident!("value");
    let cases = [
        Case {
            crossing: Crossing::Choice(format_ident!("TipDistribution")),
            owned: quote!(value.into()),
            parameter: quote!(value: EranosTipDistribution),
        },
        Case {
            crossing: Crossing::Integer(format_ident!("i32")),
            owned: quote!(value),
            parameter: quote!(value: i32),
        },
        Case {
            crossing: Crossing::List(format_ident!("Item")),
            owned: quote!(value.0.clone()),
            parameter: quote!(value: &EranosItemList),
        },
        Case {
            crossing: Crossing::Object(format_ident!("Expense")),
            owned: quote!(value.0.clone()),
            parameter: quote!(value: &EranosExpense),
        },
        Case {
            crossing: Crossing::Plain(format_ident!("CalendarDate")),
            owned: quote!(value.into()),
            parameter: quote!(value: EranosCalendarDate),
        },
        Case {
            crossing: Crossing::Text,
            owned: quote!(crate::text(value)?),
            parameter: quote!(value: &DiplomatStr),
        },
        Case {
            crossing: Crossing::TextList,
            owned: quote!(value.0.clone()),
            parameter: quote!(value: &EranosTextList),
        },
    ];
    for Case {
        crossing,
        owned,
        parameter,
    } in cases
    {
        assert_eq!(
            crossing.parameter(&name).to_string(),
            parameter.to_string(),
            "{crossing:?}"
        );
        assert_eq!(
            crossing.owned(&name).to_string(),
            owned.to_string(),
            "{crossing:?}"
        );
    }
    assert_eq!(Crossing::Text.opaque(), None);
    assert_eq!(
        Crossing::TextList.opaque(),
        Some(format_ident!("EranosTextList"))
    );
}

/// A call answers nothing, a flag, text, optional text, a record, an
/// optional record, or a list of records, and nothing else.
#[test]
fn a_call_answers_one_of_seven_kinds() {
    let core = core();
    let answer = |text: &str| Answer::of(&written(text), &core);
    assert_eq!(answer("()").unwrap(), Answer::Nothing);
    assert_eq!(answer("bool").unwrap(), Answer::Flag);
    assert_eq!(answer("String").unwrap(), Answer::Text);
    assert_eq!(answer("Option<String>").unwrap(), Answer::OptionalText);
    assert_eq!(
        answer("Expense").unwrap(),
        Answer::Record(format_ident!("Expense"))
    );
    assert_eq!(
        answer("Option<Expense>").unwrap(),
        Answer::Found(format_ident!("Expense"))
    );
    assert_eq!(
        answer("Vec<Expense>").unwrap(),
        Answer::List(format_ident!("Expense"))
    );
    for refused in [
        "i32",
        "Option<i32>",
        "Vec<String>",
        "CalendarDate",
        "TipDistribution",
        "(bool, bool)",
        "f64",
    ] {
        assert!(answer(refused).is_err(), "{refused}");
    }
}

/// A call takes text, optional text, a record by reference, a list of text
/// or of records, and anything else by value, but a record or a list by
/// value, or a reference to something else, is refused.
#[test]
fn a_call_takes_records_by_reference_and_values_by_value() {
    let core = core();
    let crossing = |kind: Kind| parameter_crossing(&kind, &core);
    assert_eq!(crossing(Kind::Text).unwrap(), Crossing::Text);
    assert_eq!(crossing(Kind::OptionalText).unwrap(), Crossing::Text);
    assert_eq!(
        crossing(Kind::Record(written("Expense"))).unwrap(),
        Crossing::Object(format_ident!("Expense"))
    );
    assert_eq!(
        crossing(Kind::List(written("String"))).unwrap(),
        Crossing::TextList
    );
    assert_eq!(
        crossing(Kind::List(written("Item"))).unwrap(),
        Crossing::List(format_ident!("Item"))
    );
    assert_eq!(
        crossing(Kind::Value(written("TipDistribution"))).unwrap(),
        Crossing::Choice(format_ident!("TipDistribution"))
    );
    assert_eq!(
        crossing(Kind::Value(written("i32"))).unwrap(),
        Crossing::Integer(format_ident!("i32"))
    );
    for refused in [
        Kind::Record(written("String")),
        Kind::Record(written("f64")),
        Kind::List(written("i32")),
        Kind::List(written("Vec<String>")),
        Kind::List(written("f64")),
        Kind::Value(written("Expense")),
        Kind::Value(written("Vec<String>")),
        Kind::Value(written("Vec<Item>")),
        Kind::Value(written("f64")),
    ] {
        assert!(crossing(refused).is_err());
    }
}

/// The plan holds what the calls, the group's constructor, and the list of
/// currencies reach, through every field: a record a call takes is taken,
/// though an earlier call answers it, and so is what it holds; a record
/// reached only from answers is not; the hand-written amount has no object.
#[test]
fn the_plan_holds_what_the_calls_reach() {
    let plan = Plan::of(&core()).unwrap();
    let names = |names: &[Reached]| {
        names
            .iter()
            .map(|reached| (reached.name.to_owned(), reached.taken))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(
        plan,
        Plan {
            choices: ["TipDistribution".to_owned()].into(),
            lists: names(&[
                Reached {
                    name: "Balance",
                    taken: false,
                },
                Reached {
                    name: "CurrencyDetails",
                    taken: false,
                },
                Reached {
                    name: "Expense",
                    taken: false,
                },
                Reached {
                    name: "Item",
                    taken: true,
                },
                Reached {
                    name: "Money",
                    taken: true,
                },
            ]),
            objects: names(&[
                Reached {
                    name: "Balance",
                    taken: false,
                },
                Reached {
                    name: "Cost",
                    taken: true,
                },
                Reached {
                    name: "CurrencyDetails",
                    taken: false,
                },
                Reached {
                    name: "Expense",
                    taken: true,
                },
                Reached {
                    name: "Item",
                    taken: true,
                },
                Reached {
                    name: "NewExpenseGroup",
                    taken: true,
                },
                Reached {
                    name: "Split",
                    taken: true,
                },
                Reached {
                    name: "Summary",
                    taken: false,
                },
                Reached {
                    name: "Tip",
                    taken: true,
                },
            ]),
        }
    );
}

/// A core without the records the constructor and the templates name gives
/// no plan, and neither does a field the bridge cannot carry.
#[test]
fn a_core_without_the_templates_records_gives_no_plan() {
    for records in [
        RECORDS.replace("pub struct NewExpenseGroup", "pub struct OldGroup"),
        RECORDS.replace("pub struct CurrencyDetails", "pub struct Currencies"),
        RECORDS.replace("pub code: String,", "pub code: f64,"),
    ] {
        let core = Core::parse(&[records, CALLS.to_owned()]).unwrap();
        assert!(Plan::of(&core).is_err());
    }
    let refused = CALLS.replace("items: &[Item]", "items: &[f64]");
    let core = Core::parse(&[RECORDS.to_owned(), refused]).unwrap();
    assert!(Plan::of(&core).is_err());
}

/// A field the bridge cannot carry is refused wherever the plan reaches it:
/// through a call's parameter, through its answer, and through a record
/// that another record holds.
#[test]
fn a_field_the_bridge_cannot_carry_is_refused_wherever_it_is_reached() {
    let records = "
        /// What a new group starts with.
        #[mirrored]
        pub struct NewExpenseGroup {
            /// The name.
            pub name: String,
        }

        /// A currency.
        #[mirrored]
        pub struct CurrencyDetails {
            /// The code.
            pub code: String,
        }

        /// A holder.
        #[mirrored]
        pub struct Outer {
            /// What it holds.
            pub inner: Inner,
        }

        /// What it holds.
        #[mirrored]
        pub struct Inner {
            /// A float.
            pub value: f64,
        }
    ";
    for calls in [
        "#[calls] trait ExpenseGroupCalls { /// Takes it. #[write] fn take(&self, inner: &Inner) -> Result<(), E>; }",
        "#[calls] trait ExpenseGroupCalls { /// Gives it. #[read] fn give(&self) -> Result<Inner, E>; }",
        "#[calls] trait ExpenseGroupCalls { /// Gives it. #[read] fn give(&self) -> Result<Outer, E>; }",
    ] {
        let calls = calls
            .replace("/// ", "#[doc = \"")
            .replace(". #[", ".\"] #[");
        let core = Core::parse(&[records.to_owned(), calls.clone()]).unwrap();
        assert!(Plan::of(&core).is_err(), "{calls}");
    }
}
