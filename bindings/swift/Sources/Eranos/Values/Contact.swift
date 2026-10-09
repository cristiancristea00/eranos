internal import EranosGenerated

/// An email address, checked by the library's rules.
public struct Email: Hashable, Sendable, Codable, CustomStringConvertible {
    /// The address, as entered but without surrounding whitespace.
    public let text: String

    /// The address, as entered but without surrounding whitespace.
    public var description: String {
        text
    }

    /// The address `decoder` holds as its text.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of the text.
    public init(from decoder: any Decoder) throws {
        let text = try decoder.singleValueContainer().decode(String.self)
        self = try decoded(decoder) { () throws(EranosError) in try Self(text) }
    }

    /// Writes the address's text to `encoder`.
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(text)
    }

    /// The address written `text`.
    ///
    /// - Throws: ``EranosError/contact(_:detail:)`` with ``ContactFailure/invalidEmail`` for text
    ///   that is not a local part and a domain around an `@`.
    public init(_ text: String) throws(EranosError) {
        self.text = try converting { try EranosGenerated.emailParse(text: text) }
    }

    /// The address the library wrote as `text`.
    init(known text: String) {
        self.text = text
    }
}

/// A phone number, valid in its country's numbering plan.
///
/// A number holds both forms the library writes it in, as the library gives them, and crosses
/// back with both. It encodes as both forms, and decodes by parsing its E.164 form; a decoder
/// whose international form is not the one the library writes for it is refused, as the library
/// refuses such a number with ``EranosError/mismatchedRecord(_:detail:)``.
public struct PhoneNumber: Hashable, Sendable, Codable, CustomStringConvertible {
    /// The number in E.164 form: "+40712345678".
    public let e164: String

    /// The number in international form, grouped as its country groups it: "+40 712 345 678".
    public let international: String

    /// The number in international form: "+40 712 345 678".
    public var description: String {
        international
    }

    var generated: EranosGenerated.PhoneNumber {
        .init(e164: e164, international: international)
    }

    /// The number written `text`, starting with `+` and the country code.
    ///
    /// - Throws: ``EranosError/contact(_:detail:)`` with ``ContactFailure/invalidPhoneNumber``.
    public init(_ text: String) throws(EranosError) {
        let phone = try converting { try EranosGenerated.phoneNumberParse(text: text) }
        self.init(phone)
    }

    init(_ phone: EranosGenerated.PhoneNumber) {
        e164 = phone.e164
        international = phone.international
    }

    /// The number `decoder` holds in both forms.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of its E.164 form, or
    ///   of an international form other than the one the library writes for it.
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let e164 = try container.decode(String.self, forKey: .e164)
        let international = try container.decode(String.self, forKey: .international)
        self = try decoded(decoder) { () throws(EranosError) in
            let phone = try Self(e164)
            guard phone.international == international else {
                throw .mismatchedRecord(.phoneInternational, detail: "the phone number's international form is not the one the library writes for it")
            }
            return phone
        }
    }
}

/// An international bank account number, checked against the SWIFT IBAN registry.
///
/// An IBAN holds its country code as the library gives it, and crosses back with it. It encodes as
/// its text and its country code, and decodes by parsing its text; a decoder whose country code is
/// not the one the library derives is refused, as the library refuses such an IBAN with
/// ``EranosError/mismatchedRecord(_:detail:)``.
public struct IBAN: Hashable, Sendable, Codable, CustomStringConvertible {
    /// The IBAN without spaces, in capitals: "RO49AAAA1B31007593840000".
    public let text: String

    /// The IBAN without spaces, in capitals: "RO49AAAA1B31007593840000".
    public var description: String {
        text
    }

    /// The two-letter country code: "RO".
    public let countryCode: String

    /// The IBAN written `text`, with or without spaces, in any letter case.
    ///
    /// - Throws: ``EranosError/paymentAccount(_:detail:)`` with ``PaymentAccountFailure/invalidIBAN(_:)`` and
    ///   the check it failed.
    public init(_ text: String) throws(EranosError) {
        let iban = try converting { try EranosGenerated.ibanParse(text: text) }
        self.init(iban)
    }

    init(_ iban: EranosGenerated.Iban) {
        text = iban.iban
        countryCode = iban.countryCode
    }

    /// The IBAN `decoder` holds with its country code.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of its text, or of a
    ///   country code other than the one the library derives from it.
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let text = try container.decode(String.self, forKey: .text)
        let countryCode = try container.decode(String.self, forKey: .countryCode)
        self = try decoded(decoder) { () throws(EranosError) in
            let iban = try Self(text)
            guard iban.countryCode == countryCode else {
                throw .mismatchedRecord(.ibanCountryCode, detail: "the IBAN's country code is not the one the library derives from it")
            }
            return iban
        }
    }
}

/// A username in a payment app.
public struct Username: Hashable, Sendable, Codable, CustomStringConvertible {
    /// The username, as entered but without surrounding whitespace.
    public let text: String

    /// The username, as entered but without surrounding whitespace.
    public var description: String {
        text
    }

    /// The username `decoder` holds as its text.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of the text.
    public init(from decoder: any Decoder) throws {
        let text = try decoder.singleValueContainer().decode(String.self)
        self = try decoded(decoder) { () throws(EranosError) in try Self(text) }
    }

    /// Writes the username's text to `encoder`.
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(text)
    }

    /// The username written `text`.
    ///
    /// - Throws: ``EranosError/paymentAccount(_:detail:)`` with ``PaymentAccountFailure/emptyUsername`` for
    ///   blank text.
    public init(_ text: String) throws(EranosError) {
        self.text = try converting { try EranosGenerated.usernameParse(text: text) }
    }

    /// The username the library wrote as `text`.
    init(known text: String) {
        self.text = text
    }
}

/// A language the apps show a group in: a BCP 47 tag, such as "pt-PT".
public struct Language: Hashable, Sendable, Codable, CustomStringConvertible {
    /// The tag in its canonical case: "pt-PT".
    public let text: String

    /// The tag in its canonical case: "pt-PT".
    public var description: String {
        text
    }

    /// The language `decoder` holds as its text.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of the text.
    public init(from decoder: any Decoder) throws {
        let text = try decoder.singleValueContainer().decode(String.self)
        self = try decoded(decoder) { () throws(EranosError) in try Self(text) }
    }

    /// Writes the language's text to `encoder`.
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(text)
    }

    /// The language of the tag `text`, in any letter case: "pt-pt".
    ///
    /// - Throws: ``EranosError/language(_:detail:)`` with ``LanguageFailure/invalidLanguage`` for
    ///   text that is not a BCP 47 tag.
    public init(_ text: String) throws(EranosError) {
        self.text = try converting { try EranosGenerated.languageParse(text: text) }
    }

    /// The language the library wrote as `text`.
    init(known text: String) {
        self.text = text
    }
}
