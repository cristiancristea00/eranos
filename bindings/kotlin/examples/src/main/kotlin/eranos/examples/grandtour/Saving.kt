// Section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed.
package eranos.examples.grandtour

import eranos.AccountDetails
import eranos.EranosException
import eranos.Group
import eranos.GroupFailure
import eranos.examples.Row
import eranos.examples.printTable
import eranos.examples.section

private data class LoadRow(
    val change: String,
    val message: String,
) : Row {
    override val cells: List<String>
        get() = listOf(change, message)

    companion object {
        val HEADERS = listOf("Change to the file", "Message")
    }
}

/** A change made to a saved group, and the text it leaves. */
private data class Tampering(
    val change: String,
    val text: String,
)

/** What loading the saved form [text] refuses: the error the replay of its entries reports. */
internal fun loadRefusal(text: String): EranosException = checkNotNull(refusal { Group.loadBlocking(text).close() }) { "the change must be refused" }

internal fun savingAndLoading(
    group: Group,
    people: People,
    categories: Categories,
) {
    section("12. Saving and loading")
    val saved = group.saveBlocking()
    println(
        """
        The group saves to ${saved.encodeToByteArray().size} bytes of JSON: its details and entries, never the totals.
        Ana's accounts are saved in clear text, as the library holds them:
        """.trimIndent(),
    )
    val ana = checkNotNull(group.memberBlocking(people.ana)) { "Ana is a member" }
    for (account in ana.accounts) {
        val details =
            when (val held = account.details) {
                is AccountDetails.Iban -> held.iban.toString()
                is AccountDetails.Username -> held.username.toString()
            }
        check(details in saved) { "$details is saved as it is" }
        println("  ${account.name}: $details")
    }
    Group.loadBlocking(saved).use { loaded ->
        println(
            """
            Loading replays every entry through the same checks as adding it and rebuilds the
            totals: the loaded group equals the saved one. Reading a SavedGroup and loading it
            with into_group reports a refusal as a typed error instead of a message:
            """.trimIndent(),
        )
        val tampered =
            listOf(
                Tampering(change = "\"format\": 3", text = saved.replaceFirst("\"format\": 2", "\"format\": 3")),
                Tampering(change = "a blank group name", text = saved.replaceFirst("\"name\": \"Lisbon and Porto\"", "\"name\": \" \"")),
                Tampering(change = "Lodging renamed FOOD", text = saved.replaceFirst("\"name\": \"Lodging\"", "\"name\": \"FOOD\"")),
            )
        val rows = mutableListOf<LoadRow>()
        val errors = mutableListOf<EranosException>()
        for (tampering in tampered) {
            check(saved != tampering.text) { "${tampering.change} changes the file" }
            val error = loadRefusal(tampering.text)
            rows.add(LoadRow(change = tampering.change, message = message(error)))
            errors.add(error)
        }
        printTable(LoadRow.HEADERS, rows, numbersFrom = 2)

        check(saved == loaded.saveBlocking())
        check(group.summaryBlocking() == loaded.summaryBlocking())
        check(
            listOf(GroupFailure.UnsupportedFormat(3), GroupFailure.EmptyName, GroupFailure.DuplicateCategoryName(categories.food)) ==
                errors.map { error -> (error as? EranosException.Group)?.failure },
        )
    }
}
