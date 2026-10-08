// The currency of an amount. The library gives a currency's details with every amount, as the
// group currency, and in a failure, and takes them back with every amount: a layer that looked
// a code up again would show another currency than the library's, and details the library does
// not list would reach a group. The tests build values of the generated code, so every name of
// the package is qualified.

@testable import Eranos
import EranosGenerated
import Testing

/// An amount that looked its currency up by its code, instead of keeping the details the
/// library gave, would show or send another currency than the library's, or none for a code the
/// lookup misses.
@Test
func anAmountKeepsTheDetailsOfItsCurrencyBothWays() {
    let details = EranosGenerated.CurrencyDetails(code: "XYZ", name: "Test", numericCode: 999, decimalPlaces: 4)
    let generated = EranosGenerated.Money(minorUnits: -12_345, currency: details)
    let money = Eranos.Money(generated)
    #expect("XYZ" == money.currency.code)
    #expect("Test" == money.currency.name)
    #expect(999 == money.currency.numericCode)
    #expect(4 == money.currency.decimalPlaces)
    #expect("-1.2345 XYZ" == money.description)
    #expect(generated == money.generated)
}

/// An amount in a currency whose details are not the library's for its code would reach the
/// group as the library's currency, with other decimal places than the app showed.
@Test
func anAmountWhoseCurrencyIsNotTheLibrarysIsRefused() {
    let details = EranosGenerated.CurrencyDetails(code: "EUR", name: "Euro", numericCode: 978, decimalPlaces: 3)
    let money = Eranos.Money(minorUnits: 1_000, currency: Eranos.CurrencyCode(details))
    #expect("1.000 EUR" == money.description)
    #expect(throws: Eranos.EranosError.money(.unknownCurrency, detail: "unknown currency code")) {
        try money.adding(money)
    }
}
