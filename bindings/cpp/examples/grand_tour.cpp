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

/// The amount @p text of euros.
[[nodiscard]] Money eur(std::string_view const text) { return Money::parse(text, Currency::of("EUR")); }

/// The amount @p text of lei.
[[nodiscard]] Money ron(std::string_view const text) { return Money::parse(text, Currency::of("RON")); }

/// A date of the trip, in October 2026.
[[nodiscard]] CalendarDate october(std::int32_t const day) { return CalendarDate::of(day, 10, 2026); }

/// "valid: " and what @p shown writes of the value @p make returns, or "refused: " and the library's message.
template <typename Make, typename Shown> [[nodiscard]] std::string outcome(Make const &make, Shown const &shown) {
    std::string result;
    auto const refused = refusal([&] {
        result = std::format("valid: {}", shown(make()));
        return 0;
    });
    return refused.has_value() ? std::format("refused: {}", message(refused)) : result;
}

/// The amount of @p cost, as entered.
[[nodiscard]] Money amountOf(Cost const &cost) {
    return std::visit([](auto const &alternative) { return alternative.amount; }, cost);
}

/// The rate @p cost was entered at: 1 for a cost in the group currency.
[[nodiscard]] ExchangeRate rateOf(Cost const &cost) {
    auto const *const converted = std::get_if<ConvertedCost>(&cost);
    return nullptr == converted ? ExchangeRate::parse("1") : converted->rate;
}

/// The sum of @p amounts, all in @p currency.
[[nodiscard]] Money sum(Currency const &currency, std::vector<Money> const &amounts) {
    Money total(currency, 0);
    for (auto const &amount : amounts) {
        total = total.plus(amount);
    }
    return total;
}

/// The identifier of the built-in category, which the group lists first.
[[nodiscard]] CategoryId builtInCategory(Group const &group) { return group.categories().at(0).id.value(); }

/// Prints what each member of @p group paid, consumed, sent, and received, and their balance.
void printBalances(Group const &group) {
    auto const summary = group.summary();
    auto const zero = Money(group.currency(), 0).toString();
    Rows rows;
    for (auto const &member : group.members()) {
        // The summary lists the members with a total; every total of the others is zero.
        auto const found = totalsOf(summary, member.id.value());
        if (found.has_value()) {
            rows.push_back({
                member.firstName,
                found->paid.toString(),
                found->share.toString(),
                found->sent.toString(),
                found->received.toString(),
                toString(found->balance),
            });
        } else {
            rows.push_back({member.firstName, zero, zero, zero, zero, toString(SettledBalance{})});
        }
    }
    printTable({"Member", "Paid", "Consumed", "Sent", "Received", "Balance"}, rows, 1);
}

/// Prints what @p group spent per category, and on how many expenses.
void printCategories(Group const &group) {
    auto const expenses = group.expenses();
    auto const builtIn = builtInCategory(group);
    Rows rows;
    for (auto const &total : group.summary().categoryTotals) {
        auto const category = group.category(total.category);
        auto const count = std::count_if(expenses.begin(), expenses.end(),
                                         [&](Expense const &expense) { return total.category == expense.category.value_or(builtIn); });
        rows.push_back({category.has_value() ? category->name : "?", std::to_string(count), total.total.toString()});
    }
    printTable({"Category", "Expenses", "Total"}, rows, 1);
}

/// Prints every expense and transfer of @p group in a currency @p shown accepts, as entered and in the group currency.
void printEntries(Group const &group, std::function<bool(Currency const &)> const &shown) {
    auto const zero = Money(group.currency(), 0).toString();
    Rows rows;
    for (auto const &expense : group.expenses()) {
        if (shown(amountOf(expense.cost).currency)) {
            auto const shares = group.expenseShares(expense.id.value());
            rows.push_back({
                expense.name,
                names(group, participants(expense.payers)),
                total(expense).toString(),
                rateOf(expense.cost).toString(),
                shares.has_value() ? shares->total.toString() : zero,
            });
        }
    }
    for (auto const &transfer : group.transfers()) {
        if (shown(amountOf(transfer.cost).currency)) {
            auto const converted = group.transferAmount(transfer.id.value());
            rows.push_back({
                std::format("Transfer to {}", firstName(group, transfer.to)),
                firstName(group, transfer.from),
                amountOf(transfer.cost).toString(),
                rateOf(transfer.cost).toString(),
                converted.has_value() ? converted->toString() : zero,
            });
        }
    }
    printTable({"Entry", "Paid by", "Entered as", "Rate", "In the group currency"}, rows, 2);
}

// 1. The group.

/// Creates the group, renames it, and prints its details.
[[nodiscard]] Group theGroup() {
    section("1. The group");
    auto group =
        Group::create(Currency::of("EUR"), "Lisbon", GroupOptions{.description = "A week by the Atlantic", .language = Language::parse("pt-pt")});
    auto const blank = refusal([&] {
        group.setName("   ");
        return 0;
    });
    group.setName("Lisbon and Porto");
    group.setDescription("A week by the Atlantic, with a day in Porto");
    auto const language = group.language().value().toString();
    printTable({"Detail", "Value"},
               {
                   {"Name", group.name()},
                   {"Currency", group.currency().code()},
                   {"Language", language},
                   {"Description", group.description().value_or("")},
               },
               2);
    std::println("The group started as Lisbon and was renamed when a day in Porto joined the plan.\n"
                 "Every total is kept in its currency, {}. The language is a BCP 47 tag the apps\n"
                 "show the group in, stored in canonical form: pt-pt became {}. A blank name is\n"
                 "refused:\n"
                 "  error: {}",
                 group.currency().code(), language, message(blank));

    check(blank.has_value() && Failure::GROUP_EMPTY_NAME == blank->failure(), "a blank name is refused");
    check("Lisbon and Porto" == group.name(), "the group is renamed");
    check("pt-PT" == language, "the language is canonical");
    return group;
}

// 2. Members.

/// An account with its label, its kind, and its details.
[[nodiscard]] std::string describe(Account const &account) {
    auto const *const iban = std::get_if<IbanAccountDetails>(&account.details);
    return nullptr == iban ? std::format("{}, username {}", account.name, std::get<UsernameAccountDetails>(account.details).username.toString())
                           : std::format("{}, IBAN {}", account.name, iban->iban.toString());
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
            member.lastName.has_value() ? std::format("{} {}", member.firstName, *member.lastName) : member.firstName,
            member.email.has_value() ? member.email->toString() : "",
            member.phone.has_value() ? member.phone->international() : "",
            cell(accounts),
        });
    }
    printTable({"Member", "Email", "Phone", "Accounts"}, rows, 4);
}

/// Prints what validation makes of IBANs, an email, phone numbers, and blank required text.
void validation(Group &group) {
    printLine("IBANs are checked against the SWIFT registry and their mod-97 check digits,\n"
              "emails against RFC 5321 and 5322, and phone numbers against their country's\n"
              "numbering plan. Required text must not be blank:");
    auto const iban = [](std::string const &input) {
        return std::vector<std::string>{
            std::format("\"{}\"", input),
            "IBAN",
            outcome([&] { return Iban::parse(input); },
                    [](Iban const &parsed) { return std::format("{}, country {}", parsed.toString(), parsed.countryCode()); }),
        };
    };
    auto const phone = [](std::string const &input) {
        return std::vector<std::string>{
            std::format("\"{}\"", input),
            "phone",
            outcome([&] { return PhoneNumber::parse(input); },
                    [](PhoneNumber const &parsed) { return std::format("{}, stored as {}", parsed.international(), parsed.toString()); }),
        };
    };
    auto const text = [](auto const &parsed) { return parsed.toString(); };
    printTable({"Input", "Checked as", "Result"},
               {
                   iban("ro49 aaaa 1b31 0075 9384 0000"),
                   iban("RO49 AAAA 1B31 0075 9384 0001"),
                   {"\"ana..popescu@example.com\"", "email", outcome([] { return Email::parse("ana..popescu@example.com"); }, text)},
                   phone("+351 912 345 678"),
                   phone("+40 612 345 678"),
                   {"\"   \"", "username", outcome([] { return Username::parse("   "); }, text)},
                   {"\"  \"", "first name", outcome([&] { return group.addMember(Member("  ")); }, [](MemberId const &id) { return id.value(); })},
               },
               3);
}

/// Adds the members of the trip to @p group, edits Bob, and prints them and what validation accepts.
[[nodiscard]] People members(Group &group) {
    section("2. Members");
    Member first("Ana");
    first.accounts = {
        Account(IbanAccountDetails{Iban::parse("RO49 AAAA 1B31 0075 9384 0000")}, "ING"),
        Account(UsernameAccountDetails{Username::parse("@ana")}, "Revolut"),
    };
    first.email = Email::parse("ana@example.com");
    first.lastName = "Popescu";
    first.phone = PhoneNumber::parse("+40 712 345 678");
    auto const ana = group.addMember(first);
    auto const bob = group.addMember(Member("Bob"));
    auto const carol = group.addMember(Member("Carol"));
    auto const dan = group.addMember(Member("Dan"));
    auto const eva = group.addMember(Member("Eva"));
    auto const filip = group.addMember(Member("Filip"));
    // An edit starts from a copy of the member, which keeps its identifier.
    auto stored = group.member(bob).value();
    stored.email = Email::parse("bob@example.com");
    stored.lastName = "Ionescu";
    auto const before = group.replaceMember(stored);
    printMembers(group);
    std::println("Only a first name is required. Ana gave every detail; Bob added his last name and\n"
                 "email later, through an edit that kept his identifier:\n"
                 "  {}\n"
                 "Eva is Carol's daughter, and Filip was added by mistake.",
                 bob.value());
    validation(group);

    check(!before.lastName.has_value(), "Bob had no last name");
    check(std::optional<std::string>("Ionescu") == group.member(bob).value().lastName, "Bob has his last name");
    check(2 == group.member(ana).value().accounts.size(), "Ana has two accounts");
    return {.ana = ana, .bob = bob, .carol = carol, .dan = dan, .eva = eva, .filip = filip};
}

// 3. Categories.

/// Adds Food, Lodging, and Transport to @p group, renames Transport, and shows what renaming refuses.
[[nodiscard]] Categories addCategories(Group &group) {
    section("3. Categories");
    auto const food = group.addCategory(Category("Food"));
    auto const lodging = group.addCategory(Category("Lodging"));
    auto const transport = group.addCategory(Category("Transport"));
    auto const homonym = refusal([&] { return group.addCategory(Category("FOOD")); });
    auto stored = group.category(transport).value();
    stored.name = "Getting around";
    static_cast<void>(group.replaceCategory(stored));
    auto uncategorised = group.categories().at(0);
    uncategorised.name = "Other";
    auto const builtIn = refusal([&] { return group.replaceCategory(uncategorised); });
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
    printTable({"Change", "Refused with"},
               {
                   {"Add a category named FOOD", message(homonym)},
                   {"Rename Uncategorised to Other", message(builtIn)},
               },
               2);

    check(homonym.has_value() && Failure::GROUP_DUPLICATE_CATEGORY_NAME == homonym->failure() && food == homonym->existing(),
          "a caseless homonym is refused");
    check(builtIn.has_value() && Failure::GROUP_BUILT_IN_CATEGORY == builtIn->failure(), "the built-in category keeps its name");
    check("Getting around" == group.category(transport).value().name, "Transport is renamed");
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
    auto const count = participants(expense.payers).size();
    auto const payers = 1 == count ? std::string() : std::format(", {} payers", count);
    std::string tip;
    if (0 != tipAmount(expense).minorUnits && expense.tip.has_value()) {
        auto const *const fixed = std::get_if<FixedTipValue>(&expense.tip->value);
        tip = std::format(", {} tip",
                          (nullptr == fixed ? std::get<PercentageTipValue>(expense.tip->value).percentage.toString() : fixed->amount.toString()));
    }
    return split + payers + tip;
}

/// Prints each expense of @p ids, with every involved member's payment and consumption.
void printShares(Group const &group, std::vector<ExpenseId> const &ids) {
    Rows rows;
    for (auto const &id : ids) {
        auto const expense = group.expense(id).value();
        auto const shares = group.expenseShares(id).value();
        std::vector<std::string> members;
        std::vector<std::string> paid;
        std::vector<std::string> consumed;
        for (auto const &share : shares.members) {
            members.push_back(firstName(group, share.member));
            paid.push_back(0 == share.paid.minorUnits ? "" : share.paid.toString());
            consumed.push_back(share.share.toString());
        }
        rows.push_back({expense.name, how(expense), cell(members), cell(paid), cell(consumed)});
    }
    printTable({"Expense", "Split", "Member", "Paid", "Consumed"}, rows, 3);
}

/// Prints the receipt of the expense @p id of @p group: who shared each line, its quantity, its unit price, and its total.
void printItems(Group const &group, ExpenseId const &id) {
    auto const expense = group.expense(id).value();
    auto const *const itemised = std::get_if<ItemisedExpenseSplit>(&expense.split);
    if (nullptr == itemised) {
        return;
    }
    Rows rows;
    for (auto const &item : itemised->items) {
        Money const lineTotal(item.unitPrice.currency, item.unitPrice.minorUnits * item.quantity);
        rows.push_back(
            {item.name, names(group, participants(item.split)), std::to_string(item.quantity), item.unitPrice.toString(), lineTotal.toString()});
    }
    printTable({"Item", "Shared by", "Quantity", "Unit price", "Line total"}, rows, 2);
}

/// The line @p name of @p quantity at @p unitPrice euros, which @p split shares.
[[nodiscard]] Item line(std::string name, Split split, std::string_view const unitPrice, std::int32_t const quantity) {
    Item item(std::move(name), std::move(split), eur(unitPrice));
    item.quantity = quantity;
    return item;
}

/// The receipt of the fado night: every line with its own split.
[[nodiscard]] std::vector<Item> fadoItems(People const &people) {
    return {
        line("Vinho verde", EqualSplit{{people.ana, people.bob, people.dan}}, "12.00", 2),
        line("Bacalhau", EqualSplit{{people.carol}}, "18.00", 1),
        line("Custard tarts",
             PartsSplit{
                 {
                     MemberParts(people.ana, Parts::parse("1")),
                     MemberParts(people.bob, Parts::parse("1")),
                     MemberParts(people.carol, Parts::parse("1")),
                     MemberParts(people.dan, Parts::parse("1")),
                     MemberParts(people.eva, Parts::parse("2")),
                 },
             },
             "1.50", 6),
        line("Juice", EqualSplit{{people.eva}}, "3.00", 1),
    };
}

/// @p expense in @p category.
[[nodiscard]] Expense inCategory(Expense expense, CategoryId const &category) {
    expense.category = category;
    return expense;
}

/// One expense for every way of dividing one, in order.
[[nodiscard]] std::vector<Expense> everyWay(People const &people, Categories const &categories) {
    std::vector<MemberPercentage> const rooms{
        MemberPercentage(people.ana, Percentage::parse("25")),
        MemberPercentage(people.bob, Percentage::parse("25")),
        MemberPercentage(people.carol, Percentage::parse("35")),
        MemberPercentage(people.dan, Percentage::parse("15")),
    };
    std::vector<MemberParts> const portions{
        MemberParts(people.ana, Parts::parse("1")), MemberParts(people.bob, Parts::parse("1")),   MemberParts(people.carol, Parts::parse("1")),
        MemberParts(people.dan, Parts::parse("1")), MemberParts(people.eva, Parts::parse("0.5")),
    };
    std::vector<MemberAmount> const tickets{
        MemberAmount(eur("24.50"), people.ana),
        MemberAmount(eur("24.50"), people.bob),
        MemberAmount(eur("24.50"), people.carol),
        MemberAmount(eur("15.00"), people.eva),
    };
    Expense fado(LocalCost{eur("54.00")}, october(4), "Fado night",
                 AmountsSplit{{MemberAmount(eur("39.40"), people.carol), MemberAmount(eur("20.00"), people.dan)}},
                 ItemisedExpenseSplit{fadoItems(people)});
    fado.category = categories.food;
    fado.description = "Tasca do Chico";
    fado.time = TimeOfDay::of(22, 15);
    fado.tip = Tip(TipDistribution::PROPORTIONAL, PercentageTipValue{Percentage::parse("10")});
    Expense ride(LocalCost{eur("45.00")}, october(5), "Tuk-tuk tour", EqualSplit{{people.dan}},
                 WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol, people.dan, people.eva}}});
    ride.category = categories.transport;
    ride.tip = Tip(TipDistribution::EQUAL, FixedTipValue{eur("5.00")});
    return {
        inCategory(Expense(LocalCost{eur("10.00")}, october(1), "Groceries", EqualSplit{{people.ana}},
                           WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol}}}),
                   categories.food),
        inCategory(Expense(LocalCost{eur("800.00")}, october(1), "Apartment", EqualSplit{{people.ana}}, WholeExpenseSplit{PercentagesSplit{rooms}}),
                   categories.lodging),
        inCategory(
            Expense(LocalCost{eur("90.00")}, october(2), "Seafood dinner", EqualSplit{{people.carol}}, WholeExpenseSplit{PartsSplit{portions}}),
            categories.food),
        inCategory(Expense(LocalCost{eur("88.50")}, october(3), "Train to Porto", EqualSplit{{people.bob}}, WholeExpenseSplit{AmountsSplit{tickets}}),
                   categories.transport),
        Expense(LocalCost{eur("60.00")}, october(3), "Museum", EqualSplit{{people.ana, people.bob}},
                WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol, people.dan}}}),
        fado,
        ride,
    };
}

/// The share of @p member in the expense @p id of @p group.
[[nodiscard]] MemberShare shareOf(Group const &group, ExpenseId const &id, MemberId const &member) {
    auto const shares = group.expenseShares(id).value();
    auto const found = std::ranges::find_if(shares.members, [&](MemberShare const &share) { return member == share.member; });
    check(shares.members.end() != found, "the member shares the expense");
    return *found;
}

/// Prints how the fado night's tip was shared, and its receipt.
void printFado(Group const &group, Expense const &fado) {
    auto const &tip = fado.tip.value();
    auto const *const fixed = std::get_if<FixedTipValue>(&tip.value);
    auto const rule = nullptr == fixed ? std::get<PercentageTipValue>(tip.value).percentage.toString() : fixed->amount.toString();
    auto const *const shared = TipDistribution::EQUAL == tip.distribution ? "equally" : "in proportion to what each member ordered";
    std::println("The groceries' 10.00 EUR did not divide into thirds, so one member consumed the\n"
                 "leftover cent; the expense's identifier picks which, so it moves from expense to\n"
                 "expense. The fado night, at {} on {} at {}, was itemised; its\n"
                 "{} tip of {} was shared {}:",
                 fado.time.value().toString(), fado.date.toString(), fado.description.value_or(""), rule, tipAmount(fado).toString(), shared);
    printItems(group, fado.id.value());
    printLine("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.");
}

/// Adds the seven expenses to @p group and prints how each divides; returns the seafood dinner's identifier.
[[nodiscard]] ExpenseId expenses(Group &group, People const &people, Categories const &categories) {
    section("4. Expenses, one for every way of dividing");
    std::vector<ExpenseId> ids;
    for (auto const &expense : everyWay(people, categories)) {
        ids.push_back(group.addExpense(expense));
    }
    printLine("Seven expenses cover every way of dividing one: equally, by percentage, by parts\n"
              "(Eva counts as half), by exact amounts, with two payers, item by item, and with a\n"
              "tip. Every member's consumption adds up to the expense's total, to the cent:");
    printShares(group, ids);
    auto const &groceries = ids.at(0);
    auto const &apartment = ids.at(1);
    auto const &seafood = ids.at(2);
    auto const &train = ids.at(3);
    auto const &museum = ids.at(4);
    auto const &fado = ids.at(5);
    auto const &tour = ids.at(6);
    auto const fadoExpense = group.expense(fado).value();
    printFado(group, fadoExpense);

    std::array<std::int64_t, 3> thirds{
        shareOf(group, groceries, people.ana).share.minorUnits,
        shareOf(group, groceries, people.bob).share.minorUnits,
        shareOf(group, groceries, people.carol).share.minorUnits,
    };
    std::ranges::sort(thirds);
    check((std::array<std::int64_t, 3>{333, 333, 334}) == thirds, "the groceries divide into 3.33, 3.33, and 3.34");
    check(eur("280.00") == shareOf(group, apartment, people.carol).share, "Carol's 35 % of the apartment is 280.00 EUR");
    check(eur("10.00") == shareOf(group, seafood, people.eva).share, "Eva's half part of the dinner is 10.00 EUR");
    check(eur("15.00") == shareOf(group, train, people.eva).share, "Eva's ticket is 15.00 EUR");
    check(eur("30.00") == shareOf(group, museum, people.bob).paid, "Bob paid 30.00 EUR of the museum");
    check(eur("15.00") == shareOf(group, museum, people.dan).share, "Dan's quarter of the museum is 15.00 EUR");
    check(4 == participants(group.expense(museum).value()).size(), "the museum names four members");
    check(eur("59.40") == total(fadoExpense), "the fado night comes to 59.40 EUR");
    check(eur("5.40") == tipAmount(fadoExpense), "its tip is 5.40 EUR");
    check(eur("21.45") == shareOf(group, fado, people.carol).share, "Carol's part of the fado night is 21.45 EUR");
    check(eur("6.60") == shareOf(group, fado, people.eva).share, "Eva's part of the fado night is 6.60 EUR");
    check(eur("10.00") == shareOf(group, tour, people.eva).share, "Eva's part of the tour is 10.00 EUR");
    return seafood;
}

// 5. Another currency.

/// Adds the airport parking in RON and Bob's repayment in cash, and shows how they convert.
void anotherCurrency(Group &group, People const &people, Categories const &categories) {
    section("5. Another currency");
    auto const euro = Currency::of("EUR");
    auto const rate = ExchangeRate::parse("0.2");
    auto const parking = group.addExpense(inCategory(Expense(ConvertedCost{.amount = ron("250.00"), .rate = rate}, october(6), "Airport parking",
                                                             EqualSplit{{people.ana}}, WholeExpenseSplit{EqualSplit{{people.ana, people.bob}}}),
                                                     categories.transport));
    Transfer repayment(ConvertedCost{.amount = ron("100.00"), .rate = rate}, october(6), people.bob, people.ana);
    repayment.note = "Cash for the parking";
    repayment.time = TimeOfDay::of(7, 40);
    auto const cash = group.addTransfer(repayment);
    printLine("Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in\n"
              "cash. An entry in another currency carries the rate into the group currency, how\n"
              "many EUR one RON is worth, and is converted once, when it is entered:");
    printEntries(group, [&](Currency const &currency) { return euro.code() != currency.code(); });
    auto const transfer = group.transfer(cash).value();
    auto const converted = ExchangeRate::parse("0.2011").convert(ron("250.00"), euro);
    std::println("Bob's transfer, at {} (\"{}\"), counts as {}.\n"
                 "A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,\n"
                 "which becomes {}.",
                 transfer.time.value().toString(), transfer.note.value_or(""), group.transferAmount(cash).value_or(Money(euro, 0)).toString(),
                 converted.toString());

    check(eur("50.28") == converted, "the conversion rounds to 50.28 EUR");
    check(eur("50.00") == group.expenseShares(parking).value().total, "the parking counts as 50.00 EUR");
    check(eur("20.00") == group.transferAmount(cash).value(), "the cash counts as 20.00 EUR");
    check(std::holds_alternative<ConvertedCost>(transfer.cost), "the cash was converted");
}

// 6. Edits.

/// Replaces an expense and a transfer, and removes one of each; returns what changed.
[[nodiscard]] Rows replaceAndRemove(Group &group, People const &people, Categories const &categories, ExpenseId const &seafood) {
    // The dinner's receipt said 99.00.
    auto dinner = group.expense(seafood).value();
    dinner.cost = LocalCost{eur("99.00")};
    dinner.description = "Cervejaria Ramiro";
    auto const previous = group.replaceExpense(dinner);
    // Bob entered the coffee twice.
    auto const coffee = inCategory(Expense(LocalCost{eur("6.00")}, october(5), "Coffee", EqualSplit{{people.bob}},
                                           WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol}}}),
                                   categories.food);
    static_cast<void>(group.addExpense(coffee));
    auto const copy = group.addExpense(coffee);
    auto const removed = group.removeExpense(copy);
    // Dan's part of the apartment was 120.00, not 100.00.
    auto const first = group.addTransfer(Transfer(LocalCost{eur("100.00")}, october(5), people.dan, people.ana));
    auto stored = group.transfer(first).value();
    stored.cost = LocalCost{eur("120.00")};
    stored.note = "Apartment";
    auto const replaced = group.replaceTransfer(stored);
    // Carol sent 10.00 to the wrong member.
    auto const mistake = group.addTransfer(Transfer(LocalCost{eur("10.00")}, october(5), people.carol, people.bob));
    auto const undone = group.removeTransfer(mistake);

    check(eur("90.00") == amountOf(previous.cost), "the dinner was 90.00 EUR");
    check(!group.expense(copy).has_value(), "the copy is gone");
    check(eur("120.00") == group.transferAmount(first).value(), "Dan's transfer is 120.00 EUR");
    check(!group.transfer(mistake).has_value(), "the mistake is gone");
    auto const corrected = group.expense(seafood).value();
    auto const apartment = group.transfer(first).value();
    return {
        {
            previous.name,
            std::format("{} became {}, at {}", amountOf(previous.cost).toString(), amountOf(corrected.cost).toString(),
                        corrected.description.value_or("")),
        },
        {removed.name, std::format("entered twice; the copy of {} was removed", amountOf(removed.cost).toString())},
        {
            std::format("{} to {}", firstName(group, people.dan), firstName(group, people.ana)),
            std::format("{} became {}, noted {}", amountOf(replaced.cost).toString(), amountOf(apartment.cost).toString(),
                        apartment.note.value_or("")),
        },
        {
            std::format("{} to {}", firstName(group, people.carol), firstName(group, people.bob)),
            std::format("{} sent to the wrong member; removed", amountOf(undone.cost).toString()),
        },
    };
}

/// Replaces and removes entries, then shows that a change that does not fit is refused.
void edits(Group &group, People const &people, Categories const &categories, ExpenseId const &seafood) {
    section("6. Edits");
    auto const changes = replaceAndRemove(group, people, categories, seafood);
    printLine("Entries are values: an edit starts from a copy made with to_builder, keeps the\n"
              "identifier, and replaces the entry; a removal takes it out with its totals.");
    printTable({"Entry", "Edit"}, changes, 2);
    printLine("The balances after the edits:");
    printBalances(group);

    auto const before = group.summary();
    std::vector<MemberPercentage> const halves{
        MemberPercentage(people.ana, Percentage::parse("50")),
        MemberPercentage(people.bob, Percentage::parse("49.99")),
    };
    auto const unbalanced = refusal([&] {
        return group.addExpense(
            Expense(LocalCost{eur("30.00")}, october(6), "Taxi", EqualSplit{{people.ana}}, WholeExpenseSplit{PercentagesSplit{halves}}));
    });
    auto const toHerself = refusal([&] { return group.addTransfer(Transfer(LocalCost{eur("5.00")}, october(6), people.ana, people.ana)); });
    auto const unrated = refusal([&] {
        return group.addExpense(
            Expense(LocalCost{ron("40.00")}, october(6), "Souvenirs", EqualSplit{{people.ana}}, WholeExpenseSplit{EqualSplit{{people.ana}}}));
    });
    printLine("A change that does not fit is refused, and the group stays as it was:");
    printTable({"Change", "Refused with"},
               {
                   {"A taxi split 50% and 49.99%", message(unbalanced)},
                   {"Ana paying herself back", message(toHerself)},
                   {"Souvenirs in RON, without a rate", message(unrated)},
               },
               2);
    printLine("The balances again, unchanged:");
    printBalances(group);

    check(unbalanced.has_value() && Failure::EXPENSE_PERCENTAGES_DO_NOT_ADD_UP == unbalanced->failure() &&
              std::optional<std::int64_t>(9'999) == unbalanced->totalBasisPoints(),
          "percentages that do not add up are refused");
    check(toHerself.has_value() && Failure::TRANSFER_SAME_MEMBER == toHerself->failure(), "a transfer to oneself is refused");
    check(unrated.has_value() && Failure::GROUP_MISSING_EXCHANGE_RATE == unrated->failure() &&
              std::optional<std::string>("RON") == unrated->currency(),
          "an amount in RON without a rate is refused");
    check(before == group.summary(), "the refusals change nothing");
}

// 7. Members come and go.

/// Removes Filip, archives Dan, shows what that refuses, and unarchives him.
void comingsAndGoings(Group &group, People const &people) {
    section("7. Members come and go");
    auto const removed = group.removeMember(people.filip);
    group.archiveMember(people.dan);
    auto const farewell = refusal([&] {
        return group.addExpense(Expense(LocalCost{eur("40.00")}, october(6), "Farewell dinner", EqualSplit{{people.ana}},
                                        WholeExpenseSplit{EqualSplit{{people.ana, people.bob, people.carol, people.dan}}}));
    });
    auto const inUse = refusal([&] { return group.removeMember(people.dan); });
    std::println("{} was never part of an entry, so removing him leaves no trace. Dan flew home\n"
                 "early and was archived: his history and his balance stay, but a new expense\n"
                 "cannot include him, and he cannot be removed while entries reference him.",
                 removed.firstName);
    auto const summary = group.summary();
    Rows rows;
    for (auto const &member : group.members()) {
        auto const found = totalsOf(summary, member.id.value());
        if (found.has_value()) {
            rows.push_back({member.firstName, group.isArchived(member.id.value()) ? "archived" : "active", toString(found->balance)});
        }
    }
    printTable({"Member", "Status", "Balance"}, rows, 2);
    printTable({"Change", "Refused with"},
               {
                   {"A farewell dinner with Dan", message(farewell)},
                   {"Removing Dan", message(inUse)},
               },
               2);
    group.unarchiveMember(people.dan);
    printLine("When Dan joins the next trip, unarchiving him makes him active again.");

    check("Filip" == removed.firstName, "Filip was removed");
    check(!group.member(people.filip).has_value(), "Filip is gone");
    check(farewell.has_value() && Failure::GROUP_ARCHIVED_MEMBER == farewell->failure() && people.dan == farewell->member(),
          "an archived member joins no expense");
    check(inUse.has_value() && Failure::GROUP_MEMBER_IN_USE == inUse->failure(), "a referenced member stays");
    check(!group.isArchived(people.dan), "Dan is active again");
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
void summary(Group const &group, People const &people, Categories const &categories) {
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
    Money const zero(group.currency(), 0);
    Rows rows;
    for (auto const &member : group.members()) {
        auto const own = byCategory(totals, member.id.value());
        rows.push_back({
            member.firstName,
            totalOf(own, categories.food).value_or(zero).toString(),
            totalOf(own, categories.lodging).value_or(zero).toString(),
            totalOf(own, categories.transport).value_or(zero).toString(),
            totalOf(own, builtIn).value_or(zero).toString(),
        });
    }
    printTable({"Member", "Food", "Lodging", "Getting around", "Uncategorised"}, rows, 1);

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
void removingACategory(Group &group, Categories const &categories) {
    section("9. Removing a category");
    auto const removed = group.removeCategory(categories.transport);
    auto const uncategorised = builtInCategory(group);
    auto const builtIn = refusal([&] { return group.removeCategory(uncategorised); });
    std::println("Removing {} moves its expenses to Uncategorised and the category\n"
                 "totals with them; no balance changes. The built-in category cannot be removed:\n"
                 "  error: {}",
                 removed.name, message(builtIn));
    printCategories(group);

    check(builtIn.has_value() && Failure::GROUP_BUILT_IN_CATEGORY == builtIn->failure(), "the built-in category cannot be removed");
    check(!group.category(categories.transport).has_value(), "Getting around is gone");
    check(std::optional<Money>(eur("248.50")) == totalOf(group.summary().categoryTotals, uncategorised), "Uncategorised comes to 248.50 EUR");
    auto const expenses = group.expenses();
    check(std::ranges::none_of(expenses, [&](Expense const &expense) { return categories.transport == expense.category; }),
          "no expense is in Getting around");
}

// 10. A new currency.

/// Moves the books of @p group to RON, which a missing rate first refuses.
void newCurrency(Group &group, People const &people) {
    section("10. A new currency");
    auto const euro = Currency::of("EUR");
    auto const leu = Currency::of("RON");
    auto const missing = refusal([&] {
        group.changeCurrency(leu, {});
        return 0;
    });
    group.changeCurrency(leu, std::array{CurrencyRate(euro, ExchangeRate::parse("5"))});
    std::println("Home again, the group moves its books to RON. The change needs a rate into RON\n"
                 "for every other currency in use, here only EUR, or it is refused:\n"
                 "  error: {}\n"
                 "Entries in EUR take the new rate, and the RON parking and cash become local:",
                 message(missing));
    printEntries(group, [](Currency const &) { return true; });
    std::println("Each entry keeps its currency and converts at the new rate. At 5 every share\n"
                 "converts exactly, so each balance is five times what it was. The trip cost\n"
                 "{}:",
                 group.summary().totalSpent.toString());
    printBalances(group);

    auto const summary = group.summary();
    check(missing.has_value() && Failure::GROUP_MISSING_EXCHANGE_RATE == missing->failure() &&
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
    case SettlementStrategy::DIRECT:
        return "Direct";
    case SettlementStrategy::GREEDY:
        return "Greedy";
    case SettlementStrategy::MINIMUM_TRANSFERS:
        return "MinimumTransfers";
    }
    std::unreachable();
}

/// Prints how many payments each strategy plans, then records the minimum plan as transfers.
void settlingUp(Group &group) {
    section("11. Settling up");
    Rows counts;
    for (auto const strategy : {SettlementStrategy::DIRECT, SettlementStrategy::GREEDY, SettlementStrategy::MINIMUM_TRANSFERS}) {
        auto const settlement = group.settlement(strategy);
        check(strategy == settlement.strategy, "the plan is the strategy's");
        counts.push_back({std::string(strategyName(settlement.strategy)), std::to_string(settlement.payments.size())});
    }
    printLine("Each strategy turns the balances into payments: the direct one pays back every\n"
              "pair's own debts, greedy pays the largest amounts first, and the minimum one\n"
              "finds the fewest payments.");
    printTable({"Strategy", "Payments"}, counts, 1);
    auto const plan = group.settlement(SettlementStrategy::MINIMUM_TRANSFERS);
    printLine("The group follows the minimum plan:");
    Rows payments;
    for (auto const &payment : plan.payments) {
        payments.push_back({firstName(group, payment.from), firstName(group, payment.to), payment.amount.toString()});
    }
    printTable({"From", "To", "Amount"}, payments, 2);
    for (auto const &payment : plan.payments) {
        static_cast<void>(group.addTransfer(Transfer(LocalCost{payment.amount}, october(7), payment.from, payment.to)));
    }
    printLine("Each payment, once made, is recorded as a transfer, and every balance is settled:");
    printBalances(group);

    check(group.settlement(SettlementStrategy::GREEDY).payments.empty(), "nothing is left to pay");
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
void savingAndLoading(Group const &group, People const &people, Categories const &categories) {
    section("12. Saving and loading");
    auto const saved = group.save();
    std::println("The group saves to {} bytes of JSON: its details and entries, never the totals.\n"
                 "Ana's accounts are saved in clear text, as the library holds them:",
                 saved.size());
    auto const ana = group.member(people.ana).value();
    for (auto const &account : ana.accounts) {
        auto const *const iban = std::get_if<IbanAccountDetails>(&account.details);
        auto const details = nullptr == iban ? std::get<UsernameAccountDetails>(account.details).username.toString() : iban->iban.toString();
        check(saved.contains(details), "the details are saved as they are");
        std::println("  {}: {}", account.name, details);
    }
    auto const loaded = Group::load(saved);
    printLine("Loading replays every entry through the same checks as adding it and rebuilds the\n"
              "totals: the loaded group equals the saved one. Reading a SavedGroup and loading it\n"
              "with into_group reports a refusal as a typed error instead of a message:");
    auto const refused = [&](std::string_view const pattern, std::string_view const replacement) {
        auto const changed = replaceFirst(saved, pattern, replacement);
        check(changed != saved, "the change changes the file");
        auto const refusedLoad = refusal([&] { return Group::load(changed); });
        check(refusedLoad.has_value() && ErrorKind::GROUP == refusedLoad->kind(), "the changed file is refused");
        return refusedLoad.value();
    };
    auto const format = refused(R"("format": 2)", R"("format": 3)");
    auto const blank = refused(R"("name": "Lisbon and Porto")", R"("name": " ")");
    auto const homonym = refused(R"("name": "Lodging")", R"("name": "FOOD")");
    printTable({"Change to the file", "Message"},
               {
                   {"\"format\": 3", format.what()},
                   {"a blank group name", blank.what()},
                   {"Lodging renamed FOOD", homonym.what()},
               },
               2);

    check(saved == loaded.save(), "the loaded group saves as the group did");
    check(group.summary() == loaded.summary(), "the loaded group has the group's totals");
    check(Failure::GROUP_UNSUPPORTED_FORMAT == format.failure() && std::optional<std::int64_t>(3) == format.format(), "a later format is refused");
    check(Failure::GROUP_EMPTY_NAME == blank.failure(), "a blank name is refused");
    check(Failure::GROUP_DUPLICATE_CATEGORY_NAME == homonym.failure() && categories.food == homonym.existing(), "a caseless homonym is refused");
}

// 13. Values on their own.

/// Prints four currencies, and that an unknown code is refused.
void currencies() {
    Rows rows;
    for (auto const *const code : {"EUR", "RON", "JPY", "BHD"}) {
        auto const currency = Currency::of(code);
        rows.push_back({currency.code(), currency.name(), std::to_string(currency.numericCode()), std::to_string(currency.decimalPlaces())});
    }
    auto const unknown = refusal([] { return Currency::of("XYZ"); });
    std::println("Currencies are the {} active ISO 4217 codes with a minor unit; an unknown code is\n"
                 "refused ({}):",
                 Currency::all().size(), message(unknown));
    printTable({"Code", "Name", "Numeric code", "Decimal places"}, rows, 2);
    check(3 == Currency::of("BHD").decimalPlaces(), "the dinar has three decimal places");
}

/// Prints what amounts do: sums, differences, refusals, minor units, and a cost's rate kept for another amount.
void amounts() {
    auto const sumOf = eur("12.34").plus(eur("0.66"));
    auto const change = eur("5.00").minus(eur("7.50"));
    auto const mixed = refusal([] { return eur("1.00").plus(ron("1.00")); });
    auto const yen = Money::parse("1500", Currency::of("JPY"));
    Money const dinars(Currency::of("BHD"), 1'234);
    Money const nothing(Currency::of("RON"), 0);
    ConvertedCost const cost{.amount = eur("10.00"), .rate = ExchangeRate::parse("4.9765")};
    ConvertedCost const doubled{.amount = eur("20.00"), .rate = cost.rate};
    printLine("Amounts are exact minor units in their currency:");
    printTable({"Input", "Result"},
               {
                   {"12.34 EUR + 0.66 EUR", sumOf.toString()},
                   {"5.00 EUR − 7.50 EUR", std::format("{}, negative: {}", change.toString(), (0 > change.minorUnits ? "true" : "false"))},
                   {"1.00 EUR + 1.00 RON", message(mixed)},
                   {"1.005 EUR", message(refusal([] { return eur("1.005"); }))},
                   {"1500 JPY", std::format("{}, {} minor units", yen.toString(), yen.minorUnits)},
                   {"1234 minor units of BHD", dinars.toString()},
                   {"zero RON", std::format("{}, zero: {}", nothing.toString(), (0 == nothing.minorUnits ? "true" : "false"))},
                   {
                       "10.00 EUR at 4.9765",
                       std::format("{} at {}, then {} at the same rate", cost.amount.toString(), cost.rate.toString(), doubled.amount.toString()),
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
    auto const percentage = Percentage::parse("12.5");
    auto const parts = Parts::parse("1.5");
    auto const exchange = ExchangeRate::parse("4.97650");
    printLine("Percentages, parts, and rates are exact decimals:");
    printTable({"Input", "Result"},
               {
                   {"12.5%", std::format("{}, {} basis points", percentage.toString(), percentage.basisPoints)},
                   {"333 basis points", Percentage{333}.toString()},
                   {"33.333%", message(refusal([] { return Percentage::parse("33.333"); }))},
                   {"1.5 parts", std::format("{}, {} hundredths", parts.toString(), parts.hundredths)},
                   {"3 whole parts", Parts{300}.toString()},
                   {"25 hundredths of a part", Parts{25}.toString()},
                   {"0 parts", message(refusal([] { return Parts::parse("0"); }))},
                   {"a rate of 4.97650", exchange.toString()},
                   {"a rate of 0", message(refusal([] { return ExchangeRate::parse("0"); }))},
               },
               2);
    check(1'250 == percentage.basisPoints, "12.5 % is 1250 basis points");
    check(150 == parts.hundredths, "1.5 parts are 150 hundredths");
}

/// Prints what dates, times, identifiers, and language tags read and print.
void datesAndIdentifiers(Group const &group, People const &people) {
    auto const date = CalendarDate::parse("2026-10-01");
    auto const time = TimeOfDay::parse("22:15");
    auto const found = group.member(people.ana).value().id;
    auto const language = Language::parse("pt-pt");
    printLine("Dates, times, identifiers, and language tags read and print the ISO forms:");
    printTable({"Input", "Result"},
               {
                   {
                       "2026-10-01",
                       std::format("{}: day {}, month {}, year {}", date.toString(), date.day(), date.month(), date.year()),
                   },
                   {"29 February 2027", message(refusal([] { return CalendarDate::of(29, 2, 2027); }))},
                   {"22:15", std::format("{}: hour {}, minute {}", time.toString(), time.hour(), time.minute())},
                   {"24:00", message(refusal([] { return TimeOfDay::of(24, 0); }))},
                   {
                       "Ana's identifier",
                       std::format("{}, the same after a round trip: {}", people.ana.value(),
                                   (std::optional<MemberId>(people.ana) == found ? "true" : "false")),
                   },
                   {"not-a-uuid", message(refusal([&] { return group.member(MemberId("not-a-uuid")); }))},
                   {"pt-pt", language.toString()},
                   {"pt_PT", message(refusal([] { return Language::parse("pt_PT"); }))},
               },
               2);
    check(1 == date.day() && 10 == date.month() && 2026 == date.year(), "the date reads as 1 October 2026");
    check(22 == time.hour() && 15 == time.minute(), "the time reads as 22:15");
    check(std::optional<MemberId>(people.ana) == found, "Ana's identifier comes back the same");
}

/// Prints the values on their own: currencies, amounts, ratios, dates, times, identifiers, and language tags.
void values(Group const &group, People const &people) {
    section("13. Values on their own");
    currencies();
    amounts();
    ratios();
    datesAndIdentifiers(group, people);
}

// 14. Sharing the group.

/// Shares @p group between tasks: an expense added through another handle, a settlement computed asynchronously, and a
/// change that waits for a report's read and is cancelled.
void sharing(Group &group, People const &people) {
    section("14. Sharing the group");
    // Another handle on the same group, as another task holds: a group is shared as it is.
    auto &other = group;
    std::vector<MemberId> const everyone{people.ana, people.bob, people.carol, people.dan, people.eva};
    static_cast<void>(other
                          .addExpenseAsync(Expense(LocalCost{ron("75.00")}, october(8), "Airport coffee", EqualSplit{{people.bob}},
                                                   WholeExpenseSplit{EqualSplit{everyone}}))
                          .get());
    auto const plan = group.settlementAsync(SettlementStrategy::MINIMUM_TRANSFERS).get();
    printLine("An app shares the group between its tasks and threads through a SharedGroup,\n"
              "whose calls are futures any executor runs; here the library's block_on runs\n"
              "each one on this thread. Bob adds the airport coffee through another handle,\n"
              "and the plan that settles it comes through the first:");
    {
        auto const reader = group.readAsync().get();
        Rows rows;
        for (auto const &payment : plan.payments) {
            rows.push_back({
                reader.member(payment.from).value().firstName,
                reader.member(payment.to).value().firstName,
                payment.amount.toString(),
            });
        }
        printTable({"From", "To", "Amount"}, rows, 2);
    }

    auto const before = group.summaryAsync().get();
    std::vector<CurrencyRate> const rates{CurrencyRate(Currency::of("RON"), ExchangeRate::parse("0.2"))};
    // A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
    // behind the change, so the report reads through its own handle.
    auto report = group.readAsync().get();
    Cancellation cancellation;
    auto change = group.changeCurrencyAsync(Currency::of("EUR"), rates, &cancellation);
    auto const reported = report.summary().totalSpent;
    // Cancelling stops the waiting change, which leaves the group as it was.
    cancellation.cancel();
    report.release();
    auto const cancelled = refusal([&] {
        change.get();
        return 0;
    });
    auto const after = group.summaryAsync().get();
    auto const currency = group.currencyAsync().get();
    std::println("While a report reads the group, a change of its currency to EUR waits for the\n"
                 "report. The app drops the waiting change, which cancels it: the group still\n"
                 "keeps its books in {}, and the trip cost {}, as the report read.",
                 currency.code(), after.totalSpent.toString());
    std::println("With the other handle gone, the app takes the group back: {} expenses.", group.expensesAsync().get().size());

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

int main() { return eranos::examples::runPort(eranos::examples::run); }
