// The failure of every module of the library, with the identifiers, amounts, and currencies it holds typed as the rest of
// the package types them.

@file:Suppress("TooManyFunctions") // One conversion from the generated code per module of the library.

package eranos

import eranos.internal.AccountFailure as GeneratedAccountFailure
import eranos.internal.CalendarFailure as GeneratedCalendarFailure
import eranos.internal.CategoryFailure as GeneratedCategoryFailure
import eranos.internal.ContactFailure as GeneratedContactFailure
import eranos.internal.ExpenseFailure as GeneratedExpenseFailure
import eranos.internal.GroupFailure as GeneratedGroupFailure
import eranos.internal.IbanReason as GeneratedIbanReason
import eranos.internal.IdFailure as GeneratedIdFailure
import eranos.internal.LanguageFailure as GeneratedLanguageFailure
import eranos.internal.MemberFailure as GeneratedMemberFailure
import eranos.internal.MoneyFailure as GeneratedMoneyFailure
import eranos.internal.SettlementFailure as GeneratedSettlementFailure
import eranos.internal.TransferFailure as GeneratedTransferFailure

/** What the library's `AccountError` reports: a type per failure, with what it holds. */
public sealed interface AccountFailure {
    /** An account whose name is blank. */
    public data object EmptyName : AccountFailure

    /** A username that is blank. */
    public data object EmptyUsername : AccountFailure

    /** An IBAN that failed validation, and why. */
    public class InvalidIban(
        /** The check the IBAN failed. */
        public val reason: IbanReason,
    ) : AccountFailure {
        /** Whether [other] is of the same class, with the same [reason]. */
        override fun equals(other: Any?): Boolean = other is InvalidIban && reason == other.reason

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = reason.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "InvalidIban(reason=$reason)"
    }
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedAccountFailure.converted(): AccountFailure =
    when (this) {
        GeneratedAccountFailure.EmptyName -> AccountFailure.EmptyName
        GeneratedAccountFailure.EmptyUsername -> AccountFailure.EmptyUsername
        is GeneratedAccountFailure.InvalidIban -> AccountFailure.InvalidIban(reason.converted())
    }

/** What the library's `CalendarError` reports: a type per failure, with what it holds. */
public sealed interface CalendarFailure {
    /** A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text. */
    public data object InvalidDate : CalendarFailure

    /** A time of day that does not exist, has seconds, or is not ISO 8601 text. */
    public data object InvalidTime : CalendarFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedCalendarFailure.converted(): CalendarFailure =
    when (this) {
        GeneratedCalendarFailure.INVALID_DATE -> CalendarFailure.InvalidDate
        GeneratedCalendarFailure.INVALID_TIME -> CalendarFailure.InvalidTime
    }

/** What the library's `CategoryError` reports: a type per failure, with what it holds. */
public sealed interface CategoryFailure {
    /** A name that is blank. */
    public data object EmptyName : CategoryFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedCategoryFailure.converted(): CategoryFailure =
    when (this) {
        GeneratedCategoryFailure.EMPTY_NAME -> CategoryFailure.EmptyName
    }

/** What the library's `ContactError` reports: a type per failure, with what it holds. */
public sealed interface ContactFailure {
    /**
     * An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around an `@`, a malformed one, one
     * too long, or whitespace or an invisible character.
     */
    public data object InvalidEmail : ContactFailure

    /**
     * A phone number that does not start with `+`, holds a letter or an extension, or is not valid in its country's
     * numbering plan.
     */
    public data object InvalidPhoneNumber : ContactFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedContactFailure.converted(): ContactFailure =
    when (this) {
        GeneratedContactFailure.INVALID_EMAIL -> ContactFailure.InvalidEmail
        GeneratedContactFailure.INVALID_PHONE_NUMBER -> ContactFailure.InvalidPhoneNumber
    }

/** What the library's `ExpenseError` reports: a type per failure, with what it holds. */
public sealed interface ExpenseFailure {
    /** The amounts of a split do not add up to the amount being split. */
    public class AmountsDoNotAddUp(
        /** What the amounts add up to. */
        public val actual: Money,
        /** The amount being split. */
        public val expected: Money,
    ) : ExpenseFailure {
        /** Whether [other] is of the same class, with the same [actual] and [expected]. */
        override fun equals(other: Any?): Boolean = other is AmountsDoNotAddUp && actual == other.actual && expected == other.expected

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(actual, expected)

        /** A description of every field, for logs. */
        override fun toString(): String = "AmountsDoNotAddUp(actual=$actual, expected=$expected)"
    }

    /** An amount inside the expense is in a different currency. */
    public class CurrencyMismatch(
        /** The expense's currency. */
        public val expected: Currency,
        /** The currency of the amount inside it. */
        public val found: Currency,
    ) : ExpenseFailure {
        /** Whether [other] is of the same class, with the same [expected] and [found]. */
        override fun equals(other: Any?): Boolean = other is CurrencyMismatch && expected == other.expected && found == other.found

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(expected, found)

        /** A description of every field, for logs. */
        override fun toString(): String = "CurrencyMismatch(expected=$expected, found=$found)"
    }

    /** A member appears more than once in one split. */
    public class DuplicateMember(
        /** The member who appears more than once. */
        public val member: MemberId,
    ) : ExpenseFailure {
        /** Whether [other] is of the same class, with the same [member]. */
        override fun equals(other: Any?): Boolean = other is DuplicateMember && member == other.member

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = member.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "DuplicateMember(member=$member)"
    }

    /** An item whose name is blank. */
    public data object EmptyItemName : ExpenseFailure

    /** An expense whose name is blank. */
    public data object EmptyName : ExpenseFailure

    /** A split lists no member. */
    public data object EmptySplit : ExpenseFailure

    /** The item line totals do not add up to the expense amount. */
    public class ItemsDoNotAddUp(
        /** What the line totals add up to. */
        public val actual: Money,
        /** The expense amount. */
        public val expected: Money,
    ) : ExpenseFailure {
        /** Whether [other] is of the same class, with the same [actual] and [expected]. */
        override fun equals(other: Any?): Boolean = other is ItemsDoNotAddUp && actual == other.actual && expected == other.expected

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(actual, expected)

        /** A description of every field, for logs. */
        override fun toString(): String = "ItemsDoNotAddUp(actual=$actual, expected=$expected)"
    }

    /** A fixed tip below zero. */
    public data object NegativeTip : ExpenseFailure

    /** An itemised expense without items. */
    public data object NoItems : ExpenseFailure

    /** The amount or unit price is zero or negative. */
    public data object NonPositiveAmount : ExpenseFailure

    /** A member's percentage or amount is zero or negative. */
    public class NonPositiveShare(
        /** The member whose share is zero or negative. */
        public val member: MemberId,
    ) : ExpenseFailure {
        /** Whether [other] is of the same class, with the same [member]. */
        override fun equals(other: Any?): Boolean = other is NonPositiveShare && member == other.member

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = member.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "NonPositiveShare(member=$member)"
    }

    /** A total that does not fit the range of an amount. */
    public data object Overflow : ExpenseFailure

    /** The percentages of a split do not add up to exactly 100 %. */
    public class PercentagesDoNotAddUp(
        /** What the percentages add up to, in basis points. */
        public val totalBasisPoints: Long,
    ) : ExpenseFailure {
        /** Whether [other] is of the same class, with the same [totalBasisPoints]. */
        override fun equals(other: Any?): Boolean = other is PercentagesDoNotAddUp && totalBasisPoints == other.totalBasisPoints

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = totalBasisPoints.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "PercentagesDoNotAddUp(totalBasisPoints=$totalBasisPoints)"
    }

    /** An item quantity of zero. */
    public data object ZeroQuantity : ExpenseFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedExpenseFailure.converted(): ExpenseFailure =
    when (this) {
        is GeneratedExpenseFailure.AmountsDoNotAddUp -> ExpenseFailure.AmountsDoNotAddUp(actual.converted(), expected.converted())
        is GeneratedExpenseFailure.CurrencyMismatch -> ExpenseFailure.CurrencyMismatch(Currency.known(expected), Currency.known(found))
        is GeneratedExpenseFailure.DuplicateMember -> ExpenseFailure.DuplicateMember(MemberId(member))
        GeneratedExpenseFailure.EmptyItemName -> ExpenseFailure.EmptyItemName
        GeneratedExpenseFailure.EmptyName -> ExpenseFailure.EmptyName
        GeneratedExpenseFailure.EmptySplit -> ExpenseFailure.EmptySplit
        is GeneratedExpenseFailure.ItemsDoNotAddUp -> ExpenseFailure.ItemsDoNotAddUp(actual.converted(), expected.converted())
        GeneratedExpenseFailure.NegativeTip -> ExpenseFailure.NegativeTip
        GeneratedExpenseFailure.NoItems -> ExpenseFailure.NoItems
        GeneratedExpenseFailure.NonPositiveAmount -> ExpenseFailure.NonPositiveAmount
        is GeneratedExpenseFailure.NonPositiveShare -> ExpenseFailure.NonPositiveShare(MemberId(member))
        GeneratedExpenseFailure.Overflow -> ExpenseFailure.Overflow
        is GeneratedExpenseFailure.PercentagesDoNotAddUp -> ExpenseFailure.PercentagesDoNotAddUp(totalBasisPoints)
        GeneratedExpenseFailure.ZeroQuantity -> ExpenseFailure.ZeroQuantity
    }

/** What the library's `GroupError` reports: a type per failure, with what it holds. */
public sealed interface GroupFailure {
    /** An archived member cannot join a new expense. */
    public class ArchivedMember(
        /** The archived member. */
        public val member: MemberId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [member]. */
        override fun equals(other: Any?): Boolean = other is ArchivedMember && member == other.member

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = member.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "ArchivedMember(member=$member)"
    }

    /** The built-in Uncategorised category cannot be renamed or removed. */
    public data object BuiltInCategory : GroupFailure

    /** A shared group's lock was asked for without waiting, while another call held it. */
    public data object Busy : GroupFailure

    /** A category with this ID already exists. */
    public class DuplicateCategory(
        /** The identifier another category has. */
        public val category: CategoryId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [category]. */
        override fun equals(other: Any?): Boolean = other is DuplicateCategory && category == other.category

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = category.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "DuplicateCategory(category=$category)"
    }

    /** Another category already has this name, compared by Unicode canonical caseless matching. */
    public class DuplicateCategoryName(
        /** The category that has the name. */
        public val existing: CategoryId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [existing]. */
        override fun equals(other: Any?): Boolean = other is DuplicateCategoryName && existing == other.existing

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = existing.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "DuplicateCategoryName(existing=$existing)"
    }

    /** The same currency appears twice among the rates of a currency change. */
    public class DuplicateExchangeRate(
        /** The currency that appears twice. */
        public val currency: Currency,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [currency]. */
        override fun equals(other: Any?): Boolean = other is DuplicateExchangeRate && currency == other.currency

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = currency.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "DuplicateExchangeRate(currency=$currency)"
    }

    /** An expense with this ID already exists. */
    public class DuplicateExpense(
        /** The identifier another expense has. */
        public val expense: ExpenseId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [expense]. */
        override fun equals(other: Any?): Boolean = other is DuplicateExpense && expense == other.expense

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = expense.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "DuplicateExpense(expense=$expense)"
    }

    /** A member with this ID already exists. */
    public class DuplicateMember(
        /** The identifier another member has. */
        public val member: MemberId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [member]. */
        override fun equals(other: Any?): Boolean = other is DuplicateMember && member == other.member

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = member.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "DuplicateMember(member=$member)"
    }

    /** A transfer with this ID already exists. */
    public class DuplicateTransfer(
        /** The identifier another transfer has. */
        public val transfer: TransferId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [transfer]. */
        override fun equals(other: Any?): Boolean = other is DuplicateTransfer && transfer == other.transfer

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = transfer.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "DuplicateTransfer(transfer=$transfer)"
    }

    /** A group name that is blank. */
    public data object EmptyName : GroupFailure

    /** The member is still referenced and cannot be removed; archive it instead. */
    public class MemberInUse(
        /** The expenses that name the member. */
        public val expenses: List<ExpenseId>,
        /** The member. */
        public val member: MemberId,
        /** The transfers that name the member. */
        public val transfers: List<TransferId>,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [expenses], [member], and [transfers]. */
        override fun equals(other: Any?): Boolean =
            other is MemberInUse && expenses == other.expenses && member == other.member && transfers == other.transfers

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(expenses, member, transfers)

        /** A description of every field, for logs. */
        override fun toString(): String = "MemberInUse(expenses=$expenses, member=$member, transfers=$transfers)"
    }

    /**
     * An entry in another currency than the group's has no exchange rate into the group currency: its cost is local, or a
     * currency change gave no rate for its currency.
     */
    public class MissingExchangeRate(
        /** The currency without a rate. */
        public val currency: Currency,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [currency]. */
        override fun equals(other: Any?): Boolean = other is MissingExchangeRate && currency == other.currency

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = currency.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "MissingExchangeRate(currency=$currency)"
    }

    /** A total would leave the range of an amount. */
    public data object Overflow : GroupFailure

    /** An entry in the group currency has a converted cost; only a local cost is in the group currency. */
    public data object UnexpectedExchangeRate : GroupFailure

    /** No category with this ID. */
    public class UnknownCategory(
        /** The identifier no category has. */
        public val category: CategoryId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [category]. */
        override fun equals(other: Any?): Boolean = other is UnknownCategory && category == other.category

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = category.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "UnknownCategory(category=$category)"
    }

    /** No expense with this ID. */
    public class UnknownExpense(
        /** The identifier no expense has. */
        public val expense: ExpenseId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [expense]. */
        override fun equals(other: Any?): Boolean = other is UnknownExpense && expense == other.expense

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = expense.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "UnknownExpense(expense=$expense)"
    }

    /** No member with this ID. */
    public class UnknownMember(
        /** The identifier no member has. */
        public val member: MemberId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [member]. */
        override fun equals(other: Any?): Boolean = other is UnknownMember && member == other.member

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = member.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "UnknownMember(member=$member)"
    }

    /** No transfer with this ID. */
    public class UnknownTransfer(
        /** The identifier no transfer has. */
        public val transfer: TransferId,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [transfer]. */
        override fun equals(other: Any?): Boolean = other is UnknownTransfer && transfer == other.transfer

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = transfer.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "UnknownTransfer(transfer=$transfer)"
    }

    /** A saved group in a format this version does not read: a newer one, whose fields it would drop, or an earlier one. */
    public class UnsupportedFormat(
        /** The format the saved group names. */
        public val format: Long,
    ) : GroupFailure {
        /** Whether [other] is of the same class, with the same [format]. */
        override fun equals(other: Any?): Boolean = other is UnsupportedFormat && format == other.format

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = format.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "UnsupportedFormat(format=$format)"
    }
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedGroupFailure.converted(): GroupFailure =
    when (this) {
        is GeneratedGroupFailure.ArchivedMember -> GroupFailure.ArchivedMember(MemberId(member))
        GeneratedGroupFailure.BuiltInCategory -> GroupFailure.BuiltInCategory
        GeneratedGroupFailure.Busy -> GroupFailure.Busy
        is GeneratedGroupFailure.DuplicateCategory -> GroupFailure.DuplicateCategory(CategoryId(category))
        is GeneratedGroupFailure.DuplicateCategoryName -> GroupFailure.DuplicateCategoryName(CategoryId(existing))
        is GeneratedGroupFailure.DuplicateExchangeRate -> GroupFailure.DuplicateExchangeRate(Currency.known(currency))
        is GeneratedGroupFailure.DuplicateExpense -> GroupFailure.DuplicateExpense(ExpenseId(expense))
        is GeneratedGroupFailure.DuplicateMember -> GroupFailure.DuplicateMember(MemberId(member))
        is GeneratedGroupFailure.DuplicateTransfer -> GroupFailure.DuplicateTransfer(TransferId(transfer))
        GeneratedGroupFailure.EmptyName -> GroupFailure.EmptyName
        is GeneratedGroupFailure.MemberInUse -> GroupFailure.MemberInUse(expenses.map(::ExpenseId), MemberId(member), transfers.map(::TransferId))
        is GeneratedGroupFailure.MissingExchangeRate -> GroupFailure.MissingExchangeRate(Currency.known(currency))
        GeneratedGroupFailure.Overflow -> GroupFailure.Overflow
        GeneratedGroupFailure.UnexpectedExchangeRate -> GroupFailure.UnexpectedExchangeRate
        is GeneratedGroupFailure.UnknownCategory -> GroupFailure.UnknownCategory(CategoryId(category))
        is GeneratedGroupFailure.UnknownExpense -> GroupFailure.UnknownExpense(ExpenseId(expense))
        is GeneratedGroupFailure.UnknownMember -> GroupFailure.UnknownMember(MemberId(member))
        is GeneratedGroupFailure.UnknownTransfer -> GroupFailure.UnknownTransfer(TransferId(transfer))
        is GeneratedGroupFailure.UnsupportedFormat -> GroupFailure.UnsupportedFormat(format)
    }

/** What the library's `IdError` reports: a type per failure, with what it holds. */
public sealed interface IdFailure {
    /** Text that is not a UUID. */
    public data object InvalidId : IdFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedIdFailure.converted(): IdFailure =
    when (this) {
        GeneratedIdFailure.INVALID_ID -> IdFailure.InvalidId
    }

/** What the library's `LanguageError` reports: a type per failure, with what it holds. */
public sealed interface LanguageFailure {
    /** A language tag that is not a Unicode BCP 47 locale identifier (UTS #35). */
    public data object InvalidLanguage : LanguageFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedLanguageFailure.converted(): LanguageFailure =
    when (this) {
        GeneratedLanguageFailure.INVALID_LANGUAGE -> LanguageFailure.InvalidLanguage
    }

/** What the library's `MemberError` reports: a type per failure, with what it holds. */
public sealed interface MemberFailure {
    /** A first name that is blank. */
    public data object EmptyFirstName : MemberFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedMemberFailure.converted(): MemberFailure =
    when (this) {
        GeneratedMemberFailure.EMPTY_FIRST_NAME -> MemberFailure.EmptyFirstName
    }

/** What the library's `MoneyError` reports: a type per failure, with what it holds. */
public sealed interface MoneyFailure {
    /** Arithmetic between amounts of two different currencies. */
    public class CurrencyMismatch(
        /** The currency of the first amount. */
        public val left: Currency,
        /** The currency of the second amount. */
        public val right: Currency,
    ) : MoneyFailure {
        /** Whether [other] is of the same class, with the same [left] and [right]. */
        override fun equals(other: Any?): Boolean = other is CurrencyMismatch && left == other.left && right == other.right

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(left, right)

        /** A description of every field, for logs. */
        override fun toString(): String = "CurrencyMismatch(left=$left, right=$right)"
    }

    /** An amount that is not a decimal number. */
    public data object InvalidAmount : MoneyFailure

    /** An exchange rate that is not a positive decimal number with at most 18 decimal places. */
    public data object InvalidExchangeRate : MoneyFailure

    /** A number of parts that is not a positive decimal number with at most two decimal places. */
    public data object InvalidParts : MoneyFailure

    /** A percentage that is not a decimal number with at most two decimal places. */
    public data object InvalidPercentage : MoneyFailure

    /** A number or a result that does not fit its range. */
    public data object Overflow : MoneyFailure

    /** An amount with more decimal places than its currency has. */
    public class TooManyDecimals(
        /** How many decimal places the currency has. */
        public val allowed: Long,
        /** The currency. */
        public val currency: Currency,
    ) : MoneyFailure {
        /** Whether [other] is of the same class, with the same [allowed] and [currency]. */
        override fun equals(other: Any?): Boolean = other is TooManyDecimals && allowed == other.allowed && currency == other.currency

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(allowed, currency)

        /** A description of every field, for logs. */
        override fun toString(): String = "TooManyDecimals(allowed=$allowed, currency=$currency)"
    }

    /** A currency code that is not an active ISO 4217 code. */
    public data object UnknownCurrency : MoneyFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedMoneyFailure.converted(): MoneyFailure =
    when (this) {
        is GeneratedMoneyFailure.CurrencyMismatch -> MoneyFailure.CurrencyMismatch(Currency.known(left), Currency.known(right))
        GeneratedMoneyFailure.InvalidAmount -> MoneyFailure.InvalidAmount
        GeneratedMoneyFailure.InvalidExchangeRate -> MoneyFailure.InvalidExchangeRate
        GeneratedMoneyFailure.InvalidParts -> MoneyFailure.InvalidParts
        GeneratedMoneyFailure.InvalidPercentage -> MoneyFailure.InvalidPercentage
        GeneratedMoneyFailure.Overflow -> MoneyFailure.Overflow
        is GeneratedMoneyFailure.TooManyDecimals -> MoneyFailure.TooManyDecimals(allowed, Currency.known(currency))
        GeneratedMoneyFailure.UnknownCurrency -> MoneyFailure.UnknownCurrency
    }

/** What the library's `SettlementError` reports: a type per failure, with what it holds. */
public sealed interface SettlementFailure {
    /**
     * The exact solver needs 2ⁿ bytes for `n` open balances, and that table does not fit in the address space or the
     * allocator refused it.
     */
    public class OutOfMemory(
        /** How many open balances the plan has. */
        public val balances: Long,
    ) : SettlementFailure {
        /** Whether [other] is of the same class, with the same [balances]. */
        override fun equals(other: Any?): Boolean = other is OutOfMemory && balances == other.balances

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = balances.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "OutOfMemory(balances=$balances)"
    }
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedSettlementFailure.converted(): SettlementFailure =
    when (this) {
        is GeneratedSettlementFailure.OutOfMemory -> SettlementFailure.OutOfMemory(balances)
    }

/** What the library's `TransferError` reports: a type per failure, with what it holds. */
public sealed interface TransferFailure {
    /** The amount is zero or negative. */
    public data object NonPositiveAmount : TransferFailure

    /** The sender and the recipient are the same member. */
    public data object SameMember : TransferFailure
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedTransferFailure.converted(): TransferFailure =
    when (this) {
        GeneratedTransferFailure.NON_POSITIVE_AMOUNT -> TransferFailure.NonPositiveAmount
        GeneratedTransferFailure.SAME_MEMBER -> TransferFailure.SameMember
    }

/** Why the library refused an IBAN: a type per check it failed. */
public sealed interface IbanReason {
    /** A character other than an ASCII letter, a digit, or whitespace (which is dropped). */
    public data object InvalidCharacter : IbanReason

    /**
     * The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97 check failed: the number
     * contains a typo.
     */
    public data object InvalidChecksum : IbanReason

    /** Not two letters followed by two check digits. */
    public data object InvalidFormat : IbanReason

    /**
     * A letter where the country's account number has a digit, or a digit where it has a letter, by the BBAN structure of
     * the SWIFT IBAN registry.
     */
    public data object InvalidStructure : IbanReason

    /** A country code without an entry in the SWIFT IBAN registry. */
    public data object UnknownCountry : IbanReason

    /** The wrong number of characters for the country. */
    public class WrongLength(
        /** How many characters the country's IBANs have. */
        public val expected: Long,
        /** How many characters the IBAN has. */
        public val found: Long,
    ) : IbanReason {
        /** Whether [other] is of the same class, with the same [expected] and [found]. */
        override fun equals(other: Any?): Boolean = other is WrongLength && expected == other.expected && found == other.found

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(expected, found)

        /** A description of every field, for logs. */
        override fun toString(): String = "WrongLength(expected=$expected, found=$found)"
    }
}

/** The failure the generated code reports as [this]. */
internal fun GeneratedIbanReason.converted(): IbanReason =
    when (this) {
        GeneratedIbanReason.InvalidCharacter -> IbanReason.InvalidCharacter
        GeneratedIbanReason.InvalidChecksum -> IbanReason.InvalidChecksum
        GeneratedIbanReason.InvalidFormat -> IbanReason.InvalidFormat
        GeneratedIbanReason.InvalidStructure -> IbanReason.InvalidStructure
        GeneratedIbanReason.UnknownCountry -> IbanReason.UnknownCountry
        is GeneratedIbanReason.WrongLength -> IbanReason.WrongLength(expected, found)
    }
