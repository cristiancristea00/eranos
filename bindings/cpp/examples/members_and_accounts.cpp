// A flat's members with their contact and account details, what validation accepts of each, and a flatmate who moves
// out. The port of the Rust example `members_and_accounts`, which prints the same report.

#include "eranos.hpp"
#include "report.hpp"

#include <cstdlib>
#include <format>
#include <print>
#include <string>
#include <string_view>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// The amount @p text of euros.
[[nodiscard]] Money euros(std::string_view const text) noexcept { return must(Money::parse(text, must(CurrencyCode::of("EUR")))); }

/// The two flatmates.
struct Flatmates final {
    /// Who holds every detail.
    MemberId ana;
    /// Who moves out.
    MemberId bob;
};

/// A row of the table of members: a member's name, contact details, and accounts.
struct MemberRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<4> HEADERS{"Member", "Email", "Phone", "Accounts"};

    /// Their first and last name.
    std::string name;
    /// Their email, or nothing.
    std::string email;
    /// Their phone number, or nothing.
    std::string phone;
    /// Their accounts, one per line.
    std::string accounts;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] inline constexpr Cells<4> cells() const noexcept { return {name, email, phone, accounts}; }
};

/// A row of the table of checks: a text, what it was checked as, and what validation made of it.
struct CheckRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<3> HEADERS{"Input", "Checked as", "Result"};

    /// The text checked.
    std::string_view input;
    /// What it was checked as.
    std::string_view kind;
    /// The value it makes, or its refusal.
    std::string result;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] inline constexpr Cells<3> cells() const noexcept { return {input, kind, result}; }
};

/// A row of the table of balances: whether a member is active, what they consumed, and their balance.
struct BalanceRow final {
    /// The headers of the table's columns, in the order it prints them.
    static inline constexpr Cells<4> HEADERS{"Member", "Status", "Consumed", "Balance"};

    /// Their first name.
    std::string name;
    /// Whether they are active or archived.
    std::string_view status;
    /// What they consumed.
    std::string consumed;
    /// What is left: what they paid minus what they consumed.
    std::string balance;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] inline constexpr Cells<4> cells() const noexcept { return {name, status, consumed, balance}; }
};

/// An account with its label and its details.
[[nodiscard]] std::string describe(PaymentAccount const &account) noexcept {
    auto const *const iban = std::get_if<IbanPaymentAccountDetails>(&account.details);
    return std::format("{}: {}", account.name,
                       (nullptr == iban ? std::get<UsernamePaymentAccountDetails>(account.details).username.toString() : iban->iban.toString()));
}

/// The row of @p input, checked as @p kind: "valid: " and what @p shown writes of the value @p parse makes of it, or
/// "refused: " and the library's message.
template <typename Parse, typename Shown>
[[nodiscard]] CheckRow validated(std::string_view const input, std::string_view const kind, Parse const &parse, Shown const &shown) noexcept {
    auto const parsed = parse(input);
    return {
        .input = input,
        .kind = kind,
        .result = parsed.has_value() ? std::format("valid: {}", shown(*parsed)) : std::format("refused: {}", parsed.error().message()),
    };
}

/// Prints every member of @p group with their contact and account details.
void printMembers(ExpenseGroup const &group) noexcept {
    std::vector<MemberRow> rows;
    for (auto const &member : group.members()) {
        std::vector<std::string> accounts;
        accounts.reserve(member.accounts.size());
        for (auto const &account : member.accounts) {
            accounts.push_back(describe(account));
        }
        rows.push_back({
            .name = member.lastName.has_value() ? std::format("{} {}", member.firstName, *member.lastName) : member.firstName,
            .email = member.email.has_value() ? member.email->toString() : "",
            .phone = member.phone.has_value() ? member.phone->international() : "",
            .accounts = cell(accounts),
        });
    }
    printTable(rows, 4);
}

/// Prints whether each member of @p group is active or archived, what they consumed, and their balance.
void printBalances(ExpenseGroup const &group) noexcept {
    auto const summary = group.summary();
    std::vector<BalanceRow> rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id).value();
        rows.push_back({
            .name = member.firstName,
            .status = must(group.isArchived(member.id))
                          .transform([](bool const archived) noexcept { return archived ? "archived" : "active"; })
                          .value_or("not in the group"),
            .consumed = found.share.toString(),
            .balance = toString(found.balance),
        });
    }
    printTable(rows, 2);
}

/// Adds Ana, with every detail, and Bob, with a first name only, to @p group, and prints them.
[[nodiscard]] Flatmates members(ExpenseGroup &group) noexcept {
    section("1. Members");
    Member first("Ana");
    first.accounts = {
        PaymentAccount("ING", IbanPaymentAccountDetails{must(Iban::parse("ro49 aaaa 1b31 0075 9384 0000"))}),
        PaymentAccount("Revolut", UsernamePaymentAccountDetails{must(Username::parse("@ana"))}),
    };
    first.email = must(Email::parse("ana@example.com"));
    first.lastName = "Popescu";
    first.phone = must(PhoneNumber::parse("+40 712 345 678"));
    auto const ana = must(group.addMember(first));
    auto const bob = must(group.addMember(Member("Bob")));
    printMembers(group);
    printLine("The phone number was stored in E.164 form, +40712345678, and the table shows it\n"
              "grouped for reading; the IBAN was stored in its electronic form, uppercase and\n"
              "without spaces, the way banking apps accept it.");
    return {.ana = ana, .bob = bob};
}

/// Prints what validation makes of valid and mistyped IBANs, emails, and phone numbers.
void validation() noexcept {
    section("2. What validation accepts");
    printLine("IBANs are checked against the SWIFT registry: the country, its length, which\n"
              "positions hold digits and which letters, and the mod-97 check digits, which catch\n"
              "any single mistyped character. Emails are checked against the rules of RFC 5321\n"
              "and 5322, which refuse the doubled dots and stray commas of hurried typing, and\n"
              "phone numbers against their country's numbering plan, stored in E.164 form.");
    auto const iban = [](std::string_view const input) noexcept {
        return validated(
            input, "IBAN", [](std::string_view const text) noexcept { return Iban::parse(text); },
            [](Iban const &parsed) noexcept { return parsed.toString(); });
    };
    auto const email = [](std::string_view const input) noexcept {
        return validated(
            input, "email", [](std::string_view const text) noexcept { return Email::parse(text); },
            [](Email const &parsed) noexcept { return parsed.toString(); });
    };
    auto const phone = [](std::string_view const input) noexcept {
        return validated(
            input, "phone", [](std::string_view const text) noexcept { return PhoneNumber::parse(text); },
            [](PhoneNumber const &parsed) noexcept { return parsed.toString(); });
    };
    printTable<CheckRow>(
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
void movingOut(ExpenseGroup &group, Flatmates const &flatmates) noexcept {
    section("3. A flatmate moves out");
    WholeExpenseSplit const both{EqualSplit{{flatmates.ana, flatmates.bob}}};
    auto const rent =
        must(group.addExpense(Expense("Rent", LocalCost{euros("900.00")}, EqualSplit{{flatmates.ana}}, both, must(CalendarDate::of(2026, 9, 1)))));
    must(group.archiveMember(flatmates.bob));
    printLine("Bob shared September's rent, then moved out. He is archived, not removed: his\n"
              "history and his balance stay, and he can still pay Ana back.");
    printBalances(group);

    auto const refused = refusal(
        group.addExpense(Expense("Utilities", LocalCost{euros("60.00")}, EqualSplit{{flatmates.ana}}, both, must(CalendarDate::of(2026, 10, 1)))));
    check(refused.has_value(), "the archived member is refused");
    std::println("October's utilities cannot include him:\n  error: {}", message(refused));
    auto const removal = refusal(group.removeMember(flatmates.bob));
    check(removal.has_value(), "the referenced member stays");
    std::println("Nor can he be removed while the rent references him:\n  error: {}", message(removal));

    check(Failure::EXPENSE_GROUP_ARCHIVED_MEMBER == refused->failure() && flatmates.bob == refused->member(), "the refusal names Bob");
    check(Failure::EXPENSE_GROUP_MEMBER_IN_USE == removal->failure(), "Bob is in use");
    check(Balance(OwesBalance{euros("450.00")}) == totalsOf(group.summary(), flatmates.bob).value().balance, "Bob owes 450.00 EUR");
    check(must(group.expense(rent)).has_value(), "the group keeps the rent");
}

/// Runs the example.
void run() noexcept {
    auto group = must(ExpenseGroup::create("Flat 4B", must(CurrencyCode::of("EUR"))));
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

int main() noexcept {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
