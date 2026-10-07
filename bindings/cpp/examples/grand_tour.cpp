// Every feature of the library in one story: a group and its details, members with contact details and accounts,
// categories, an expense for every way of dividing one, another currency, edits and refusals, members who come and go,
// the summary, a new group currency, settling up, saving and loading, the values on their own, and the group shared
// between tasks. Every section checks its key numbers. The port of the Rust example `grand_tour`, which prints the same
// report.

#include "eranos.hpp"
#include "report.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <format>
#include <functional>
#include <optional>
#include <print>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <variant>
#include <vector>

namespace eranos::examples {
namespace {

/// The members of the trip.
struct People final {
    /// Who gave every detail.
    MemberId ana;
    /// Who edits his details.
    MemberId bob;
    /// Eva's mother.
    MemberId carol;
    /// Who flies home early.
    MemberId dan;
    /// Carol's daughter.
    MemberId eva;
    /// Who was added by mistake.
    MemberId filip;
};

/// The categories the group adds to the built-in one.
struct Categories final {
    /// Food.
    CategoryId food;
    /// Lodging.
    CategoryId lodging;
    /// Transport, renamed Getting around.
    CategoryId transport;
};

/// The seven expenses, one for every way of dividing one.
struct Expenses final {
    /// The apartment, by percentage.
    ExpenseId apartment;
    /// The fado night, item by item with a tip.
    ExpenseId fado;
    /// The groceries, equally.
    ExpenseId groceries;
    /// The museum, with two payers.
    ExpenseId museum;
    /// The seafood dinner, by parts.
    ExpenseId seafood;
    /// The tuk-tuk tour, with a fixed tip shared equally.
    ExpenseId tour;
    /// The train to Porto, by exact amounts.
    ExpenseId train;
};

/// A row of the table of the group's details: a detail and its value.
struct DetailRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<2> HEADERS{"Detail", "Value"};

    /// What the detail is.
    std::string_view detail;
    /// Its value.
    std::string value;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<2> cells() const noexcept { return {detail, value}; }
};

/// A row of the table of members: a member's name, contact details, and accounts.
struct MemberRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<4> HEADERS{"Member", "Email", "Phone", "Accounts"};

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
    [[nodiscard]] Cells<4> cells() const noexcept { return {name, email, phone, accounts}; }
};

/// A row of the table of checks: a text, what it was checked as, and what validation made of it.
struct CheckRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<3> HEADERS{"Input", "Checked as", "Result"};

    /// The text checked, quoted.
    std::string input;
    /// What it was checked as.
    std::string_view kind;
    /// The value it makes, or its refusal.
    std::string result;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<3> cells() const noexcept { return {input, kind, result}; }
};

/// A row of the table of refusals: a change and the message the library refused it with.
struct RefusalRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<2> HEADERS{"Change", "Refused with"};

    /// The change.
    std::string_view change;
    /// The library's message.
    std::string error;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<2> cells() const noexcept { return {change, error}; }
};

/// A row of the table of shares: an expense, how it divides, and every involved member's payment and consumption.
struct ShareRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<5> HEADERS{"Expense", "Split", "Member", "Paid", "Consumed"};

    /// The expense's name.
    std::string expense;
    /// How it divides, in a few words.
    std::string how;
    /// The involved members, one per line.
    std::string members;
    /// What each member paid, one per line, or nothing for one who paid nothing.
    std::string paid;
    /// What each member consumed, one per line.
    std::string consumed;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<5> cells() const noexcept { return {expense, how, members, paid, consumed}; }
};

/// A row of a receipt: a line, who shared it, its quantity, its unit price, and its total.
struct ItemRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<5> HEADERS{"Item", "Shared by", "Quantity", "Unit price", "Line total"};

    /// What was ordered.
    std::string name;
    /// Who shared it.
    std::string sharedBy;
    /// How many were ordered.
    std::string quantity;
    /// The price of one.
    std::string unitPrice;
    /// The quantity times the unit price.
    std::string lineTotal;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<5> cells() const noexcept { return {name, sharedBy, quantity, unitPrice, lineTotal}; }
};

/// A row of the table of entries: an expense or a transfer, who paid it, and its amount as entered and in the group
/// currency.
struct EntryRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<5> HEADERS{"Entry", "Paid by", "Entered as", "Rate", "In the group currency"};

    /// What it was.
    std::string name;
    /// Who paid it.
    std::string payer;
    /// Its amount as entered.
    std::string original;
    /// The rate it was entered at.
    std::string rate;
    /// Its amount in the group currency.
    std::string converted;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<5> cells() const noexcept { return {name, payer, original, rate, converted}; }
};

/// A row of the table of edits: an entry and what happened to it.
struct EditRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<2> HEADERS{"Entry", "Edit"};

    /// The entry.
    std::string entry;
    /// What happened to it.
    std::string edit;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<2> cells() const noexcept { return {entry, edit}; }
};

/// A row of the table of balances: what a member paid, consumed, sent, and received, and their balance.
struct BalanceRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<6> HEADERS{"Member", "Paid", "Consumed", "Sent", "Received", "Balance"};

    /// Their first name.
    std::string name;
    /// What they paid.
    std::string paid;
    /// What they consumed.
    std::string consumed;
    /// What they sent.
    std::string sent;
    /// What they received.
    std::string received;
    /// What is left.
    std::string balance;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<6> cells() const noexcept { return {name, paid, consumed, sent, received, balance}; }
};

/// A row of the table of statuses: whether a member is active or archived, and their balance.
struct StatusRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<3> HEADERS{"Member", "Status", "Balance"};

    /// Their first name.
    std::string name;
    /// Whether they are active or archived.
    std::string_view status;
    /// Their balance.
    std::string balance;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<3> cells() const noexcept { return {name, status, balance}; }
};

/// A row of the table of categories: a category, how many expenses it holds, and their total.
struct CategoryRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<3> HEADERS{"Category", "Expenses", "Total"};

    /// Its name.
    std::string name;
    /// How many expenses it holds.
    std::string expenses;
    /// What its expenses came to.
    std::string total;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<3> cells() const noexcept { return {name, expenses, total}; }
};

/// A row of the table of each member's consumption per category.
struct MemberCategoryRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<5> HEADERS{"Member", "Food", "Lodging", "Getting around", "Uncategorised"};

    /// Their first name.
    std::string name;
    /// What they consumed of Food.
    std::string food;
    /// What they consumed of Lodging.
    std::string lodging;
    /// What they consumed of Getting around.
    std::string transport;
    /// What they consumed of Uncategorised.
    std::string uncategorised;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<5> cells() const noexcept { return {name, food, lodging, transport, uncategorised}; }
};

/// A row of the table of strategies: a strategy, the strategy whose plan it chose, and how many payments that plan makes.
struct StrategyRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<3> HEADERS{"Strategy", "Computed by", "Payments"};

    /// Its name.
    std::string_view strategy;
    /// The name of the strategy whose plan it chose.
    std::string_view computedBy;
    /// How many payments the plan makes.
    std::string payments;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<3> cells() const noexcept { return {strategy, computedBy, payments}; }
};

/// A row of the table of payments: who pays whom, and how much.
struct PaymentRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<3> HEADERS{"From", "To", "Amount"};

    /// Who pays.
    std::string from;
    /// Who is paid.
    std::string to;
    /// How much is paid.
    std::string amount;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<3> cells() const noexcept { return {from, to, amount}; }
};

/// A row of the table of loads: a change to a saved group, and the message its load is refused with.
struct LoadRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<2> HEADERS{"Change to the file", "Message"};

    /// The change to the file.
    std::string_view change;
    /// The library's message.
    std::string message;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<2> cells() const noexcept { return {change, message}; }
};

/// A row of the table of currencies: a currency's code, name, numeric code, and decimal places.
struct CurrencyRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<4> HEADERS{"Code", "Name", "Numeric code", "Decimal places"};

    /// Its ISO 4217 code.
    std::string code;
    /// Its name.
    std::string name;
    /// Its ISO 4217 number.
    std::string numericCode;
    /// How many decimal places its minor unit has.
    std::string decimalPlaces;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<4> cells() const noexcept { return {code, name, numericCode, decimalPlaces}; }
};

/// A row of the table of values: an input and what the library makes of it.
struct ValueRow final {
    /// The headers of the table's columns, in the order it prints them.
    static constexpr Cells<2> HEADERS{"Input", "Result"};

    /// The input.
    std::string_view input;
    /// What the library makes of it, or its refusal.
    std::string result;

    /// The row's cells, in the order of the table's headers.
    ///
    /// @return The cells
    [[nodiscard]] Cells<2> cells() const noexcept { return {input, result}; }
};

/// The amount @p text of euros.
[[nodiscard]] Money eur(std::string_view const text) { return must(Money::parse(text, must(CurrencyCode::of("EUR")))); }

/// The amount @p text of lei.
[[nodiscard]] Money ron(std::string_view const text) { return must(Money::parse(text, must(CurrencyCode::of("RON")))); }

/// A date of the trip, in October 2026.
[[nodiscard]] CalendarDate october(std::int32_t const day) { return must(CalendarDate::of(2026, 10, day)); }

/// "valid: " and what @p shown writes of the value @p make returns, or "refused: " and the library's message.
template <typename Make, typename Shown> [[nodiscard]] std::string outcome(Make const &make, Shown const &shown) {
    auto const made = make();
    return made.has_value() ? std::format("valid: {}", shown(*made)) : std::format("refused: {}", made.error().message());
}

/// The amount of @p cost, as entered.
[[nodiscard]] Money amountOf(Cost const &cost) {
    return std::visit([](auto const &alternative) { return alternative.amount; }, cost);
}

/// The rate @p cost was entered at: 1 for a cost in the group currency.
[[nodiscard]] ExchangeRate rateOf(Cost const &cost) {
    auto const *const converted = std::get_if<ConvertedCost>(&cost);
    return nullptr == converted ? must(ExchangeRate::parse("1")) : converted->rate;
}

/// The sum of @p amounts, all in @p currency.
[[nodiscard]] Money sum(CurrencyCode const &currency, std::vector<Money> const &amounts) {
    Money total(0, currency);
    for (auto const &amount : amounts) {
        total = must(total.plus(amount));
    }
    return total;
}

/// The identifier of the built-in category, which the group lists first.
[[nodiscard]] CategoryId builtInCategory(ExpenseGroup const &group) { return group.categories().at(0).id.value(); }

/// Prints what each member of @p group paid, consumed, sent, and received, and their balance.
void printBalances(ExpenseGroup const &group) {
    auto const summary = group.summary();
    auto const zero = Money(0, group.currency()).toString();
    std::vector<BalanceRow> rows;
    for (auto const &member : group.members()) {
        // The summary lists the members with a total; every total of the others is zero.
        auto const found = totalsOf(summary, member.id.value());
        if (found.has_value()) {
            rows.push_back({
                .name = member.firstName,
                .paid = found->paid.toString(),
                .consumed = found->share.toString(),
                .sent = found->sent.toString(),
                .received = found->received.toString(),
                .balance = toString(found->balance),
            });
        } else {
            rows.push_back({
                .name = member.firstName,
                .paid = zero,
                .consumed = zero,
                .sent = zero,
                .received = zero,
                .balance = toString(SettledBalance{}),
            });
        }
    }
    printTable(rows, 1);
}

/// Prints what @p group spent per category, and on how many expenses.
void printCategories(ExpenseGroup const &group) {
    auto const expenses = group.expenses();
    auto const builtIn = builtInCategory(group);
    std::vector<CategoryRow> rows;
    for (auto const &total : group.summary().categoryTotals) {
        auto const category = must(group.category(total.category));
        auto const count = std::count_if(expenses.begin(), expenses.end(),
                                         [&](Expense const &expense) { return total.category == expense.category.value_or(builtIn); });
        rows.push_back({
            .name = category.has_value() ? category->name : "?",
            .expenses = std::to_string(count),
            .total = total.total.toString(),
        });
    }
    printTable(rows, 1);
}

/// Prints every expense and transfer of @p group in a currency @p shown accepts, as entered and in the group currency.
void printEntries(ExpenseGroup const &group, std::function<bool(CurrencyCode const &)> const &shown) {
    auto const zero = Money(0, group.currency()).toString();
    std::vector<EntryRow> rows;
    for (auto const &expense : group.expenses()) {
        if (shown(amountOf(expense.cost).currency)) {
            auto const shares = must(group.expenseShares(expense.id.value()));
            rows.push_back({
                .name = expense.name,
                .payer = names(group, must(participants(expense.payers))),
                .original = must(total(expense)).toString(),
                .rate = rateOf(expense.cost).toString(),
                .converted = shares.has_value() ? shares->total.toString() : zero,
            });
        }
    }
    for (auto const &transfer : group.transfers()) {
        if (shown(amountOf(transfer.cost).currency)) {
            auto const converted = must(group.transferAmount(transfer.id.value()));
            rows.push_back({
                .name = std::format("Transfer to {}", firstName(group, transfer.to)),
                .payer = firstName(group, transfer.from),
                .original = amountOf(transfer.cost).toString(),
                .rate = rateOf(transfer.cost).toString(),
                .converted = converted.has_value() ? converted->toString() : zero,
            });
        }
    }
    printTable(rows, 2);
}

// 1. The group.

/// Creates the group, renames it, and prints its details.
[[nodiscard]] ExpenseGroup theGroup() {
    section("1. The group");
    auto group = must(ExpenseGroup::create("Lisbon", must(CurrencyCode::of("EUR")),
                                           ExpenseGroupOptions{.description = "A week by the Atlantic", .language = must(Language::parse("pt-pt"))}));
    auto const blank = refusal(group.setName("   "));
    must(group.setName("Lisbon and Porto"));
    must(group.setDescription("A week by the Atlantic, with a day in Porto"));
    auto const language = group.language().value().toString();
    printTable<DetailRow>(
        {
            {.detail = "Name", .value = group.name()},
            {.detail = "Currency", .value = group.currency().code()},
            {.detail = "Language", .value = language},
            {.detail = "Description", .value = group.description().value_or("")},
        },
        2);
    std::println("The group started as Lisbon and was renamed when a day in Porto joined the plan.\n"
                 "Every total is kept in its currency, {}. The language is a BCP 47 tag the apps\n"
                 "show the group in, stored in canonical form: pt-pt became {}. A blank name is\n"
                 "refused:\n"
                 "  error: {}",
                 group.currency().code(), language, message(blank));

    check(blank.has_value() && Failure::EXPENSE_GROUP_EMPTY_NAME == blank->failure(), "a blank name is refused");
    check("Lisbon and Porto" == group.name(), "the group is renamed");
    check("pt-PT" == language, "the language is canonical");
    return group;
}

// 2. Members.

/// An account with its label, its kind, and its details.
[[nodiscard]] std::string describe(PaymentAccount const &account) {
    auto const *const iban = std::get_if<IbanPaymentAccountDetails>(&account.details);
    return nullptr == iban
               ? std::format("{}, username {}", account.name, std::get<UsernamePaymentAccountDetails>(account.details).username.toString())
               : std::format("{}, IBAN {}", account.name, iban->iban.toString());
}

/// Prints every member of @p group with their contact and account details.
void printMembers(ExpenseGroup const &group) {
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

/// Prints what validation makes of IBANs, an email, phone numbers, and blank required text.
void validation(ExpenseGroup &group) {
    printLine("IBANs are checked against the SWIFT registry and their mod-97 check digits,\n"
              "emails against RFC 5321 and 5322, and phone numbers against their country's\n"
              "numbering plan. Required text must not be blank:");
    auto const iban = [](std::string_view const input) {
        return CheckRow{
            .input = std::format("\"{}\"", input),
            .kind = "IBAN",
            .result = outcome([&] { return Iban::parse(input); },
                              [](Iban const &parsed) { return std::format("{}, country {}", parsed.toString(), parsed.countryCode()); }),
        };
    };
    auto const phone = [](std::string_view const input) {
        return CheckRow{
            .input = std::format("\"{}\"", input),
            .kind = "phone",
            .result = outcome([&] { return PhoneNumber::parse(input); },
                              [](PhoneNumber const &parsed) { return std::format("{}, stored as {}", parsed.international(), parsed.toString()); }),
        };
    };
    auto const text = [](auto const &parsed) { return parsed.toString(); };
    printTable<CheckRow>(
        {
            iban("ro49 aaaa 1b31 0075 9384 0000"),
            iban("RO49 AAAA 1B31 0075 9384 0001"),
            {
                .input = "\"ana..popescu@example.com\"",
                .kind = "email",
                .result = outcome([] { return Email::parse("ana..popescu@example.com"); }, text),
            },
            phone("+351 912 345 678"),
            phone("+40 612 345 678"),
            {.input = "\"   \"", .kind = "username", .result = outcome([] { return Username::parse("   "); }, text)},
            {
                .input = "\"  \"",
                .kind = "first name",
                .result = outcome([&] { return group.addMember(Member("  ")); }, [](MemberId const &id) { return id.value(); }),
            },
        },
        3);
}

/// Adds the members of the trip to @p group, edits Bob, and prints them and what validation accepts.
[[nodiscard]] People members(ExpenseGroup &group) {
    section("2. Members");
    Member first("Ana");
    first.accounts = {
        PaymentAccount("ING", IbanPaymentAccountDetails{must(Iban::parse("RO49 AAAA 1B31 0075 9384 0000"))}),
        PaymentAccount("Revolut", UsernamePaymentAccountDetails{must(Username::parse("@ana"))}),
    };
    first.email = must(Email::parse("ana@example.com"));
    first.lastName = "Popescu";
    first.phone = must(PhoneNumber::parse("+40 712 345 678"));
    auto const ana = must(group.addMember(first));
    auto const bob = must(group.addMember(Member("Bob")));
    auto const carol = must(group.addMember(Member("Carol")));
    auto const dan = must(group.addMember(Member("Dan")));
    auto const eva = must(group.addMember(Member("Eva")));
    auto const filip = must(group.addMember(Member("Filip")));
    // An edit starts from a copy of the member, which keeps its identifier.
    auto stored = must(group.member(bob)).value();
    stored.email = must(Email::parse("bob@example.com"));
    stored.lastName = "Ionescu";
    auto const before = must(group.replaceMember(stored));
    printMembers(group);
    std::println("Only a first name is required. Ana gave every detail; Bob added his last name and\n"
                 "email later, through an edit that kept his identifier:\n"
                 "  {}\n"
                 "Eva is Carol's daughter, and Filip was added by mistake.",
                 bob.value());
    validation(group);

    check(!before.lastName.has_value(), "Bob had no last name");
    check(std::optional<std::string>("Ionescu") == must(group.member(bob)).value().lastName, "Bob has his last name");
    check(2 == must(group.member(ana)).value().accounts.size(), "Ana has two accounts");
    return {.ana = ana, .bob = bob, .carol = carol, .dan = dan, .eva = eva, .filip = filip};
}

// 3. Categories.

/// Adds Food, Lodging, and Transport to @p group, renames Transport, and shows what renaming refuses.
[[nodiscard]] Categories addCategories(ExpenseGroup &group) {
    section("3. Categories");
    auto const food = must(group.addCategory(Category("Food")));
    auto const lodging = must(group.addCategory(Category("Lodging")));
    auto const transport = must(group.addCategory(Category("Transport")));
    auto const homonym = refusal(group.addCategory(Category("FOOD")));
    auto stored = must(group.category(transport)).value();
    stored.name = "Getting around";
    static_cast<void>(must(group.replaceCategory(stored)));
    auto uncategorised = group.categories().at(0);
    uncategorised.name = "Other";
    auto const builtIn = refusal(group.replaceCategory(uncategorised));
    std::vector<std::string> names;
    for (auto const &category : group.categories()) {
        names.push_back(category.name);
    }
    std::string joined;
    for (std::size_t index = 0; names.size() > index; ++index) {
        joined += (0 == index ? "" : ", ") + names.at(index);
    }
    std::println("Every group has the built-in Uncategorised; this one adds Food, Lodging, and\n"
                 "Transport, then renames Transport, which keeps its identifier. In identifier order:\n"
                 "  {}\n"
                 "Names are compared caselessly, and the built-in category keeps its name:",
                 joined);
    printTable<RefusalRow>(
        {
            {.change = "Add a category named FOOD", .error = message(homonym)},
            {.change = "Rename Uncategorised to Other", .error = message(builtIn)},
        },
        2);

    check(homonym.has_value() && Failure::EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME == homonym->failure() && food == homonym->existing(),
          "a caseless homonym is refused");
    check(builtIn.has_value() && Failure::EXPENSE_GROUP_BUILT_IN_CATEGORY == builtIn->failure(), "the built-in category keeps its name");
    check("Getting around" == must(group.category(transport)).value().name, "Transport is renamed");
    return {.food = food, .lodging = lodging, .transport = transport};
}

// 4. Expenses, one for every way of dividing.

/// How a whole expense's amount divides, in a word or two.
[[nodiscard]] std::string divided(Split const &split) {
    return std::visit(
        [](auto const &alternative) -> std::string {
            using Alternative = std::decay_t<decltype(alternative)>;
            if constexpr (std::is_same_v<Alternative, AmountsSplit>) {
                return "by amounts";
            } else if constexpr (std::is_same_v<Alternative, EqualSplit>) {
                return "equally";
            } else if constexpr (std::is_same_v<Alternative, PartsSplit>) {
                return "by parts";
            } else {
                return "by percentage";
            }
        },
        split);
}

/// How @p expense divides, in a few words.
[[nodiscard]] std::string how(Expense const &expense) {
    auto const *const itemised = std::get_if<ItemisedExpenseSplit>(&expense.split);
    auto const split =
        nullptr == itemised ? divided(std::get<WholeExpenseSplit>(expense.split).split) : std::format("{} items", itemised->items.size());
    auto const count = must(participants(expense.payers)).size();
    auto const payers = 1 == count ? std::string() : std::format(", {} payers", count);
    std::string tip;
    if (0 != must(tipAmount(expense)).minorUnits && expense.tip.has_value()) {
        auto const *const fixed = std::get_if<FixedTipValue>(&expense.tip->value);
        tip = std::format(
            ", {} tip", (nullptr == fixed ? must(std::get<PercentageTipValue>(expense.tip->value).percentage.toString()) : fixed->amount.toString()));
    }
    return split + payers + tip;
}

/// Prints each expense of @p ids, with every involved member's payment and consumption.
void printShares(ExpenseGroup const &group, std::vector<ExpenseId> const &ids) {
    std::vector<ShareRow> rows;
    rows.reserve(ids.size());
    for (auto const &id : ids) {
        auto const expense = must(group.expense(id)).value();
        auto const shares = must(group.expenseShares(id)).value();
        std::vector<std::string> members;
        std::vector<std::string> paid;
        std::vector<std::string> consumed;
        for (auto const &share : shares.members) {
            members.push_back(firstName(group, share.member));
            paid.push_back(0 == share.paid.minorUnits ? "" : share.paid.toString());
            consumed.push_back(share.share.toString());
        }
        rows.push_back({.expense = expense.name, .how = how(expense), .members = cell(members), .paid = cell(paid), .consumed = cell(consumed)});
    }
    printTable(rows, 3);
}

/// Prints the receipt of the expense @p id of @p group: who shared each line, its quantity, its unit price, and its total.
void printItems(ExpenseGroup const &group, ExpenseId const &id) {
    auto const expense = must(group.expense(id)).value();
    auto const *const itemised = std::get_if<ItemisedExpenseSplit>(&expense.split);
    if (nullptr == itemised) {
        return;
    }
    std::vector<ItemRow> rows;
    rows.reserve(itemised->items.size());
    for (auto const &item : itemised->items) {
        Money const lineTotal(item.unitPrice.minorUnits * item.quantity, item.unitPrice.currency);
        rows.push_back({
            .name = item.name,
            .sharedBy = names(group, must(participants(item.split))),
            .quantity = std::to_string(item.quantity),
            .unitPrice = item.unitPrice.toString(),
            .lineTotal = lineTotal.toString(),
        });
    }
    printTable(rows, 2);
}

/// The line @p name of @p quantity at @p unitPrice euros, which @p split shares.
[[nodiscard]] ExpenseItem line(std::string name, std::string_view const unitPrice, Split split, std::int32_t const quantity) {
    ExpenseItem item(std::move(name), eur(unitPrice), std::move(split));
    item.quantity = quantity;
    return item;
}

/// The receipt of the fado night: every line with its own split.
[[nodiscard]] std::vector<ExpenseItem> fadoItems(People const &people) {
    return {
        line("Vinho verde", "12.00", EqualSplit{{people.ana, people.bob, people.dan}}, 2),
        line("Bacalhau", "18.00", EqualSplit{{people.carol}}, 1),
        line("Custard tarts", "1.50",
             PartsSplit{
                 {
                     MemberParts(people.ana, must(Parts::parse("1"))),
                     MemberParts(people.bob, must(Parts::parse("1"))),
                     MemberParts(people.carol, must(Parts::parse("1"))),
                     MemberParts(people.dan, must(Parts::parse("1"))),
                     MemberParts(people.eva, must(Parts::parse("2"))),
                 },
             },
             6),
        line("Juice", "3.00", EqualSplit{{people.eva}}, 1),
    };
}

/// @p expense in @p category.
[[nodiscard]] Expense inCategory(Expense expense, CategoryId const &category) {
    expense.category = category;
    return expense;
}

/// Adds one expense for every way of dividing one to @p group, in order.
[[nodiscard]] Expenses addEveryWay(ExpenseGroup &group, People const &people, Categories const &categories) {
    std::vector<MemberPercentage> const rooms{
        MemberPercentage(people.ana, must(Percentage::parse("25"))),
        MemberPercentage(people.bob, must(Percentage::parse("25"))),
        MemberPercentage(people.carol, must(Percentage::parse("35"))),
        MemberPercentage(people.dan, must(Percentage::parse("15"))),
    };
    std::vector<MemberParts> const portions{
        MemberParts(people.ana, must(Parts::parse("1"))),   MemberParts(people.bob, must(Parts::parse("1"))),
        MemberParts(people.carol, must(Parts::parse("1"))), MemberParts(people.dan, must(Parts::parse("1"))),
        MemberParts(people.eva, must(Parts::parse("0.5"))),
    };
    std::vector<MemberAmount> const tickets{
        MemberAmount(people.ana, eur("24.50")),
        MemberAmount(people.bob, eur("24.50")),
        MemberAmount(people.carol, eur("24.50")),
        MemberAmount(people.eva, eur("15.00")),
    };
    Expense fadoNight("Fado night", LocalCost{eur("54.00")},
                      AmountsSplit{{MemberAmount(people.carol, eur("39.40")), MemberAmount(people.dan, eur("20.00"))}},
                      ItemisedExpenseSplit{fadoItems(people)}, october(4));
    fadoNight.category = categories.food;
    fadoNight.description = "Tasca do Chico";
    fadoNight.time = must(TimeOfDay::of(22, 15));
    fadoNight.tip = TipRule(PercentageTipValue{must(Percentage::parse("10"))}, TipDistribution::PROPORTIONAL);
    Expense ride("Tuk-tuk tour", LocalCost{eur("45.00")}, EqualSplit{{people.dan}},
                 WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol, people.dan, people.eva}}}, october(5));
    ride.category = categories.transport;
    ride.tip = TipRule(FixedTipValue{eur("5.00")}, TipDistribution::EQUAL);
    // The group gives each expense its identifier as it adds it: an initialiser of Expenses would add them in the order of
    // its fields, not this one.
    auto const groceries =
        must(group.addExpense(inCategory(Expense("Groceries", LocalCost{eur("10.00")}, EqualSplit{{people.ana}},
                                                 WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol}}}, october(1)),
                                         categories.food)));
    auto const apartment = must(group.addExpense(
        inCategory(Expense("Apartment", LocalCost{eur("800.00")}, EqualSplit{{people.ana}}, WholeExpenseSplit{PercentagesSplit{rooms}}, october(1)),
                   categories.lodging)));
    auto const seafood = must(group.addExpense(inCategory(
        Expense("Seafood dinner", LocalCost{eur("90.00")}, EqualSplit{{people.carol}}, WholeExpenseSplit{PartsSplit{portions}}, october(2)),
        categories.food)));
    auto const train = must(group.addExpense(
        inCategory(Expense("Train to Porto", LocalCost{eur("88.50")}, EqualSplit{{people.bob}}, WholeExpenseSplit{AmountsSplit{tickets}}, october(3)),
                   categories.transport)));
    auto const museum =
        must(group.addExpense(Expense("Museum", LocalCost{eur("60.00")}, EqualSplit{{people.ana, people.bob}},
                                      WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol, people.dan}}}, october(3))));
    auto const fado = must(group.addExpense(fadoNight));
    auto const tour = must(group.addExpense(ride));
    return {
        .apartment = apartment,
        .fado = fado,
        .groceries = groceries,
        .museum = museum,
        .seafood = seafood,
        .tour = tour,
        .train = train,
    };
}

/// The share of @p member in the expense @p id of @p group.
[[nodiscard]] MemberShare shareOf(ExpenseGroup const &group, ExpenseId const &id, MemberId const &member) {
    auto const shares = must(group.expenseShares(id)).value();
    auto const found = std::ranges::find_if(shares.members, [&](MemberShare const &share) { return member == share.member; });
    check(shares.members.end() != found, "the member shares the expense");
    return *found;
}

/// Prints how the fado night's tip was shared, and its receipt.
void printFado(ExpenseGroup const &group, Expense const &fado) {
    auto const &tip = fado.tip.value();
    auto const *const fixed = std::get_if<FixedTipValue>(&tip.value);
    auto const rule = nullptr == fixed ? must(std::get<PercentageTipValue>(tip.value).percentage.toString()) : fixed->amount.toString();
    auto const *const shared = TipDistribution::EQUAL == tip.distribution ? "equally" : "in proportion to what each member ordered";
    std::println("The groceries' 10.00 EUR did not divide into thirds, so one member consumed the\n"
                 "leftover cent; the expense's identifier picks which, so it moves from expense to\n"
                 "expense. The fado night, at {} on {} at {}, was itemised; its\n"
                 "{} tip of {} was shared {}:",
                 fado.time.value().toString(), fado.date.toString(), fado.description.value_or(""), rule, must(tipAmount(fado)).toString(), shared);
    printItems(group, fado.id.value());
    printLine("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.");
}

/// Adds the seven expenses to @p group and prints how each divides; returns the seafood dinner's identifier.
[[nodiscard]] ExpenseId expenses(ExpenseGroup &group, People const &people, Categories const &categories) {
    section("4. Expenses, one for every way of dividing");
    auto const added = addEveryWay(group, people, categories);
    printLine("Seven expenses cover every way of dividing one: equally, by percentage, by parts\n"
              "(Eva counts as half), by exact amounts, with two payers, item by item, and with a\n"
              "tip. Every member's consumption adds up to the expense's total, to the cent:");
    printShares(group, {added.groceries, added.apartment, added.seafood, added.train, added.museum, added.fado, added.tour});
    auto const fadoExpense = must(group.expense(added.fado)).value();
    printFado(group, fadoExpense);

    std::array<std::int64_t, 3> thirds{
        shareOf(group, added.groceries, people.ana).share.minorUnits,
        shareOf(group, added.groceries, people.bob).share.minorUnits,
        shareOf(group, added.groceries, people.carol).share.minorUnits,
    };
    std::ranges::sort(thirds);
    check((std::array<std::int64_t, 3>{333, 333, 334}) == thirds, "the groceries divide into 3.33, 3.33, and 3.34");
    check(eur("280.00") == shareOf(group, added.apartment, people.carol).share, "Carol's 35 % of the apartment is 280.00 EUR");
    check(eur("10.00") == shareOf(group, added.seafood, people.eva).share, "Eva's half part of the dinner is 10.00 EUR");
    check(eur("15.00") == shareOf(group, added.train, people.eva).share, "Eva's ticket is 15.00 EUR");
    check(eur("30.00") == shareOf(group, added.museum, people.bob).paid, "Bob paid 30.00 EUR of the museum");
    check(eur("15.00") == shareOf(group, added.museum, people.dan).share, "Dan's quarter of the museum is 15.00 EUR");
    check(4 == must(participants(must(group.expense(added.museum)).value())).size(), "the museum names four members");
    check(eur("59.40") == must(total(fadoExpense)), "the fado night comes to 59.40 EUR");
    check(eur("5.40") == must(tipAmount(fadoExpense)), "its tip is 5.40 EUR");
    check(eur("21.45") == shareOf(group, added.fado, people.carol).share, "Carol's part of the fado night is 21.45 EUR");
    check(eur("6.60") == shareOf(group, added.fado, people.eva).share, "Eva's part of the fado night is 6.60 EUR");
    check(eur("10.00") == shareOf(group, added.tour, people.eva).share, "Eva's part of the tour is 10.00 EUR");
    return added.seafood;
}

// 5. Another currency.

/// Adds the airport parking in RON and Bob's repayment in cash, and shows how they convert.
void anotherCurrency(ExpenseGroup &group, People const &people, Categories const &categories) {
    section("5. Another currency");
    auto const euro = must(CurrencyCode::of("EUR"));
    auto const rate = must(ExchangeRate::parse("0.2"));
    auto const parking =
        must(group.addExpense(inCategory(Expense("Airport parking", ConvertedCost{.amount = ron("250.00"), .rate = rate}, EqualSplit{{people.ana}},
                                                 WholeExpenseSplit{EqualSplit{{people.ana, people.bob}}}, october(6)),
                                         categories.transport)));
    Transfer repayment(people.bob, people.ana, ConvertedCost{.amount = ron("100.00"), .rate = rate}, october(6));
    repayment.note = "Cash for the parking";
    repayment.time = must(TimeOfDay::of(7, 40));
    auto const cash = must(group.addTransfer(repayment));
    printLine("Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in\n"
              "cash. An entry in another currency carries the rate into the group currency, how\n"
              "many EUR one RON is worth, and is converted once, when it is entered:");
    printEntries(group, [&](CurrencyCode const &currency) { return euro.code() != currency.code(); });
    auto const transfer = must(group.transfer(cash)).value();
    auto const converted = must(must(ExchangeRate::parse("0.2011")).convert(ron("250.00"), euro));
    std::println("Bob's transfer, at {} (\"{}\"), counts as {}.\n"
                 "A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,\n"
                 "which becomes {}.",
                 transfer.time.value().toString(), transfer.note.value_or(""), must(group.transferAmount(cash)).value_or(Money(0, euro)).toString(),
                 converted.toString());

    check(eur("50.28") == converted, "the conversion rounds to 50.28 EUR");
    check(eur("50.00") == must(group.expenseShares(parking)).value().total, "the parking counts as 50.00 EUR");
    check(eur("20.00") == must(group.transferAmount(cash)).value(), "the cash counts as 20.00 EUR");
    check(std::holds_alternative<ConvertedCost>(transfer.cost), "the cash was converted");
}

// 6. Edits.

/// Replaces an expense and a transfer, and removes one of each; returns what changed.
[[nodiscard]] std::vector<EditRow> replaceAndRemove(ExpenseGroup &group, People const &people, Categories const &categories,
                                                    ExpenseId const &seafood) {
    // The dinner's receipt said 99.00.
    auto dinner = must(group.expense(seafood)).value();
    dinner.cost = LocalCost{eur("99.00")};
    dinner.description = "Cervejaria Ramiro";
    auto const previous = must(group.replaceExpense(dinner));
    // Bob entered the coffee twice.
    auto const coffee = inCategory(Expense("Coffee", LocalCost{eur("6.00")}, EqualSplit{{people.bob}},
                                           WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol}}}, october(5)),
                                   categories.food);
    static_cast<void>(must(group.addExpense(coffee)));
    auto const copy = must(group.addExpense(coffee));
    auto const removed = must(group.removeExpense(copy));
    // Dan's part of the apartment was 120.00, not 100.00.
    auto const first = must(group.addTransfer(Transfer(people.dan, people.ana, LocalCost{eur("100.00")}, october(5))));
    auto stored = must(group.transfer(first)).value();
    stored.cost = LocalCost{eur("120.00")};
    stored.note = "Apartment";
    auto const replaced = must(group.replaceTransfer(stored));
    // Carol sent 10.00 to the wrong member.
    auto const mistake = must(group.addTransfer(Transfer(people.carol, people.bob, LocalCost{eur("10.00")}, october(5))));
    auto const undone = must(group.removeTransfer(mistake));

    check(eur("90.00") == amountOf(previous.cost), "the dinner was 90.00 EUR");
    check(!must(group.expense(copy)).has_value(), "the copy is gone");
    check(eur("120.00") == must(group.transferAmount(first)).value(), "Dan's transfer is 120.00 EUR");
    check(!must(group.transfer(mistake)).has_value(), "the mistake is gone");
    auto const corrected = must(group.expense(seafood)).value();
    auto const apartment = must(group.transfer(first)).value();
    return {
        {
            .entry = previous.name,
            .edit = std::format("{} became {}, at {}", amountOf(previous.cost).toString(), amountOf(corrected.cost).toString(),
                                corrected.description.value_or("")),
        },
        {.entry = removed.name, .edit = std::format("entered twice; the copy of {} was removed", amountOf(removed.cost).toString())},
        {
            .entry = std::format("{} to {}", firstName(group, people.dan), firstName(group, people.ana)),
            .edit = std::format("{} became {}, noted {}", amountOf(replaced.cost).toString(), amountOf(apartment.cost).toString(),
                                apartment.note.value_or("")),
        },
        {
            .entry = std::format("{} to {}", firstName(group, people.carol), firstName(group, people.bob)),
            .edit = std::format("{} sent to the wrong member; removed", amountOf(undone.cost).toString()),
        },
    };
}

/// Replaces and removes entries, then shows that a change that does not fit is refused.
void edits(ExpenseGroup &group, People const &people, Categories const &categories, ExpenseId const &seafood) {
    section("6. Edits");
    auto const changes = replaceAndRemove(group, people, categories, seafood);
    printLine("Entries are values: an edit starts from a copy made with to_builder, keeps the\n"
              "identifier, and replaces the entry; a removal takes it out with its totals.");
    printTable(changes, 2);
    printLine("The balances after the edits:");
    printBalances(group);

    auto const before = group.summary();
    std::vector<MemberPercentage> const halves{
        MemberPercentage(people.ana, must(Percentage::parse("50"))),
        MemberPercentage(people.bob, must(Percentage::parse("49.99"))),
    };
    auto const unbalanced = refusal(group.addExpense(
        Expense("Taxi", LocalCost{eur("30.00")}, EqualSplit{{people.ana}}, WholeExpenseSplit{PercentagesSplit{halves}}, october(6))));
    auto const toHerself = refusal(group.addTransfer(Transfer(people.ana, people.ana, LocalCost{eur("5.00")}, october(6))));
    auto const unrated = refusal(group.addExpense(
        Expense("Souvenirs", LocalCost{ron("40.00")}, EqualSplit{{people.ana}}, WholeExpenseSplit{EqualSplit{{people.ana}}}, october(6))));
    printLine("A change that does not fit is refused, and the group stays as it was:");
    printTable<RefusalRow>(
        {
            {.change = "A taxi split 50% and 49.99%", .error = message(unbalanced)},
            {.change = "Ana paying herself back", .error = message(toHerself)},
            {.change = "Souvenirs in RON, without a rate", .error = message(unrated)},
        },
        2);
    printLine("The balances again, unchanged:");
    printBalances(group);

    check(unbalanced.has_value() && Failure::EXPENSE_PERCENTAGES_DO_NOT_ADD_UP == unbalanced->failure() &&
              std::optional<std::int64_t>(9'999) == unbalanced->totalBasisPoints(),
          "percentages that do not add up are refused");
    check(toHerself.has_value() && Failure::TRANSFER_SAME_MEMBER == toHerself->failure(), "a transfer to oneself is refused");
    check(unrated.has_value() && Failure::EXPENSE_GROUP_MISSING_EXCHANGE_RATE == unrated->failure() &&
              std::optional<std::string>("RON") == unrated->currency(),
          "an amount in RON without a rate is refused");
    check(before == group.summary(), "the refusals change nothing");
}

// 7. Members come and go.

/// Removes Filip, archives Dan, shows what that refuses, and unarchives him.
void comingsAndGoings(ExpenseGroup &group, People const &people) {
    section("7. Members come and go");
    auto const removed = must(group.removeMember(people.filip));
    must(group.archiveMember(people.dan));
    auto const farewell =
        refusal(group.addExpense(Expense("Farewell dinner", LocalCost{eur("40.00")}, EqualSplit{{people.ana}},
                                         WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol, people.dan}}}, october(6))));
    auto const inUse = refusal(group.removeMember(people.dan));
    std::println("{} was never part of an entry, so removing him leaves no trace. Dan flew home\n"
                 "early and was archived: his history and his balance stay, but a new expense\n"
                 "cannot include him, and he cannot be removed while entries reference him.",
                 removed.firstName);
    auto const summary = group.summary();
    std::vector<StatusRow> rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id.value());
        if (found.has_value()) {
            rows.push_back({
                .name = member.firstName,
                .status = must(group.isArchived(member.id.value())) ? "archived" : "active",
                .balance = toString(found->balance),
            });
        }
    }
    printTable(rows, 2);
    printTable<RefusalRow>(
        {
            {.change = "A farewell dinner with Dan", .error = message(farewell)},
            {.change = "Removing Dan", .error = message(inUse)},
        },
        2);
    must(group.unarchiveMember(people.dan));
    printLine("When Dan joins the next trip, unarchiving him makes him active again.");

    check("Filip" == removed.firstName, "Filip was removed");
    check(!must(group.member(people.filip)).has_value(), "Filip is gone");
    check(farewell.has_value() && Failure::EXPENSE_GROUP_ARCHIVED_MEMBER == farewell->failure() && people.dan == farewell->member(),
          "an archived member joins no expense");
    check(inUse.has_value() && Failure::EXPENSE_GROUP_MEMBER_IN_USE == inUse->failure(), "a referenced member stays");
    check(!must(group.isArchived(people.dan)), "Dan is active again");
}

// 8. The summary.

/// The totals per category of @p member in @p summary.
[[nodiscard]] std::vector<CategoryTotal> byCategory(Summary const &summary, MemberId const &member) {
    auto const found = std::ranges::find_if(summary.memberCategoryTotals, [&](MemberCategoryTotals const &entry) { return member == entry.member; });
    return summary.memberCategoryTotals.end() == found ? std::vector<CategoryTotal>() : found->totals;
}

/// The total of @p category among @p totals, absent when they list none.
[[nodiscard]] std::optional<Money> totalOf(std::vector<CategoryTotal> const &totals, CategoryId const &category) {
    auto const found = std::ranges::find_if(totals, [&](CategoryTotal const &entry) { return category == entry.category; });
    return totals.end() == found ? std::nullopt : std::optional<Money>(found->total);
}

/// Prints the summary: every member's totals, the spending per category, and each member's consumption per category.
void summary(ExpenseGroup const &group, People const &people, Categories const &categories) {
    section("8. The summary");
    auto const totals = group.summary();
    std::println("The group spent {} in all. For each member, what they paid, what they\n"
                 "consumed, the transfers they sent and received, and what is left:",
                 totals.totalSpent.toString());
    printBalances(group);
    printLine("The spending per category:");
    printCategories(group);
    printLine("And each member's consumption per category:");
    auto const builtIn = builtInCategory(group);
    Money const zero(0, group.currency());
    std::vector<MemberCategoryRow> rows;
    for (auto const &member : group.members()) {
        auto const own = byCategory(totals, member.id.value());
        rows.push_back({
            .name = member.firstName,
            .food = totalOf(own, categories.food).value_or(zero).toString(),
            .lodging = totalOf(own, categories.lodging).value_or(zero).toString(),
            .transport = totalOf(own, categories.transport).value_or(zero).toString(),
            .uncategorised = totalOf(own, builtIn).value_or(zero).toString(),
        });
    }
    printTable(rows, 1);

    check(eur("1222.90") == totals.totalSpent, "the group spent 1222.90 EUR");
    check(eur("890.00") == totalsOf(totals, people.ana).value().paid, "Ana paid 890.00 EUR");
    check(eur("140.00") == totalsOf(totals, people.ana).value().received, "Ana received 140.00 EUR");
    check(eur("20.00") == totalsOf(totals, people.bob).value().sent, "Bob sent 20.00 EUR");
    check(Balance(GetsBackBalance{eur("12.55")}) == totalsOf(totals, people.dan).value().balance, "Dan gets back 12.55 EUR");
    check(Balance(OwesBalance{eur("42.60")}) == totalsOf(totals, people.eva).value().balance, "Eva owes 42.60 EUR");
    check(std::optional<Money>(eur("174.40")) == totalOf(totals.categoryTotals, categories.food), "the food came to 174.40 EUR");
    check(std::optional<Money>(eur("800.00")) == totalOf(totals.categoryTotals, categories.lodging), "the lodging came to 800.00 EUR");
    check(std::optional<Money>(eur("188.50")) == totalOf(totals.categoryTotals, categories.transport), "getting around came to 188.50 EUR");
    check(std::optional<Money>(eur("60.00")) == totalOf(totals.categoryTotals, builtIn), "the rest came to 60.00 EUR");
    std::vector<Money> nets;
    for (auto const &found : totals.members) {
        std::vector<Money> amounts;
        for (auto const &entry : byCategory(totals, found.member)) {
            amounts.push_back(entry.total);
        }
        check(sum(zero.currency, amounts) == found.share, "every member's consumption per category adds up to their consumption");
        nets.push_back(found.net);
    }
    check(0 == sum(zero.currency, nets).minorUnits, "every net adds up to zero");
}

// 9. Removing a category.

/// Removes Getting around, whose expenses move to Uncategorised, and shows that the built-in category stays.
void removingACategory(ExpenseGroup &group, Categories const &categories) {
    section("9. Removing a category");
    auto const removed = must(group.removeCategory(categories.transport));
    auto const uncategorised = builtInCategory(group);
    auto const builtIn = refusal(group.removeCategory(uncategorised));
    std::println("Removing {} moves its expenses to Uncategorised and the category\n"
                 "totals with them; no balance changes. The built-in category cannot be removed:\n"
                 "  error: {}",
                 removed.name, message(builtIn));
    printCategories(group);

    check(builtIn.has_value() && Failure::EXPENSE_GROUP_BUILT_IN_CATEGORY == builtIn->failure(), "the built-in category cannot be removed");
    check(!must(group.category(categories.transport)).has_value(), "Getting around is gone");
    check(std::optional<Money>(eur("248.50")) == totalOf(group.summary().categoryTotals, uncategorised), "Uncategorised comes to 248.50 EUR");
    auto const expenses = group.expenses();
    check(std::ranges::none_of(expenses, [&](Expense const &expense) { return categories.transport == expense.category; }),
          "no expense is in Getting around");
}

// 10. A new currency.

/// Moves the books of @p group to RON, which a missing rate first refuses.
void newCurrency(ExpenseGroup &group, People const &people) {
    section("10. A new currency");
    auto const euro = must(CurrencyCode::of("EUR"));
    auto const leu = must(CurrencyCode::of("RON"));
    auto const missing = refusal(group.changeCurrency(leu, {}));
    must(group.changeCurrency(leu, std::array{CurrencyRate(euro, must(ExchangeRate::parse("5")))}));
    std::println("Home again, the group moves its books to RON. The change needs a rate into RON\n"
                 "for every other currency in use, here only EUR, or it is refused:\n"
                 "  error: {}\n"
                 "Entries in EUR take the new rate, and the RON parking and cash become local:",
                 message(missing));
    printEntries(group, [](CurrencyCode const &) { return true; });
    std::println("Each entry keeps its currency and converts at the new rate. At 5 every share\n"
                 "converts exactly, so each balance is five times what it was. The trip cost\n"
                 "{}:",
                 group.summary().totalSpent.toString());
    printBalances(group);

    auto const summary = group.summary();
    check(missing.has_value() && Failure::EXPENSE_GROUP_MISSING_EXCHANGE_RATE == missing->failure() &&
              std::optional<std::string>("EUR") == missing->currency(),
          "a change without the rate from EUR is refused");
    check(leu == group.currency(), "the group keeps its books in RON");
    check(ron("6114.50") == summary.totalSpent, "the trip cost 6114.50 RON");
    check(Balance(GetsBackBalance{ron("62.75")}) == totalsOf(summary, people.dan).value().balance, "Dan gets back 62.75 RON");
    check(Balance(OwesBalance{ron("213.00")}) == totalsOf(summary, people.eva).value().balance, "Eva owes 213.00 RON");
}

// 11. Settling up.

/// The name of @p strategy as the apps show it.
[[nodiscard]] std::string_view strategyName(SettlementStrategy const strategy) {
    switch (strategy) {
    case SettlementStrategy::ADAPTIVE:
        return "Adaptive";
    case SettlementStrategy::MINIMUM_TRANSFERS:
        return "MinimumTransfers";
    case SettlementStrategy::GREEDY:
        return "Greedy";
    case SettlementStrategy::DIRECT:
        return "Direct";
    }
    std::unreachable();
}

/// Prints how many payments each strategy plans, then records the minimum plan as transfers.
void settlingUp(ExpenseGroup &group) {
    section("11. Settling up");
    std::vector<StrategyRow> counts;
    for (auto const strategy :
         {SettlementStrategy::ADAPTIVE, SettlementStrategy::MINIMUM_TRANSFERS, SettlementStrategy::GREEDY, SettlementStrategy::DIRECT}) {
        auto const settlement = must(group.settlement(strategy));
        // The group's open balances are well within the adaptive strategy's limit, so it settles them exactly.
        auto const computedBy = SettlementStrategy::ADAPTIVE == strategy ? SettlementStrategy::MINIMUM_TRANSFERS : strategy;
        check(computedBy == settlement.strategy, "the settlement names the strategy whose plan it is");
        counts.push_back({
            .strategy = strategyName(strategy),
            .computedBy = strategyName(settlement.strategy),
            .payments = std::to_string(settlement.payments.size()),
        });
    }
    printLine("Each strategy turns the balances into payments: the direct one pays back every\n"
              "pair's own debts, greedy pays the largest amounts first, the minimum one finds\n"
              "the fewest payments, and the adaptive one finds the fewest while that is quick\n"
              "and greedy's beyond, naming the strategy whose plan it chose.");
    printTable(counts, 2);
    auto const plan = must(group.settlement(SettlementStrategy::MINIMUM_TRANSFERS));
    printLine("The group follows the minimum plan:");
    std::vector<PaymentRow> payments;
    payments.reserve(plan.payments.size());
    for (auto const &payment : plan.payments) {
        payments.push_back({.from = firstName(group, payment.from), .to = firstName(group, payment.to), .amount = payment.amount.toString()});
    }
    printTable(payments, 2);
    for (auto const &payment : plan.payments) {
        static_cast<void>(must(group.addTransfer(Transfer(payment.from, payment.to, LocalCost{payment.amount}, october(7)))));
    }
    printLine("Each payment, once made, is recorded as a transfer, and every balance is settled:");
    printBalances(group);

    check(must(group.settlement(SettlementStrategy::GREEDY)).payments.empty(), "nothing is left to pay");
    check(4 == plan.payments.size(), "the minimum plan takes four payments");
    auto const summary = group.summary();
    check(std::ranges::all_of(summary.members, [](MemberSummary const &found) { return std::holds_alternative<SettledBalance>(found.balance); }),
          "every balance is settled");
}

// 12. Saving and loading.

/// @p text with the first @p pattern in it replaced by @p replacement.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the text, then what to find in it and what replaces it, as a search reads.
[[nodiscard]] std::string replaceFirst(std::string text, std::string_view const pattern, std::string_view const replacement) {
    auto const found = text.find(pattern);
    check(std::string::npos != found, "the text holds what the example replaces");
    return text.replace(found, pattern.size(), replacement);
}

/// Saves @p group, loads it again, and shows that a changed file is refused with a typed error.
void savingAndLoading(ExpenseGroup const &group, People const &people, Categories const &categories) {
    section("12. Saving and loading");
    auto const saved = group.save();
    std::println("The group saves to {} bytes of JSON: its details and entries, never the totals.\n"
                 "Ana's accounts are saved in clear text, as the library holds them:",
                 saved.size());
    auto const ana = must(group.member(people.ana)).value();
    for (auto const &account : ana.accounts) {
        auto const *const iban = std::get_if<IbanPaymentAccountDetails>(&account.details);
        auto const details = nullptr == iban ? std::get<UsernamePaymentAccountDetails>(account.details).username.toString() : iban->iban.toString();
        check(saved.contains(details), "the details are saved as they are");
        std::println("  {}: {}", account.name, details);
    }
    auto const loaded = must(ExpenseGroup::load(saved));
    printLine("Loading replays every entry through the same checks as adding it and rebuilds the\n"
              "totals: the loaded group equals the saved one. Reading a SavedExpenseGroup and\n"
              "loading it with into_expense_group reports a refusal as a typed error, not as text:");
    auto const refused = [&](std::string_view const pattern, std::string_view const replacement) {
        auto const changed = replaceFirst(saved, pattern, replacement);
        check(changed != saved, "the change changes the file");
        auto const refusedLoad = refusal(ExpenseGroup::load(changed));
        check(refusedLoad.has_value() && ErrorKind::EXPENSE_GROUP == refusedLoad->kind(), "the changed file is refused");
        return refusedLoad.value();
    };
    auto const format = refused(R"("format": 2)", R"("format": 3)");
    auto const blank = refused(R"("name": "Lisbon and Porto")", R"("name": " ")");
    auto const homonym = refused(R"("name": "Lodging")", R"("name": "FOOD")");
    printTable<LoadRow>(
        {
            {.change = "\"format\": 3", .message = format.message()},
            {.change = "a blank group name", .message = blank.message()},
            {.change = "Lodging renamed FOOD", .message = homonym.message()},
        },
        2);

    check(saved == loaded.save(), "the loaded group saves as the group did");
    check(group.summary() == loaded.summary(), "the loaded group has the group's totals");
    check(Failure::EXPENSE_GROUP_UNSUPPORTED_FORMAT == format.failure() && std::optional<std::int64_t>(3) == format.format(),
          "a later format is refused");
    check(Failure::EXPENSE_GROUP_EMPTY_NAME == blank.failure(), "a blank name is refused");
    check(Failure::EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME == homonym.failure() && categories.food == homonym.existing(),
          "a caseless homonym is refused");
}

// 13. Values on their own.

/// Prints four currencies, and that an unknown code is refused.
void currencies() {
    std::vector<CurrencyRow> rows;
    for (auto const *const code : {"EUR", "RON", "JPY", "BHD"}) {
        auto const currency = must(CurrencyCode::of(code));
        rows.push_back({
            .code = currency.code(),
            .name = currency.name(),
            .numericCode = std::to_string(currency.numericCode()),
            .decimalPlaces = std::to_string(currency.decimalPlaces()),
        });
    }
    auto const unknown = refusal(CurrencyCode::of("XYZ"));
    std::println("Currencies are the {} active ISO 4217 codes with a minor unit; an unknown code is\n"
                 "refused ({}):",
                 CurrencyCode::all().size(), message(unknown));
    printTable(rows, 2);
    check(3 == must(CurrencyCode::of("BHD")).decimalPlaces(), "the dinar has three decimal places");
}

/// Prints what amounts do: sums, differences, refusals, minor units, and a cost's rate kept for another amount.
void amounts() {
    auto const sumOf = must(eur("12.34").plus(eur("0.66")));
    auto const change = must(eur("5.00").minus(eur("7.50")));
    auto const mixed = refusal(eur("1.00").plus(ron("1.00")));
    auto const yen = must(Money::parse("1500", must(CurrencyCode::of("JPY"))));
    Money const dinars(1'234, must(CurrencyCode::of("BHD")));
    Money const nothing(0, must(CurrencyCode::of("RON")));
    ConvertedCost const cost{.amount = eur("10.00"), .rate = must(ExchangeRate::parse("4.9765"))};
    ConvertedCost const doubled{.amount = eur("20.00"), .rate = cost.rate};
    printLine("Amounts are exact minor units in their currency:");
    printTable<ValueRow>(
        {
            {.input = "12.34 EUR + 0.66 EUR", .result = sumOf.toString()},
            {
                .input = "5.00 EUR − 7.50 EUR",
                .result = std::format("{}, negative: {}", change.toString(), (0 > change.minorUnits ? "true" : "false")),
            },
            {.input = "1.00 EUR + 1.00 RON", .result = message(mixed)},
            {.input = "1.005 EUR", .result = message(refusal(Money::parse("1.005", must(CurrencyCode::of("EUR")))))},
            {.input = "1500 JPY", .result = std::format("{}, {} minor units", yen.toString(), yen.minorUnits)},
            {.input = "1234 minor units of BHD", .result = dinars.toString()},
            {.input = "zero RON", .result = std::format("{}, zero: {}", nothing.toString(), (0 == nothing.minorUnits ? "true" : "false"))},
            {
                .input = "10.00 EUR at 4.9765",
                .result = std::format("{} at {}, then {} at the same rate", cost.amount.toString(), cost.rate.toString(), doubled.amount.toString()),
            },
        },
        2);
    check(eur("13.00") == sumOf, "12.34 EUR and 0.66 EUR make 13.00 EUR");
    check(-250 == change.minorUnits, "5.00 EUR less 7.50 EUR is -2.50 EUR");
    check(mixed.has_value(), "amounts in two currencies do not combine");
    check("1.234 BHD" == dinars.toString(), "a dinar has three decimal places");
}

/// Prints what percentages, parts, and rates read and print.
void ratios() {
    auto const percentage = must(Percentage::parse("12.5"));
    auto const parts = must(Parts::parse("1.5"));
    auto const exchange = must(ExchangeRate::parse("4.97650"));
    printLine("Percentages, parts, and rates are exact decimals:");
    printTable<ValueRow>(
        {
            {.input = "12.5%", .result = std::format("{}, {} basis points", must(percentage.toString()), percentage.basisPoints)},
            {.input = "333 basis points", .result = must(Percentage{333}.toString())},
            {.input = "33.333%", .result = message(refusal(Percentage::parse("33.333")))},
            {.input = "1.5 parts", .result = std::format("{}, {} hundredths", must(parts.toString()), parts.hundredths)},
            {.input = "3 whole parts", .result = must(Parts{300}.toString())},
            {.input = "25 hundredths of a part", .result = must(Parts{25}.toString())},
            {.input = "0 parts", .result = message(refusal(Parts::parse("0")))},
            {.input = "a rate of 4.97650", .result = exchange.toString()},
            {.input = "a rate of 0", .result = message(refusal(ExchangeRate::parse("0")))},
        },
        2);
    check(1'250 == percentage.basisPoints, "12.5 % is 1250 basis points");
    check(150 == parts.hundredths, "1.5 parts are 150 hundredths");
}

/// Prints what dates, times, identifiers, and language tags read and print.
void datesAndIdentifiers(ExpenseGroup const &group, People const &people) {
    auto const date = must(CalendarDate::parse("2026-10-01"));
    auto const time = must(TimeOfDay::parse("22:15"));
    auto const found = must(group.member(people.ana)).value().id;
    auto const language = must(Language::parse("pt-pt"));
    printLine("Dates, times, identifiers, and language tags read and print the ISO forms:");
    printTable<ValueRow>(
        {
            {.input = "2026-10-01", .result = std::format("{}: day {}, month {}, year {}", date.toString(), date.day(), date.month(), date.year())},
            {.input = "29 February 2027", .result = message(refusal(CalendarDate::of(2027, 2, 29)))},
            {.input = "22:15", .result = std::format("{}: hour {}, minute {}", time.toString(), time.hour(), time.minute())},
            {.input = "24:00", .result = message(refusal(TimeOfDay::of(24, 0)))},
            {
                .input = "Ana's identifier",
                .result = std::format("{}, the same after a round trip: {}", people.ana.value(),
                                      (std::optional<MemberId>(people.ana) == found ? "true" : "false")),
            },
            {.input = "not-a-uuid", .result = message(refusal(group.member(MemberId("not-a-uuid"))))},
            {.input = "pt-pt", .result = language.toString()},
            {.input = "pt_PT", .result = message(refusal(Language::parse("pt_PT")))},
        },
        2);
    check(1 == date.day() && 10 == date.month() && 2026 == date.year(), "the date reads as 1 October 2026");
    check(22 == time.hour() && 15 == time.minute(), "the time reads as 22:15");
    check(std::optional<MemberId>(people.ana) == found, "Ana's identifier comes back the same");
}

/// Prints the values on their own: currencies, amounts, ratios, dates, times, identifiers, and language tags.
void values(ExpenseGroup const &group, People const &people) {
    section("13. Values on their own");
    currencies();
    amounts();
    ratios();
    datesAndIdentifiers(group, people);
}

// 14. Sharing the group.

/// Shares @p group between tasks: an expense added through another handle, a settlement computed asynchronously, and a
/// change that waits for a report's read and is cancelled.
void sharing(ExpenseGroup &group, People const &people) {
    section("14. Sharing the group");
    // Another handle on the same group, as another task holds: a group is shared as it is.
    auto &other = group;
    std::vector<MemberId> const everyone{people.ana, people.bob, people.carol, people.dan, people.eva};
    static_cast<void>(must(other
                               .addExpenseAsync(Expense("Airport coffee", LocalCost{ron("75.00")}, EqualSplit{{people.bob}},
                                                        WholeExpenseSplit{EqualSplit{everyone}}, october(8)))
                               .get()));
    auto const plan = must(group.settlementAsync(SettlementStrategy::MINIMUM_TRANSFERS).get());
    printLine("An app shares the group between its tasks and threads through a\n"
              "SharedExpenseGroup, whose calls are futures any executor runs; here the library's\n"
              "block_on runs each one on this thread. Bob adds the airport coffee through another\n"
              "handle, and the plan that settles it comes through the first:");
    {
        auto const reader = must(group.readAsync().get());
        std::vector<PaymentRow> rows;
        rows.reserve(plan.payments.size());
        for (auto const &payment : plan.payments) {
            rows.push_back({
                .from = must(reader.member(payment.from)).value().firstName,
                .to = must(reader.member(payment.to)).value().firstName,
                .amount = payment.amount.toString(),
            });
        }
        printTable(rows, 2);
    }

    auto const before = must(group.summaryAsync().get());
    std::vector<CurrencyRate> const rates{CurrencyRate(must(CurrencyCode::of("RON")), must(ExchangeRate::parse("0.2")))};
    // A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
    // behind the change, so the report reads through its own handle.
    auto report = must(group.readAsync().get());
    Cancellation cancellation;
    auto change = group.changeCurrencyAsync(must(CurrencyCode::of("EUR")), rates, &cancellation);
    auto const reported = must(report.summary()).totalSpent;
    // Cancelling stops the waiting change, which leaves the group as it was.
    cancellation.cancel();
    report.release();
    auto const cancelled = refusal(change.get());
    auto const after = must(group.summaryAsync().get());
    auto const currency = must(group.currencyAsync().get());
    std::println("While a report reads the group, a change of its currency to EUR waits for the\n"
                 "report. The app drops the waiting change, which cancels it: the group still\n"
                 "keeps its books in {}, and the trip cost {}, as the report read.",
                 currency.code(), after.totalSpent.toString());
    std::println("With the other handle gone, the app takes the group back: {} expenses.", must(group.expensesAsync().get()).size());

    check(4 == plan.payments.size(), "four payments settle the coffee");
    check(std::ranges::all_of(plan.payments, [&](Payment const &payment) { return people.bob == payment.to && ron("15.00") == payment.amount; }),
          "everyone pays Bob 15.00 RON");
    check(cancelled.has_value() && ErrorKind::CANCELLED == cancelled->kind(), "the change is cancelled");
    check(before == after, "the cancelled change leaves the group as it was");
    check(reported == after.totalSpent, "the report read what the group holds");
    check("RON" == currency.code(), "the group keeps its books in RON");
}

/// Runs the example.
void run() {
    printLine("Lisbon and Porto: a grand tour of eranos\n"
              "════════════════════════════════════════");
    printLine("Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each\n"
              "section below uses another part of the library, from creating the group to saving\n"
              "it, and checks its key numbers.");
    auto group = theGroup();
    auto const people = members(group);
    auto const categories = addCategories(group);
    auto const seafood = expenses(group, people, categories);
    anotherCurrency(group, people, categories);
    edits(group, people, categories, seafood);
    comingsAndGoings(group, people);
    summary(group, people, categories);
    removingACategory(group, categories);
    newCurrency(group, people);
    settlingUp(group);
    savingAndLoading(group, people, categories);
    values(group, people);
    sharing(group, people);
}

} // namespace
} // namespace eranos::examples

int main() {
    eranos::examples::run();
    return EXIT_SUCCESS;
}
