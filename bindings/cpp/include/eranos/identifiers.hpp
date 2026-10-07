/// @file eranos/identifiers.hpp
/// The identifiers of a group and of its members, categories, expenses, and transfers, one type for each. Each type's generate() calls the
/// generated API, and its parse() returns an error, which names identifiers, so both are defined with the conversions, after every part.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_IDENTIFIERS_HPP
#define ERANOS_IDENTIFIERS_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"

#include <compare>
#include <cstddef>
#include <expected>
#include <functional>
#include <string>
#include <string_view>
#include <utility>

namespace eranos {

// Every identifier writes its order out: the defaulted one holds a branch that no comparison takes, which the layer's coverage would report as
// unrun. std::string compares its characters as unsigned char, even where char is signed, so the order is that of the text's bytes, the library's.

/// The identifier of a group, which the app gave it or the library generated: text of at most 64 bytes of UTF-8, not empty, without control
/// characters, and without whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a
/// type of its own, so one's identifier cannot stand where another's is expected.
class ExpenseGroupId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit ExpenseGroupId(std::string text) noexcept : text(std::move(text)) {}

    /// A new identifier: a UUID v7 in lowercase hyphenated form. Every kind of identifier draws from the one generator of the library, and
    /// identifiers generated one after another are in increasing order.
    ///
    /// @return The identifier
    [[nodiscard]] static ExpenseGroupId generate();

    /// The identifier written @p text, as it is: not trimmed, and in its case, checked by the rules every kind of identifier shares.
    ///
    /// @param text The text
    /// @return The identifier, or an error of the kind ErrorKind::ID for text that is not one, with the first rule it breaks, and
    ///   ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<ExpenseGroupId, Error> parse(std::string_view const text);

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

/// The identifier of a member, which the app gave them or the library generated: text of at most 64 bytes of UTF-8, not empty, without control
/// characters, and without whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a
/// type of its own, so one's identifier cannot stand where another's is expected.
class MemberId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit MemberId(std::string text) noexcept : text(std::move(text)) {}

    /// A new identifier: a UUID v7 in lowercase hyphenated form. Every kind of identifier draws from the one generator of the library, and
    /// identifiers generated one after another are in increasing order.
    ///
    /// @return The identifier
    [[nodiscard]] static MemberId generate();

    /// The identifier written @p text, as it is: not trimmed, and in its case, checked by the rules every kind of identifier shares.
    ///
    /// @param text The text
    /// @return The identifier, or an error of the kind ErrorKind::ID for text that is not one, with the first rule it breaks, and
    ///   ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<MemberId, Error> parse(std::string_view const text);

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

/// The identifier of a category, which the app gave it or the library generated: text of at most 64 bytes of UTF-8, not empty, without control
/// characters, and without whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a
/// type of its own, so one's identifier cannot stand where another's is expected.
class CategoryId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit CategoryId(std::string text) noexcept : text(std::move(text)) {}

    /// A new identifier: a UUID v7 in lowercase hyphenated form. Every kind of identifier draws from the one generator of the library, and
    /// identifiers generated one after another are in increasing order.
    ///
    /// @return The identifier
    [[nodiscard]] static CategoryId generate();

    /// The identifier written @p text, as it is: not trimmed, and in its case, checked by the rules every kind of identifier shares.
    ///
    /// @param text The text
    /// @return The identifier, or an error of the kind ErrorKind::ID for text that is not one, with the first rule it breaks, and
    ///   ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<CategoryId, Error> parse(std::string_view const text);

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

/// The identifier of an expense, which the app gave it or the library generated: text of at most 64 bytes of UTF-8, not empty, without control
/// characters, and without whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a
/// type of its own, so one's identifier cannot stand where another's is expected.
class ExpenseId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit ExpenseId(std::string text) noexcept : text(std::move(text)) {}

    /// A new identifier: a UUID v7 in lowercase hyphenated form. Every kind of identifier draws from the one generator of the library, and
    /// identifiers generated one after another are in increasing order.
    ///
    /// @return The identifier
    [[nodiscard]] static ExpenseId generate();

    /// The identifier written @p text, as it is: not trimmed, and in its case, checked by the rules every kind of identifier shares.
    ///
    /// @param text The text
    /// @return The identifier, or an error of the kind ErrorKind::ID for text that is not one, with the first rule it breaks, and
    ///   ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<ExpenseId, Error> parse(std::string_view const text);

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

/// The identifier of a transfer, which the app gave it or the library generated: text of at most 64 bytes of UTF-8, not empty, without control
/// characters, and without whitespace at either end, which for a generated identifier is a UUID v7 in lowercase hyphenated form; each entity has a
/// type of its own, so one's identifier cannot stand where another's is expected.
class TransferId final {
  public:
    /// The identifier written @p text, which the library checks when a call carries it.
    ///
    /// @param text The identifier's text, kept as it is: the library refuses text that is empty, longer than 64 bytes, with a control character, or
    ///   with whitespace at either end
    explicit TransferId(std::string text) noexcept : text(std::move(text)) {}

    /// A new identifier: a UUID v7 in lowercase hyphenated form. Every kind of identifier draws from the one generator of the library, and
    /// identifiers generated one after another are in increasing order.
    ///
    /// @return The identifier
    [[nodiscard]] static TransferId generate();

    /// The identifier written @p text, as it is: not trimmed, and in its case, checked by the rules every kind of identifier shares.
    ///
    /// @param text The text
    /// @return The identifier, or an error of the kind ErrorKind::ID for text that is not one, with the first rule it breaks, and
    ///   ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<TransferId, Error> parse(std::string_view const text);

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
