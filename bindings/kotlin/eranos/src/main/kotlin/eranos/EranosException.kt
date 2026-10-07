package eranos

import kotlin.coroutines.cancellation.CancellationException

/**
 * Why a call failed: a subclass per module of the library, holding what the module reports, with the library's message.
 *
 * Every call of the package that the library refuses throws a subclass, so a `catch` takes a module's failures together,
 * and a `when` over its `failure` takes one of them:
 *
 * ```kotlin
 * try {
 *     group.remove(member)
 * } catch (exception: EranosException.ExpenseGroup) {
 *     if (exception.failure !is ExpenseGroupFailure.MemberInUse) throw exception
 *     group.archive(member) // The member still has entries; archive them instead.
 * }
 * ```
 *
 * A call the program should not make throws the standard exception instead: [IllegalArgumentException] for an entry to
 * replace that has no identifier, and [IllegalStateException] for a read through an [ExpenseGroupReader] after it is closed. A
 * call whose coroutine is cancelled throws [CancellationException].
 */
public sealed class EranosException(
    /** What went wrong, as the library words it. */
    override val message: String,
) : Exception(message) {
    /** The library's `IdError`, with the message it words. */
    public class Id(
        /** What the module reports. */
        public val failure: IdFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `PaymentAccountError`, with the message it words. */
    public class PaymentAccount(
        /** What the module reports. */
        public val failure: PaymentAccountFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `CalendarError`, with the message it words. */
    public class Calendar(
        /** What the module reports. */
        public val failure: CalendarFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `CategoryError`, with the message it words. */
    public class Category(
        /** What the module reports. */
        public val failure: CategoryFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `ContactError`, with the message it words. */
    public class Contact(
        /** What the module reports. */
        public val failure: ContactFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `MoneyError`, with the message it words. */
    public class Money(
        /** What the module reports. */
        public val failure: MoneyFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `MemberError`, with the message it words. */
    public class Member(
        /** What the module reports. */
        public val failure: MemberFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `ExpenseError`, with the message it words. */
    public class Expense(
        /** What the module reports. */
        public val failure: ExpenseFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `SettlementError`, with the message it words. */
    public class Settlement(
        /** What the module reports. */
        public val failure: SettlementFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `TransferError`, with the message it words. */
    public class Transfer(
        /** What the module reports. */
        public val failure: TransferFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `ExpenseGroupError`, with the message it words. */
    public class ExpenseGroup(
        /** What the module reports. */
        public val failure: ExpenseGroupFailure,
        message: String,
    ) : EranosException(message)

    /** The library's `LanguageError`, with the message it words. */
    public class Language(
        /** What the module reports. */
        public val failure: LanguageFailure,
        message: String,
    ) : EranosException(message)

    /**
     * A saved group that is not JSON of the saved form, at [line] and [column]; the message names the fault but never the
     * text there, which can hold an account number.
     */
    public class InvalidJson(
        /** The line of the fault, from 1. */
        public val line: Long,
        /** The column of the fault, from 1. */
        public val column: Long,
        message: String,
    ) : EranosException(message)
}

/**
 * The exception the generated code throws as [this], as this package throws it: a call the program should not make as the
 * standard exception for it, and a cancelled call as [CancellationException].
 */
internal fun eranos.internal.EranosException.converted(): Exception =
    when (this) {
        is eranos.internal.EranosException.Id -> EranosException.Id(failure.converted(), detail)
        is eranos.internal.EranosException.PaymentAccount -> EranosException.PaymentAccount(failure.converted(), detail)
        is eranos.internal.EranosException.Calendar -> EranosException.Calendar(failure.converted(), detail)
        is eranos.internal.EranosException.Category -> EranosException.Category(failure.converted(), detail)
        is eranos.internal.EranosException.Contact -> EranosException.Contact(failure.converted(), detail)
        is eranos.internal.EranosException.Money -> EranosException.Money(failure.converted(), detail)
        is eranos.internal.EranosException.Member -> EranosException.Member(failure.converted(), detail)
        is eranos.internal.EranosException.Expense -> EranosException.Expense(failure.converted(), detail)
        is eranos.internal.EranosException.Settlement -> EranosException.Settlement(failure.converted(), detail)
        is eranos.internal.EranosException.Transfer -> EranosException.Transfer(failure.converted(), detail)
        is eranos.internal.EranosException.ExpenseGroup -> EranosException.ExpenseGroup(failure.converted(), detail)
        is eranos.internal.EranosException.Language -> EranosException.Language(failure.converted(), detail)
        is eranos.internal.EranosException.IdentifierMissing -> IllegalArgumentException(detail)
        is eranos.internal.EranosException.InvalidJson -> EranosException.InvalidJson(line, column, detail)
        is eranos.internal.EranosException.Cancelled -> CancellationException(detail)
        is eranos.internal.EranosException.Released -> IllegalStateException(detail)
    }
