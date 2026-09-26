// Every feature of the library in one story: a group and its details, members with contact details and accounts,
// categories, an expense for every way of dividing one, another currency, edits and refusals, members who come and go,
// the summary, a new group currency, settling up, saving and loading, the values on their own, and the group shared
// between coroutines. Every section checks its key numbers. The port of the Rust example `grand_tour`, which prints
// the same report.
package eranos.examples.grandtour

import kotlinx.coroutines.runBlocking

/** Prints the report of the trip to Lisbon and Porto. */
fun main() {
    println("Lisbon and Porto: a grand tour of eranos")
    println("════════════════════════════════════════")
    println(
        """
        Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each
        section below uses another part of the library, from creating the group to saving
        it, and checks its key numbers.
        """.trimIndent(),
    )
    theGroup().use { group ->
        val people = members(group)
        val categories = addCategories(group)
        val seafood = expenses(group, people, categories)
        anotherCurrency(group, people, categories)
        edits(group, people, categories, seafood)
        comingsAndGoings(group, people)
        summary(group, people, categories)
        removingACategory(group, categories)
        newCurrency(group, people)
        settlingUp(group)
        savingAndLoading(group, people, categories)
        values(group, people)
        runBlocking { sharing(group, people) }
    }
}
