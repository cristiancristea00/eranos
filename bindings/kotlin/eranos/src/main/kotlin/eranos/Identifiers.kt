package eranos

import kotlin.jvm.JvmInline

/**
 * The identifier of a group: the text the library gave it, a UUID v7 in lowercase hyphenated form.
 *
 * An identifier is text of at most 64 bytes of UTF-8, not empty, without control characters, and without whitespace at
 * either end. Two identifiers are equal when their text is, byte for byte, so case matters. Every entity has an
 * identifier type of its own, so one entity's identifier cannot stand where another's is expected. An identifier
 * converts to its text and back, for storing it; the library checks the text when a call carries it.
 */
@JvmInline
public value class ExpenseGroupId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/**
 * The identifier of a member: the text the group gave them, a UUID v7 in lowercase hyphenated form, which follows the
 * rules [ExpenseGroupId] states.
 */
@JvmInline
public value class MemberId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/**
 * The identifier of a category: the text the group gave it, a UUID v7 in lowercase hyphenated form, which follows the
 * rules [ExpenseGroupId] states.
 */
@JvmInline
public value class CategoryId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/**
 * The identifier of an expense: the text the group gave it, a UUID v7 in lowercase hyphenated form, which follows the
 * rules [ExpenseGroupId] states.
 */
@JvmInline
public value class ExpenseId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/**
 * The identifier of a transfer: the text the group gave it, a UUID v7 in lowercase hyphenated form, which follows the
 * rules [ExpenseGroupId] states.
 */
@JvmInline
public value class TransferId(
    /** The identifier's text. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}
