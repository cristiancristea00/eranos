/// @file eranos/identifiers.hpp
/// The identifiers of a group and of its members, categories, expenses, and transfers, one type for each.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_IDENTIFIERS_HPP
#define ERANOS_IDENTIFIERS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include <compare>
#include <cstddef>
#include <functional>
#include <string>
#include <utility>

namespace eranos {

// Every identifier writes its order out: the defaulted one holds a branch that no comparison takes, which the layer's coverage would report as
// unrun. std::string compares its characters as unsigned char, even where char is signed, so the order is that of the text's bytes, the library's.

/// The identifier of a group, which the library gave it: text of at most 64 bytes of UTF-8, not empty, without control characters, and without
/// whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a type of its own, so one's
/// identifier cannot stand where another's is expected.
class ExpenseGroupId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit ExpenseGroupId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text, as it was given or generated.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers have one text, byte for byte, so case matters: equal ones name one entity.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ExpenseGroupId const &left, ExpenseGroupId const &right) noexcept = default;

    /// How two identifiers order: by the bytes of their text, as the library orders them, which is creation order for generated identifiers.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Their order
    [[nodiscard]] friend std::strong_ordering operator<=>(ExpenseGroupId const &left, ExpenseGroupId const &right) noexcept {
        return left.text <=> right.text;
    }

  private:
    std::string text;
};

/// The identifier of a member, which the group gave them: text of at most 64 bytes of UTF-8, not empty, without control characters, and without
/// whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a type of its own, so one's
/// identifier cannot stand where another's is expected.
class MemberId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit MemberId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text, as it was given or generated.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers have one text, byte for byte, so case matters: equal ones name one entity.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(MemberId const &left, MemberId const &right) noexcept = default;

    /// How two identifiers order: by the bytes of their text, as the library orders them, which is creation order for generated identifiers.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Their order
    [[nodiscard]] friend std::strong_ordering operator<=>(MemberId const &left, MemberId const &right) noexcept { return left.text <=> right.text; }

  private:
    std::string text;
};

/// The identifier of a category, which the group gave it: text of at most 64 bytes of UTF-8, not empty, without control characters, and without
/// whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a type of its own, so one's
/// identifier cannot stand where another's is expected.
class CategoryId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit CategoryId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text, as it was given or generated.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers have one text, byte for byte, so case matters: equal ones name one entity.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(CategoryId const &left, CategoryId const &right) noexcept = default;

    /// How two identifiers order: by the bytes of their text, as the library orders them, which is creation order for generated identifiers.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Their order
    [[nodiscard]] friend std::strong_ordering operator<=>(CategoryId const &left, CategoryId const &right) noexcept {
        return left.text <=> right.text;
    }

  private:
    std::string text;
};

/// The identifier of an expense, which the group gave it: text of at most 64 bytes of UTF-8, not empty, without control characters, and without
/// whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a type of its own, so one's
/// identifier cannot stand where another's is expected.
class ExpenseId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit ExpenseId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text, as it was given or generated.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers have one text, byte for byte, so case matters: equal ones name one entity.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(ExpenseId const &left, ExpenseId const &right) noexcept = default;

    /// How two identifiers order: by the bytes of their text, as the library orders them, which is creation order for generated identifiers.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Their order
    [[nodiscard]] friend std::strong_ordering operator<=>(ExpenseId const &left, ExpenseId const &right) noexcept { return left.text <=> right.text; }

  private:
    std::string text;
};

/// The identifier of a transfer, which the group gave it: text of at most 64 bytes of UTF-8, not empty, without control characters, and without
/// whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a type of its own, so one's
/// identifier cannot stand where another's is expected.
class TransferId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit TransferId(std::string text) noexcept : text(std::move(text)) {}

    /// The identifier's text, as it was given or generated.
    ///
    /// @return The text
    [[nodiscard]] std::string const &value() const noexcept { return text; }

    /// Whether two identifiers have one text, byte for byte, so case matters: equal ones name one entity.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(TransferId const &left, TransferId const &right) noexcept = default;

    /// How two identifiers order: by the bytes of their text, as the library orders them, which is creation order for generated identifiers.
    ///
    /// @param left An identifier
    /// @param right Another identifier
    /// @return Their order
    [[nodiscard]] friend std::strong_ordering operator<=>(TransferId const &left, TransferId const &right) noexcept {
        return left.text <=> right.text;
    }

  private:
    std::string text;
};

} // namespace eranos

/// The hash of an identifier: its text's.
template <> struct std::hash<eranos::ExpenseGroupId> final {
    /// The hash of @p identifier.
    ///
    /// @param identifier The identifier
    /// @return The hash
    [[nodiscard]] std::size_t operator()(eranos::ExpenseGroupId const &identifier) const noexcept {
        return std::hash<std::string>()(identifier.value());
    }
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
