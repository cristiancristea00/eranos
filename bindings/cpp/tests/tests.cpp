// The tests of the C++ layer: every value, record, and alternative through its conversions, every group call in both
// forms, the read handle, the cancellation, and every error the library reports with what it holds. Each test is a
// claim, named for what it states, whose comment says what breaks in an app if it regresses. The program prints every
// claim that fails and ends with a failure status when one does.

#include "eranos.hpp"
// The generated classes whose helpers the tests hold the layer's constants to.
#include "eranos/detail/EranosIdentifier.d.hpp"
#include "eranos/detail/EranosTipRule.d.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <expected>
#include <functional>
#include <future>
#include <optional>
#include <print>
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
    inline constexpr void that(bool const holds, std::string_view const claim) noexcept {
        if (!holds) {
            failures.emplace_back(claim);
        }
    }

    /// The claims that failed.
    ///
    /// @return The claims
    [[nodiscard]] inline constexpr std::vector<std::string> const &failed() const noexcept { return failures; }

  private:
    std::vector<std::string> failures;
};

/// The error @p result holds, or none when it holds a value.
template <typename Value> [[nodiscard]] std::optional<Error> refusal(std::expected<Value, Error> const &result) noexcept {
    if (result) {
        return std::nullopt;
    }
    return result.error();
}

/// Whether @p error is of @p kind with @p failure.
[[nodiscard]] bool isFailure(std::optional<Error> const &error, ErrorKind const kind, std::optional<Failure> const failure) noexcept {
    return error.has_value() && error->kind() == kind && error->failure() == failure;
}

/// Checks that @p value equals @p same and differs from @p other through both operators: a comparison that left a field out
/// would merge two values an app holds apart, and one whose two operators disagreed would answer a question twice.
template <typename Value>
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the value, its twin, and another, in the order the claim reads them.
inline constexpr void checkValue(Check &check, Value const &value, Value const &same, Value const &other, std::string_view const what) noexcept {
    check.that(value == same && !(value != same), what);
    check.that(value != other && !(value == other), what);
}

/// Checks an identifier type: equal to its twin with one hash, different from another, and ordered by the bytes of its text, as the
/// library orders the entries it lists and as a map or a set of them expects.
template <typename Identifier> void checkIdentifier(Check &check, std::string_view const what) noexcept {
    Identifier const first("0199a000-0000-7000-8000-000000000001");
    Identifier const twin(first.value());
    Identifier const second("0199a000-0000-7000-8000-000000000002");
    checkValue(check, first, twin, second, what);
    check.that(first < second && !(second < first), what);
    check.that(Identifier("ana") < Identifier("ana-1") && Identifier("Ana") != Identifier("ana"), what);
    check.that(Identifier("zoe") < Identifier("éva"), what);
    check.that(std::hash<Identifier>()(first) == std::hash<Identifier>()(twin), what);
    check.that("0199a000-0000-7000-8000-000000000001" == first.value(), what);
}

/// Text that is not an identifier, and the failure the library refuses it with.
struct RefusedText final {
    Failure failure;
    std::string text;
};

/// Text that breaks each rule of identifiers, with the failure of the rule.
[[nodiscard]] inline constexpr std::array<RefusedText, 4> refusedTexts() noexcept {
    return {
        RefusedText{.failure = Failure::ID_EMPTY, .text = ""},
        RefusedText{.failure = Failure::ID_TOO_LONG, .text = std::string(65, 'a')},
        RefusedText{.failure = Failure::ID_CONTROL_CHARACTER, .text = "a\tb"},
        RefusedText{.failure = Failure::ID_SURROUNDING_WHITESPACE, .text = " ana"},
    };
}

/// Checks the parse() of an identifier type: text the rules accept comes back as it is written, byte for byte and in its case, and text
/// they refuse is refused with the first rule it breaks.
template <typename Identifier> void checkParse(Check &check, std::string_view const what) noexcept {
    check.that(Identifier("Ana-1") == Identifier::parse("Ana-1"), what);
    check.that(Identifier("caf\u00e9 \u2615") == Identifier::parse("caf\u00e9 \u2615"), what);
    for (auto const &[failure, text] : refusedTexts()) {
        check.that(isFailure(refusal(Identifier::parse(text)), ErrorKind::ID, failure), what);
    }
    check.that(isFailure(refusal(Identifier::parse("\xff")), ErrorKind::INVALID_TEXT, std::nullopt), what);
}

/// Checks the generate() of an identifier type: two in a row differ, the second after the first in the order the library lists entries by, and
/// each is text that parse() accepts as it is.
template <typename Identifier> void checkGenerate(Check &check, std::string_view const what) noexcept {
    auto const first = Identifier::generate();
    auto const second = Identifier::generate();
    check.that(first != second && first < second, what);
    check.that(first == Identifier::parse(first.value()) && second == Identifier::parse(second.value()), what);
}

/// A group of two, Ana and Bob, with a category, an expense, and a transfer, and the identifiers the group gave them.
class Flat final {
  public:
    inline Flat() noexcept
        : euro(CurrencyCode::of("EUR").value()), day(CalendarDate::of(2026, 9, 23).value()),
          group(ExpenseGroup::create("Flat", euro, ExpenseGroupOptions{.description = "Rent", .language = Language::parse("pt-pt").value()}).value()),
          ana(group.addMember(Member("Ana")).value()), bob(group.addMember(Member("Bob")).value()), food(group.addCategory(Category("Food")).value()),
          dinner(group.addExpense(dinnerExpense()).value()), repayment(group.addTransfer(Transfer(bob, ana, LocalCost{euros(1'000)}, day)).value()) {}

    /// @p minorUnits cents.
    [[nodiscard]] inline Money euros(std::int64_t const minorUnits) const noexcept { return {minorUnits, euro}; }

    /// An expense of 30.00 EUR, called @p name, that @p payer paid and @p members share equally.
    [[nodiscard]] inline Expense expense(std::string name, MemberId const &payer, std::vector<MemberId> members) const noexcept {
        return {std::move(name), LocalCost{euros(3'000)}, EqualSplit{{payer}}, WholeExpenseSplit{EqualSplit{std::move(members)}}, day};
    }

    CurrencyCode euro;
    CalendarDate day;
    ExpenseGroup group;
    MemberId ana;
    MemberId bob;
    CategoryId food;
    ExpenseId dinner;
    TransferId repayment;

  private:
    /// The dinner Ana paid for the two of them, in the food category.
    [[nodiscard]] inline Expense dinnerExpense() const noexcept {
        auto dinnerEntry = expense("Dinner", ana, {ana, bob});
        dinnerEntry.category = food;
        return dinnerEntry;
    }
};

// --- Values -----------------------------------------------------------------------------------------------------------

/// A currency whose details came out wrong would format every amount in it wrongly.
void aCurrencyHoldsWhatTheLibraryKnowsOfIt(Check &check) noexcept {
    auto const euro = CurrencyCode::of("EUR").value();
    auto const yen = CurrencyCode::of("JPY").value();
    check.that("EUR" == euro.code() && 2 == euro.decimalPlaces() && "Euro" == euro.name() && 978 == euro.numericCode(), "the euro's details");
    check.that(0 == yen.decimalPlaces(), "the yen has no decimal places");
    checkValue(check, euro, CurrencyCode::of("EUR").value(), yen, "currencies compare by code");
    check.that(euro < yen && !(yen < euro), "currencies order by code");
}

/// A list that missed a currency or came out of order would hide one from the currency pickers of an app.
void everyCurrencyTheLibraryListsIsListedByCode(Check &check) noexcept {
    auto const all = CurrencyCode::all();
    check.that(std::ranges::is_sorted(all), "the currencies are in the order of their codes");
    check.that(std::ranges::contains(all, CurrencyCode::of("EUR").value()), "the euro is listed");
}

/// A code the library does not list, taken, would make an amount no call can format.
void aCodeTheLibraryDoesNotListIsRefused(Check &check) noexcept {
    auto const unknown = refusal(CurrencyCode::of("XYZ"));
    check.that(isFailure(unknown, ErrorKind::MONEY, Failure::MONEY_UNKNOWN_CURRENCY), "an unknown code is refused");
    check.that(unknown.has_value() && "unknown currency code" == unknown.value().message(), "the message names the failure, not the code");
}

/// Bytes that are not UTF-8, passed on, would reach the library as text it cannot hold.
void textThatIsNotUtf8IsRefusedBeforeTheLibrarySeesIt(Check &check) noexcept {
    auto const refused = refusal(CurrencyCode::of("\xff"));
    check.that(isFailure(refused, ErrorKind::INVALID_TEXT, std::nullopt), "the bytes are refused as text");
}

/// An amount that parsed or formatted otherwise than the library does would show one number in an app and store another.
void anAmountParsesAndFormatsAsTheLibraryWritesIt(Check &check) noexcept {
    auto const euro = CurrencyCode::of("EUR").value();
    auto const price = Money::parse("12.5", euro).value();
    check.that(price.currency == euro && 1'250 == price.minorUnits, "12.5 is 1250 cents");
    check.that("12.50 EUR" == price.toString(), "the amount formats with its currency");
    checkValue(check, price, Money(1'250, euro), Money(1'251, euro), "amounts compare by minor units and currency");
    check.that(isFailure(refusal(Money::parse("twelve", euro)), ErrorKind::MONEY, Failure::MONEY_INVALID_AMOUNT), "a word is no amount");
    auto const precise = refusal(Money::parse("1.234", euro));
    check.that(isFailure(precise, ErrorKind::MONEY, Failure::MONEY_TOO_MANY_DECIMALS), "the euro has two decimal places");
    check.that(precise.has_value() && 2 == precise.value().allowed() && "EUR" == precise.value().currency(),
               "the error holds the decimal places and the currency");
}

/// Arithmetic that mixed currencies or wrapped around would put a wrong total in a report.
void amountsInOneCurrencyAddAndSubtract(Check &check) noexcept {
    auto const euro = CurrencyCode::of("EUR").value();
    auto const lei = CurrencyCode::of("RON").value();
    check.that(Money(1'000, euro).plus(Money(250, euro)) == Money(1'250, euro), "10.00 EUR and 2.50 EUR make 12.50 EUR");
    check.that(Money(1'000, euro).minus(Money(250, euro)) == Money(750, euro), "10.00 EUR less 2.50 EUR is 7.50 EUR");
    auto const mixed = refusal(Money(1, euro).plus(Money(1, lei)));
    check.that(isFailure(mixed, ErrorKind::MONEY, Failure::MONEY_CURRENCY_MISMATCH), "amounts in two currencies do not add");
    check.that(mixed.has_value() && "EUR" == mixed.value().left() && "RON" == mixed.value().right(), "the error names both currencies");
    auto const largest = Money(INT64_MAX, euro);
    check.that(isFailure(refusal(largest.plus(Money(1, euro))), ErrorKind::MONEY, Failure::MONEY_OVERFLOW), "a sum out of range");
}

/// A rate that parsed or converted otherwise than the library does would change the share of every converted expense.
void aRateParsesAndConvertsAsTheLibraryDoes(Check &check) noexcept {
    auto const euro = CurrencyCode::of("EUR").value();
    auto const lei = CurrencyCode::of("RON").value();
    auto const rate = ExchangeRate::parse("4.970").value();
    check.that("4.97" == rate.toString(), "the rate is written without trailing zeros");
    checkValue(check, rate, ExchangeRate::parse("4.97").value(), ExchangeRate::parse("5").value(), "rates compare by value");
    check.that(ExchangeRate::parse("0.25").value().convert(Money(2, lei), euro) == Money(1, euro),
               "0.02 RON at 0.25 is 0.01 EUR, rounded half away from zero");
    check.that(isFailure(refusal(ExchangeRate::parse("0")), ErrorKind::MONEY, Failure::MONEY_INVALID_EXCHANGE_RATE), "a rate is positive");
    auto const overflow = refusal(ExchangeRate::parse("2").value().convert(Money(INT64_MAX, lei), euro));
    check.that(isFailure(overflow, ErrorKind::MONEY, Failure::MONEY_OVERFLOW), "a converted amount out of range");
}

/// A percentage or a number of parts read otherwise than the library reads it would divide a split otherwise than it shows.
void percentagesAndPartsParseAndFormatAsTheLibraryDoes(Check &check) noexcept {
    auto const percentage = Percentage::parse("12.5 %").value();
    check.that(1'250 == percentage.basisPoints && "12.5%" == percentage.toString(), "12.5 % is 1250 basis points");
    checkValue(check, percentage, Percentage{1'250}, Percentage{1'251}, "percentages compare by basis points");
    check.that(isFailure(refusal(Percentage::parse("half")), ErrorKind::MONEY, Failure::MONEY_INVALID_PERCENTAGE), "a word is no percentage");
    check.that(isFailure(refusal(Percentage{-1}.toString()), ErrorKind::MONEY, Failure::MONEY_INVALID_PERCENTAGE), "a negative percentage");
    auto const parts = Parts::parse("0.5").value();
    check.that(50 == parts.hundredths && "0.5" == parts.toString(), "half a part is 50 hundredths");
    checkValue(check, parts, Parts{50}, Parts{51}, "parts compare by hundredths");
    check.that(isFailure(refusal(Parts::parse("half")), ErrorKind::MONEY, Failure::MONEY_INVALID_PARTS), "a word is no number of parts");
    check.that(isFailure(refusal(Parts{-1}.toString()), ErrorKind::MONEY, Failure::MONEY_INVALID_PARTS), "negative parts");
}

/// A date or a time the calendar does not have, taken, would date an expense on a day that never was.
void datesAndTimesAreCheckedWhenMade(Check &check) noexcept {
    auto const date = CalendarDate::of(2026, 9, 23).value();
    check.that(2026 == date.year() && 9 == date.month() && 23 == date.day() && "2026-09-23" == date.toString(), "the date's fields and text");
    checkValue(check, date, CalendarDate::parse("2026-09-23").value(), CalendarDate::of(2026, 9, 24).value(), "dates compare by day");
    check.that(isFailure(refusal(CalendarDate::of(2026, 2, 30)), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_DATE), "30 February");
    check.that(isFailure(refusal(CalendarDate::parse("2026-02-30")), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_DATE), "parsed");
    auto const time = TimeOfDay::of(18, 30).value();
    check.that(18 == time.hour() && 30 == time.minute() && "18:30" == time.toString(), "the time's fields and text");
    checkValue(check, time, TimeOfDay::parse("18:30").value(), TimeOfDay::of(18, 31).value(), "times compare by minute");
    check.that(isFailure(refusal(TimeOfDay::of(25, 0)), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_TIME), "hour 25");
    check.that(isFailure(refusal(TimeOfDay::parse("25:00")), ErrorKind::CALENDAR, Failure::CALENDAR_INVALID_TIME), "parsed");
}

/// Contact details and accounts kept as typed would differ from the same details typed otherwise, and a wrong one would pass.
void contactsAndAccountsAreCheckedAndMadeCanonical(Check &check) noexcept {
    auto const email = Email::parse(" ana@example.com ").value();
    check.that("ana@example.com" == email.toString(), "an email is trimmed");
    checkValue(check, email, Email::parse("ana@example.com").value(), Email::parse("bob@example.com").value(), "emails compare by text");
    check.that(isFailure(refusal(Email::parse("ana")), ErrorKind::CONTACT, Failure::CONTACT_INVALID_EMAIL), "an email has a domain");
    auto const phone = PhoneNumber::parse("+40 712 345 678").value();
    check.that("+40712345678" == phone.toString() && "+40 712 345 678" == phone.international(), "a phone number's two forms");
    checkValue(check, phone, PhoneNumber::parse("+40712345678").value(), PhoneNumber::parse("+40712345679").value(),
               "phone numbers compare by number");
    auto const local = refusal(PhoneNumber::parse("0712345678"));
    check.that(isFailure(local, ErrorKind::CONTACT, Failure::CONTACT_INVALID_PHONE_NUMBER), "a phone number has a country code");
    auto const iban = Iban::parse("ro49 aaaa 1b31 0075 9384 0000").value();
    check.that("RO49AAAA1B31007593840000" == iban.toString() && "RO" == iban.countryCode(), "an IBAN in its electronic form");
    checkValue(check, iban, Iban::parse("RO49AAAA1B31007593840000").value(), Iban::parse("GB82WEST12345698765432").value(), "IBANs compare by text");
    auto const checksum = refusal(Iban::parse("RO49AAAA1B31007593840001"));
    check.that(isFailure(checksum, ErrorKind::PAYMENT_ACCOUNT, Failure::PAYMENT_ACCOUNT_INVALID_IBAN) &&
                   IbanReason::INVALID_CHECKSUM == checksum.value().reason(),
               "an IBAN whose check digits do not match");
    check.that(checksum.has_value() && "invalid IBAN: the check digits do not match" == checksum.value().message(), "the message");
    auto const length = refusal(Iban::parse("RO49AAAA1B3100759384000"));
    check.that(length.has_value() && IbanReason::WRONG_LENGTH == length.value().reason() && 24 == length.value().expectedLength() &&
                   23 == length.value().foundLength(),
               "an IBAN one character short holds both lengths");
    auto const username = Username::parse(" ana.pays ").value();
    check.that("ana.pays" == username.toString(), "a username is trimmed");
    checkValue(check, username, Username::parse("ana.pays").value(), Username::parse("bob.pays").value(), "usernames compare by text");
    check.that(isFailure(refusal(Username::parse(" ")), ErrorKind::PAYMENT_ACCOUNT, Failure::PAYMENT_ACCOUNT_EMPTY_USERNAME), "a blank username");
    auto const language = Language::parse("pt-pt").value();
    check.that("pt-PT" == language.toString(), "a language tag is canonical");
    checkValue(check, language, Language::parse("pt-PT").value(), Language::parse("ro-RO").value(), "languages compare by tag");
    auto const tag = refusal(Language::parse("not a tag"));
    check.that(isFailure(tag, ErrorKind::LANGUAGE, Failure::LANGUAGE_INVALID_LANGUAGE), "a language is a tag");
}

/// An identifier that compared or hashed otherwise than by its text would lose an entry in the maps and sets of an app, and one that
/// ordered otherwise than by its bytes would sort entries otherwise than the library lists them.
void everyIdentifierComparesAndHashesByItsText(Check &check) noexcept {
    checkIdentifier<ExpenseGroupId>(check, "a group's identifier");
    checkIdentifier<MemberId>(check, "a member's identifier");
    checkIdentifier<CategoryId>(check, "a category's identifier");
    checkIdentifier<ExpenseId>(check, "an expense's identifier");
    checkIdentifier<TransferId>(check, "a transfer's identifier");
}

/// An identifier kept otherwise than it was written, trimmed or in another case, would no longer match the key the app gave, and text the library
/// refuses, taken, would fail every call that carries it.
void everyIdentifierIsCheckedAndKeptAsItIsWritten(Check &check) noexcept {
    checkParse<ExpenseGroupId>(check, "a group's identifier");
    checkParse<MemberId>(check, "a member's identifier");
    checkParse<CategoryId>(check, "a category's identifier");
    checkParse<ExpenseId>(check, "an expense's identifier");
    checkParse<TransferId>(check, "a transfer's identifier");
}

/// A generated identifier that repeated an earlier one would put a new entry in the place of another, and one the rules refused would fail every
/// call that carries it.
void everyIdentifierTypeGeneratesANewIdentifierTheRulesAccept(Check &check) noexcept {
    checkGenerate<ExpenseGroupId>(check, "a group's identifier");
    checkGenerate<MemberId>(check, "a member's identifier");
    checkGenerate<CategoryId>(check, "a category's identifier");
    checkGenerate<ExpenseId>(check, "an expense's identifier");
    checkGenerate<TransferId>(check, "a transfer's identifier");
}

// --- Records and alternatives ------------------------------------------------------------------------------------------

/// A record or an alternative whose comparison left a field out would merge two entries an app holds apart.
void everyRecordAndAlternativeEqualsItsTwinAndDiffersFromAnother(Check &check) noexcept {
    auto const euro = CurrencyCode::of("EUR").value();
    MemberId const ana("0199a000-0000-7000-8000-000000000001");
    MemberId const bob("0199a000-0000-7000-8000-000000000002");
    CategoryId const food("0199a000-0000-7000-8000-000000000003");
    Money const one(100, euro);
    Money const two(200, euro);
    auto const day = CalendarDate::of(2026, 9, 23).value();
    checkValue(check, MemberParts(ana, Parts{100}), MemberParts(ana, Parts{100}), MemberParts(ana, Parts{50}), "member parts");
    checkValue(check, MemberPercentage(ana, Percentage{5'000}), MemberPercentage(ana, Percentage{5'000}), MemberPercentage(bob, Percentage{5'000}),
               "member percentages");
    checkValue(check, MemberAmount(ana, one), MemberAmount(ana, one), MemberAmount(bob, one), "member amounts");
    checkValue(check, EqualSplit{{ana}}, EqualSplit{{ana}}, EqualSplit{{ana, bob}}, "equal splits");
    checkValue(check, PartsSplit{{MemberParts(ana, Parts{100})}}, PartsSplit{{MemberParts(ana, Parts{100})}}, PartsSplit{{}}, "parts splits");
    checkValue(check, PercentagesSplit{{MemberPercentage(ana, Percentage{10'000})}}, PercentagesSplit{{MemberPercentage(ana, Percentage{10'000})}},
               PercentagesSplit{{}}, "percentage splits");
    checkValue(check, AmountsSplit{{MemberAmount(ana, one)}}, AmountsSplit{{MemberAmount(ana, one)}}, AmountsSplit{{MemberAmount(ana, two)}},
               "amounts");
    ExpenseItem const pizza("Pizza", one, EqualSplit{{ana, bob}});
    auto twoPizzas = pizza;
    twoPizzas.quantity = 2;
    checkValue(check, pizza, ExpenseItem("Pizza", one, EqualSplit{{ana, bob}}), twoPizzas, "items compare by quantity too");
    checkValue(check, PercentageTipValue{Percentage{1'000}}, PercentageTipValue{Percentage{1'000}}, PercentageTipValue{Percentage{500}},
               "percentage tips");
    checkValue(check, FixedTipValue{one}, FixedTipValue{one}, FixedTipValue{two}, "fixed tips");
    TipRule const tip(FixedTipValue{one}, TipDistribution::EQUAL);
    checkValue(check, tip, TipRule(FixedTipValue{one}, TipDistribution::EQUAL), TipRule(FixedTipValue{one}, TipDistribution::PROPORTIONAL), "tips");
    checkValue(check, LocalCost{one}, LocalCost{one}, LocalCost{two}, "local costs");
    auto const rate = ExchangeRate::parse("0.2").value();
    checkValue(check, ConvertedCost{.amount = one, .rate = rate}, ConvertedCost{.amount = one, .rate = rate},
               ConvertedCost{.amount = one, .rate = ExchangeRate::parse("0.3").value()}, "converted costs");
    checkValue(check, WholeExpenseSplit{EqualSplit{{ana}}}, WholeExpenseSplit{EqualSplit{{ana}}}, WholeExpenseSplit{EqualSplit{{bob}}},
               "whole splits");
    checkValue(check, ItemisedExpenseSplit{{pizza}}, ItemisedExpenseSplit{{pizza}}, ItemisedExpenseSplit{{twoPizzas}}, "itemised splits");
    ExpenseId const dinnerId("0199a000-0000-7000-8000-000000000004");
    Expense const dinner(dinnerId, "Dinner", LocalCost{two}, EqualSplit{{ana}}, WholeExpenseSplit{EqualSplit{{ana, bob}}}, day);
    auto described = dinner;
    described.description = "At the corner";
    checkValue(check, dinner, Expense(dinnerId, "Dinner", LocalCost{two}, EqualSplit{{ana}}, WholeExpenseSplit{EqualSplit{{ana, bob}}}, day),
               described, "expenses compare by description too");
    checkValue(check, dinner, Expense(dinnerId, "Dinner", LocalCost{two}, EqualSplit{{ana}}, WholeExpenseSplit{EqualSplit{{ana, bob}}}, day),
               Expense("Dinner", LocalCost{two}, EqualSplit{{ana}}, WholeExpenseSplit{EqualSplit{{ana, bob}}}, day),
               "expenses compare by identifier too");
    TransferId const repaymentId("0199a000-0000-7000-8000-000000000005");
    Transfer const repayment(repaymentId, bob, ana, LocalCost{one}, day);
    auto noted = repayment;
    noted.note = "Cash";
    checkValue(check, repayment, Transfer(repaymentId, bob, ana, LocalCost{one}, day), noted, "transfers compare by note too");
    checkValue(check, repayment, Transfer(repaymentId, bob, ana, LocalCost{one}, day), Transfer(bob, ana, LocalCost{one}, day),
               "transfers compare by identifier too");
    auto const iban = Iban::parse("RO49AAAA1B31007593840000").value();
    checkValue(check, IbanPaymentAccountDetails{iban}, IbanPaymentAccountDetails{iban},
               IbanPaymentAccountDetails{Iban::parse("GB82WEST12345698765432").value()}, "IBAN details");
    auto const username = Username::parse("ana.pays").value();
    checkValue(check, UsernamePaymentAccountDetails{username}, UsernamePaymentAccountDetails{username},
               UsernamePaymentAccountDetails{Username::parse("bob.pays").value()}, "username details");
    checkValue(check, PaymentAccount("Bank", IbanPaymentAccountDetails{iban}), PaymentAccount("Bank", IbanPaymentAccountDetails{iban}),
               PaymentAccount("App", UsernamePaymentAccountDetails{username}), "accounts");
    Member const member(ana, "Ana");
    auto withLastName = member;
    withLastName.lastName = "Pop";
    checkValue(check, member, Member(ana, "Ana"), withLastName, "members compare by last name too");
    checkValue(check, member, Member(ana, "Ana"), Member("Ana"), "members compare by identifier too");
    checkValue(check, Category(food, "Food"), Category(food, "Food"), Category(food, "Lodging"), "categories");
    checkValue(check, Category(food, "Food"), Category(food, "Food"), Category("Food"), "categories compare by identifier too");
    checkValue(check, OwesBalance{one}, OwesBalance{one}, OwesBalance{two}, "balances a member owes");
    check.that(SettledBalance{} == SettledBalance{} && !(SettledBalance{} != SettledBalance{}), "settled balances are all equal");
    checkValue(check, GetsBackBalance{one}, GetsBackBalance{one}, GetsBackBalance{two}, "balances owed to a member");
    MemberSummary const summary(ana, two, one, one, one, one, GetsBackBalance{one});
    checkValue(check, summary, MemberSummary(ana, two, one, one, one, one, GetsBackBalance{one}),
               MemberSummary(ana, two, one, one, one, one, OwesBalance{one}), "member summaries compare by balance too");
    CategoryTotal const total(food, one);
    checkValue(check, total, CategoryTotal(food, one), CategoryTotal(food, two), "category totals");
    checkValue(check, MemberCategoryTotals(ana, {total}), MemberCategoryTotals(ana, {total}), MemberCategoryTotals(ana, {}),
               "member category totals");
    checkValue(check, Summary(two, {summary}, {total}, {}), Summary(two, {summary}, {total}, {}), Summary(two, {summary}, {}, {}), "summaries");
    MemberShare const share(ana, two, one);
    checkValue(check, share, MemberShare(ana, two, one), MemberShare(ana, one, one), "member shares");
    checkValue(check, ExpenseShares({share}, two, one), ExpenseShares({share}, two, one), ExpenseShares({share}, two, two), "expense shares");
    Payment const payment(bob, ana, one);
    checkValue(check, payment, Payment(bob, ana, one), Payment(ana, bob, one), "payments compare by direction");
    checkValue(check, SettlementPlan({payment}, SettlementStrategy::GREEDY), SettlementPlan({payment}, SettlementStrategy::GREEDY),
               SettlementPlan({payment}, SettlementStrategy::DIRECT), "settlements compare by strategy too");
    checkValue(check, CurrencyRate(euro, rate), CurrencyRate(euro, rate), CurrencyRate(CurrencyCode::of("RON").value(), rate), "currency rates");
}

// --- Entries through the group -----------------------------------------------------------------------------------------

/// An expense that came back from the group otherwise than it went in would change what an app shows after a save.
void anExpenseReadsBackAsItWasAdded(Check &check) noexcept {
    Flat flat;
    auto const lei = CurrencyCode::of("RON").value();
    Expense lunch("Lunch", ConvertedCost{.amount = Money(5'000, lei), .rate = ExchangeRate::parse("0.2").value()},
                  PercentagesSplit{{MemberPercentage(flat.ana, Percentage{5'000}), MemberPercentage(flat.bob, Percentage{5'000})}},
                  WholeExpenseSplit{PartsSplit{{MemberParts(flat.ana, Parts{100}), MemberParts(flat.bob, Parts{50})}}}, flat.day);
    lunch.tip = TipRule(PercentageTipValue{Percentage{1'000}}, TipDistribution::PROPORTIONAL);
    lunch.time = TimeOfDay::of(13, 15).value();
    lunch.category = flat.food;
    lunch.description = "By the sea";
    check.that(flat.group.addExpense(lunch) == lunch.id && flat.group.expense(lunch.id) == lunch,
               "a converted expense with every detail reads back unchanged, under its identifier");
    ExpenseItem pizza("Pizza", flat.euros(1'200), EqualSplit{{flat.ana, flat.bob}});
    pizza.quantity = 2;
    Expense receipt(
        "Receipt", LocalCost{flat.euros(3'000)}, AmountsSplit{{MemberAmount(flat.bob, flat.euros(3'300))}},
        ItemisedExpenseSplit{{pizza, ExpenseItem("Wine", flat.euros(600), PercentagesSplit{{MemberPercentage(flat.ana, Percentage{10'000})}})}},
        flat.day);
    receipt.tip = TipRule(FixedTipValue{flat.euros(300)}, TipDistribution::EQUAL);
    check.that(flat.group.addExpense(receipt) == receipt.id && flat.group.expense(receipt.id) == receipt,
               "an itemised receipt with a fixed tip reads back unchanged");
    auto const expenses = flat.group.expenses();
    check.that(3 == expenses.size(), "the group holds the dinner, the lunch, and the receipt");
    check.that(!flat.group.expense(ExpenseId("0199a000-0000-7000-8000-00000000ffff")).value().has_value(), "an absent expense is none");
}

static_assert(TipDistribution::PROPORTIONAL == TipRule::none().distribution, "the no-tip rule is made at compile time");
static_assert(36 == CategoryId::uncategorised().value().size(), "and so is the built-in category's identifier");

/// An expense the group returned without the built-in category's identifier or the no-tip rule would leave an app unable to tell which category
/// it is in or what tip it has, and a built-in identifier or a no-tip rule other than the library's would file it under a category the group does
/// not hold or give it a tip.
void anExpenseWithoutACategoryOrATipReadsBackInTheBuiltInOneWithTheNoTipRule(Check &check) noexcept {
    check.that(CategoryId::uncategorised() == CategoryId(detail::EranosIdentifier::uncategorised()),
               "the built-in category's identifier is the library's");
    check.that(TipRule::none() == detail::fromDetail(*detail::EranosTipRule::none()), "the no-tip rule is the library's");
    Flat flat;
    check.that(flat.group.category(CategoryId::uncategorised()) == flat.group.categories().at(0), "the group holds the built-in category under it");
    auto taxi = flat.expense("Taxi", flat.ana, {flat.ana, flat.bob});
    check.that(CategoryId::uncategorised() == taxi.category && TipRule::none() == taxi.tip, "a new expense starts in it, with the no-tip rule");
    check.that(flat.group.addExpense(taxi) == taxi.id && flat.group.expense(taxi.id) == taxi, "and reads back so from the group");
    taxi.tip = TipRule(PercentageTipValue{Percentage{0}}, TipDistribution::PROPORTIONAL);
    check.that(flat.group.replaceExpense(taxi).has_value() && TipRule::none() == flat.group.expense(taxi.id).value().value().tip,
               "a 0 % tip shared proportionally reads back as the no-tip rule");
}

/// A transfer or a member that came back otherwise than it went in would move money to the wrong account after a save.
void aTransferAndAMemberReadBackAsTheyWereAdded(Check &check) noexcept {
    Flat flat;
    Transfer cash(flat.ana, flat.bob,
                  ConvertedCost{.amount = Money(1'000, CurrencyCode::of("RON").value()), .rate = ExchangeRate::parse("0.2").value()}, flat.day);
    cash.time = TimeOfDay::of(9, 5).value();
    cash.note = "Cash";
    check.that(flat.group.addTransfer(cash) == cash.id && flat.group.transfer(cash.id) == cash,
               "a converted transfer with a note and a time reads back unchanged");
    check.that(flat.group.transferAmount(cash.id) == flat.euros(200), "the transfer is 2.00 EUR in the group currency");
    check.that(2 == flat.group.transfers().size(), "the group holds both transfers");
    check.that(!flat.group.transferAmount(TransferId("0199a000-0000-7000-8000-00000000ffff")).value().has_value(),
               "an absent transfer has no amount");
    Member carla("Carla");
    carla.lastName = "Pop";
    carla.email = Email::parse("carla@example.com").value();
    carla.phone = PhoneNumber::parse("+40712345678").value();
    carla.accounts = {
        PaymentAccount("Bank", IbanPaymentAccountDetails{Iban::parse("RO49AAAA1B31007593840000").value()}),
        PaymentAccount("App", UsernamePaymentAccountDetails{Username::parse("carla.pays").value()}),
    };
    check.that(flat.group.addMember(carla) == carla.id && flat.group.member(carla.id) == carla, "a member with every detail reads back unchanged");
    check.that(3 == flat.group.members().size(), "the group holds three members");
    check.that(flat.group.category(flat.food) == Category(flat.food, "Food"), "the category reads back");
    check.that(2 == flat.group.categories().size(), "the built-in category and the food");
}

/// An edit that lost an entry's identifier, or returned the entry as it became, would leave an app's undo pointing nowhere.
void replacingAnEntryKeepsItsIdentifierAndReturnsItAsItWas(Check &check) noexcept {
    Flat flat;
    auto const dinner = flat.group.expense(flat.dinner).value().value();
    auto lunch = dinner;
    lunch.name = "Lunch";
    check.that(flat.group.replaceExpense(lunch) == dinner, "replacing an expense returns it as it was");
    check.that(flat.group.expense(flat.dinner) == lunch, "the expense is replaced under its identifier");
    auto const repayment = flat.group.transfer(flat.repayment).value().value();
    auto noted = repayment;
    noted.note = "Back";
    check.that(flat.group.replaceTransfer(noted) == repayment && flat.group.transfer(flat.repayment) == noted, "a transfer");
    auto const ana = flat.group.member(flat.ana).value().value();
    auto renamed = ana;
    renamed.firstName = "Ana Maria";
    check.that(flat.group.replaceMember(renamed) == ana && flat.group.member(flat.ana) == renamed, "a member");
    auto const food = flat.group.category(flat.food).value().value();
    auto meals = food;
    meals.name = "Meals";
    check.that(flat.group.replaceCategory(meals) == food && flat.group.category(flat.food) == meals, "a category");
    auto const duplicate = refusal(flat.group.addExpense(lunch));
    check.that(isFailure(duplicate, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_DUPLICATE_EXPENSE),
               "an entry to add takes no identifier the group holds");
    Member const dan(MemberId(" x"), "Dan");
    check.that(isFailure(refusal(flat.group.addMember(dan)), ErrorKind::ID, Failure::ID_SURROUNDING_WHITESPACE), "nor one the library refuses");
}

/// A replacement of an entry the group does not hold would change another entry, or report a change it never made; each is refused with the
/// failure that names the entry.
void anEntryTheGroupDoesNotHoldIsNotReplaced(Check &check) noexcept {
    Flat flat;
    auto const snack = flat.expense("Snack", flat.ana, {flat.ana});
    auto const expense = refusal(flat.group.replaceExpense(snack));
    check.that(isFailure(expense, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_EXPENSE) && expense.value().expense() == snack.id,
               "an expense");
    Transfer const cash(flat.ana, flat.bob, LocalCost{flat.euros(500)}, flat.day);
    auto const transfer = refusal(flat.group.replaceTransfer(cash));
    check.that(isFailure(transfer, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_TRANSFER) && transfer.value().transfer() == cash.id,
               "a transfer");
    Member const carla("Carla");
    auto const member = refusal(flat.group.replaceMember(carla));
    check.that(isFailure(member, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_MEMBER) && member.value().member() == carla.id, "a member");
    Category const travel("Travel");
    auto const category = refusal(flat.group.replaceCategory(travel));
    check.that(isFailure(category, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_CATEGORY) && category.value().category() == travel.id,
               "a category");
    check.that(2 == flat.group.members().size() && 2 == flat.group.categories().size() && 1 == flat.group.expenses().size() &&
                   1 == flat.group.transfers().size(),
               "and the group holds what it held");
}

/// An entry the group kept under another identifier than the app gave it would break the app's link to its own record, and a second entry under
/// one identifier would stand for the first in every lookup.
void anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt(Check &check) noexcept {
    Flat flat;
    Member const carla(MemberId::parse("carla").value(), "Carla");
    check.that(flat.group.addMember(carla) == carla.id && flat.group.member(carla.id).value() == carla, "a member keeps it");
    auto const memberTwice = refusal(flat.group.addMember(carla));
    check.that(isFailure(memberTwice, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_DUPLICATE_MEMBER) && memberTwice.value().member() == carla.id,
               "a second member under it is refused");
    Category lodging(CategoryId::parse("lodging").value(), "Lodging");
    check.that(flat.group.addCategory(lodging) == lodging.id && flat.group.category(lodging.id).value() == lodging, "a category keeps it");
    lodging.name = "Rooms";
    auto const categoryTwice = refusal(flat.group.addCategory(lodging));
    check.that(isFailure(categoryTwice, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_DUPLICATE_CATEGORY) &&
                   categoryTwice.value().category() == lodging.id,
               "a second category under it is refused, whatever its name");
    Expense const receipt(ExpenseId::parse("receipt-17").value(), "Receipt", LocalCost{flat.euros(3'000)}, EqualSplit{{flat.ana}},
                          WholeExpenseSplit{EqualSplit{{flat.ana, flat.bob}}}, flat.day);
    check.that(flat.group.addExpense(receipt) == receipt.id && flat.group.expense(receipt.id).value() == receipt, "an expense keeps it");
    auto const expenseTwice = refusal(flat.group.addExpense(receipt));
    check.that(isFailure(expenseTwice, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_DUPLICATE_EXPENSE) &&
                   expenseTwice.value().expense() == receipt.id,
               "a second expense under it is refused");
    Transfer const cash(TransferId::parse("payment-3").value(), flat.ana, flat.bob, LocalCost{flat.euros(500)}, flat.day);
    check.that(flat.group.addTransfer(cash) == cash.id && flat.group.transfer(cash.id).value() == cash, "a transfer keeps it");
    auto const transferTwice = refusal(flat.group.addTransfer(cash));
    check.that(isFailure(transferTwice, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_DUPLICATE_TRANSFER) &&
                   transferTwice.value().transfer() == cash.id,
               "a second transfer under it is refused");
}

/// A removal that returned something else, or left the entry behind, would let an app show a deleted entry.
void removingAnEntryReturnsItAndForgetsIt(Check &check) noexcept {
    Flat flat;
    auto const dinner = flat.group.expense(flat.dinner).value().value();
    check.that(flat.group.removeExpense(flat.dinner) == dinner && !flat.group.expense(flat.dinner).value().has_value(), "an expense");
    auto const repayment = flat.group.transfer(flat.repayment).value().value();
    check.that(flat.group.removeTransfer(flat.repayment) == repayment && !flat.group.transfer(flat.repayment).value().has_value(), "a transfer");
    auto const food = flat.group.category(flat.food).value().value();
    check.that(flat.group.removeCategory(flat.food) == food && !flat.group.category(flat.food).value().has_value(), "a category");
    auto const bob = flat.group.member(flat.bob).value().value();
    check.that(flat.group.removeMember(flat.bob) == bob && !flat.group.member(flat.bob).value().has_value(), "a member no entry names");
}

/// A member removed while entries name them would leave shares nobody owes.
void aMemberInUseIsNotRemoved(Check &check) noexcept {
    Flat flat;
    auto const refused = refusal(flat.group.removeMember(flat.ana));
    check.that(isFailure(refused, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_MEMBER_IN_USE), "Ana paid the dinner and received the repayment");
    check.that(refused.has_value() && refused.value().member() == flat.ana && refused.value().expenses() == std::vector<ExpenseId>{flat.dinner} &&
                   refused.value().transfers() == std::vector<TransferId>{flat.repayment},
               "the error names the member and every entry that names them");
}

/// An archived member who could join a new expense would owe money for a trip they left.
void anArchivedMemberJoinsNoNewExpense(Check &check) noexcept {
    Flat flat;
    flat.group.archiveMember(flat.bob).value();
    check.that(flat.group.isArchived(flat.bob).value() && !flat.group.isArchived(flat.ana).value(), "Bob is archived and Ana is not");
    auto const refused = refusal(flat.group.addExpense(flat.expense("Snack", flat.ana, {flat.ana, flat.bob})));
    check.that(isFailure(refused, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_ARCHIVED_MEMBER) && refused.value().member() == flat.bob,
               "the error names Bob");
    flat.group.unarchiveMember(flat.bob).value();
    check.that(!flat.group.isArchived(flat.bob).value(), "Bob is back");
    auto const unknown = refusal(flat.group.archiveMember(MemberId("0199a000-0000-7000-8000-00000000ffff")));
    check.that(isFailure(unknown, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_MEMBER) && unknown.value().member().has_value(),
               "an unknown member");
}

/// A helper that read an entry otherwise than the library would show an app a total the group does not hold.
void theHelpersReadAnEntryAsTheLibraryDoes(Check &check) noexcept {
    Flat flat;
    Split const split = EqualSplit{{flat.bob, flat.ana}};
    check.that(participants(split) == std::vector<MemberId>{flat.bob, flat.ana}, "a split's participants, in the order it lists them");
    auto dinner = flat.expense("Dinner", flat.ana, {flat.bob});
    dinner.tip = TipRule(PercentageTipValue{Percentage{1'000}}, TipDistribution::EQUAL);
    check.that(participants(dinner) == std::vector<MemberId>{flat.ana, flat.bob},
               "the payer and the participants, in the order of their identifiers");
    check.that(tipAmount(dinner) == flat.euros(300) && total(dinner) == flat.euros(3'300), "a tip of 10 % on 30.00 EUR");
    check.that("owes 12.00 EUR" == toString(Balance{OwesBalance{flat.euros(1'200)}}), "a balance a member owes");
    check.that("settled" == toString(Balance{SettledBalance{}}), "a settled balance");
    check.that("gets back 12.00 EUR" == toString(Balance{GetsBackBalance{flat.euros(1'200)}}), "a balance owed to a member");
    auto const unknown = refusal(participants(Split{EqualSplit{{MemberId(" ana")}}}));
    check.that(isFailure(unknown, ErrorKind::ID, Failure::ID_SURROUNDING_WHITESPACE), "an identifier that is not one");
}

// --- The group ---------------------------------------------------------------------------------------------------------

/// A group that started without the identifier and the details it was given, or whose details did not change as asked, would show an app a name
/// or a language the user replaced.
void aGroupHoldsItsDetailsAndChangesThem(Check &check) noexcept {
    Flat flat;
    check.that(!flat.group.id().value().empty(), "the group has an identifier");
    check.that("Flat" == flat.group.name() && flat.group.currency() == flat.euro, "the name and the currency");
    check.that("Rent" == flat.group.description() && flat.group.language() == Language::parse("pt-PT").value(), "the options it started with");
    flat.group.setName("Home").value();
    flat.group.setDescription("Rent and bills").value();
    flat.group.setLanguage(Language::parse("ro-RO").value());
    check.that("Home" == flat.group.name() && "Rent and bills" == flat.group.description() &&
                   flat.group.language() == Language::parse("ro-RO").value(),
               "the details as set");
    flat.group.clearDescription();
    flat.group.clearLanguage();
    check.that(!flat.group.description().has_value() && !flat.group.language().has_value(), "the details cleared");
    check.that(isFailure(refusal(flat.group.setName(" ")), ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_EMPTY_NAME), "a blank name");
    check.that(isFailure(refusal(flat.group.setDescription("\xff")), ErrorKind::INVALID_TEXT, std::nullopt), "a description that is not text");
    auto const trip = ExpenseGroup::create("Trip", flat.euro).value();
    check.that(!trip.description().has_value() && !trip.language().has_value(), "a group made without options has neither");
    check.that(trip.id() != flat.group.id(), "a group made without an identifier has a generated one of its own");
    check.that(isFailure(refusal(ExpenseGroup::create(" ", flat.euro)), ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_EMPTY_NAME),
               "a group needs a name");
    auto const home = ExpenseGroup::create(ExpenseGroupId::parse("home").value(), "Home", flat.euro).value();
    check.that(ExpenseGroupId("home") == home.id(), "a group under the app's identifier keeps it");
    auto const refused = refusal(ExpenseGroup::create(ExpenseGroupId(" x"), "Home", flat.euro));
    check.that(isFailure(refused, ErrorKind::ID, Failure::ID_SURROUNDING_WHITESPACE), "an identifier the library refuses");
}

/// Totals that did not add up would tell a member they owe what they do not.
void theSummaryAndTheSharesAddUp(Check &check) noexcept {
    Flat flat;
    auto const summary = flat.group.summary();
    check.that(summary.totalSpent == flat.euros(3'000), "the dinner is all that was spent");
    check.that(summary.categoryTotals == std::vector<CategoryTotal>{CategoryTotal(flat.food, flat.euros(3'000))}, "all of it on food");
    auto const ana = std::ranges::find_if(summary.members, [&](MemberSummary const &member) noexcept { return member.member == flat.ana; });
    check.that(ana != summary.members.end() && ana->paid == flat.euros(3'000) && ana->share == flat.euros(1'500) &&
                   ana->received == flat.euros(1'000) && ana->sent == flat.euros(0) && ana->net == flat.euros(500) &&
                   ana->balance == Balance{GetsBackBalance{flat.euros(500)}},
               "Ana paid 30.00, consumed 15.00, and got 10.00 back, so she is owed 5.00");
    check.that(2 == summary.memberCategoryTotals.size(), "both members have a share of the food");
    auto const shares = flat.group.expenseShares(flat.dinner).value();
    check.that(shares.has_value() && shares.value().total == flat.euros(3'000) && shares.value().tip == flat.euros(0) &&
                   2 == shares.value().members.size(),
               "the dinner's shares");
}

/// Every strategy, in the order they are declared.
constexpr std::array STRATEGIES = {
    SettlementStrategy::ADAPTIVE,
    SettlementStrategy::MINIMUM_TRANSFERS,
    SettlementStrategy::GREEDY,
    SettlementStrategy::DIRECT,
};

/// A strategy that settled otherwise than the library would ask a member to pay twice.
void eachStrategySettlesEveryBalance(Check &check) noexcept {
    Flat flat;
    for (auto const strategy : STRATEGIES) {
        auto const settlement = flat.group.settlement(strategy).value();
        // The fixture's two balances are well within the adaptive strategy's limit, so it settles them exactly.
        auto const computedBy = SettlementStrategy::ADAPTIVE == strategy ? SettlementStrategy::MINIMUM_TRANSFERS : strategy;
        check.that(settlement.strategy == computedBy, "the plan names the strategy whose plan it is");
        check.that(settlement.payments == std::vector<Payment>{Payment(flat.bob, flat.ana, flat.euros(500))}, "Bob pays Ana 5.00");
    }
    static_cast<void>(flat.group.addTransfer(Transfer(flat.bob, flat.ana, LocalCost{flat.euros(500)}, flat.day)).value());
    auto const members = flat.group.summary().members;
    check.that(std::ranges::all_of(members, [](MemberSummary const &member) noexcept { return member.balance == Balance{SettledBalance{}}; }),
               "once Bob pays, every balance is settled");
}

/// A strategy that converted to another would settle by a plan the app did not ask for, or name another than the one that made it. No
/// settlement holds the adaptive strategy, so only this round trip converts it back from the generated value.
void everyStrategyConvertsToTheGeneratedOneAndBack(Check &check) noexcept {
    for (auto const strategy : STRATEGIES) {
        check.that(detail::fromDetail(detail::toDetail(strategy)) == strategy, "the strategy converts to the generated one and back");
    }
}

/// A change of currency that left an entry unconverted, or took a rate it was not given, would mix currencies in the totals.
void aChangeOfCurrencyConvertsEveryEntry(Check &check) noexcept {
    Flat flat;
    auto const lei = CurrencyCode::of("RON").value();
    auto const missing = refusal(flat.group.changeCurrency(lei, {}));
    check.that(isFailure(missing, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_MISSING_EXCHANGE_RATE) && "EUR" == missing.value().currency(),
               "the euro needs a rate");
    auto const rate = CurrencyRate(flat.euro, ExchangeRate::parse("5").value());
    auto const twice = refusal(flat.group.changeCurrency(lei, std::array{rate, rate}));
    check.that(isFailure(twice, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_DUPLICATE_EXCHANGE_RATE) && "EUR" == twice.value().currency(),
               "one rate per currency");
    auto converted = flat.expense("Taxi", flat.ana, {flat.ana});
    converted.cost = ConvertedCost{.amount = flat.euros(1'000), .rate = ExchangeRate::parse("1").value()};
    auto const unexpected = refusal(flat.group.addExpense(converted));
    check.that(isFailure(unexpected, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNEXPECTED_EXCHANGE_RATE),
               "an amount in the group currency has no rate");
    auto local = converted;
    local.cost = LocalCost{Money(1'000, lei)};
    auto const unconverted = refusal(flat.group.addExpense(local));
    check.that(isFailure(unconverted, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_MISSING_EXCHANGE_RATE) &&
                   "RON" == unconverted.value().currency(),
               "an amount in another currency needs a rate");
    flat.group.changeCurrency(lei, std::array{rate}).value();
    check.that(flat.group.currency() == lei && flat.group.summary().totalSpent == Money(15'000, lei), "30.00 EUR at 5 is 150.00 RON");
}

/// A saved group that loaded otherwise than it was saved would lose entries between two runs of an app.
void aSavedGroupLoadsAsItWasSaved(Check &check) noexcept {
    Flat const flat;
    auto const json = flat.group.save();
    check.that(ExpenseGroup::load(json).value().save() == json, "the loaded group saves the same text");
    auto const broken = refusal(ExpenseGroup::load("{"));
    check.that(isFailure(broken, ErrorKind::INVALID_JSON, std::nullopt) && 1 == broken.value().line() && 1 == broken.value().column(),
               "where the text stops being JSON");
    auto tampered = json;
    auto const format = tampered.find("\"format\": 3");
    tampered.replace(format, std::string_view("\"format\": 3").size(), "\"format\": 4");
    auto const newer = refusal(ExpenseGroup::load(tampered));
    check.that(isFailure(newer, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNSUPPORTED_FORMAT) && 4 == newer.value().format(),
               "a format this release cannot read");
    check.that("unsupported group format 4: this version reads format 3" == newer.value().message(), "the format it reads");
}

/// An identifier that is not one would reach the library as no entry's, or as another's; each of the group's lookups refuses it with
/// the failure of the rule it breaks, which converts from the generated failure by its number.
void anIdentifierThatIsNotOneIsRefusedByEveryLookup(Check &check) noexcept {
    Flat const flat;
    for (auto const &[failure, text] : refusedTexts()) {
        check.that(isFailure(refusal(flat.group.category(CategoryId(text))), ErrorKind::ID, failure), "a category");
        check.that(isFailure(refusal(flat.group.expense(ExpenseId(text))), ErrorKind::ID, failure), "an expense");
        check.that(isFailure(refusal(flat.group.expenseShares(ExpenseId(text))), ErrorKind::ID, failure), "an expense's shares");
        check.that(isFailure(refusal(flat.group.member(MemberId(text))), ErrorKind::ID, failure), "a member");
        check.that(isFailure(refusal(flat.group.transfer(TransferId(text))), ErrorKind::ID, failure), "a transfer");
        check.that(isFailure(refusal(flat.group.transferAmount(TransferId(text))), ErrorKind::ID, failure), "a transfer's amount");
    }
}

/// An error that lost what it holds would leave an app unable to say which entry, amount, or category the library refused.
void everyErrorHoldsWhatItsFailureHolds(Check &check) noexcept {
    Flat flat;
    auto uneven = flat.expense("Uneven", flat.ana, {});
    uneven.split = WholeExpenseSplit{AmountsSplit{{MemberAmount(flat.ana, flat.euros(1'000))}}};
    auto const amounts = refusal(flat.group.addExpense(uneven));
    check.that(isFailure(amounts, ErrorKind::EXPENSE, Failure::EXPENSE_AMOUNTS_DO_NOT_ADD_UP) && amounts.value().actual() == flat.euros(1'000) &&
                   amounts.value().expectedAmount() == flat.euros(3'000),
               "the amounts and what they should add up to");
    auto foreign = uneven;
    foreign.split = WholeExpenseSplit{AmountsSplit{{MemberAmount(flat.ana, Money(3'000, CurrencyCode::of("RON").value()))}}};
    auto const currencies = refusal(flat.group.addExpense(foreign));
    check.that(isFailure(currencies, ErrorKind::EXPENSE, Failure::EXPENSE_CURRENCY_MISMATCH) && "EUR" == currencies.value().expectedCurrency() &&
                   "RON" == currencies.value().foundCurrency(),
               "the expense's currency and the share's");
    auto halves = uneven;
    halves.split = WholeExpenseSplit{PercentagesSplit{{MemberPercentage(flat.ana, Percentage{5'000})}}};
    auto const percentages = refusal(flat.group.addExpense(halves));
    check.that(isFailure(percentages, ErrorKind::EXPENSE, Failure::EXPENSE_PERCENTAGES_DO_NOT_ADD_UP) &&
                   5'000 == percentages.value().totalBasisPoints(),
               "what the percentages add up to");
    auto const duplicate = refusal(flat.group.addCategory(Category("food")));
    check.that(isFailure(duplicate, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_DUPLICATE_CATEGORY_NAME) &&
                   duplicate.value().existing() == flat.food,
               "the category that has the name");
    CategoryId const absent("0199a000-0000-7000-8000-00000000ffff");
    auto uncategorised = flat.expense("Taxi", flat.ana, {flat.ana});
    uncategorised.category = absent;
    auto const category = refusal(flat.group.addExpense(uncategorised));
    check.that(isFailure(category, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_CATEGORY) && category.value().category() == absent,
               "the unknown category");
    auto const expense = refusal(flat.group.removeExpense(ExpenseId("0199a000-0000-7000-8000-00000000ffff")));
    check.that(isFailure(expense, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_EXPENSE) && expense.value().expense().has_value(),
               "the unknown expense");
    auto const transfer = refusal(flat.group.removeTransfer(TransferId("0199a000-0000-7000-8000-00000000ffff")));
    check.that(isFailure(transfer, ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_TRANSFER) && transfer.value().transfer().has_value(),
               "the unknown transfer");
    auto const identifier = refusal(flat.group.member(MemberId("a\tb")));
    check.that(isFailure(identifier, ErrorKind::ID, Failure::ID_CONTROL_CHARACTER), "text that is no identifier");
    check.that(identifier.has_value() && !identifier.value().balances().has_value() && !identifier.value().actual().has_value() &&
                   !identifier.value().reason().has_value(),
               "an error holds only what its failure holds");
}

// --- Asynchronous calls, the read handle, and the cancellation -----------------------------------------------------------

/// An asynchronous call that returned otherwise than its synchronous form would give an app two answers to one question.
void everyAsynchronousCallReturnsWhatItsSynchronousFormReturns(Check &check) noexcept {
    Flat flat;
    auto &group = flat.group;
    check.that(group.idAsync().get() == group.id() && group.nameAsync().get() == group.name(), "the identifier and the name");
    check.that(group.currencyAsync().get() == group.currency() && group.descriptionAsync().get().value() == group.description(),
               "the currency and the description");
    check.that(group.languageAsync().get().value() == group.language() && group.saveAsync().get() == group.save(),
               "the language and the saved group");
    check.that(group.categoriesAsync().get() == group.categories() &&
                   group.categoryAsync(flat.food).get().value() == group.category(flat.food).value(),
               "categories");
    check.that(group.expensesAsync().get() == group.expenses() && group.expenseAsync(flat.dinner).get().value() == group.expense(flat.dinner).value(),
               "expenses");
    check.that(group.expenseSharesAsync(flat.dinner).get().value() == group.expenseShares(flat.dinner).value() &&
                   group.summaryAsync().get() == group.summary(),
               "totals");
    check.that(group.membersAsync().get() == group.members() && group.memberAsync(flat.ana).get().value() == group.member(flat.ana).value(),
               "members");
    check.that(group.isArchivedAsync(flat.ana).get() == group.isArchived(flat.ana).value(), "whether a member is archived");
    check.that(group.settlementAsync(SettlementStrategy::GREEDY).get() == group.settlement(SettlementStrategy::GREEDY).value(), "a settlement");
    check.that(group.transfersAsync().get() == group.transfers() &&
                   group.transferAsync(flat.repayment).get().value() == group.transfer(flat.repayment).value(),
               "transfers");
    check.that(group.transferAmountAsync(flat.repayment).get().value() == group.transferAmount(flat.repayment).value(), "a transfer's amount");
    auto const lunch = group.addExpenseAsync(flat.expense("Lunch", flat.bob, {flat.ana})).get().value();
    auto renamed = group.expense(lunch).value().value();
    renamed.name = "Brunch";
    check.that("Lunch" == group.replaceExpenseAsync(renamed).get().value().name && group.removeExpenseAsync(lunch).get() == renamed,
               "an expense's life");
    auto const cash = group.addTransferAsync(Transfer(flat.ana, flat.bob, LocalCost{flat.euros(100)}, flat.day)).get().value();
    auto noted = group.transfer(cash).value().value();
    noted.note = "Coins";
    check.that(!group.replaceTransferAsync(noted).get().value().note.has_value() && group.removeTransferAsync(cash).get() == noted,
               "a transfer's life");
    auto const carla = group.addMemberAsync(Member("Carla")).get().value();
    auto renamedCarla = group.member(carla).value().value();
    renamedCarla.firstName = "Carla Maria";
    check.that("Carla" == group.replaceMemberAsync(renamedCarla).get().value().firstName, "a member replaced");
    group.archiveMemberAsync(carla).get().value();
    check.that(group.isArchived(carla).value(), "a member archived");
    group.unarchiveMemberAsync(carla).get().value();
    check.that(!group.isArchived(carla).value() && group.removeMemberAsync(carla).get() == renamedCarla, "a member unarchived and removed");
    auto const drinks = group.addCategoryAsync(Category("Drinks")).get().value();
    auto renamedDrinks = group.category(drinks).value().value();
    renamedDrinks.name = "Coffee";
    check.that("Drinks" == group.replaceCategoryAsync(renamedDrinks).get().value().name && group.removeCategoryAsync(drinks).get() == renamedDrinks,
               "a category");
    group.setNameAsync("Home").get().value();
    group.setDescriptionAsync("Bills").get().value();
    group.setLanguageAsync(Language::parse("ro-RO").value()).get().value();
    check.that("Home" == group.name() && "Bills" == group.description() && group.language() == Language::parse("ro-RO").value(), "the details set");
    group.clearDescriptionAsync().get().value();
    group.clearLanguageAsync().get().value();
    check.that(!group.description().has_value() && !group.language().has_value(), "the details cleared");
    group.changeCurrencyAsync(CurrencyCode::of("RON").value(), std::array{CurrencyRate(flat.euro, ExchangeRate::parse("5").value())}).get().value();
    check.that(group.currency() == CurrencyCode::of("RON").value(), "the currency changed");
    auto const loaded = ExpenseGroup::loadAsync(group.save()).get().value();
    check.that(loaded.save() == group.save(), "a group loaded on a thread of its own");
}

/// An asynchronous call that dropped the error its synchronous form returns would leave an app waiting for a change that never happened;
/// one given an entry the layer cannot convert would start a call the library never sees.
void anAsynchronousCallReportsTheErrorItsSynchronousFormReturns(Check &check) noexcept {
    Flat flat;
    MemberId const nobody("0199a000-0000-7000-8000-00000000ffff");
    auto removing = flat.group.removeMemberAsync(nobody);
    check.that(isFailure(refusal(removing.get()), ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_MEMBER), "a call that returns a value");
    auto archiving = flat.group.archiveMemberAsync(nobody);
    check.that(isFailure(refusal(archiving.get()), ErrorKind::EXPENSE_GROUP, Failure::EXPENSE_GROUP_UNKNOWN_MEMBER), "a call that returns nothing");
    auto loading = ExpenseGroup::loadAsync("{");
    check.that(isFailure(refusal(loading.get()), ErrorKind::INVALID_JSON, std::nullopt), "a load");
    auto adding = flat.group.addCategoryAsync(Category("\xff"));
    check.that(isFailure(refusal(adding.get()), ErrorKind::INVALID_TEXT, std::nullopt), "an entry whose text is not UTF-8");
    check.that(2 == flat.group.categories().size(), "and the group gains no category");
}

/// A read handle that let a change in, or read after its release, would show a report two states of one group.
void aReaderReadsTheGroupUntilItIsReleased(Check &check) noexcept {
    Flat const flat;
    auto reader = flat.group.read();
    check.that(reader.id() == flat.group.id() && "Flat" == reader.name() && reader.currency() == flat.euro, "the details");
    check.that("Rent" == reader.description() && reader.language() == Language::parse("pt-PT").value() && reader.save() == flat.group.save(),
               "the options");
    check.that(reader.categories() == flat.group.categories() && reader.category(flat.food).value() == flat.group.category(flat.food).value(),
               "the categories");
    check.that(reader.expenses() == flat.group.expenses() && reader.expense(flat.dinner).value() == flat.group.expense(flat.dinner).value(),
               "the expenses");
    check.that(reader.expenseShares(flat.dinner).value() == flat.group.expenseShares(flat.dinner).value() && reader.summary() == flat.group.summary(),
               "the totals");
    check.that(reader.members() == flat.group.members() && reader.member(flat.ana).value() == flat.group.member(flat.ana).value(), "the members");
    check.that(!reader.isArchived(flat.ana).value(), "whether a member is archived");
    check.that(reader.settlement(SettlementStrategy::DIRECT) == flat.group.settlement(SettlementStrategy::DIRECT).value(), "a settlement");
    check.that(reader.transfers() == flat.group.transfers() && reader.transfer(flat.repayment).value() == flat.group.transfer(flat.repayment).value(),
               "the transfers");
    check.that(reader.transferAmount(flat.repayment) == flat.euros(1'000), "a transfer's amount");
    reader.release();
    auto const released = refusal(reader.name());
    check.that(isFailure(released, ErrorKind::RELEASED, std::nullopt), "a read after the release");
    check.that(released.has_value() && "the read handle was released" == released.value().message(), "the message");
    auto const asynchronous = flat.group.readAsync().get().value();
    check.that("Flat" == asynchronous.name(), "a read handle from an asynchronous call");
}

/// A cancelled change that still ran would change the group after the app gave the change up; one that kept waiting would hold
/// the worker thread for a lock it no longer needs.
void aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas(Check &check) noexcept {
    Flat flat;
    auto reader = flat.group.read();
    Cancellation cancellation;
    auto changing = flat.group.changeCurrencyAsync(CurrencyCode::of("RON").value(),
                                                   std::array{CurrencyRate(flat.euro, ExchangeRate::parse("5").value())}, &cancellation);
    check.that(!cancellation.isCancelled(), "a cancellation starts uncancelled");
    cancellation.cancel();
    check.that(cancellation.isCancelled(), "and is cancelled once cancelled");
    check.that(isFailure(refusal(changing.get()), ErrorKind::CANCELLED, std::nullopt), "the change reports its cancellation");
    reader.release();
    check.that(flat.group.currency() == flat.euro, "the group keeps its currency");
    auto const late = refusal(flat.group.nameAsync(&cancellation).get());
    check.that(isFailure(late, ErrorKind::CANCELLED, std::nullopt), "a call given a cancelled cancellation does not run");
}

/// A kind of call whose outcome lost its failure would leave the app's future waiting, or hand it a value the library never
/// gave; a call given a cancelled cancellation reports the cancellation, whatever it returns.
void everyKindOfAsynchronousCallReportsItsCancellation(Check &check) noexcept {
    Flat const flat;
    Cancellation cancelled;
    cancelled.cancel();
    auto const reportsIt = [](std::optional<Error> const &error) noexcept { return isFailure(error, ErrorKind::CANCELLED, std::nullopt); };
    check.that(reportsIt(refusal(flat.group.categoriesAsync(&cancelled).get())), "the categories");
    check.that(reportsIt(refusal(flat.group.categoryAsync(flat.food, &cancelled).get())), "a category");
    check.that(reportsIt(refusal(flat.group.expensesAsync(&cancelled).get())), "the expenses");
    check.that(reportsIt(refusal(flat.group.expenseAsync(flat.dinner, &cancelled).get())), "an expense");
    check.that(reportsIt(refusal(flat.group.expenseSharesAsync(flat.dinner, &cancelled).get())), "an expense's shares");
    check.that(reportsIt(refusal(flat.group.isArchivedAsync(flat.ana, &cancelled).get())), "whether a member is archived");
    check.that(reportsIt(refusal(flat.group.membersAsync(&cancelled).get())), "the members");
    check.that(reportsIt(refusal(flat.group.readAsync(&cancelled).get())), "a read handle");
    check.that(reportsIt(refusal(flat.group.settlementAsync(SettlementStrategy::DIRECT, &cancelled).get())), "a settlement");
    check.that(reportsIt(refusal(flat.group.summaryAsync(&cancelled).get())), "the summary");
    check.that(reportsIt(refusal(flat.group.transferAmountAsync(flat.repayment, &cancelled).get())), "a transfer's amount");
    check.that(reportsIt(refusal(flat.group.transfersAsync(&cancelled).get())), "the transfers");
    check.that(reportsIt(refusal(flat.group.transferAsync(flat.repayment, &cancelled).get())), "a transfer");
}

/// A test: its name, the claim it makes, and its body.
struct Test final {
    std::string_view name;
    void (*body)(Check &check) noexcept;
};

constexpr std::array TESTS = {
    Test{.name = "aCurrencyHoldsWhatTheLibraryKnowsOfIt", .body = aCurrencyHoldsWhatTheLibraryKnowsOfIt},
    Test{.name = "everyCurrencyTheLibraryListsIsListedByCode", .body = everyCurrencyTheLibraryListsIsListedByCode},
    Test{.name = "aCodeTheLibraryDoesNotListIsRefused", .body = aCodeTheLibraryDoesNotListIsRefused},
    Test{.name = "textThatIsNotUtf8IsRefusedBeforeTheLibrarySeesIt", .body = textThatIsNotUtf8IsRefusedBeforeTheLibrarySeesIt},
    Test{.name = "anAmountParsesAndFormatsAsTheLibraryWritesIt", .body = anAmountParsesAndFormatsAsTheLibraryWritesIt},
    Test{.name = "amountsInOneCurrencyAddAndSubtract", .body = amountsInOneCurrencyAddAndSubtract},
    Test{.name = "aRateParsesAndConvertsAsTheLibraryDoes", .body = aRateParsesAndConvertsAsTheLibraryDoes},
    Test{.name = "percentagesAndPartsParseAndFormatAsTheLibraryDoes", .body = percentagesAndPartsParseAndFormatAsTheLibraryDoes},
    Test{.name = "datesAndTimesAreCheckedWhenMade", .body = datesAndTimesAreCheckedWhenMade},
    Test{.name = "contactsAndAccountsAreCheckedAndMadeCanonical", .body = contactsAndAccountsAreCheckedAndMadeCanonical},
    Test{.name = "everyIdentifierComparesAndHashesByItsText", .body = everyIdentifierComparesAndHashesByItsText},
    Test{.name = "everyIdentifierIsCheckedAndKeptAsItIsWritten", .body = everyIdentifierIsCheckedAndKeptAsItIsWritten},
    Test{.name = "everyIdentifierTypeGeneratesANewIdentifierTheRulesAccept", .body = everyIdentifierTypeGeneratesANewIdentifierTheRulesAccept},
    Test{.name = "everyRecordAndAlternativeEqualsItsTwinAndDiffersFromAnother", .body = everyRecordAndAlternativeEqualsItsTwinAndDiffersFromAnother},
    Test{.name = "anExpenseReadsBackAsItWasAdded", .body = anExpenseReadsBackAsItWasAdded},
    Test{
        .name = "anExpenseWithoutACategoryOrATipReadsBackInTheBuiltInOneWithTheNoTipRule",
        .body = anExpenseWithoutACategoryOrATipReadsBackInTheBuiltInOneWithTheNoTipRule,
    },
    Test{.name = "aTransferAndAMemberReadBackAsTheyWereAdded", .body = aTransferAndAMemberReadBackAsTheyWereAdded},
    Test{.name = "replacingAnEntryKeepsItsIdentifierAndReturnsItAsItWas", .body = replacingAnEntryKeepsItsIdentifierAndReturnsItAsItWas},
    Test{.name = "anEntryTheGroupDoesNotHoldIsNotReplaced", .body = anEntryTheGroupDoesNotHoldIsNotReplaced},
    Test{.name = "anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt", .body = anEntryKeepsTheAppsIdentifierAndNoOtherEntryTakesIt},
    Test{.name = "removingAnEntryReturnsItAndForgetsIt", .body = removingAnEntryReturnsItAndForgetsIt},
    Test{.name = "aMemberInUseIsNotRemoved", .body = aMemberInUseIsNotRemoved},
    Test{.name = "anArchivedMemberJoinsNoNewExpense", .body = anArchivedMemberJoinsNoNewExpense},
    Test{.name = "theHelpersReadAnEntryAsTheLibraryDoes", .body = theHelpersReadAnEntryAsTheLibraryDoes},
    Test{.name = "aGroupHoldsItsDetailsAndChangesThem", .body = aGroupHoldsItsDetailsAndChangesThem},
    Test{.name = "theSummaryAndTheSharesAddUp", .body = theSummaryAndTheSharesAddUp},
    Test{.name = "eachStrategySettlesEveryBalance", .body = eachStrategySettlesEveryBalance},
    Test{.name = "everyStrategyConvertsToTheGeneratedOneAndBack", .body = everyStrategyConvertsToTheGeneratedOneAndBack},
    Test{.name = "aChangeOfCurrencyConvertsEveryEntry", .body = aChangeOfCurrencyConvertsEveryEntry},
    Test{.name = "aSavedGroupLoadsAsItWasSaved", .body = aSavedGroupLoadsAsItWasSaved},
    Test{.name = "anIdentifierThatIsNotOneIsRefusedByEveryLookup", .body = anIdentifierThatIsNotOneIsRefusedByEveryLookup},
    Test{.name = "everyErrorHoldsWhatItsFailureHolds", .body = everyErrorHoldsWhatItsFailureHolds},
    Test{.name = "everyAsynchronousCallReturnsWhatItsSynchronousFormReturns", .body = everyAsynchronousCallReturnsWhatItsSynchronousFormReturns},
    Test{.name = "anAsynchronousCallReportsTheErrorItsSynchronousFormReturns", .body = anAsynchronousCallReportsTheErrorItsSynchronousFormReturns},
    Test{.name = "aReaderReadsTheGroupUntilItIsReleased", .body = aReaderReadsTheGroupUntilItIsReleased},
    Test{
        .name = "aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas",
        .body = aChangeWaitingBehindAReaderIsCancelledAndLeavesTheGroupAsItWas,
    },
    Test{.name = "everyKindOfAsynchronousCallReportsItsCancellation", .body = everyKindOfAsynchronousCallReportsItsCancellation},
};

/// Runs every test and prints each claim that failed.
///
/// @return How many tests failed
[[nodiscard]] std::size_t runTests() noexcept {
    std::size_t failed = 0;
    for (auto const &test : TESTS) {
        Check check;
        test.body(check);
        for (auto const &claim : check.failed()) {
            std::println(stderr, "{}: {}", test.name, claim);
        }
        failed += check.failed().empty() ? 0 : 1;
    }
    std::println("{} of {} tests passed", TESTS.size() - failed, TESTS.size());
    return failed;
}

} // namespace
} // namespace eranos

int main() noexcept { return 0 == eranos::runTests() ? EXIT_SUCCESS : EXIT_FAILURE; }
