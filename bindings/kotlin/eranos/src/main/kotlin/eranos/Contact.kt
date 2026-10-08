package eranos

import eranos.internal.emailParse
import eranos.internal.ibanParse
import eranos.internal.languageParse
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

/**
 * A phone number, valid in its country's numbering plan.
 *
 * A number holds the two forms the library writes it in, which it crosses with: the E.164 form it is stored in, and the
 * international form it prints.
 */
public class PhoneNumber internal constructor(
    /** The number in E.164 form: "+40712345678". */
    public val e164: String,
    /** The number grouped as its country groups it: "+40 712 345 678". */
    public val international: String,
) {
    /** The number as the generated code holds it. */
    internal val generated: eranos.internal.PhoneNumber
        get() = eranos.internal.PhoneNumber(e164, international)

    /** Whether [other] is of the same class, with the same [e164] and [international]. */
    override fun equals(other: Any?): Boolean = other is PhoneNumber && e164 == other.e164 && international == other.international

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(e164, international)

    /** The number in its [international] form: "+40 712 345 678". */
    override fun toString(): String = international

    /** The parsing of a number from its text. */
    public companion object {
        /**
         * The number written [text], starting with `+` and the country code.
         *
         * @throws EranosException.Contact with [ContactFailure.InvalidPhoneNumber].
         */
        @Throws(EranosException::class)
        public fun parse(text: String): PhoneNumber = converting { phoneNumberParse(text).converted() }
    }
}

/** The number the generated code holds as [this]. */
internal fun eranos.internal.PhoneNumber.converted(): PhoneNumber = PhoneNumber(e164, international)

/**
 * An international bank account number, checked against the SWIFT IBAN registry.
 *
 * An IBAN holds the country code the library reads from it, which it crosses with.
 */
public class Iban internal constructor(
    /** The IBAN without spaces, in capitals. */
    private val text: String,
    /** The two-letter country code: "RO". */
    public val countryCode: String,
) {
    /** Whether [other] is of the same class, with the same text and [countryCode]. */
    override fun equals(other: Any?): Boolean = other is Iban && text == other.text && countryCode == other.countryCode

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(text, countryCode)

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
        public fun parse(text: String): Iban = converting { ibanParse(text).converted() }
    }
}

/** The IBAN the generated code holds as [this]. */
internal fun eranos.internal.Iban.converted(): Iban = Iban(iban, countryCode)

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
