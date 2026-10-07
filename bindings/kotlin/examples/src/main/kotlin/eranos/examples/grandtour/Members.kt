// Sections 1 to 3 of the tour: the group and its details, the members with their contact details and accounts, and
// the categories.
package eranos.examples.grandtour

import eranos.Category
import eranos.CategoryId
import eranos.CurrencyCode
import eranos.Email
import eranos.EranosException
import eranos.ExpenseGroup
import eranos.ExpenseGroupFailure
import eranos.Iban
import eranos.Language
import eranos.Member
import eranos.PaymentAccount
import eranos.PaymentAccountDetails
import eranos.PhoneNumber
import eranos.Username
import eranos.examples.Row
import eranos.examples.printTable
import eranos.examples.section

private data class DetailRow(
    val detail: String,
    val value: String,
) : Row {
    override val cells: List<String>
        get() = listOf(detail, value)

    companion object {
        val HEADERS = listOf("Detail", "Value")
    }
}

private data class MemberRow(
    val name: String,
    val email: String,
    val phone: String,
    val accounts: String,
) : Row {
    override val cells: List<String>
        get() = listOf(name, email, phone, accounts)

    companion object {
        val HEADERS = listOf("Member", "Email", "Phone", "Accounts")
    }
}

private data class CheckRow(
    val input: String,
    val kind: String,
    val result: String,
) : Row {
    override val cells: List<String>
        get() = listOf(input, kind, result)

    companion object {
        val HEADERS = listOf("Input", "Checked as", "Result")
    }
}

internal fun theGroup(): ExpenseGroup {
    section("1. The group")
    val group = ExpenseGroup("Lisbon", CurrencyCode.of("EUR"), "A week by the Atlantic", Language.parse("pt-pt"))
    val blank = refusal { group.renameBlocking("   ") }
    group.renameBlocking("Lisbon and Porto")
    group.setDescriptionBlocking("A week by the Atlantic, with a day in Porto")
    printTable(
        DetailRow.HEADERS,
        listOf(
            DetailRow("Name", group.nameBlocking()),
            DetailRow("Currency", "${group.currencyBlocking()}"),
            DetailRow("Language", group.languageBlocking()?.toString().orEmpty()),
            DetailRow("Description", group.descriptionBlocking().orEmpty()),
        ),
        numbersFrom = 2,
    )
    println(
        """
        The group started as Lisbon and was renamed when a day in Porto joined the plan.
        Every total is kept in its currency, ${group.currencyBlocking()}. The language is a BCP 47 tag the apps
        show the group in, stored in canonical form: pt-pt became ${group.languageBlocking()?.toString().orEmpty()}. A blank name is
        refused:
          error: ${message(blank)}
        """.trimIndent(),
    )

    check(ExpenseGroupFailure.EmptyName == (blank as? EranosException.ExpenseGroup)?.failure)
    check("Lisbon and Porto" == group.nameBlocking())
    check("pt-PT" == group.languageBlocking()?.toString())
    return group
}

/** An account with its label, its kind, and its details. */
internal fun describe(account: PaymentAccount): String =
    when (val details = account.details) {
        is PaymentAccountDetails.Iban -> "${account.name}, IBAN ${details.iban}"
        is PaymentAccountDetails.Username -> "${account.name}, username ${details.username}"
    }

internal fun printMembers(group: ExpenseGroup) {
    printTable(
        MemberRow.HEADERS,
        group.membersBlocking().map { member ->
            MemberRow(
                name = listOfNotNull(member.firstName, member.lastName).joinToString(" "),
                email = member.email?.toString().orEmpty(),
                phone = member.phone?.toString().orEmpty(),
                accounts = cell(member.accounts.map(::describe)),
            )
        },
        numbersFrom = 4,
    )
}

internal fun validation(group: ExpenseGroup) {
    println(
        """
        IBANs are checked against the SWIFT registry and their mod-97 check digits,
        emails against RFC 5321 and 5322, and phone numbers against their country's
        numbering plan. Required text must not be blank:
        """.trimIndent(),
    )
    val iban = { input: String ->
        CheckRow("\"$input\"", "IBAN", outcome({ Iban.parse(input) }, { iban -> "$iban, country ${iban.countryCode}" }))
    }
    val email = { input: String -> CheckRow("\"$input\"", "email", outcome({ Email.parse(input) }, Email::toString)) }
    val phone = { input: String ->
        CheckRow("\"$input\"", "phone", outcome({ PhoneNumber.parse(input) }, { phone -> "$phone, stored as ${phone.e164}" }))
    }
    printTable(
        CheckRow.HEADERS,
        listOf(
            iban("ro49 aaaa 1b31 0075 9384 0000"),
            iban("RO49 AAAA 1B31 0075 9384 0001"),
            email("ana..popescu@example.com"),
            phone("+351 912 345 678"),
            phone("+40 612 345 678"),
            CheckRow("\"   \"", "username", outcome({ Username.parse("   ") }, Username::toString)),
            CheckRow("\"  \"", "first name", outcome({ group.addBlocking(Member("  ")) }, { id -> id.value })),
        ),
        numbersFrom = 3,
    )
}

internal fun members(group: ExpenseGroup): People {
    section("2. Members")
    val ana =
        group.addBlocking(
            Member(
                "Ana",
                lastName = "Popescu",
                email = Email.parse("ana@example.com"),
                phone = PhoneNumber.parse("+40 712 345 678"),
                accounts =
                    listOf(
                        PaymentAccount("ING", PaymentAccountDetails.Iban(Iban.parse("RO49 AAAA 1B31 0075 9384 0000"))),
                        PaymentAccount("Revolut", PaymentAccountDetails.Username(Username.parse("@ana"))),
                    ),
            ),
        )
    val bob = group.addBlocking(Member("Bob"))
    val carol = group.addBlocking(Member("Carol"))
    val dan = group.addBlocking(Member("Dan"))
    val eva = group.addBlocking(Member("Eva"))
    val filip = group.addBlocking(Member("Filip"))
    // An edit starts from a copy of the member, which keeps its identifier.
    val edited = checkNotNull(group.memberBlocking(bob)) { "Bob is a member" }.copy(lastName = "Ionescu", email = Email.parse("bob@example.com"))
    val before = group.replaceBlocking(edited)
    printMembers(group)
    println(
        """
        Only a first name is required. Ana gave every detail; Bob added his last name and
        email later, through an edit that kept his identifier:
          $bob
        Eva is Carol's daughter, and Filip was added by mistake.
        """.trimIndent(),
    )
    validation(group)

    check(null == before.lastName)
    check("Ionescu" == group.memberBlocking(bob)?.lastName)
    check(2 == group.memberBlocking(ana)?.accounts?.size)
    return People(ana, bob, carol, dan, eva, filip)
}

internal fun addCategories(group: ExpenseGroup): Categories {
    section("3. Categories")
    val food = group.addBlocking(Category("Food"))
    // An app that keeps its categories elsewhere gives them its own keys.
    val lodging = group.addBlocking(Category(CategoryId.parse("lodging"), "Lodging"))
    val transport = group.addBlocking(Category("Transport"))
    val homonym = refusal { group.addBlocking(Category("FOOD")) }
    val renamed = checkNotNull(group.categoryBlocking(transport)) { "Transport was added" }.copy(name = "Getting around")
    group.replaceBlocking(renamed)
    val other = checkNotNull(group.categoriesBlocking().firstOrNull()) { "every group has the built-in category" }.copy(name = "Other")
    val builtIn = refusal { group.replaceBlocking(other) }
    println(
        """
        Every group has the built-in Uncategorised; this one adds Food, Lodging, under the
        identifier lodging that the app gives it, and Transport, then renames Transport, which
        keeps its identifier. In identifier order, where lodging follows the generated ones:
          ${group.categoriesBlocking().joinToString(", ") { category -> category.name }}
        Names are compared caselessly, and the built-in category keeps its name:
        """.trimIndent(),
    )
    printTable(
        RefusalRow.HEADERS,
        listOf(
            RefusalRow("Add a category named FOOD", message(homonym)),
            RefusalRow("Rename Uncategorised to Other", message(builtIn)),
        ),
        numbersFrom = 2,
    )

    check(ExpenseGroupFailure.DuplicateCategoryName(food) == (homonym as? EranosException.ExpenseGroup)?.failure)
    check(ExpenseGroupFailure.BuiltInCategory == (builtIn as? EranosException.ExpenseGroup)?.failure)
    check("Getting around" == group.categoryBlocking(transport)?.name)
    check("lodging" == lodging.value)
    check(listOf(builtInCategory(group), food, transport, lodging) == group.categoriesBlocking().map { category -> category.id })
    return Categories(food, lodging, transport)
}
