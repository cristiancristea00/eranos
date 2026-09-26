// The tests of the C++ layer: every value, record, and alternative through its conversions, every group call in both
// forms, the read handle, the cancellation, and every error the library reports with what it holds. Each test is a
// claim, named for what it states, whose comment says what breaks in an app if it regresses. The program prints every
// claim that fails and ends with a failure status when one does.

#include "eranos.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <exception>
#include <functional>
#include <future>
#include <iostream>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos {
namespace {

/// The claims of one test that failed.
class Check final {
  public:
    /// Records @p claim as failed unless @p holds.
    ///
    /// @param holds Whether the claim holds
    /// @param claim What the test claims
    void that(bool const holds, std::string_view const claim) {
        if (!holds) {
            failures.emplace_back(claim);
        }
    }

    /// The claims that failed.
    ///
    /// @return The claims
    [[nodiscard]] std::vector<std::string> const &failed() const noexcept { return failures; }

  private:
    std::vector<std::string> failures;
};

/// The error @p call throws, or none when it returns.
template <typename Call> [[nodiscard]] std::optional<Error> thrown(Call const &call) {
    try {
        static_cast<void>(call());
    } catch (Error const &error) {
        return error;
    }
    return std::nullopt;
}

/// Whether @p error is of @p kind with @p failure.
[[nodiscard]] bool isFailure(std::optional<Error> const &error, ErrorKind const kind, std::optional<Failure> const failure) noexcept {
    return error.has_value() && error->kind() == kind && error->failure() == failure;
}

/// Checks that @p value equals @p same and differs from @p other through both operators: a comparison that left a field out
/// would merge two values an app holds apart, and one whose two operators disagreed would answer a question twice.
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the value, its twin, and another, in the order the claim reads them.
template <typename Value> void checkValue(Check &check, Value const &value, Value const &same, Value const &other, std::string_view const what) {
    check.that(value == same && !(value != same), what);
    check.that(value != other && !(value == other), what);
}

/// Checks an identifier type: equal to its twin with one hash, different from another, and ordered by text, as a map or a set of
/// them expects.
template <typename Identifier> void checkIdentifier(Check &check, std::string_view const what) {
    Identifier const first("0199a000-0000-7000-8000-000000000001");
    Identifier const twin(first.value());
    Identifier const second("0199a000-0000-7000-8000-000000000002");
    checkValue(check, first, twin, second, what);
    check.that(first < second && !(second < first), what);
    check.that(std::hash<Identifier>()(first) == std::hash<Identifier>()(twin), what);
    check.that("0199a000-0000-7000-8000-000000000001" == first.value(), what);
}

/// A group of two, Ana and Bob, with a category, an expense, and a transfer, and the identifiers the group gave them.
class Flat final {
  public:
    Flat()
        : euro(Currency::of("EUR")), day(CalendarDate::of(23, 9, 2026)),
          group(Group::create(euro, "Flat", GroupOptions{"Rent", Language::parse("pt-pt")})), ana(group.addMember(Member("Ana"))),
          bob(group.addMember(Member("Bob"))), food(group.addCategory(Category("Food"))), dinner(group.addExpense(dinnerExpense())),
          repayment(group.addTransfer(Transfer(LocalCost{euros(1'000)}, day, bob, ana))) {}

    /// @p minorUnits cents.
    [[nodiscard]] Money euros(std::int64_t const minorUnits) const { return {euro, minorUnits}; }

    /// An expense of 30.00 EUR, called @p name, that @p payer paid and @p members share equally.
    [[nodiscard]] Expense expense(std::string name, MemberId const &payer, std::vector<MemberId> members) const {
        return {LocalCost{euros(3'000)}, day, std::move(name), EqualSplit{{payer}}, WholeExpenseSplit{EqualSplit{std::move(members)}}};
    }

    Currency euro;
    CalendarDate day;
    Group group;
    MemberId ana;
    MemberId bob;
    CategoryId food;
    ExpenseId dinner;
    TransferId repayment;

  private:
    /// The dinner Ana paid for the two of them, in the food category.
    [[nodiscard]] Expense dinnerExpense() const {
        auto dinnerEntry = expense("Dinner", ana, {ana, bob});
        dinnerEntry.category = food;
        return dinnerEntry;
    }
};

// --- Values -----------------------------------------------------------------------------------------------------------

/// A currency whose details came out wrong would format every amount in it wrongly.
void aCurrencyHoldsWhatTheLibraryKnowsOfIt(Check &check) {
    auto const euro = Currency::of("EUR");
    auto const yen = Currency::of("JPY");
    check.that("EUR" == euro.code() && 2 == euro.decimalPlaces() && "Euro" == euro.name() && 978 == euro.numericCode(), "the euro's details");
    check.that(0 == yen.decimalPlaces(), "the yen has no decimal places");
    checkValue(check, euro, Currency::of("EUR"), yen, "currencies compare by code");
    check.that(euro < yen && !(yen < euro), "currencies order by code");
}

/// A list that missed a currency or came out of order would hide one from the currency pickers of an app.
void everyCurrencyTheLibraryListsIsListedByCode(Check &check) {
    auto const all = Currency::all();
    check.that(std::is_sorted(all.begin(), all.end()), "the currencies are in the order of their codes");
    check.that(std::find(all.begin(), all.end(), Currency::of("EUR")) != all.end(), "the euro is listed");
}

/// A code the library does not list, taken, would make an amount no call can format.
void aCodeTheLibraryDoesNotListIsRefused(Check &check) {
    auto const unknown = thrown([] { return Currency::of("XYZ"); });
    check.that(isFailure(unknown, ErrorKind::MONEY, Failure::MONEY_UNKNOWN_CURRENCY), "an unknown code is refused");
    check.that(unknown.has_value() && "unknown currency code" == std::string_view(unknown.value().what()),
               "the message names the failure, not the code");
}

/// Bytes that are not UTF-8, passed on, would reach the library as text it cannot hold.
void textThatIsNotUtf8IsRefusedBeforeTheLibrarySeesIt(Check &check) {
    auto const refused = thrown([] { return Currency::of("\xff"); });
    check.that(isFailure(refused, ErrorKind::INVALID_TEXT, std::nullopt), "the bytes are refused as text");
}

/// An amount that parsed or formatted otherwise than the library does would show one number in an app and store another.
void anAmountParsesAndFormatsAsTheLibraryWritesIt(Check &check) {
    auto const euro = Currency::of("EUR");
    auto const price = Money::parse("12.5", euro);
    check.that(price.currency == euro && 1'250 == price.minorUnits, "12.5 is 1250 cents");
    check.that("12.50 EUR" == price.toString(), "the amount formats with its currency");
    checkValue(check, price, Money(euro, 1'250), Money(euro, 1'251), "amounts compare by currency and minor units");
    check.that(isFailure(thrown([&] { return Money::parse("twelve", euro); }), ErrorKind::MONEY, Failure::MONEY_INVALID_AMOUNT),
               "a word is no amount");
    auto const precise = thrown([&] { return Money::parse("1.234", euro); });
    check.that(isFailure(precise, ErrorKind::MONEY, Failure::MONEY_TOO_MANY_DECIMALS), "the euro has two decimal places");
    check.that(precise.has_value() && 2 == precise.value().allowed() && "EUR" == precise.value().currency(),
               "the error holds the decimal places and the currency");
}

/// Arithmetic that mixed currencies or wrapped around would put a wrong total in a report.
void amountsInOneCurrencyAddAndSubtract(Check &check) {
    auto const euro = Currency::of("EUR");
    auto const lei = Currency::of("RON");
    check.that(Money(euro, 1'000).plus(Money(euro, 250)) == Money(euro, 1'250), "10.00 EUR and 2.50 EUR make 12.50 EUR");
    check.that(Money(euro, 1'000).minus(Money(euro, 250)) == Money(euro, 750), "10.00 EUR less 2.50 EUR is 7.50 EUR");
    auto const mixed = thrown([&] { return Money(euro, 1).plus(Money(lei, 1)); });
    check.that(isFailure(mixed, ErrorKind::MONEY, Failure::MONEY_CURRENCY_MISMATCH), "amounts in two currencies do not add");
    check.that(mixed.has_value() && "EUR" == mixed.value().left() && "RON" == mixed.value().right(), "the error names both currencies");
    auto const largest = Money(euro, INT64_MAX);
    check.that(isFailure(thrown([&] { return largest.plus(Money(euro, 1)); }), ErrorKind::MONEY, Failure::MONEY_OVERFLOW), "a sum out of range");
}

/// A rate that parsed or converted otherwise than the library does would change the share of every converted expense.
void aRateParsesAndConvertsAsTheLibraryDoes(Check &check) {
    auto const euro = Currency::of("EUR");
    auto const lei = Currency::of("RON");
    auto const rate = ExchangeRate::parse("4.970");
    check.that("4.97" == rate.toString(), "the rate is written without trailing zeros");
    checkValue(check, rate, ExchangeRate::parse("4.97"), ExchangeRate::parse("5"), "rates compare by value");
    check.that(ExchangeRate::parse("0.25").convert(Money(lei, 2), euro) == Money(euro, 1),
               "0.02 RON at 0.25 is 0.01 EUR, rounded half away from zero");
    check.that(isFailure(thrown([] { return ExchangeRate::parse("0"); }), ErrorKind::MONEY, Failure::MONEY_INVALID_EXCHANGE_RATE),
               "a rate is positive");
    auto const overflow = thrown([&] { return ExchangeRate::parse("2").convert(Money(lei, INT64_MAX), euro); });
    check.that(isFailure(overflow, ErrorKind::MONEY, Failure::MONEY_OVERFLOW), "a converted amount out of range");
}

/// A percentage or a number of parts read otherwise than the library reads it would divide a split otherwise than it shows.
void percentagesAndPartsParseAndFormatAsTheLibraryDoes(Check &check) {
    auto const percentage = Percentage::parse("12.5 %");
    check.that(1'250 == percentage.basisPoints && "12.5%" == percentage.toString(), "12.5 % is 1250 basis points");
    checkValue(check, percentage, Percentage{1'250}, Percentage{1'251}, "percentages compare by basis points");
    check.that(isFailure(thrown([] { return Percentage::parse("half"); }), ErrorKind::MONEY, Failure::MONEY_INVALID_PERCENTAGE),
               "a word is no percentage");
    check.that(isFailure(thrown([] { return Percentage{-1}.toString(); }), ErrorKind::MONEY, Failure::MONEY_INVALID_PERCENTAGE),
               "a negative percentage");
    auto const parts = Parts::parse("0.5");
    check.that(50 == parts.hundredths && "0.5" == parts.toString(), "half a part is 50 hundredths");
    checkValue(check, parts, Parts{50}, Parts{51}, "parts compare by hundredths");
    check.that(isFailure(thrown([] { return Parts::parse("half"); }), ErrorKind::MONEY, Failure::MONEY_INVALID_PARTS),
               "a word is no number of parts");
    check.that(isFailure(thrown([] { return Parts{-1}.toString(); }), ErrorKind::MONEY, Failure::MONEY_INVALID_PARTS), "negative parts");
}

/// A date or a time the calendar does not have, taken, would date an expense on a day that never was.
void datesAndTimesAreCheckedWhenMade(Check &check) {
    auto const date = CalendarDate::of(23, 9, 2026);
    check.that(23 == date.day() && 9 == date.month() && 2026 == date.year() && "2026-09-23" == date.toString(), "the date's fields and text");
    checkValue(check, date, CalendarDate::parse("2026-09-23"), CalendarDate::of(24, 9, 2026), "dates compare by day");
    check.that(isFailure(thrown([] { return CalendarDate::of(30, 2, 2026); }), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_DATE), "30 February");
    check.that(isFailure(thrown([] { return CalendarDate::parse("2026-02-30"); }), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_DATE), "parsed");
    auto const time = TimeOfDay::of(18, 30);
    check.that(18 == time.hour() && 30 == time.minute() && "18:30" == time.toString(), "the time's fields and text");
    checkValue(check, time, TimeOfDay::parse("18:30"), TimeOfDay::of(18, 31), "times compare by minute");
    check.that(isFailure(thrown([] { return TimeOfDay::of(25, 0); }), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_TIME), "hour 25");
    check.that(isFailure(thrown([] { return TimeOfDay::parse("25:00"); }), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_TIME), "parsed");
}

/// Contact details and accounts kept as typed would differ from the same details typed otherwise, and a wrong one would pass.
void contactsAndAccountsAreCheckedAndMadeCanonical(Check &check) {
    auto const email = Email::parse(" ana@example.com ");
    check.that("ana@example.com" == email.toString(), "an email is trimmed");
    checkValue(check, email, Email::parse("ana@example.com"), Email::parse("bob@example.com"), "emails compare by text");
    check.that(isFailure(thrown([] { return Email::parse("ana"); }), ErrorKind::CONTACT, Failure::CONTACT_INVALID_EMAIL), "an email has a domain");
    auto const phone = PhoneNumber::parse("+40 712 345 678");
    check.that("+40712345678" == phone.toString() && "+40 712 345 678" == phone.international(), "a phone number's two forms");
    checkValue(check, phone, PhoneNumber::parse("+40712345678"), PhoneNumber::parse("+40712345679"), "phone numbers compare by number");
    auto const local = thrown([] { return PhoneNumber::parse("0712345678"); });
    check.that(isFailure(local, ErrorKind::CONTACT, Failure::CONTACT_INVALID_PHONE_NUMBER), "a phone number has a country code");
    auto const iban = Iban::parse("ro49 aaaa 1b31 0075 9384 0000");
    check.that("RO49AAAA1B31007593840000" == iban.toString() && "RO" == iban.countryCode(), "an IBAN in its electronic form");
    checkValue(check, iban, Iban::parse("RO49AAAA1B31007593840000"), Iban::parse("GB82WEST12345698765432"), "IBANs compare by text");
    auto const checksum = thrown([] { return Iban::parse("RO49AAAA1B31007593840001"); });
    check.that(isFailure(checksum, ErrorKind::ACCOUNT, Failure::ACCOUNT_INVALID_IBAN) && IbanReason::INVALID_CHECKSUM == checksum.value().reason(),
               "an IBAN whose check digits do not match");
    check.that(checksum.has_value() && "invalid IBAN: the check digits do not match" == std::string_view(checksum.value().what()), "the message");
    auto const length = thrown([] { return Iban::parse("RO49AAAA1B3100759384000"); });
    check.that(length.has_value() && IbanReason::WRONG_LENGTH == length.value().reason() && 24 == length.value().expectedLength() &&
                   23 == length.value().foundLength(),
               "an IBAN one character short holds both lengths");
    auto const username = Username::parse(" ana.pays ");
    check.that("ana.pays" == username.toString(), "a username is trimmed");
    checkValue(check, username, Username::parse("ana.pays"), Username::parse("bob.pays"), "usernames compare by text");
    check.that(isFailure(thrown([] { return Username::parse(" "); }), ErrorKind::ACCOUNT, Failure::ACCOUNT_EMPTY_USERNAME), "a blank username");
    auto const language = Language::parse("pt-pt");
    check.that("pt-PT" == language.toString(), "a language tag is canonical");
    checkValue(check, language, Language::parse("pt-PT"), Language::parse("ro-RO"), "languages compare by tag");
    auto const tag = thrown([] { return Language::parse("not a tag"); });
    check.that(isFailure(tag, ErrorKind::LANGUAGE, Failure::LANGUAGE_INVALID_LANGUAGE), "a language is a tag");
}

/// An identifier that compared or hashed otherwise than by its text would lose an entry in the maps and sets of an app.
void everyIdentifierComparesAndHashesByItsText(Check &check) {
    checkIdentifier<GroupId>(check, "a group's identifier");
    checkIdentifier<MemberId>(check, "a member's identifier");
    checkIdentifier<CategoryId>(check, "a category's identifier");
    checkIdentifier<ExpenseId>(check, "an expense's identifier");
    checkIdentifier<TransferId>(check, "a transfer's identifier");
}

// --- Records and alternatives ------------------------------------------------------------------------------------------

/// A record or an alternative whose comparison left a field out would merge two entries an app holds apart.
void everyRecordAndAlternativeEqualsItsTwinAndDiffersFromAnother(Check &check) {
    auto const euro = Currency::of("EUR");
    MemberId const ana("0199a000-0000-7000-8000-000000000001");
    MemberId const bob("0199a000-0000-7000-8000-000000000002");
    CategoryId const food("0199a000-0000-7000-8000-000000000003");
    Money const one(euro, 100);
    Money const two(euro, 200);
    auto const day = CalendarDate::of(23, 9, 2026);
    checkValue(check, MemberAmount(one, ana), MemberAmount(one, ana), MemberAmount(one, bob), "member amounts");
    checkValue(check, MemberParts(ana, Parts{100}), MemberParts(ana, Parts{100}), MemberParts(ana, Parts{50}), "member parts");
    checkValue(check, MemberPercentage(ana, Percentage{5'000}), MemberPercentage(ana, Percentage{5'000}), MemberPercentage(bob, Percentage{5'000}),
               "member percentages");
    checkValue(check, AmountsSplit{{MemberAmount(one, ana)}}, AmountsSplit{{MemberAmount(one, ana)}}, AmountsSplit{{MemberAmount(two, ana)}},
               "amounts");
    checkValue(check, EqualSplit{{ana}}, EqualSplit{{ana}}, EqualSplit{{ana, bob}}, "equal splits");
    checkValue(check, PartsSplit{{MemberParts(ana, Parts{100})}}, PartsSplit{{MemberParts(ana, Parts{100})}}, PartsSplit{{}}, "parts splits");
    checkValue(check, PercentagesSplit{{MemberPercentage(ana, Percentage{10'000})}}, PercentagesSplit{{MemberPercentage(ana, Percentage{10'000})}},
               PercentagesSplit{{}}, "percentage splits");
    Item const pizza("Pizza", EqualSplit{{ana, bob}}, one);
    auto twoPizzas = pizza;
    twoPizzas.quantity = 2;
    checkValue(check, pizza, Item("Pizza", EqualSplit{{ana, bob}}, one), twoPizzas, "items compare by quantity too");
    checkValue(check, FixedTipValue{one}, FixedTipValue{one}, FixedTipValue{two}, "fixed tips");
    checkValue(check, PercentageTipValue{Percentage{1'000}}, PercentageTipValue{Percentage{1'000}}, PercentageTipValue{Percentage{500}},
               "percentage tips");
    Tip const tip(TipDistribution::EQUAL, FixedTipValue{one});
    checkValue(check, tip, Tip(TipDistribution::EQUAL, FixedTipValue{one}), Tip(TipDistribution::PROPORTIONAL, FixedTipValue{one}), "tips");
    auto const rate = ExchangeRate::parse("0.2");
    checkValue(check, ConvertedCost{one, rate}, ConvertedCost{one, rate}, ConvertedCost{one, ExchangeRate::parse("0.3")}, "converted costs");
    checkValue(check, LocalCost{one}, LocalCost{one}, LocalCost{two}, "local costs");
    checkValue(check, ItemisedExpenseSplit{{pizza}}, ItemisedExpenseSplit{{pizza}}, ItemisedExpenseSplit{{twoPizzas}}, "itemised splits");
    checkValue(check, WholeExpenseSplit{EqualSplit{{ana}}}, WholeExpenseSplit{EqualSplit{{ana}}}, WholeExpenseSplit{EqualSplit{{bob}}},
               "whole splits");
    Expense const dinner(LocalCost{two}, day, "Dinner", EqualSplit{{ana}}, WholeExpenseSplit{EqualSplit{{ana, bob}}});
    auto described = dinner;
    described.description = "At the corner";
    checkValue(check, dinner, Expense(LocalCost{two}, day, "Dinner", EqualSplit{{ana}}, WholeExpenseSplit{EqualSplit{{ana, bob}}}), described,
               "expenses compare by description too");
    Transfer const repayment(LocalCost{one}, day, bob, ana);
    auto noted = repayment;
    noted.note = "Cash";
    checkValue(check, repayment, Transfer(LocalCost{one}, day, bob, ana), noted, "transfers compare by note too");
    auto const iban = Iban::parse("RO49AAAA1B31007593840000");
    checkValue(check, IbanAccountDetails{iban}, IbanAccountDetails{iban}, IbanAccountDetails{Iban::parse("GB82WEST12345698765432")}, "IBAN details");
    auto const username = Username::parse("ana.pays");
    checkValue(check, UsernameAccountDetails{username}, UsernameAccountDetails{username}, UsernameAccountDetails{Username::parse("bob.pays")},
               "username details");
    checkValue(check, Account(IbanAccountDetails{iban}, "Bank"), Account(IbanAccountDetails{iban}, "Bank"),
               Account(UsernameAccountDetails{username}, "App"), "accounts");
    Member const member("Ana");
    auto withLastName = member;
    withLastName.lastName = "Pop";
    checkValue(check, member, Member("Ana"), withLastName, "members compare by last name too");
    checkValue(check, Category("Food"), Category("Food"), Category("Lodging"), "categories");
    checkValue(check, GetsBackBalance{one}, GetsBackBalance{one}, GetsBackBalance{two}, "balances owed to a member");
    checkValue(check, OwesBalance{one}, OwesBalance{one}, OwesBalance{two}, "balances a member owes");
    check.that(SettledBalance{} == SettledBalance{} && !(SettledBalance{} != SettledBalance{}), "settled balances are all equal");
    MemberSummary const summary(GetsBackBalance{one}, ana, one, two, one, one, one);
    checkValue(check, summary, MemberSummary(GetsBackBalance{one}, ana, one, two, one, one, one),
               MemberSummary(OwesBalance{one}, ana, one, two, one, one, one), "member summaries compare by balance too");
    CategoryTotal const total(food, one);
    checkValue(check, total, CategoryTotal(food, one), CategoryTotal(food, two), "category totals");
    checkValue(check, MemberCategoryTotals(ana, {total}), MemberCategoryTotals(ana, {total}), MemberCategoryTotals(ana, {}),
               "member category totals");
    checkValue(check, Summary({total}, {}, {summary}, two), Summary({total}, {}, {summary}, two), Summary({}, {}, {summary}, two), "summaries");
    MemberShare const share(ana, two, one);
    checkValue(check, share, MemberShare(ana, two, one), MemberShare(ana, one, one), "member shares");
    checkValue(check, ExpenseShares({share}, one, two), ExpenseShares({share}, one, two), ExpenseShares({share}, two, two), "expense shares");
    Payment const payment(one, bob, ana);
    checkValue(check, payment, Payment(one, bob, ana), Payment(one, ana, bob), "payments compare by direction");
    checkValue(check, Settlement({payment}, SettlementStrategy::GREEDY), Settlement({payment}, SettlementStrategy::GREEDY),
               Settlement({payment}, SettlementStrategy::DIRECT), "settlements compare by strategy too");
    checkValue(check, CurrencyRate(euro, rate), CurrencyRate(euro, rate), CurrencyRate(Currency::of("RON"), rate), "currency rates");
}

// --- Entries through the group -----------------------------------------------------------------------------------------

/// An expense that came back from the group otherwise than it went in would change what an app shows after a save.
void anExpenseReadsBackAsItWasAdded(Check &check) {
    Flat flat;
    auto const lei = Currency::of("RON");
    Expense lunch(ConvertedCost{Money(lei, 5'000), ExchangeRate::parse("0.2")}, flat.day, "Lunch",
                  PercentagesSplit{{MemberPercentage(flat.ana, Percentage{5'000}), MemberPercentage(flat.bob, Percentage{5'000})}},
                  WholeExpenseSplit{PartsSplit{{MemberParts(flat.ana, Parts{100}), MemberParts(flat.bob, Parts{50})}}});
    lunch.category = flat.food;
    lunch.description = "By the sea";
    lunch.time = TimeOfDay::of(13, 15);
    lunch.tip = Tip(TipDistribution::PROPORTIONAL, PercentageTipValue{Percentage{1'000}});
    auto const lunchId = flat.group.addExpense(lunch);
    lunch.id = lunchId;
    check.that(flat.group.expense(lunchId) == lunch, "a converted expense with every detail reads back unchanged");
    Item pizza("Pizza", EqualSplit{{flat.ana, flat.bob}}, flat.euros(1'200));
    pizza.quantity = 2;
    Expense receipt(LocalCost{flat.euros(3'000)}, flat.day, "Receipt", AmountsSplit{{MemberAmount(flat.euros(3'300), flat.bob)}},
                    ItemisedExpenseSplit{{pizza, Item("Wine", PercentagesSplit{{MemberPercentage(flat.ana, Percentage{10'000})}}, flat.euros(600))}});
    receipt.tip = Tip(TipDistribution::EQUAL, FixedTipValue{flat.euros(300)});
    receipt.id = flat.group.addExpense(receipt);
    check.that(flat.group.expense(receipt.id.value()) == receipt, "an itemised receipt with a fixed tip reads back unchanged");
    auto const expenses = flat.group.expenses();
    check.that(3 == expenses.size(), "the group holds the dinner, the lunch, and the receipt");
    check.that(!flat.group.expense(ExpenseId("0199a000-0000-7000-8000-00000000ffff")).has_value(), "an absent expense is none");
}

/// A transfer or a member that came back otherwise than it went in would move money to the wrong account after a save.
void aTransferAndAMemberReadBackAsTheyWereAdded(Check &check) {
    Flat flat;
    Transfer cash(ConvertedCost{Money(Currency::of("RON"), 1'000), ExchangeRate::parse("0.2")}, flat.day, flat.ana, flat.bob);
    cash.note = "Cash";
    cash.time = TimeOfDay::of(9, 5);
    cash.id = flat.group.addTransfer(cash);
    check.that(flat.group.transfer(cash.id.value()) == cash, "a converted transfer with a note and a time reads back unchanged");
    check.that(flat.group.transferAmount(cash.id.value()) == flat.euros(200), "the transfer is 2.00 EUR in the group currency");
    check.that(2 == flat.group.transfers().size(), "the group holds both transfers");
    check.that(!flat.group.transferAmount(TransferId("0199a000-0000-7000-8000-00000000ffff")).has_value(), "an absent transfer has no amount");
    Member carla("Carla");
    carla.accounts = {
        Account(IbanAccountDetails{Iban::parse("RO49AAAA1B31007593840000")}, "Bank"),
        Account(UsernameAccountDetails{Username::parse("carla.pays")}, "App"),
    };
    carla.email = Email::parse("carla@example.com");
    carla.lastName = "Pop";
    carla.phone = PhoneNumber::parse("+40712345678");
    carla.id = flat.group.addMember(carla);
    check.that(flat.group.member(carla.id.value()) == carla, "a member with every detail reads back unchanged");
    check.that(3 == flat.group.members().size(), "the group holds three members");
    check.that(flat.group.category(flat.food) == Category("Food") || "Food" == flat.group.category(flat.food).value().name,
               "the category reads back");
    check.that(2 == flat.group.categories().size(), "the built-in category and the food");
}

/// An edit that lost an entry's identifier, or returned the entry as it became, would leave an app's undo pointing nowhere.
void replacingAnEntryKeepsItsIdentifierAndReturnsItAsItWas(Check &check) {
    Flat flat;
    auto const dinner = flat.group.expense(flat.dinner).value();
    auto lunch = dinner;
    lunch.name = "Lunch";
    check.that(flat.group.replaceExpense(lunch) == dinner, "replacing an expense returns it as it was");
    check.that(flat.group.expense(flat.dinner) == lunch, "the expense is replaced under its identifier");
    auto const repayment = flat.group.transfer(flat.repayment).value();
    auto noted = repayment;
    noted.note = "Back";
    check.that(flat.group.replaceTransfer(noted) == repayment && flat.group.transfer(flat.repayment) == noted, "a transfer");
    auto const ana = flat.group.member(flat.ana).value();
    auto renamed = ana;
    renamed.firstName = "Ana Maria";
    check.that(flat.group.replaceMember(renamed) == ana && flat.group.member(flat.ana) == renamed, "a member");
    auto const food = flat.group.category(flat.food).value();
    auto meals = food;
    meals.name = "Meals";
    check.that(flat.group.replaceCategory(meals) == food && flat.group.category(flat.food) == meals, "a category");
    auto const missing = thrown([&] { return flat.group.replaceExpense(flat.expense("Snack", flat.ana, {flat.ana})); });
    check.that(isFailure(missing, ErrorKind::IDENTIFIER_MISSING, std::nullopt), "an entry to replace names its identifier");
    auto const given = thrown([&] { return flat.group.addExpense(lunch); });
    check.that(isFailure(given, ErrorKind::IDENTIFIER_GIVEN, std::nullopt), "an entry to add has none");
}

/// A removal that returned something else, or left the entry behind, would let an app show a deleted entry.
void removingAnEntryReturnsItAndForgetsIt(Check &check) {
    Flat flat;
    auto const dinner = flat.group.expense(flat.dinner);
    check.that(flat.group.removeExpense(flat.dinner) == dinner && !flat.group.expense(flat.dinner).has_value(), "an expense");
    auto const repayment = flat.group.transfer(flat.repayment);
    check.that(flat.group.removeTransfer(flat.repayment) == repayment && !flat.group.transfer(flat.repayment).has_value(), "a transfer");
    auto const food = flat.group.category(flat.food);
    check.that(flat.group.removeCategory(flat.food) == food && !flat.group.category(flat.food).has_value(), "a category");
    auto const bob = flat.group.member(flat.bob);
    check.that(flat.group.removeMember(flat.bob) == bob && !flat.group.member(flat.bob).has_value(), "a member no entry names");
}

/// A member removed while entries name them would leave shares nobody owes.
void aMemberInUseIsNotRemoved(Check &check) {
    Flat flat;
    auto const refused = thrown([&] { return flat.group.removeMember(flat.ana); });
    check.that(isFailure(refused, ErrorKind::GROUP, Failure::GROUP_MEMBER_IN_USE), "Ana paid the dinner and received the repayment");
    check.that(refused.has_value() && refused.value().member() == flat.ana && refused.value().expenses() == std::vector<ExpenseId>{flat.dinner} &&
                   refused.value().transfers() == std::vector<TransferId>{flat.repayment},
               "the error names the member and every entry that names them");
}

/// An archived member who could join a new expense would owe money for a trip they left.
void anArchivedMemberJoinsNoNewExpense(Check &check) {
    Flat flat;
    flat.group.archiveMember(flat.bob);
    check.that(flat.group.isArchived(flat.bob) && !flat.group.isArchived(flat.ana), "Bob is archived and Ana is not");
    auto const refused = thrown([&] { return flat.group.addExpense(flat.expense("Snack", flat.ana, {flat.ana, flat.bob})); });
    check.that(isFailure(refused, ErrorKind::GROUP, Failure::GROUP_ARCHIVED_MEMBER) && refused.value().member() == flat.bob, "the error names Bob");
    flat.group.unarchiveMember(flat.bob);
    check.that(!flat.group.isArchived(flat.bob), "Bob is back");
    auto const unknown = thrown([&] { flat.group.archiveMember(MemberId("0199a000-0000-7000-8000-00000000ffff")); });
    check.that(isFailure(unknown, ErrorKind::GROUP, Failure::GROUP_UNKNOWN_MEMBER) && unknown.value().member().has_value(), "an unknown member");
}

/// A helper that read an entry otherwise than the library would show an app a total the group does not hold.
void theHelpersReadAnEntryAsTheLibraryDoes(Check &check) {
    Flat flat;
    Split const split = EqualSplit{{flat.bob, flat.ana}};
    check.that(participants(split) == std::vector<MemberId>{flat.bob, flat.ana}, "a split's participants, in the order it lists them");
    auto dinner = flat.expense("Dinner", flat.ana, {flat.bob});
    dinner.tip = Tip(TipDistribution::EQUAL, PercentageTipValue{Percentage{1'000}});
    check.that(participants(dinner) == std::vector<MemberId>{flat.ana, flat.bob},
               "the payer and the participants, in the order of their identifiers");
    check.that(tipAmount(dinner) == flat.euros(300) && total(dinner) == flat.euros(3'300), "a tip of 10 % on 30.00 EUR");
    check.that("gets back 12.00 EUR" == toString(Balance{GetsBackBalance{flat.euros(1'200)}}), "a balance owed to a member");
    check.that("owes 12.00 EUR" == toString(Balance{OwesBalance{flat.euros(1'200)}}), "a balance a member owes");
    check.that("settled" == toString(Balance{SettledBalance{}}), "a settled balance");
    auto const unknown = thrown([] { return participants(Split{EqualSplit{{MemberId("not an identifier")}}}); });
    check.that(isFailure(unknown, ErrorKind::ID, Failure::ID_INVALID_ID), "an identifier that is not one");
}

// --- The group ---------------------------------------------------------------------------------------------------------

/// A group whose details did not change as asked would show an app a name or a language the user replaced.
void aGroupHoldsItsDetailsAndChangesThem(Check &check) {
    Flat flat;
    check.that(!flat.group.id().value().empty(), "the group has an identifier");
    check.that("Flat" == flat.group.name() && flat.group.currency() == flat.euro, "the name and the currency");
    check.that("Rent" == flat.group.description() && flat.group.language() == Language::parse("pt-PT"), "the options it started with");
    flat.group.setName("Home");
    flat.group.setDescription("Rent and bills");
    flat.group.setLanguage(Language::parse("ro-RO"));
    check.that("Home" == flat.group.name() && "Rent and bills" == flat.group.description() && flat.group.language() == Language::parse("ro-RO"),
               "the details as set");
    flat.group.clearDescription();
    flat.group.clearLanguage();
    check.that(!flat.group.description().has_value() && !flat.group.language().has_value(), "the details cleared");
    check.that(isFailure(thrown([&] { flat.group.setName(" "); }), ErrorKind::GROUP, Failure::GROUP_EMPTY_NAME), "a blank name");
    auto const trip = Group::create(flat.euro, "Trip");
    check.that(!trip.description().has_value() && !trip.language().has_value(), "a group made without options has neither");
    check.that(isFailure(thrown([&] { return Group::create(flat.euro, " "); }), ErrorKind::GROUP, Failure::GROUP_EMPTY_NAME), "a group needs a name");
}

/// Totals that did not add up would tell a member they owe what they do not.
void theSummaryAndTheSharesAddUp(Check &check) {
    Flat flat;
    auto const summary = flat.group.summary();
    check.that(summary.totalSpent == flat.euros(3'000), "the dinner is all that was spent");
    check.that(summary.categoryTotals == std::vector<CategoryTotal>{CategoryTotal(flat.food, flat.euros(3'000))}, "all of it on food");
    auto const ana =
        std::find_if(summary.members.begin(), summary.members.end(), [&](MemberSummary const &member) { return member.member == flat.ana; });
    check.that(ana != summary.members.end() && ana->paid == flat.euros(3'000) && ana->share == flat.euros(1'500) &&
                   ana->received == flat.euros(1'000) && ana->sent == flat.euros(0) && ana->net == flat.euros(500) &&
                   ana->balance == Balance{GetsBackBalance{flat.euros(500)}},
               "Ana paid 30.00, consumed 15.00, and got 10.00 back, so she is owed 5.00");
    check.that(2 == summary.memberCategoryTotals.size(), "both members have a share of the food");
    auto const shares = flat.group.expenseShares(flat.dinner);
    check.that(shares.has_value() && shares.value().total == flat.euros(3'000) && shares.value().tip == flat.euros(0) &&
                   2 == shares.value().members.size(),
               "the dinner's shares");
}

/// A strategy that settled otherwise than the library would ask a member to pay twice.
void eachStrategySettlesEveryBalance(Check &check) {
    Flat flat;
    for (auto const strategy : {SettlementStrategy::DIRECT, SettlementStrategy::GREEDY, SettlementStrategy::MINIMUM_TRANSFERS}) {
        auto const settlement = flat.group.settlement(strategy);
        check.that(settlement.strategy == strategy, "the plan names its strategy");
        check.that(settlement.payments == std::vector<Payment>{Payment(flat.euros(500), flat.bob, flat.ana)}, "Bob pays Ana 5.00");
    }
    static_cast<void>(flat.group.addTransfer(Transfer(LocalCost{flat.euros(500)}, flat.day, flat.bob, flat.ana)));
    auto const members = flat.group.summary().members;
    check.that(std::all_of(members.begin(), members.end(), [](MemberSummary const &member) { return member.balance == Balance{SettledBalance{}}; }),
               "once Bob pays, every balance is settled");
}

/// A change of currency that left an entry unconverted, or took a rate it was not given, would mix currencies in the totals.
void aChangeOfCurrencyConvertsEveryEntry(Check &check) {
    Flat flat;
    auto const lei = Currency::of("RON");
    auto const missing = thrown([&] { flat.group.changeCurrency(lei, {}); });
    check.that(isFailure(missing, ErrorKind::GROUP, Failure::GROUP_MISSING_EXCHANGE_RATE) && "EUR" == missing.value().currency(),
               "the euro needs a rate");
    auto const rate = CurrencyRate(flat.euro, ExchangeRate::parse("5"));
    auto const twice = thrown([&] { flat.group.changeCurrency(lei, {rate, rate}); });
    check.that(isFailure(twice, ErrorKind::GROUP, Failure::GROUP_DUPLICATE_EXCHANGE_RATE) && "EUR" == twice.value().currency(),
               "one rate per currency");
    auto converted = flat.expense("Taxi", flat.ana, {flat.ana});
    converted.cost = ConvertedCost{flat.euros(1'000), ExchangeRate::parse("1")};
    auto const unexpected = thrown([&] { return flat.group.addExpense(converted); });
    check.that(isFailure(unexpected, ErrorKind::GROUP, Failure::GROUP_UNEXPECTED_EXCHANGE_RATE), "an amount in the group currency has no rate");
    auto local = converted;
    local.cost = LocalCost{Money(lei, 1'000)};
    auto const unconverted = thrown([&] { return flat.group.addExpense(local); });
    check.that(isFailure(unconverted, ErrorKind::GROUP, Failure::GROUP_MISSING_EXCHANGE_RATE) && "RON" == unconverted.value().currency(),
               "an amount in another currency needs a rate");
    flat.group.changeCurrency(lei, {rate});
    check.that(flat.group.currency() == lei && flat.group.summary().totalSpent == Money(lei, 15'000), "30.00 EUR at 5 is 150.00 RON");
}

/// A saved group that loaded otherwise than it was saved would lose entries between two runs of an app.
void aSavedGroupLoadsAsItWasSaved(Check &check) {
    Flat const flat;
    auto const json = flat.group.save();
    check.that(Group::load(json).save() == json, "the loaded group saves the same text");
    auto const broken = thrown([] { return Group::load("{"); });
    check.that(isFailure(broken, ErrorKind::INVALID_JSON, std::nullopt) && 1 == broken.value().line() && 1 == broken.value().column(),
               "where the text stops being JSON");
    auto tampered = json;
    auto const format = tampered.find("\"format\": 2");
    tampered.replace(format, std::string_view("\"format\": 2").size(), "\"format\": 26");
    auto const newer = thrown([&] { return Group::load(tampered); });
    check.that(isFailure(newer, ErrorKind::GROUP, Failure::GROUP_UNSUPPORTED_FORMAT) && 26 == newer.value().format(),
               "a format this release cannot read");
}

/// An error that lost what it holds would leave an app unable to say which entry, amount, or category the library refused.
void everyErrorHoldsWhatItsFailureHolds(Check &check) {
    Flat flat;
    auto uneven = flat.expense("Uneven", flat.ana, {});
    uneven.split = WholeExpenseSplit{AmountsSplit{{MemberAmount(flat.euros(1'000), flat.ana)}}};
    auto const amounts = thrown([&] { return flat.group.addExpense(uneven); });
    check.that(isFailure(amounts, ErrorKind::EXPENSE, Failure::EXPENSE_AMOUNTS_DO_NOT_ADD_UP) && amounts.value().actual() == flat.euros(1'000) &&
                   amounts.value().expectedAmount() == flat.euros(3'000),
               "the amounts and what they should add up to");
    auto foreign = uneven;
    foreign.split = WholeExpenseSplit{AmountsSplit{{MemberAmount(Money(Currency::of("RON"), 3'000), flat.ana)}}};
    auto const currencies = thrown([&] { return flat.group.addExpense(foreign); });
    check.that(isFailure(currencies, ErrorKind::EXPENSE, Failure::EXPENSE_CURRENCY_MISMATCH) && "EUR" == currencies.value().expectedCurrency() &&
                   "RON" == currencies.value().foundCurrency(),
               "the expense's currency and the share's");
    auto halves = uneven;
    halves.split = WholeExpenseSplit{PercentagesSplit{{MemberPercentage(flat.ana, Percentage{5'000})}}};
    auto const percentages = thrown([&] { return flat.group.addExpense(halves); });
    check.that(isFailure(percentages, ErrorKind::EXPENSE, Failure::EXPENSE_PERCENTAGES_DO_NOT_ADD_UP) &&
                   5'000 == percentages.value().totalBasisPoints(),
               "what the percentages add up to");
    auto const duplicate = thrown([&] { return flat.group.addCategory(Category("food")); });
    check.that(isFailure(duplicate, ErrorKind::GROUP, Failure::GROUP_DUPLICATE_CATEGORY_NAME) && duplicate.value().existing() == flat.food,
               "the category that has the name");
    CategoryId const absent("0199a000-0000-7000-8000-00000000ffff");
    auto uncategorised = flat.expense("Taxi", flat.ana, {flat.ana});
    uncategorised.category = absent;
    auto const category = thrown([&] { return flat.group.addExpense(uncategorised); });
    check.that(isFailure(category, ErrorKind::GROUP, Failure::GROUP_UNKNOWN_CATEGORY) && category.value().category() == absent,
               "the unknown category");
    auto const expense = thrown([&] { return flat.group.removeExpense(ExpenseId("0199a000-0000-7000-8000-00000000ffff")); });
    check.that(isFailure(expense, ErrorKind::GROUP, Failure::GROUP_UNKNOWN_EXPENSE) && expense.value().expense().has_value(), "the unknown expense");
    auto const transfer = thrown([&] { return flat.group.removeTransfer(TransferId("0199a000-0000-7000-8000-00000000ffff")); });
    check.that(isFailure(transfer, ErrorKind::GROUP, Failure::GROUP_UNKNOWN_TRANSFER) && transfer.value().transfer().has_value(),
               "the unknown transfer");
    auto const identifier = thrown([&] { return flat.group.member(MemberId("not an identifier")); });
    check.that(isFailure(identifier, ErrorKind::ID, Failure::ID_INVALID_ID), "text that is no identifier");
    check.that(identifier.has_value() && !identifier.value().balances().has_value() && !identifier.value().actual().has_value() &&
                   !identifier.value().reason().has_value(),
               "an error holds only what its failure holds");
}

// --- Asynchronous calls, the read handle, and the cancellation -----------------------------------------------------------

/// An asynchronous call that returned otherwise than its synchronous form would give an app two answers to one question.
void everyAsynchronousCallReturnsWhatItsSynchronousFormReturns(Check &check) {
    Flat flat;
    auto &group = flat.group;
    check.that(group.idAsync().get() == group.id() && group.nameAsync().get() == group.name(), "the identifier and the name");
    check.that(group.currencyAsync().get() == group.currency() && group.descriptionAsync().get() == group.description(),
               "the currency and the description");
    check.that(group.languageAsync().get() == group.language() && group.saveAsync().get() == group.save(), "the language and the saved group");
    check.that(group.categoriesAsync().get() == group.categories() && group.categoryAsync(flat.food).get() == group.category(flat.food),
               "categories");
    check.that(group.expensesAsync().get() == group.expenses() && group.expenseAsync(flat.dinner).get() == group.expense(flat.dinner), "expenses");
    check.that(group.expenseSharesAsync(flat.dinner).get() == group.expenseShares(flat.dinner) && group.summaryAsync().get() == group.summary(),
               "totals");
    check.that(group.membersAsync().get() == group.members() && group.memberAsync(flat.ana).get() == group.member(flat.ana), "members");
    check.that(group.isArchivedAsync(flat.ana).get() == group.isArchived(flat.ana), "whether a member is archived");
    check.that(group.settlementAsync(SettlementStrategy::GREEDY).get() == group.settlement(SettlementStrategy::GREEDY), "a settlement");
    check.that(group.transfersAsync().get() == group.transfers() && group.transferAsync(flat.repayment).get() == group.transfer(flat.repayment),
               "transfers");
    check.that(group.transferAmountAsync(flat.repayment).get() == group.transferAmount(flat.repayment), "a transfer's amount");
    auto const lunch = group.addExpenseAsync(flat.expense("Lunch", flat.bob, {flat.ana})).get();
    auto renamed = group.expense(lunch).value();
    renamed.name = "Brunch";
    check.that("Lunch" == group.replaceExpenseAsync(renamed).get().name && group.removeExpenseAsync(lunch).get() == renamed, "an expense's life");
    auto const cash = group.addTransferAsync(Transfer(LocalCost{flat.euros(100)}, flat.day, flat.ana, flat.bob)).get();
    auto noted = group.transfer(cash).value();
    noted.note = "Coins";
    check.that(!group.replaceTransferAsync(noted).get().note.has_value() && group.removeTransferAsync(cash).get() == noted, "a transfer's life");
    auto const carla = group.addMemberAsync(Member("Carla")).get();
    auto renamedCarla = group.member(carla).value();
    renamedCarla.firstName = "Carla Maria";
    check.that("Carla" == group.replaceMemberAsync(renamedCarla).get().firstName, "a member replaced");
    group.archiveMemberAsync(carla).get();
    check.that(group.isArchived(carla), "a member archived");
    group.unarchiveMemberAsync(carla).get();
    check.that(!group.isArchived(carla) && group.removeMemberAsync(carla).get() == renamedCarla, "a member unarchived and removed");
    auto const drinks = group.addCategoryAsync(Category("Drinks")).get();
    auto renamedDrinks = group.category(drinks).value();
    renamedDrinks.name = "Coffee";
    check.that("Drinks" == group.replaceCategoryAsync(renamedDrinks).get().name && group.removeCategoryAsync(drinks).get() == renamedDrinks,
               "a category");
    group.setNameAsync("Home").get();
    group.setDescriptionAsync("Bills").get();
    group.setLanguageAsync(Language::parse("ro-RO")).get();
    check.that("Home" == group.name() && "Bills" == group.description() && group.language() == Language::parse("ro-RO"), "the details set");
    group.clearDescriptionAsync().get();
    group.clearLanguageAsync().get();
    check.that(!group.description().has_value() && !group.language().has_value(), "the details cleared");
    group.changeCurrencyAsync(Currency::of("RON"), {CurrencyRate(flat.euro, ExchangeRate::parse("5"))}).get();
    check.that(group.currency() == Currency::of("RON"), "the currency changed");
    auto const loaded = Group::loadAsync(group.save()).get();
    check.that(loaded.save() == group.save(), "a group loaded on a thread of its own");
}

/// An asynchronous call that swallowed its failure would leave an app waiting for a change that never happened.
void anAsynchronousCallReportsWhatItsSynchronousFormThrows(Check &check) {
    Flat flat;
    MemberId const nobody("0199a000-0000-7000-8000-00000000ffff");
    auto removing = flat.group.removeMemberAsync(nobody);
    check.that(isFailure(thrown([&] { return removing.get(); }), ErrorKind::GROUP, Failure::GROUP_UNKNOWN_MEMBER), "a call that returns a value");
    auto archiving = flat.group.archiveMemberAsync(nobody);
    check.that(isFailure(thrown([&] { archiving.get(); }), ErrorKind::GROUP, Failure::GROUP_UNKNOWN_MEMBER), "a call that returns nothing");
    auto loading = Group::loadAsync("{");
    check.that(isFailure(thrown([&] { return loading.get(); }), ErrorKind::INVALID_JSON, std::nullopt), "a load");
}

/// A read handle that let a change in, or read after its release, would show a report two states of one group.
void aReaderReadsTheGroupUntilItIsReleased(Check &check) {
    Flat const flat;
    auto reader = flat.group.read();
    check.that(reader.id() == flat.group.id() && "Flat" == reader.name() && reader.currency() == flat.euro, "the details");
    check.that("Rent" == reader.description() && reader.language() == Language::parse("pt-PT") && reader.save() == flat.group.save(), "the options");
    check.that(reader.categories() == flat.group.categories() && reader.category(flat.food) == flat.group.category(flat.food), "the categories");
    check.that(reader.expenses() == flat.group.expenses() && reader.expense(flat.dinner) == flat.group.expense(flat.dinner), "the expenses");
    check.that(reader.expenseShares(flat.dinner) == flat.group.expenseShares(flat.dinner) && reader.summary() == flat.group.summary(), "the totals");
    check.that(reader.members() == flat.group.members() && reader.member(flat.ana) == flat.group.member(flat.ana), "the members");
    check.that(!reader.isArchived(flat.ana), "whether a member is archived");
    check.that(reader.settlement(SettlementStrategy::DIRECT) == flat.group.settlement(SettlementStrategy::DIRECT), "a settlement");
    check.that(reader.transfers() == flat.group.transfers() && reader.transfer(flat.repayment) == flat.group.transfer(flat.repayment),
               "the transfers");
    check.that(reader.transferAmount(flat.repayment) == flat.euros(1'000), "a transfer's amount");
    reader.release();
    auto const released = thrown([&] { return reader.name(); });
    check.that(isFailure(released, ErrorKind::RELEASED, std::nullopt), "a read after the release");
    check.that(released.has_value() && "the read handle was released" == std::string_view(released.value().what()), "the message");
    auto const asynchronous = flat.group.readAsync().get();
    check.that("Flat" == asynchronous.name(), "a read handle from an asynchronous call");
}

/// A cancelled change that still ran would change the group after the app gave the change up; one that kept waiting would hold
/// the worker thread for a lock it no longer needs.
void aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas(Check &check) {
    Flat flat;
    auto reader = flat.group.read();
    Cancellation cancellation;
    auto changing = flat.group.changeCurrencyAsync(Currency::of("RON"), {CurrencyRate(flat.euro, ExchangeRate::parse("5"))}, &cancellation);
    check.that(!cancellation.isCancelled(), "a cancellation starts uncancelled");
    cancellation.cancel();
    check.that(cancellation.isCancelled(), "and is cancelled once cancelled");
    check.that(isFailure(thrown([&] { changing.get(); }), ErrorKind::CANCELLED, std::nullopt), "the change reports its cancellation");
    reader.release();
    check.that(flat.group.currency() == flat.euro, "the group keeps its currency");
    auto const late = thrown([&] { return flat.group.nameAsync(&cancellation).get(); });
    check.that(isFailure(late, ErrorKind::CANCELLED, std::nullopt), "a call given a cancelled cancellation does not run");
}

/// A kind of call whose outcome lost its failure would leave the app's future waiting, or hand it a value the library never
/// gave; a call given a cancelled cancellation reports the cancellation, whatever it returns.
void everyKindOfAsynchronousCallReportsItsCancellation(Check &check) {
    Flat const flat;
    Cancellation cancelled;
    cancelled.cancel();
    auto const reportsIt = [](std::optional<Error> const &error) { return isFailure(error, ErrorKind::CANCELLED, std::nullopt); };
    check.that(reportsIt(thrown([&] { return flat.group.categoriesAsync(&cancelled).get(); })), "the categories");
    check.that(reportsIt(thrown([&] { return flat.group.categoryAsync(flat.food, &cancelled).get(); })), "a category");
    check.that(reportsIt(thrown([&] { return flat.group.expensesAsync(&cancelled).get(); })), "the expenses");
    check.that(reportsIt(thrown([&] { return flat.group.expenseAsync(flat.dinner, &cancelled).get(); })), "an expense");
    check.that(reportsIt(thrown([&] { return flat.group.expenseSharesAsync(flat.dinner, &cancelled).get(); })), "an expense's shares");
    check.that(reportsIt(thrown([&] { return flat.group.isArchivedAsync(flat.ana, &cancelled).get(); })), "whether a member is archived");
    check.that(reportsIt(thrown([&] { return flat.group.membersAsync(&cancelled).get(); })), "the members");
    check.that(reportsIt(thrown([&] { return flat.group.readAsync(&cancelled).get(); })), "a read handle");
    check.that(reportsIt(thrown([&] { return flat.group.settlementAsync(SettlementStrategy::DIRECT, &cancelled).get(); })), "a settlement");
    check.that(reportsIt(thrown([&] { return flat.group.summaryAsync(&cancelled).get(); })), "the summary");
    check.that(reportsIt(thrown([&] { return flat.group.transferAmountAsync(flat.repayment, &cancelled).get(); })), "a transfer's amount");
    check.that(reportsIt(thrown([&] { return flat.group.transfersAsync(&cancelled).get(); })), "the transfers");
    check.that(reportsIt(thrown([&] { return flat.group.transferAsync(flat.repayment, &cancelled).get(); })), "a transfer");
}

/// A test: its name, the claim it makes, and its body.
struct Test final {
    std::string_view name;
    void (*body)(Check &check);
};

constexpr std::array TESTS = {
    Test{"aCurrencyHoldsWhatTheLibraryKnowsOfIt", aCurrencyHoldsWhatTheLibraryKnowsOfIt},
    Test{"everyCurrencyTheLibraryListsIsListedByCode", everyCurrencyTheLibraryListsIsListedByCode},
    Test{"aCodeTheLibraryDoesNotListIsRefused", aCodeTheLibraryDoesNotListIsRefused},
    Test{"textThatIsNotUtf8IsRefusedBeforeTheLibrarySeesIt", textThatIsNotUtf8IsRefusedBeforeTheLibrarySeesIt},
    Test{"anAmountParsesAndFormatsAsTheLibraryWritesIt", anAmountParsesAndFormatsAsTheLibraryWritesIt},
    Test{"amountsInOneCurrencyAddAndSubtract", amountsInOneCurrencyAddAndSubtract},
    Test{"aRateParsesAndConvertsAsTheLibraryDoes", aRateParsesAndConvertsAsTheLibraryDoes},
    Test{"percentagesAndPartsParseAndFormatAsTheLibraryDoes", percentagesAndPartsParseAndFormatAsTheLibraryDoes},
    Test{"datesAndTimesAreCheckedWhenMade", datesAndTimesAreCheckedWhenMade},
    Test{"contactsAndAccountsAreCheckedAndMadeCanonical", contactsAndAccountsAreCheckedAndMadeCanonical},
    Test{"everyIdentifierComparesAndHashesByItsText", everyIdentifierComparesAndHashesByItsText},
    Test{"everyRecordAndAlternativeEqualsItsTwinAndDiffersFromAnother", everyRecordAndAlternativeEqualsItsTwinAndDiffersFromAnother},
    Test{"anExpenseReadsBackAsItWasAdded", anExpenseReadsBackAsItWasAdded},
    Test{"aTransferAndAMemberReadBackAsTheyWereAdded", aTransferAndAMemberReadBackAsTheyWereAdded},
    Test{"replacingAnEntryKeepsItsIdentifierAndReturnsItAsItWas", replacingAnEntryKeepsItsIdentifierAndReturnsItAsItWas},
    Test{"removingAnEntryReturnsItAndForgetsIt", removingAnEntryReturnsItAndForgetsIt},
    Test{"aMemberInUseIsNotRemoved", aMemberInUseIsNotRemoved},
    Test{"anArchivedMemberJoinsNoNewExpense", anArchivedMemberJoinsNoNewExpense},
    Test{"theHelpersReadAnEntryAsTheLibraryDoes", theHelpersReadAnEntryAsTheLibraryDoes},
    Test{"aGroupHoldsItsDetailsAndChangesThem", aGroupHoldsItsDetailsAndChangesThem},
    Test{"theSummaryAndTheSharesAddUp", theSummaryAndTheSharesAddUp},
    Test{"eachStrategySettlesEveryBalance", eachStrategySettlesEveryBalance},
    Test{"aChangeOfCurrencyConvertsEveryEntry", aChangeOfCurrencyConvertsEveryEntry},
    Test{"aSavedGroupLoadsAsItWasSaved", aSavedGroupLoadsAsItWasSaved},
    Test{"everyErrorHoldsWhatItsFailureHolds", everyErrorHoldsWhatItsFailureHolds},
    Test{"everyAsynchronousCallReturnsWhatItsSynchronousFormReturns", everyAsynchronousCallReturnsWhatItsSynchronousFormReturns},
    Test{"anAsynchronousCallReportsWhatItsSynchronousFormThrows", anAsynchronousCallReportsWhatItsSynchronousFormThrows},
    Test{"aReaderReadsTheGroupUntilItIsReleased", aReaderReadsTheGroupUntilItIsReleased},
    Test{"aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas", aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas},
    Test{"everyKindOfAsynchronousCallReportsItsCancellation", everyKindOfAsynchronousCallReportsItsCancellation},
};

/// Runs every test and prints each claim that failed.
///
/// @return How many tests failed
[[nodiscard]] std::size_t runTests() {
    std::size_t failed = 0;
    for (auto const &test : TESTS) {
        Check check;
        try {
            test.body(check);
        } catch (std::exception const &error) {
            check.that(false, std::string("an unexpected exception: ") + error.what());
        }
        for (auto const &claim : check.failed()) {
            std::cerr << test.name << ": " << claim << '\n';
        }
        failed += check.failed().empty() ? 0 : 1;
    }
    std::cout << TESTS.size() - failed << " of " << TESTS.size() << " tests passed\n";
    return failed;
}

} // namespace
} // namespace eranos

int main() { return 0 == eranos::runTests() ? EXIT_SUCCESS : EXIT_FAILURE; }
