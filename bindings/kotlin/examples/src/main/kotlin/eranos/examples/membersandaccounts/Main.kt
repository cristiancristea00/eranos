// A flat's members with their contact and account details, what validation accepts of each, and a flatmate who moves
// out. The port of the Rust example `members_and_accounts`, which prints the same report.
package eranos.examples.membersandaccounts

import eranos.Balance
import eranos.CalendarDate
import eranos.Cost
import eranos.CurrencyCode
import eranos.Email
import eranos.EranosException
import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseGroupFailure
import eranos.ExpenseSplit
import eranos.Iban
import eranos.Member
import eranos.MemberId
import eranos.Money
import eranos.PaymentAccount
import eranos.PaymentAccountDetails
import eranos.PhoneNumber
import eranos.Split
import eranos.Username
import eranos.examples.Row
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.totals

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

private data class BalanceRow(
    val name: String,
    val status: String,
    val consumed: Money,
    val balance: Balance,
) : Row {
    override val cells: List<String>
        get() = listOf(name, status, "$consumed", "$balance")

    companion object {
        val HEADERS = listOf("Member", "Status", "Consumed", "Balance")
    }
}

/** The two flatmates: Ana, who gives every detail, and Bob, who moves out. */
private data class Flatmates(
    val ana: MemberId,
    val bob: MemberId,
)

/** An account with its label and its details. */
private fun describe(account: PaymentAccount): String {
    val details =
        when (val details = account.details) {
            is PaymentAccountDetails.Iban -> details.iban.toString()
            is PaymentAccountDetails.Username -> details.username.toString()
        }
    return "${account.name}: $details"
}

/** `valid: ` and what [shown] writes of the value [make] returns, or `refused: ` and the library's message. */
private fun <Value> outcome(
    make: () -> Value,
    shown: (Value) -> String,
): String =
    try {
        "valid: ${shown(make())}"
    } catch (refusal: EranosException) {
        "refused: ${refusal.message}"
    }

private fun printMembers(group: ExpenseGroup) {
    printTable(
        MemberRow.HEADERS,
        group.membersBlocking().map { member ->
            MemberRow(
                name = listOfNotNull(member.firstName, member.lastName).joinToString(" "),
                email = member.email?.toString().orEmpty(),
                phone = member.phone?.toString().orEmpty(),
                accounts = member.accounts.joinToString("\n", transform = ::describe),
            )
        },
        numbersFrom = 4,
    )
}

private fun printBalances(group: ExpenseGroup) {
    val summary = group.summaryBlocking()
    val rows =
        group.membersBlocking().mapNotNull { member ->
            member.id?.let { id ->
                totals(id, summary)?.let { totals ->
                    BalanceRow(
                        name = member.firstName,
                        status = if (group.isArchivedBlocking(id)) "archived" else "active",
                        consumed = totals.share,
                        balance = totals.balance,
                    )
                }
            }
        }
    printTable(BalanceRow.HEADERS, rows, numbersFrom = 2)
}

private fun members(group: ExpenseGroup): Flatmates {
    section("1. Members")
    val ana =
        group.addBlocking(
            Member(
                "Ana",
                lastName = "Popescu",
                email = Email.parse("ana@example.com"),
                phone = PhoneNumber.parse("+40 712 345 678"),
                accounts =
                    listOf(
                        PaymentAccount("ING", PaymentAccountDetails.Iban(Iban.parse("ro49 aaaa 1b31 0075 9384 0000"))),
                        PaymentAccount("Revolut", PaymentAccountDetails.Username(Username.parse("@ana"))),
                    ),
            ),
        )
    val bob = group.addBlocking(Member("Bob"))
    printMembers(group)
    println(
        """
        The phone number was stored in E.164 form, +40712345678, and the table shows it
        grouped for reading; the IBAN was stored in its electronic form, uppercase and
        without spaces, the way banking apps accept it.
        """.trimIndent(),
    )
    return Flatmates(ana, bob)
}

private fun validation() {
    section("2. What validation accepts")
    println(
        """
        IBANs are checked against the SWIFT registry: the country, its length, which
        positions hold digits and which letters, and the mod-97 check digits, which catch
        any single mistyped character. Emails are checked against the rules of RFC 5321
        and 5322, which refuse the doubled dots and stray commas of hurried typing, and
        phone numbers against their country's numbering plan, stored in E.164 form.
        """.trimIndent(),
    )
    val iban = { input: String -> CheckRow(input, "IBAN", outcome({ Iban.parse(input) }, Iban::toString)) }
    val email = { input: String -> CheckRow(input, "email", outcome({ Email.parse(input) }, Email::toString)) }
    val phone = { input: String -> CheckRow(input, "phone", outcome({ PhoneNumber.parse(input) }, PhoneNumber::e164)) }
    printTable(
        CheckRow.HEADERS,
        listOf(
            iban("ro49 aaaa 1b31 0075 9384 0000"),
            iban("RO49 AAAA 1B31 0075 9384 0001"),
            iban("RO49 AAAA 1B31 0075"),
            iban("DE46 37O4 0044 0532 0130 00"),
            email("ana@example.com"),
            email("ana.example.com"),
            email("ana..popescu@example.com"),
            phone("+40 (712) 345-678"),
            phone("0712 345 678"),
            phone("+40 612 345 678"),
            phone("+44 (0)20 7946 0958"),
        ),
        numbersFrom = 3,
    )
}

/** What adding [change] to the group refuses it with. */
private fun refusal(change: () -> Unit): EranosException =
    try {
        change()
        error("the change must be refused")
    } catch (refused: EranosException) {
        refused
    }

private fun movingOut(
    group: ExpenseGroup,
    flatmates: Flatmates,
) {
    section("3. A flatmate moves out")
    val ana = flatmates.ana
    val bob = flatmates.bob
    val euro = CurrencyCode.of("EUR")
    val both = ExpenseSplit.Whole(Split.Equal(listOf(ana, bob)))
    val rent =
        group.addBlocking(
            Expense(
                "Rent",
                Cost.Local(Money.parse("900.00", euro)),
                Split.Equal(listOf(ana)),
                both,
                CalendarDate.of(year = 2026, month = 9, day = 1),
            ),
        )
    group.archiveBlocking(bob)
    println(
        """
        Bob shared September's rent, then moved out. He is archived, not removed: his
        history and his balance stay, and he can still pay Ana back.
        """.trimIndent(),
    )
    printBalances(group)

    val utilities =
        Expense(
            "Utilities",
            Cost.Local(Money.parse("60.00", euro)),
            Split.Equal(listOf(ana)),
            both,
            CalendarDate.of(year = 2026, month = 10, day = 1),
        )
    val refused = refusal { group.addBlocking(utilities) }
    println("October's utilities cannot include him:\n  error: ${refused.message}")
    val removal = refusal { group.removeBlocking(bob) }
    println("Nor can he be removed while the rent references him:\n  error: ${removal.message}")

    check((refused as? EranosException.ExpenseGroup)?.failure == ExpenseGroupFailure.ArchivedMember(bob))
    check((removal as? EranosException.ExpenseGroup)?.failure is ExpenseGroupFailure.MemberInUse)
    check(Balance.Owes(Money.parse("450.00", euro)) == totals(bob, group.summaryBlocking())?.balance)
    check(null != group.expenseBlocking(rent))
}

/** Prints the report of the flat's members. */
fun main() {
    ExpenseGroup("Flat 4B", CurrencyCode.of("EUR")).use { group ->
        println("Flat 4B: members and their accounts")
        println("═══════════════════════════════════")
        println(
            """
            Ana and Bob share a flat. A member needs only a first name; the last name,
            email, phone, and the accounts they can be paid to are optional.
            """.trimIndent(),
        )
        val flatmates = members(group)
        validation()
        movingOut(group, flatmates)
    }
}
