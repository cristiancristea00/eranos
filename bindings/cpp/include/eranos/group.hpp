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
#include "eranos/error.hpp"
#include "eranos/expense.hpp"
#include "eranos/group_reader.hpp"
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
    [[nodiscard]] friend bool operator==(CurrencyRate const &left, CurrencyRate const &right) noexcept = default;
};

/// The details a new group can start with besides its name and currency.
struct GroupOptions final {
    /// A longer description.
    std::optional<std::string> description;
    /// The language the apps show the group in.
    std::optional<Language> language;
};

/// A cancellation of asynchronous calls, which one cancellation can be given to several of: cancelling it stops each at its
/// next yield, or at once while it waits for the lock, and its future then holds an error of the kind ErrorKind::CANCELLED.
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
    /// @param name The group's name
    /// @param currency The group currency, which every total is in
    /// @param options The details it starts with besides
    /// @return The group, or an error of the kind ErrorKind::GROUP for a blank name, ErrorKind::LANGUAGE for a language that is not a tag, and
    ///   ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] static std::expected<Group, Error> create(std::string_view const name, Currency const &currency, GroupOptions const &options = {});

    /// The group saved as @p json, which save() wrote.
    ///
    /// @param json The saved group
    /// @return The group, or an error of the kind ErrorKind::INVALID_JSON for text that is not a saved group
    [[nodiscard]] static std::expected<Group, Error> load(std::string_view const json);

    /// The asynchronous form of load(), which runs on a thread of its own.
    ///
    /// @param json The saved group
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the group, or of the error the call ends with
    [[nodiscard]] static std::future<std::expected<Group, Error>> loadAsync(std::string_view const json,
                                                                            Cancellation const *const cancellation = nullptr);

    /// The group's identifier. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] GroupId id() const;

    /// The asynchronous form of id(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<GroupId, Error>> idAsync(Cancellation const *const cancellation = nullptr) const;

    /// Every category, the built-in one first. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Category> categories() const;

    /// The asynchronous form of categories(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Category>, Error>> categoriesAsync(Cancellation const *const cancellation = nullptr) const;

    /// The category with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Category>, Error> category(CategoryId const &id) const;

    /// The asynchronous form of category(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Category>, Error>> categoryAsync(CategoryId const &id,
                                                                                           Cancellation const *const cancellation = nullptr) const;

    /// Adds category, which has no identifier yet, and returns the one it gets. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param category The category
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<CategoryId, Error> addCategory(Category const &category);

    /// The asynchronous form of addCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param category The category
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<CategoryId, Error>> addCategoryAsync(Category const &category,
                                                                                 Cancellation const *const cancellation = nullptr);

    /// Renames the category that category names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param category The category
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Category, Error> replaceCategory(Category const &category);

    /// The asynchronous form of replaceCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param category The category
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Category, Error>> replaceCategoryAsync(Category const &category,
                                                                                   Cancellation const *const cancellation = nullptr);

    /// Removes the category with the identifier id and returns it; its expenses move to the built-in one. Blocks the calling thread while another
    /// call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Category, Error> removeCategory(CategoryId const &id);

    /// The asynchronous form of removeCategory(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Category, Error>> removeCategoryAsync(CategoryId const &id,
                                                                                  Cancellation const *const cancellation = nullptr);

    /// The ISO 4217 code of the group currency. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] Currency currency() const;

    /// The asynchronous form of currency(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Currency, Error>> currencyAsync(Cancellation const *const cancellation = nullptr) const;

    /// Moves the group to currency, with a rate into it for every other currency its entries use. Blocks the calling thread while another call holds
    /// the group's lock.
    ///
    /// @param currency The currency
    /// @param rates The rates
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> changeCurrency(Currency const &currency, std::span<CurrencyRate const> const rates);

    /// The asynchronous form of changeCurrency(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param currency The currency
    /// @param rates The rates
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> changeCurrencyAsync(Currency const &currency, std::span<CurrencyRate const> const rates,
                                                                              Cancellation const *const cancellation = nullptr);

    /// Every expense, in the order they were created. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Expense> expenses() const;

    /// The asynchronous form of expenses(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Expense>, Error>> expensesAsync(Cancellation const *const cancellation = nullptr) const;

    /// The expense with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Expense>, Error> expense(ExpenseId const &id) const;

    /// The asynchronous form of expense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Expense>, Error>> expenseAsync(ExpenseId const &id,
                                                                                         Cancellation const *const cancellation = nullptr) const;

    /// How the expense with the identifier id divides, in the group currency. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<ExpenseShares>, Error> expenseShares(ExpenseId const &id) const;

    /// The asynchronous form of expenseShares(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<ExpenseShares>, Error>>
    expenseSharesAsync(ExpenseId const &id, Cancellation const *const cancellation = nullptr) const;

    /// Adds expense, which has no identifier yet, and returns the one it gets. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param expense The expense
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<ExpenseId, Error> addExpense(Expense const &expense);

    /// The asynchronous form of addExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param expense The expense
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<ExpenseId, Error>> addExpenseAsync(Expense const &expense,
                                                                               Cancellation const *const cancellation = nullptr);

    /// Replaces the expense that expense names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param expense The expense
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Expense, Error> replaceExpense(Expense const &expense);

    /// The asynchronous form of replaceExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param expense The expense
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Expense, Error>> replaceExpenseAsync(Expense const &expense,
                                                                                 Cancellation const *const cancellation = nullptr);

    /// Removes the expense with the identifier id and returns it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Expense, Error> removeExpense(ExpenseId const &id);

    /// The asynchronous form of removeExpense(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Expense, Error>> removeExpenseAsync(ExpenseId const &id,
                                                                                Cancellation const *const cancellation = nullptr);

    /// The group's totals. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] Summary summary() const;

    /// The asynchronous form of summary(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Summary, Error>> summaryAsync(Cancellation const *const cancellation = nullptr) const;

    /// Every member, archived ones included, in the order they were created. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Member> members() const;

    /// The asynchronous form of members(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Member>, Error>> membersAsync(Cancellation const *const cancellation = nullptr) const;

    /// The member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Member>, Error> member(MemberId const &id) const;

    /// The asynchronous form of member(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Member>, Error>> memberAsync(MemberId const &id,
                                                                                       Cancellation const *const cancellation = nullptr) const;

    /// Whether the member with the identifier id is archived; false for an unknown member. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<bool, Error> isArchived(MemberId const &id) const;

    /// The asynchronous form of isArchived(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<bool, Error>> isArchivedAsync(MemberId const &id, Cancellation const *const cancellation = nullptr) const;

    /// Adds member, who has no identifier yet, and returns the one they get. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param member The member
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<MemberId, Error> addMember(Member const &member);

    /// The asynchronous form of addMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param member The member
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<MemberId, Error>> addMemberAsync(Member const &member, Cancellation const *const cancellation = nullptr);

    /// Replaces the member that member names by their identifier, and returns them as they were. Blocks the calling thread while another call holds
    /// the group's lock.
    ///
    /// @param member The member
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Member, Error> replaceMember(Member const &member);

    /// The asynchronous form of replaceMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param member The member
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Member, Error>> replaceMemberAsync(Member const &member,
                                                                               Cancellation const *const cancellation = nullptr);

    /// Archives the member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> archiveMember(MemberId const &id);

    /// The asynchronous form of archiveMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> archiveMemberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr);

    /// Unarchives the member with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> unarchiveMember(MemberId const &id);

    /// The asynchronous form of unarchiveMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> unarchiveMemberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr);

    /// Removes the member with the identifier id, whom no entry references, and returns them. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Member, Error> removeMember(MemberId const &id);

    /// The asynchronous form of removeMember(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Member, Error>> removeMemberAsync(MemberId const &id, Cancellation const *const cancellation = nullptr);

    /// The group's name. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::string name() const;

    /// The asynchronous form of name(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::string, Error>> nameAsync(Cancellation const *const cancellation = nullptr) const;

    /// Renames the group. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param name The name
    /// @return Nothing, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<void, Error> setName(std::string_view const name);

    /// The asynchronous form of setName(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param name The name
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> setNameAsync(std::string_view const name, Cancellation const *const cancellation = nullptr);

    /// The payments that settle every balance by strategy. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param strategy The strategy
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Settlement, Error> settlement(SettlementStrategy const strategy) const;

    /// The asynchronous form of settlement(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param strategy The strategy
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Settlement, Error>> settlementAsync(SettlementStrategy const strategy,
                                                                                Cancellation const *const cancellation = nullptr) const;

    /// Every transfer, in the order they were created. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::vector<Transfer> transfers() const;

    /// The asynchronous form of transfers(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::vector<Transfer>, Error>> transfersAsync(Cancellation const *const cancellation = nullptr) const;

    /// The transfer with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Transfer>, Error> transfer(TransferId const &id) const;

    /// The asynchronous form of transfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Transfer>, Error>> transferAsync(TransferId const &id,
                                                                                           Cancellation const *const cancellation = nullptr) const;

    /// The amount in the group currency of the transfer with the identifier id. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<std::optional<Money>, Error> transferAmount(TransferId const &id) const;

    /// The asynchronous form of transferAmount(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Money>, Error>> transferAmountAsync(TransferId const &id,
                                                                                              Cancellation const *const cancellation = nullptr) const;

    /// Adds transfer, which has no identifier yet, and returns the one it gets. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param transfer The transfer
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<TransferId, Error> addTransfer(Transfer const &transfer);

    /// The asynchronous form of addTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param transfer The transfer
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<TransferId, Error>> addTransferAsync(Transfer const &transfer,
                                                                                 Cancellation const *const cancellation = nullptr);

    /// Replaces the transfer that transfer names by its identifier, and returns it as it was. Blocks the calling thread while another call holds the
    /// group's lock.
    ///
    /// @param transfer The transfer
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Transfer, Error> replaceTransfer(Transfer const &transfer);

    /// The asynchronous form of replaceTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param transfer The transfer
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Transfer, Error>> replaceTransferAsync(Transfer const &transfer,
                                                                                   Cancellation const *const cancellation = nullptr);

    /// Removes the transfer with the identifier id and returns it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param id The id
    /// @return The result, or the error the library refuses the call with; its kind names the module that refused it
    [[nodiscard]] std::expected<Transfer, Error> removeTransfer(TransferId const &id);

    /// The asynchronous form of removeTransfer(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param id The id
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<Transfer, Error>> removeTransferAsync(TransferId const &id,
                                                                                  Cancellation const *const cancellation = nullptr);

    /// The group's description. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::optional<std::string> description() const;

    /// The asynchronous form of description(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<std::string>, Error>>
    descriptionAsync(Cancellation const *const cancellation = nullptr) const;

    /// Sets the description; blank text clears it. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param description The description
    /// @return Nothing, or an error of the kind ErrorKind::INVALID_TEXT for text that is not UTF-8
    [[nodiscard]] std::expected<void, Error> setDescription(std::string_view const description);

    /// The asynchronous form of setDescription(): it runs on the group's worker thread, in the order of the calls, without blocking the calling
    /// thread; cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an
    /// error of the kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param description The description
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> setDescriptionAsync(std::string_view const description,
                                                                              Cancellation const *const cancellation = nullptr);

    /// The language the apps show the group in, as a BCP 47 tag. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::optional<Language> language() const;

    /// The asynchronous form of language(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::optional<Language>, Error>> languageAsync(Cancellation const *const cancellation = nullptr) const;

    /// The group in its saved form, as JSON. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] std::string save() const;

    /// The asynchronous form of save(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<std::string, Error>> saveAsync(Cancellation const *const cancellation = nullptr) const;

    /// A read handle, which holds the shared lock until it is released or dropped, so that its reads see no change in between. Blocks the calling
    /// thread while another call holds the group's lock.
    ///
    /// @return The result
    [[nodiscard]] GroupReader read() const;

    /// The asynchronous form of read(): it runs on the group's worker thread, in the order of the calls, without blocking the calling thread;
    /// cancelling @p cancellation stops the call at its next yield, or at once while it waits for the lock, and its future then holds an error of the
    /// kind ErrorKind::CANCELLED, the group left as it was.
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the result, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<GroupReader, Error>> readAsync(Cancellation const *const cancellation = nullptr) const;

    /// Clears the group's description. Blocks the calling thread while another call holds the group's lock.
    void clearDescription();

    /// The asynchronous form of clearDescription().
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> clearDescriptionAsync(Cancellation const *const cancellation = nullptr);

    /// Sets the language the apps show the group in. Blocks the calling thread while another call holds the group's lock.
    ///
    /// @param language The language
    void setLanguage(Language const &language);

    /// The asynchronous form of setLanguage().
    ///
    /// @param language The language
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> setLanguageAsync(Language const &language,
                                                                           Cancellation const *const cancellation = nullptr);

    /// Clears the language the apps show the group in. Blocks the calling thread while another call holds the group's lock.
    void clearLanguage();

    /// The asynchronous form of clearLanguage().
    ///
    /// @param cancellation A cancellation for the call, or none
    /// @return A future of the end of the call, or of the error the call ends with
    [[nodiscard]] std::future<std::expected<void, Error>> clearLanguageAsync(Cancellation const *const cancellation = nullptr);

  private:
    explicit Group(std::unique_ptr<detail::EranosGroup> handle) noexcept : handle(std::move(handle)) {}

    std::unique_ptr<detail::EranosGroup> handle;
};

namespace detail {

[[nodiscard]] inline std::unique_ptr<EranosCurrencyRate> toDetail(CurrencyRate const &value);
[[nodiscard]] inline std::unique_ptr<EranosCurrencyRateList> toDetail(std::span<CurrencyRate const> const records);

} // namespace detail

inline std::expected<Group, Error> Group::create(std::string_view const name, Currency const &currency, GroupOptions const &options) {
    return detail::expectedOf(detail::EranosNewGroup::create(name, currency.code()))
        .and_then([&options](std::unique_ptr<detail::EranosNewGroup> details) {
            return detail::setIfPresent(std::move(details), options.description,
                                        [](detail::EranosNewGroup &record, std::string const &text) { return record.set_description(text); });
        })
        .and_then([&options](std::unique_ptr<detail::EranosNewGroup> details) {
            return detail::setIfPresent(std::move(details), options.language, [](detail::EranosNewGroup &record, Language const &language) {
                return record.set_language(language.toString());
            });
        })
        .and_then([](std::unique_ptr<detail::EranosNewGroup> const &details) { return detail::expectedOf(detail::EranosGroup::create(*details)); })
        .transform([](std::unique_ptr<detail::EranosGroup> handle) { return Group(std::move(handle)); });
}

inline std::expected<Group, Error> Group::load(std::string_view const json) {
    return detail::expectedOf(detail::EranosGroup::load(json)).transform([](std::unique_ptr<detail::EranosGroup> handle) {
        return Group(std::move(handle));
    });
}

inline std::future<std::expected<Group, Error>> Group::loadAsync(std::string_view const json, Cancellation const *const cancellation) {
    return detail::launch<Group>([&](auto callback) { detail::EranosGroup::load_async(json, detail::handleOf(cancellation), std::move(callback)); },
                                 [](detail::EranosGroupOutcome &done) { return Group(done.take()); });
}

inline void Group::clearDescription() { static_cast<void>(detail::waited(handle->set_description(""))); }

inline std::future<std::expected<void, Error>> Group::clearDescriptionAsync(Cancellation const *const cancellation) {
    return setDescriptionAsync("", cancellation);
}

inline void Group::setLanguage(Language const &language) { static_cast<void>(detail::waited(handle->set_language(language.toString()))); }

inline std::future<std::expected<void, Error>> Group::setLanguageAsync(Language const &language, Cancellation const *const cancellation) {
    return detail::launch<void>(
        [&](auto callback) { handle->set_language_async(language.toString(), detail::handleOf(cancellation), std::move(callback)); },
        detail::finished);
}

inline void Group::clearLanguage() { static_cast<void>(detail::waited(handle->clear_language())); }

inline std::future<std::expected<void, Error>> Group::clearLanguageAsync(Cancellation const *const cancellation) {
    return detail::launch<void>([&](auto callback) { handle->clear_language_async(detail::handleOf(cancellation), std::move(callback)); },
                                detail::finished);
}

inline GroupId Group::id() const { return GroupId(detail::waited(handle->id())); }

inline std::future<std::expected<GroupId, Error>> Group::idAsync(Cancellation const *const cancellation) const {
    return detail::launch<GroupId>([&](auto callback) { handle->id_async(detail::handleOf(cancellation), std::move(callback)); },
                                   [](detail::EranosTextOutcome const &done) { return GroupId(*done.value()); });
}

inline std::vector<Category> Group::categories() const { return detail::fromDetail(*detail::waited(handle->categories())); }

inline std::future<std::expected<std::vector<Category>, Error>> Group::categoriesAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::vector<Category>>(
        [&](auto callback) { handle->categories_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosCategoryListOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Category>, Error> Group::category(CategoryId const &id) const {
    return detail::expectedOf(handle->category(id.value())).transform([](auto const &value) { return detail::optionalRecord<Category>(value); });
}

inline std::future<std::expected<std::optional<Category>, Error>> Group::categoryAsync(CategoryId const &id,
                                                                                       Cancellation const *const cancellation) const {
    return detail::launch<std::optional<Category>>(
        [&](auto callback) { handle->category_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosCategoryOutcome const &done) { return detail::optionalRecord<Category>(done.value()); });
}

inline std::expected<CategoryId, Error> Group::addCategory(Category const &category) {
    return detail::toDetail(category)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->add_category(*entry)); })
        .transform([](std::string id) { return CategoryId(std::move(id)); });
}

inline std::future<std::expected<CategoryId, Error>> Group::addCategoryAsync(Category const &category, Cancellation const *const cancellation) {
    return detail::launch<CategoryId>(
        detail::toDetail(category),
        [&](auto const &entry, auto callback) { handle->add_category_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) { return CategoryId(*done.value()); });
}

inline std::expected<Category, Error> Group::replaceCategory(Category const &category) {
    return detail::toDetail(category)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->replace_category(*entry)); })
        .transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Category, Error>> Group::replaceCategoryAsync(Category const &category, Cancellation const *const cancellation) {
    return detail::launch<Category>(
        detail::toDetail(category),
        [&](auto const &entry, auto callback) { handle->replace_category_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosCategoryOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<Category, Error> Group::removeCategory(CategoryId const &id) {
    return detail::expectedOf(handle->remove_category(id.value())).transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Category, Error>> Group::removeCategoryAsync(CategoryId const &id, Cancellation const *const cancellation) {
    return detail::launch<Category>(
        [&](auto callback) { handle->remove_category_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosCategoryOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline Currency Group::currency() const { return detail::listedCurrency(detail::waited(handle->currency())); }

inline std::future<std::expected<Currency, Error>> Group::currencyAsync(Cancellation const *const cancellation) const {
    return detail::launch<Currency>([&](auto callback) { handle->currency_async(detail::handleOf(cancellation), std::move(callback)); },
                                    [](detail::EranosTextOutcome const &done) { return detail::listedCurrency(*done.value()); });
}

inline std::expected<void, Error> Group::changeCurrency(Currency const &currency, std::span<CurrencyRate const> const rates) {
    return detail::expectedOf(handle->change_currency(currency.code(), *detail::toDetail(rates)));
}

inline std::future<std::expected<void, Error>> Group::changeCurrencyAsync(Currency const &currency, std::span<CurrencyRate const> const rates,
                                                                          Cancellation const *const cancellation) {
    return detail::launch<void>(
        [&](auto callback) {
            handle->change_currency_async(currency.code(), *detail::toDetail(rates), detail::handleOf(cancellation), std::move(callback));
        },
        detail::finished);
}

inline std::vector<Expense> Group::expenses() const { return detail::fromDetail(*detail::waited(handle->expenses())); }

inline std::future<std::expected<std::vector<Expense>, Error>> Group::expensesAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::vector<Expense>>([&](auto callback) { handle->expenses_async(detail::handleOf(cancellation), std::move(callback)); },
                                                [](detail::EranosExpenseListOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Expense>, Error> Group::expense(ExpenseId const &id) const {
    return detail::expectedOf(handle->expense(id.value())).transform([](auto const &value) { return detail::optionalRecord<Expense>(value); });
}

inline std::future<std::expected<std::optional<Expense>, Error>> Group::expenseAsync(ExpenseId const &id,
                                                                                     Cancellation const *const cancellation) const {
    return detail::launch<std::optional<Expense>>(
        [&](auto callback) { handle->expense_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseOutcome const &done) { return detail::optionalRecord<Expense>(done.value()); });
}

inline std::expected<std::optional<ExpenseShares>, Error> Group::expenseShares(ExpenseId const &id) const {
    return detail::expectedOf(handle->expense_shares(id.value())).transform([](auto const &value) {
        return detail::optionalRecord<ExpenseShares>(value);
    });
}

inline std::future<std::expected<std::optional<ExpenseShares>, Error>> Group::expenseSharesAsync(ExpenseId const &id,
                                                                                                 Cancellation const *const cancellation) const {
    return detail::launch<std::optional<ExpenseShares>>(
        [&](auto callback) { handle->expense_shares_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseSharesOutcome const &done) { return detail::optionalRecord<ExpenseShares>(done.value()); });
}

inline std::expected<ExpenseId, Error> Group::addExpense(Expense const &expense) {
    return detail::toDetail(expense)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->add_expense(*entry)); })
        .transform([](std::string id) { return ExpenseId(std::move(id)); });
}

inline std::future<std::expected<ExpenseId, Error>> Group::addExpenseAsync(Expense const &expense, Cancellation const *const cancellation) {
    return detail::launch<ExpenseId>(
        detail::toDetail(expense),
        [&](auto const &entry, auto callback) { handle->add_expense_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) { return ExpenseId(*done.value()); });
}

inline std::expected<Expense, Error> Group::replaceExpense(Expense const &expense) {
    return detail::toDetail(expense)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->replace_expense(*entry)); })
        .transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Expense, Error>> Group::replaceExpenseAsync(Expense const &expense, Cancellation const *const cancellation) {
    return detail::launch<Expense>(
        detail::toDetail(expense),
        [&](auto const &entry, auto callback) { handle->replace_expense_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<Expense, Error> Group::removeExpense(ExpenseId const &id) {
    return detail::expectedOf(handle->remove_expense(id.value())).transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Expense, Error>> Group::removeExpenseAsync(ExpenseId const &id, Cancellation const *const cancellation) {
    return detail::launch<Expense>(
        [&](auto callback) { handle->remove_expense_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosExpenseOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline Summary Group::summary() const { return detail::fromDetail(*detail::waited(handle->summary())); }

inline std::future<std::expected<Summary, Error>> Group::summaryAsync(Cancellation const *const cancellation) const {
    return detail::launch<Summary>([&](auto callback) { handle->summary_async(detail::handleOf(cancellation), std::move(callback)); },
                                   [](detail::EranosSummaryOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::vector<Member> Group::members() const { return detail::fromDetail(*detail::waited(handle->members())); }

inline std::future<std::expected<std::vector<Member>, Error>> Group::membersAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::vector<Member>>([&](auto callback) { handle->members_async(detail::handleOf(cancellation), std::move(callback)); },
                                               [](detail::EranosMemberListOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Member>, Error> Group::member(MemberId const &id) const {
    return detail::expectedOf(handle->member(id.value())).transform([](auto const &value) { return detail::optionalRecord<Member>(value); });
}

inline std::future<std::expected<std::optional<Member>, Error>> Group::memberAsync(MemberId const &id, Cancellation const *const cancellation) const {
    return detail::launch<std::optional<Member>>(
        [&](auto callback) { handle->member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosMemberOutcome const &done) { return detail::optionalRecord<Member>(done.value()); });
}

inline std::expected<bool, Error> Group::isArchived(MemberId const &id) const { return detail::expectedOf(handle->is_archived(id.value())); }

inline std::future<std::expected<bool, Error>> Group::isArchivedAsync(MemberId const &id, Cancellation const *const cancellation) const {
    return detail::launch<bool>([&](auto callback) { handle->is_archived_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
                                [](detail::EranosFlagOutcome const &done) { return done.value(); });
}

inline std::expected<MemberId, Error> Group::addMember(Member const &member) {
    return detail::toDetail(member)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->add_member(*entry)); })
        .transform([](std::string id) { return MemberId(std::move(id)); });
}

inline std::future<std::expected<MemberId, Error>> Group::addMemberAsync(Member const &member, Cancellation const *const cancellation) {
    return detail::launch<MemberId>(
        detail::toDetail(member),
        [&](auto const &entry, auto callback) { handle->add_member_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) { return MemberId(*done.value()); });
}

inline std::expected<Member, Error> Group::replaceMember(Member const &member) {
    return detail::toDetail(member)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->replace_member(*entry)); })
        .transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Member, Error>> Group::replaceMemberAsync(Member const &member, Cancellation const *const cancellation) {
    return detail::launch<Member>(
        detail::toDetail(member),
        [&](auto const &entry, auto callback) { handle->replace_member_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosMemberOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<void, Error> Group::archiveMember(MemberId const &id) { return detail::expectedOf(handle->archive_member(id.value())); }

inline std::future<std::expected<void, Error>> Group::archiveMemberAsync(MemberId const &id, Cancellation const *const cancellation) {
    return detail::launch<void>([&](auto callback) { handle->archive_member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
                                detail::finished);
}

inline std::expected<void, Error> Group::unarchiveMember(MemberId const &id) { return detail::expectedOf(handle->unarchive_member(id.value())); }

inline std::future<std::expected<void, Error>> Group::unarchiveMemberAsync(MemberId const &id, Cancellation const *const cancellation) {
    return detail::launch<void>(
        [&](auto callback) { handle->unarchive_member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); }, detail::finished);
}

inline std::expected<Member, Error> Group::removeMember(MemberId const &id) {
    return detail::expectedOf(handle->remove_member(id.value())).transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Member, Error>> Group::removeMemberAsync(MemberId const &id, Cancellation const *const cancellation) {
    return detail::launch<Member>(
        [&](auto callback) { handle->remove_member_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosMemberOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::string Group::name() const { return detail::waited(handle->name()); }

inline std::future<std::expected<std::string, Error>> Group::nameAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::string>([&](auto callback) { handle->name_async(detail::handleOf(cancellation), std::move(callback)); },
                                       [](detail::EranosTextOutcome const &done) { return *done.value(); });
}

inline std::expected<void, Error> Group::setName(std::string_view const name) { return detail::expectedOf(handle->set_name(name)); }

inline std::future<std::expected<void, Error>> Group::setNameAsync(std::string_view const name, Cancellation const *const cancellation) {
    return detail::launch<void>([&](auto callback) { handle->set_name_async(name, detail::handleOf(cancellation), std::move(callback)); },
                                detail::finished);
}

inline std::expected<Settlement, Error> Group::settlement(SettlementStrategy const strategy) const {
    return detail::expectedOf(handle->settlement(detail::toDetail(strategy))).transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Settlement, Error>> Group::settlementAsync(SettlementStrategy const strategy,
                                                                            Cancellation const *const cancellation) const {
    return detail::launch<Settlement>(
        [&](auto callback) { handle->settlement_async(detail::toDetail(strategy), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosSettlementOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::vector<Transfer> Group::transfers() const { return detail::fromDetail(*detail::waited(handle->transfers())); }

inline std::future<std::expected<std::vector<Transfer>, Error>> Group::transfersAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::vector<Transfer>>([&](auto callback) { handle->transfers_async(detail::handleOf(cancellation), std::move(callback)); },
                                                 [](detail::EranosTransferListOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<std::optional<Transfer>, Error> Group::transfer(TransferId const &id) const {
    return detail::expectedOf(handle->transfer(id.value())).transform([](auto const &value) { return detail::optionalRecord<Transfer>(value); });
}

inline std::future<std::expected<std::optional<Transfer>, Error>> Group::transferAsync(TransferId const &id,
                                                                                       Cancellation const *const cancellation) const {
    return detail::launch<std::optional<Transfer>>(
        [&](auto callback) { handle->transfer_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTransferOutcome const &done) { return detail::optionalRecord<Transfer>(done.value()); });
}

inline std::expected<std::optional<Money>, Error> Group::transferAmount(TransferId const &id) const {
    return detail::expectedOf(handle->transfer_amount(id.value())).transform([](auto const &value) { return detail::optionalRecord<Money>(value); });
}

inline std::future<std::expected<std::optional<Money>, Error>> Group::transferAmountAsync(TransferId const &id,
                                                                                          Cancellation const *const cancellation) const {
    return detail::launch<std::optional<Money>>(
        [&](auto callback) { handle->transfer_amount_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosMoneyOutcome const &done) { return detail::optionalRecord<Money>(done.value()); });
}

inline std::expected<TransferId, Error> Group::addTransfer(Transfer const &transfer) {
    return detail::toDetail(transfer)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->add_transfer(*entry)); })
        .transform([](std::string id) { return TransferId(std::move(id)); });
}

inline std::future<std::expected<TransferId, Error>> Group::addTransferAsync(Transfer const &transfer, Cancellation const *const cancellation) {
    return detail::launch<TransferId>(
        detail::toDetail(transfer),
        [&](auto const &entry, auto callback) { handle->add_transfer_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTextOutcome const &done) { return TransferId(*done.value()); });
}

inline std::expected<Transfer, Error> Group::replaceTransfer(Transfer const &transfer) {
    return detail::toDetail(transfer)
        .and_then([this](auto const &entry) { return detail::expectedOf(handle->replace_transfer(*entry)); })
        .transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Transfer, Error>> Group::replaceTransferAsync(Transfer const &transfer, Cancellation const *const cancellation) {
    return detail::launch<Transfer>(
        detail::toDetail(transfer),
        [&](auto const &entry, auto callback) { handle->replace_transfer_async(*entry, detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTransferOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::expected<Transfer, Error> Group::removeTransfer(TransferId const &id) {
    return detail::expectedOf(handle->remove_transfer(id.value())).transform([](auto const &value) { return detail::fromDetail(*value); });
}

inline std::future<std::expected<Transfer, Error>> Group::removeTransferAsync(TransferId const &id, Cancellation const *const cancellation) {
    return detail::launch<Transfer>(
        [&](auto callback) { handle->remove_transfer_async(id.value(), detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosTransferOutcome const &done) { return detail::fromDetail(*done.value()); });
}

inline std::optional<std::string> Group::description() const { return detail::present(detail::waited(handle->description())); }

inline std::future<std::expected<std::optional<std::string>, Error>> Group::descriptionAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::optional<std::string>>(
        [&](auto callback) { handle->description_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosOptionalTextOutcome const &done) { return done.value(); });
}

inline std::expected<void, Error> Group::setDescription(std::string_view const description) {
    return detail::expectedOf(handle->set_description(description));
}

inline std::future<std::expected<void, Error>> Group::setDescriptionAsync(std::string_view const description,
                                                                          Cancellation const *const cancellation) {
    return detail::launch<void>(
        [&](auto callback) { handle->set_description_async(description, detail::handleOf(cancellation), std::move(callback)); }, detail::finished);
}

inline std::optional<Language> Group::language() const {
    return detail::optionalText<Language>(detail::present(detail::waited(handle->language())), detail::languageOf);
}

inline std::future<std::expected<std::optional<Language>, Error>> Group::languageAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::optional<Language>>(
        [&](auto callback) { handle->language_async(detail::handleOf(cancellation), std::move(callback)); },
        [](detail::EranosOptionalTextOutcome const &done) { return detail::optionalText<Language>(done.value(), detail::languageOf); });
}

inline std::string Group::save() const { return detail::waited(handle->save()); }

inline std::future<std::expected<std::string, Error>> Group::saveAsync(Cancellation const *const cancellation) const {
    return detail::launch<std::string>([&](auto callback) { handle->save_async(detail::handleOf(cancellation), std::move(callback)); },
                                       [](detail::EranosTextOutcome const &done) { return *done.value(); });
}

inline GroupReader Group::read() const { return GroupReader(detail::waited(handle->read())); }

inline std::future<std::expected<GroupReader, Error>> Group::readAsync(Cancellation const *const cancellation) const {
    return detail::launch<GroupReader>([&](auto callback) { handle->read_async(detail::handleOf(cancellation), std::move(callback)); },
                                       [](detail::EranosGroupReaderOutcome &done) { return GroupReader(done.take()); });
}

} // namespace eranos

#endif // ERANOS_GROUP_HPP
