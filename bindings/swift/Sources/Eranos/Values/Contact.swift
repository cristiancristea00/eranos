internal import EranosGenerated

/// An email address, checked by the library's rules.
public struct Email: Hashable, Sendable, CustomStringConvertible {
    /// The address, as entered but without surrounding whitespace.
    public let description: String

    /// The address written `text`.
    ///
    /// - Throws: ``EranosError/contact(_:detail:)`` with ``ContactFailure/invalidEmail`` for text
    ///   that is not a local part and a domain around an `@`.
    public init(_ text: String) throws(EranosError) {
        description = try converting { try EranosGenerated.emailParse(text: text) }
    }

    /// The address the library wrote as `text`.
    init(known text: String) {
        description = text
    }
}

/// A phone number, valid in its country's numbering plan.
public struct PhoneNumber: Hashable, Sendable, CustomStringConvertible {
    /// The number in E.164 form: "+40712345678".
    public let e164: String

    /// The number grouped as its country groups it: "+40 712 345 678".
    public var description: String {
        (try? EranosGenerated.phoneNumberInternational(text: e164)) ?? e164
    }

    /// The number written `text`, starting with `+` and the country code.
    ///
    /// - Throws: ``EranosError/contact(_:detail:)`` with ``ContactFailure/invalidPhoneNumber``.
    public init(_ text: String) throws(EranosError) {
        e164 = try converting { try EranosGenerated.phoneNumberParse(text: text) }
    }

    /// The number the library wrote as `e164`.
    init(known e164: String) {
        self.e164 = e164
    }
}

/// An international bank account number, checked against the SWIFT IBAN registry.
public struct IBAN: Hashable, Sendable, CustomStringConvertible {
    /// The IBAN without spaces, in capitals: "RO49AAAA1B31007593840000".
    public let description: String

    /// The two-letter country code: "RO".
    public var countryCode: String {
        (try? EranosGenerated.ibanCountryCode(text: description)) ?? String(description.prefix(2))
    }

    /// The IBAN written `text`, with or without spaces, in any letter case.
    ///
    /// - Throws: ``EranosError/account(_:detail:)`` with ``AccountFailure/invalidIBAN(_:)`` and
    ///   the check it failed.
    public init(_ text: String) throws(EranosError) {
        description = try converting { try EranosGenerated.ibanParse(text: text) }
    }

    /// The IBAN the library wrote as `text`.
    init(known text: String) {
        description = text
    }
}

/// A username in a payment app.
public struct Username: Hashable, Sendable, CustomStringConvertible {
    /// The username, as entered but without surrounding whitespace.
    public let description: String

    /// The username written `text`.
    ///
    /// - Throws: ``EranosError/account(_:detail:)`` with ``AccountFailure/emptyUsername`` for
    ///   blank text.
    public init(_ text: String) throws(EranosError) {
        description = try converting { try EranosGenerated.usernameParse(text: text) }
    }

    /// The username the library wrote as `text`.
    init(known text: String) {
        description = text
    }
}

/// A language the apps show a group in: a BCP 47 tag, such as "pt-PT".
public struct Language: Hashable, Sendable, CustomStringConvertible {
    /// The tag in its canonical case: "pt-PT".
    public let description: String

    /// The language of the tag `text`, in any letter case: "pt-pt".
    ///
    /// - Throws: ``EranosError/language(_:detail:)`` with ``LanguageFailure/invalidLanguage`` for
    ///   text that is not a BCP 47 tag.
    public init(_ text: String) throws(EranosError) {
        description = try converting { try EranosGenerated.languageParse(text: text) }
    }

    /// The language the library wrote as `text`.
    init(known text: String) {
        description = text
    }
}
