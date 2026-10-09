//! The value helpers are the only way the apps parse and write values, so
//! every text a binding shows is the library's. A helper that formatted a
//! value its own way, let through text the library refuses, or wrapped a
//! number too large for the bindings' integers would show the apps other
//! values than the library holds.

use eranos::{CurrencyCode, MemberId, ledger};
use eranos_ledger::FromBalance;

use super::*;
use crate::expense::{
    Discount, DiscountScope, DiscountValue, ExpenseItem, MemberAmount, TipDistribution, TipRule,
    TipValue,
};
use crate::money::Cost;
use crate::{
    CalendarFailure, ContactFailure, ExpenseFailure, IdFailure, LanguageFailure, MoneyFailure,
    PaymentAccountFailure,
};

fn euros(minor_units: i64) -> Money {
    Money::from_library(library::Money::from_minor_units(
        minor_units,
        CurrencyCode::EUR,
    ))
}

fn lei(minor_units: i64) -> Money {
    Money::from_library(library::Money::from_minor_units(
        minor_units,
        CurrencyCode::RON,
    ))
}

/// Details of a currency the library does not list, with `decimal_places`
/// decimal places.
fn unlisted(decimal_places: u8) -> CurrencyDetails {
    CurrencyDetails {
        code: "XYZ".to_owned(),
        name: "Unlisted".to_owned(),
        numeric_code: 999,
        decimal_places,
    }
}

fn unknown_money() -> Money {
    Money {
        minor_units: 1,
        currency: unlisted(2),
    }
}

/// The identifier of the member `number`; members sort by number.
fn member(number: u8) -> String {
    MemberId::new(&format!("member-{number:03}"))
        .unwrap()
        .to_string()
}

fn equal(members: &[String]) -> Split {
    Split::Equal {
        members: members.to_vec(),
    }
}

/// A dinner of 30.00 EUR with a 10 % tip, which the first member paid and
/// the third and second share, listed in that order.
fn dinner() -> Expense {
    Expense {
        id: identifier_generate(),
        name: "Dinner".to_owned(),
        cost: Cost::Local {
            amount: euros(3_000),
        },
        payers: Split::Amounts {
            shares: vec![MemberAmount {
                member: member(1),
                amount: euros(3_300),
            }],
        },
        split: ExpenseSplit::Whole {
            split: equal(&[member(3), member(2)]),
        },
        date: CalendarDate {
            year: 2026,
            month: 9,
            day: 23,
        },
        tip: TipRule {
            value: TipValue::Percentage {
                basis_points: 1_000,
            },
            distribution: TipDistribution::Proportional,
        },
        time: None,
        category: category_id_uncategorised(),
        note: None,
    }
}

fn is_unknown_currency<T>(result: &Result<T, EranosError>) -> bool {
    matches!(
        result,
        Err(EranosError::Money {
            failure: MoneyFailure::UnknownCurrency,
            ..
        })
    )
}

/// A text the identifier rules refuse, and the failure that names why.
struct IdentifierRefusal {
    text: String,
    failure: IdFailure,
}

/// An app would give an entry an identifier the library refuses, change
/// its text, or not learn which rule the text broke.
#[test]
fn identifiers_parse_as_the_library_reads_them() {
    // Kept as it is, case and inner spaces included.
    for text in ["-food", "!a", "zz", "Lisboa · 2026", &"é".repeat(32)] {
        assert_eq!(identifier_parse(text), Ok(text.to_owned()), "{text}");
    }
    // Each refusal names the first rule the text breaks.
    for IdentifierRefusal { text, failure } in [
        IdentifierRefusal {
            text: String::new(),
            failure: IdFailure::Empty,
        },
        IdentifierRefusal {
            text: "x".repeat(65),
            failure: IdFailure::TooLong,
        },
        IdentifierRefusal {
            text: "a\tb".to_owned(),
            failure: IdFailure::ControlCharacter,
        },
        IdentifierRefusal {
            text: " x".to_owned(),
            failure: IdFailure::SurroundingWhitespace,
        },
    ] {
        assert!(
            matches!(
                identifier_parse(&text),
                Err(EranosError::Id { failure: refused, .. }) if refused == failure
            ),
            "{text:?}"
        );
    }
}

/// An app that asked for a new identifier would get one that the rules
/// refuse, that is not a UUID v7's text, or that another request also got.
#[test]
fn generated_identifiers_are_distinct_and_valid() {
    let (first, second) = (identifier_generate(), identifier_generate());
    // Distinct, and in the order they were generated.
    assert!(first < second, "{first} and {second}");
    for generated in [first, second] {
        assert_eq!(identifier_parse(&generated), Ok(generated.clone()));
        // A UUID v7, as lowercase hyphenated text.
        assert_eq!(generated.len(), 36);
        assert_eq!(generated.chars().nth(14), Some('7'));
        assert_eq!(generated.to_lowercase(), generated);
    }
}

/// An app that left an expense in the built-in category would name another
/// category than the one every group has, or one the identifier rules
/// refuse.
#[test]
fn the_built_in_category_is_named_by_the_nil_uuid() {
    let uncategorised = category_id_uncategorised();
    assert_eq!(uncategorised, "00000000-0000-0000-0000-000000000000");
    assert_eq!(identifier_parse(&uncategorised), Ok(uncategorised.clone()));
}

/// Amounts would be read or written another way than the library's, or a
/// sum across currencies, past the range, or in an unknown currency would
/// come back as an amount.
#[test]
fn amounts_parse_format_and_add_as_the_library_does() {
    assert_eq!(money_parse("12.5", "EUR"), Ok(euros(1_250)));
    assert!(is_unknown_currency(&money_parse("1", "XYZ")));
    assert!(matches!(
        money_parse("twelve", "EUR"),
        Err(EranosError::Money {
            failure: MoneyFailure::InvalidAmount,
            ..
        })
    ));
    assert!(matches!(
        money_parse("1.234", "EUR"),
        Err(EranosError::Money {
            failure: MoneyFailure::TooManyDecimals { .. },
            ..
        })
    ));
    assert_eq!(money_format(&euros(1_250)), "12.50 EUR");

    assert_eq!(money_add(&euros(1_250), &euros(50)), Ok(euros(1_300)));
    assert_eq!(money_subtract(&euros(1_250), &euros(50)), Ok(euros(1_200)));
    for helper in [money_add, money_subtract] {
        assert!(matches!(
            helper(&euros(1), &lei(1)),
            Err(EranosError::Money {
                failure: MoneyFailure::CurrencyMismatch { .. },
                ..
            })
        ));
        assert!(is_unknown_currency(&helper(&unknown_money(), &euros(1))));
        assert!(is_unknown_currency(&helper(&euros(1), &unknown_money())));
    }
    assert!(matches!(
        money_add(&euros(i64::MAX), &euros(1)),
        Err(EranosError::Money {
            failure: MoneyFailure::Overflow,
            ..
        })
    ));
    assert!(matches!(
        money_subtract(&euros(i64::MIN), &euros(1)),
        Err(EranosError::Money {
            failure: MoneyFailure::Overflow,
            ..
        })
    ));
}

/// A rate would be written another way than the library's, or a conversion
/// would round another way, or come back from a bad rate, an unknown
/// currency, or past the range.
#[test]
fn rates_parse_and_convert_as_the_library_does() {
    assert_eq!(exchange_rate_parse("4.970").as_deref(), Ok("4.97"));
    assert!(matches!(
        exchange_rate_parse("0"),
        Err(EranosError::Money {
            failure: MoneyFailure::InvalidExchangeRate,
            ..
        })
    ));
    // 0.05 lei at 0.2 is one euro cent: 0.01 exactly.
    assert_eq!(exchange_rate_convert("0.2", &lei(5), "EUR"), Ok(euros(1)));
    // 0.02 lei at 0.25 is half a euro cent, which rounds away from zero.
    assert_eq!(exchange_rate_convert("0.25", &lei(2), "EUR"), Ok(euros(1)));
    assert!(matches!(
        exchange_rate_convert("rate", &lei(1), "EUR"),
        Err(EranosError::Money {
            failure: MoneyFailure::InvalidExchangeRate,
            ..
        })
    ));
    assert!(is_unknown_currency(&exchange_rate_convert(
        "1",
        &unknown_money(),
        "EUR"
    )));
    assert!(is_unknown_currency(&exchange_rate_convert(
        "1",
        &lei(1),
        "XYZ"
    )));
    assert!(matches!(
        exchange_rate_convert("2", &lei(i64::MAX), "EUR"),
        Err(EranosError::Money {
            failure: MoneyFailure::Overflow,
            ..
        })
    ));
}

/// Percentages and parts would be read or written another way than the
/// library's, or one too large for the bindings' integers would wrap.
#[test]
fn percentages_and_parts_parse_and_format_as_the_library_does() {
    assert_eq!(percentage_parse("12.5 %"), Ok(1_250));
    assert_eq!(percentage_format(1_250), "12.5%");
    // Above 21 474 836.47 %, the basis points no longer fit an i32.
    for refused in [percentage_parse("half"), percentage_parse("30000000")] {
        assert!(
            matches!(
                refused,
                Err(EranosError::Money {
                    failure: MoneyFailure::InvalidPercentage,
                    ..
                })
            ),
            "{refused:?}"
        );
    }

    assert_eq!(parts_parse("0.5"), Ok(50));
    assert_eq!(parts_format(50), "0.5");
    for refused in [parts_parse("half"), parts_parse("30000000")] {
        assert!(
            matches!(
                refused,
                Err(EranosError::Money {
                    failure: MoneyFailure::InvalidParts,
                    ..
                })
            ),
            "{refused:?}"
        );
    }
}

/// The money failure `failure`, with the library's message for `error`.
fn money_failure(failure: MoneyFailure, error: MoneyError) -> EranosError {
    EranosError::Money {
        failure,
        detail: error.to_string(),
    }
}

/// An app that holds amounts as doubles would get other cents in a binding
/// than the library rounds to, or an amount back from NaN, from one past the
/// range, or in an unknown currency.
#[test]
fn amounts_convert_from_doubles_as_the_library_converts_them() {
    assert_eq!(money_from_f64(1.005, "EUR"), Ok(euros(101)));
    assert_eq!(money_from_f64(-2.675, "RON"), Ok(lei(-268)));
    assert!(is_unknown_currency(&money_from_f64(1.0, "XYZ")));
    assert_eq!(
        money_from_f64(f64::NAN, "EUR"),
        Err(money_failure(
            MoneyFailure::NotFinite,
            MoneyError::NotFinite
        ))
    );
    assert_eq!(
        money_from_f64(1e21, "EUR"),
        Err(money_failure(MoneyFailure::Overflow, MoneyError::Overflow))
    );
}

/// A rate, a percentage, or parts held as a double would become other digits
/// in a binding than in the library, a number too large for the bindings'
/// integers would wrap, or NaN would come back as a value.
#[test]
fn rates_and_ratios_convert_from_doubles_as_the_library_converts_them() {
    assert_eq!(exchange_rate_from_f64(25.5).as_deref(), Ok("25.5"));
    assert_eq!(
        exchange_rate_from_f64(0.1 + 0.2).as_deref(),
        Ok("0.30000000000000004")
    );
    assert_eq!(
        exchange_rate_from_f64(0.0),
        Err(money_failure(
            MoneyFailure::InvalidExchangeRate,
            MoneyError::InvalidExchangeRate
        ))
    );

    assert_eq!(percentage_from_f64(0.1 + 0.2), Ok(30));
    assert_eq!(percentage_from_f64(12.345), Ok(1_235));
    // Above 21 474 836.47 %, the basis points no longer fit an i32.
    for value in [-1.0, 30_000_000.0] {
        assert_eq!(
            percentage_from_f64(value),
            Err(money_failure(
                MoneyFailure::InvalidPercentage,
                MoneyError::InvalidPercentage
            )),
            "{value}"
        );
    }

    assert_eq!(parts_from_f64(1.005), Ok(101));
    for value in [0.004, 30_000_000.0] {
        assert_eq!(
            parts_from_f64(value),
            Err(money_failure(
                MoneyFailure::InvalidParts,
                MoneyError::InvalidParts
            )),
            "{value}"
        );
    }

    let not_finite = money_failure(MoneyFailure::NotFinite, MoneyError::NotFinite);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            exchange_rate_from_f64(value),
            Err(not_finite.clone()),
            "{value}"
        );
        assert_eq!(
            percentage_from_f64(value),
            Err(not_finite.clone()),
            "{value}"
        );
        assert_eq!(parts_from_f64(value), Err(not_finite.clone()), "{value}");
    }
}

/// A binding would show a percentage or parts otherwise than the library
/// writes them, or, holding a number the library refuses, would fail to show
/// it, or fall back to a text of its own.
#[test]
fn percentages_and_parts_format_any_number_as_the_library_writes_them() {
    // Every number the library holds, as its text form writes it, up to the
    // largest an i32 holds.
    for number in [
        0,
        1,
        5,
        10,
        50,
        100,
        105,
        150,
        1_250,
        3_333,
        10_000,
        i32::MAX,
    ] {
        let unsigned = u32::try_from(number).unwrap();
        assert_eq!(
            percentage_format(number),
            library::Percentage::from_basis_points(unsigned)
                .unwrap()
                .to_string()
        );
        if 0 < number {
            assert_eq!(
                parts_format(number),
                library::Parts::from_hundredths(unsigned)
                    .unwrap()
                    .to_string()
            );
        }
    }
    // Zero parts and negative numbers, which the library refuses, written
    // the same way.
    assert_eq!(parts_format(0), "0");
    assert_eq!(percentage_format(-1), "-0.01%");
    assert_eq!(percentage_format(-1_250), "-12.5%");
    assert_eq!(percentage_format(i32::MIN), "-21474836.48%");
    assert_eq!(parts_format(-50), "-0.5");
    assert_eq!(parts_format(i32::MIN), "-21474836.48");
}

/// A binding that checks a percentage or parts before it builds a split
/// would let through a number the library refuses, or refuse one it holds.
#[test]
fn percentages_and_parts_check_as_the_library_does() {
    for accepted in [0, 1, 1_250, i32::MAX] {
        assert_eq!(percentage_check(accepted), Ok(()));
    }
    for refused in [-1, i32::MIN] {
        assert!(
            matches!(
                percentage_check(refused),
                Err(EranosError::Money {
                    failure: MoneyFailure::InvalidPercentage,
                    ..
                })
            ),
            "{refused}"
        );
    }
    for accepted in [1, 50, i32::MAX] {
        assert_eq!(parts_check(accepted), Ok(()));
    }
    for refused in [0, -1, i32::MIN] {
        assert!(
            matches!(
                parts_check(refused),
                Err(EranosError::Money {
                    failure: MoneyFailure::InvalidParts,
                    ..
                })
            ),
            "{refused}"
        );
    }
}

/// A quantity would be read, built, or converted from a double another way
/// than the library's, or one the library refuses, or too large for the
/// bindings' 64-bit integers, would come back as a number an item then
/// carries.
#[test]
fn quantities_parse_and_convert_as_the_library_does() {
    let invalid = money_failure(MoneyFailure::InvalidQuantity, MoneyError::InvalidQuantity);
    assert_eq!(quantity_parse("0.452"), Ok(4_520));
    assert_eq!(quantity_parse(" 3 "), Ok(30_000));
    assert_eq!(quantity_parse("922337203685477.5807"), Ok(i64::MAX));
    for text in ["0", "0.00005", "-1", "1e2", "half", "922337203685477.5808"] {
        assert_eq!(quantity_parse(text), Err(invalid.clone()), "{text}");
    }

    assert_eq!(quantity_from_whole(1), Ok(10_000));
    assert_eq!(quantity_from_whole(3), Ok(30_000));
    assert_eq!(
        quantity_from_whole(922_337_203_685_477),
        Ok(9_223_372_036_854_770_000)
    );
    for units in [0, -1, i64::MIN, 922_337_203_685_478, i64::MAX] {
        assert_eq!(quantity_from_whole(units), Err(invalid.clone()), "{units}");
    }

    assert_eq!(quantity_from_f64(0.45205), Ok(4_521));
    assert_eq!(quantity_from_f64(1.5), Ok(15_000));
    for value in [0.00004, -1.0, 1e21] {
        assert_eq!(quantity_from_f64(value), Err(invalid.clone()), "{value}");
    }
    let not_finite = money_failure(MoneyFailure::NotFinite, MoneyError::NotFinite);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(quantity_from_f64(value), Err(not_finite.clone()), "{value}");
    }
}

/// A binding would show a quantity otherwise than the library writes it,
/// or, holding a number the library refuses, would fail to show it or fall
/// back to a text of its own; and a check before an item is built would let
/// through a number the library refuses, or refuse one it holds.
#[test]
fn quantities_format_any_number_and_check_as_the_library_does() {
    for number in [1, 10, 4_520, 10_000, 15_000, 20_500, i64::MAX] {
        let unsigned = u64::try_from(number).unwrap();
        assert_eq!(
            quantity_format(number),
            library::Quantity::from_ten_thousandths(unsigned)
                .unwrap()
                .to_string()
        );
        assert_eq!(quantity_check(number), Ok(()));
    }
    // Zero and negative numbers, which the library refuses, written the
    // same way.
    assert_eq!(quantity_format(0), "0");
    assert_eq!(quantity_format(-4_520), "-0.452");
    assert_eq!(quantity_format(i64::MIN), "-922337203685477.5808");
    let invalid = money_failure(MoneyFailure::InvalidQuantity, MoneyError::InvalidQuantity);
    for refused in [0, -1, i64::MIN] {
        assert_eq!(quantity_check(refused), Err(invalid.clone()), "{refused}");
    }
}

/// Dates, times, and language tags would be read or written another way
/// than the library's, or one the library refuses would come back as a
/// value.
#[test]
fn dates_times_and_languages_parse_and_format_as_the_library_does() {
    let date = CalendarDate {
        year: 2026,
        month: 9,
        day: 23,
    };
    assert_eq!(calendar_date_parse("2026-09-23"), Ok(date));
    assert_eq!(calendar_date_format(date), "2026-09-23");
    assert!(matches!(
        calendar_date_parse("2026-02-30"),
        Err(EranosError::Calendar {
            failure: CalendarFailure::InvalidDate,
            ..
        })
    ));

    let time = TimeOfDay {
        hour: 18,
        minute: 30,
    };
    assert_eq!(time_of_day_parse("18:30"), Ok(time));
    assert_eq!(time_of_day_format(time), "18:30");
    assert!(matches!(
        time_of_day_parse("25:00"),
        Err(EranosError::Calendar {
            failure: CalendarFailure::InvalidTime,
            ..
        })
    ));

    assert_eq!(language_parse("pt-pt").as_deref(), Ok("pt-PT"));
    assert!(matches!(
        language_parse("not a tag"),
        Err(EranosError::Language {
            failure: LanguageFailure::InvalidLanguage,
            ..
        })
    ));
}

/// A binding would show a date or a time otherwise than the library writes
/// it, or, holding fields the library refuses, would fail to show them, or
/// fall back to a text of its own.
#[test]
fn dates_and_times_format_any_fields_as_the_library_writes_them() {
    // Every day of a leap year, and the bounds, as the library writes them.
    let days = (1..=12)
        .flat_map(|month| (1..=31).map(move |day| calendar::CalendarDate::new(2028, month, day)))
        .filter_map(Result::ok)
        .chain([
            calendar::CalendarDate::new(1, 1, 1).unwrap(),
            calendar::CalendarDate::new(9999, 12, 31).unwrap(),
        ]);
    for day in days {
        assert_eq!(
            calendar_date_format(CalendarDate::from_library(day)),
            day.to_string()
        );
    }
    // Every minute of the day, as the library writes it.
    for hour in 0..=23 {
        for minute in 0..=59 {
            let time = calendar::TimeOfDay::new(hour, minute).unwrap();
            assert_eq!(
                time_of_day_format(TimeOfDay::from_library(time)),
                time.to_string()
            );
        }
    }
    // Fields the library refuses, written the same way.
    assert_eq!(
        calendar_date_format(CalendarDate {
            year: 2026,
            month: 2,
            day: 30,
        }),
        "2026-02-30"
    );
    assert_eq!(
        calendar_date_format(CalendarDate {
            year: 0,
            month: 13,
            day: 0,
        }),
        "0000-13-00"
    );
    assert_eq!(
        calendar_date_format(CalendarDate {
            year: -1,
            month: 100,
            day: -5,
        }),
        "-001-100--5"
    );
    assert_eq!(
        time_of_day_format(TimeOfDay {
            hour: 24,
            minute: 60,
        }),
        "24:60"
    );
    assert_eq!(
        time_of_day_format(TimeOfDay {
            hour: -1,
            minute: i32::MAX,
        }),
        "-1:2147483647"
    );
}

/// A binding that checks a date or a time before it builds an entry would
/// let through one the library refuses, or refuse one it holds.
#[test]
fn dates_and_times_check_as_the_library_does() {
    for accepted in [
        CalendarDate {
            year: 2028,
            month: 2,
            day: 29,
        },
        CalendarDate {
            year: 1,
            month: 1,
            day: 1,
        },
        CalendarDate {
            year: 9999,
            month: 12,
            day: 31,
        },
    ] {
        assert_eq!(calendar_date_check(accepted), Ok(()));
    }
    // A day the month lacks, a year outside 1 to 9999, and numbers too large
    // for the library's fields.
    for refused in [
        CalendarDate {
            year: 2026,
            month: 2,
            day: 29,
        },
        CalendarDate {
            year: 0,
            month: 1,
            day: 1,
        },
        CalendarDate {
            year: 10_000,
            month: 1,
            day: 1,
        },
        CalendarDate {
            year: 2026,
            month: 256,
            day: 1,
        },
        CalendarDate {
            year: -1,
            month: 1,
            day: 1,
        },
    ] {
        assert!(
            matches!(
                calendar_date_check(refused),
                Err(EranosError::Calendar {
                    failure: CalendarFailure::InvalidDate,
                    ..
                })
            ),
            "{refused:?}"
        );
    }
    for accepted in [
        TimeOfDay { hour: 0, minute: 0 },
        TimeOfDay {
            hour: 23,
            minute: 59,
        },
    ] {
        assert_eq!(time_of_day_check(accepted), Ok(()));
    }
    for refused in [
        TimeOfDay {
            hour: 24,
            minute: 0,
        },
        TimeOfDay {
            hour: 0,
            minute: 60,
        },
        TimeOfDay {
            hour: -1,
            minute: 0,
        },
        TimeOfDay {
            hour: 0,
            minute: 256,
        },
    ] {
        assert!(
            matches!(
                time_of_day_check(refused),
                Err(EranosError::Calendar {
                    failure: CalendarFailure::InvalidTime,
                    ..
                })
            ),
            "{refused:?}"
        );
    }
}

/// A balance would be written another way than the library's, and the
/// currencies would reach the apps other than ISO 4217 lists them.
#[test]
fn balances_and_currencies_read_as_the_library_writes_them() {
    assert_eq!(
        balance_format(&Balance::GetsBack {
            amount: euros(1_200)
        }),
        "gets back 12.00 EUR"
    );
    assert_eq!(
        balance_format(&Balance::Owes { amount: euros(300) }),
        "owes 3.00 EUR"
    );
    assert_eq!(balance_format(&Balance::Settled), "settled");

    let euro = currency_details("EUR").unwrap();
    assert_eq!(euro.name, "Euro");
    assert!(is_unknown_currency(&currency_details("XYZ")));
    let all = currencies();
    assert_eq!(all.len(), CurrencyCode::all().len());
    assert!(all.contains(&euro));
    assert!(all.windows(2).all(|pair| pair[0].code < pair[1].code));
}

/// The text the bindings write for an amount or a balance would drift from
/// the library's text form of it, for a currency, a number of decimal
/// places, a sign, or an amount at either end of the range.
#[test]
fn amounts_and_balances_format_as_the_library_writes_them() {
    let mut decimal_places = BTreeSet::new();
    for &currency in CurrencyCode::all() {
        decimal_places.insert(currency.decimal_places());
        for minor_units in [0, 1, -1, 5, -5, 1_234, -1_234, i64::MIN, i64::MAX] {
            let amount = library::Money::from_minor_units(minor_units, currency);
            let record = Money::from_library(amount);
            assert_eq!(money_format(&record), amount.to_string());
            for balance in [
                ledger::Balance::Owes(amount),
                ledger::Balance::Settled,
                ledger::Balance::GetsBack(amount),
            ] {
                assert_eq!(
                    balance_format(&Balance::from_balance(balance)),
                    balance.to_string()
                );
            }
        }
    }
    // The currencies cover every number of decimal places ISO 4217 gives.
    assert_eq!(decimal_places, BTreeSet::from([0, 2, 3, 4]));
}

/// A record whose currency has more decimal places than an `i64` has
/// digits would overflow while it is written, or fail where the apps show
/// it, though nothing about it can fail.
#[test]
fn an_amount_with_any_number_of_decimal_places_formats() {
    let most = Money {
        minor_units: -5,
        currency: unlisted(u8::MAX),
    };
    assert_eq!(money_format(&most), format!("-0.{}5 XYZ", "0".repeat(254)));
    let past_an_i64 = Money {
        minor_units: i64::MIN,
        currency: unlisted(19),
    };
    assert_eq!(money_format(&past_an_i64), "-0.9223372036854775808 XYZ");
    assert_eq!(
        balance_format(&Balance::GetsBack {
            amount: past_an_i64
        }),
        "gets back -0.9223372036854775808 XYZ"
    );
}

/// Contact and account details would be kept in another form than the
/// library's, or one the library refuses would come back as a value.
#[test]
fn contact_and_account_details_parse_as_the_library_does() {
    assert_eq!(
        email_parse(" ana@example.com ").as_deref(),
        Ok("ana@example.com")
    );
    assert!(matches!(
        email_parse("ana"),
        Err(EranosError::Contact {
            failure: ContactFailure::InvalidEmail,
            ..
        })
    ));
    assert_eq!(
        phone_number_parse("+40 712 345 678"),
        Ok(PhoneNumber {
            e164: "+40712345678".to_owned(),
            international: "+40 712 345 678".to_owned(),
        })
    );
    assert!(matches!(
        phone_number_parse("0712345678"),
        Err(EranosError::Contact {
            failure: ContactFailure::InvalidPhoneNumber,
            ..
        })
    ));
    assert_eq!(
        iban_parse("ro49 aaaa 1b31 0075 9384 0000"),
        Ok(Iban {
            iban: "RO49AAAA1B31007593840000".to_owned(),
            country_code: "RO".to_owned(),
        })
    );
    for refused in [iban_parse("RO49AAAA1B31007593840001"), iban_parse("RO49")] {
        assert!(
            matches!(
                refused,
                Err(EranosError::PaymentAccount {
                    failure: PaymentAccountFailure::InvalidIban { .. },
                    ..
                })
            ),
            "{refused:?}"
        );
    }
    assert_eq!(username_parse(" ana.pays ").as_deref(), Ok("ana.pays"));
    assert!(matches!(
        username_parse(" "),
        Err(EranosError::PaymentAccount {
            failure: PaymentAccountFailure::EmptyUsername,
            ..
        })
    ));
}

/// An app that left an expense without a tip would give it a tip, or one
/// shared another way than the library's own expense without a tip.
#[test]
fn the_no_tip_rule_is_zero_per_cent_shared_proportionally() {
    let none = tip_rule_none();
    assert_eq!(
        none,
        TipRule {
            value: TipValue::Percentage { basis_points: 0 },
            distribution: TipDistribution::Proportional,
        }
    );
    assert_eq!(none.to_library(), Ok(expenses::TipRule::NONE));
}

/// The tip and the total the apps show for an expense would differ from
/// what the group records when it adds the expense.
#[test]
fn an_expense_derives_its_tip_and_total_as_the_group_does() {
    assert_eq!(expense_tip_amount(&dinner()), Ok(euros(300)));
    assert_eq!(expense_total(&dinner()), Ok(euros(3_300)));
    let untipped = Expense {
        tip: tip_rule_none(),
        ..dinner()
    };
    assert_eq!(expense_tip_amount(&untipped), Ok(euros(0)));
    assert_eq!(expense_total(&untipped), Ok(euros(3_000)));

    let unknown = Expense {
        cost: Cost::Local {
            amount: unknown_money(),
        },
        ..dinner()
    };
    assert!(is_unknown_currency(&expense_tip_amount(&unknown)));
    assert!(is_unknown_currency(&expense_total(&unknown)));
    let negative = Expense {
        tip: TipRule {
            value: TipValue::Percentage { basis_points: -1 },
            distribution: TipDistribution::Equal,
        },
        ..dinner()
    };
    for refused in [expense_tip_amount(&negative), expense_total(&negative)] {
        assert!(
            matches!(
                refused,
                Err(EranosError::Money {
                    failure: MoneyFailure::InvalidPercentage,
                    ..
                })
            ),
            "{refused:?}"
        );
    }
    let fixed_in_lei = Expense {
        tip: TipRule {
            value: TipValue::Fixed { amount: lei(100) },
            distribution: TipDistribution::Equal,
        },
        ..dinner()
    };
    assert!(matches!(
        expense_tip_amount(&fixed_in_lei),
        Err(EranosError::Expense {
            failure: ExpenseFailure::CurrencyMismatch { .. },
            ..
        })
    ));
    let past_the_range = Expense {
        cost: Cost::Local {
            amount: euros(i64::MAX),
        },
        tip: TipRule {
            value: TipValue::Fixed { amount: euros(1) },
            distribution: TipDistribution::Equal,
        },
        ..dinner()
    };
    assert!(matches!(
        expense_total(&past_the_range),
        Err(EranosError::Money {
            failure: MoneyFailure::Overflow,
            ..
        })
    ));
}

/// The line total the apps show for an item would differ from the one the
/// library adds up against the expense's amount: rounded another way,
/// multiplied as a double, or refused for a free line; and an item the
/// library refuses would show a line total.
#[test]
fn an_item_derives_its_line_total_as_the_library_does() {
    // 0.335 at 1.00 is 33.5 cents.
    let olives = ExpenseItem {
        name: "Olives".to_owned(),
        unit_price: euros(100),
        split: equal(&[member(1), member(2)]),
        quantity: 3_350,
        discount: None,
    };
    assert_eq!(expense_item_line_total(&olives), Ok(euros(34)));
    let cheese = ExpenseItem {
        unit_price: euros(4_200),
        quantity: 3_500,
        ..olives.clone()
    };
    assert_eq!(expense_item_line_total(&cheese), Ok(euros(1_470)));
    let two = ExpenseItem {
        quantity: 20_000,
        ..olives.clone()
    };
    assert_eq!(expense_item_line_total(&two), Ok(euros(200)));

    // A ten-thousandth of a cent rounds to zero: a free line.
    let free = ExpenseItem {
        quantity: 1,
        ..olives.clone()
    };
    assert_eq!(expense_item_line_total(&free), Ok(euros(0)));

    // No quantity, amounts that cannot add up to a free line's zero, and a
    // line past the range.
    assert_eq!(
        expense_item_line_total(&ExpenseItem {
            quantity: 0,
            ..olives.clone()
        }),
        Err(money_failure(
            MoneyFailure::InvalidQuantity,
            MoneyError::InvalidQuantity
        ))
    );
    assert_eq!(
        expense_item_line_total(&ExpenseItem {
            split: Split::Amounts {
                shares: vec![MemberAmount {
                    member: member(1),
                    amount: euros(1),
                }],
            },
            ..free
        }),
        Err(EranosError::from(
            eranos::error::ExpenseError::AmountsDoNotAddUp {
                expected: eranos::Money::from_minor_units(0, CurrencyCode::EUR),
                actual: eranos::Money::from_minor_units(1, CurrencyCode::EUR),
            }
        ))
    );
    assert_eq!(
        expense_item_line_total(&ExpenseItem {
            unit_price: euros(i64::MAX),
            ..two
        }),
        Err(EranosError::from(eranos::error::ExpenseError::Overflow))
    );
    assert!(is_unknown_currency(&expense_item_line_total(
        &ExpenseItem {
            unit_price: unknown_money(),
            ..olives
        }
    )));
}

/// The receipt the apps show for a discounted item would differ from the
/// one the library adds up: a percentage rounded another way, a discount
/// off each unit multiplied as one off the line, or columns that do not add
/// up; and an item whose discount the library refuses would show totals.
#[test]
fn an_item_derives_its_discount_as_the_library_does() {
    // Three lemonades at 3.33, half off.
    let lemonades = |scope| ExpenseItem {
        name: "Lemonade".to_owned(),
        unit_price: euros(333),
        split: equal(&[member(1), member(2)]),
        quantity: 30_000,
        discount: Some(Discount {
            value: DiscountValue::Percentage {
                basis_points: 5_000,
            },
            scope,
        }),
    };
    // Off the line: 5.00 off 9.99.
    let off_the_line = lemonades(DiscountScope::Line);
    assert_eq!(
        expense_item_line_total_before_discount(&off_the_line),
        Ok(euros(999))
    );
    assert_eq!(expense_item_discount_amount(&off_the_line), Ok(euros(500)));
    assert_eq!(expense_item_line_total(&off_the_line), Ok(euros(499)));
    // Off each unit: 1.67 off each of three.
    let off_each_unit = lemonades(DiscountScope::Unit);
    assert_eq!(
        expense_item_line_total_before_discount(&off_each_unit),
        Ok(euros(999))
    );
    assert_eq!(expense_item_discount_amount(&off_each_unit), Ok(euros(501)));
    assert_eq!(expense_item_line_total(&off_each_unit), Ok(euros(498)));
    // Without a discount nothing is taken off.
    let full_price = ExpenseItem {
        discount: None,
        ..off_the_line.clone()
    };
    assert_eq!(expense_item_discount_amount(&full_price), Ok(euros(0)));
    assert_eq!(
        expense_item_line_total_before_discount(&full_price),
        expense_item_line_total(&full_price)
    );

    // A discount above the line refuses every total.
    let above = ExpenseItem {
        discount: Some(Discount {
            value: DiscountValue::Fixed {
                amount: euros(1_000),
            },
            scope: DiscountScope::Line,
        }),
        ..off_the_line
    };
    let refused = Err(EranosError::from(
        eranos::error::ExpenseError::DiscountAbovePrice,
    ));
    assert_eq!(expense_item_line_total_before_discount(&above), refused);
    assert_eq!(expense_item_discount_amount(&above), refused);
    assert_eq!(expense_item_line_total(&above), refused);
}

/// The members the apps list for an expense or a split would differ from
/// the ones it involves, or come in another order than the library's.
#[test]
fn an_expense_and_a_split_list_their_members() {
    let everyone = [member(1), member(2), member(3)];
    assert_eq!(expense_members(&dinner()), Ok(everyone.to_vec()));
    let itemised = Expense {
        split: ExpenseSplit::Itemised {
            items: vec![ExpenseItem {
                name: "Soup".to_owned(),
                unit_price: euros(3_000),
                split: equal(&[member(3), member(2)]),
                quantity: 10_000,
                discount: None,
            }],
        },
        ..dinner()
    };
    assert_eq!(expense_members(&itemised), Ok(everyone.to_vec()));
    let listed = equal(&[member(3), member(2)]);
    assert_eq!(split_members(&listed), Ok(vec![member(3), member(2)]));

    let unreadable = equal(&[" nobody".to_owned()]);
    let refused = [
        expense_members(&Expense {
            payers: unreadable.clone(),
            ..dinner()
        }),
        expense_members(&Expense {
            split: ExpenseSplit::Whole {
                split: unreadable.clone(),
            },
            ..dinner()
        }),
        split_members(&unreadable),
    ];
    for result in refused {
        assert!(
            matches!(
                result,
                Err(EranosError::Id {
                    failure: IdFailure::SurroundingWhitespace,
                    ..
                })
            ),
            "{result:?}"
        );
    }
    let bad_item = Expense {
        split: ExpenseSplit::Itemised {
            items: vec![ExpenseItem {
                name: "Soup".to_owned(),
                unit_price: euros(3_000),
                split: equal(&[member(2)]),
                quantity: 0,
                discount: None,
            }],
        },
        ..dinner()
    };
    assert!(matches!(
        expense_members(&bad_item),
        Err(EranosError::Money {
            failure: MoneyFailure::InvalidQuantity,
            ..
        })
    ));
}
