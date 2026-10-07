/// @file eranos/expense_group_reader.hpp
/// The read handle, which holds a group's shared lock for the reads of one report.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_EXPENSE_GROUP_READER_HPP
#define ERANOS_EXPENSE_GROUP_READER_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/category.hpp"
#include "eranos/contact.hpp"
#include "eranos/currency_code.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"
#include "eranos/expense.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/members.hpp"
#include "eranos/money.hpp"
#include "eranos/totals.hpp"
#include "eranos/transfer.hpp"

#include <expected>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace eranos {

/// A read handle: it holds the group's shared lock until it is released or destroyed, so the reads of one report see no change in between; a change
/// waits for it.
class ExpenseGroupReader final {
  public:
    /// The group's identifier.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<ExpenseGroupId, Error> id() const noexcept;

    /// Every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::vector<Category>, Error> categories() const noexcept;

    /// The category with the identifier id.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<std::optional<Category>, Error> category(CategoryId const &id) const noexcept;

    /// The ISO 4217 code of the group currency.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<CurrencyCode, Error> currency() const noexcept;

    /// Every expense, in identifier order, which is creation order for generated identifiers.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::vector<Expense>, Error> expenses() const noexcept;

    /// The expense with the identifier id.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<std::optional<Expense>, Error> expense(ExpenseId const &id) const noexcept;

    /// How the expense with the identifier id divides, in the group currency.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<std::optional<ExpenseShares>, Error> expenseShares(ExpenseId const &id) const noexcept;

    /// The group's totals.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<Summary, Error> summary() const noexcept;

    /// Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::vector<Member>, Error> members() const noexcept;

    /// The member with the identifier id.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<std::optional<Member>, Error> member(MemberId const &id) const noexcept;

    /// Whether the member with the identifier id is archived; false for an unknown member.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<bool, Error> isArchived(MemberId const &id) const noexcept;

    /// The group's name.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::string, Error> name() const noexcept;

    /// The payments that settle every balance by strategy.
    ///
    /// @param strategy The strategy
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<SettlementPlan, Error> settlement(SettlementStrategy const strategy) const noexcept;

    /// Every transfer, in identifier order, which is creation order for generated identifiers.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::vector<Transfer>, Error> transfers() const noexcept;

    /// The transfer with the identifier id.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<std::optional<Transfer>, Error> transfer(TransferId const &id) const noexcept;

    /// The amount in the group currency of the transfer with the identifier id.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<std::optional<Money>, Error> transferAmount(TransferId const &id) const noexcept;

    /// The group's description.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::optional<std::string>, Error> description() const noexcept;

    /// The language the apps show the group in, as a BCP 47 tag.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::optional<Language>, Error> language() const noexcept;

    /// The group in its saved form, as JSON.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::string, Error> save() const noexcept;

    /// Releases the lock at once; later reads return an error of the kind ErrorKind::RELEASED.
    inline void release() noexcept { handle->release(); }

  private:
    friend class ExpenseGroup;

    inline constexpr explicit ExpenseGroupReader(std::unique_ptr<detail::EranosExpenseGroupReader> handle) noexcept : handle(std::move(handle)) {}

    std::unique_ptr<detail::EranosExpenseGroupReader> handle;
};

inline std::expected<ExpenseGroupId, Error> ExpenseGroupReader::id() const noexcept {
    return detail::expectedOf(handle->id()).transform([](std::string id) noexcept { return ExpenseGroupId(std::move(id)); });
}

inline std::expected<std::vector<Category>, Error> ExpenseGroupReader::categories() const noexcept {
    return detail::expectedOf(handle->categories()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Category>, Error> ExpenseGroupReader::category(CategoryId const &id) const noexcept {
    return detail::expectedOf(handle->category(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Category>(value);
    });
}

inline std::expected<CurrencyCode, Error> ExpenseGroupReader::currency() const noexcept {
    return detail::expectedOf(handle->currency()).transform([](std::string const &code) noexcept { return detail::listedCurrency(code); });
}

inline std::expected<std::vector<Expense>, Error> ExpenseGroupReader::expenses() const noexcept {
    return detail::expectedOf(handle->expenses()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Expense>, Error> ExpenseGroupReader::expense(ExpenseId const &id) const noexcept {
    return detail::expectedOf(handle->expense(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Expense>(value);
    });
}

inline std::expected<std::optional<ExpenseShares>, Error> ExpenseGroupReader::expenseShares(ExpenseId const &id) const noexcept {
    return detail::expectedOf(handle->expense_shares(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<ExpenseShares>(value);
    });
}

inline std::expected<Summary, Error> ExpenseGroupReader::summary() const noexcept {
    return detail::expectedOf(handle->summary()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::vector<Member>, Error> ExpenseGroupReader::members() const noexcept {
    return detail::expectedOf(handle->members()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Member>, Error> ExpenseGroupReader::member(MemberId const &id) const noexcept {
    return detail::expectedOf(handle->member(id.value())).transform([](auto const &value) noexcept { return detail::optionalRecord<Member>(value); });
}

inline std::expected<bool, Error> ExpenseGroupReader::isArchived(MemberId const &id) const noexcept {
    return detail::expectedOf(handle->is_archived(id.value()));
}

inline std::expected<std::string, Error> ExpenseGroupReader::name() const noexcept { return detail::expectedOf(handle->name()); }

inline std::expected<SettlementPlan, Error> ExpenseGroupReader::settlement(SettlementStrategy const strategy) const noexcept {
    return detail::expectedOf(handle->settlement(detail::toDetail(strategy))).transform([](auto const &value) noexcept {
        return detail::fromDetail(*value);
    });
}

inline std::expected<std::vector<Transfer>, Error> ExpenseGroupReader::transfers() const noexcept {
    return detail::expectedOf(handle->transfers()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Transfer>, Error> ExpenseGroupReader::transfer(TransferId const &id) const noexcept {
    return detail::expectedOf(handle->transfer(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Transfer>(value);
    });
}

inline std::expected<std::optional<Money>, Error> ExpenseGroupReader::transferAmount(TransferId const &id) const noexcept {
    return detail::expectedOf(handle->transfer_amount(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Money>(value);
    });
}

inline std::expected<std::optional<std::string>, Error> ExpenseGroupReader::description() const noexcept {
    return detail::expectedOf(handle->description()).transform(detail::present);
}

inline std::expected<std::optional<Language>, Error> ExpenseGroupReader::language() const noexcept {
    return detail::expectedOf(handle->language()).transform([](std::string text) noexcept {
        return detail::optionalText<Language>(detail::present(std::move(text)), detail::languageOf);
    });
}

inline std::expected<std::string, Error> ExpenseGroupReader::save() const noexcept { return detail::expectedOf(handle->save()); }

} // namespace eranos

#endif // ERANOS_EXPENSE_GROUP_READER_HPP
