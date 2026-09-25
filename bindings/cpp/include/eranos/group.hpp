/// @file eranos/group.hpp
/// The group, the details it starts with, the rates of a currency change, and the cancellation of asynchronous calls.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_GROUP_HPP
#define ERANOS_GROUP_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/category.hpp"
#include "eranos/contact.hpp"
#include "eranos/currency.hpp"
#include "eranos/detail/declarations.hpp"
#include "eranos/expense.hpp"
#include "eranos/group_reader.hpp"
#include "eranos/identifiers.hpp"
#include "eranos/members.hpp"
#include "eranos/money.hpp"
#include "eranos/ratios.hpp"
#include "eranos/totals.hpp"
#include "eranos/transfer.hpp"

#include <future>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace eranos {

namespace detail {

[[nodiscard]] inline EranosCancellation const *handleOf(Cancellation const *const cancellation) noexcept;

} // namespace detail

/// The rate into a new group currency of an amount in another.
struct CurrencyRate final {
    /// The record of its fields.
    ///
    /// @param currency The ISO 4217 code of the currency the rate converts from.
    /// @param rate The rate, as a decimal.
    CurrencyRate(Currency currency, ExchangeRate rate) noexcept : currency(std::move(currency)), rate(std::move(rate)) {}

    /// The ISO 4217 code of the currency the rate converts from.
    Currency currency;

    /// The rate, as a decimal.
    ExchangeRate rate;

    /// Whether every field of two records is equal.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they are equal
    [[nodiscard]] friend bool operator==(CurrencyRate const &left, CurrencyRate const &right) noexcept {
        return left.currency == right.currency && left.rate == right.rate;
    }

    /// Whether a field of two records differs.
    ///
    /// @param left A record
    /// @param right Another record
    /// @return Whether they differ
    [[nodiscard]] friend bool operator!=(CurrencyRate const &left, CurrencyRate const &right) noexcept { return !(left == right); }
};

/// The details a new group can start with besides its currency and name.
struct GroupOptions final {
    /// A longer description.
    std::optional<std::string> description;
    /// The language the apps show the group in.
    std::optional<Language> language;
};

/// A cancellation of asynchronous calls, which one cancellation can be given to several of: cancelling it stops each at its
/// next yield, or at once while it waits for the lock, and its future then throws Error of the kind ErrorKind::CANCELLED.
class Cancellation final {
  public:
    /// A cancellation not yet cancelled.
    Cancellation() noexcept : handle(detail::EranosCancellation::create()) {}

    /// Cancels every call it was given.
    void cancel() noexcept { handle->cancel(); }

    /// Whether it was cancelled.
    ///
    /// @return Whether it was
    [[nodiscard]] bool isCancelled() const noexcept { return handle->is_cancelled(); }

  private:
    friend detail::EranosCancellation const *detail::handleOf(Cancellation const *const cancellation) noexcept;

    std::unique_ptr<detail::EranosCancellation> handle;
};

/// A group: its details, members, categories, expenses, transfers, totals, and settlements. It owns the group; a moved-from group may only be
/// destroyed or assigned.
class Group final {
  public:
    /// A new group.
    ///
    /// @param currency The group currency, which every total is in
    /// @param name The group's name
    /// @param options The details it starts with besides
    /// @return The group
    /// @throws Error Of the kind ErrorKind::GROUP for a blank name, ErrorKind::LANGUAGE for a language that is not a tag, and
    ///   ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static Group create(Currency const &currency, std::string_view const name, GroupOptions const &options = {});

    /// The group saved as @p json, which save() wrote.
    ///
    /// @param json The saved group
    /// @return The group
    /// @throws Error Of the kind ErrorKind::INVALID_JSON for text that is not a saved group
    [[nodiscard]] static Group load(std::string_view const json);

    /// The asynchronous form of load(), which runs on a thread of its own.
    ///
    /// @param json The saved group
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the group
    [[nodiscard]] static std::future<Group> loadAsync(std::string_view const json, Cancellation const *const cancellation = nullptr);

    /// The group's identifier. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] GroupId id() const;

    /// The asynchronous form of id(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<GroupId> idAsync(Cancellation const *const cancellation = nullptr) const;

    /// Every category, the built-in one first. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Category> categories() const;

    /// The asynchronous form of categories(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::vector<Category>> categoriesAsync(Cancellation const *const cancellation = nullptr) const;

    /// The category with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] std::optional<Category> category(CategoryId const &id) const;

    /// The asynchronous form of category(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<Category>> categoryAsync(CategoryId const &id, Cancellation const *const cancellation = nullptr) const;

    /// Adds category, which has no identifier yet, and returns the one it gets. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param category The category
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] CategoryId addCategory(Category const &category);

    /// The asynchronous form of addCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param category The category
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<CategoryId> addCategoryAsync(Category const &category, Cancellation const *const cancellation = nullptr);

    /// Renames the category that category names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param category The category
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Category replaceCategory(Category const &category);

    /// The asynchronous form of replaceCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param category The category
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Category> replaceCategoryAsync(Category const &category, Cancellation const *const cancellation = nullptr);

    /// Removes the category with the identifier id and returns it; its expenses move to the built-in one. Blocks the calling thread while another
    /// call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Category removeCategory(CategoryId const &id);

    /// The asynchronous form of removeCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Category> removeCategoryAsync(CategoryId const &id, Cancellation const *const cancellation = nullptr);

    /// The ISO 4217 code of the group currency. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] Currency currency() const;

    /// The asynchronous form of currency(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Currency> currencyAsync(Cancellation const *const cancellation = nullptr) const;

    /// Moves the group to currency, with a rate into it for every other currency its entries use. Blocks the calling thread while another call holds
    /// the group's lock.
    ///
    /// @param currency The currency
    /// @param rates The rates
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    void changeCurrency(Currency const &currency, std::vector<CurrencyRate> const &rates);

    /// The asynchronous form of changeCurrency(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param currency The currency
    /// @param rates The rates
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<void> changeCurrencyAsync(Currency const &currency, std::vector<CurrencyRate> const &rates,
                                                        Cancellation const *const cancellation = nullptr);

    /// Every expense, in the order they were created. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Expense> expenses() const;

    /// The asynchronous form of expenses(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::vector<Expense>> expensesAsync(Cancellation const *const cancellation = nullptr) const;

    /// The expense with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] std::optional<Expense> expense(ExpenseId const &id) const;

    /// The asynchronous form of expense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<Expense>> expenseAsync(ExpenseId const &id, Cancellation const *const cancellation = nullptr) const;

    /// How the expense with the identifier id divides, in the group currency. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] std::optional<ExpenseShares> expenseShares(ExpenseId const &id) const;

    /// The asynchronous form of expenseShares(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<ExpenseShares>> expenseSharesAsync(ExpenseId const &id,
                                                                               Cancellation const *const cancellation = nullptr) const;

    /// Adds expense, which has no identifier yet, and returns the one it gets. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param expense The expense
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] ExpenseId addExpense(Expense const &expense);

    /// The asynchronous form of addExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param expense The expense
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<ExpenseId> addExpenseAsync(Expense const &expense, Cancellation const *const cancellation = nullptr);

    /// Replaces the expense that expense names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param expense The expense
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Expense replaceExpense(Expense const &expense);

    /// The asynchronous form of replaceExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param expense The expense
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Expense> replaceExpenseAsync(Expense const &expense, Cancellation const *const cancellation = nullptr);

    /// Removes the expense with the identifier id and returns it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Expense removeExpense(ExpenseId const &id);

    /// The asynchronous form of removeExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Expense> removeExpenseAsync(ExpenseId const &id, Cancellation const *const cancellation = nullptr);

    /// The group's totals. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] Summary summary() const;

    /// The asynchronous form of summary(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Summary> summaryAsync(Cancellation const *const cancellation = nullptr) const;

    /// Every member, archived ones included, in the order they were created. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Member> members() const;

    /// The asynchronous form of members(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::vector<Member>> membersAsync(Cancellation const *const cancellation = nullptr) const;

    /// The member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] std::optional<Member> member(MemberId const &id) const;

    /// The asynchronous form of member(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<Member>> memberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr) const;

    /// Whether the member with the identifier id is archived; false for an unknown member. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] bool isArchived(MemberId const &id) const;

    /// The asynchronous form of isArchived(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<bool> isArchivedAsync(MemberId const &id, Cancellation const *const cancellation = nullptr) const;

    /// Adds member, who has no identifier yet, and returns the one they get. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param member The member
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] MemberId addMember(Member const &member);

    /// The asynchronous form of addMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param member The member
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<MemberId> addMemberAsync(Member const &member, Cancellation const *const cancellation = nullptr);

    /// Replaces the member that member names by their identifier, and returns them as they were. Blocks the calling thread while another call holds
    /// the group's lock.
    ///
    /// @param member The member
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Member replaceMember(Member const &member);

    /// The asynchronous form of replaceMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param member The member
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Member> replaceMemberAsync(Member const &member, Cancellation const *const cancellation = nullptr);

    /// Archives the member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    void archiveMember(MemberId const &id);

    /// The asynchronous form of archiveMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<void> archiveMemberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr);

    /// Unarchives the member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    void unarchiveMember(MemberId const &id);

    /// The asynchronous form of unarchiveMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<void> unarchiveMemberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr);

    /// Removes the member with the identifier id, whom no entry references, and returns them. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Member removeMember(MemberId const &id);

    /// The asynchronous form of removeMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Member> removeMemberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr);

    /// The group's name. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::string name() const;

    /// The asynchronous form of name(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::string> nameAsync(Cancellation const *const cancellation = nullptr) const;

    /// Renames the group. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param name The name
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    void setName(std::string_view const name);

    /// The asynchronous form of setName(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param name The name
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<void> setNameAsync(std::string_view const name, Cancellation const *const cancellation = nullptr);

    /// The payments that settle every balance by strategy. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param strategy The strategy
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Settlement settlement(SettlementStrategy const strategy) const;

    /// The asynchronous form of settlement(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param strategy The strategy
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Settlement> settlementAsync(SettlementStrategy const strategy, Cancellation const *const cancellation = nullptr) const;

    /// Every transfer, in the order they were created. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Transfer> transfers() const;

    /// The asynchronous form of transfers(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::vector<Transfer>> transfersAsync(Cancellation const *const cancellation = nullptr) const;

    /// The transfer with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] std::optional<Transfer> transfer(TransferId const &id) const;

    /// The asynchronous form of transfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<Transfer>> transferAsync(TransferId const &id, Cancellation const *const cancellation = nullptr) const;

    /// The amount in the group currency of the transfer with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] std::optional<Money> transferAmount(TransferId const &id) const;

    /// The asynchronous form of transferAmount(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<Money>> transferAmountAsync(TransferId const &id, Cancellation const *const cancellation = nullptr) const;

    /// Adds transfer, which has no identifier yet, and returns the one it gets. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param transfer The transfer
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] TransferId addTransfer(Transfer const &transfer);

    /// The asynchronous form of addTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param transfer The transfer
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<TransferId> addTransferAsync(Transfer const &transfer, Cancellation const *const cancellation = nullptr);

    /// Replaces the transfer that transfer names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param transfer The transfer
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Transfer replaceTransfer(Transfer const &transfer);

    /// The asynchronous form of replaceTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param transfer The transfer
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Transfer> replaceTransferAsync(Transfer const &transfer, Cancellation const *const cancellation = nullptr);

    /// Removes the transfer with the identifier id and returns it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result
    /// @throws Error When the library refuses the call; its kind names the module that refused it
    [[nodiscard]] Transfer removeTransfer(TransferId const &id);

    /// The asynchronous form of removeTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<Transfer> removeTransferAsync(TransferId const &id, Cancellation const *const cancellation = nullptr);

    /// The group's description. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::optional<std::string> description() const;

    /// The asynchronous form of description(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<std::string>> descriptionAsync(Cancellation const *const cancellation = nullptr) const;

    /// Sets the description; blank text clears it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param description The description
    void setDescription(std::string_view const description);

    /// The asynchronous form of setDescription(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error
    /// of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param description The description
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<void> setDescriptionAsync(std::string_view const description, Cancellation const *const cancellation = nullptr);

    /// The language the apps show the group in, as a BCP 47 tag. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::optional<Language> language() const;

    /// The asynchronous form of language(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::optional<Language>> languageAsync(Cancellation const *const cancellation = nullptr) const;

    /// The group in its saved form, as JSON. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::string save() const;

    /// The asynchronous form of save(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<std::string> saveAsync(Cancellation const *const cancellation = nullptr) const;

    /// A read handle, which holds the shared lock until it is released or dropped, so that its reads see no change in between. Blocks the calling
    /// thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] GroupReader read() const;

    /// The asynchronous form of read(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then throws Error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, whose get() throws what the synchronous form throws
    [[nodiscard]] std::future<GroupReader> readAsync(Cancellation const *const cancellation = nullptr) const;

    /// Clears the group's description. Blocks the calling thread while another call holds the group's lock.
    void clearDescription();

    /// The asynchronous form of clearDescription().
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call
    [[nodiscard]] std::future<void> clearDescriptionAsync(Cancellation const *const cancellation = nullptr);

    /// Sets the language the apps show the group in. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param language The language
    void setLanguage(Language const &language);

    /// The asynchronous form of setLanguage().
    ///
    /// @param language The language
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call
    [[nodiscard]] std::future<void> setLanguageAsync(Language const &language, Cancellation const *const cancellation = nullptr);

    /// Clears the language the apps show the group in. Blocks the calling thread while another call holds the group's lock.
    void clearLanguage();

    /// The asynchronous form of clearLanguage().
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call
    [[nodiscard]] std::future<void> clearLanguageAsync(Cancellation const *const cancellation = nullptr);

  private:
    explicit Group(std::unique_ptr<detail::EranosGroup> handle) noexcept : handle(std::move(handle)) {}

    std::unique_ptr<detail::EranosGroup> handle;
};

namespace detail {

[[nodiscard]] inline std::unique_ptr<EranosCurrencyRate> toDetail(CurrencyRate const &value);
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRateList> toDetail(std::vector<CurrencyRate> const &records);

} // namespace detail

inline Group Group::create(Currency const &currency, std::string_view const name, GroupOptions const &options) {
    auto const details = detail::unwrap(detail::EranosNewGroup::create(currency.code(), name));
    if (options.description) {
        detail::check(details->set_description(*options.description));
    }
    if (options.language) {
        detail::check(details->set_language(options.language->toString()));
    }
    return Group(detail::unwrap(detail::EranosGroup::create(*details)));
}

inline Group Group::load(std::string_view const json) { return Group(detail::unwrap(detail::EranosGroup::load(json))); }

inline std::future<Group> Group::loadAsync(std::string_view const json, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Group>>();
    auto future = promise->get_future();
    detail::EranosGroup::load_async(json, detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosGroupOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosGroupOutcome &done) { return Group(done.take()); });
    });
    return future;
}

inline void Group::clearDescription() { detail::check(handle->set_description("")); }

inline std::future<void> Group::clearDescriptionAsync(Cancellation const *const cancellation) { return setDescriptionAsync("", cancellation); }

inline void Group::setLanguage(Language const &language) { detail::check(handle->set_language(language.toString())); }

inline std::future<void> Group::setLanguageAsync(Language const &language, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<void>>();
    auto future = promise->get_future();
    handle->set_language_async(language.toString(), detail::handleOf(cancellation),
                               [promise](std::unique_ptr<detail::EranosUnitOutcome> const outcome) { detail::complete(*promise, *outcome); });
    return future;
}

inline void Group::clearLanguage() { detail::check(handle->clear_language()); }

inline std::future<void> Group::clearLanguageAsync(Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<void>>();
    auto future = promise->get_future();
    handle->clear_language_async(detail::handleOf(cancellation),
                                 [promise](std::unique_ptr<detail::EranosUnitOutcome> const outcome) { detail::complete(*promise, *outcome); });
    return future;
}

inline GroupId Group::id() const { return GroupId(detail::unwrap(handle->id())); }

inline std::future<GroupId> Group::idAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<GroupId>>();
    auto future = promise->get_future();
    handle->id_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return GroupId(*done.value()); });
    });
    return future;
}

inline std::vector<Category> Group::categories() const { return detail::fromDetail(*detail::unwrap(handle->categories())); }

inline std::future<std::vector<Category>> Group::categoriesAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::vector<Category>>>();
    auto future = promise->get_future();
    handle->categories_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosCategoryListOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosCategoryListOutcome const &done) { return detail::fromDetail(*done.value()); });
    });
    return future;
}

inline std::optional<Category> Group::category(CategoryId const &id) const {
    return detail::optionalRecord<Category>(detail::unwrap(handle->category(id.value())));
}

inline std::future<std::optional<Category>> Group::categoryAsync(CategoryId const &id, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<Category>>>();
    auto future = promise->get_future();
    handle->category_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosCategoryOutcome> const outcome) {
        detail::complete(*promise, *outcome,
                         [](detail::EranosCategoryOutcome const &done) { return detail::optionalRecord<Category>(done.value()); });
    });
    return future;
}

inline CategoryId Group::addCategory(Category const &category) {
    return CategoryId(detail::unwrap(handle->add_category(*detail::toDetail(category))));
}

inline std::future<CategoryId> Group::addCategoryAsync(Category const &category, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<CategoryId>>();
    auto future = promise->get_future();
    handle->add_category_async(
        *detail::toDetail(category), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return CategoryId(*done.value()); });
        });
    return future;
}

inline Category Group::replaceCategory(Category const &category) {
    return detail::fromDetail(*detail::unwrap(handle->replace_category(*detail::toDetail(category))));
}

inline std::future<Category> Group::replaceCategoryAsync(Category const &category, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Category>>();
    auto future = promise->get_future();
    handle->replace_category_async(
        *detail::toDetail(category), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosCategoryOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosCategoryOutcome const &done) { return detail::fromDetail(*done.value()); });
        });
    return future;
}

inline Category Group::removeCategory(CategoryId const &id) { return detail::fromDetail(*detail::unwrap(handle->remove_category(id.value()))); }

inline std::future<Category> Group::removeCategoryAsync(CategoryId const &id, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Category>>();
    auto future = promise->get_future();
    handle->remove_category_async(
        id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosCategoryOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosCategoryOutcome const &done) { return detail::fromDetail(*done.value()); });
        });
    return future;
}

inline Currency Group::currency() const { return Currency::of(detail::unwrap(handle->currency())); }

inline std::future<Currency> Group::currencyAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<Currency>>();
    auto future = promise->get_future();
    handle->currency_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return Currency::of(*done.value()); });
    });
    return future;
}

inline void Group::changeCurrency(Currency const &currency, std::vector<CurrencyRate> const &rates) {
    detail::check(handle->change_currency(currency.code(), *detail::toDetail(rates)));
}

inline std::future<void> Group::changeCurrencyAsync(Currency const &currency, std::vector<CurrencyRate> const &rates,
                                                    Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<void>>();
    auto future = promise->get_future();
    handle->change_currency_async(currency.code(), *detail::toDetail(rates), detail::handleOf(cancellation),
                                  [promise](std::unique_ptr<detail::EranosUnitOutcome> const outcome) { detail::complete(*promise, *outcome); });
    return future;
}

inline std::vector<Expense> Group::expenses() const { return detail::fromDetail(*detail::unwrap(handle->expenses())); }

inline std::future<std::vector<Expense>> Group::expensesAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::vector<Expense>>>();
    auto future = promise->get_future();
    handle->expenses_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosExpenseListOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosExpenseListOutcome const &done) { return detail::fromDetail(*done.value()); });
    });
    return future;
}

inline std::optional<Expense> Group::expense(ExpenseId const &id) const {
    return detail::optionalRecord<Expense>(detail::unwrap(handle->expense(id.value())));
}

inline std::future<std::optional<Expense>> Group::expenseAsync(ExpenseId const &id, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<Expense>>>();
    auto future = promise->get_future();
    handle->expense_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosExpenseOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosExpenseOutcome const &done) { return detail::optionalRecord<Expense>(done.value()); });
    });
    return future;
}

inline std::optional<ExpenseShares> Group::expenseShares(ExpenseId const &id) const {
    return detail::optionalRecord<ExpenseShares>(detail::unwrap(handle->expense_shares(id.value())));
}

inline std::future<std::optional<ExpenseShares>> Group::expenseSharesAsync(ExpenseId const &id, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<ExpenseShares>>>();
    auto future = promise->get_future();
    handle->expense_shares_async(
        id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosExpenseSharesOutcome> const outcome) {
            detail::complete(*promise, *outcome,
                             [](detail::EranosExpenseSharesOutcome const &done) { return detail::optionalRecord<ExpenseShares>(done.value()); });
        });
    return future;
}

inline ExpenseId Group::addExpense(Expense const &expense) { return ExpenseId(detail::unwrap(handle->add_expense(*detail::toDetail(expense)))); }

inline std::future<ExpenseId> Group::addExpenseAsync(Expense const &expense, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<ExpenseId>>();
    auto future = promise->get_future();
    handle->add_expense_async(
        *detail::toDetail(expense), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return ExpenseId(*done.value()); });
        });
    return future;
}

inline Expense Group::replaceExpense(Expense const &expense) {
    return detail::fromDetail(*detail::unwrap(handle->replace_expense(*detail::toDetail(expense))));
}

inline std::future<Expense> Group::replaceExpenseAsync(Expense const &expense, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Expense>>();
    auto future = promise->get_future();
    handle->replace_expense_async(
        *detail::toDetail(expense), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosExpenseOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosExpenseOutcome const &done) { return detail::fromDetail(*done.value()); });
        });
    return future;
}

inline Expense Group::removeExpense(ExpenseId const &id) { return detail::fromDetail(*detail::unwrap(handle->remove_expense(id.value()))); }

inline std::future<Expense> Group::removeExpenseAsync(ExpenseId const &id, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Expense>>();
    auto future = promise->get_future();
    handle->remove_expense_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosExpenseOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosExpenseOutcome const &done) { return detail::fromDetail(*done.value()); });
    });
    return future;
}

inline Summary Group::summary() const { return detail::fromDetail(*detail::unwrap(handle->summary())); }

inline std::future<Summary> Group::summaryAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<Summary>>();
    auto future = promise->get_future();
    handle->summary_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosSummaryOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosSummaryOutcome const &done) { return detail::fromDetail(*done.value()); });
    });
    return future;
}

inline std::vector<Member> Group::members() const { return detail::fromDetail(*detail::unwrap(handle->members())); }

inline std::future<std::vector<Member>> Group::membersAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::vector<Member>>>();
    auto future = promise->get_future();
    handle->members_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosMemberListOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosMemberListOutcome const &done) { return detail::fromDetail(*done.value()); });
    });
    return future;
}

inline std::optional<Member> Group::member(MemberId const &id) const {
    return detail::optionalRecord<Member>(detail::unwrap(handle->member(id.value())));
}

inline std::future<std::optional<Member>> Group::memberAsync(MemberId const &id, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<Member>>>();
    auto future = promise->get_future();
    handle->member_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosMemberOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosMemberOutcome const &done) { return detail::optionalRecord<Member>(done.value()); });
    });
    return future;
}

inline bool Group::isArchived(MemberId const &id) const { return detail::unwrap(handle->is_archived(id.value())); }

inline std::future<bool> Group::isArchivedAsync(MemberId const &id, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<bool>>();
    auto future = promise->get_future();
    handle->is_archived_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosFlagOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosFlagOutcome const &done) { return done.value(); });
    });
    return future;
}

inline MemberId Group::addMember(Member const &member) { return MemberId(detail::unwrap(handle->add_member(*detail::toDetail(member)))); }

inline std::future<MemberId> Group::addMemberAsync(Member const &member, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<MemberId>>();
    auto future = promise->get_future();
    handle->add_member_async(*detail::toDetail(member), detail::handleOf(cancellation),
                             [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
                                 detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return MemberId(*done.value()); });
                             });
    return future;
}

inline Member Group::replaceMember(Member const &member) {
    return detail::fromDetail(*detail::unwrap(handle->replace_member(*detail::toDetail(member))));
}

inline std::future<Member> Group::replaceMemberAsync(Member const &member, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Member>>();
    auto future = promise->get_future();
    handle->replace_member_async(
        *detail::toDetail(member), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosMemberOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosMemberOutcome const &done) { return detail::fromDetail(*done.value()); });
        });
    return future;
}

inline void Group::archiveMember(MemberId const &id) { detail::check(handle->archive_member(id.value())); }

inline std::future<void> Group::archiveMemberAsync(MemberId const &id, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<void>>();
    auto future = promise->get_future();
    handle->archive_member_async(id.value(), detail::handleOf(cancellation),
                                 [promise](std::unique_ptr<detail::EranosUnitOutcome> const outcome) { detail::complete(*promise, *outcome); });
    return future;
}

inline void Group::unarchiveMember(MemberId const &id) { detail::check(handle->unarchive_member(id.value())); }

inline std::future<void> Group::unarchiveMemberAsync(MemberId const &id, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<void>>();
    auto future = promise->get_future();
    handle->unarchive_member_async(id.value(), detail::handleOf(cancellation),
                                   [promise](std::unique_ptr<detail::EranosUnitOutcome> const outcome) { detail::complete(*promise, *outcome); });
    return future;
}

inline Member Group::removeMember(MemberId const &id) { return detail::fromDetail(*detail::unwrap(handle->remove_member(id.value()))); }

inline std::future<Member> Group::removeMemberAsync(MemberId const &id, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Member>>();
    auto future = promise->get_future();
    handle->remove_member_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosMemberOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosMemberOutcome const &done) { return detail::fromDetail(*done.value()); });
    });
    return future;
}

inline std::string Group::name() const { return detail::unwrap(handle->name()); }

inline std::future<std::string> Group::nameAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::string>>();
    auto future = promise->get_future();
    handle->name_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return *done.value(); });
    });
    return future;
}

inline void Group::setName(std::string_view const name) { detail::check(handle->set_name(name)); }

inline std::future<void> Group::setNameAsync(std::string_view const name, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<void>>();
    auto future = promise->get_future();
    handle->set_name_async(name, detail::handleOf(cancellation),
                           [promise](std::unique_ptr<detail::EranosUnitOutcome> const outcome) { detail::complete(*promise, *outcome); });
    return future;
}

inline Settlement Group::settlement(SettlementStrategy const strategy) const {
    return detail::fromDetail(*detail::unwrap(handle->settlement(detail::toDetail(strategy))));
}

inline std::future<Settlement> Group::settlementAsync(SettlementStrategy const strategy, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<Settlement>>();
    auto future = promise->get_future();
    handle->settlement_async(
        detail::toDetail(strategy), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosSettlementOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosSettlementOutcome const &done) { return detail::fromDetail(*done.value()); });
        });
    return future;
}

inline std::vector<Transfer> Group::transfers() const { return detail::fromDetail(*detail::unwrap(handle->transfers())); }

inline std::future<std::vector<Transfer>> Group::transfersAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::vector<Transfer>>>();
    auto future = promise->get_future();
    handle->transfers_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTransferListOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosTransferListOutcome const &done) { return detail::fromDetail(*done.value()); });
    });
    return future;
}

inline std::optional<Transfer> Group::transfer(TransferId const &id) const {
    return detail::optionalRecord<Transfer>(detail::unwrap(handle->transfer(id.value())));
}

inline std::future<std::optional<Transfer>> Group::transferAsync(TransferId const &id, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<Transfer>>>();
    auto future = promise->get_future();
    handle->transfer_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTransferOutcome> const outcome) {
        detail::complete(*promise, *outcome,
                         [](detail::EranosTransferOutcome const &done) { return detail::optionalRecord<Transfer>(done.value()); });
    });
    return future;
}

inline std::optional<Money> Group::transferAmount(TransferId const &id) const {
    return detail::optionalRecord<Money>(detail::unwrap(handle->transfer_amount(id.value())));
}

inline std::future<std::optional<Money>> Group::transferAmountAsync(TransferId const &id, Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<Money>>>();
    auto future = promise->get_future();
    handle->transfer_amount_async(id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosMoneyOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosMoneyOutcome const &done) { return detail::optionalRecord<Money>(done.value()); });
    });
    return future;
}

inline TransferId Group::addTransfer(Transfer const &transfer) {
    return TransferId(detail::unwrap(handle->add_transfer(*detail::toDetail(transfer))));
}

inline std::future<TransferId> Group::addTransferAsync(Transfer const &transfer, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<TransferId>>();
    auto future = promise->get_future();
    handle->add_transfer_async(
        *detail::toDetail(transfer), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return TransferId(*done.value()); });
        });
    return future;
}

inline Transfer Group::replaceTransfer(Transfer const &transfer) {
    return detail::fromDetail(*detail::unwrap(handle->replace_transfer(*detail::toDetail(transfer))));
}

inline std::future<Transfer> Group::replaceTransferAsync(Transfer const &transfer, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Transfer>>();
    auto future = promise->get_future();
    handle->replace_transfer_async(
        *detail::toDetail(transfer), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTransferOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosTransferOutcome const &done) { return detail::fromDetail(*done.value()); });
        });
    return future;
}

inline Transfer Group::removeTransfer(TransferId const &id) { return detail::fromDetail(*detail::unwrap(handle->remove_transfer(id.value()))); }

inline std::future<Transfer> Group::removeTransferAsync(TransferId const &id, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<Transfer>>();
    auto future = promise->get_future();
    handle->remove_transfer_async(
        id.value(), detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTransferOutcome> const outcome) {
            detail::complete(*promise, *outcome, [](detail::EranosTransferOutcome const &done) { return detail::fromDetail(*done.value()); });
        });
    return future;
}

inline std::optional<std::string> Group::description() const { return detail::present(detail::unwrap(handle->description())); }

inline std::future<std::optional<std::string>> Group::descriptionAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<std::string>>>();
    auto future = promise->get_future();
    handle->description_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosOptionalTextOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosOptionalTextOutcome const &done) { return done.value(); });
    });
    return future;
}

inline void Group::setDescription(std::string_view const description) { detail::check(handle->set_description(description)); }

inline std::future<void> Group::setDescriptionAsync(std::string_view const description, Cancellation const *const cancellation) {
    auto const promise = std::make_shared<std::promise<void>>();
    auto future = promise->get_future();
    handle->set_description_async(description, detail::handleOf(cancellation),
                                  [promise](std::unique_ptr<detail::EranosUnitOutcome> const outcome) { detail::complete(*promise, *outcome); });
    return future;
}

inline std::optional<Language> Group::language() const {
    return detail::optionalText<Language>(detail::present(detail::unwrap(handle->language())), detail::languageOf);
}

inline std::future<std::optional<Language>> Group::languageAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::optional<Language>>>();
    auto future = promise->get_future();
    handle->language_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosOptionalTextOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosOptionalTextOutcome const &done) {
            return detail::optionalText<Language>(done.value(), detail::languageOf);
        });
    });
    return future;
}

inline std::string Group::save() const { return detail::unwrap(handle->save()); }

inline std::future<std::string> Group::saveAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<std::string>>();
    auto future = promise->get_future();
    handle->save_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosTextOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosTextOutcome const &done) { return *done.value(); });
    });
    return future;
}

inline GroupReader Group::read() const { return GroupReader(detail::unwrap(handle->read())); }

inline std::future<GroupReader> Group::readAsync(Cancellation const *const cancellation) const {
    auto const promise = std::make_shared<std::promise<GroupReader>>();
    auto future = promise->get_future();
    handle->read_async(detail::handleOf(cancellation), [promise](std::unique_ptr<detail::EranosGroupReaderOutcome> const outcome) {
        detail::complete(*promise, *outcome, [](detail::EranosGroupReaderOutcome &done) { return GroupReader(done.take()); });
    });
    return future;
}

} // namespace eranos

#endif // ERANOS_GROUP_HPP
