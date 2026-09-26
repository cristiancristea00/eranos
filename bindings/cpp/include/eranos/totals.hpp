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
    /// Every debt between two members, unsimplified.
    DIRECT,
    /// A fast plan that pairs the largest debtor with the largest creditor.
    GREEDY,
    /// The fewest payments possible.
    MINIMUM_TRANSFERS,
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
    [[nodiscard]] friend bool operator==(GetsBackBalance const &left, GetsBackBalance const &right) noexcept = default;
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
    [[nodiscard]] friend bool operator==(OwesBalance const &left, OwesBalance const &right) noexcept = default;
};

/// The member owes nothing and is owed nothing.
struct SettledBalance final {
    /// Whether every field of two alternatives is equal.
    ///
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(SettledBalance const & /* left */, SettledBalance const & /* right */) noexcept = default;
};

/// Where a member stands. One of its alternatives: GetsBackBalance, OwesBalance, SettledBalance.
using Balance = std::variant<GetsBackBalance, OwesBalance, SettledBalance>;

/// Where a member stands: what they paid, consumed, sent, and received.
struct MemberSummary final {
    /// The record of its fields.
    ///
    /// @param balance Where the member stands.
    /// @param member The member's identifier.
    /// @param net What the member is owed, less what they owe: positive when they get money back.
    /// @param paid What the member paid for expenses.
    /// @param received What the member received in transfers.
    /// @param sent What the member sent in transfers.
    /// @param share The member's share of the expenses: what they consumed.
    MemberSummary(Balance balance, MemberId member, Money net, Money paid, Money received, Money sent, Money share) noexcept
        : balance(std::move(balance)), member(std::move(member)), net(std::move(net)), paid(std::move(paid)), received(std::move(received)),
          sent(std::move(sent)), share(std::move(share)) {}

    /// Where the member stands.
    Balance balance;

    /// The member's identifier.
    MemberId member;

    /// What the member is owed, less what they owe: positive when they get money back.
    Money net;

    /// What the member paid for expenses.
    Money paid;

    /// What the member received in transfers.
    Money received;

    /// What the member sent in transfers.
    Money sent;

    /// The member's share of the expenses: what they consumed.
    Money share;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(MemberSummary const &left, MemberSummary const &right) noexcept = default;
};

/// What the expenses of one category add up to.
struct CategoryTotal final {
    /// The record of its fields.
    ///
    /// @param category The category's identifier.
    /// @param total The total.
    CategoryTotal(CategoryId category, Money total) noexcept : category(std::move(category)), total(std::move(total)) {}

    /// The category's identifier.
    CategoryId category;

    /// The total.
    Money total;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(CategoryTotal const &left, CategoryTotal const &right) noexcept = default;
};

/// A member's share of each category.
struct MemberCategoryTotals final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param totals The member's total in every category they have a share in.
    MemberCategoryTotals(MemberId member, std::vector<CategoryTotal> totals) noexcept : member(std::move(member)), totals(std::move(totals)) {}

    /// The member's identifier.
    MemberId member;

    /// The member's total in every category they have a share in.
    std::vector<CategoryTotal> totals;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(MemberCategoryTotals const &left, MemberCategoryTotals const &right) noexcept = default;
};

/// The group's totals.
struct Summary final {
    /// The record of its fields.
    ///
    /// @param categoryTotals The total of every category.
    /// @param memberCategoryTotals The totals per category of every member.
    /// @param members Where every member stands.
    /// @param totalSpent What the expenses add up to.
    Summary(std::vector<CategoryTotal> categoryTotals, std::vector<MemberCategoryTotals> memberCategoryTotals, std::vector<MemberSummary> members,
            Money totalSpent) noexcept
        : categoryTotals(std::move(categoryTotals)), memberCategoryTotals(std::move(memberCategoryTotals)), members(std::move(members)),
          totalSpent(std::move(totalSpent)) {}

    /// The total of every category.
    std::vector<CategoryTotal> categoryTotals;

    /// The totals per category of every member.
    std::vector<MemberCategoryTotals> memberCategoryTotals;

    /// Where every member stands.
    std::vector<MemberSummary> members;

    /// What the expenses add up to.
    Money totalSpent;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Summary const &left, Summary const &right) noexcept = default;
};

/// A member's part of an expense.
struct MemberShare final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param paid What the member paid of it.
    /// @param share The member's share of it.
    MemberShare(MemberId member, Money paid, Money share) noexcept : member(std::move(member)), paid(std::move(paid)), share(std::move(share)) {}

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
    [[nodiscard]] friend bool operator==(MemberShare const &left, MemberShare const &right) noexcept = default;
};

/// An expense's shares, in the group currency.
struct ExpenseShares final {
    /// The record of its fields.
    ///
    /// @param members Every member's part.
    /// @param tip The tip.
    /// @param total The amount with the tip.
    ExpenseShares(std::vector<MemberShare> members, Money tip, Money total) noexcept
        : members(std::move(members)), tip(std::move(tip)), total(std::move(total)) {}

    /// Every member's part.
    std::vector<MemberShare> members;

    /// The tip.
    Money tip;

    /// The amount with the tip.
    Money total;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ExpenseShares const &left, ExpenseShares const &right) noexcept = default;
};

/// A payment of a settlement.
struct Payment final {
    /// The record of its fields.
    ///
    /// @param amount What is paid.
    /// @param from The identifier of the member who pays.
    /// @param to The identifier of the member who is paid.
    Payment(Money amount, MemberId from, MemberId to) noexcept : amount(std::move(amount)), from(std::move(from)), to(std::move(to)) {}

    /// What is paid.
    Money amount;

    /// The identifier of the member who pays.
    MemberId from;

    /// The identifier of the member who is paid.
    MemberId to;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Payment const &left, Payment const &right) noexcept = default;
};

/// A plan of payments that settles every balance.
struct Settlement final {
    /// The record of its fields.
    ///
    /// @param payments The payments.
    /// @param strategy The strategy that made the plan.
    Settlement(std::vector<Payment> payments, SettlementStrategy const strategy) noexcept : payments(std::move(payments)), strategy(strategy) {}

    /// The payments.
    std::vector<Payment> payments;

    /// The strategy that made the plan.
    SettlementStrategy strategy;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Settlement const &left, Settlement const &right) noexcept = default;
};

/// The balance as the library writes it: "gets back 12.00 EUR", "owes 12.00 EUR", or "settled".
///
/// @param balance The balance
/// @return The text
[[nodiscard]] std::string toString(Balance const &balance);

namespace detail {

[[nodiscard]] inline Balance fromDetail(EranosBalance const &value);
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(GetsBackBalance const &value);
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(OwesBalance const &value);
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(SettledBalance const & /* value */);
[[nodiscard]] inline std::unique_ptr<EranosBalance> toDetail(Balance const &value);
[[nodiscard]] inline MemberSummary fromDetail(EranosMemberSummary const &value);
[[nodiscard]] inline CategoryTotal fromDetail(EranosCategoryTotal const &value);
[[nodiscard]] inline MemberCategoryTotals fromDetail(EranosMemberCategoryTotals const &value);
[[nodiscard]] inline Summary fromDetail(EranosSummary const &value);
[[nodiscard]] inline MemberShare fromDetail(EranosMemberShare const &value);
[[nodiscard]] inline ExpenseShares fromDetail(EranosExpenseShares const &value);
[[nodiscard]] inline Payment fromDetail(EranosPayment const &value);
[[nodiscard]] inline Settlement fromDetail(EranosSettlement const &value);
[[nodiscard]] inline std::vector<CategoryTotal> fromDetail(EranosCategoryTotalList const &list);
[[nodiscard]] inline std::vector<MemberCategoryTotals> fromDetail(EranosMemberCategoryTotalsList const &list);
[[nodiscard]] inline std::vector<MemberShare> fromDetail(EranosMemberShareList const &list);
[[nodiscard]] inline std::vector<MemberSummary> fromDetail(EranosMemberSummaryList const &list);
[[nodiscard]] inline std::vector<Payment> fromDetail(EranosPaymentList const &list);
[[nodiscard]] constexpr SettlementStrategy fromDetail(EranosSettlementStrategy const value) noexcept;
[[nodiscard]] constexpr EranosSettlementStrategy toDetail(SettlementStrategy const value) noexcept;

} // namespace detail

// A balance's amount is in a currency the library lists.
inline std::string toString(Balance const &balance) { return detail::known(detail::toDetail(balance)->format()); }

} // namespace eranos

#endif // ERANOS_TOTALS_HPP
