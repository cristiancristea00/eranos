/// @file eranos/detail/declarations.hpp
/// The declarations every part of the layer names: each type of the layer, and each conversion between the layer's values and the classes that
/// Diplomat generates, which live in eranos::detail and are not part of the layer.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_DETAIL_DECLARATIONS_HPP
#define ERANOS_DETAIL_DECLARATIONS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/EranosBalance.hpp"
#include "eranos/detail/EranosBalanceKind.hpp"
#include "eranos/detail/EranosCalendarDate.hpp"
#include "eranos/detail/EranosCancellation.hpp"
#include "eranos/detail/EranosCategory.hpp"
#include "eranos/detail/EranosCategoryList.hpp"
#include "eranos/detail/EranosCategoryListOutcome.hpp"
#include "eranos/detail/EranosCategoryOutcome.hpp"
#include "eranos/detail/EranosCategoryTotal.hpp"
#include "eranos/detail/EranosCategoryTotalList.hpp"
#include "eranos/detail/EranosCost.hpp"
#include "eranos/detail/EranosCostKind.hpp"
#include "eranos/detail/EranosCurrencyDetails.hpp"
#include "eranos/detail/EranosCurrencyDetailsList.hpp"
#include "eranos/detail/EranosCurrencyRate.hpp"
#include "eranos/detail/EranosCurrencyRateList.hpp"
#include "eranos/detail/EranosEmail.hpp"
#include "eranos/detail/EranosError.hpp"
#include "eranos/detail/EranosErrorKind.hpp"
#include "eranos/detail/EranosExchangeRate.hpp"
#include "eranos/detail/EranosExpense.hpp"
#include "eranos/detail/EranosExpenseGroup.hpp"
#include "eranos/detail/EranosExpenseGroupOutcome.hpp"
#include "eranos/detail/EranosExpenseGroupReader.hpp"
#include "eranos/detail/EranosExpenseGroupReaderOutcome.hpp"
#include "eranos/detail/EranosExpenseItem.hpp"
#include "eranos/detail/EranosExpenseItemList.hpp"
#include "eranos/detail/EranosExpenseList.hpp"
#include "eranos/detail/EranosExpenseListOutcome.hpp"
#include "eranos/detail/EranosExpenseOutcome.hpp"
#include "eranos/detail/EranosExpenseShares.hpp"
#include "eranos/detail/EranosExpenseSharesOutcome.hpp"
#include "eranos/detail/EranosExpenseSplit.hpp"
#include "eranos/detail/EranosExpenseSplitKind.hpp"
#include "eranos/detail/EranosFailure.hpp"
#include "eranos/detail/EranosFlagOutcome.hpp"
#include "eranos/detail/EranosIban.hpp"
#include "eranos/detail/EranosIbanReason.hpp"
#include "eranos/detail/EranosLanguage.hpp"
#include "eranos/detail/EranosMember.hpp"
#include "eranos/detail/EranosMemberAmount.hpp"
#include "eranos/detail/EranosMemberAmountList.hpp"
#include "eranos/detail/EranosMemberCategoryTotals.hpp"
#include "eranos/detail/EranosMemberCategoryTotalsList.hpp"
#include "eranos/detail/EranosMemberList.hpp"
#include "eranos/detail/EranosMemberListOutcome.hpp"
#include "eranos/detail/EranosMemberOutcome.hpp"
#include "eranos/detail/EranosMemberParts.hpp"
#include "eranos/detail/EranosMemberPartsList.hpp"
#include "eranos/detail/EranosMemberPercentage.hpp"
#include "eranos/detail/EranosMemberPercentageList.hpp"
#include "eranos/detail/EranosMemberShare.hpp"
#include "eranos/detail/EranosMemberShareList.hpp"
#include "eranos/detail/EranosMemberSummary.hpp"
#include "eranos/detail/EranosMemberSummaryList.hpp"
#include "eranos/detail/EranosMoney.hpp"
#include "eranos/detail/EranosMoneyOutcome.hpp"
#include "eranos/detail/EranosNewExpenseGroup.hpp"
#include "eranos/detail/EranosOptionalTextOutcome.hpp"
#include "eranos/detail/EranosParts.hpp"
#include "eranos/detail/EranosPayment.hpp"
#include "eranos/detail/EranosPaymentAccount.hpp"
#include "eranos/detail/EranosPaymentAccountDetails.hpp"
#include "eranos/detail/EranosPaymentAccountDetailsKind.hpp"
#include "eranos/detail/EranosPaymentAccountList.hpp"
#include "eranos/detail/EranosPaymentList.hpp"
#include "eranos/detail/EranosPercentage.hpp"
#include "eranos/detail/EranosPhoneNumber.hpp"
#include "eranos/detail/EranosSettlementPlan.hpp"
#include "eranos/detail/EranosSettlementPlanOutcome.hpp"
#include "eranos/detail/EranosSettlementStrategy.hpp"
#include "eranos/detail/EranosSplit.hpp"
#include "eranos/detail/EranosSplitKind.hpp"
#include "eranos/detail/EranosSummary.hpp"
#include "eranos/detail/EranosSummaryOutcome.hpp"
#include "eranos/detail/EranosTextList.hpp"
#include "eranos/detail/EranosTextOutcome.hpp"
#include "eranos/detail/EranosTimeOfDay.hpp"
#include "eranos/detail/EranosTip.hpp"
#include "eranos/detail/EranosTipDistribution.hpp"
#include "eranos/detail/EranosTipValue.hpp"
#include "eranos/detail/EranosTipValueKind.hpp"
#include "eranos/detail/EranosTransfer.hpp"
#include "eranos/detail/EranosTransferList.hpp"
#include "eranos/detail/EranosTransferListOutcome.hpp"
#include "eranos/detail/EranosTransferOutcome.hpp"
#include "eranos/detail/EranosUnitOutcome.hpp"
#include "eranos/detail/EranosUsername.hpp"

#include <cstdint>
#include <expected>
#include <future>
#include <memory>
#include <optional>
#include <string>
#include <variant>
#include <vector>

namespace eranos {

class CalendarDate;
class Cancellation;
class CategoryId;
class Currency;
class Email;
class Error;
class ExchangeRate;
class ExpenseId;
class ExpenseGroup;
class ExpenseGroupId;
class ExpenseGroupReader;
class Iban;
class Language;
class MemberId;
class PhoneNumber;
class TimeOfDay;
class TransferId;
class Username;
struct PaymentAccount;
struct AmountsSplit;
struct Category;
struct CategoryTotal;
struct ConvertedCost;
struct CurrencyRate;
struct EqualSplit;
struct Expense;
struct ExpenseShares;
struct FixedTipValue;
struct GetsBackBalance;
struct ExpenseGroupOptions;
struct IbanPaymentAccountDetails;
struct ExpenseItem;
struct ItemisedExpenseSplit;
struct LocalCost;
struct Member;
struct MemberAmount;
struct MemberCategoryTotals;
struct MemberParts;
struct MemberPercentage;
struct MemberShare;
struct MemberSummary;
struct Money;
struct OwesBalance;
struct Parts;
struct PartsSplit;
struct Payment;
struct Percentage;
struct PercentageTipValue;
struct PercentagesSplit;
struct SettledBalance;
struct SettlementPlan;
struct Summary;
struct Tip;
struct Transfer;
struct UsernamePaymentAccountDetails;
struct WholeExpenseSplit;
enum class ErrorKind : std::uint8_t;
enum class Failure : std::uint8_t;
enum class IbanReason : std::uint8_t;
enum class SettlementStrategy : std::uint8_t;
enum class TipDistribution : std::uint8_t;

} // namespace eranos

namespace eranos::detail {

template <typename Value> [[nodiscard]] inline std::expected<Value, Error> expectedOf(diplomat::result<Value, std::unique_ptr<EranosError>> &&result);
[[nodiscard]] inline std::expected<void, Error> expectedOf(diplomat::result<std::monostate, std::unique_ptr<EranosError>> &&result);
template <typename Value> [[nodiscard]] inline Value known(diplomat::result<Value, std::unique_ptr<EranosError>> &&result);
template <typename Value> [[nodiscard]] inline Value waited(diplomat::result<Value, std::unique_ptr<EranosError>> &&result);
template <typename Record, typename Value, typename Set>
[[nodiscard]] inline std::expected<std::unique_ptr<Record>, Error> setIfPresent(std::unique_ptr<Record> record, std::optional<Value> const &value,
                                                                                Set const &set);
[[nodiscard]] inline std::optional<std::string> present(std::string text) noexcept;
template <typename Identifier> [[nodiscard]] inline std::optional<Identifier> optionalId(std::optional<std::string> text) noexcept;
template <typename Value>
[[nodiscard]] inline std::optional<Value> optionalText(std::optional<std::string> text, Value (*const make)(std::string) noexcept) noexcept;
template <typename Value, typename Detail> [[nodiscard]] inline std::optional<Value> optionalRecord(std::unique_ptr<Detail> const &value);
template <typename Identifier> [[nodiscard]] inline std::vector<Identifier> identifiers(EranosTextList const &list);
[[nodiscard]] inline std::expected<void, Error> finished(EranosUnitOutcome const &outcome) noexcept;
template <typename Value, typename Start, typename Convert>
[[nodiscard]] inline std::future<std::expected<Value, Error>> launch(Start const &start, Convert convert);
template <typename Value, typename Argument, typename Start, typename Convert>
[[nodiscard]] inline std::future<std::expected<Value, Error>> launch(std::expected<Argument, Error> const &argument, Start const &start,
                                                                     Convert convert);

} // namespace eranos::detail

#endif // ERANOS_DETAIL_DECLARATIONS_HPP
