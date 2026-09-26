// Section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed.
package eranos.examples.grandtour

import eranos.AccountDetails
import eranos.EranosException
import eranos.Group
import eranos.GroupFailure
import eranos.examples.printTable
import eranos.examples.section

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
        val refusals =
            listOf(
                "\"format\": 3" to saved.replaceFirst("\"format\": 2", "\"format\": 3"),
                "a blank group name" to saved.replaceFirst("\"name\": \"Lisbon and Porto\"", "\"name\": \" \""),
                "Lodging renamed FOOD" to saved.replaceFirst("\"name\": \"Lodging\"", "\"name\": \"FOOD\""),
            ).map { (change, text) ->
                check(saved != text) { "$change changes the file" }
                change to loadRefusal(text)
            }
        printTable(listOf("Change to the file", "Message"), refusals.map { (change, error) -> listOf(change, message(error)) }, numbersFrom = 2)

        check(saved == loaded.saveBlocking())
        check(group.summaryBlocking() == loaded.summaryBlocking())
        check(
            listOf(GroupFailure.UnsupportedFormat(3), GroupFailure.EmptyName, GroupFailure.DuplicateCategoryName(categories.food)) ==
                refusals.map { (_, error) -> (error as? EranosException.Group)?.failure },
        )
    }
}
