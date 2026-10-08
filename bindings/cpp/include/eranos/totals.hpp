/// @file eranos/totals.hpp
/// The group's totals: the summary, the members' balances, an expense's shares, and the settlements.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_TOTALS_HPP
#define ERANOS_TOTALS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/money.hpp"

#include <cstdint>
#include <memory>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace eranos {

/// How a settlement plans its payments.
enum class SettlementStrategy : std::uint8_t {
    /// The fewest payments while at most 20 balances are left after exact opposites pay each other, the greedy plan beyond; the settlement names the
    /// strategy whose plan it is.
    ADAPTIVE,
    /// The fewest payments possible.
    MINIMUM_TRANSFERS,
    /// A fast plan that pairs the largest debtor with the largest creditor, after exact opposites where that saves a payment.
    GREEDY,
    /// Every debt between two members, unsimplified.
    DIRECT,
};

/// The member owes the others `amount`.
struct OwesBalance final {
    /// What the member owes the others.
    Money amount;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(OwesBalance const &left, OwesBalance const &right) noexcept = default;
};

/// The member owes nothing and is owed nothing.
struct SettledBalance final {
    /// Whether every field of two alternatives is equal.
    ///
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(SettledBalance const & /* left */, SettledBalance const & /* right */) noexcept = default;
};

/// The others owe the member `amount`.
struct GetsBackBalance final {
    /// What the others owe the member.
    Money amount;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(GetsBackBalance const &left, GetsBackBalance const &right) noexcept = default;
};

/// Where a member stands. One of its alternatives: OwesBalance, SettledBalance, GetsBackBalance.
using Balance = std::variant<OwesBalance, SettledBalance, GetsBackBalance>;

/// Where a member stands: what they paid, consumed, sent, and received.
struct MemberSummary final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param paid What the member paid for expenses.
    /// @param share The member's share of the expenses: what they consumed.
    /// @param sent What the member sent in transfers.
    /// @param received What the member received in transfers.
    /// @param net What the member is owed, less what they owe: positive when they get money back.
    /// @param balance Where the member stands.
    inline MemberSummary(MemberId member, Money paid, Money share, Money sent, Money received, Money net, Balance balance) noexcept
        : member(std::move(member)), paid(std::move(paid)), share(std::move(share)), sent(std::move(sent)), received(std::move(received)),
          net(std::move(net)), balance(std::move(balance)) {}

    /// The member's identifier.
    MemberId member;

    /// What the member paid for expenses.
    Money paid;

    /// The member's share of the expenses: what they consumed.
    Money share;

    /// What the member sent in transfers.
    Money sent;

    /// What the member received in transfers.
    Money received;

    /// What the member is owed, less what they owe: positive when they get money back.
    Money net;

    /// Where the member stands.
    Balance balance;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(MemberSummary const &left, MemberSummary const &right) noexcept = default;
};

/// What the expenses of one category add up to.
struct CategoryTotal final {
    /// The record of its fields.
    ///
    /// @param category The category's identifier.
    /// @param total The total.
    inline CategoryTotal(CategoryId category, Money total) noexcept : category(std::move(category)), total(std::move(total)) {}

    /// The category's identifier.
    CategoryId category;

    /// The total.
    Money total;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(CategoryTotal const &left, CategoryTotal const &right) noexcept = default;
};

/// A member's share of each category.
struct MemberCategoryTotals final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param totals The member's total in every category they have a share in.
    inline constexpr MemberCategoryTotals(MemberId member, std::vector<CategoryTotal> totals) noexcept
        : member(std::move(member)), totals(std::move(totals)) {}

    /// The member's identifier.
    MemberId member;

    /// The member's total in every category they have a share in.
    std::vector<CategoryTotal> totals;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(MemberCategoryTotals const &left, MemberCategoryTotals const &right) noexcept = default;
};

/// The group's totals.
struct Summary final {
    /// The record of its fields.
    ///
    /// @param totalSpent What the expenses add up to.
    /// @param members Where every member stands.
    /// @param categoryTotals The total of every category.
    /// @param memberCategoryTotals The totals per category of every member.
    inline Summary(Money totalSpent, std::vector<MemberSummary> members, std::vector<CategoryTotal> categoryTotals,
                   std::vector<MemberCategoryTotals> memberCategoryTotals) noexcept
        : totalSpent(std::move(totalSpent)), members(std::move(members)), categoryTotals(std::move(categoryTotals)),
          memberCategoryTotals(std::move(memberCategoryTotals)) {}

    /// What the expenses add up to.
    Money totalSpent;

    /// Where every member stands.
    std::vector<MemberSummary> members;

    /// The total of every category.
    std::vector<CategoryTotal> categoryTotals;

    /// The totals per category of every member.
    std::vector<MemberCategoryTotals> memberCategoryTotals;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Summary const &left, Summary const &right) noexcept = default;
};

/// A member's part of an expense.
struct MemberShare final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param paid What the member paid of it.
    /// @param share The member's share of it.
    inline MemberShare(MemberId member, Money paid, Money share) noexcept
        : member(std::move(member)), paid(std::move(paid)), share(std::move(share)) {}

    /// The member's identifier.
    MemberId member;

    /// What the member paid of it.
    Money paid;

    /// The member's share of it.
    Money share;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(MemberShare const &left, MemberShare const &right) noexcept = default;
};

/// An expense's shares, in the group currency.
struct ExpenseShares final {
    /// The record of its fields.
    ///
    /// @param members Every member's part.
    /// @param total The amount with the tip.
    /// @param tip The tip.
    inline ExpenseShares(std::vector<MemberShare> members, Money total, Money tip) noexcept
        : members(std::move(members)), total(std::move(total)), tip(std::move(tip)) {}

    /// Every member's part.
    std::vector<MemberShare> members;

    /// The amount with the tip.
    Money total;

    /// The tip.
    Money tip;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(ExpenseShares const &left, ExpenseShares const &right) noexcept = default;
};

/// A payment of a settlement.
struct Payment final {
    /// The record of its fields.
    ///
    /// @param from The identifier of the member who pays.
    /// @param to The identifier of the member who is paid.
    /// @param amount What is paid.
    inline Payment(MemberId from, MemberId to, Money amount) noexcept : from(std::move(from)), to(std::move(to)), amount(std::move(amount)) {}

    /// The identifier of the member who pays.
    MemberId from;

    /// The identifier of the member who is paid.
    MemberId to;

    /// What is paid.
    Money amount;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(Payment const &left, Payment const &right) noexcept = default;
};

/// A plan of payments that settles every balance.
struct SettlementPlan final {
    /// The record of its fields.
    ///
    /// @param payments The payments.
    /// @param strategy The strategy that made the plan: the one asked for, or, for the adaptive strategy, the one whose plan it chose.
    inline constexpr SettlementPlan(std::vector<Payment> payments, SettlementStrategy const strategy) noexcept
        : payments(std::move(payments)), strategy(strategy) {}

    /// The payments.
    std::vector<Payment> payments;

    /// The strategy that made the plan: the one asked for, or, for the adaptive strategy, the one whose plan it chose.
    SettlementStrategy strategy;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(SettlementPlan const &left, SettlementPlan const &right) noexcept = default;
};

/// The balance as the library writes it: "owes 12.00 EUR", "settled", or "gets back 12.00 EUR".
///
/// @param balance The balance
/// @return The text
[[nodiscard]] std::string toString(Balance const &balance) noexcept;

namespace detail {

[[nodiscard]] inline Balance fromDetail(EranosBalance const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(OwesBalance const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(SettledBalance const & /* value */) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(GetsBackBalance const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(Balance const &value) noexcept;
[[nodiscard]] inline MemberSummary fromDetail(EranosMemberSummary const &value) noexcept;
[[nodiscard]] inline CategoryTotal fromDetail(EranosCategoryTotal const &value) noexcept;
[[nodiscard]] inline MemberCategoryTotals fromDetail(EranosMemberCategoryTotals const &value) noexcept;
[[nodiscard]] inline Summary fromDetail(EranosSummary const &value) noexcept;
[[nodiscard]] inline MemberShare fromDetail(EranosMemberShare const &value) noexcept;
[[nodiscard]] inline ExpenseShares fromDetail(EranosExpenseShares const &value) noexcept;
[[nodiscard]] inline Payment fromDetail(EranosPayment const &value) noexcept;
[[nodiscard]] inline SettlementPlan fromDetail(EranosSettlementPlan const &value) noexcept;
[[nodiscard]] inline std::vector<CategoryTotal> fromDetail(EranosCategoryTotalList const &list) noexcept;
[[nodiscard]] inline std::vector<MemberCategoryTotals> fromDetail(EranosMemberCategoryTotalsList const &list) noexcept;
[[nodiscard]] inline std::vector<MemberShare> fromDetail(EranosMemberShareList const &list) noexcept;
[[nodiscard]] inline std::vector<MemberSummary> fromDetail(EranosMemberSummaryList const &list) noexcept;
[[nodiscard]] inline std::vector<Payment> fromDetail(EranosPaymentList const &list) noexcept;
[[nodiscard]] inline constexpr SettlementStrategy fromDetail(EranosSettlementStrategy const value) noexcept;
[[nodiscard]] inline constexpr EranosSettlementStrategy toDetail(SettlementStrategy const value) noexcept;

} // namespace detail

inline std::string toString(Balance const &balance) noexcept { return detail::toDetail(balance)->format(); }

} // namespace eranos

#endif // ERANOS_TOTALS_HPP
