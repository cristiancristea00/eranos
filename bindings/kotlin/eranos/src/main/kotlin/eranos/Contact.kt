package eranos

import eranos.internal.emailParse
import eranos.internal.ibanCountryCode
import eranos.internal.ibanParse
import eranos.internal.languageParse
import eranos.internal.phoneNumberInternational
import eranos.internal.phoneNumberParse
import eranos.internal.usernameParse

/** An email address, checked by the library's rules. */
public class Email internal constructor(
    /** The address, as entered but without surrounding whitespace. */
    private val address: String,
) {
    /** Whether [other] is of the same class, with the same address. */
    override fun equals(other: Any?): Boolean = other is Email && address == other.address

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = address.hashCode()

    /** The address, as entered but without surrounding whitespace. */
    override fun toString(): String = address

    /** The parsing of an address from its text. */
    public companion object {
        /**
         * The address written [text].
         *
         * @throws EranosException.Contact with [ContactFailure.InvalidEmail] for text that is not a local part and a
         *   domain around an `@`.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): Email = converting { Email(emailParse(text)) }
    }
}

/** A phone number, valid in its country's numbering plan. */
public class PhoneNumber internal constructor(
    /** The number in E.164 form: "+40712345678". */
    public val e164: String,
) {
    /** Whether [other] is of the same class, with the same [e164]. */
    override fun equals(other: Any?): Boolean = other is PhoneNumber && e164 == other.e164

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = e164.hashCode()

    /** The number grouped as its country groups it: "+40 712 345 678". */
    override fun toString(): String =
        try {
            phoneNumberInternational(e164)
        } catch (_: eranos.internal.EranosException) {
            e164
        }

    /** The parsing of a number from its text. */
    public companion object {
        /**
         * The number written [text], starting with `+` and the country code.
         *
         * @throws EranosException.Contact with [ContactFailure.InvalidPhoneNumber].
         */
        @Throws(EranosException::class)
        public fun parse(text: String): PhoneNumber = converting { PhoneNumber(phoneNumberParse(text)) }
    }
}

/** An international bank account number, checked against the SWIFT IBAN registry. */
public class Iban internal constructor(
    /** The IBAN without spaces, in capitals. */
    private val text: String,
) {
    /** The two-letter country code: "RO". */
    public val countryCode: String
        get() =
            try {
                ibanCountryCode(text)
            } catch (_: eranos.internal.EranosException) {
                text.take(2)
            }

    /** Whether [other] is of the same class, with the same text. */
    override fun equals(other: Any?): Boolean = other is Iban && text == other.text

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = text.hashCode()

    /** The IBAN without spaces, in capitals: "RO49AAAA1B31007593840000". */
    override fun toString(): String = text

    /** The parsing of an IBAN from its text. */
    public companion object {
        /**
         * The IBAN written [text], with or without spaces, in any letter case.
         *
         * @throws EranosException.PaymentAccount with [PaymentAccountFailure.InvalidIban] and the check it failed.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): Iban = converting { Iban(ibanParse(text)) }
    }
}

/** A username in a payment app. */
public class Username internal constructor(
    /** The username, as entered but without surrounding whitespace. */
    private val text: String,
) {
    /** Whether [other] is of the same class, with the same text. */
    override fun equals(other: Any?): Boolean = other is Username && text == other.text

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = text.hashCode()

    /** The username, as entered but without surrounding whitespace. */
    override fun toString(): String = text

    /** The parsing of a username from its text. */
    public companion object {
        /**
         * The username written [text].
         *
         * @throws EranosException.PaymentAccount with [PaymentAccountFailure.EmptyUsername] for blank text.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): Username = converting { Username(usernameParse(text)) }
    }
}

/** A language the apps show a group in: a BCP 47 tag, such as "pt-PT". */
public class Language internal constructor(
    /** The tag in its canonical case. */
    private val tag: String,
) {
    /** Whether [other] is of the same class, with the same tag. */
    override fun equals(other: Any?): Boolean = other is Language && tag == other.tag

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = tag.hashCode()

    /** The tag in its canonical case: "pt-PT". */
    override fun toString(): String = tag

    /** The parsing of a language from its tag. */
    public companion object {
        /**
         * The language of the tag [text], in any letter case: "pt-pt".
         *
         * @throws EranosException.Language with [LanguageFailure.InvalidLanguage] for text that is not a BCP 47 tag.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): Language = converting { Language(languageParse(text)) }
    }
}
