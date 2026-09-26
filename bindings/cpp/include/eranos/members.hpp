/// @file eranos/members.hpp
/// The members of a group and the accounts they can be paid into.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_MEMBERS_HPP
#define ERANOS_MEMBERS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/contact.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/identifiers.hpp"

#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace eranos {

/// A bank account.
struct IbanAccountDetails final {
    /// The account's IBAN.
    Iban iban;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(IbanAccountDetails const &left, IbanAccountDetails const &right) noexcept = default;
};

/// A username in a payment app.
struct UsernameAccountDetails final {
    /// The username.
    Username username;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(UsernameAccountDetails const &left, UsernameAccountDetails const &right) noexcept = default;
};

/// Where an account is: a bank account or a username in a payment app. One of its alternatives: IbanAccountDetails, UsernameAccountDetails.
using AccountDetails = std::variant<IbanAccountDetails, UsernameAccountDetails>;

/// An account a member can be paid into.
struct Account final {
    /// The record of its fields.
    ///
    /// @param details The IBAN or the username.
    /// @param name The label.
    Account(AccountDetails details, std::string name) noexcept : details(std::move(details)), name(std::move(name)) {}

    /// The IBAN or the username.
    AccountDetails details;

    /// The label.
    std::string name;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Account const &left, Account const &right) noexcept = default;
};

/// A member of the group.
struct Member final {
    /// The record of its required fields; the others start absent, empty, or at their default.
    ///
    /// @param firstName The first name.
    explicit Member(std::string firstName) noexcept : firstName(std::move(firstName)) {}

    /// The identifier: absent for a member to add, present for one the group holds.
    std::optional<MemberId> id;

    /// The first name.
    std::string firstName;

    /// Where the member can be paid, in the order they were given.
    std::vector<Account> accounts;

    /// The email address.
    std::optional<Email> email;

    /// The last name.
    std::optional<std::string> lastName;

    /// The phone number.
    std::optional<PhoneNumber> phone;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Member const &left, Member const &right) noexcept = default;
};

namespace detail {

[[nodiscard]] inline AccountDetails fromDetail(EranosAccountDetails const &value);
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(IbanAccountDetails const &value);
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(UsernameAccountDetails const &value);
[[nodiscard]] inline std::unique_ptr<EranosAccountDetails> toDetail(AccountDetails const &value);
[[nodiscard]] inline Account fromDetail(EranosAccount const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosAccount>, Error> toDetail(Account const &value);
[[nodiscard]] inline Member fromDetail(EranosMember const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMember>, Error> toDetail(Member const &value);
[[nodiscard]] inline std::vector<Account> fromDetail(EranosAccountList const &list);
[[nodiscard]] inline std::vector<Member> fromDetail(EranosMemberList const &list);

} // namespace detail

} // namespace eranos

#endif // ERANOS_MEMBERS_HPP
