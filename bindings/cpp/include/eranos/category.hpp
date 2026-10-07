/// @file eranos/category.hpp
/// The categories of a group's expenses.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_CATEGORY_HPP
#define ERANOS_CATEGORY_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/identifiers.hpp"

#include <memory>
#include <string>
#include <utility>
#include <vector>

namespace eranos {

/// A category of expenses.
struct Category final {
    /// The record of its required fields under the identifier @p id, such as the key an app's own store gives it; the others start absent, empty,
    /// or at their default. Category(std::string) generates the identifier instead.
    ///
    /// @param id The identifier, which the app gives: its own, or one CategoryId::generate() makes.
    /// @param name The name, which no other category of the group shares.
    Category(CategoryId id, std::string name) noexcept : id(std::move(id)), name(std::move(name)) {}

    /// The record of its required fields under an identifier generated when it is called, which Category(CategoryId, std::string) takes instead;
    /// the others start absent, empty, or at their default.
    ///
    /// @param name The name, which no other category of the group shares.
    explicit Category(std::string name) : Category(CategoryId::generate(), std::move(name)) {}

    /// The identifier, which the app gives: its own, or one CategoryId::generate() makes.
    CategoryId id;

    /// The name, which no other category of the group shares.
    std::string name;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(Category const &left, Category const &right) noexcept = default;
};

namespace detail {

[[nodiscard]] inline Category fromDetail(EranosCategory const &value);
[[nodiscard]] inline std::expected<std::unique_ptr<EranosCategory>, Error> toDetail(Category const &value);
[[nodiscard]] inline std::vector<Category> fromDetail(EranosCategoryList const &list);

} // namespace detail

} // namespace eranos

#endif // ERANOS_CATEGORY_HPP
