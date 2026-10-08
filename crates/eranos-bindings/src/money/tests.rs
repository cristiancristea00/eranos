//! Amounts, costs, and rates cross the bindings as currency details,
//! codes, integers, and decimals. One converted back and forth would change
//! its currency, its minor units, or its rate, and an unknown currency, a
//! currency with details of its own, or a bad rate would reach the library.

use super::*;
use crate::MoneyFailure;

fn euro() -> CurrencyDetails {
    CurrencyDetails::from_library(library::CurrencyCode::EUR)
}

fn euros(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: euro(),
    }
}

fn unknown() -> Money {
    Money {
        minor_units: 1,
        currency: CurrencyDetails {
            code: "XYZ".to_owned(),
            name: "Unknown".to_owned(),
            numeric_code: 999,
            decimal_places: 2,
        },
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

/// An amount would come back in another currency or with other minor
/// units, or one in an unknown currency would reach the library.
#[test]
fn amounts_convert_both_ways_and_refuse_unknown_currencies() {
    assert_eq!(
        euros(1_250).to_library().map(Money::from_library),
        Ok(euros(1_250))
    );
    assert!(is_unknown_currency(&unknown().to_library()));
}

/// The details of every currency would come back other than the library
/// holds them, or an amount in one would come back in another currency.
#[test]
fn the_details_of_every_currency_convert_both_ways() {
    for &currency in library::CurrencyCode::all() {
        let details = CurrencyDetails::from_library(currency);
        assert_eq!(details.code, currency.code());
        assert_eq!(details.name, currency.name());
        assert_eq!(details.numeric_code, i32::from(currency.numeric_code()));
        assert_eq!(details.decimal_places, currency.decimal_places());
        assert_eq!(details.to_library(), Ok(currency));
        let amount = library::Money::from_minor_units(-1_234, currency);
        assert_eq!(Money::from_library(amount).to_library(), Ok(amount));
    }
}

/// An amount whose record gives its currency details other than the
/// library's would reach the library as the library's currency of its code,
/// so the apps would show it with other decimal places, another name, or
/// another numeric code than the group holds.
#[test]
fn details_that_differ_from_the_library_are_refused() {
    let altered = [
        CurrencyDetails {
            code: "eur".to_owned(),
            ..euro()
        },
        CurrencyDetails {
            name: "Euros".to_owned(),
            ..euro()
        },
        CurrencyDetails {
            numeric_code: 840,
            ..euro()
        },
        CurrencyDetails {
            decimal_places: 3,
            ..euro()
        },
    ];
    for currency in altered {
        assert!(is_unknown_currency(&currency.to_library()), "{currency:?}");
        let amount = Money {
            minor_units: 100,
            currency,
        };
        assert!(is_unknown_currency(&amount.to_library()), "{amount:?}");
    }
    assert!(is_unknown_currency(&unknown().currency.to_library()));
}

/// A cost would come back without its rate, a foreign cost would reach the
/// library as a local one, or a bad rate or currency would get through.
#[test]
fn costs_convert_both_ways_and_refuse_bad_parts() {
    let converted = Cost::Converted {
        amount: Money {
            minor_units: 5_000,
            currency: CurrencyDetails::from_library(library::CurrencyCode::RON),
        },
        rate: "0.2".to_owned(),
    };
    assert_eq!(converted.to_library().map(Cost::from_cost), Ok(converted));
    let local = Cost::Local { amount: euros(900) };
    assert_eq!(local.to_library().map(Cost::from_cost), Ok(local));

    let bad_rate = Cost::Converted {
        amount: euros(100),
        rate: "-1".to_owned(),
    };
    assert!(matches!(
        bad_rate.to_library(),
        Err(EranosError::Money {
            failure: MoneyFailure::InvalidExchangeRate,
            ..
        })
    ));
    let foreign_in_unknown = Cost::Converted {
        amount: unknown(),
        rate: "1".to_owned(),
    };
    assert!(is_unknown_currency(&foreign_in_unknown.to_library()));
    let local_in_unknown = Cost::Local { amount: unknown() };
    assert!(is_unknown_currency(&local_in_unknown.to_library()));
}

/// A currency change's rate would reach the library for another currency
/// or at another rate, and a currency's details would reach the apps
/// changed.
#[test]
fn rates_and_currency_details_convert() {
    let rate = CurrencyRate {
        currency: "EUR".to_owned(),
        rate: "4.97".to_owned(),
    }
    .to_library()
    .unwrap();
    assert_eq!(rate.currency().code(), "EUR");
    assert_eq!(rate.rate().to_string(), "4.97");
    let unknown_currency = CurrencyRate {
        currency: "XYZ".to_owned(),
        rate: "1".to_owned(),
    };
    assert!(is_unknown_currency(&unknown_currency.to_library()));
    let bad_rate = CurrencyRate {
        currency: "EUR".to_owned(),
        rate: "zero".to_owned(),
    };
    assert!(matches!(
        bad_rate.to_library(),
        Err(EranosError::Money {
            failure: MoneyFailure::InvalidExchangeRate,
            ..
        })
    ));
    assert_eq!(
        CurrencyDetails::from_library(library::CurrencyCode::EUR),
        CurrencyDetails {
            code: "EUR".to_owned(),
            name: "Euro".to_owned(),
            numeric_code: 978,
            decimal_places: 2,
        }
    );
}
