package eranos

import kotlin.jvm.JvmInline

/**
 * The identifier of a group: a hyphenated UUID the library gave it.
 *
 * Every entity has an identifier type of its own, so one entity's identifier cannot stand where another's is expected. An
 * identifier converts to its text and back, for storing it; the library checks the text when a call carries it.
 */
@JvmInline
public value class GroupId(
    /** The identifier's text: a hyphenated UUID. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/** The identifier of a member: a hyphenated UUID the group gave them. */
@JvmInline
public value class MemberId(
    /** The identifier's text: a hyphenated UUID. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/** The identifier of a category: a hyphenated UUID the group gave it. */
@JvmInline
public value class CategoryId(
    /** The identifier's text: a hyphenated UUID. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/** The identifier of an expense: a hyphenated UUID the group gave it. */
@JvmInline
public value class ExpenseId(
    /** The identifier's text: a hyphenated UUID. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}

/** The identifier of a transfer: a hyphenated UUID the group gave it. */
@JvmInline
public value class TransferId(
    /** The identifier's text: a hyphenated UUID. */
    public val value: String,
) {
    /** The identifier's text. */
    override fun toString(): String = value
}
