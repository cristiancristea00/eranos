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
struct IbanPaymentAccountDetails final {
    /// The account's IBAN.
    Iban iban;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(IbanPaymentAccountDetails const &left, IbanPaymentAccountDetails const &right) noexcept = default;
};

/// A username in a payment app.
struct UsernamePaymentAccountDetails final {
    /// The username.
    Username username;

    /// Whether every field of two alternatives is equal.
    ///
    /// @param left An alternative
    /// @param right Another alternative
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(UsernamePaymentAccountDetails const &left, UsernamePaymentAccountDetails const &right) noexcept = default;
};

/// Where an account is: a bank account or a username in a payment app. One of its alternatives: IbanPaymentAccountDetails,
/// UsernamePaymentAccountDetails.
using PaymentAccountDetails = std::variant<IbanPaymentAccountDetails, UsernamePaymentAccountDetails>;

/// An account a member can be paid into.
struct PaymentAccount final {
    /// The record of its fields.
    ///
    /// @param name The label.
    /// @param details The IBAN or the username.
    PaymentAccount(std::string name, PaymentAccountDetails details) noexcept : name(std::move(name)), details(std::move(details)) {}

    /// The label.
    std::string name;

    /// The IBAN or the username.
    PaymentAccountDetails details;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(PaymentAccount const &left, PaymentAccount const &right) noexcept = default;
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

    /// The last name.
    std::optional<std::string> lastName;

    /// The email address.
    std::optional<Email> email;

    /// The phone number.
    std::optional<PhoneNumber> phone;

    /// Where the member can be paid, in the order they were given.
    std::vector<PaymentAccount> accounts;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Member const &left, Member const &right) noexcept = default;
};

namespace detail {

[[nodiscard]] inline PaymentAccountDetails fromDetail(EranosPaymentAccountDetails const &value);
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(IbanPaymentAccountDetails const &value);
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(UsernamePaymentAccountDetails const &value);
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(PaymentAccountDetails const &value);
[[nodiscard]] inline PaymentAccount fromDetail(EranosPaymentAccount const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosPaymentAccount>, Error> toDetail(PaymentAccount const &value);
[[nodiscard]] inline Member fromDetail(EranosMember const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMember>, Error> toDetail(Member const &value);
[[nodiscard]] inline std::vector<PaymentAccount> fromDetail(EranosPaymentAccountList const &list);
[[nodiscard]] inline std::vector<Member> fromDetail(EranosMemberList const &list);

} // namespace detail

} // namespace eranos

#endif // ERANOS_MEMBERS_HPP
