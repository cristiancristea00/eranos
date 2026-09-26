// A flat's members with their contact and account details, what validation accepts of each, and a flatmate who moves
// out. The port of the Rust example `members_and_accounts`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) { return Money::parse(text, Currency::of("EUR")); }

/// The two flatmates.
struct Flatmates final {
    /// Who holds every detail.
    MemberId ana;
    /// Who moves out.
    MemberId bob;
};

/// An account with its label and its details.
[[nodiscard]] std::string describe(Account const &account) {
    auto const *const iban = std::get_if<IbanAccountDetails>(&account.details);
    return account.name + ": " + (nullptr == iban ? std::get<UsernameAccountDetails>(account.details).username.toString() : iban->iban.toString());
}

/// "valid: " and what @p shown writes of the value @p parse makes of @p input, or "refused: " and the library's message.
template <typename Parse, typename Shown>
[[nodiscard]] std::vector<std::string> validated(std::string const &input, std::string const &kind, Parse const &parse, Shown const &shown) {
    std::string result;
    auto const refused = refusal([&] {
        result = "valid: " + shown(parse(input));
        return 0;
    });
    return {input, kind, refused.has_value() ? "refused: " + message(refused) : result};
}

/// Prints every member of @p group with their contact and account details.
void printMembers(Group const &group) {
    Rows rows;
    for (auto const &member : group.members()) {
        std::vector<std::string> accounts;
        accounts.reserve(member.accounts.size());
        for (auto const &account : member.accounts) {
            accounts.push_back(describe(account));
        }
        rows.push_back({
            member.lastName.has_value() ? member.firstName + " " + *member.lastName : member.firstName,
            member.email.has_value() ? member.email->toString() : "",
            member.phone.has_value() ? member.phone->international() : "",
            cell(accounts),
        });
    }
    printTable({"Member", "Email", "Phone", "Accounts"}, rows, 4);
}

/// Prints whether each member of @p group is active or archived, what they consumed, and their balance.
void printBalances(Group const &group) {
    auto const summary = group.summary();
    Rows rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id.value()).value();
        rows.push_back(
            {member.firstName, group.isArchived(member.id.value()) ? "archived" : "active", found.share.toString(), toString(found.balance)});
    }
    printTable({"Member", "Status", "Consumed", "Balance"}, rows, 2);
}

/// Adds Ana, with every detail, and Bob, with a first name only, to @p group, and prints them.
[[nodiscard]] Flatmates members(Group &group) {
    section("1. Members");
    Member first("Ana");
    first.accounts = {
        Account(IbanAccountDetails{Iban::parse("ro49 aaaa 1b31 0075 9384 0000")}, "ING"),
        Account(UsernameAccountDetails{Username::parse("@ana")}, "Revolut"),
    };
    first.email = Email::parse("ana@example.com");
    first.lastName = "Popescu";
    first.phone = PhoneNumber::parse("+40 712 345 678");
    auto const ana = group.addMember(first);
    auto const bob = group.addMember(Member("Bob"));
    printMembers(group);
    printLine("The phone number was stored in E.164 form, +40712345678, and the table shows it\n"
              "grouped for reading; the IBAN was stored in its electronic form, uppercase and\n"
              "without spaces, the way banking apps accept it.");
    return {ana, bob};
}

/// Prints what validation makes of valid and mistyped IBANs, emails, and phone numbers.
void validation() {
    section("2. What validation accepts");
    printLine("IBANs are checked against the SWIFT registry: the country, its length, which\n"
              "positions hold digits and which letters, and the mod-97 check digits, which catch\n"
              "any single mistyped character. Emails are checked against the rules of RFC 5321\n"
              "and 5322, which refuse the doubled dots and stray commas of hurried typing, and\n"
              "phone numbers against their country's numbering plan, stored in E.164 form.");
    auto const iban = [](std::string const &input) {
        return validated(
            input, "IBAN", [](std::string const &text) { return Iban::parse(text); }, [](Iban const &parsed) { return parsed.toString(); });
    };
    auto const email = [](std::string const &input) {
        return validated(
            input, "email", [](std::string const &text) { return Email::parse(text); }, [](Email const &parsed) { return parsed.toString(); });
    };
    auto const phone = [](std::string const &input) {
        return validated(
            input, "phone", [](std::string const &text) { return PhoneNumber::parse(text); },
            [](PhoneNumber const &parsed) { return parsed.toString(); });
    };
    printTable({"Input", "Checked as", "Result"},
               {
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
               },
               3);
}

/// Adds the rent that Ana and Bob share, archives Bob, and shows what that allows and refuses.
void movingOut(Group &group, Flatmates const &flatmates) {
    section("3. A flatmate moves out");
    WholeExpenseSplit const both{EqualSplit{{flatmates.ana, flatmates.bob}}};
    auto const rent = group.addExpense(Expense(LocalCost{euros("900.00")}, CalendarDate::of(1, 9, 2026), "Rent", EqualSplit{{flatmates.ana}}, both));
    group.archiveMember(flatmates.bob);
    printLine("Bob shared September's rent, then moved out. He is archived, not removed: his\n"
              "history and his balance stay, and he can still pay Ana back.");
    printBalances(group);

    auto const refused = refusal([&] {
        return group.addExpense(Expense(LocalCost{euros("60.00")}, CalendarDate::of(1, 10, 2026), "Utilities", EqualSplit{{flatmates.ana}}, both));
    });
    check(refused.has_value(), "the archived member is refused");
    printLine("October's utilities cannot include him:\n  error: " + message(refused));
    auto const removal = refusal([&] { return group.removeMember(flatmates.bob); });
    check(removal.has_value(), "the referenced member stays");
    printLine("Nor can he be removed while the rent references him:\n  error: " + message(removal));

    check(Failure::GROUP_ARCHIVED_MEMBER == refused->failure() && flatmates.bob == refused->member(), "the refusal names Bob");
    check(Failure::GROUP_MEMBER_IN_USE == removal->failure(), "Bob is in use");
    check(Balance(OwesBalance{euros("450.00")}) == totalsOf(group.summary(), flatmates.bob).value().balance, "Bob owes 450.00 EUR");
    check(group.expense(rent).has_value(), "the group keeps the rent");
}

/// Runs the example.
void run() {
    auto group = Group::create(Currency::of("EUR"), "Flat 4B");
    printLine("Flat 4B: members and their accounts\n"
              "═══════════════════════════════════");
    printLine("Ana and Bob share a flat. A member needs only a first name; the last name,\n"
              "email, phone, and the accounts they can be paid to are optional.");
    auto const flatmates = members(group);
    validation();
    movingOut(group, flatmates);
}

} // namespace
} // namespace eranos::examples

int main() { return eranos::examples::runPort(eranos::examples::run); }
