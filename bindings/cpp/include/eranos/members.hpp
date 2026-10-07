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
    [[nodiscard]] friend constexpr bool operator==(IbanPaymentAccountDetails const &left, IbanPaymentAccountDetails const &right) noexcept = default;
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
    [[nodiscard]] friend constexpr bool operator==(UsernamePaymentAccountDetails const &left,
                                                   UsernamePaymentAccountDetails const &right) noexcept = default;
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
    inline constexpr PaymentAccount(std::string name, PaymentAccountDetails details) noexcept : name(std::move(name)), details(std::move(details)) {}

    /// The label.
    std::string name;

    /// The IBAN or the username.
    PaymentAccountDetails details;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(PaymentAccount const &left, PaymentAccount const &right) noexcept = default;
};

/// A member of the group.
struct Member final {
    /// The record of its required fields under the identifier @p id, such as the key an app's own store gives the member; the others start absent,
    /// empty, or at their default. Member(std::string) generates the identifier instead.
    ///
    /// @param id The identifier, which the app gives: its own, or one MemberId::generate() makes.
    /// @param firstName The first name.
    inline constexpr Member(MemberId id, std::string firstName) noexcept : id(std::move(id)), firstName(std::move(firstName)) {}

    /// The record of its required fields under an identifier generated when it is called, which Member(MemberId, std::string) takes instead; the
    /// others start absent, empty, or at their default.
    ///
    /// @param firstName The first name.
    inline explicit Member(std::string firstName) noexcept : Member(MemberId::generate(), std::move(firstName)) {}

    /// The identifier, which the app gives: its own, or one MemberId::generate() makes.
    MemberId id;

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
    [[nodiscard]] friend constexpr bool operator==(Member const &left, Member const &right) noexcept = default;
};

namespace detail {

[[nodiscard]] inline PaymentAccountDetails fromDetail(EranosPaymentAccountDetails const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(IbanPaymentAccountDetails const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(UsernamePaymentAccountDetails const &value) noexcept;
[[nodiscard]] inline std::unique_ptr<EranosPaymentAccountDetails> toDetail(PaymentAccountDetails const &value) noexcept;
[[nodiscard]] inline PaymentAccount fromDetail(EranosPaymentAccount const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosPaymentAccount>, Error> toDetail(PaymentAccount const &value) noexcept;
[[nodiscard]] inline Member fromDetail(EranosMember const &value) noexcept;
[[nodiscard]] inline std::expected<std::unique_ptr<EranosMember>, Error> toDetail(Member const &value) noexcept;
[[nodiscard]] inline std::vector<PaymentAccount> fromDetail(EranosPaymentAccountList const &list) noexcept;
[[nodiscard]] inline std::vector<Member> fromDetail(EranosMemberList const &list) noexcept;

} // namespace detail

} // namespace eranos

#endif // ERANOS_MEMBERS_HPP
