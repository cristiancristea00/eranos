// What the sections of the tour share: the members and categories they name, the amounts and dates of the trip, and
// the outcomes of the changes a section tries.
package eranos.examples.grandtour

import eranos.CalendarDate
import eranos.CategoryId
import eranos.Cost
import eranos.CurrencyCode
import eranos.EranosException
import eranos.ExchangeRate
import eranos.ExpenseGroup
import eranos.MemberId
import eranos.Money
import eranos.examples.names

/** The members of the trip. */
internal data class People(
    val ana: MemberId,
    val bob: MemberId,
    val carol: MemberId,
    val dan: MemberId,
    val eva: MemberId,
    val filip: MemberId,
)

/** The categories the group adds to the built-in one. */
internal data class Categories(
    val food: CategoryId,
    val lodging: CategoryId,
    val transport: CategoryId,
)

internal fun eur(text: String): Money = Money.parse(text, CurrencyCode.of("EUR"))

internal fun ron(text: String): Money = Money.parse(text, CurrencyCode.of("RON"))

/** A date of the trip, in October 2026. */
internal fun october(date: Int): CalendarDate = CalendarDate.of(year = 2026, month = 10, day = date)

internal fun name(
    group: ExpenseGroup,
    member: MemberId,
): String = names(group, listOf(member))

/** [lines], one below the other in a table cell. */
internal fun cell(lines: List<String>): String = lines.joinToString("\n")

/** "valid: …" with [shown] of the value [make] returns, or "refused: …" with the message. */
internal fun <Value> outcome(
    make: () -> Value,
    shown: (Value) -> String,
): String =
    try {
        "valid: ${shown(make())}"
    } catch (refused: EranosException) {
        "refused: ${refused.message}"
    }

/** What the library refuses [change] with, or `null` for a change it accepts. */
internal fun refusal(change: () -> Unit): EranosException? =
    try {
        change()
        null
    } catch (refused: EranosException) {
        refused
    }

/** The message of [refused], or "accepted". */
internal fun message(refused: EranosException?): String = refused?.message ?: "accepted"

/** The sum of [amounts], all in [currency]. */
internal fun sum(
    currency: CurrencyCode,
    amounts: List<Money>,
): Money = amounts.fold(Money.zero(currency)) { total, amount -> total + amount }

/** The rate [cost] was entered at: 1 for a cost in the group currency. */
internal fun rate(cost: Cost): ExchangeRate =
    when (cost) {
        is Cost.Local -> ExchangeRate.parse("1")
        is Cost.Converted -> cost.rate
    }
