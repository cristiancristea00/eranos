// Values the library never sends, which the package still prints: a type that trapped on one
// would crash an app instead. The tests build them from the generated code's values, so every
// name of the package is qualified.

@testable import Eranos
import EranosGenerated
import Foundation
import Testing

/// A value the library never sends, such as an amount in a currency it does not list, would
/// trap an app instead of printing.
@Test
func aValueTheLibraryNeverSendsStillPrints() {
    let unlisted = Eranos.Currency(known: "XYZ")
    #expect(unlisted == Eranos.Currency(unlisted: "XYZ"))
    #expect(Eranos.Money(currency: unlisted, minorUnits: 5).description == "5 XYZ")
    #expect(Eranos.Percentage(known: -1).description == "-1 bp")
    #expect(Eranos.Parts(known: 0).description == "0/100")
    #expect(Eranos.CalendarDate(EranosGenerated.CalendarDate(day: 30, month: 2, year: 2026)).description == "2026/2/30")
    #expect(Eranos.TimeOfDay(EranosGenerated.TimeOfDay(hour: 25, minute: 0)).description == "25h0")
    #expect(Eranos.PhoneNumber(known: "0712").description == "0712")
    #expect(Eranos.IBAN(known: "XY").countryCode == "XY")
    #expect(Eranos.Balance.owes(Eranos.Money(currency: unlisted, minorUnits: 1)).description.contains("XYZ"))
}
