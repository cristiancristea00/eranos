//! The codes that have a constant: every ISO 4217 code that `iso4217-static`
//! lists with a minor unit, in code order, which is the order `all` lists.
//!
//! A code the pinned crate lacks fails the build at its constant, and
//! `all_matches_every_iso4217_static_code_with_a_minor_unit` names a code the
//! crate has and this list lacks.

use super::Currency;

currencies! {
    AED AFN ALL AMD AOA ARS AUD AWG AZN
    BAM BBD BDT BHD BIF BMD BND BOB BOV BRL BSD BTN BWP BYN BZD
    CAD CDF CHE CHF CHW CLF CLP CNY COP COU CRC CUP CVE CZK
    DJF DKK DOP DZD
    EGP ERN ETB EUR
    FJD FKP
    GBP GEL GHS GIP GMD GNF GTQ GYD
    HKD HNL HTG HUF
    IDR ILS INR IQD IRR ISK
    JMD JOD JPY
    KES KGS KHR KMF KPW KRW KWD KYD KZT
    LAK LBP LKR LRD LSL LYD
    MAD MDL MGA MKD MMK MNT MOP MRU MUR MVR MWK MXN MXV MYR MZN
    NAD NGN NIO NOK NPR NZD
    OMR
    PAB PEN PGK PHP PKR PLN PYG
    QAR
    RON RSD RUB RWF
    SAR SBD SCR SDG SEK SGD SHP SLE SOS SRD SSP STN SVC SYP SZL
    THB TJS TMT TND TOP TRY TTD TWD TZS
    UAH UGX USD USN UYI UYU UYW UZS
    VED VES VND VUV
    WST
    XAD XAF XCD XCG XOF XPF
    YER
    ZAR ZMW ZWG
}
