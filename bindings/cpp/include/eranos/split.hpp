/// @file eranos/split.hpp
/// How an amount divides among members: equally, by exact amounts, by parts, or by percentages.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_SPLIT_HPP
#define ERANOS_SPLIT_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/money.hpp"
#include "eranos/ratios.hpp"

#include <memory>
#include <utility>
#include <variant>
#include <vector>

namespace eranos {

/// A member's exact amount in a split.
struct MemberAmount final {
    /// The record of its fields.
    ///
    /// @param amount The amount.
    /// @param member The member's identifier.
    MemberAmount(Money amount, MemberId member) noexcept : amount(std::move(amount)), member(std::move(member)) {}

    /// The amount.
    Money amount;

    /// The member's identifier.
    MemberId member;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(MemberAmount const &left, MemberAmount const &right) noexcept = default;
};

/// A member's parts in a split, in hundredths of a part.
struct MemberParts final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param parts The parts, in hundredths: 100 for one part.
    MemberParts(MemberId member, Parts const parts) noexcept : member(std::move(member)), parts(parts) {}

    /// The member's identifier.
    MemberId member;

    /// The parts, in hundredths: 100 for one part.
    Parts parts;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(MemberParts const &left, MemberParts const &right) noexcept = default;
};

/// A member's percentage in a split, in basis points.
struct MemberPercentage final {
    /// The record of its fields.
    ///
    /// @param member The member's identifier.
    /// @param percentage The percentage, in basis points: 10000 for 100 %.
    MemberPercentage(MemberId member, Percentage const percentage) noexcept : member(std::move(member)), percentage(percentage) {}

    /// The member's identifier.
    MemberId member;

    /// The percentage, in basis points: 10000 for 100 %.
    Percentage percentage;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(MemberPercentage const &left, MemberPercentage const &right) noexcept = default;
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
    [[nodiscard]] friend bool operator==(AmountsSplit const &left, AmountsSplit const &right) noexcept = default;
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
    [[nodiscard]] friend bool operator==(EqualSplit const &left, EqualSplit const &right) noexcept = default;
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
    [[nodiscard]] friend bool operator==(PartsSplit const &left, PartsSplit const &right) noexcept = default;
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
    [[nodiscard]] friend bool operator==(PercentagesSplit const &left, PercentagesSplit const &right) noexcept = default;
};

/// How an amount is divided among members. One of its alternatives: AmountsSplit, EqualSplit, PartsSplit, PercentagesSplit.
using Split = std::variant<AmountsSplit, EqualSplit, PartsSplit, PercentagesSplit>;

/// The members a split divides among, in the order it lists them.
///
/// @param split The split
/// @return The members
/// @throws Error Of the kind ErrorKind::ID for an identifier that is not one
[[nodiscard]] std::vector<MemberId> participants(Split const &split);

namespace detail {

[[nodiscard]] inline std::unique_ptr<EranosTextList> toDetail(std::vector<MemberId> const &members);
[[nodiscard]] inline MemberAmount fromDetail(EranosMemberAmount const &value);
[[nodiscard]] inline std::unique_ptr<EranosMemberAmount> toDetail(MemberAmount const &value);
[[nodiscard]] inline MemberParts fromDetail(EranosMemberParts const &value);
[[nodiscard]] inline std::unique_ptr<EranosMemberParts> toDetail(MemberParts const &value);
[[nodiscard]] inline MemberPercentage fromDetail(EranosMemberPercentage const &value);
[[nodiscard]] inline std::unique_ptr<EranosMemberPercentage> toDetail(MemberPercentage const &value);
[[nodiscard]] inline Split fromDetail(EranosSplit const &value);
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(AmountsSplit const &value);
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(EqualSplit const &value);
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(PartsSplit const &value);
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(PercentagesSplit const &value);
[[nodiscard]] inline std::unique_ptr<EranosSplit> toDetail(Split const &value);
[[nodiscard]] inline std::vector<MemberAmount> fromDetail(EranosMemberAmountList const &list);
[[nodiscard]] inline std::unique_ptr<EranosMemberAmountList> toDetail(std::vector<MemberAmount> const &records);
[[nodiscard]] inline std::vector<MemberParts> fromDetail(EranosMemberPartsList const &list);
[[nodiscard]] inline std::unique_ptr<EranosMemberPartsList> toDetail(std::vector<MemberParts> const &records);
[[nodiscard]] inline std::vector<MemberPercentage> fromDetail(EranosMemberPercentageList const &list);
[[nodiscard]] inline std::unique_ptr<EranosMemberPercentageList> toDetail(std::vector<MemberPercentage> const &records);

} // namespace detail

inline std::vector<MemberId> participants(Split const &split) {
    return detail::identifiers<MemberId>(*detail::unwrap(detail::toDetail(split)->participants()));
}

} // namespace eranos

#endif // ERANOS_SPLIT_HPP
