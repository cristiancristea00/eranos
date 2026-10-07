package eranos

import eranos.internal.identifierParse
import kotlin.jvm.JvmInline

/**
 * The identifier of a group: the text the app gave it, such as the key of its own store, or one the library generated, a
 * UUID v7 in lowercase hyphenated form.
 *
 * An identifier is text of at most 64 bytes of UTF-8, not empty, without control characters, and without whitespace at
 * either end. Two identifiers are equal when their text is, byte for byte, so case matters. Every entity has an
 * identifier type of its own, so one entity's identifier cannot stand where another's is expected. An identifier
 * converts to its text and back, for storing it: the constructor keeps the text unchecked, and the library checks it when
 * a call carries it, while [parse] checks it at once.
 */
@JvmInline
public value class ExpenseGroupId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value

    /** The checked reading of an identifier from its text. */
    public companion object {
        /**
         * The identifier written [text], as it is: not trimmed, and in its case.
         *
         * @throws EranosException.Id with the first rule the text breaks: [IdFailure.Empty], [IdFailure.TooLong] for more
         *   than 64 bytes of UTF-8, [IdFailure.ControlCharacter], or [IdFailure.SurroundingWhitespace].
         */
        @Throws(EranosException::class)
        public fun parse(text: String): ExpenseGroupId = converting { ExpenseGroupId(identifierParse(text)) }
    }
}

/**
 * The identifier of a member: the text the app gave them, or one the library generated, a UUID v7 in lowercase hyphenated
 * form, which follows the rules [ExpenseGroupId] states.
 */
@JvmInline
public value class MemberId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value

    /** The checked reading of an identifier from its text. */
    public companion object {
        /**
         * The identifier written [text], as it is: not trimmed, and in its case.
         *
         * @throws EranosException.Id as [ExpenseGroupId.parse] does.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): MemberId = converting { MemberId(identifierParse(text)) }
    }
}

/**
 * The identifier of a category: the text the app gave it, or one the library generated, a UUID v7 in lowercase hyphenated
 * form, which follows the rules [ExpenseGroupId] states.
 */
@JvmInline
public value class CategoryId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value

    /** The checked reading of an identifier from its text. */
    public companion object {
        /**
         * The identifier written [text], as it is: not trimmed, and in its case.
         *
         * @throws EranosException.Id as [ExpenseGroupId.parse] does.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): CategoryId = converting { CategoryId(identifierParse(text)) }
    }
}

/**
 * The identifier of an expense: the text the app gave it, or one the library generated, a UUID v7 in lowercase hyphenated
 * form, which follows the rules [ExpenseGroupId] states.
 */
@JvmInline
public value class ExpenseId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value

    /** The checked reading of an identifier from its text. */
    public companion object {
        /**
         * The identifier written [text], as it is: not trimmed, and in its case.
         *
         * @throws EranosException.Id as [ExpenseGroupId.parse] does.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): ExpenseId = converting { ExpenseId(identifierParse(text)) }
    }
}

/**
 * The identifier of a transfer: the text the app gave it, or one the library generated, a UUID v7 in lowercase hyphenated
 * form, which follows the rules [ExpenseGroupId] states.
 */
@JvmInline
public value class TransferId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value

    /** The checked reading of an identifier from its text. */
    public companion object {
        /**
         * The identifier written [text], as it is: not trimmed, and in its case.
         *
         * @throws EranosException.Id as [ExpenseGroupId.parse] does.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): TransferId = converting { TransferId(identifierParse(text)) }
    }
}
