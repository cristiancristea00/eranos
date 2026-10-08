// Values the library never sends, which the package still prints: a type that trapped on one
// would crash an app instead. The tests build them from the generated code's values, so every
// name of the package is qualified.

@testable import Eranos
import EranosGenerated
import Testing

/// A value the library never sends, such as a date that does not exist, would trap an app
/// instead of printing.
@Test
func aValueTheLibraryNeverSendsStillPrints() {
    #expect("-1 bp" == Eranos.Percentage(known: -1).description)
    #expect("0/100" == Eranos.Parts(known: 0).description)
    #expect("2026/2/30" == Eranos.CalendarDate(EranosGenerated.CalendarDate(year: 2026, month: 2, day: 30)).description)
    #expect("25h0" == Eranos.TimeOfDay(EranosGenerated.TimeOfDay(hour: 25, minute: 0)).description)
    #expect("0712" == Eranos.PhoneNumber(known: "0712").description)
    #expect("XY" == Eranos.IBAN(known: "XY").countryCode)
}
