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
#include <exception>
#include <future>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace eranos::detail {

/// The value @p result holds, or the error it holds, thrown.
template <typename Value> [[nodiscard]] inline Value unwrap(diplomat::result<Value, std::unique_ptr<EranosError>> &&result) {
    if (result.is_ok()) {
        return std::move(std::move(result).ok().value()); // NOLINT(bugprone-unchecked-optional-access): is_ok() says which side it holds.
    }
    throw errorOf(*std::move(result).err().value()); // NOLINT(bugprone-unchecked-optional-access): is_ok() says which side it holds.
}

/// Throws the error @p result holds, if it holds one.
inline void check(diplomat::result<std::monostate, std::unique_ptr<EranosError>> &&result) { static_cast<void>(unwrap(std::move(result))); }

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

// NOLINTBEGIN(bugprone-exception-escape): a completion runs on the library's thread, where an exception must not travel, so one ends the
// program, as the library's own failure to allocate does; set_value throws only for a promise already satisfied, and the one callback of a
// call never is, and a conversion throws only when memory runs out.

/// Completes @p promise with the value @p convert makes of @p outcome, or with the error it holds.
template <typename Value, typename Outcome, typename Convert>
inline void complete(std::promise<Value> &promise, Outcome &outcome, Convert const &convert) noexcept {
    if (outcome.is_ok()) {
        promise.set_value(convert(outcome));
    } else {
        promise.set_exception(std::make_exception_ptr(errorOf(*outcome.error())));
    }
}

/// Completes @p promise when @p outcome succeeded, or with the error it holds.
template <typename Outcome> inline void complete(std::promise<void> &promise, Outcome const &outcome) noexcept {
    if (outcome.is_ok()) {
        promise.set_value();
    } else {
        promise.set_exception(std::make_exception_ptr(errorOf(*outcome.error())));
    }
}

// NOLINTEND(bugprone-exception-escape)

/// The currency of @p details.
[[nodiscard]] inline Currency currencyOf(EranosCurrencyDetails const &details) {
    return Currency({details.code(), details.decimal_places(), details.name(), details.numeric_code()});
}

/// The amount of @p value, which the library made, so that its currency is one the library lists. The currency is read without
/// Currency::of, whose failure would convert an error, whose amounts this converts.
[[nodiscard]] inline Money fromDetail(EranosMoney const &value) {
    // NOLINTNEXTLINE(bugprone-unchecked-optional-access): an amount the library made is in a currency it lists.
    return {currencyOf(*EranosCurrencyDetails::of(value.currency()).ok().value()), value.minor_units()};
}

/// The generated amount of @p value.
[[nodiscard]] inline std::unique_ptr<EranosMoney> toDetail(Money const &value) {
    return unwrap(EranosMoney::create(value.currency.code(), value.minorUnits));
}

/// The rate written @p text, which the library wrote.
[[nodiscard]] inline ExchangeRate exchangeRateOf(std::string text) noexcept { return ExchangeRate(std::move(text)); }

/// The date of @p date, which the library checked.
[[nodiscard]] constexpr CalendarDate calendarDateOf(EranosCalendarDate const date) noexcept {
    return CalendarDate({date.day, date.month, date.year});
}

/// The generated date of @p date.
[[nodiscard]] constexpr EranosCalendarDate toDetail(CalendarDate const date) noexcept {
    return EranosCalendarDate{date.day(), date.month(), date.year()};
}

/// The time of @p time, which the library checked.
[[nodiscard]] constexpr TimeOfDay timeOfDayOf(EranosTimeOfDay const time) noexcept { return TimeOfDay({time.hour, time.minute}); }

/// The generated time of @p time.
[[nodiscard]] constexpr EranosTimeOfDay toDetail(TimeOfDay const time) noexcept { return EranosTimeOfDay{time.hour(), time.minute()}; }

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
[[nodiscard]] inline ErrorKind fromDetail(EranosErrorKind const value) noexcept {
    return static_cast<ErrorKind>(static_cast<std::uint8_t>(value.AsFFI()));
}

/// The failure of @p value. The two enumerations list their 55 values in one order, which the generator of both
/// writes, so a value converts by its number; a library failure that no call can provoke, such as a settlement out of memory, converts
/// as every other does.
[[nodiscard]] inline Failure fromDetail(EranosFailure const value) noexcept { return static_cast<Failure>(static_cast<std::uint8_t>(value.AsFFI())); }

/// The iban reason of @p value. The two enumerations list their 6 values in one order, which the generator of both
/// writes, so a value converts by its number; a library failure that no call can provoke, such as a settlement out of memory, converts
/// as every other does.
[[nodiscard]] inline IbanReason fromDetail(EranosIbanReason const value) noexcept {
    return static_cast<IbanReason>(static_cast<std::uint8_t>(value.AsFFI()));
}

/// The error of @p detail, with everything the failure holds.
[[nodiscard]] inline Error errorOf(EranosError const &detail) {
    Error::Fields fields{};
    fields.kind = fromDetail(detail.kind());
    auto const failure = detail.failure();
    fields.failure = failure ? std::optional<Failure>(fromDetail(*failure)) : std::nullopt;
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
    auto const reason = detail.reason();
    fields.reason = reason ? std::optional<IbanReason>(fromDetail(*reason)) : std::nullopt;
    fields.right = detail.right();
    fields.totalBasisPoints = detail.total_basis_points();
    fields.transfer = optionalId<TransferId>(detail.transfer());
    auto const transfers = detail.transfers();
    fields.transfers = transfers ? std::optional<std::vector<TransferId>>(identifiers<TransferId>(*transfers)) : std::nullopt;
    return {detail.message(), std::make_shared<Error::Fields const>(std::move(fields))};
}

/// The generated list of the identifiers @p members.
[[nodiscard]] inline std::unique_ptr<EranosTextList> toDetail(std::vector<MemberId> const &members) {
    auto result = EranosTextList::create();
    for (auto const &member : members) {
        check(result->push(member.value()));
    }
    return result;
}

/// The record of @p value.
[[nodiscard]] inline MemberAmount fromDetail(EranosMemberAmount const &value) {
    return {detail::fromDetail(*value.amount()), MemberId(value.member())};
}

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosMemberAmount> toDetail(MemberAmount const &value) {
    return unwrap(EranosMemberAmount::create(*detail::toDetail(value.amount), value.member.value()));
}

/// The record of @p value.
[[nodiscard]] inline MemberParts fromDetail(EranosMemberParts const &value) { return {MemberId(value.member()), Parts{value.hundredths()}}; }

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosMemberParts> toDetail(MemberParts const &value) {
    return unwrap(EranosMemberParts::create(value.parts.hundredths, value.member.value()));
}

/// The record of @p value.
[[nodiscard]] inline MemberPercentage fromDetail(EranosMemberPercentage const &value) {
    return {MemberId(value.member()), Percentage{value.basis_points()}};
}

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosMemberPercentage> toDetail(MemberPercentage const &value) {
    return unwrap(EranosMemberPercentage::create(value.percentage.basisPoints, value.member.value()));
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
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(AmountsSplit const &value) {
    return EranosSplit::create_amounts(*detail::toDetail(value.shares));
}

/// The generated split of @p value.
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(EqualSplit const &value) {
    return EranosSplit::create_equal(*detail::toDetail(value.members));
}

/// The generated split of @p value.
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(PartsSplit const &value) {
    return EranosSplit::create_parts(*detail::toDetail(value.shares));
}

/// The generated split of @p value.
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(PercentagesSplit const &value) {
    return EranosSplit::create_percentages(*detail::toDetail(value.shares));
}

/// The generated split of @p value.
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(Split const &value) {
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

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosMemberAmountList> toDetail(std::vector<MemberAmount> const &records) {
    auto result = EranosMemberAmountList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
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

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosMemberPartsList> toDetail(std::vector<MemberParts> const &records) {
    auto result = EranosMemberPartsList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
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

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosMemberPercentageList> toDetail(std::vector<MemberPercentage> const &records) {
    auto result = EranosMemberPercentageList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
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
[[nodiscard]] inline std::unique_ptr<EranosItem> toDetail(Item const &value) {
    auto result = unwrap(EranosItem::create(value.name, *detail::toDetail(value.split), *detail::toDetail(value.unitPrice)));
    result->set_quantity(value.quantity);
    return result;
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
        return ConvertedCost{detail::fromDetail(*value.amount()), detail::exchangeRateOf(value.rate().value())};
    case EranosCostKind::Local:
        break;
    }
    return LocalCost{detail::fromDetail(*value.amount())};
}

/// The generated cost of @p value.
[[nodiscard]] inline std::unique_ptr<EranosCost> toDetail(ConvertedCost const &value) {
    return unwrap(EranosCost::create_converted(*detail::toDetail(value.amount), value.rate.toString()));
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
[[nodiscard]] inline std::unique_ptr<EranosExpenseSplit> toDetail(ItemisedExpenseSplit const &value) {
    return EranosExpenseSplit::create_itemised(*detail::toDetail(value.items));
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::unique_ptr<EranosExpenseSplit> toDetail(WholeExpenseSplit const &value) {
    return EranosExpenseSplit::create_whole(*detail::toDetail(value.split));
}

/// The generated expense split of @p value.
[[nodiscard]] inline std::unique_ptr<EranosExpenseSplit> toDetail(ExpenseSplit const &value) {
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
[[nodiscard]] inline std::unique_ptr<EranosExpense> toDetail(Expense const &value) {
    auto result = unwrap(EranosExpense::create(*detail::toDetail(value.cost), detail::toDetail(value.date), value.name,
                                               *detail::toDetail(value.payers), *detail::toDetail(value.split)));
    if (value.id) {
        check(result->set_id(value.id->value()));
    }
    if (value.category) {
        check(result->set_category(value.category->value()));
    }
    if (value.description) {
        check(result->set_description(*value.description));
    }
    if (value.time) {
        result->set_time(detail::toDetail((*value.time)));
    }
    if (value.tip) {
        result->set_tip(*detail::toDetail((*value.tip)));
    }
    return result;
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

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosItemList> toDetail(std::vector<Item> const &records) {
    auto result = EranosItemList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
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
[[nodiscard]] inline std::unique_ptr<EranosTransfer> toDetail(Transfer const &value) {
    auto result = unwrap(EranosTransfer::create(*detail::toDetail(value.cost), detail::toDetail(value.date), value.from.value(), value.to.value()));
    if (value.id) {
        check(result->set_id(value.id->value()));
    }
    if (value.note) {
        check(result->set_note(*value.note));
    }
    if (value.time) {
        result->set_time(detail::toDetail((*value.time)));
    }
    return result;
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
    return unwrap(EranosAccountDetails::create_iban(value.iban.toString()));
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(UsernameAccountDetails const &value) {
    return unwrap(EranosAccountDetails::create_username(value.username.toString()));
}

/// The generated account details of @p value.
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(AccountDetails const &value) {
    return std::visit([](auto const &alternative) { return toDetail(alternative); }, value);
}

/// The record of @p value.
[[nodiscard]] inline Account fromDetail(EranosAccount const &value) { return {detail::fromDetail(*value.details()), value.name()}; }

/// The generated record of @p value.
[[nodiscard]] inline std::unique_ptr<EranosAccount> toDetail(Account const &value) {
    return unwrap(EranosAccount::create(*detail::toDetail(value.details), value.name));
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
[[nodiscard]] inline std::unique_ptr<EranosMember> toDetail(Member const &value) {
    auto result = unwrap(EranosMember::create(value.firstName));
    if (value.id) {
        check(result->set_id(value.id->value()));
    }
    result->set_accounts(*toDetail(value.accounts));
    if (value.email) {
        check(result->set_email(value.email->toString()));
    }
    if (value.lastName) {
        check(result->set_last_name(*value.lastName));
    }
    if (value.phone) {
        check(result->set_phone(value.phone->toString()));
    }
    return result;
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

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosAccountList> toDetail(std::vector<Account> const &records) {
    auto result = EranosAccountList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
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
[[nodiscard]] inline std::unique_ptr<EranosCategory> toDetail(Category const &value) {
    auto result = unwrap(EranosCategory::create(value.name));
    if (value.id) {
        check(result->set_id(value.id->value()));
    }
    return result;
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
    return unwrap(EranosCurrencyRate::create(value.currency.code(), value.rate.toString()));
}

/// The generated list of @p records.
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRateList> toDetail(std::vector<CurrencyRate> const &records) {
    auto result = EranosCurrencyRateList::create();
    for (auto const &record : records) {
        result->push(*toDetail(record));
    }
    return result;
}

} // namespace eranos::detail

#endif // ERANOS_DETAIL_CONVERSIONS_HPP
