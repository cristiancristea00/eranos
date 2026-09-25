/// @file eranos/group_reader.hpp
/// The read handle, which holds a group's shared lock for the reads of one report.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_GROUP_READER_HPP
#define ERANOS_GROUP_READER_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/category.hpp"
#include "eranos/contact.hpp"
#include "eranos/currency.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/expense.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/members.hpp"
#include "eranos/money.hpp"
#include "eranos/totals.hpp"
#include "eranos/transfer.hpp"

#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace eranos {

/// A read handle: it holds the group's shared lock until it is released or destroyed, so the reads of one report see no change in between; a change
/// waits for it.
class GroupReader final {
  public:
    /// The group's identifier.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] GroupId id() const;

    /// Every category, the built-in one first.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::vector<Category> categories() const;

    /// The category with the identifier id.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<Category> category(CategoryId const &id) const;

    /// The ISO 4217 code of the group currency.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] Currency currency() const;

    /// Every expense, in the order they were created.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::vector<Expense> expenses() const;

    /// The expense with the identifier id.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<Expense> expense(ExpenseId const &id) const;

    /// How the expense with the identifier id divides, in the group currency.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<ExpenseShares> expenseShares(ExpenseId const &id) const;

    /// The group's totals.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] Summary summary() const;

    /// Every member, archived ones included, in the order they were created.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::vector<Member> members() const;

    /// The member with the identifier id.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<Member> member(MemberId const &id) const;

    /// Whether the member with the identifier id is archived; false for an unknown member.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] bool isArchived(MemberId const &id) const;

    /// The group's name.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::string name() const;

    /// The payments that settle every balance by strategy.
    ///
    /// @param strategy The strategy
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] Settlement settlement(SettlementStrategy const strategy) const;

    /// Every transfer, in the order they were created.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::vector<Transfer> transfers() const;

    /// The transfer with the identifier id.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<Transfer> transfer(TransferId const &id) const;

    /// The amount in the group currency of the transfer with the identifier id.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<Money> transferAmount(TransferId const &id) const;

    /// The group's description.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<std::string> description() const;

    /// The language the apps show the group in, as a BCP 47 tag.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::optional<Language> language() const;

    /// The group in its saved form, as JSON.
    ///
    /// @return The result
    /// @throws Error Of the kind ErrorKind::RELEASED after release()
    [[nodiscard]] std::string save() const;

    /// Releases the lock at once; later reads throw Error of the kind ErrorKind::RELEASED.
    void release() noexcept { handle->release(); }

  private:
    friend class Group;

    explicit GroupReader(std::unique_ptr<detail::EranosGroupReader> handle) noexcept : handle(std::move(handle)) {}

    std::unique_ptr<detail::EranosGroupReader> handle;
};

inline GroupId GroupReader::id() const { return GroupId(detail::unwrap(handle->id())); }

inline std::vector<Category> GroupReader::categories() const { return detail::fromDetail(*detail::unwrap(handle->categories())); }

inline std::optional<Category> GroupReader::category(CategoryId const &id) const {
    return detail::optionalRecord<Category>(detail::unwrap(handle->category(id.value())));
}

inline Currency GroupReader::currency() const { return Currency::of(detail::unwrap(handle->currency())); }

inline std::vector<Expense> GroupReader::expenses() const { return detail::fromDetail(*detail::unwrap(handle->expenses())); }

inline std::optional<Expense> GroupReader::expense(ExpenseId const &id) const {
    return detail::optionalRecord<Expense>(detail::unwrap(handle->expense(id.value())));
}

inline std::optional<ExpenseShares> GroupReader::expenseShares(ExpenseId const &id) const {
    return detail::optionalRecord<ExpenseShares>(detail::unwrap(handle->expense_shares(id.value())));
}

inline Summary GroupReader::summary() const { return detail::fromDetail(*detail::unwrap(handle->summary())); }

inline std::vector<Member> GroupReader::members() const { return detail::fromDetail(*detail::unwrap(handle->members())); }

inline std::optional<Member> GroupReader::member(MemberId const &id) const {
    return detail::optionalRecord<Member>(detail::unwrap(handle->member(id.value())));
}

inline bool GroupReader::isArchived(MemberId const &id) const { return detail::unwrap(handle->is_archived(id.value())); }

inline std::string GroupReader::name() const { return detail::unwrap(handle->name()); }

inline Settlement GroupReader::settlement(SettlementStrategy const strategy) const {
    return detail::fromDetail(*detail::unwrap(handle->settlement(detail::toDetail(strategy))));
}

inline std::vector<Transfer> GroupReader::transfers() const { return detail::fromDetail(*detail::unwrap(handle->transfers())); }

inline std::optional<Transfer> GroupReader::transfer(TransferId const &id) const {
    return detail::optionalRecord<Transfer>(detail::unwrap(handle->transfer(id.value())));
}

inline std::optional<Money> GroupReader::transferAmount(TransferId const &id) const {
    return detail::optionalRecord<Money>(detail::unwrap(handle->transfer_amount(id.value())));
}

inline std::optional<std::string> GroupReader::description() const { return detail::present(detail::unwrap(handle->description())); }

inline std::optional<Language> GroupReader::language() const {
    return detail::optionalText<Language>(detail::present(detail::unwrap(handle->language())), detail::languageOf);
}

inline std::string GroupReader::save() const { return detail::unwrap(handle->save()); }

} // namespace eranos

#endif // ERANOS_GROUP_READER_HPP
