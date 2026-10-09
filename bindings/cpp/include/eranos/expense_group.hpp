/// @file eranos/expense_group.hpp
/// The group, the details it starts with, the rates of a currency change, and the cancellation of asynchronous calls.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_EXPENSE_GROUP_HPP
#define ERANOS_EXPENSE_GROUP_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/category.hpp"
#include "eranos/contact.hpp"
#include "eranos/currency_code.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"
#include "eranos/expense.hpp"
#include "eranos/expense_group_reader.hpp"
#include "eranos/expense_group_writer.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/members.hpp"
#include "eranos/money.hpp"
#include "eranos/ratios.hpp"
#include "eranos/totals.hpp"
#include "eranos/transfer.hpp"

#include <expected>
#include <future>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos {

namespace detail {

[[nodiscard]] inline constexpr EranosCancellation const *handleOf(Cancellation const *const cancellation) noexcept;

} // namespace detail

/// The details a new group can start with besides its identifier, name, and currency. Each starts absent, so a designated initialiser names only
/// those it gives.
struct ExpenseGroupOptions final {
    /// A note on it.
    std::optional<std::string> note = std::nullopt;
    /// The language the apps show the group in.
    std::optional<Language> language = std::nullopt;
};

/// A cancellation of asynchronous calls, which one cancellation can be given to several of: cancelling it stops each at its
/// next yield, or at once while it waits for the lock, or, while the call is queued behind earlier calls, when its turn comes, and its future then
/// holds an error of the kind ErrorKind::CANCELLED.
class Cancellation final {
  public:
    /// A cancellation not yet cancelled.
    inline Cancellation() noexcept : handle(detail::EranosCancellation::create()) {}

    /// Cancels every call it was given.
    inline void cancel() noexcept { handle->cancel(); }

    /// Whether it was cancelled.
    ///
    /// @return Whether it was
    [[nodiscard]] inline bool isCancelled() const noexcept { return handle->is_cancelled(); }

  private:
    friend constexpr detail::EranosCancellation const *detail::handleOf(Cancellation const *const cancellation) noexcept;

    std::unique_ptr<detail::EranosCancellation> handle;
};

/// A group: its details, members, categories, expenses, transfers, totals, and settlements. It owns the group; a moved-from group may only be
/// destroyed or assigned.
class ExpenseGroup final {
  public:
    /// A new group under the identifier @p id, such as the key an app's own store gives it. The overload without @p id generates the identifier
    /// instead.
    ///
    /// @param id The group's identifier, which the app gives: its own, or one ExpenseGroupId::generate() makes
    /// @param name The group's name
    /// @param currency The group currency, which every total is in
    /// @param options The details it starts with besides
    /// @return The group, or an error of the kind ErrorKind::ID for an identifier the library refuses, ErrorKind::EXPENSE_GROUP for a blank name,
    ///   ErrorKind::LANGUAGE for a language that is not a tag, and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<ExpenseGroup, Error> create(ExpenseGroupId const &id, std::string_view const name,
                                                                   CurrencyCode const &currency, ExpenseGroupOptions const &options = {}) noexcept;

    /// A new group under an identifier generated when it is called, which the overload with an identifier takes instead.
    ///
    /// @param name The group's name
    /// @param currency The group currency, which every total is in
    /// @param options The details it starts with besides
    /// @return The group, or an error of the kind ErrorKind::EXPENSE_GROUP for a blank name, ErrorKind::LANGUAGE for a language that is not a tag,
    ///   and ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<ExpenseGroup, Error> create(std::string_view const name, CurrencyCode const &currency,
                                                                   ExpenseGroupOptions const &options = {}) noexcept;

    /// The group saved as @p json, which save() wrote.
    ///
    /// @param json The saved group
    /// @return The group, or an error of the kind ErrorKind::INVALID_JSON for text that is not a saved group
    [[nodiscard]] static std::expected<ExpenseGroup, Error> load(std::string_view const json) noexcept;

    /// The asynchronous form of load(), which runs on a thread of its own.
    ///
    /// @param json The saved group
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the group, or of the error the call ends with
    [[nodiscard]] static std::future<std::expected<ExpenseGroup, Error>> loadAsync(std::string_view const json,
                                                                                   Cancellation const *const cancellation = nullptr) noexcept;

    /// The group's identifier. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] ExpenseGroupId id() const noexcept;

    /// The asynchronous form of id(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<ExpenseGroupId, Error>> idAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// Every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers. Blocks the
    /// calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Category> categories() const noexcept;

    /// The asynchronous form of categories(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Category>, Error>>
    categoriesAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// The category with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Category>, Error> category(CategoryId const &id) const noexcept;

    /// The asynchronous form of category(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Category>, Error>>
    categoryAsync(CategoryId const &id, Cancellation const *const cancellation = nullptr) const noexcept;

    /// Adds category under its identifier, and returns the identifier. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param category The category
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<CategoryId, Error> addCategory(Category const &category) noexcept;

    /// The asynchronous form of addCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param category The category
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<CategoryId, Error>> addCategoryAsync(Category const &category,
                                                                                 Cancellation const *const cancellation = nullptr) noexcept;

    /// Renames the category that category names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param category The category
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Category, Error> replaceCategory(Category const &category) noexcept;

    /// The asynchronous form of replaceCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param category The category
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Category, Error>> replaceCategoryAsync(Category const &category,
                                                                                   Cancellation const *const cancellation = nullptr) noexcept;

    /// Removes the category with the identifier id and returns it; its expenses move to the built-in one. Blocks the calling thread while another
    /// call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Category, Error> removeCategory(CategoryId const &id) noexcept;

    /// The asynchronous form of removeCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Category, Error>> removeCategoryAsync(CategoryId const &id,
                                                                                  Cancellation const *const cancellation = nullptr) noexcept;

    /// The group currency, with what the library knows of it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] CurrencyCode currency() const noexcept;

    /// The asynchronous form of currency(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<CurrencyCode, Error>> currencyAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// Moves the group to currency, with a rate into it for every other currency its entries use. Blocks the calling thread while another call holds
    /// the group's lock.
    ///
    /// @param currency The currency
    /// @param rates The rates
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> changeCurrency(CurrencyCode const &currency, std::span<CurrencyRate const> const rates) noexcept;

    /// The asynchronous form of changeCurrency(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param currency The currency
    /// @param rates The rates
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> changeCurrencyAsync(CurrencyCode const &currency, std::span<CurrencyRate const> const rates,
                                                                              Cancellation const *const cancellation = nullptr) noexcept;

    /// Every expense, in identifier order, which is creation order for generated identifiers. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Expense> expenses() const noexcept;

    /// The asynchronous form of expenses(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Expense>, Error>>
    expensesAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// The expense with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Expense>, Error> expense(ExpenseId const &id) const noexcept;

    /// The asynchronous form of expense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Expense>, Error>>
    expenseAsync(ExpenseId const &id, Cancellation const *const cancellation = nullptr) const noexcept;

    /// How the expense with the identifier id divides, in the group currency. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<ExpenseShares>, Error> expenseShares(ExpenseId const &id) const noexcept;

    /// The asynchronous form of expenseShares(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<ExpenseShares>, Error>>
    expenseSharesAsync(ExpenseId const &id, Cancellation const *const cancellation = nullptr) const noexcept;

    /// Adds expense under its identifier, and returns the identifier. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param expense The expense
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<ExpenseId, Error> addExpense(Expense const &expense) noexcept;

    /// The asynchronous form of addExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param expense The expense
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<ExpenseId, Error>> addExpenseAsync(Expense const &expense,
                                                                               Cancellation const *const cancellation = nullptr) noexcept;

    /// Replaces the expense that expense names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param expense The expense
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Expense, Error> replaceExpense(Expense const &expense) noexcept;

    /// The asynchronous form of replaceExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param expense The expense
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Expense, Error>> replaceExpenseAsync(Expense const &expense,
                                                                                 Cancellation const *const cancellation = nullptr) noexcept;

    /// Removes the expense with the identifier id and returns it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Expense, Error> removeExpense(ExpenseId const &id) noexcept;

    /// The asynchronous form of removeExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Expense, Error>> removeExpenseAsync(ExpenseId const &id,
                                                                                Cancellation const *const cancellation = nullptr) noexcept;

    /// The group's totals. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] Summary summary() const noexcept;

    /// The asynchronous form of summary(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Summary, Error>> summaryAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// Every member, archived ones included, in identifier order, which is creation order for generated identifiers. Blocks the calling thread while
    /// another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Member> members() const noexcept;

    /// The asynchronous form of members(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Member>, Error>>
    membersAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// The member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Member>, Error> member(MemberId const &id) const noexcept;

    /// The asynchronous form of member(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Member>, Error>>
    memberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr) const noexcept;

    /// Whether the member with the identifier id is archived, or none when no member has the identifier. Blocks the calling thread while another
    /// call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<bool>, Error> isArchived(MemberId const &id) const noexcept;

    /// The asynchronous form of isArchived(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<bool>, Error>>
    isArchivedAsync(MemberId const &id, Cancellation const *const cancellation = nullptr) const noexcept;

    /// Adds member under their identifier, and returns the identifier. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param member The member
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<MemberId, Error> addMember(Member const &member) noexcept;

    /// The asynchronous form of addMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param member The member
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<MemberId, Error>> addMemberAsync(Member const &member,
                                                                             Cancellation const *const cancellation = nullptr) noexcept;

    /// Replaces the member that member names by their identifier, and returns them as they were. Blocks the calling thread while another call holds
    /// the group's lock.
    ///
    /// @param member The member
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Member, Error> replaceMember(Member const &member) noexcept;

    /// The asynchronous form of replaceMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param member The member
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Member, Error>> replaceMemberAsync(Member const &member,
                                                                               Cancellation const *const cancellation = nullptr) noexcept;

    /// Archives the member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> archiveMember(MemberId const &id) noexcept;

    /// The asynchronous form of archiveMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> archiveMemberAsync(MemberId const &id,
                                                                             Cancellation const *const cancellation = nullptr) noexcept;

    /// Unarchives the member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> unarchiveMember(MemberId const &id) noexcept;

    /// The asynchronous form of unarchiveMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> unarchiveMemberAsync(MemberId const &id,
                                                                               Cancellation const *const cancellation = nullptr) noexcept;

    /// Removes the member with the identifier id, whom no entry references, and returns them. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Member, Error> removeMember(MemberId const &id) noexcept;

    /// The asynchronous form of removeMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Member, Error>> removeMemberAsync(MemberId const &id,
                                                                              Cancellation const *const cancellation = nullptr) noexcept;

    /// The group's name. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::string name() const noexcept;

    /// The asynchronous form of name(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::string, Error>> nameAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// Renames the group. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param name The name
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> setName(std::string_view const name) noexcept;

    /// The asynchronous form of setName(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param name The name
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> setNameAsync(std::string_view const name,
                                                                       Cancellation const *const cancellation = nullptr) noexcept;

    /// The payments that settle every balance by strategy. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param strategy The strategy
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<SettlementPlan, Error> settlement(SettlementStrategy const strategy) const noexcept;

    /// The asynchronous form of settlement(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param strategy The strategy
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<SettlementPlan, Error>> settlementAsync(SettlementStrategy const strategy,
                                                                                    Cancellation const *const cancellation = nullptr) const noexcept;

    /// Every transfer, in identifier order, which is creation order for generated identifiers. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Transfer> transfers() const noexcept;

    /// The asynchronous form of transfers(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Transfer>, Error>>
    transfersAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// The transfer with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Transfer>, Error> transfer(TransferId const &id) const noexcept;

    /// The asynchronous form of transfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Transfer>, Error>>
    transferAsync(TransferId const &id, Cancellation const *const cancellation = nullptr) const noexcept;

    /// The amount in the group currency of the transfer with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Money>, Error> transferAmount(TransferId const &id) const noexcept;

    /// The asynchronous form of transferAmount(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Money>, Error>>
    transferAmountAsync(TransferId const &id, Cancellation const *const cancellation = nullptr) const noexcept;

    /// Adds transfer under its identifier, and returns the identifier. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param transfer The transfer
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<TransferId, Error> addTransfer(Transfer const &transfer) noexcept;

    /// The asynchronous form of addTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param transfer The transfer
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<TransferId, Error>> addTransferAsync(Transfer const &transfer,
                                                                                 Cancellation const *const cancellation = nullptr) noexcept;

    /// Replaces the transfer that transfer names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param transfer The transfer
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Transfer, Error> replaceTransfer(Transfer const &transfer) noexcept;

    /// The asynchronous form of replaceTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param transfer The transfer
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Transfer, Error>> replaceTransferAsync(Transfer const &transfer,
                                                                                   Cancellation const *const cancellation = nullptr) noexcept;

    /// Removes the transfer with the identifier id and returns it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Transfer, Error> removeTransfer(TransferId const &id) noexcept;

    /// The asynchronous form of removeTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Transfer, Error>> removeTransferAsync(TransferId const &id,
                                                                                  Cancellation const *const cancellation = nullptr) noexcept;

    /// The group's note. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::optional<std::string> note() const noexcept;

    /// The asynchronous form of note(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<std::string>, Error>>
    noteAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// Sets the note, trimmed, or clears it when the text is blank. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param note The note
    /// @return Nothing, or an error of the kind ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] std::expected<void, Error> setNote(std::string_view const note) noexcept;

    /// The asynchronous form of setNote(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued
    /// behind earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param note The note
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> setNoteAsync(std::string_view const note,
                                                                       Cancellation const *const cancellation = nullptr) noexcept;

    /// The language the apps show the group in, as a BCP 47 tag. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::optional<Language> language() const noexcept;

    /// The asynchronous form of language(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Language>, Error>>
    languageAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// The group in its saved form, as JSON. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::string save() const noexcept;

    /// The asynchronous form of save(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::string, Error>> saveAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// A read handle, which holds the shared lock until it is released or dropped, so that its reads see no change in between. Blocks the calling
    /// thread while another call holds the group's lock; any synchronous call of the group on the thread that holds a handle can wait forever,
    /// a change for the handle and a read behind a change another thread started, so release the handle first.
    ///
    /// @return The result
    [[nodiscard]] ExpenseGroupReader read() const noexcept;

    /// The asynchronous form of read(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<ExpenseGroupReader, Error>> readAsync(Cancellation const *const cancellation = nullptr) const noexcept;

    /// A write handle, which holds the exclusive lock until it is released or destroyed, so that its reads and changes run with no other call in
    /// between. Blocks the calling thread while another call holds the group's lock; any synchronous call of the group on the thread that holds a
    /// handle waits forever, since the handle holds the lock the call needs and the thread cannot release it meanwhile, so release the handle first.
    ///
    /// @return The result
    [[nodiscard]] ExpenseGroupWriter write() noexcept;

    /// The asynchronous form of write(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, or, while the call is queued behind
    /// earlier calls, when its turn comes, and its future then holds an error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<ExpenseGroupWriter, Error>> writeAsync(Cancellation const *const cancellation = nullptr) noexcept;

    /// Clears the group's note. Blocks the calling thread while another call holds the group's lock.
    void clearNote() noexcept;

    /// The asynchronous form of clearNote().
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> clearNoteAsync(Cancellation const *const cancellation = nullptr) noexcept;

    /// Sets the language the apps show the group in. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param language The language
    void setLanguage(Language const &language) noexcept;

    /// The asynchronous form of setLanguage().
    ///
    /// @param language The language
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> setLanguageAsync(Language const &language,
                                                                           Cancellation const *const cancellation = nullptr) noexcept;

    /// Clears the language the apps show the group in. Blocks the calling thread while another call holds the group's lock.
    void clearLanguage() noexcept;

    /// The asynchronous form of clearLanguage().
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> clearLanguageAsync(Cancellation const *const cancellation = nullptr) noexcept;

  private:
    inline constexpr explicit ExpenseGroup(std::unique_ptr<detail::EranosExpenseGroup> handle) noexcept : handle(std::move(handle)) {}

    std::unique_ptr<detail::EranosExpenseGroup> handle;
};

inline std::expected<ExpenseGroup, Error> ExpenseGroup::create(ExpenseGroupId const &id, std::string_view const name, CurrencyCode const &currency,
                                                               ExpenseGroupOptions const &options) noexcept {
    return detail::expectedOf(detail::EranosNewExpenseGroup::create(id.value(), name, currency.code()))
        .and_then([&options](std::unique_ptr<detail::EranosNewExpenseGroup> details) noexcept {
            return detail::setIfPresent(
                std::move(details), options.note,
                [](detail::EranosNewExpenseGroup &record, std::string const &text) noexcept { return record.set_note(text); });
        })
        .and_then([&options](std::unique_ptr<detail::EranosNewExpenseGroup> details) noexcept {
            return detail::setIfPresent(
                std::move(details), options.language,
                [](detail::EranosNewExpenseGroup &record, Language const &language) noexcept { return record.set_language(language.toString()); });
        })
        .and_then([](std::unique_ptr<detail::EranosNewExpenseGroup> const &details) noexcept {
            return detail::expectedOf(detail::EranosExpenseGroup::create(*details));
        })
        .transform([](std::unique_ptr<detail::EranosExpenseGroup> handle) noexcept { return ExpenseGroup(std::move(handle)); });
}

inline std::expected<ExpenseGroup, Error> ExpenseGroup::create(std::string_view const name, CurrencyCode const &currency,
                                                               ExpenseGroupOptions const &options) noexcept {
    return create(ExpenseGroupId::generate(), name, currency, options);
}

inline std::expected<ExpenseGroup, Error> ExpenseGroup::load(std::string_view const json) noexcept {
    return detail::expectedOf(detail::EranosExpenseGroup::load(json)).transform([](std::unique_ptr<detail::EranosExpenseGroup> handle) noexcept {
        return ExpenseGroup(std::move(handle));
    });
}

inline std::future<std::expected<ExpenseGroup, Error>> ExpenseGroup::loadAsync(std::string_view const json,
                                                                               Cancellation const *const cancellation) noexcept {
    return detail::launch<ExpenseGroup>(
        [&](auto callback) noexcept { detail::EranosExpenseGroup::load_async(json, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseGroupOutcome &done) noexcept { return ExpenseGroup(done.take()); });
}

inline void ExpenseGroup::clearNote() noexcept { handle->clear_note(); }

inline std::future<std::expected<void, Error>> ExpenseGroup::clearNoteAsync(Cancellation const *const cancellation) noexcept {
    return detail::launch<void>([&](auto callback) noexcept { handle->clear_note_async(detail::handleOf(cancellation), std::move(callback)); },
                                detail::finished);
}

inline void ExpenseGroup::setLanguage(Language const &language) noexcept {
    static_cast<void>(detail::waited(handle->set_language(language.toString())));
}

inline std::future<std::expected<void, Error>> ExpenseGroup::setLanguageAsync(Language const &language,
                                                                              Cancellation const *const cancellation) noexcept {
    return detail::launch<void>(
        [&](auto callback) noexcept { handle->set_language_async(language.toString(), detail::handleOf(cancellation), std::move(callback)); },
        detail::finished);
}

inline void ExpenseGroup::clearLanguage() noexcept { static_cast<void>(detail::waited(handle->clear_language())); }

inline std::future<std::expected<void, Error>> ExpenseGroup::clearLanguageAsync(Cancellation const *const cancellation) noexcept {
    return detail::launch<void>([&](auto callback) noexcept { handle->clear_language_async(detail::handleOf(cancellation), std::move(callback)); },
                                detail::finished);
}

inline ExpenseGroupId ExpenseGroup::id() const noexcept { return ExpenseGroupId(handle->id()); }

inline std::future<std::expected<ExpenseGroupId, Error>> ExpenseGroup::idAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<ExpenseGroupId>([&](auto callback) noexcept { handle->id_async(detail::handleOf(cancellation), std::move(callback)); },
                                          [](detail::EranosTextOutcome const &done) noexcept { return ExpenseGroupId(*done.value()); });
}

inline std::vector<Category> ExpenseGroup::categories() const noexcept { return detail::fromDetail(*handle->categories()); }

inline std::future<std::expected<std::vector<Category>, Error>> ExpenseGroup::categoriesAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::vector<Category>>(
        [&](auto callback) noexcept { handle->categories_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosCategoryListOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Category>, Error> ExpenseGroup::category(CategoryId const &id) const noexcept {
    return detail::expectedOf(handle->category(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Category>(value);
    });
}

inline std::future<std::expected<std::optional<Category>, Error>> ExpenseGroup::categoryAsync(CategoryId const &id,
                                                                                              Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<Category>>(
        [&](auto callback) noexcept { handle->category_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosFoundCategoryOutcome const &done) noexcept { return detail::optionalRecord<Category>(done.value()); });
}

inline std::expected<CategoryId, Error> ExpenseGroup::addCategory(Category const &category) noexcept {
    return detail::toDetail(category)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_category(*entry)); })
        .transform([](std::string id) noexcept { return CategoryId(std::move(id)); });
}

inline std::future<std::expected<CategoryId, Error>> ExpenseGroup::addCategoryAsync(Category const &category,
                                                                                    Cancellation const *const cancellation) noexcept {
    return detail::launch<CategoryId>(
        detail::toDetail(category),
        [&](auto const &entry, auto callback) noexcept { handle->add_category_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) noexcept { return CategoryId(*done.value()); });
}

inline std::expected<Category, Error> ExpenseGroup::replaceCategory(Category const &category) noexcept {
    return detail::toDetail(category)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_category(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Category, Error>> ExpenseGroup::replaceCategoryAsync(Category const &category,
                                                                                      Cancellation const *const cancellation) noexcept {
    return detail::launch<Category>(
        detail::toDetail(category),
        [&](auto const &entry, auto callback) noexcept {
            handle->replace_category_async(*entry, detail::handleOf(cancellation), std::move(callback));
        },
        [](detail::EranosCategoryOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<Category, Error> ExpenseGroup::removeCategory(CategoryId const &id) noexcept {
    return detail::expectedOf(handle->remove_category(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Category, Error>> ExpenseGroup::removeCategoryAsync(CategoryId const &id,
                                                                                     Cancellation const *const cancellation) noexcept {
    return detail::launch<Category>(
        [&](auto callback) noexcept { handle->remove_category_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosCategoryOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline CurrencyCode ExpenseGroup::currency() const noexcept { return detail::currencyOf(handle->currency()); }

inline std::future<std::expected<CurrencyCode, Error>> ExpenseGroup::currencyAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<CurrencyCode>([&](auto callback) noexcept { handle->currency_async(detail::handleOf(cancellation), std::move(callback)); },
                                        [](detail::EranosCurrencyDetailsOutcome const &done) noexcept { return detail::currencyOf(done.value()); });
}

inline std::expected<void, Error> ExpenseGroup::changeCurrency(CurrencyCode const &currency, std::span<CurrencyRate const> const rates) noexcept {
    return detail::expectedOf(handle->change_currency(currency.code(), *detail::toDetail(rates)));
}

inline std::future<std::expected<void, Error>> ExpenseGroup::changeCurrencyAsync(CurrencyCode const &currency,
                                                                                 std::span<CurrencyRate const> const rates,
                                                                                 Cancellation const *const cancellation) noexcept {
    return detail::launch<void>(
        [&](auto callback) noexcept {
            handle->change_currency_async(currency.code(), *detail::toDetail(rates), detail::handleOf(cancellation), std::move(callback));
        },
        detail::finished);
}

inline std::vector<Expense> ExpenseGroup::expenses() const noexcept { return detail::fromDetail(*handle->expenses()); }

inline std::future<std::expected<std::vector<Expense>, Error>> ExpenseGroup::expensesAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::vector<Expense>>(
        [&](auto callback) noexcept { handle->expenses_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseListOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Expense>, Error> ExpenseGroup::expense(ExpenseId const &id) const noexcept {
    return detail::expectedOf(handle->expense(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Expense>(value);
    });
}

inline std::future<std::expected<std::optional<Expense>, Error>> ExpenseGroup::expenseAsync(ExpenseId const &id,
                                                                                            Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<Expense>>(
        [&](auto callback) noexcept { handle->expense_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosFoundExpenseOutcome const &done) noexcept { return detail::optionalRecord<Expense>(done.value()); });
}

inline std::expected<std::optional<ExpenseShares>, Error> ExpenseGroup::expenseShares(ExpenseId const &id) const noexcept {
    return detail::expectedOf(handle->expense_shares(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<ExpenseShares>(value);
    });
}

inline std::future<std::expected<std::optional<ExpenseShares>, Error>>
ExpenseGroup::expenseSharesAsync(ExpenseId const &id, Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<ExpenseShares>>(
        [&](auto callback) noexcept { handle->expense_shares_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosFoundExpenseSharesOutcome const &done) noexcept { return detail::optionalRecord<ExpenseShares>(done.value()); });
}

inline std::expected<ExpenseId, Error> ExpenseGroup::addExpense(Expense const &expense) noexcept {
    return detail::toDetail(expense)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_expense(*entry)); })
        .transform([](std::string id) noexcept { return ExpenseId(std::move(id)); });
}

inline std::future<std::expected<ExpenseId, Error>> ExpenseGroup::addExpenseAsync(Expense const &expense,
                                                                                  Cancellation const *const cancellation) noexcept {
    return detail::launch<ExpenseId>(
        detail::toDetail(expense),
        [&](auto const &entry, auto callback) noexcept { handle->add_expense_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) noexcept { return ExpenseId(*done.value()); });
}

inline std::expected<Expense, Error> ExpenseGroup::replaceExpense(Expense const &expense) noexcept {
    return detail::toDetail(expense)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_expense(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Expense, Error>> ExpenseGroup::replaceExpenseAsync(Expense const &expense,
                                                                                    Cancellation const *const cancellation) noexcept {
    return detail::launch<Expense>(
        detail::toDetail(expense),
        [&](auto const &entry, auto callback) noexcept {
            handle->replace_expense_async(*entry, detail::handleOf(cancellation), std::move(callback));
        },
        [](detail::EranosExpenseOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<Expense, Error> ExpenseGroup::removeExpense(ExpenseId const &id) noexcept {
    return detail::expectedOf(handle->remove_expense(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Expense, Error>> ExpenseGroup::removeExpenseAsync(ExpenseId const &id,
                                                                                   Cancellation const *const cancellation) noexcept {
    return detail::launch<Expense>(
        [&](auto callback) noexcept { handle->remove_expense_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline Summary ExpenseGroup::summary() const noexcept { return detail::fromDetail(*handle->summary()); }

inline std::future<std::expected<Summary, Error>> ExpenseGroup::summaryAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<Summary>([&](auto callback) noexcept { handle->summary_async(detail::handleOf(cancellation), std::move(callback)); },
                                   [](detail::EranosSummaryOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::vector<Member> ExpenseGroup::members() const noexcept { return detail::fromDetail(*handle->members()); }

inline std::future<std::expected<std::vector<Member>, Error>> ExpenseGroup::membersAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::vector<Member>>(
        [&](auto callback) noexcept { handle->members_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosMemberListOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Member>, Error> ExpenseGroup::member(MemberId const &id) const noexcept {
    return detail::expectedOf(handle->member(id.value())).transform([](auto const &value) noexcept { return detail::optionalRecord<Member>(value); });
}

inline std::future<std::expected<std::optional<Member>, Error>> ExpenseGroup::memberAsync(MemberId const &id,
                                                                                          Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<Member>>(
        [&](auto callback) noexcept { handle->member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosFoundMemberOutcome const &done) noexcept { return detail::optionalRecord<Member>(done.value()); });
}

inline std::expected<std::optional<bool>, Error> ExpenseGroup::isArchived(MemberId const &id) const noexcept {
    return detail::expectedOf(handle->is_archived(id.value()));
}

inline std::future<std::expected<std::optional<bool>, Error>> ExpenseGroup::isArchivedAsync(MemberId const &id,
                                                                                            Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<bool>>(
        [&](auto callback) noexcept { handle->is_archived_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosFoundFlagOutcome const &done) noexcept { return done.value(); });
}

inline std::expected<MemberId, Error> ExpenseGroup::addMember(Member const &member) noexcept {
    return detail::toDetail(member)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_member(*entry)); })
        .transform([](std::string id) noexcept { return MemberId(std::move(id)); });
}

inline std::future<std::expected<MemberId, Error>> ExpenseGroup::addMemberAsync(Member const &member,
                                                                                Cancellation const *const cancellation) noexcept {
    return detail::launch<MemberId>(
        detail::toDetail(member),
        [&](auto const &entry, auto callback) noexcept { handle->add_member_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) noexcept { return MemberId(*done.value()); });
}

inline std::expected<Member, Error> ExpenseGroup::replaceMember(Member const &member) noexcept {
    return detail::toDetail(member)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_member(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Member, Error>> ExpenseGroup::replaceMemberAsync(Member const &member,
                                                                                  Cancellation const *const cancellation) noexcept {
    return detail::launch<Member>(
        detail::toDetail(member),
        [&](auto const &entry, auto callback) noexcept { handle->replace_member_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosMemberOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<void, Error> ExpenseGroup::archiveMember(MemberId const &id) noexcept {
    return detail::expectedOf(handle->archive_member(id.value()));
}

inline std::future<std::expected<void, Error>> ExpenseGroup::archiveMemberAsync(MemberId const &id, Cancellation const *const cancellation) noexcept {
    return detail::launch<void>(
        [&](auto callback) noexcept { handle->archive_member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        detail::finished);
}

inline std::expected<void, Error> ExpenseGroup::unarchiveMember(MemberId const &id) noexcept {
    return detail::expectedOf(handle->unarchive_member(id.value()));
}

inline std::future<std::expected<void, Error>> ExpenseGroup::unarchiveMemberAsync(MemberId const &id,
                                                                                  Cancellation const *const cancellation) noexcept {
    return detail::launch<void>(
        [&](auto callback) noexcept { handle->unarchive_member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        detail::finished);
}

inline std::expected<Member, Error> ExpenseGroup::removeMember(MemberId const &id) noexcept {
    return detail::expectedOf(handle->remove_member(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Member, Error>> ExpenseGroup::removeMemberAsync(MemberId const &id,
                                                                                 Cancellation const *const cancellation) noexcept {
    return detail::launch<Member>(
        [&](auto callback) noexcept { handle->remove_member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosMemberOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::string ExpenseGroup::name() const noexcept { return handle->name(); }

inline std::future<std::expected<std::string, Error>> ExpenseGroup::nameAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::string>([&](auto callback) noexcept { handle->name_async(detail::handleOf(cancellation), std::move(callback)); },
                                       [](detail::EranosTextOutcome const &done) noexcept { return *done.value(); });
}

inline std::expected<void, Error> ExpenseGroup::setName(std::string_view const name) noexcept { return detail::expectedOf(handle->set_name(name)); }

inline std::future<std::expected<void, Error>> ExpenseGroup::setNameAsync(std::string_view const name,
                                                                          Cancellation const *const cancellation) noexcept {
    return detail::launch<void>([&](auto callback) noexcept { handle->set_name_async(name, detail::handleOf(cancellation), std::move(callback)); },
                                detail::finished);
}

inline std::expected<SettlementPlan, Error> ExpenseGroup::settlement(SettlementStrategy const strategy) const noexcept {
    return detail::expectedOf(handle->settlement(detail::toDetail(strategy))).transform([](auto const &value) noexcept {
        return detail::fromDetail(*value);
    });
}

inline std::future<std::expected<SettlementPlan, Error>> ExpenseGroup::settlementAsync(SettlementStrategy const strategy,
                                                                                       Cancellation const *const cancellation) const noexcept {
    return detail::launch<SettlementPlan>(
        [&](auto callback) noexcept { handle->settlement_async(detail::toDetail(strategy), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosSettlementPlanOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::vector<Transfer> ExpenseGroup::transfers() const noexcept { return detail::fromDetail(*handle->transfers()); }

inline std::future<std::expected<std::vector<Transfer>, Error>> ExpenseGroup::transfersAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::vector<Transfer>>(
        [&](auto callback) noexcept { handle->transfers_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTransferListOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Transfer>, Error> ExpenseGroup::transfer(TransferId const &id) const noexcept {
    return detail::expectedOf(handle->transfer(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Transfer>(value);
    });
}

inline std::future<std::expected<std::optional<Transfer>, Error>> ExpenseGroup::transferAsync(TransferId const &id,
                                                                                              Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<Transfer>>(
        [&](auto callback) noexcept { handle->transfer_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosFoundTransferOutcome const &done) noexcept { return detail::optionalRecord<Transfer>(done.value()); });
}

inline std::expected<std::optional<Money>, Error> ExpenseGroup::transferAmount(TransferId const &id) const noexcept {
    return detail::expectedOf(handle->transfer_amount(id.value())).transform([](auto const &value) noexcept {
        return detail::optionalRecord<Money>(value);
    });
}

inline std::future<std::expected<std::optional<Money>, Error>>
ExpenseGroup::transferAmountAsync(TransferId const &id, Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<Money>>(
        [&](auto callback) noexcept { handle->transfer_amount_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosFoundMoneyOutcome const &done) noexcept { return detail::optionalRecord<Money>(done.value()); });
}

inline std::expected<TransferId, Error> ExpenseGroup::addTransfer(Transfer const &transfer) noexcept {
    return detail::toDetail(transfer)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->add_transfer(*entry)); })
        .transform([](std::string id) noexcept { return TransferId(std::move(id)); });
}

inline std::future<std::expected<TransferId, Error>> ExpenseGroup::addTransferAsync(Transfer const &transfer,
                                                                                    Cancellation const *const cancellation) noexcept {
    return detail::launch<TransferId>(
        detail::toDetail(transfer),
        [&](auto const &entry, auto callback) noexcept { handle->add_transfer_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) noexcept { return TransferId(*done.value()); });
}

inline std::expected<Transfer, Error> ExpenseGroup::replaceTransfer(Transfer const &transfer) noexcept {
    return detail::toDetail(transfer)
        .and_then([this](auto const &entry) noexcept { return detail::expectedOf(handle->replace_transfer(*entry)); })
        .transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Transfer, Error>> ExpenseGroup::replaceTransferAsync(Transfer const &transfer,
                                                                                      Cancellation const *const cancellation) noexcept {
    return detail::launch<Transfer>(
        detail::toDetail(transfer),
        [&](auto const &entry, auto callback) noexcept {
            handle->replace_transfer_async(*entry, detail::handleOf(cancellation), std::move(callback));
        },
        [](detail::EranosTransferOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::expected<Transfer, Error> ExpenseGroup::removeTransfer(TransferId const &id) noexcept {
    return detail::expectedOf(handle->remove_transfer(id.value())).transform([](auto const &value) noexcept { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Transfer, Error>> ExpenseGroup::removeTransferAsync(TransferId const &id,
                                                                                     Cancellation const *const cancellation) noexcept {
    return detail::launch<Transfer>(
        [&](auto callback) noexcept { handle->remove_transfer_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTransferOutcome const &done) noexcept { return detail::fromDetail(*done.value()); });
}

inline std::optional<std::string> ExpenseGroup::note() const noexcept { return handle->note(); }

inline std::future<std::expected<std::optional<std::string>, Error>> ExpenseGroup::noteAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<std::string>>(
        [&](auto callback) noexcept { handle->note_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosOptionalTextOutcome const &done) noexcept { return done.value(); });
}

inline std::expected<void, Error> ExpenseGroup::setNote(std::string_view const note) noexcept { return detail::expectedOf(handle->set_note(note)); }

inline std::future<std::expected<void, Error>> ExpenseGroup::setNoteAsync(std::string_view const note,
                                                                          Cancellation const *const cancellation) noexcept {
    return detail::launch<void>([&](auto callback) noexcept { handle->set_note_async(note, detail::handleOf(cancellation), std::move(callback)); },
                                detail::finished);
}

inline std::optional<Language> ExpenseGroup::language() const noexcept {
    return detail::optionalText<Language>(handle->language(), detail::languageOf);
}

inline std::future<std::expected<std::optional<Language>, Error>> ExpenseGroup::languageAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::optional<Language>>(
        [&](auto callback) noexcept { handle->language_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosOptionalTextOutcome const &done) noexcept { return detail::optionalText<Language>(done.value(), detail::languageOf); });
}

inline std::string ExpenseGroup::save() const noexcept { return handle->save(); }

inline std::future<std::expected<std::string, Error>> ExpenseGroup::saveAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<std::string>([&](auto callback) noexcept { handle->save_async(detail::handleOf(cancellation), std::move(callback)); },
                                       [](detail::EranosTextOutcome const &done) noexcept { return *done.value(); });
}

inline ExpenseGroupReader ExpenseGroup::read() const noexcept { return ExpenseGroupReader(handle->read()); }

inline std::future<std::expected<ExpenseGroupReader, Error>> ExpenseGroup::readAsync(Cancellation const *const cancellation) const noexcept {
    return detail::launch<ExpenseGroupReader>(
        [&](auto callback) noexcept { handle->read_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseGroupReaderOutcome &done) noexcept { return ExpenseGroupReader(done.take()); });
}

inline ExpenseGroupWriter ExpenseGroup::write() noexcept { return ExpenseGroupWriter(handle->write()); }

inline std::future<std::expected<ExpenseGroupWriter, Error>> ExpenseGroup::writeAsync(Cancellation const *const cancellation) noexcept {
    return detail::launch<ExpenseGroupWriter>(
        [&](auto callback) noexcept { handle->write_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseGroupWriterOutcome &done) noexcept { return ExpenseGroupWriter(done.take()); });
}

} // namespace eranos

#endif // ERANOS_EXPENSE_GROUP_HPP
