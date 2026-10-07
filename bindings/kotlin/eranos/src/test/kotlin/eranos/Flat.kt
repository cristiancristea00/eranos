// A group the tests share the shape of, and the check every value type's equality takes.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.fail

/** A group of two with a category, an expense, and a transfer, and the identifiers the group gave them. */
internal class Flat {
    val euro: Currency = Currency.of("EUR")
    val day: CalendarDate = CalendarDate.of(2026, 9, 23)
    val group: ExpenseGroup = ExpenseGroup("Flat", euro, "Rent", Language.parse("pt-pt"))
    val ana: MemberId = group.addBlocking(Member("Ana"))
    val bob: MemberId = group.addBlocking(Member("Bob"))
    val food: CategoryId = group.addBlocking(Category("Food"))
    val dinner: ExpenseId =
        group.addBlocking(
            Expense(
                "Dinner",
                Cost.Local(euros(3_000)),
                Split.Equal(listOf(ana)),
                ExpenseSplit.Whole(Split.Equal(listOf(ana, bob))),
                day,
                category = food,
            ),
        )
    val repayment: TransferId = group.addBlocking(Transfer(bob, ana, Cost.Local(euros(1_000)), day))

    /** [minorUnits] cents. */
    fun euros(minorUnits: Long): Money = Money(minorUnits, euro)

    /** An expense of 30.00 EUR, called [name], that [payer] paid and [members] share equally. */
    fun expense(
        name: String,
        payer: MemberId,
        members: List<MemberId>,
    ): Expense = Expense(name, Cost.Local(euros(3_000)), Split.Equal(listOf(payer)), ExpenseSplit.Whole(Split.Equal(members)), day)
}

/**
 * Checks that [value] equals [same], with the same hash and text, and differs from [other]: a class that wrote its equality
 * out with a field left out would merge two values an app holds apart.
 */
internal fun <T : Any> assertValue(
    value: T,
    same: T,
    other: T,
) {
    assertEquals(value, same)
    assertEquals(value.hashCode(), same.hashCode())
    assertEquals(value.toString(), same.toString())
    assertNotEquals(value, other)
}

/** [value], which the test expects to be there: a missing one fails the test. */
internal fun <T : Any> present(value: T?): T = value ?: fail("expected a value, found none")
