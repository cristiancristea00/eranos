/// @file eranos/expense_group_writer.hpp
/// The write handle, which holds a group's exclusive lock for the reads and changes of one piece of work.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_EXPENSE_GROUP_WRITER_HPP
#define ERANOS_EXPENSE_GROUP_WRITER_HPP

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
#include "eranos/ratios.hpp"
#include "eranos/totals.hpp"
#include "eranos/transfer.hpp"

#include <expected>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos {

/// A write handle: it holds the group's exclusive lock until it is released or destroyed, so its reads and changes run with no other call in
/// between; every other call waits for it.
class ExpenseGroupWriter final {
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

    /// The group currency, with what the library knows of it.
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

    /// Whether the member with the identifier id is archived, or none when no member has the identifier.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    ///   module that refused it
    [[nodiscard]] std::expected<std::optional<bool>, Error> isArchived(MemberId const &id) const noexcept;

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

    /// The group's note.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::optional<std::string>, Error> note() const noexcept;

    /// The language the apps show the group in, as a BCP 47 tag.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::optional<Language>, Error> language() const noexcept;

    /// The group in its saved form, as JSON.
    ///
    /// @return The result, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<std::string, Error> save() const noexcept;

    /// Renames the group.
    ///
    /// @param name The name
    /// @return Nothing, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the module
    /// that refused it
    [[nodiscard]] std::expected<void, Error> setName(std::string_view const name) noexcept;

    /// Moves the group to currency, with a rate into it for every other currency its entries use.
    ///
    /// @param currency The currency
    /// @param rates The rates
    /// @return Nothing, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the module
    /// that refused it
    [[nodiscard]] std::expected<void, Error> changeCurrency(CurrencyCode const &currency, std::span<CurrencyRate const> const rates) noexcept;

    /// Sets the note, trimmed, or clears it when the text is blank.
    ///
    /// @param note The note
    /// @return Nothing, or an error of the kind ErrorKind::RELEASED after release() or ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] std::expected<void, Error> setNote(std::string_view const note) noexcept;

    /// Clears the group's note.
    ///
    /// @return Nothing, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<void, Error> clearNote() noexcept;

    /// Sets the language the apps show the group in.
    ///
    /// @param language The language
    ///
    /// @return Nothing, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<void, Error> setLanguage(Language const &language) noexcept;

    /// Clears the language the apps show the group in.
    ///
    /// @return Nothing, or an error of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::expected<void, Error> clearLanguage() noexcept;

    /// Adds member under their identifier, and returns the identifier.
    ///
    /// @param member The member
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<MemberId, Error> addMember(Member const &member) noexcept;

    /// Replaces the member that member names by their identifier, and returns them as they were.
    ///
    /// @param member The member
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Member, Error> replaceMember(Member const &member) noexcept;

    /// Archives the member with the identifier id.
    ///
    /// @param id The id
    /// @return Nothing, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the module
    /// that refused it
    [[nodiscard]] std::expected<void, Error> archiveMember(MemberId const &id) noexcept;

    /// Unarchives the member with the identifier id.
    ///
    /// @param id The id
    /// @return Nothing, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the module
    /// that refused it
    [[nodiscard]] std::expected<void, Error> unarchiveMember(MemberId const &id) noexcept;

    /// Removes the member with the identifier id, whom no entry references, and returns them.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Member, Error> removeMember(MemberId const &id) noexcept;

    /// Adds category under its identifier, and returns the identifier.
    ///
    /// @param category The category
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<CategoryId, Error> addCategory(Category const &category) noexcept;

    /// Renames the category that category names by its identifier, and returns it as it was.
    ///
    /// @param category The category
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Category, Error> replaceCategory(Category const &category) noexcept;

    /// Removes the category with the identifier id and returns it; its expenses move to the built-in one.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Category, Error> removeCategory(CategoryId const &id) noexcept;

    /// Adds expense under its identifier, and returns the identifier.
    ///
    /// @param expense The expense
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<ExpenseId, Error> addExpense(Expense const &expense) noexcept;

    /// Replaces the expense that expense names by its identifier, and returns it as it was.
    ///
    /// @param expense The expense
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Expense, Error> replaceExpense(Expense const &expense) noexcept;

    /// Removes the expense with the identifier id and returns it.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Expense, Error> removeExpense(ExpenseId const &id) noexcept;

    /// Adds transfer under its identifier, and returns the identifier.
    ///
    /// @param transfer The transfer
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<TransferId, Error> addTransfer(Transfer const &transfer) noexcept;

    /// Replaces the transfer that transfer names by its identifier, and returns it as it was.
    ///
    /// @param transfer The transfer
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Transfer, Error> replaceTransfer(Transfer const &transfer) noexcept;

    /// Removes the transfer with the identifier id and returns it.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with, of the kind ErrorKind::RELEASED after release() and otherwise of the
    /// module that refused it
    [[nodiscard]] std::expected<Transfer, Error> removeTransfer(TransferId const &id) noexcept;

    /// Releases the lock at once; later calls return an error of the kind ErrorKind::RELEASED.
    inline void release() noexcept { handle->release(); }

  private:
    friend class ExpenseGroup;

    inline constexpr explicit ExpenseGroupWriter(std::unique_ptr<detail::EranosExpenseGroupWriter> handle) noexcept : handle(std::move(handle)) {}

    std::unique_ptr<detail::EranosExpenseGroupWriter> handle;
};

inline std::expected<ExpenseGroupId, Error> ExpenseGroupWriter::id() const noexcept {
    return detail::expectedOf(handle->id()).transform([](std::string id) noexcept { return ExpenseGroupId(std::move(id)); });
}

inline std::expected<std::vector<Category>, Error> ExpenseGroupWriter::categories() const noexcept {
    return detail::expectedOf(handle->categories()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Category>, Error> ExpenseGroupWriter::category(CategoryId const &id) const noexcept {
    return detail::expectedOf(handle->category(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Category>(value);
    });
}

inline std::expected<CurrencyCode, Error> ExpenseGroupWriter::currency() const noexcept {
    return detail::expectedOf(handle->currency()).transform([](std::unique_ptr<detail::EranosCurrencyDetails> details) noexcept {
        return detail::currencyOf(std::move(details));
    });
}

inline std::expected<std::vector<Expense>, Error> ExpenseGroupWriter::expenses() const noexcept {
    return detail::expectedOf(handle->expenses()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Expense>, Error> ExpenseGroupWriter::expense(ExpenseId const &id) const noexcept {
    return detail::expectedOf(handle->expense(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Expense>(value);
    });
}

inline std::expected<std::optional<ExpenseShares>, Error> ExpenseGroupWriter::expenseShares(ExpenseId const &id) const noexcept {
    return detail::expectedOf(handle->expense_shares(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<ExpenseShares>(value);
    });
}

inline std::expected<Summary, Error> ExpenseGroupWriter::summary() const noexcept {
    return detail::expectedOf(handle->summary()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::vector<Member>, Error> ExpenseGroupWriter::members() const noexcept {
    return detail::expectedOf(handle->members()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Member>, Error> ExpenseGroupWriter::member(MemberId const &id) const noexcept {
    return detail::expectedOf(handle->member(id.value())).transform([](auto const &value) noexcept { return detail::optionalRecord<Member>(value); });
}

inline std::expected<std::optional<bool>, Error> ExpenseGroupWriter::isArchived(MemberId const &id) const noexcept {
    return detail::expectedOf(handle->is_archived(id.value()));
}

inline std::expected<std::string, Error> ExpenseGroupWriter::name() const noexcept { return detail::expectedOf(handle->name()); }

inline std::expected<SettlementPlan, Error> ExpenseGroupWriter::settlement(SettlementStrategy const strategy) const noexcept {
    return detail::expectedOf(handle->settlement(detail::toDetail(strategy))).transform([](auto const &value) noexcept {
        return detail::fromDetail(*value);
    });
}

inline std::expected<std::vector<Transfer>, Error> ExpenseGroupWriter::transfers() const noexcept {
    return detail::expectedOf(handle->transfers()).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<std::optional<Transfer>, Error> ExpenseGroupWriter::transfer(TransferId const &id) const noexcept {
    return detail::expectedOf(handle->transfer(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Transfer>(value);
    });
}

inline std::expected<std::optional<Money>, Error> ExpenseGroupWriter::transferAmount(TransferId const &id) const noexcept {
    return detail::expectedOf(handle->transfer_amount(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Money>(value);
    });
}

inline std::expected<std::optional<std::string>, Error> ExpenseGroupWriter::note() const noexcept {
    return detail::expectedOf(handle->note()).transform(detail::present);
}

inline std::expected<std::optional<Language>, Error> ExpenseGroupWriter::language() const noexcept {
    return detail::expectedOf(handle->language()).transform([](std::string text) noexcept {
        return detail::optionalText<Language>(detail::present(std::move(text)), detail::languageOf);
    });
}

inline std::expected<std::string, Error> ExpenseGroupWriter::save() const noexcept { return detail::expectedOf(handle->save()); }

inline std::expected<void, Error> ExpenseGroupWriter::setName(std::string_view const name) noexcept {
    return detail::expectedOf(handle->set_name(name));
}

inline std::expected<void, Error> ExpenseGroupWriter::changeCurrency(CurrencyCode const &currency,
                                                                     std::span<CurrencyRate const> const rates) noexcept {
    return detail::expectedOf(handle->change_currency(currency.code(), *detail::toDetail(rates)));
}

inline std::expected<void, Error> ExpenseGroupWriter::setNote(std::string_view const note) noexcept {
    return detail::expectedOf(handle->set_note(note));
}

inline std::expected<void, Error> ExpenseGroupWriter::clearNote() noexcept { return detail::expectedOf(handle->clear_note()); }

inline std::expected<void, Error> ExpenseGroupWriter::setLanguage(Language const &language) noexcept {
    return detail::expectedOf(handle->set_language(language.toString()));
}

inline std::expected<void, Error> ExpenseGroupWriter::clearLanguage() noexcept { return detail::expectedOf(handle->clear_language()); }

inline std::expected<MemberId, Error> ExpenseGroupWriter::addMember(Member const &member) noexcept {
    return detail::toDetail(member)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_member(*entry)); })
        .transform([](std::string id) noexcept { return MemberId(std::move(id)); });
}

inline std::expected<Member, Error> ExpenseGroupWriter::replaceMember(Member const &member) noexcept {
    return detail::toDetail(member)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_member(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<void, Error> ExpenseGroupWriter::archiveMember(MemberId const &id) noexcept {
    return detail::expectedOf(handle->archive_member(id.value()));
}

inline std::expected<void, Error> ExpenseGroupWriter::unarchiveMember(MemberId const &id) noexcept {
    return detail::expectedOf(handle->unarchive_member(id.value()));
}

inline std::expected<Member, Error> ExpenseGroupWriter::removeMember(MemberId const &id) noexcept {
    return detail::expectedOf(handle->remove_member(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<CategoryId, Error> ExpenseGroupWriter::addCategory(Category const &category) noexcept {
    return detail::toDetail(category)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_category(*entry)); })
        .transform([](std::string id) noexcept { return CategoryId(std::move(id)); });
}

inline std::expected<Category, Error> ExpenseGroupWriter::replaceCategory(Category const &category) noexcept {
    return detail::toDetail(category)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_category(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<Category, Error> ExpenseGroupWriter::removeCategory(CategoryId const &id) noexcept {
    return detail::expectedOf(handle->remove_category(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<ExpenseId, Error> ExpenseGroupWriter::addExpense(Expense const &expense) noexcept {
    return detail::toDetail(expense)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_expense(*entry)); })
        .transform([](std::string id) noexcept { return ExpenseId(std::move(id)); });
}

inline std::expected<Expense, Error> ExpenseGroupWriter::replaceExpense(Expense const &expense) noexcept {
    return detail::toDetail(expense)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_expense(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<Expense, Error> ExpenseGroupWriter::removeExpense(ExpenseId const &id) noexcept {
    return detail::expectedOf(handle->remove_expense(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<TransferId, Error> ExpenseGroupWriter::addTransfer(Transfer const &transfer) noexcept {
    return detail::toDetail(transfer)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_transfer(*entry)); })
        .transform([](std::string id) noexcept { return TransferId(std::move(id)); });
}

inline std::expected<Transfer, Error> ExpenseGroupWriter::replaceTransfer(Transfer const &transfer) noexcept {
    return detail::toDetail(transfer)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_transfer(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::expected<Transfer, Error> ExpenseGroupWriter::removeTransfer(TransferId const &id) noexcept {
    return detail::expectedOf(handle->remove_transfer(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

} // namespace eranos

#endif // ERANOS_EXPENSE_GROUP_WRITER_HPP
