/// @file eranos/identifiers.hpp
/// The identifiers of a group and of its members, categories, expenses, and transfers, one type for each.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_IDENTIFIERS_HPP
#define ERANOS_IDENTIFIERS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include <cstddef>
#include <functional>
#include <string>
#include <utility>

namespace eranos {

/// The identifier of a group: a hyphenated UUID the library gave it; each entity has a type of its own, so one's identifier cannot stand where
/// another's is expected.
class GroupId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text A hyphenated UUID
    explicit GroupId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text: a hyphenated UUID.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers are equal.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(GroupId const &left, GroupId const &right) noexcept { return left.text == right.text; }

    /// Whether two identifiers differ.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(GroupId const &left, GroupId const &right) noexcept { return !(left == right); }

    /// Whether @p left orders before @p right, by text.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether @p left comes first
    [[nodiscard]] friend bool operator<(GroupId const &left, GroupId const &right) noexcept { return left.text < right.text; }

  private:
    std::string text;
};

/// The identifier of a member: a hyphenated UUID the group gave them; each entity has a type of its own, so one's identifier cannot stand where
/// another's is expected.
class MemberId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text A hyphenated UUID
    explicit MemberId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text: a hyphenated UUID.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers are equal.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(MemberId const &left, MemberId const &right) noexcept { return left.text == right.text; }

    /// Whether two identifiers differ.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(MemberId const &left, MemberId const &right) noexcept { return !(left == right); }

    /// Whether @p left orders before @p right, by text.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether @p left comes first
    [[nodiscard]] friend bool operator<(MemberId const &left, MemberId const &right) noexcept { return left.text < right.text; }

  private:
    std::string text;
};

/// The identifier of a category: a hyphenated UUID the group gave it; each entity has a type of its own, so one's identifier cannot stand where
/// another's is expected.
class CategoryId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text A hyphenated UUID
    explicit CategoryId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text: a hyphenated UUID.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers are equal.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(CategoryId const &left, CategoryId const &right) noexcept { return left.text == right.text; }

    /// Whether two identifiers differ.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(CategoryId const &left, CategoryId const &right) noexcept { return !(left == right); }

    /// Whether @p left orders before @p right, by text.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether @p left comes first
    [[nodiscard]] friend bool operator<(CategoryId const &left, CategoryId const &right) noexcept { return left.text < right.text; }

  private:
    std::string text;
};

/// The identifier of an expense: a hyphenated UUID the group gave it; each entity has a type of its own, so one's identifier cannot stand where
/// another's is expected.
class ExpenseId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text A hyphenated UUID
    explicit ExpenseId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text: a hyphenated UUID.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers are equal.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ExpenseId const &left, ExpenseId const &right) noexcept { return left.text == right.text; }

    /// Whether two identifiers differ.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(ExpenseId const &left, ExpenseId const &right) noexcept { return !(left == right); }

    /// Whether @p left orders before @p right, by text.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether @p left comes first
    [[nodiscard]] friend bool operator<(ExpenseId const &left, ExpenseId const &right) noexcept { return left.text < right.text; }

  private:
    std::string text;
};

/// The identifier of a transfer: a hyphenated UUID the group gave it; each entity has a type of its own, so one's identifier cannot stand where
/// another's is expected.
class TransferId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text A hyphenated UUID
    explicit TransferId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text: a hyphenated UUID.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers are equal.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(TransferId const &left, TransferId const &right) noexcept { return left.text == right.text; }

    /// Whether two identifiers differ.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(TransferId const &left, TransferId const &right) noexcept { return !(left == right); }

    /// Whether @p left orders before @p right, by text.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether @p left comes first
    [[nodiscard]] friend bool operator<(TransferId const &left, TransferId const &right) noexcept { return left.text < right.text; }

  private:
    std::string text;
};

} // namespace eranos

/// The hash of an identifier: its text's.
template <> struct std::hash<eranos::GroupId> final {
    /// The hash of @p identifier.
    ///
    /// @param identifier The identifier
    /// @return The hash
    [[nodiscard]] std::size_t operator()(eranos::GroupId const &identifier) const noexcept { return std::hash<std::string>()(identifier.value()); }
};

/// The hash of an identifier: its text's.
template <> struct std::hash<eranos::MemberId> final {
    /// The hash of @p identifier.
    ///
    /// @param identifier The identifier
    /// @return The hash
    [[nodiscard]] std::size_t operator()(eranos::MemberId const &identifier) const noexcept { return std::hash<std::string>()(identifier.value()); }
};

/// The hash of an identifier: its text's.
template <> struct std::hash<eranos::CategoryId> final {
    /// The hash of @p identifier.
    ///
    /// @param identifier The identifier
    /// @return The hash
    [[nodiscard]] std::size_t operator()(eranos::CategoryId const &identifier) const noexcept { return std::hash<std::string>()(identifier.value()); }
};

/// The hash of an identifier: its text's.
template <> struct std::hash<eranos::ExpenseId> final {
    /// The hash of @p identifier.
    ///
    /// @param identifier The identifier
    /// @return The hash
    [[nodiscard]] std::size_t operator()(eranos::ExpenseId const &identifier) const noexcept { return std::hash<std::string>()(identifier.value()); }
};

/// The hash of an identifier: its text's.
template <> struct std::hash<eranos::TransferId> final {
    /// The hash of @p identifier.
    ///
    /// @param identifier The identifier
    /// @return The hash
    [[nodiscard]] std::size_t operator()(eranos::TransferId const &identifier) const noexcept { return std::hash<std::string>()(identifier.value()); }
};

#endif // ERANOS_IDENTIFIERS_HPP
