/// @file eranos/split.hpp
/// How an amount divides among members: equally, by parts, by percentages, or by exact amounts.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_SPLIT_HPP
#define ERANOS_SPLIT_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/money.hpp"
#include "eranos/ratios.hpp"

#include <expected>
#include <memory>
#include <utility>
#include <variant>
#include <vector>

namespace eranos {

/// A member's parts in a split, in hundredths of a part.
struct MemberParts final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param parts The parts, in hundredths: 100 for one part.
    inline constexpr MemberParts(MemberId member, Parts const parts) noexcept : member(std::move(member)), parts(parts) {}

    /// The member's identifier.
    MemberId member;

    /// The parts, in hundredths: 100 for one part.
    Parts parts;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(MemberParts const &left, MemberParts const &right) noexcept = default;
};

/// A member's percentage in a split, in basis points.
struct MemberPercentage final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param percentage The percentage, in basis points: 10000 for 100 %.
    inline constexpr MemberPercentage(MemberId member, Percentage const percentage) noexcept : member(std::move(member)), percentage(percentage) {}

    /// The member's identifier.
    MemberId member;

    /// The percentage, in basis points: 10000 for 100 %.
    Percentage percentage;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(MemberPercentage const &left, MemberPercentage const &right) noexcept = default;
};

/// A member's exact amount in a split.
struct MemberAmount final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param amount The amount.
    inline constexpr MemberAmount(MemberId member, Money amount) noexcept : member(std::move(member)), amount(std::move(amount)) {}

    /// The member's identifier.
    MemberId member;

    /// The amount.
    Money amount;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(MemberAmount const &left, MemberAmount const &right) noexcept = default;
};

/// Every member pays the same.
struct EqualSplit final {
    /// The identifiers of the members who share the amount.
    std::vector<MemberId> members;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(EqualSplit const &left, EqualSplit const &right) noexcept = default;
};

/// Every member pays in proportion to their parts.
struct PartsSplit final {
    /// Each member's parts.
    std::vector<MemberParts> shares;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(PartsSplit const &left, PartsSplit const &right) noexcept = default;
};

/// Every member pays a percentage; the percentages add up to 100 %.
struct PercentagesSplit final {
    /// Each member's percentage.
    std::vector<MemberPercentage> shares;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(PercentagesSplit const &left, PercentagesSplit const &right) noexcept = default;
};

/// Every member pays an exact amount; the amounts add up to what they divide.
struct AmountsSplit final {
    /// Each member's amount.
    std::vector<MemberAmount> shares;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(AmountsSplit const &left, AmountsSplit const &right) noexcept = default;
};

/// How an amount is divided among members. One of its alternatives: EqualSplit, PartsSplit, PercentagesSplit, AmountsSplit.
using Split = std::variant<EqualSplit, PartsSplit, PercentagesSplit, AmountsSplit>;

/// The members a split divides among, in the order it lists them.
///
/// @param split The split
/// @return The members, or an error of the kind ErrorKind::ID for an identifier that is not one
[[nodiscard]] std::expected<std::vector<MemberId>, Error> participants(Split const &split) noexcept;

namespace detail {

[[nodiscard]] inline std::expected<std::unique_ptr<EranosTextList>, Error> toDetail(std::vector<MemberId> const &members) noexcept;
[[nodiscard]] inline MemberParts fromDetail(EranosMemberParts const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberParts>, Error> toDetail(MemberParts const &value) noexcept;
[[nodiscard]] inline MemberPercentage fromDetail(EranosMemberPercentage const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberPercentage>, Error> toDetail(MemberPercentage const &value) noexcept;
[[nodiscard]] inline MemberAmount fromDetail(EranosMemberAmount const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMemberAmount>, Error> toDetail(MemberAmount const &value) noexcept;
[[nodiscard]] inline Split fromDetail(EranosSplit const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(EqualSplit const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(PartsSplit const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(PercentagesSplit const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(AmountsSplit const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosSplit>, Error> toDetail(Split const &value) noexcept;
[[nodiscard]] inline std::vector<MemberParts> fromDetail(EranosMemberPartsList const &list) noexcept;
[[nodiscard]] inline std::vector<MemberPercentage> fromDetail(EranosMemberPercentageList const &list) noexcept;
[[nodiscard]] inline std::vector<MemberAmount> fromDetail(EranosMemberAmountList const &list) noexcept;

} // namespace detail

inline std::expected<std::vector<MemberId>, Error> participants(Split const &split) noexcept {
    return detail::toDetail(split)
        .and_then([](std::unique_ptr<detail::EranosSplit> const &converted) noexcept { return detail::expectedOf(converted->participants()); })
        .transform([](std::unique_ptr<detail::EranosTextList> const &members) noexcept { return detail::identifiers<MemberId>(*members); });
}

} // namespace eranos

#endif // ERANOS_SPLIT_HPP
