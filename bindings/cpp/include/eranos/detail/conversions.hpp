/// @file eranos/detail/conversions.hpp
/// The conversions between the layer's values and the classes that Diplomat generates, which every part's functions call; eranos.hpp includes them
/// after every part, since they need every type complete.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_DETAIL_CONVERSIONS_HPP
#define ERANOS_DETAIL_CONVERSIONS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/calendar.hpp"
#include "eranos/category.hpp"
#include "eranos/contact.hpp"
#include "eranos/currency_code.hpp"
#include "eranos/error.hpp"
#include "eranos/expense.hpp"
#include "eranos/expense_group.hpp"
#include "eranos/expense_group_reader.hpp"
#include "eranos/failures.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/members.hpp"
#include "eranos/money.hpp"
#include "eranos/ratios.hpp"
#include "eranos/split.hpp"
#include "eranos/totals.hpp"
#include "eranos/transfer.hpp"

#include <cstddef>
#include <cstdint>
#include <expected>
#include <future>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace eranos {

/// What an error holds: its kind, its message, the failure, and what the failure holds.
struct Error::Fields final {
    ErrorKind kind;
    std::string message;
    std::optional<Failure> failure;
    std::optional<Money> actual;
    std::optional<std::int64_t> allowed;
    std::optional<std::int64_t> balances;
    std::optional<CategoryId> category;
    std::optional<std::int64_t> column;
    std::optional<CurrencyCode> currency;
    std::optional<CategoryId> existing;
    std::optional<Money> expectedAmount;
    std::optional<CurrencyCode> expectedCurrency;
    std::optional<std::int64_t> expectedLength;
    std::optional<ExpenseId> expense;
    std::optional<std::vector<ExpenseId>> expenses;
    std::optional<std::int64_t> format;
    std::optional<CurrencyCode> foundCurrency;
    std::optional<std::int64_t> foundLength;
    std::optional<CurrencyCode> left;
    std::optional<std::int64_t> line;
    std::optional<MemberId> member;
    std::optional<IbanReason> reason;
    std::optional<CurrencyCode> right;
    std::optional<std::int64_t> totalBasisPoints;
    std::optional<TransferId> transfer;
    std::optional<std::vector<TransferId>> transfers;
};

inline ErrorKind Error::kind() const noexcept { return fields->kind; }

inline std::string const &Error::message() const noexcept { return fields->message; }

inline std::optional<Failure> Error::failure() const noexcept { return fields->failure; }

inline std::optional<Money> const &Error::actual() const noexcept { return fields->actual; }

inline std::optional<std::int64_t> const &Error::allowed() const noexcept { return fields->allowed; }

inline std::optional<std::int64_t> const &Error::balances() const noexcept { return fields->balances; }

inline std::optional<CategoryId> const &Error::category() const noexcept { return fields->category; }

inline std::optional<std::int64_t> const &Error::column() const noexcept { return fields->column; }

inline std::optional<CurrencyCode> const &Error::currency() const noexcept { return fields->currency; }

inline std::optional<CategoryId> const &Error::existing() const noexcept { return fields->existing; }

inline std::optional<Money> const &Error::expectedAmount() const noexcept { return fields->expectedAmount; }

inline std::optional<CurrencyCode> const &Error::expectedCurrency() const noexcept { return fields->expectedCurrency; }

inline std::optional<std::int64_t> const &Error::expectedLength() const noexcept { return fields->expectedLength; }

inline std::optional<ExpenseId> const &Error::expense() const noexcept { return fields->expense; }

inline std::optional<std::vector<ExpenseId>> const &Error::expenses() const noexcept { return fields->expenses; }

inline std::optional<std::int64_t> const &Error::format() const noexcept { return fields->format; }

inline std::optional<CurrencyCode> const &Error::foundCurrency() const noexcept { return fields->foundCurrency; }

inline std::optional<std::int64_t> const &Error::foundLength() const noexcept { return fields->foundLength; }

inline std::optional<CurrencyCode> const &Error::left() const noexcept { return fields->left; }

inline std::optional<std::int64_t> const &Error::line() const noexcept { return fields->line; }

inline std::optional<MemberId> const &Error::member() const noexcept { return fields->member; }

inline std::optional<IbanReason> const &Error::reason() const noexcept { return fields->reason; }

inline std::optional<CurrencyCode> const &Error::right() const noexcept { return fields->right; }

inline std::optional<std::int64_t> const &Error::totalBasisPoints() const noexcept { return fields->totalBasisPoints; }

inline std::optional<TransferId> const &Error::transfer() const noexcept { return fields->transfer; }

inline std::optional<std::vector<TransferId>> const &Error::transfers() const noexcept { return fields->transfers; }

inline ExpenseGroupId ExpenseGroupId::generate() noexcept { return ExpenseGroupId(detail::EranosIdentifier::generate()); }

inline std::expected<ExpenseGroupId, Error> ExpenseGroupId::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosIdentifier::parse(text)).transform([](std::string identifier) noexcept {
        return ExpenseGroupId(std::move(identifier));
    });
}

inline MemberId MemberId::generate() noexcept { return MemberId(detail::EranosIdentifier::generate()); }

inline std::expected<MemberId, Error> MemberId::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosIdentifier::parse(text)).transform([](std::string identifier) noexcept {
        return MemberId(std::move(identifier));
    });
}

inline CategoryId CategoryId::generate() noexcept { return CategoryId(detail::EranosIdentifier::generate()); }

inline std::expected<CategoryId, Error> CategoryId::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosIdentifier::parse(text)).transform([](std::string identifier) noexcept {
        return CategoryId(std::move(identifier));
    });
}

inline ExpenseId ExpenseId::generate() noexcept { return ExpenseId(detail::EranosIdentifier::generate()); }

inline std::expected<ExpenseId, Error> ExpenseId::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosIdentifier::parse(text)).transform([](std::string identifier) noexcept {
        return ExpenseId(std::move(identifier));
    });
}

inline TransferId TransferId::generate() noexcept { return TransferId(detail::EranosIdentifier::generate()); }

inline std::expected<TransferId, Error> TransferId::parse(std::string_view const text) noexcept {
    return detail::expectedOf(detail::EranosIdentifier::parse(text)).transform([](std::string identifier) noexcept {
        return TransferId(std::move(identifier));
    });
}

} // namespace eranos

namespace eranos::detail {

/// The value @p result holds, or the error it holds.
template <typename Value>
[[nodiscard]] inline std::expected<Value, Error> expectedOf(diplomat::result<Value, std::unique_ptr<EranosError>> &&result) noexcept {
    if (result.is_ok()) {
        return std::move(std::move(result).ok().value()); // NOLINT(bugprone-unchecked-optional-access): is_ok() says which side it holds.
    }
    return std::unexpected(
        errorOf(*std::move(result).err().value())); // NOLINT(bugprone-unchecked-optional-access): is_ok() says which side it holds.
}

/// Nothing when @p result, of a call that returns nothing, succeeded, or the error it holds.
[[nodiscard]] inline std::expected<void, Error> expectedOf(diplomat::result<std::monostate, std::unique_ptr<EranosError>> &&result) noexcept {
    return expectedOf<std::monostate>(std::move(result)).transform([](std::monostate const /* value */) noexcept {});
}

/// The value of @p result, which holds no error: the call fails only for a value that the layer's types rule out, and its caller says which.
template <typename Value> [[nodiscard]] inline Value known(diplomat::result<Value, std::unique_ptr<EranosError>> &&result) noexcept {
    return std::move(std::move(result).ok().value()); // NOLINT(bugprone-unchecked-optional-access): the caller says why it holds a value.
}

/// The value of @p result, of a synchronous group call that takes nothing the library could refuse: the call waits for the group's lock, so it
/// fails only where it cannot wait, on wasm32, for which the C API does not build.
template <typename Value> [[nodiscard]] inline Value waited(diplomat::result<Value, std::unique_ptr<EranosError>> &&result) noexcept {
    return known(std::move(result));
}

/// @p record, once @p set has given it @p value when @p value is present, or the error that @p set reports.
template <typename Record, typename Value, typename Set>
[[nodiscard]] inline std::expected<std::unique_ptr<Record>, Error> setIfPresent(std::unique_ptr<Record> record, std::optional<Value> const &value,
                                                                                Set const &set) noexcept {
    if (!value) {
        return record;
    }
    return expectedOf(set(*record, *value)).transform([&record] noexcept { return std::move(record); });
}

/// The generated list @p List of @p records, or the error that the conversion of the first record it refuses reports.
template <typename List, typename Record>
[[nodiscard]] inline std::expected<std::unique_ptr<List>, Error> listOf(std::vector<Record> const &records) noexcept {
    std::expected<std::unique_ptr<List>, Error> result = List::create();
    for (auto const &record : records) {
        result = std::move(result).and_then([&record](std::unique_ptr<List> list) noexcept {
            return toDetail(record).transform([&list](auto const &element) noexcept {
                list->push(*element);
                return std::move(list);
            });
        });
    }
    return result;
}

/// @p text, absent when empty: the group holds no blank description and no blank language.
[[nodiscard]] inline constexpr std::optional<std::string> present(std::string text) noexcept {
    if (text.empty()) {
        return std::nullopt;
    }
    return text;
}

/// The identifier of @p text, if present.
template <typename Identifier> [[nodiscard]] inline constexpr std::optional<Identifier> optionalId(std::optional<std::string> text) noexcept {
    if (!text) {
        return std::nullopt;
    }
    return Identifier(std::move(*text));
}

/// The value that @p make makes of @p text, if present.
template <typename Value>
[[nodiscard]] inline constexpr std::optional<Value> optionalText(std::optional<std::string> text,
                                                                 Value (*const make)(std::string) noexcept) noexcept {
    if (!text) {
        return std::nullopt;
    }
    return make(std::move(*text));
}

/// The value of the generated object @p value, if present.
template <typename Value, typename Detail> [[nodiscard]] inline std::optional<Value> optionalRecord(std::unique_ptr<Detail> const &value) noexcept {
    if (!value) {
        return std::nullopt;
    }
    return fromDetail(*value);
}

/// The identifiers of the texts of @p list.
template <typename Identifier> [[nodiscard]] inline std::vector<Identifier> identifiers(EranosTextList const &list) noexcept {
    std::vector<Identifier> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.emplace_back(*list.get(index)); // NOLINT(bugprone-unchecked-optional-access): the index is below len().
    }
    return result;
}

/// The success of an asynchronous call that returns nothing, whose outcome @p outcome holds no error.
[[nodiscard]] inline constexpr std::expected<void, Error> finished(EranosUnitOutcome const & /* outcome */) noexcept { return {}; }

/// Completes @p promise with the value that @p convert makes of @p outcome, or with the error it holds. A completion runs on the library's thread,
/// in a callback of its C API, which no exception may leave, so a conversion that runs out of memory ends the program, as the library's own
/// failure to allocate does.
template <typename Value, typename Outcome, typename Convert>
inline void complete(std::promise<std::expected<Value, Error>> &promise, Outcome &outcome, Convert const &convert) noexcept {
    if (outcome.is_ok()) {
        promise.set_value(convert(outcome));
    } else {
        promise.set_value(std::unexpected(errorOf(*outcome.error())));
    }
}

/// A future of the asynchronous call that @p start makes with the callback it is given, which completes the future with the value that
/// @p convert makes of the call's outcome, or with the error the outcome holds.
template <typename Value, typename Start, typename Convert>
[[nodiscard]] inline std::future<std::expected<Value, Error>> launch(Start const &start, Convert convert) noexcept {
    auto const promise = std::make_shared<std::promise<std::expected<Value, Error>>>();
    auto future = promise->get_future();
    start([promise, convert = std::move(convert)](auto const outcome) noexcept { complete(*promise, *outcome, convert); });
    return future;
}

/// A future of the asynchronous call that @p start makes with the generated form of its argument, @p argument, and the callback it is given, as
/// the other launch() does; when the argument's conversion failed, a future of its error, and no call.
template <typename Value, typename Argument, typename Start, typename Convert>
[[nodiscard]] inline std::future<std::expected<Value, Error>> launch(std::expected<Argument, Error> const &argument, Start const &start,
                                                                     Convert convert) noexcept {
    if (argument) {
        return launch<Value>([&argument, &start](auto callback) noexcept { start(*argument, std::move(callback)); }, std::move(convert));
    }
    std::promise<std::expected<Value, Error>> promise;
    promise.set_value(std::unexpected(argument.error()));
    return promise.get_future();
}

/// The currency of @p details, which the library made: the currency keeps them, so an amount in it crosses with them.
[[nodiscard]] inline CurrencyCode currencyOf(std::unique_ptr<EranosCurrencyDetails> details) noexcept {
    std::shared_ptr<EranosCurrencyDetails const> const shared = std::move(details);
    return CurrencyCode({
        .code = shared->code(),
        .name = shared->name(),
        .numericCode = shared->numeric_code(),
        .decimalPlaces = shared->decimal_places(),
        .details = shared,
    });
}

/// The currency of @p details, if present.
[[nodiscard]] inline std::optional<CurrencyCode> optionalCurrency(std::unique_ptr<EranosCurrencyDetails> details) noexcept {
    if (!details) {
        return std::nullopt;
    }
    return currencyOf(std::move(details));
}

/// The amount of @p value, which the library made.
[[nodiscard]] inline Money fromDetail(EranosMoney const &value) noexcept { return {value.minor_units(), currencyOf(value.currency())}; }

/// The generated amount of @p value, with the details its currency keeps.
[[nodiscard]] inline std::unique_ptr<EranosMoney> toDetail(Money const &value) noexcept {
    return EranosMoney::create(value.minorUnits, *value.currency.fields.details);
}

/// The rate written @p text, which the library wrote.
[[nodiscard]] inline constexpr ExchangeRate exchangeRateOf(std::string text) noexcept { return ExchangeRate(std::move(text)); }

/// The date of @p date, which the library checked.
[[nodiscard]] inline constexpr CalendarDate calendarDateOf(EranosCalendarDate const date) noexcept {
    return CalendarDate({.year = date.year, .month = date.month, .day = date.day});
}

/// The generated date of @p date.
[[nodiscard]] inline constexpr EranosCalendarDate toDetail(CalendarDate const date) noexcept {
    return EranosCalendarDate{.year = date.year(), .month = date.month(), .day = date.day()};
}

/// The time of @p time, which the library checked.
[[nodiscard]] inline constexpr TimeOfDay timeOfDayOf(EranosTimeOfDay const time) noexcept {
    return TimeOfDay({.hour = time.hour, .minute = time.minute});
}

/// The generated time of @p time.
[[nodiscard]] inline constexpr EranosTimeOfDay toDetail(TimeOfDay const time) noexcept {
    return EranosTimeOfDay{.hour = time.hour(), .minute = time.minute()};
}

/// The time of @p time, if present.
[[nodiscard]] inline constexpr std::optional<TimeOfDay> optionalTime(std::optional<EranosTimeOfDay> const time) noexcept {
    if (!time) {
        return std::nullopt;
    }
    return timeOfDayOf(*time);
}

/// The email written @p text, which the library wrote.
[[nodiscard]] inline constexpr Email emailOf(std::string text) noexcept { return Email(std::move(text)); }

/// The phone number of @p value, in both the forms the library wrote it in.
[[nodiscard]] inline PhoneNumber fromDetail(EranosPhoneNumber const &value) noexcept {
    return PhoneNumber({.e164 = value.e164(), .international = value.international()});
}

/// The IBAN of @p value, with the country code the library derived from it.
[[nodiscard]] inline Iban fromDetail(EranosIban const &value) noexcept { return Iban({.iban = value.iban(), .countryCode = value.country_code()}); }

/// The IBAN that @p value, account details of the kind Iban, holds, with the country code the library derived from it.
[[nodiscard]] inline Iban ibanOf(EranosPaymentAccountDetails const &value) noexcept {
    // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says both values are present.
    return Iban({.iban = value.iban().value(), .countryCode = value.country_code().value()});
}

/// The username written @p text, which the library wrote.
[[nodiscard]] inline constexpr Username usernameOf(std::string text) noexcept { return Username(std::move(text)); }

/// The language written @p text, which the library wrote.
[[nodiscard]] inline constexpr Language languageOf(std::string text) noexcept { return Language(std::move(text)); }

/// The error kind of @p value. The two enumerations list their 17 values in one order, which the generator of both
/// writes, so a value converts by its number; a library failure that no call can provoke, such as a settlement out of memory, converts
/// as every other does.
[[nodiscard]] inline ErrorKind fromDetail(EranosErrorKind const value) noexcept { return static_cast<ErrorKind>(std::to_underlying(value.AsFFI())); }

/// The failure of @p value. The two enumerations list their 58 values in one order, which the generator of both
/// writes, so a value converts by its number; a library failure that no call can provoke, such as a settlement out of memory, converts
/// as every other does.
[[nodiscard]] inline Failure fromDetail(EranosFailure const value) noexcept { return static_cast<Failure>(std::to_underlying(value.AsFFI())); }

/// The iban reason of @p value. The two enumerations list their 6 values in one order, which the generator of both
/// writes, so a value converts by its number; a library failure that no call can provoke, such as a settlement out of memory, converts
/// as every other does.
[[nodiscard]] inline IbanReason fromDetail(EranosIbanReason const value) noexcept {
    return static_cast<IbanReason>(std::to_underlying(value.AsFFI()));
}

/// The error of @p detail, with everything the failure holds.
[[nodiscard]] inline Error errorOf(EranosError const &detail) noexcept {
    Error::Fields fields{};
    fields.kind = fromDetail(detail.kind());
    fields.message = detail.message();
    fields.failure = detail.failure().transform([](EranosFailure const value) noexcept { return fromDetail(value); });
    fields.actual = optionalRecord<Money>(detail.actual());
    fields.allowed = detail.allowed();
    fields.balances = detail.balances();
    fields.category = optionalId<CategoryId>(detail.category());
    fields.column = detail.column();
    fields.currency = optionalCurrency(detail.currency());
    fields.existing = optionalId<CategoryId>(detail.existing());
    fields.expectedAmount = optionalRecord<Money>(detail.expected_amount());
    fields.expectedCurrency = optionalCurrency(detail.expected_currency());
    fields.expectedLength = detail.expected_length();
    fields.expense = optionalId<ExpenseId>(detail.expense());
    auto const expenses = detail.expenses();
    fields.expenses = expenses ? std::optional<std::vector<ExpenseId>>(identifiers<ExpenseId>(*expenses)) : std::nullopt;
    fields.format = detail.format();
    fields.foundCurrency = optionalCurrency(detail.found_currency());
    fields.foundLength = detail.found_length();
    fields.left = optionalCurrency(detail.left());
    fields.line = detail.line();
    fields.member = optionalId<MemberId>(detail.member());
    fields.reason = detail.reason().transform([](EranosIbanReason const value) noexcept { return fromDetail(value); });
    fields.right = optionalCurrency(detail.right());
    fields.totalBasisPoints = detail.total_basis_points();
    fields.transfer = optionalId<TransferId>(detail.transfer());
    auto const transfers = detail.transfers();
    fields.transfers = transfers ? std::optional<std::vector<TransferId>>(identifiers<TransferId>(*transfers)) : std::nullopt;
    return Error(std::make_shared<Error::Fields const>(std::move(fields)));
}

/// The generated list of the identifiers @p members, or the error of the first that is not text.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosTextList>, Error> toDetail(std::vector<MemberId> const &members) noexcept {
    std::expected<std::unique_ptr<EranosTextList>, Error> result = EranosTextList::create();
    for (auto const &member : members) {
        result = std::move(result).and_then([&member](std::unique_ptr<EranosTextList> list) noexcept {
            return expectedOf(list->push(member.value())).transform([&list] noexcept { return std::move(list); });
        });
    }
    return result;
}

/// The record of @p value.
[[nodiscard]] inline MemberParts fromDetail(EranosMemberParts const &value) noexcept { return {MemberId(value.member()), Parts{value.hundredths()}}; }

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberParts>, Error> toDetail(MemberParts const &value) noexcept {
    return expectedOf(EranosMemberParts::create(value.member.value(), value.parts.hundredths));
}

/// The record of @p value.
[[nodiscard]] inline MemberPercentage fromDetail(EranosMemberPercentage const &value) noexcept {
    return {MemberId(value.member()), Percentage{value.basis_points()}};
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberPercentage>, Error> toDetail(MemberPercentage const &value) noexcept {
    return expectedOf(EranosMemberPercentage::create(value.member.value(), value.percentage.basisPoints));
}

/// The record of @p value.
[[nodiscard]] inline MemberAmount fromDetail(EranosMemberAmount const &value) noexcept {
    return {MemberId(value.member()), detail::fromDetail(*value.amount())};
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberAmount>, Error> toDetail(MemberAmount const &value) noexcept {
    return expectedOf(EranosMemberAmount::create(value.member.value(), *detail::toDetail(value.amount)));
}

/// The alternative of @p value.
[[nodiscard]] inline Split fromDetail(EranosSplit const &value) noexcept {
    switch (value.kind()) {
    case EranosSplitKind::Equal:
        return EqualSplit{detail::identifiers<MemberId>(*value.members())};
    case EranosSplitKind::Parts:
        return PartsSplit{detail::fromDetail(*value.parts_shares())};
    case EranosSplitKind::Percentages:
        return PercentagesSplit{detail::fromDetail(*value.percentages_shares())};
    case EranosSplitKind::Amounts:
        break;
    }
    return AmountsSplit{detail::fromDetail(*value.amounts_shares())};
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(EqualSplit const &value) noexcept {
    return detail::toDetail(value.members).transform([](std::unique_ptr<EranosTextList> const &members) noexcept {
        return EranosSplit::create_equal(*members);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(PartsSplit const &value) noexcept {
    return listOf<EranosMemberPartsList>(value.shares).transform([](std::unique_ptr<EranosMemberPartsList> const &shares) noexcept {
        return EranosSplit::create_parts(*shares);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(PercentagesSplit const &value) noexcept {
    return listOf<EranosMemberPercentageList>(value.shares).transform([](std::unique_ptr<EranosMemberPercentageList> const &shares) noexcept {
        return EranosSplit::create_percentages(*shares);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(AmountsSplit const &value) noexcept {
    return listOf<EranosMemberAmountList>(value.shares).transform([](std::unique_ptr<EranosMemberAmountList> const &shares) noexcept {
        return EranosSplit::create_amounts(*shares);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(Split const &value) noexcept {
    return std::visit([](auto const &alternative) noexcept { return toDetail(alternative); }, value);
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberParts> fromDetail(EranosMemberPartsList const &list) noexcept {
    std::vector<MemberParts> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberPercentage> fromDetail(EranosMemberPercentageList const &list) noexcept {
    std::vector<MemberPercentage> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberAmount> fromDetail(EranosMemberAmountList const &list) noexcept {
    std::vector<MemberAmount> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The record of @p value.
[[nodiscard]] inline ExpenseItem fromDetail(EranosExpenseItem const &value) noexcept {
    ExpenseItem result(value.name(), detail::fromDetail(*value.unit_price()), detail::fromDetail(*value.split()));
    result.quantity = value.quantity();
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseItem>, Error> toDetail(ExpenseItem const &value) noexcept {
    return detail::toDetail(value.split)
        .and_then([&value](std::unique_ptr<EranosSplit> const &split) noexcept {
            return expectedOf(EranosExpenseItem::create(value.name, *detail::toDetail(value.unitPrice), *split));
        })
        .transform([&value](std::unique_ptr<EranosExpenseItem> result) noexcept {
            result->set_quantity(value.quantity);
            return result;
        });
}

/// The alternative of @p value.
[[nodiscard]] inline TipValue fromDetail(EranosTipValue const &value) noexcept {
    switch (value.kind()) {
    case EranosTipValueKind::Percentage:
        // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says the value is present.
        return PercentageTipValue{Percentage{value.basis_points().value()}};
    case EranosTipValueKind::Fixed:
        break;
    }
    return FixedTipValue{detail::fromDetail(*value.amount())};
}

/// The generated tip value of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(PercentageTipValue const &value) noexcept {
    return EranosTipValue::create_percentage(value.percentage.basisPoints);
}

/// The generated tip value of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(FixedTipValue const &value) noexcept {
    return EranosTipValue::create_fixed(*detail::toDetail(value.amount));
}

/// The generated tip value of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(TipValue const &value) noexcept {
    return std::visit([](auto const &alternative) noexcept { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline TipRule fromDetail(EranosTipRule const &value) noexcept {
    return {detail::fromDetail(*value.value()), detail::fromDetail(value.distribution())};
}

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTipRule> toDetail(TipRule const &value) noexcept {
    return EranosTipRule::create(*detail::toDetail(value.value), detail::toDetail(value.distribution));
}

/// The alternative of @p value.
[[nodiscard]] inline Cost fromDetail(EranosCost const &value) noexcept {
    switch (value.kind()) {
    case EranosCostKind::Local:
        return LocalCost{detail::fromDetail(*value.amount())};
    case EranosCostKind::Converted:
        break;
    }
    // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says the value is present.
    return ConvertedCost{.amount = detail::fromDetail(*value.amount()), .rate = detail::exchangeRateOf(value.rate().value())};
}

/// The generated cost of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(LocalCost const &value) noexcept {
    return EranosCost::create_local(*detail::toDetail(value.amount));
}

/// The generated cost of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(ConvertedCost const &value) noexcept {
    // A rate is text the library wrote.
    return known(EranosCost::create_converted(*detail::toDetail(value.amount), value.rate.toString()));
}

/// The generated cost of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(Cost const &value) noexcept {
    return std::visit([](auto const &alternative) noexcept { return toDetail(alternative); }, value);
}

/// The alternative of @p value.
[[nodiscard]] inline ExpenseSplit fromDetail(EranosExpenseSplit const &value) noexcept {
    switch (value.kind()) {
    case EranosExpenseSplitKind::Whole:
        return WholeExpenseSplit{detail::fromDetail(*value.split())};
    case EranosExpenseSplitKind::Itemised:
        break;
    }
    return ItemisedExpenseSplit{detail::fromDetail(*value.items())};
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(WholeExpenseSplit const &value) noexcept {
    return detail::toDetail(value.split).transform([](std::unique_ptr<EranosSplit> const &split) noexcept {
        return EranosExpenseSplit::create_whole(*split);
    });
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ItemisedExpenseSplit const &value) noexcept {
    return listOf<EranosExpenseItemList>(value.items).transform([](std::unique_ptr<EranosExpenseItemList> const &items) noexcept {
        return EranosExpenseSplit::create_itemised(*items);
    });
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ExpenseSplit const &value) noexcept {
    return std::visit([](auto const &alternative) noexcept { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline Expense fromDetail(EranosExpense const &value) noexcept {
    Expense result(ExpenseId(value.id()), value.name(), detail::fromDetail(*value.cost()), detail::fromDetail(*value.payers()),
                   detail::fromDetail(*value.split()), detail::calendarDateOf(value.date()));
    result.tip = detail::fromDetail(*value.tip());
    result.time = detail::optionalTime(value.time());
    result.category = CategoryId(value.category());
    result.description = value.description();
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpense>, Error> toDetail(Expense const &value) noexcept {
    return detail::toDetail(value.payers)
        .and_then([&value](std::unique_ptr<EranosSplit> const &payers) noexcept {
            return detail::toDetail(value.split).and_then([&value, &payers](std::unique_ptr<EranosExpenseSplit> const &split) noexcept {
                return expectedOf(EranosExpense::create(value.id.value(), value.name, *detail::toDetail(value.cost), *payers, *split,
                                                        detail::toDetail(value.date)));
            });
        })
        .and_then([&value](std::unique_ptr<EranosExpense> result) noexcept {
            result->set_tip(*detail::toDetail(value.tip));
            if (value.time) {
                result->set_time(detail::toDetail(*value.time));
            }
            return expectedOf(result->set_category(value.category.value())).transform([&result] noexcept { return std::move(result); });
        })
        .and_then([&value](std::unique_ptr<EranosExpense> result) noexcept {
            return setIfPresent(std::move(result), value.description,
                                [](EranosExpense &record, std::string const &text) noexcept { return record.set_description(text); });
        });
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Expense> fromDetail(EranosExpenseList const &list) noexcept {
    std::vector<Expense> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<ExpenseItem> fromDetail(EranosExpenseItemList const &list) noexcept {
    std::vector<ExpenseItem> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The value of @p value.
[[nodiscard]] inline constexpr TipDistribution fromDetail(EranosTipDistribution const value) noexcept {
    switch (value) {
    case EranosTipDistribution::Proportional:
        return TipDistribution::PROPORTIONAL;
    case EranosTipDistribution::Equal:
        break;
    }
    return TipDistribution::EQUAL;
}

/// The generated value of @p value.
[[nodiscard]] inline constexpr EranosTipDistribution toDetail(TipDistribution const value) noexcept {
    switch (value) {
    case TipDistribution::PROPORTIONAL:
        return EranosTipDistribution::Proportional;
    case TipDistribution::EQUAL:
        break;
    }
    return EranosTipDistribution::Equal;
}

/// The record of @p value.
[[nodiscard]] inline Transfer fromDetail(EranosTransfer const &value) noexcept {
    Transfer result(TransferId(value.id()), MemberId(value.from()), MemberId(value.to()), detail::fromDetail(*value.cost()),
                    detail::calendarDateOf(value.date()));
    result.time = detail::optionalTime(value.time());
    result.note = value.note();
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosTransfer>, Error> toDetail(Transfer const &value) noexcept {
    return expectedOf(EranosTransfer::create(value.id.value(), value.from.value(), value.to.value(), *detail::toDetail(value.cost),
                                             detail::toDetail(value.date)))
        .and_then([&value](std::unique_ptr<EranosTransfer> result) noexcept {
            if (value.time) {
                result->set_time(detail::toDetail(*value.time));
            }
            return setIfPresent(std::move(result), value.note,
                                [](EranosTransfer &record, std::string const &text) noexcept { return record.set_note(text); });
        });
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Transfer> fromDetail(EranosTransferList const &list) noexcept {
    std::vector<Transfer> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The alternative of @p value.
[[nodiscard]] inline PaymentAccountDetails fromDetail(EranosPaymentAccountDetails const &value) noexcept {
    switch (value.kind()) {
    case EranosPaymentAccountDetailsKind::Iban:
        return IbanPaymentAccountDetails{detail::ibanOf(value)};
    case EranosPaymentAccountDetailsKind::Username:
        break;
    }
    // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says the value is present.
    return UsernamePaymentAccountDetails{detail::usernameOf(value.username().value())};
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(IbanPaymentAccountDetails const &value) noexcept {
    // An IBAN is one the library parsed, which it parses again into the same IBAN and country code.
    return known(EranosPaymentAccountDetails::create_iban(value.iban.toString()));
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(UsernamePaymentAccountDetails const &value) noexcept {
    // A username is text the library wrote.
    return known(EranosPaymentAccountDetails::create_username(value.username.toString()));
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(PaymentAccountDetails const &value) noexcept {
    return std::visit([](auto const &alternative) noexcept { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline PaymentAccount fromDetail(EranosPaymentAccount const &value) noexcept {
    return {value.name(), detail::fromDetail(*value.details())};
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosPaymentAccount>, Error> toDetail(PaymentAccount const &value) noexcept {
    return expectedOf(EranosPaymentAccount::create(value.name, *detail::toDetail(value.details)));
}

/// The record of @p value.
[[nodiscard]] inline Member fromDetail(EranosMember const &value) noexcept {
    Member result(MemberId(value.id()), value.first_name());
    result.lastName = value.last_name();
    result.email = detail::optionalText<Email>(value.email(), detail::emailOf);
    result.phone = detail::optionalRecord<PhoneNumber>(value.phone());
    result.accounts = detail::fromDetail(*value.accounts());
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMember>, Error> toDetail(Member const &value) noexcept {
    return listOf<EranosPaymentAccountList>(value.accounts)
        .and_then([&value](std::unique_ptr<EranosPaymentAccountList> const &accounts) noexcept {
            return expectedOf(EranosMember::create(value.id.value(), value.firstName))
                .transform([&accounts](std::unique_ptr<EranosMember> result) noexcept {
                    result->set_accounts(*accounts);
                    return result;
                });
        })
        .and_then([&value](std::unique_ptr<EranosMember> result) noexcept {
            return setIfPresent(std::move(result), value.lastName,
                                [](EranosMember &record, std::string const &text) noexcept { return record.set_last_name(text); });
        })
        .and_then([&value](std::unique_ptr<EranosMember> result) noexcept {
            return setIfPresent(std::move(result), value.email,
                                [](EranosMember &record, Email const &email) noexcept { return record.set_email(email.toString()); });
        })
        .and_then([&value](std::unique_ptr<EranosMember> result) noexcept {
            return setIfPresent(std::move(result), value.phone,
                                [](EranosMember &record, PhoneNumber const &phone) noexcept { return record.set_phone(phone.toString()); });
        });
}

/// The records of @p list.
[[nodiscard]] inline std::vector<PaymentAccount> fromDetail(EranosPaymentAccountList const &list) noexcept {
    std::vector<PaymentAccount> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Member> fromDetail(EranosMemberList const &list) noexcept {
    std::vector<Member> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The record of @p value.
[[nodiscard]] inline Category fromDetail(EranosCategory const &value) noexcept { return {CategoryId(value.id()), value.name()}; }

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosCategory>, Error> toDetail(Category const &value) noexcept {
    return expectedOf(EranosCategory::create(value.id.value(), value.name));
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Category> fromDetail(EranosCategoryList const &list) noexcept {
    std::vector<Category> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The alternative of @p value.
[[nodiscard]] inline Balance fromDetail(EranosBalance const &value) noexcept {
    switch (value.kind()) {
    case EranosBalanceKind::Owes:
        return OwesBalance{detail::fromDetail(*value.amount())};
    case EranosBalanceKind::Settled:
        return SettledBalance{};
    case EranosBalanceKind::GetsBack:
        break;
    }
    return GetsBackBalance{detail::fromDetail(*value.amount())};
}

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(OwesBalance const &value) noexcept {
    return EranosBalance::create_owes(*detail::toDetail(value.amount));
}

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(SettledBalance const & /* value */) noexcept { return EranosBalance::create_settled(); }

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(GetsBackBalance const &value) noexcept {
    return EranosBalance::create_gets_back(*detail::toDetail(value.amount));
}

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(Balance const &value) noexcept {
    return std::visit([](auto const &alternative) noexcept { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline MemberSummary fromDetail(EranosMemberSummary const &value) noexcept {
    return {MemberId(value.member()),
            detail::fromDetail(*value.paid()),
            detail::fromDetail(*value.share()),
            detail::fromDetail(*value.sent()),
            detail::fromDetail(*value.received()),
            detail::fromDetail(*value.net()),
            detail::fromDetail(*value.balance())};
}

/// The record of @p value.
[[nodiscard]] inline CategoryTotal fromDetail(EranosCategoryTotal const &value) noexcept {
    return {CategoryId(value.category()), detail::fromDetail(*value.total())};
}

/// The record of @p value.
[[nodiscard]] inline MemberCategoryTotals fromDetail(EranosMemberCategoryTotals const &value) noexcept {
    return {MemberId(value.member()), detail::fromDetail(*value.totals())};
}

/// The record of @p value.
[[nodiscard]] inline Summary fromDetail(EranosSummary const &value) noexcept {
    return {detail::fromDetail(*value.total_spent()), detail::fromDetail(*value.members()), detail::fromDetail(*value.category_totals()),
            detail::fromDetail(*value.member_category_totals())};
}

/// The record of @p value.
[[nodiscard]] inline MemberShare fromDetail(EranosMemberShare const &value) noexcept {
    return {MemberId(value.member()), detail::fromDetail(*value.paid()), detail::fromDetail(*value.share())};
}

/// The record of @p value.
[[nodiscard]] inline ExpenseShares fromDetail(EranosExpenseShares const &value) noexcept {
    return {detail::fromDetail(*value.members()), detail::fromDetail(*value.total()), detail::fromDetail(*value.tip())};
}

/// The record of @p value.
[[nodiscard]] inline Payment fromDetail(EranosPayment const &value) noexcept {
    return {MemberId(value.from()), MemberId(value.to()), detail::fromDetail(*value.amount())};
}

/// The record of @p value.
[[nodiscard]] inline SettlementPlan fromDetail(EranosSettlementPlan const &value) noexcept {
    return {detail::fromDetail(*value.payments()), detail::fromDetail(value.strategy())};
}

/// The records of @p list.
[[nodiscard]] inline std::vector<CategoryTotal> fromDetail(EranosCategoryTotalList const &list) noexcept {
    std::vector<CategoryTotal> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberCategoryTotals> fromDetail(EranosMemberCategoryTotalsList const &list) noexcept {
    std::vector<MemberCategoryTotals> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberShare> fromDetail(EranosMemberShareList const &list) noexcept {
    std::vector<MemberShare> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberSummary> fromDetail(EranosMemberSummaryList const &list) noexcept {
    std::vector<MemberSummary> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Payment> fromDetail(EranosPaymentList const &list) noexcept {
    std::vector<Payment> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The value of @p value.
[[nodiscard]] inline constexpr SettlementStrategy fromDetail(EranosSettlementStrategy const value) noexcept {
    switch (value) {
    case EranosSettlementStrategy::Adaptive:
        return SettlementStrategy::ADAPTIVE;
    case EranosSettlementStrategy::MinimumTransfers:
        return SettlementStrategy::MINIMUM_TRANSFERS;
    case EranosSettlementStrategy::Greedy:
        return SettlementStrategy::GREEDY;
    case EranosSettlementStrategy::Direct:
        break;
    }
    return SettlementStrategy::DIRECT;
}

/// The generated value of @p value.
[[nodiscard]] inline constexpr EranosSettlementStrategy toDetail(SettlementStrategy const value) noexcept {
    switch (value) {
    case SettlementStrategy::ADAPTIVE:
        return EranosSettlementStrategy::Adaptive;
    case SettlementStrategy::MINIMUM_TRANSFERS:
        return EranosSettlementStrategy::MinimumTransfers;
    case SettlementStrategy::GREEDY:
        return EranosSettlementStrategy::Greedy;
    case SettlementStrategy::DIRECT:
        break;
    }
    return EranosSettlementStrategy::Direct;
}

/// The generated cancellation of @p cancellation, or none.
[[nodiscard]] inline constexpr EranosCancellation const *handleOf(Cancellation const *const cancellation) noexcept {
    return nullptr == cancellation ? nullptr : cancellation->handle.get();
}

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRate> toDetail(CurrencyRate const &value) noexcept {
    // A currency's code and a rate are text the library wrote.
    return known(EranosCurrencyRate::create(value.currency.code(), value.rate.toString()));
}

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRateList> toDetail(std::span<CurrencyRate const> const records) noexcept {
    auto result = EranosCurrencyRateList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
    }
    return result;
}

} // namespace eranos::detail

#endif // ERANOS_DETAIL_CONVERSIONS_HPP
