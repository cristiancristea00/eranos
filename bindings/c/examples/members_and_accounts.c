// A flat's members with their contact and account details, what validation accepts of each, and a flatmate who moves
// out. The port of the Rust example `members_and_accounts`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdlib.h>
#include <string.h>

/// The two flatmates.
typedef struct Flatmates {
    /// Who holds every detail.
    Id ana;
    /// Who moves out.
    Id bob;
} Flatmates;

/// A row of the table of members: a member's name, contact details, and accounts.
typedef struct MemberRow {
    /// Their accounts, one per line.
    Line accounts;
    /// Their email, or nothing.
    Line email;
    /// Their first and last name.
    Line name;
    /// Their phone number, or nothing.
    Line phone;
} MemberRow;

/// A table of members, its columns from @p numbersFrom on aligned to the right.
static Table *memberTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Member", "Email", "Phone", "Accounts"}, numbersFrom);
}

/// Adds @p row to @p table, a table of members, its cells in the order of the table's headers.
static void addMemberRow(Table *const table, MemberRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->email, row->phone, row->accounts});
}

/// A row of the table of checks: a text, what it was checked as, and what validation made of it.
typedef struct CheckRow {
    /// The text checked.
    Line input;
    /// What it was checked as.
    Line kind;
    /// The value it makes, or its refusal.
    Line result;
} CheckRow;

/// A table of checks, its columns from @p numbersFrom on aligned to the right.
static Table *checkTable(size_t const numbersFrom) { return tableCreate(3, (char const *const[]){"Input", "Checked as", "Result"}, numbersFrom); }

/// Adds @p row to @p table, a table of checks, its cells in the order of the table's headers.
static void addCheckRow(Table *const table, CheckRow const *const row) { tableAdd(table, (Line const[]){row->input, row->kind, row->result}); }

/// A row of the table of balances: whether a member is active, what they consumed, and their balance.
typedef struct BalanceRow {
    /// What is left: what they paid minus what they consumed.
    Line balance;
    /// What they consumed.
    Line consumed;
    /// Their first name.
    Line name;
    /// Whether they are active or archived.
    Line status;
} BalanceRow;

/// A table of balances, its columns from @p numbersFrom on aligned to the right.
static Table *balanceTable(size_t const numbersFrom) {
    return tableCreate(4, (char const *const[]){"Member", "Status", "Consumed", "Balance"}, numbersFrom);
}

/// Adds @p row to @p table, a table of balances, its cells in the order of the table's headers.
static void addBalanceRow(Table *const table, BalanceRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->status, row->consumed, row->balance});
}

/// A check of the library that writes the value it makes of a text, or refuses it.
typedef EranosError *(*Validation)(char const *text, EranosText *value);

/// The account @p name, paid to @p details; destroys @p details.
static EranosAccount *accountOf(EranosAccountDetails *const details, char const *const name) {
    EranosAccount *account = nullptr;
    must(eranosAccountCreate(details, name, &account));
    eranosAccountDetailsDestroy(details);
    return account;
}

/// The label and the details of every account of @p member, one per line.
static Line accountsOf(EranosMember const *const member) {
    EranosAccountList *const accounts = eranosMemberAccounts(member);
    EranosText *const name = eranosTextCreate();
    EranosText *const text = eranosTextCreate();
    Cell cell = {.lines = 0, .text = {}};
    for (size_t index = 0; eranosAccountListLength(accounts) > index; ++index) {
        EranosAccount *const account = eranosAccountListGet(accounts, index);
        eranosAccountName(account, name);
        EranosAccountDetails *const details = eranosAccountDetails(account);
        if (!eranosAccountDetailsIban(details, text)) {
            (void)eranosAccountDetailsUsername(details, text);
        }
        Line const line = lineOf("%s: %s", eranosTextData(name), eranosTextData(text));
        cellAdd(&cell, line.text);
        eranosAccountDetailsDestroy(details);
        eranosAccountDestroy(account);
    }
    eranosTextDestroy(text);
    eranosTextDestroy(name);
    eranosAccountListDestroy(accounts);
    return cell.text;
}

/// The first and the last name of @p member, the last when there is one.
static Line fullName(EranosMember const *const member) {
    EranosText *const first = eranosTextCreate();
    EranosText *const last = eranosTextCreate();
    eranosMemberFirstName(member, first);
    Line const line = eranosMemberLastName(member, last) ? lineOf("%s %s", eranosTextData(first), eranosTextData(last)) : textLine(first);
    eranosTextDestroy(last);
    eranosTextDestroy(first);
    return line;
}

/// The phone number of @p member, grouped for reading, or nothing when there is none.
static Line phoneOf(EranosMember const *const member) {
    EranosText *const text = eranosTextCreate();
    Line line = {};
    if (eranosMemberPhone(member, text)) {
        EranosText *const international = eranosTextCreate();
        must(eranosPhoneNumberInternational(eranosTextData(text), international));
        line = textLine(international);
        eranosTextDestroy(international);
    }
    eranosTextDestroy(text);
    return line;
}

/// The email of @p member, or nothing when there is none.
static Line emailOf(EranosMember const *const member) {
    EranosText *const text = eranosTextCreate();
    Line const line = eranosMemberEmail(member, text) ? textLine(text) : lineOf("%s", "");
    eranosTextDestroy(text);
    return line;
}

/// Prints every member of @p group with their contact and account details.
static void printMembers(EranosGroup const *const group) {
    EranosMemberList *members = nullptr;
    must(eranosGroupMembers(group, &members));
    Table *const table = memberTable(4);
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        addMemberRow(table, &(MemberRow){
                                .accounts = accountsOf(member),
                                .email = emailOf(member),
                                .name = fullName(member),
                                .phone = phoneOf(member),
                            });
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosMemberListDestroy(members);
}

/// Prints whether each member of @p group is active or archived, what they consumed, and their balance.
static void printBalances(EranosGroup const *const group) {
    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMemberList *members = nullptr;
    must(eranosGroupMembers(group, &members));
    Table *const table = balanceTable(2);
    EranosText *const text = eranosTextCreate();
    for (size_t index = 0; eranosMemberListLength(members) > index; ++index) {
        EranosMember *const member = eranosMemberListGet(members, index);
        (void)eranosMemberId(member, text);
        bool archived = false;
        must(eranosGroupIsArchived(group, eranosTextData(text), &archived));
        EranosMemberSummary *const found = totalsOf(summary, eranosTextData(text));
        check(nullptr != found, "every member has totals");
        eranosMemberFirstName(member, text);
        addBalanceRow(table, &(BalanceRow){
                                 .balance = balanceLine(eranosMemberSummaryBalance(found)),
                                 .consumed = moneyLine(eranosMemberSummaryShare(found)),
                                 .name = textLine(text),
                                 .status = lineOf("%s", archived ? "archived" : "active"),
                             });
        eranosMemberSummaryDestroy(found);
        eranosMemberDestroy(member);
    }
    tablePrint(table);
    eranosTextDestroy(text);
    eranosMemberListDestroy(members);
    eranosSummaryDestroy(summary);
}

/// Adds Ana, with every detail, and Bob, with a first name only, to @p group, and prints them.
static Flatmates members(EranosGroup *const group) {
    section("1. Members");
    EranosMember *member = nullptr;
    must(eranosMemberCreate("Ana", &member));
    EranosAccountDetails *iban = nullptr;
    must(eranosAccountDetailsCreateIban("ro49 aaaa 1b31 0075 9384 0000", &iban));
    EranosAccountDetails *username = nullptr;
    must(eranosAccountDetailsCreateUsername("@ana", &username));
    EranosAccount *const bank = accountOf(iban, "ING");
    EranosAccount *const card = accountOf(username, "Revolut");
    EranosAccountList *const accounts = eranosAccountListCreate();
    eranosAccountListPush(accounts, bank);
    eranosAccountListPush(accounts, card);
    eranosMemberSetAccounts(member, accounts);
    must(eranosMemberSetEmail(member, "ana@example.com"));
    must(eranosMemberSetLastName(member, "Popescu"));
    must(eranosMemberSetPhone(member, "+40 712 345 678"));
    EranosText *const id = eranosTextCreate();
    must(eranosGroupAddMember(group, member, id));
    Id const ana = idOf(id);
    eranosTextDestroy(id);
    eranosAccountListDestroy(accounts);
    eranosAccountDestroy(card);
    eranosAccountDestroy(bank);
    eranosMemberDestroy(member);
    Id const bob = addMember(group, "Bob");
    printMembers(group);
    printLine("The phone number was stored in E.164 form, +40712345678, and the table shows it\n"
              "grouped for reading; the IBAN was stored in its electronic form, uppercase and\n"
              "without spaces, the way banking apps accept it.");
    return (Flatmates){.ana = ana, .bob = bob};
}

/// Adds to @p table the row of @p input, checked as @p kind by @p validation: the value it makes, or its refusal.
static void addValidated(Table *const table, char const *const kind, Validation const validation, char const *const input) {
    EranosText *const text = eranosTextCreate();
    EranosError *const refused = validation(input, text);
    bool const valid = nullptr == refused;
    Line const message = messageLine(refused);
    Line const result = valid ? lineOf("valid: %s", eranosTextData(text)) : lineOf("refused: %s", message.text);
    eranosTextDestroy(text);
    addCheckRow(table, &(CheckRow){.input = lineOf("%s", input), .kind = lineOf("%s", kind), .result = result});
}

/// Prints what validation makes of valid and mistyped IBANs, emails, and phone numbers.
static void validation(void) {
    section("2. What validation accepts");
    printLine("IBANs are checked against the SWIFT registry: the country, its length, which\n"
              "positions hold digits and which letters, and the mod-97 check digits, which catch\n"
              "any single mistyped character. Emails are checked against the rules of RFC 5321\n"
              "and 5322, which refuse the doubled dots and stray commas of hurried typing, and\n"
              "phone numbers against their country's numbering plan, stored in E.164 form.");
    Table *const table = checkTable(3);
    addValidated(table, "IBAN", eranosIbanParse, "ro49 aaaa 1b31 0075 9384 0000");
    addValidated(table, "IBAN", eranosIbanParse, "RO49 AAAA 1B31 0075 9384 0001");
    addValidated(table, "IBAN", eranosIbanParse, "RO49 AAAA 1B31 0075");
    addValidated(table, "IBAN", eranosIbanParse, "DE46 37O4 0044 0532 0130 00");
    addValidated(table, "email", eranosEmailParse, "ana@example.com");
    addValidated(table, "email", eranosEmailParse, "ana.example.com");
    addValidated(table, "email", eranosEmailParse, "ana..popescu@example.com");
    addValidated(table, "phone", eranosPhoneNumberParse, "+40 (712) 345-678");
    addValidated(table, "phone", eranosPhoneNumberParse, "0712 345 678");
    addValidated(table, "phone", eranosPhoneNumberParse, "+40 612 345 678");
    addValidated(table, "phone", eranosPhoneNumberParse, "+44 (0)20 7946 0958");
    tablePrint(table);
}

/// Whether @p error names @p member.
static bool errorNames(EranosError const *const error, Id const member) {
    EranosText *const text = eranosTextCreate();
    bool const same = eranosErrorMember(error, text) && 0 == strcmp(eranosTextData(text), member.text);
    eranosTextDestroy(text);
    return same;
}

/// Adds the rent that Ana and Bob share, archives Bob, and shows what that allows and refuses.
static void movingOut(EranosGroup *const group, Id const ana, Id const bob) {
    section("3. A flatmate moves out");
    Members const both = {.count = 2, .ids = {ana, bob}};
    Id const rent = addExpense(group, expenseOf(localCost(amountOf("900.00", "EUR")), (EranosCalendarDate){.day = 1, .month = 9, .year = 2026},
                                                "Rent", soleSplit(ana), equalSplit(&both)));
    must(eranosGroupArchiveMember(group, bob.text));
    printLine("Bob shared September's rent, then moved out. He is archived, not removed: his\n"
              "history and his balance stay, and he can still pay Ana back.");
    printBalances(group);

    EranosError *const refused =
        expenseRefusal(group, expenseOf(localCost(amountOf("60.00", "EUR")), (EranosCalendarDate){.day = 1, .month = 10, .year = 2026}, "Utilities",
                                        soleSplit(ana), equalSplit(&both)));
    check(isFailure(refused, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_ARCHIVED_MEMBER) && errorNames(refused, bob),
          "the archived member is refused");
    Line const refusedMessage = messageLine(refused);
    printFormatted("October's utilities cannot include him:\n"
                   "  error: %s",
                   refusedMessage.text);
    EranosMember *removed = nullptr;
    EranosError *const removal = eranosGroupRemoveMember(group, bob.text, &removed);
    check(isFailure(removal, ERANOS_ERROR_KIND_GROUP, ERANOS_FAILURE_GROUP_MEMBER_IN_USE) && nullptr == removed, "the referenced member stays");
    Line const removalMessage = messageLine(removal);
    printFormatted("Nor can he be removed while the rent references him:\n"
                   "  error: %s",
                   removalMessage.text);

    EranosSummary *summary = nullptr;
    must(eranosGroupSummary(group, &summary));
    EranosMemberSummary *const totals = totalsOf(summary, bob.text);
    check(nullptr != totals, "Bob has totals");
    EranosBalance *const balance = eranosMemberSummaryBalance(totals);
    EranosMoney *const owed = eranosBalanceAmount(balance);
    check(ERANOS_BALANCE_KIND_OWES == eranosBalanceKind(balance) && 45'000 == eranosMoneyMinorUnits(owed), "Bob owes 450.00 EUR");
    EranosExpense *expense = nullptr;
    must(eranosGroupExpense(group, rent.text, &expense));
    check(nullptr != expense, "the group keeps the rent");
    eranosExpenseDestroy(expense);
    eranosMoneyDestroy(owed);
    eranosBalanceDestroy(balance);
    eranosMemberSummaryDestroy(totals);
    eranosSummaryDestroy(summary);
}

int main(void) {
    EranosNewGroup *details = nullptr;
    must(eranosNewGroupCreate("EUR", "Flat 4B", &details));
    EranosGroup *group = nullptr;
    must(eranosGroupCreate(details, &group));
    eranosNewGroupDestroy(details);
    printLine("Flat 4B: members and their accounts\n"
              "═══════════════════════════════════");
    printLine("Ana and Bob share a flat. A member needs only a first name; the last name,\n"
              "email, phone, and the accounts they can be paid to are optional.");
    Flatmates const flatmates = members(group);
    validation();
    movingOut(group, flatmates.ana, flatmates.bob);
    eranosGroupDestroy(group);
    return EXIT_SUCCESS;
}
