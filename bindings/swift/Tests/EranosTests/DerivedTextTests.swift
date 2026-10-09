// The text the library derives from a phone number and from an IBAN. The library gives a phone
// number's international form and an IBAN's country code with the value, and takes them back
// with it: a layer that derived them again would call the library to print a value it holds,
// and a value whose derived text is not the library's would reach a group. The tests build
// values of the generated code, so every name of the package is qualified.

@testable import Eranos
import EranosGenerated
import Testing

/// A phone number or an IBAN that derived its text again, instead of keeping the text the library
/// gave, would call the library to print a value it holds, and could show another text than the
/// library's.
@Test
func aPhoneNumberAndAnIBANKeepTheTextTheLibraryDerivedBothWays() {
    let generatedPhone = EranosGenerated.PhoneNumber(e164: "+40712345678", international: "+40 7123 45678")
    let phone = Eranos.PhoneNumber(generatedPhone)
    #expect("+40712345678" == phone.e164)
    #expect("+40 7123 45678" == phone.international)
    #expect("+40 7123 45678" == phone.description)
    #expect(generatedPhone == phone.generated)

    let iban = Eranos.IBAN(EranosGenerated.Iban(iban: "RO49AAAA1B31007593840000", countryCode: "XY"))
    #expect("RO49AAAA1B31007593840000" == iban.description)
    #expect("XY" == iban.countryCode)
    let details = EranosGenerated.PaymentAccountDetails.iban(iban: "RO49AAAA1B31007593840000", countryCode: "XY")
    #expect(Eranos.PaymentAccountDetails.iban(iban) == Eranos.PaymentAccountDetails(details))
    #expect(details == Eranos.PaymentAccountDetails.iban(iban).generated)
}

/// A phone number or an IBAN whose derived text is not the library's would reach the group as the
/// library's value, which the app showed another way; refused as an invalid number or IBAN, it
/// would send the app to check text the library accepts.
@Test
func aPhoneNumberOrAnIBANWhoseDerivedTextIsNotTheLibrarysIsRefused() throws {
    let group = try Flat().group
    let phone = Eranos.PhoneNumber(EranosGenerated.PhoneNumber(e164: "+40712345678", international: "+40712345678"))
    #expect(throws: Eranos.EranosError.mismatchedRecord(.phoneInternational, detail: "the phone number's international form is not the one the library writes for it")) {
        try group.add(Eranos.Member(firstName: "Carol", phone: phone))
    }
    let iban = Eranos.IBAN(EranosGenerated.Iban(iban: "RO49AAAA1B31007593840000", countryCode: "DE"))
    let account = Eranos.PaymentAccount(name: "Main account", details: .iban(iban))
    #expect(throws: Eranos.EranosError.mismatchedRecord(.ibanCountryCode, detail: "the IBAN's country code is not the one the library derives from it")) {
        try group.add(Eranos.Member(firstName: "Carol", accounts: [account]))
    }
}
