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
#include "eranos/currency.hpp"
#include "eranos/error.hpp"
#include "eranos/expense.hpp"
#include "eranos/failures.hpp"
#include "eranos/group.hpp"
#include "eranos/group_reader.hpp"
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
    std::optional<std::string> currency;
    std::optional<CategoryId> existing;
    std::optional<Money> expectedAmount;
    std::optional<std::string> expectedCurrency;
    std::optional<std::int64_t> expectedLength;
    std::optional<ExpenseId> expense;
    std::optional<std::vector<ExpenseId>> expenses;
    std::optional<std::int64_t> format;
    std::optional<std::string> foundCurrency;
    std::optional<std::int64_t> foundLength;
    std::optional<std::string> left;
    std::optional<std::int64_t> line;
    std::optional<MemberId> member;
    std::optional<IbanReason> reason;
    std::optional<std::string> right;
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

inline std::optional<std::string> const &Error::currency() const noexcept { return fields->currency; }

inline std::optional<CategoryId> const &Error::existing() const noexcept { return fields->existing; }

inline std::optional<Money> const &Error::expectedAmount() const noexcept { return fields->expectedAmount; }

inline std::optional<std::string> const &Error::expectedCurrency() const noexcept { return fields->expectedCurrency; }

inline std::optional<std::int64_t> const &Error::expectedLength() const noexcept { return fields->expectedLength; }

inline std::optional<ExpenseId> const &Error::expense() const noexcept { return fields->expense; }

inline std::optional<std::vector<ExpenseId>> const &Error::expenses() const noexcept { return fields->expenses; }

inline std::optional<std::int64_t> const &Error::format() const noexcept { return fields->format; }

inline std::optional<std::string> const &Error::foundCurrency() const noexcept { return fields->foundCurrency; }

inline std::optional<std::int64_t> const &Error::foundLength() const noexcept { return fields->foundLength; }

inline std::optional<std::string> const &Error::left() const noexcept { return fields->left; }

inline std::optional<std::int64_t> const &Error::line() const noexcept { return fields->line; }

inline std::optional<MemberId> const &Error::member() const noexcept { return fields->member; }

inline std::optional<IbanReason> const &Error::reason() const noexcept { return fields->reason; }

inline std::optional<std::string> const &Error::right() const noexcept { return fields->right; }

inline std::optional<std::int64_t> const &Error::totalBasisPoints() const noexcept { return fields->totalBasisPoints; }

inline std::optional<TransferId> const &Error::transfer() const noexcept { return fields->transfer; }

inline std::optional<std::vector<TransferId>> const &Error::transfers() const noexcept { return fields->transfers; }

} // namespace eranos

namespace eranos::detail {

/// The value @p result holds, or the error it holds.
template <typename Value>
[[nodiscard]] inline std::expected<Value, Error> expectedOf(diplomat::result<Value, std::unique_ptr<EranosError>> &&result) {
    if (result.is_ok()) {
        return std::move(std::move(result).ok().value()); // NOLINT(bugprone-unchecked-optional-access): is_ok() says which side it holds.
    }
    return std::unexpected(
        errorOf(*std::move(result).err().value())); // NOLINT(bugprone-unchecked-optional-access): is_ok() says which side it holds.
}

/// Nothing when @p result, of a call that returns nothing, succeeded, or the error it holds.
[[nodiscard]] inline std::expected<void, Error> expectedOf(diplomat::result<std::monostate, std::unique_ptr<EranosError>> &&result) {
    return expectedOf<std::monostate>(std::move(result)).transform([](std::monostate const /* value */) noexcept {});
}

/// The value of @p result, which holds no error: the call fails only for a value that the layer's types rule out, and its caller says which.
template <typename Value> [[nodiscard]] inline Value known(diplomat::result<Value, std::unique_ptr<EranosError>> &&result) {
    return std::move(std::move(result).ok().value()); // NOLINT(bugprone-unchecked-optional-access): the caller says why it holds a value.
}

/// The value of @p result, of a synchronous group call that takes nothing the library could refuse: the call waits for the group's lock, so it
/// fails only where it cannot wait, on wasm32, for which the C API does not build.
template <typename Value> [[nodiscard]] inline Value waited(diplomat::result<Value, std::unique_ptr<EranosError>> &&result) {
    return known(std::move(result));
}

/// @p record, once @p set has given it @p value when @p value is present, or the error that @p set reports.
template <typename Record, typename Value, typename Set>
[[nodiscard]] inline std::expected<std::unique_ptr<Record>, Error> setIfPresent(std::unique_ptr<Record> record, std::optional<Value> const &value,
                                                                                Set const &set) {
    if (!value) {
        return record;
    }
    return expectedOf(set(*record, *value)).transform([&record] { return std::move(record); });
}

/// The generated list @p List of @p records, or the error that the conversion of the first record it refuses reports.
template <typename List, typename Record>
[[nodiscard]] inline std::expected<std::unique_ptr<List>, Error> listOf(std::vector<Record> const &records) {
    std::expected<std::unique_ptr<List>, Error> result = List::create();
    for (auto const &record : records) {
        result = std::move(result).and_then([&record](std::unique_ptr<List> list) {
            return toDetail(record).transform([&list](auto const &element) {
                list->push(*element);
                return std::move(list);
            });
        });
    }
    return result;
}

/// @p text, absent when empty: the group holds no blank description and no blank language.
[[nodiscard]] inline std::optional<std::string> present(std::string text) noexcept {
    if (text.empty()) {
        return std::nullopt;
    }
    return text;
}

/// The identifier of @p text, if present.
template <typename Identifier> [[nodiscard]] inline std::optional<Identifier> optionalId(std::optional<std::string> text) noexcept {
    if (!text) {
        return std::nullopt;
    }
    return Identifier(std::move(*text));
}

/// The value that @p make makes of @p text, if present.
template <typename Value>
[[nodiscard]] inline std::optional<Value> optionalText(std::optional<std::string> text, Value (*const make)(std::string) noexcept) noexcept {
    if (!text) {
        return std::nullopt;
    }
    return make(std::move(*text));
}

/// The value of the generated object @p value, if present.
template <typename Value, typename Detail> [[nodiscard]] inline std::optional<Value> optionalRecord(std::unique_ptr<Detail> const &value) {
    if (!value) {
        return std::nullopt;
    }
    return fromDetail(*value);
}

/// The identifiers of the texts of @p list.
template <typename Identifier> [[nodiscard]] inline std::vector<Identifier> identifiers(EranosTextList const &list) {
    std::vector<Identifier> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.emplace_back(*list.get(index)); // NOLINT(bugprone-unchecked-optional-access): the index is below len().
    }
    return result;
}

/// The success of an asynchronous call that returns nothing, whose outcome @p outcome holds no error.
[[nodiscard]] inline std::expected<void, Error> finished(EranosUnitOutcome const & /* outcome */) noexcept { return {}; }

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
[[nodiscard]] inline std::future<std::expected<Value, Error>> launch(Start const &start, Convert convert) {
    auto const promise = std::make_shared<std::promise<std::expected<Value, Error>>>();
    auto future = promise->get_future();
    start([promise, convert = std::move(convert)](auto const outcome) noexcept { complete(*promise, *outcome, convert); });
    return future;
}

/// A future of the asynchronous call that @p start makes with the generated form of its argument, @p argument, and the callback it is given, as
/// the other launch() does; when the argument's conversion failed, a future of its error, and no call.
template <typename Value, typename Argument, typename Start, typename Convert>
[[nodiscard]] inline std::future<std::expected<Value, Error>> launch(std::expected<Argument, Error> const &argument, Start const &start,
                                                                     Convert convert) {
    if (argument) {
        return launch<Value>([&argument, &start](auto callback) { start(*argument, std::move(callback)); }, std::move(convert));
    }
    std::promise<std::expected<Value, Error>> promise;
    promise.set_value(std::unexpected(argument.error()));
    return promise.get_future();
}

/// The currency of @p details.
[[nodiscard]] inline Currency currencyOf(EranosCurrencyDetails const &details) {
    return Currency(
        {.code = details.code(), .decimalPlaces = details.decimal_places(), .name = details.name(), .numericCode = details.numeric_code()});
}

/// The currency of the ISO 4217 code @p code, which the library wrote.
[[nodiscard]] inline Currency listedCurrency(std::string_view const code) {
    // A code the library wrote is one it lists.
    return currencyOf(*known(EranosCurrencyDetails::of(code)));
}

/// The amount of @p value, which the library made. It reads the currency with known(), which converts no error: an error's amounts convert
/// through this function, so converting an error here would make the two call each other.
[[nodiscard]] inline Money fromDetail(EranosMoney const &value) { return {listedCurrency(value.currency()), value.minor_units()}; }

/// The generated amount of @p value.
[[nodiscard]] inline std::unique_ptr<EranosMoney> toDetail(Money const &value) {
    // A currency's code is text the library wrote.
    return known(EranosMoney::create(value.currency.code(), value.minorUnits));
}

/// The rate written @p text, which the library wrote.
[[nodiscard]] inline ExchangeRate exchangeRateOf(std::string text) noexcept { return ExchangeRate(std::move(text)); }

/// The date of @p date, which the library checked.
[[nodiscard]] constexpr CalendarDate calendarDateOf(EranosCalendarDate const date) noexcept {
    return CalendarDate({.day = date.day, .month = date.month, .year = date.year});
}

/// The generated date of @p date.
[[nodiscard]] constexpr EranosCalendarDate toDetail(CalendarDate const date) noexcept {
    return EranosCalendarDate{.day = date.day(), .month = date.month(), .year = date.year()};
}

/// The time of @p time, which the library checked.
[[nodiscard]] constexpr TimeOfDay timeOfDayOf(EranosTimeOfDay const time) noexcept { return TimeOfDay({.hour = time.hour, .minute = time.minute}); }

/// The generated time of @p time.
[[nodiscard]] constexpr EranosTimeOfDay toDetail(TimeOfDay const time) noexcept {
    return EranosTimeOfDay{.hour = time.hour(), .minute = time.minute()};
}

/// The time of @p time, if present.
[[nodiscard]] constexpr std::optional<TimeOfDay> optionalTime(std::optional<EranosTimeOfDay> const time) noexcept {
    if (!time) {
        return std::nullopt;
    }
    return timeOfDayOf(*time);
}

/// The email written @p text, which the library wrote.
[[nodiscard]] inline Email emailOf(std::string text) noexcept { return Email(std::move(text)); }

/// The phone number written @p text, which the library wrote.
[[nodiscard]] inline PhoneNumber phoneNumberOf(std::string text) noexcept { return PhoneNumber(std::move(text)); }

/// The IBAN written @p text, which the library wrote.
[[nodiscard]] inline Iban ibanOf(std::string text) noexcept { return Iban(std::move(text)); }

/// The username written @p text, which the library wrote.
[[nodiscard]] inline Username usernameOf(std::string text) noexcept { return Username(std::move(text)); }

/// The language written @p text, which the library wrote.
[[nodiscard]] inline Language languageOf(std::string text) noexcept { return Language(std::move(text)); }

/// The error kind of @p value. The two enumerations list their 18 values in one order, which the generator of both
/// writes, so a value converts by its number; a library failure that no call can provoke, such as a settlement out of memory, converts
/// as every other does.
[[nodiscard]] inline ErrorKind fromDetail(EranosErrorKind const value) noexcept { return static_cast<ErrorKind>(std::to_underlying(value.AsFFI())); }

/// The failure of @p value. The two enumerations list their 55 values in one order, which the generator of both
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
[[nodiscard]] inline Error errorOf(EranosError const &detail) {
    Error::Fields fields{};
    fields.kind = fromDetail(detail.kind());
    fields.message = detail.message();
    fields.failure = detail.failure().transform([](EranosFailure const value) noexcept { return fromDetail(value); });
    fields.actual = optionalRecord<Money>(detail.actual());
    fields.allowed = detail.allowed();
    fields.balances = detail.balances();
    fields.category = optionalId<CategoryId>(detail.category());
    fields.column = detail.column();
    fields.currency = detail.currency();
    fields.existing = optionalId<CategoryId>(detail.existing());
    fields.expectedAmount = optionalRecord<Money>(detail.expected_amount());
    fields.expectedCurrency = detail.expected_currency();
    fields.expectedLength = detail.expected_length();
    fields.expense = optionalId<ExpenseId>(detail.expense());
    auto const expenses = detail.expenses();
    fields.expenses = expenses ? std::optional<std::vector<ExpenseId>>(identifiers<ExpenseId>(*expenses)) : std::nullopt;
    fields.format = detail.format();
    fields.foundCurrency = detail.found_currency();
    fields.foundLength = detail.found_length();
    fields.left = detail.left();
    fields.line = detail.line();
    fields.member = optionalId<MemberId>(detail.member());
    fields.reason = detail.reason().transform([](EranosIbanReason const value) noexcept { return fromDetail(value); });
    fields.right = detail.right();
    fields.totalBasisPoints = detail.total_basis_points();
    fields.transfer = optionalId<TransferId>(detail.transfer());
    auto const transfers = detail.transfers();
    fields.transfers = transfers ? std::optional<std::vector<TransferId>>(identifiers<TransferId>(*transfers)) : std::nullopt;
    return Error(std::make_shared<Error::Fields const>(std::move(fields)));
}

/// The generated list of the identifiers @p members, or the error of the first that is not text.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosTextList>, Error> toDetail(std::vector<MemberId> const &members) {
    std::expected<std::unique_ptr<EranosTextList>, Error> result = EranosTextList::create();
    for (auto const &member : members) {
        result = std::move(result).and_then([&member](std::unique_ptr<EranosTextList> list) {
            return expectedOf(list->push(member.value())).transform([&list] { return std::move(list); });
        });
    }
    return result;
}

/// The record of @p value.
[[nodiscard]] inline MemberAmount fromDetail(EranosMemberAmount const &value) {
    return {detail::fromDetail(*value.amount()), MemberId(value.member())};
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberAmount>, Error> toDetail(MemberAmount const &value) {
    return expectedOf(EranosMemberAmount::create(*detail::toDetail(value.amount), value.member.value()));
}

/// The record of @p value.
[[nodiscard]] inline MemberParts fromDetail(EranosMemberParts const &value) { return {MemberId(value.member()), Parts{value.hundredths()}}; }

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberParts>, Error> toDetail(MemberParts const &value) {
    return expectedOf(EranosMemberParts::create(value.parts.hundredths, value.member.value()));
}

/// The record of @p value.
[[nodiscard]] inline MemberPercentage fromDetail(EranosMemberPercentage const &value) {
    return {MemberId(value.member()), Percentage{value.basis_points()}};
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberPercentage>, Error> toDetail(MemberPercentage const &value) {
    return expectedOf(EranosMemberPercentage::create(value.percentage.basisPoints, value.member.value()));
}

/// The alternative of @p value.
[[nodiscard]] inline Split fromDetail(EranosSplit const &value) {
    switch (value.kind()) {
    case EranosSplitKind::Amounts:
        return AmountsSplit{detail::fromDetail(*value.amounts_shares())};
    case EranosSplitKind::Equal:
        return EqualSplit{detail::identifiers<MemberId>(*value.members())};
    case EranosSplitKind::Parts:
        return PartsSplit{detail::fromDetail(*value.parts_shares())};
    case EranosSplitKind::Percentages:
        break;
    }
    return PercentagesSplit{detail::fromDetail(*value.percentages_shares())};
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(AmountsSplit const &value) {
    return listOf<EranosMemberAmountList>(value.shares).transform([](std::unique_ptr<EranosMemberAmountList> const &shares) {
        return EranosSplit::create_amounts(*shares);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(EqualSplit const &value) {
    return detail::toDetail(value.members).transform([](std::unique_ptr<EranosTextList> const &members) {
        return EranosSplit::create_equal(*members);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(PartsSplit const &value) {
    return listOf<EranosMemberPartsList>(value.shares).transform([](std::unique_ptr<EranosMemberPartsList> const &shares) {
        return EranosSplit::create_parts(*shares);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(PercentagesSplit const &value) {
    return listOf<EranosMemberPercentageList>(value.shares).transform([](std::unique_ptr<EranosMemberPercentageList> const &shares) {
        return EranosSplit::create_percentages(*shares);
    });
}

/// The generated split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(Split const &value) {
    return std::visit([](auto const &alternative) { return toDetail(alternative); }, value);
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberAmount> fromDetail(EranosMemberAmountList const &list) {
    std::vector<MemberAmount> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberParts> fromDetail(EranosMemberPartsList const &list) {
    std::vector<MemberParts> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberPercentage> fromDetail(EranosMemberPercentageList const &list) {
    std::vector<MemberPercentage> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The record of @p value.
[[nodiscard]] inline Item fromDetail(EranosItem const &value) {
    Item result(value.name(), detail::fromDetail(*value.split()), detail::fromDetail(*value.unit_price()));
    result.quantity = value.quantity();
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosItem>, Error> toDetail(Item const &value) {
    return detail::toDetail(value.split)
        .and_then([&value](std::unique_ptr<EranosSplit> const &split) {
            return expectedOf(EranosItem::create(value.name, *split, *detail::toDetail(value.unitPrice)));
        })
        .transform([&value](std::unique_ptr<EranosItem> result) {
            result->set_quantity(value.quantity);
            return result;
        });
}

/// The alternative of @p value.
[[nodiscard]] inline TipValue fromDetail(EranosTipValue const &value) {
    switch (value.kind()) {
    case EranosTipValueKind::Fixed:
        return FixedTipValue{detail::fromDetail(*value.amount())};
    case EranosTipValueKind::Percentage:
        break;
    }
    // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says the value is present.
    return PercentageTipValue{Percentage{value.basis_points().value()}};
}

/// The generated tip value of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(FixedTipValue const &value) {
    return EranosTipValue::create_fixed(*detail::toDetail(value.amount));
}

/// The generated tip value of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(PercentageTipValue const &value) {
    return EranosTipValue::create_percentage(value.percentage.basisPoints);
}

/// The generated tip value of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTipValue> toDetail(TipValue const &value) {
    return std::visit([](auto const &alternative) { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline Tip fromDetail(EranosTip const &value) { return {detail::fromDetail(value.distribution()), detail::fromDetail(*value.value())}; }

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosTip> toDetail(Tip const &value) {
    return EranosTip::create(*detail::toDetail(value.value), detail::toDetail(value.distribution));
}

/// The alternative of @p value.
[[nodiscard]] inline Cost fromDetail(EranosCost const &value) {
    switch (value.kind()) {
    case EranosCostKind::Converted:
        // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says the value is present.
        return ConvertedCost{.amount = detail::fromDetail(*value.amount()), .rate = detail::exchangeRateOf(value.rate().value())};
    case EranosCostKind::Local:
        break;
    }
    return LocalCost{detail::fromDetail(*value.amount())};
}

/// The generated cost of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(ConvertedCost const &value) {
    // A rate is text the library wrote.
    return known(EranosCost::create_converted(*detail::toDetail(value.amount), value.rate.toString()));
}

/// The generated cost of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(LocalCost const &value) {
    return EranosCost::create_local(*detail::toDetail(value.amount));
}

/// The generated cost of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(Cost const &value) {
    return std::visit([](auto const &alternative) { return toDetail(alternative); }, value);
}

/// The alternative of @p value.
[[nodiscard]] inline ExpenseSplit fromDetail(EranosExpenseSplit const &value) {
    switch (value.kind()) {
    case EranosExpenseSplitKind::Itemised:
        return ItemisedExpenseSplit{detail::fromDetail(*value.items())};
    case EranosExpenseSplitKind::Whole:
        break;
    }
    return WholeExpenseSplit{detail::fromDetail(*value.split())};
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ItemisedExpenseSplit const &value) {
    return listOf<EranosItemList>(value.items).transform([](std::unique_ptr<EranosItemList> const &items) {
        return EranosExpenseSplit::create_itemised(*items);
    });
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(WholeExpenseSplit const &value) {
    return detail::toDetail(value.split).transform([](std::unique_ptr<EranosSplit> const &split) {
        return EranosExpenseSplit::create_whole(*split);
    });
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpenseSplit>, Error> toDetail(ExpenseSplit const &value) {
    return std::visit([](auto const &alternative) { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline Expense fromDetail(EranosExpense const &value) {
    Expense result(detail::fromDetail(*value.cost()), detail::calendarDateOf(value.date()), value.name(), detail::fromDetail(*value.payers()),
                   detail::fromDetail(*value.split()));
    result.id = detail::optionalId<ExpenseId>(value.id());
    result.category = detail::optionalId<CategoryId>(value.category());
    result.description = value.description();
    result.time = detail::optionalTime(value.time());
    result.tip = detail::optionalRecord<Tip>(value.tip());
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosExpense>, Error> toDetail(Expense const &value) {
    return detail::toDetail(value.payers)
        .and_then([&value](std::unique_ptr<EranosSplit> const &payers) {
            return detail::toDetail(value.split).and_then([&value, &payers](std::unique_ptr<EranosExpenseSplit> const &split) {
                return expectedOf(EranosExpense::create(*detail::toDetail(value.cost), detail::toDetail(value.date), value.name, *payers, *split));
            });
        })
        .and_then([&value](std::unique_ptr<EranosExpense> result) {
            if (value.time) {
                result->set_time(detail::toDetail(*value.time));
            }
            if (value.tip) {
                result->set_tip(*detail::toDetail(*value.tip));
            }
            return setIfPresent(std::move(result), value.id, [](EranosExpense &record, ExpenseId const &id) { return record.set_id(id.value()); });
        })
        .and_then([&value](std::unique_ptr<EranosExpense> result) {
            return setIfPresent(std::move(result), value.category,
                                [](EranosExpense &record, CategoryId const &id) { return record.set_category(id.value()); });
        })
        .and_then([&value](std::unique_ptr<EranosExpense> result) {
            return setIfPresent(std::move(result), value.description,
                                [](EranosExpense &record, std::string const &text) { return record.set_description(text); });
        });
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Expense> fromDetail(EranosExpenseList const &list) {
    std::vector<Expense> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Item> fromDetail(EranosItemList const &list) {
    std::vector<Item> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The value of @p value.
[[nodiscard]] constexpr TipDistribution fromDetail(EranosTipDistribution const value) noexcept {
    switch (value) {
    case EranosTipDistribution::Equal:
        return TipDistribution::EQUAL;
    case EranosTipDistribution::Proportional:
        break;
    }
    return TipDistribution::PROPORTIONAL;
}

/// The generated value of @p value.
[[nodiscard]] constexpr EranosTipDistribution toDetail(TipDistribution const value) noexcept {
    switch (value) {
    case TipDistribution::EQUAL:
        return EranosTipDistribution::Equal;
    case TipDistribution::PROPORTIONAL:
        break;
    }
    return EranosTipDistribution::Proportional;
}

/// The record of @p value.
[[nodiscard]] inline Transfer fromDetail(EranosTransfer const &value) {
    Transfer result(detail::fromDetail(*value.cost()), detail::calendarDateOf(value.date()), MemberId(value.from()), MemberId(value.to()));
    result.id = detail::optionalId<TransferId>(value.id());
    result.note = value.note();
    result.time = detail::optionalTime(value.time());
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosTransfer>, Error> toDetail(Transfer const &value) {
    return expectedOf(EranosTransfer::create(*detail::toDetail(value.cost), detail::toDetail(value.date), value.from.value(), value.to.value()))
        .and_then([&value](std::unique_ptr<EranosTransfer> result) {
            if (value.time) {
                result->set_time(detail::toDetail(*value.time));
            }
            return setIfPresent(std::move(result), value.id, [](EranosTransfer &record, TransferId const &id) { return record.set_id(id.value()); });
        })
        .and_then([&value](std::unique_ptr<EranosTransfer> result) {
            return setIfPresent(std::move(result), value.note, [](EranosTransfer &record, std::string const &text) { return record.set_note(text); });
        });
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Transfer> fromDetail(EranosTransferList const &list) {
    std::vector<Transfer> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The alternative of @p value.
[[nodiscard]] inline AccountDetails fromDetail(EranosAccountDetails const &value) {
    switch (value.kind()) {
    case EranosAccountDetailsKind::Iban:
        // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says the value is present.
        return IbanAccountDetails{detail::ibanOf(value.iban().value())};
    case EranosAccountDetailsKind::Username:
        break;
    }
    // NOLINTNEXTLINE(bugprone-unchecked-optional-access): the kind says the value is present.
    return UsernameAccountDetails{detail::usernameOf(value.username().value())};
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(IbanAccountDetails const &value) {
    // An IBAN is text the library wrote.
    return known(EranosAccountDetails::create_iban(value.iban.toString()));
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(UsernameAccountDetails const &value) {
    // A username is text the library wrote.
    return known(EranosAccountDetails::create_username(value.username.toString()));
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(AccountDetails const &value) {
    return std::visit([](auto const &alternative) { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline Account fromDetail(EranosAccount const &value) { return {detail::fromDetail(*value.details()), value.name()}; }

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosAccount>, Error> toDetail(Account const &value) {
    return expectedOf(EranosAccount::create(*detail::toDetail(value.details), value.name));
}

/// The record of @p value.
[[nodiscard]] inline Member fromDetail(EranosMember const &value) {
    Member result(value.first_name());
    result.id = detail::optionalId<MemberId>(value.id());
    result.accounts = detail::fromDetail(*value.accounts());
    result.email = detail::optionalText<Email>(value.email(), detail::emailOf);
    result.lastName = value.last_name();
    result.phone = detail::optionalText<PhoneNumber>(value.phone(), detail::phoneNumberOf);
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMember>, Error> toDetail(Member const &value) {
    return listOf<EranosAccountList>(value.accounts)
        .and_then([&value](std::unique_ptr<EranosAccountList> const &accounts) {
            return expectedOf(EranosMember::create(value.firstName)).transform([&accounts](std::unique_ptr<EranosMember> result) {
                result->set_accounts(*accounts);
                return result;
            });
        })
        .and_then([&value](std::unique_ptr<EranosMember> result) {
            return setIfPresent(std::move(result), value.id, [](EranosMember &record, MemberId const &id) { return record.set_id(id.value()); });
        })
        .and_then([&value](std::unique_ptr<EranosMember> result) {
            return setIfPresent(std::move(result), value.email,
                                [](EranosMember &record, Email const &email) { return record.set_email(email.toString()); });
        })
        .and_then([&value](std::unique_ptr<EranosMember> result) {
            return setIfPresent(std::move(result), value.lastName,
                                [](EranosMember &record, std::string const &text) { return record.set_last_name(text); });
        })
        .and_then([&value](std::unique_ptr<EranosMember> result) {
            return setIfPresent(std::move(result), value.phone,
                                [](EranosMember &record, PhoneNumber const &phone) { return record.set_phone(phone.toString()); });
        });
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Account> fromDetail(EranosAccountList const &list) {
    std::vector<Account> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Member> fromDetail(EranosMemberList const &list) {
    std::vector<Member> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The record of @p value.
[[nodiscard]] inline Category fromDetail(EranosCategory const &value) {
    Category result(value.name());
    result.id = detail::optionalId<CategoryId>(value.id());
    return result;
}

/// The generated record of @p value.
[[nodiscard]] inline std::expected<std::unique_ptr<EranosCategory>, Error> toDetail(Category const &value) {
    return expectedOf(EranosCategory::create(value.name)).and_then([&value](std::unique_ptr<EranosCategory> result) {
        return setIfPresent(std::move(result), value.id, [](EranosCategory &record, CategoryId const &id) { return record.set_id(id.value()); });
    });
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Category> fromDetail(EranosCategoryList const &list) {
    std::vector<Category> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The alternative of @p value.
[[nodiscard]] inline Balance fromDetail(EranosBalance const &value) {
    switch (value.kind()) {
    case EranosBalanceKind::GetsBack:
        return GetsBackBalance{detail::fromDetail(*value.amount())};
    case EranosBalanceKind::Owes:
        return OwesBalance{detail::fromDetail(*value.amount())};
    case EranosBalanceKind::Settled:
        break;
    }
    return SettledBalance{};
}

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(GetsBackBalance const &value) {
    return EranosBalance::create_gets_back(*detail::toDetail(value.amount));
}

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(OwesBalance const &value) {
    return EranosBalance::create_owes(*detail::toDetail(value.amount));
}

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(SettledBalance const & /* value */) { return EranosBalance::create_settled(); }

/// The generated balance of @p value.
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(Balance const &value) {
    return std::visit([](auto const &alternative) { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline MemberSummary fromDetail(EranosMemberSummary const &value) {
    return {detail::fromDetail(*value.balance()),  MemberId(value.member()),
            detail::fromDetail(*value.net()),      detail::fromDetail(*value.paid()),
            detail::fromDetail(*value.received()), detail::fromDetail(*value.sent()),
            detail::fromDetail(*value.share())};
}

/// The record of @p value.
[[nodiscard]] inline CategoryTotal fromDetail(EranosCategoryTotal const &value) {
    return {CategoryId(value.category()), detail::fromDetail(*value.total())};
}

/// The record of @p value.
[[nodiscard]] inline MemberCategoryTotals fromDetail(EranosMemberCategoryTotals const &value) {
    return {MemberId(value.member()), detail::fromDetail(*value.totals())};
}

/// The record of @p value.
[[nodiscard]] inline Summary fromDetail(EranosSummary const &value) {
    return {detail::fromDetail(*value.category_totals()), detail::fromDetail(*value.member_category_totals()), detail::fromDetail(*value.members()),
            detail::fromDetail(*value.total_spent())};
}

/// The record of @p value.
[[nodiscard]] inline MemberShare fromDetail(EranosMemberShare const &value) {
    return {MemberId(value.member()), detail::fromDetail(*value.paid()), detail::fromDetail(*value.share())};
}

/// The record of @p value.
[[nodiscard]] inline ExpenseShares fromDetail(EranosExpenseShares const &value) {
    return {detail::fromDetail(*value.members()), detail::fromDetail(*value.tip()), detail::fromDetail(*value.total())};
}

/// The record of @p value.
[[nodiscard]] inline Payment fromDetail(EranosPayment const &value) {
    return {detail::fromDetail(*value.amount()), MemberId(value.from()), MemberId(value.to())};
}

/// The record of @p value.
[[nodiscard]] inline Settlement fromDetail(EranosSettlement const &value) {
    return {detail::fromDetail(*value.payments()), detail::fromDetail(value.strategy())};
}

/// The records of @p list.
[[nodiscard]] inline std::vector<CategoryTotal> fromDetail(EranosCategoryTotalList const &list) {
    std::vector<CategoryTotal> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberCategoryTotals> fromDetail(EranosMemberCategoryTotalsList const &list) {
    std::vector<MemberCategoryTotals> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberShare> fromDetail(EranosMemberShareList const &list) {
    std::vector<MemberShare> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<MemberSummary> fromDetail(EranosMemberSummaryList const &list) {
    std::vector<MemberSummary> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The records of @p list.
[[nodiscard]] inline std::vector<Payment> fromDetail(EranosPaymentList const &list) {
    std::vector<Payment> result;
    result.reserve(list.len());
    for (std::size_t index = 0; index < list.len(); ++index) {
        result.push_back(fromDetail(*list.get(index)));
    }
    return result;
}

/// The value of @p value.
[[nodiscard]] constexpr SettlementStrategy fromDetail(EranosSettlementStrategy const value) noexcept {
    switch (value) {
    case EranosSettlementStrategy::Adaptive:
        return SettlementStrategy::ADAPTIVE;
    case EranosSettlementStrategy::Direct:
        return SettlementStrategy::DIRECT;
    case EranosSettlementStrategy::Greedy:
        return SettlementStrategy::GREEDY;
    case EranosSettlementStrategy::MinimumTransfers:
        break;
    }
    return SettlementStrategy::MINIMUM_TRANSFERS;
}

/// The generated value of @p value.
[[nodiscard]] constexpr EranosSettlementStrategy toDetail(SettlementStrategy const value) noexcept {
    switch (value) {
    case SettlementStrategy::ADAPTIVE:
        return EranosSettlementStrategy::Adaptive;
    case SettlementStrategy::DIRECT:
        return EranosSettlementStrategy::Direct;
    case SettlementStrategy::GREEDY:
        return EranosSettlementStrategy::Greedy;
    case SettlementStrategy::MINIMUM_TRANSFERS:
        break;
    }
    return EranosSettlementStrategy::MinimumTransfers;
}

/// The generated cancellation of @p cancellation, or none.
[[nodiscard]] inline EranosCancellation const *handleOf(Cancellation const *const cancellation) noexcept {
    return nullptr == cancellation ? nullptr : cancellation->handle.get();
}

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRate> toDetail(CurrencyRate const &value) {
    // A currency's code and a rate are text the library wrote.
    return known(EranosCurrencyRate::create(value.currency.code(), value.rate.toString()));
}

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRateList> toDetail(std::span<CurrencyRate const> const records) {
    auto result = EranosCurrencyRateList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
    }
    return result;
}

} // namespace eranos::detail

#endif // ERANOS_DETAIL_CONVERSIONS_HPP
