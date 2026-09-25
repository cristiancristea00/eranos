//! Amounts, costs, and rates cross the bindings as codes, integers, and
//! decimals. One converted back and forth would change its currency, its
//! minor units, or its rate, and an unknown currency or a bad rate would
//! reach the library.

use super::*;

fn euros(minor_units: i64) -> Money {
    Money {
        currency: "EUR".to_owned(),
        minor_units,
    }
}

fn unknown() -> Money {
    Money {
        currency: "XYZ".to_owned(),
        minor_units: 1,
    }
}

/// An amount would come back in another currency or with other minor
/// units, or one in an unknown currency would reach the library.
#[test]
fn amounts_convert_both_ways_and_refuse_unknown_currencies() {
    assert_eq!(
        euros(1_250).to_library().map(Money::from_library),
        Ok(euros(1_250))
    );
    assert!(matches!(
        unknown().to_library(),
        Err(EranosError::UnknownCurrency { .. })
    ));
}

/// A cost would come back without its rate, a foreign cost would reach the
/// library as a local one, or a bad rate or currency would get through.
#[test]
fn costs_convert_both_ways_and_refuse_bad_parts() {
    let converted = Cost::Converted {
        amount: Money {
            currency: "RON".to_owned(),
            minor_units: 5_000,
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
        Err(EranosError::InvalidExchangeRate { .. })
    ));
    let foreign_in_unknown = Cost::Converted {
        amount: unknown(),
        rate: "1".to_owned(),
    };
    assert!(matches!(
        foreign_in_unknown.to_library(),
        Err(EranosError::UnknownCurrency { .. })
    ));
    let local_in_unknown = Cost::Local { amount: unknown() };
    assert!(matches!(
        local_in_unknown.to_library(),
        Err(EranosError::UnknownCurrency { .. })
    ));
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
    assert!(matches!(
        unknown_currency.to_library(),
        Err(EranosError::UnknownCurrency { .. })
    ));
    let bad_rate = CurrencyRate {
        currency: "EUR".to_owned(),
        rate: "zero".to_owned(),
    };
    assert!(matches!(
        bad_rate.to_library(),
        Err(EranosError::InvalidExchangeRate { .. })
    ));
    assert_eq!(
        CurrencyDetails::from_library(library::Currency::EUR),
        CurrencyDetails {
            code: "EUR".to_owned(),
            decimal_places: 2,
            name: "Euro".to_owned(),
            numeric_code: 978,
        }
    );
}
