//! The value helpers are the only way the apps parse and write values, so
//! every text a binding shows is the library's. A helper that formatted a
//! value its own way, let through text the library refuses, or wrapped a
//! number too large for the bindings' integers would show the apps other
//! values than the library holds.

use eranos::MemberId;

use super::*;
use crate::expense::{Item, MemberAmount, Tip, TipDistribution, TipValue};
use crate::money::Cost;

fn euros(minor_units: i64) -> Money {
    Money {
        currency: "EUR".to_owned(),
        minor_units,
    }
}

fn lei(minor_units: i64) -> Money {
    Money {
        currency: "RON".to_owned(),
        minor_units,
    }
}

fn unknown_money() -> Money {
    Money {
        currency: "XYZ".to_owned(),
        minor_units: 1,
    }
}

fn member(byte: u8) -> String {
    MemberId::from_bytes([byte; 16]).to_string()
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
        id: None,
        cost: Cost::Local {
            amount: euros(3_000),
        },
        date: CalendarDate {
            day: 23,
            month: 9,
            year: 2026,
        },
        name: "Dinner".to_owned(),
        payers: Split::Amounts {
            shares: vec![MemberAmount {
                amount: euros(3_300),
                member: member(1),
            }],
        },
        split: ExpenseSplit::Whole {
            split: equal(&[member(3), member(2)]),
        },
        category: None,
        description: None,
        time: None,
        tip: Some(Tip {
            value: TipValue::Percentage {
                basis_points: 1_000,
            },
            distribution: TipDistribution::Proportional,
        }),
    }
}

fn is_unknown_currency<T>(result: &Result<T, EranosError>) -> bool {
    matches!(result, Err(EranosError::UnknownCurrency { .. }))
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
        Err(EranosError::InvalidAmount { .. })
    ));
    assert!(matches!(
        money_parse("1.234", "EUR"),
        Err(EranosError::TooManyDecimals { .. })
    ));
    assert_eq!(money_format(&euros(1_250)).as_deref(), Ok("12.50 EUR"));
    assert!(is_unknown_currency(&money_format(&unknown_money())));

    assert_eq!(money_add(&euros(1_250), &euros(50)), Ok(euros(1_300)));
    assert_eq!(money_subtract(&euros(1_250), &euros(50)), Ok(euros(1_200)));
    for helper in [money_add, money_subtract] {
        assert!(matches!(
            helper(&euros(1), &lei(1)),
            Err(EranosError::MoneyCurrencyMismatch { .. })
        ));
        assert!(is_unknown_currency(&helper(&unknown_money(), &euros(1))));
        assert!(is_unknown_currency(&helper(&euros(1), &unknown_money())));
    }
    assert!(matches!(
        money_add(&euros(i64::MAX), &euros(1)),
        Err(EranosError::MoneyOverflow { .. })
    ));
    assert!(matches!(
        money_subtract(&euros(i64::MIN), &euros(1)),
        Err(EranosError::MoneyOverflow { .. })
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
        Err(EranosError::InvalidExchangeRate { .. })
    ));
    // 0.05 lei at 0.2 is one euro cent: 0.01 exactly.
    assert_eq!(exchange_rate_convert("0.2", &lei(5), "EUR"), Ok(euros(1)));
    // 0.02 lei at 0.25 is half a euro cent, which rounds away from zero.
    assert_eq!(exchange_rate_convert("0.25", &lei(2), "EUR"), Ok(euros(1)));
    assert!(matches!(
        exchange_rate_convert("rate", &lei(1), "EUR"),
        Err(EranosError::InvalidExchangeRate { .. })
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
        Err(EranosError::MoneyOverflow { .. })
    ));
}

/// Percentages and parts would be read or written another way than the
/// library's, or one too large for the bindings' integers would wrap.
#[test]
fn percentages_and_parts_parse_and_format_as_the_library_does() {
    assert_eq!(percentage_parse("12.5 %"), Ok(1_250));
    assert_eq!(percentage_format(1_250).as_deref(), Ok("12.5%"));
    // Above 21 474 836.47 %, the basis points no longer fit an i32.
    for refused in [percentage_parse("half"), percentage_parse("30000000")] {
        assert!(
            matches!(refused, Err(EranosError::InvalidPercentage { .. })),
            "{refused:?}"
        );
    }
    assert!(matches!(
        percentage_format(-1),
        Err(EranosError::InvalidPercentage { .. })
    ));

    assert_eq!(parts_parse("0.5"), Ok(50));
    assert_eq!(parts_format(50).as_deref(), Ok("0.5"));
    for refused in [parts_parse("half"), parts_parse("30000000")] {
        assert!(
            matches!(refused, Err(EranosError::InvalidParts { .. })),
            "{refused:?}"
        );
    }
    for refused in [parts_format(-1), parts_format(0)] {
        assert!(
            matches!(refused, Err(EranosError::InvalidParts { .. })),
            "{refused:?}"
        );
    }
}

/// Dates, times, and language tags would be read or written another way
/// than the library's, or one the library refuses would come back as a
/// value.
#[test]
fn dates_times_and_languages_parse_and_format_as_the_library_does() {
    let date = CalendarDate {
        day: 23,
        month: 9,
        year: 2026,
    };
    assert_eq!(calendar_date_parse("2026-09-23"), Ok(date));
    assert_eq!(calendar_date_format(date).as_deref(), Ok("2026-09-23"));
    assert!(matches!(
        calendar_date_parse("2026-02-30"),
        Err(EranosError::InvalidDate { .. })
    ));
    assert!(matches!(
        calendar_date_format(CalendarDate {
            day: 30,
            month: 2,
            ..date
        }),
        Err(EranosError::InvalidDate { .. })
    ));

    let time = TimeOfDay {
        hour: 18,
        minute: 30,
    };
    assert_eq!(time_of_day_parse("18:30"), Ok(time));
    assert_eq!(time_of_day_format(time).as_deref(), Ok("18:30"));
    assert!(matches!(
        time_of_day_parse("25:00"),
        Err(EranosError::InvalidTime { .. })
    ));
    assert!(matches!(
        time_of_day_format(TimeOfDay { hour: 25, ..time }),
        Err(EranosError::InvalidTime { .. })
    ));

    assert_eq!(language_parse("pt-pt").as_deref(), Ok("pt-PT"));
    assert!(matches!(
        language_parse("not a tag"),
        Err(EranosError::InvalidLanguage { .. })
    ));
}

/// A balance would be written another way than the library's, and the
/// currencies would reach the apps other than ISO 4217 lists them.
#[test]
fn balances_and_currencies_read_as_the_library_writes_them() {
    let balances = [
        (
            Balance::GetsBack {
                amount: euros(1_200),
            },
            "gets back 12.00 EUR",
        ),
        (Balance::Owes { amount: euros(300) }, "owes 3.00 EUR"),
        (Balance::Settled, "settled"),
    ];
    for (balance, text) in balances {
        assert_eq!(balance_format(&balance).as_deref(), Ok(text));
    }
    for balance in [
        Balance::GetsBack {
            amount: unknown_money(),
        },
        Balance::Owes {
            amount: unknown_money(),
        },
    ] {
        assert!(is_unknown_currency(&balance_format(&balance)));
    }

    let euro = currency_details("EUR").unwrap();
    assert_eq!(euro.name, "Euro");
    assert!(is_unknown_currency(&currency_details("XYZ")));
    let all = currencies();
    assert_eq!(all.len(), eranos::Currency::all().len());
    assert!(all.contains(&euro));
    assert!(all.windows(2).all(|pair| pair[0].code < pair[1].code));
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
        Err(EranosError::InvalidEmail { .. })
    ));
    assert_eq!(
        phone_number_parse("+40 712 345 678").as_deref(),
        Ok("+40712345678")
    );
    assert_eq!(
        phone_number_international("+40712345678").as_deref(),
        Ok("+40 712 345 678")
    );
    for refused in [
        phone_number_parse("0712345678"),
        phone_number_international("0712345678"),
    ] {
        assert!(
            matches!(refused, Err(EranosError::InvalidPhoneNumber { .. })),
            "{refused:?}"
        );
    }
    assert_eq!(
        iban_parse("ro49 aaaa 1b31 0075 9384 0000").as_deref(),
        Ok("RO49AAAA1B31007593840000")
    );
    assert_eq!(
        iban_country_code("RO49AAAA1B31007593840000").as_deref(),
        Ok("RO")
    );
    for refused in [
        iban_parse("RO49AAAA1B31007593840001"),
        iban_country_code("RO49"),
    ] {
        assert!(
            matches!(refused, Err(EranosError::InvalidIban { .. })),
            "{refused:?}"
        );
    }
    assert_eq!(username_parse(" ana.pays ").as_deref(), Ok("ana.pays"));
    assert!(matches!(
        username_parse(" "),
        Err(EranosError::EmptyUsername { .. })
    ));
}

/// The tip and the total the apps show for an expense would differ from
/// what the group records when it adds the expense.
#[test]
fn an_expense_derives_its_tip_and_total_as_the_group_does() {
    assert_eq!(expense_tip_amount(&dinner()), Ok(euros(300)));
    assert_eq!(expense_total(&dinner()), Ok(euros(3_300)));
    let untipped = Expense {
        tip: None,
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
        tip: Some(Tip {
            value: TipValue::Percentage { basis_points: -1 },
            distribution: TipDistribution::Equal,
        }),
        ..dinner()
    };
    for refused in [expense_tip_amount(&negative), expense_total(&negative)] {
        assert!(
            matches!(refused, Err(EranosError::InvalidPercentage { .. })),
            "{refused:?}"
        );
    }
    let fixed_in_lei = Expense {
        tip: Some(Tip {
            value: TipValue::Fixed { amount: lei(100) },
            distribution: TipDistribution::Equal,
        }),
        ..dinner()
    };
    assert!(matches!(
        expense_tip_amount(&fixed_in_lei),
        Err(EranosError::ExpenseCurrencyMismatch { .. })
    ));
    let past_the_range = Expense {
        cost: Cost::Local {
            amount: euros(i64::MAX),
        },
        tip: Some(Tip {
            value: TipValue::Fixed { amount: euros(1) },
            distribution: TipDistribution::Equal,
        }),
        ..dinner()
    };
    assert!(matches!(
        expense_total(&past_the_range),
        Err(EranosError::MoneyOverflow { .. })
    ));
}

/// The members the apps list for an expense or a split would differ from
/// the ones it involves, or come in another order than the library's.
#[test]
fn an_expense_and_a_split_list_their_members() {
    let everyone = [member(1), member(2), member(3)];
    assert_eq!(expense_members(&dinner()), Ok(everyone.to_vec()));
    let itemised = Expense {
        split: ExpenseSplit::Itemised {
            items: vec![Item {
                name: "Soup".to_owned(),
                split: equal(&[member(3), member(2)]),
                unit_price: euros(3_000),
                quantity: 1,
            }],
        },
        ..dinner()
    };
    assert_eq!(expense_members(&itemised), Ok(everyone.to_vec()));
    let listed = equal(&[member(3), member(2)]);
    assert_eq!(split_members(&listed), Ok(vec![member(3), member(2)]));

    let unreadable = equal(&["nobody".to_owned()]);
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
            matches!(result, Err(EranosError::InvalidId { .. })),
            "{result:?}"
        );
    }
    let bad_item = Expense {
        split: ExpenseSplit::Itemised {
            items: vec![Item {
                name: "Soup".to_owned(),
                split: equal(&[member(2)]),
                unit_price: euros(3_000),
                quantity: 0,
            }],
        },
        ..dinner()
    };
    assert!(matches!(
        expense_members(&bad_item),
        Err(EranosError::ZeroQuantity { .. })
    ));
}
