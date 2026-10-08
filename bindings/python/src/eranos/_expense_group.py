"""Hold a group: its details, members, categories, expenses, transfers, totals, and settlements."""

import dataclasses
from collections.abc import Iterable
from types import TracebackType
from typing import Self, final

from eranos._contact import Language
from eranos._crossing import optional
from eranos._currency_code import CurrencyCode
from eranos._entries import (
    Category,
    Expense,
    Member,
    Transfer,
    category_of,
    expense_of,
    generated_category,
    generated_expense,
    generated_member,
    generated_transfer,
    member_of,
    transfer_of,
)
from eranos._errors import library_errors
from eranos._expense_group_reader import ExpenseGroupReader
from eranos._identifiers import CategoryId, ExpenseId, ExpenseGroupId, MemberId, TransferId, generate_expense_group_id
from eranos._internal import eranos as generated
from eranos._money import Money, money_of
from eranos._ratios import ExchangeRate
from eranos._totals import (
    ExpenseShares,
    SettlementPlan,
    SettlementStrategy,
    Summary,
    expense_shares_of,
    generated_strategy,
    settlement_of,
    summary_of,
)


@final
@dataclasses.dataclass(frozen=True, slots=True)
class CurrencyRate:
    """The rate into a new group currency of an amount in currency."""

    currency: CurrencyCode
    """The currency the rate converts from."""

    rate: ExchangeRate
    """The rate."""


# One method per call of the library, twice over: the class is its surface.
# ruff: ignore[too-many-public-methods]
@final
class ExpenseGroup:
    """
    A group of people who share expenses.

    Every call comes in two forms. The one named for the call blocks the calling thread while another call holds the
    group's lock; its twin, whose name ends in ``_async``, is a coroutine that waits for the lock without blocking the
    event loop, and cancelling its task cancels the call, which leaves the group as it was::

        group = ExpenseGroup.create('Flat', CurrencyCode('EUR'))
        ana = group.add_member(Member.create(first_name='Ana'))
        bob = await group.add_member_async(Member.create(first_name='Bob'))

    Reads share the lock and run together; a change takes it alone, after the reads in progress. A group is a context
    manager: leaving its block frees it, and every call after that raises ``RuntimeError``.
    """

    __slots__ = ('_handle',)

    def __init__(self, handle: generated.ExpenseGroup) -> None:
        """
        Wrap handle, a group of the generated module: an app makes a group with create or load.

        Args:
            handle: The generated group.

        """
        self._handle: generated.ExpenseGroup | None = handle

    @classmethod
    def create(cls, name: str, currency: CurrencyCode, *, description: str | None = None, language: Language | None = None) -> Self:
        """
        Make a group with no members and no entries, called name, whose totals are in currency, under a generated identifier.

        ``generate_expense_group_id`` generates the identifier when this is called, and create_with_id takes one instead. The library
        refuses a blank name with ``ExpenseGroupError.EmptyName``, in an ``ExpenseGroupError``.

        Args:
            name: The name.
            currency: The group currency.
            description: A longer description, or none.
            language: The language the apps show the group in, or none.

        Returns:
            The group.

        """
        return cls.create_with_id(generate_expense_group_id(), name, currency, description=description, language=language)

    @classmethod
    def create_with_id(
        cls,
        # Named id, as every entry's identifier is, though the name shadows the builtin.
        id: ExpenseGroupId,  # ruff: ignore[builtin-argument-shadowing]
        name: str,
        currency: CurrencyCode,
        *,
        description: str | None = None,
        language: Language | None = None,
    ) -> Self:
        """
        Make a group with no members and no entries, called name, whose totals are in currency, under the identifier id.

        create generates the identifier instead. The library refuses text that is not an identifier with an ``IdError``, and a blank name
        with ``ExpenseGroupError.EmptyName``, in an ``ExpenseGroupError``.

        Args:
            id: The identifier the app gives the group, such as the key of its own store, or one ``generate_expense_group_id`` returns.
            name: The name.
            currency: The group currency.
            description: A longer description, or none.
            language: The language the apps show the group in, or none.

        Returns:
            The group.

        """
        details = generated.NewExpenseGroup(
            id=id, name=name, currency=currency.code, description=description, language=None if language is None else str(language)
        )
        with library_errors():
            return cls(generated.ExpenseGroup(details))

    @classmethod
    def load(cls, json: str) -> Self:
        """
        Load the group saved as json, through the checks that adding its entries makes.

        The library refuses text that is not the saved form with an ``InvalidJsonError``, a format this release cannot read
        with ``ExpenseGroupError.UnsupportedFormat``, and entries that do not fit together with their failures.

        Args:
            json: The saved group, which save wrote.

        Returns:
            The group.

        """
        with library_errors():
            return cls(generated.ExpenseGroup.load(json))

    @classmethod
    async def load_async(cls, json: str) -> Self:
        """
        Load the group saved as json, through the checks that adding its entries makes.

        The load runs on a thread of its own and yields while it replays the entries; cancelling the call's task cancels
        it. It refuses what load refuses.

        Args:
            json: The saved group, which save wrote.

        Returns:
            The group.

        """
        with library_errors():
            # The generated load names its result by the object's protocol; the object is always the class.
            loaded = await generated.load_expense_group_async(json, None)
        return cls(loaded)

    def __enter__(self) -> Self:
        """
        Enter the group's block.

        Returns:
            The group.

        """
        return self

    def __exit__(self, kind: type[BaseException] | None, error: BaseException | None, trace: TracebackType | None) -> None:
        """
        Free the group at the end of its block; every call after it raises ``RuntimeError``.

        Args:
            kind: The class of the error that ends the block, or none.
            error: The error that ends the block, or none.
            trace: The error's traceback, or none.

        """
        self._handle = None

    def _generated(self) -> generated.ExpenseGroup:
        """
        Read the generated group, which a group that was freed no longer holds.

        Returns:
            The generated group.

        Raises:
            RuntimeError: When the group was freed.

        """
        if self._handle is None:
            message = 'the group was freed'
            raise RuntimeError(message)
        return self._handle

    def read(self) -> ExpenseGroupReader:
        """
        Take a read handle, which holds the shared lock until it is released.

        It blocks the calling thread while a change holds the group's lock. A read handle is a context manager, which releases
        the lock at the end of its block.

        Returns:
            The read handle.

        """
        with library_errors():
            return ExpenseGroupReader(self._generated().read())

    async def read_async(self) -> ExpenseGroupReader:
        """
        Take a read handle, which holds the shared lock until it is released.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the wait.

        Returns:
            The read handle.

        """
        with library_errors():
            return ExpenseGroupReader(await self._generated().read_async(None))

    def id(self) -> ExpenseGroupId:
        """
        Read the group's identifier.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The identifier.

        """
        with library_errors():
            return ExpenseGroupId(self._generated().id())

    async def id_async(self) -> ExpenseGroupId:
        """
        Read the group's identifier.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what id refuses.

        Returns:
            The identifier.

        """
        with library_errors():
            return ExpenseGroupId(await self._generated().id_async(None))

    def categories(self) -> tuple[Category, ...]:
        """
        List every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The categories.

        """
        with library_errors():
            return tuple(category_of(category) for category in self._generated().categories())

    async def categories_async(self) -> tuple[Category, ...]:
        """
        List every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what categories refuses.

        Returns:
            The categories.

        """
        with library_errors():
            return tuple(category_of(category) for category in await self._generated().categories_async(None))

    def category(self, category_id: CategoryId) -> Category | None:
        """
        Find the category with the identifier category_id, if the group holds it.

        It blocks the calling thread while another call holds the group's lock. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            category_id: The category's identifier.

        Returns:
            The category, or none.

        """
        with library_errors():
            return optional(self._generated().category(category_id), category_of)

    async def category_async(self, category_id: CategoryId) -> Category | None:
        """
        Find the category with the identifier category_id, if the group holds it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what category refuses.

        Args:
            category_id: The category's identifier.

        Returns:
            The category, or none.

        """
        with library_errors():
            return optional(await self._generated().category_async(category_id, None), category_of)

    def add_category(self, category: Category) -> CategoryId:
        """
        Add category under its identifier and return the identifier.

        It blocks the calling thread while another call holds the group's lock. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateCategory``, a blank name with ``CategoryError.EmptyName``, and a
        name another category has, compared caselessly, with ``ExpenseGroupError.DuplicateCategoryName``.

        Args:
            category: The category.

        Returns:
            The category's identifier.

        """
        with library_errors():
            return CategoryId(self._generated().add_category(generated_category(category)))

    async def add_category_async(self, category: Category) -> CategoryId:
        """
        Add category under its identifier and return the identifier.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what add_category refuses.

        Args:
            category: The category.

        Returns:
            The category's identifier.

        """
        with library_errors():
            return CategoryId(await self._generated().add_category_async(generated_category(category), None))

    def replace_category(self, category: Category) -> Category:
        """
        Rename the category that category names by its identifier, and return it as it was.

        It blocks the calling thread while another call holds the group's lock. The library refuses a blank name with ``CategoryError.EmptyName``, and
        the category the group does not hold, the built-in one, or a name another category has with ``ExpenseGroupError.UnknownCategory``,
        ``ExpenseGroupError.BuiltInCategory``, and ``ExpenseGroupError.DuplicateCategoryName``.

        Args:
            category: The category.

        Returns:
            The category as it was.

        """
        with library_errors():
            return category_of(self._generated().replace_category(generated_category(category)))

    async def replace_category_async(self, category: Category) -> Category:
        """
        Rename the category that category names by its identifier, and return it as it was.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what replace_category refuses.

        Args:
            category: The category.

        Returns:
            The category as it was.

        """
        with library_errors():
            return category_of(await self._generated().replace_category_async(generated_category(category), None))

    def remove_category(self, category_id: CategoryId) -> Category:
        """
        Remove the category with the identifier category_id and return it; its expenses move to the built-in one.

        It blocks the calling thread while another call holds the group's lock. The library refuses a category the group does not hold with
        ``ExpenseGroupError.UnknownCategory``, and the built-in one with ``ExpenseGroupError.BuiltInCategory``.

        Args:
            category_id: The category's identifier.

        Returns:
            The category.

        """
        with library_errors():
            return category_of(self._generated().remove_category(category_id))

    async def remove_category_async(self, category_id: CategoryId) -> Category:
        """
        Remove the category with the identifier category_id and return it; its expenses move to the built-in one.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what remove_category refuses.

        Args:
            category_id: The category's identifier.

        Returns:
            The category.

        """
        with library_errors():
            return category_of(await self._generated().remove_category_async(category_id, None))

    def currency(self) -> CurrencyCode:
        """
        Read the group currency, which every total is in.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The currency.

        """
        with library_errors():
            return CurrencyCode(self._generated().currency())

    async def currency_async(self) -> CurrencyCode:
        """
        Read the group currency, which every total is in.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what currency refuses.

        Returns:
            The currency.

        """
        with library_errors():
            return CurrencyCode(await self._generated().currency_async(None))

    def change_currency(self, currency: CurrencyCode, rates: Iterable[CurrencyRate]) -> None:
        """
        Move the group to currency, converting every entry with the rate rates give its currency.

        It blocks the calling thread while another call holds the group's lock. The library refuses a currency in use without a rate with
        ``ExpenseGroupError.MissingExchangeRate``, a rate of the new group currency with ``ExpenseGroupError.UnexpectedExchangeRate``, and a currency
        given two rates with ``ExpenseGroupError.DuplicateExchangeRate``.

        Args:
            currency: The new group currency.
            rates: The rate into the new group currency of every other currency the entries use.

        """
        with library_errors():
            self._generated().change_currency(currency.code, _generated_rates(rates))

    async def change_currency_async(self, currency: CurrencyCode, rates: Iterable[CurrencyRate]) -> None:
        """
        Move the group to currency, converting every entry with the rate rates give its currency.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what change_currency refuses.

        Args:
            currency: The new group currency.
            rates: The rate into the new group currency of every other currency the entries use.

        """
        with library_errors():
            await self._generated().change_currency_async(currency.code, _generated_rates(rates), None)

    def expenses(self) -> tuple[Expense, ...]:
        """
        List every expense, in identifier order, which is creation order for generated identifiers.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The expenses.

        """
        with library_errors():
            return tuple(expense_of(expense) for expense in self._generated().expenses())

    async def expenses_async(self) -> tuple[Expense, ...]:
        """
        List every expense, in identifier order, which is creation order for generated identifiers.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what expenses refuses.

        Returns:
            The expenses.

        """
        with library_errors():
            return tuple(expense_of(expense) for expense in await self._generated().expenses_async(None))

    def expense(self, expense_id: ExpenseId) -> Expense | None:
        """
        Find the expense with the identifier expense_id, if the group holds it.

        It blocks the calling thread while another call holds the group's lock. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense, or none.

        """
        with library_errors():
            return optional(self._generated().expense(expense_id), expense_of)

    async def expense_async(self, expense_id: ExpenseId) -> Expense | None:
        """
        Find the expense with the identifier expense_id, if the group holds it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what expense refuses.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense, or none.

        """
        with library_errors():
            return optional(await self._generated().expense_async(expense_id, None), expense_of)

    def expense_shares(self, expense_id: ExpenseId) -> ExpenseShares | None:
        """
        Work out how the expense with the identifier expense_id divides, in the group currency, if the group holds it.

        It blocks the calling thread while another call holds the group's lock. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense's shares, or none.

        """
        with library_errors():
            return optional(self._generated().expense_shares(expense_id), expense_shares_of)

    async def expense_shares_async(self, expense_id: ExpenseId) -> ExpenseShares | None:
        """
        Work out how the expense with the identifier expense_id divides, in the group currency, if the group holds it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what expense_shares refuses.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense's shares, or none.

        """
        with library_errors():
            return optional(await self._generated().expense_shares_async(expense_id, None), expense_shares_of)

    def add_expense(self, expense: Expense) -> ExpenseId:
        """
        Add expense under its identifier and return the identifier.

        It blocks the calling thread while another call holds the group's lock. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateExpense``, an expense that does not build, such as one whose
        amounts do not add up, with an ``ExpenseError``, and one that does not fit the group, such as one that names a member it does not hold or an
        archived one, with ``ExpenseGroupError.UnknownMember`` and ``ExpenseGroupError.ArchivedMember``.

        Args:
            expense: The expense.

        Returns:
            The expense's identifier.

        """
        with library_errors():
            return ExpenseId(self._generated().add_expense(generated_expense(expense)))

    async def add_expense_async(self, expense: Expense) -> ExpenseId:
        """
        Add expense under its identifier and return the identifier.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what add_expense refuses.

        Args:
            expense: The expense.

        Returns:
            The expense's identifier.

        """
        with library_errors():
            return ExpenseId(await self._generated().add_expense_async(generated_expense(expense), None))

    def replace_expense(self, expense: Expense) -> Expense:
        """
        Replace the expense that expense names by its identifier, and return it as it was.

        It blocks the calling thread while another call holds the group's lock. The library refuses what adding one refuses, and an expense the group
        does not hold with ``ExpenseGroupError.UnknownExpense``.

        Args:
            expense: The expense.

        Returns:
            The expense as it was.

        """
        with library_errors():
            return expense_of(self._generated().replace_expense(generated_expense(expense)))

    async def replace_expense_async(self, expense: Expense) -> Expense:
        """
        Replace the expense that expense names by its identifier, and return it as it was.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what replace_expense refuses.

        Args:
            expense: The expense.

        Returns:
            The expense as it was.

        """
        with library_errors():
            return expense_of(await self._generated().replace_expense_async(generated_expense(expense), None))

    def remove_expense(self, expense_id: ExpenseId) -> Expense:
        """
        Remove the expense with the identifier expense_id and return it.

        It blocks the calling thread while another call holds the group's lock. The library refuses an expense the group does not hold with
        ``ExpenseGroupError.UnknownExpense``.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense.

        """
        with library_errors():
            return expense_of(self._generated().remove_expense(expense_id))

    async def remove_expense_async(self, expense_id: ExpenseId) -> Expense:
        """
        Remove the expense with the identifier expense_id and return it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what remove_expense refuses.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense.

        """
        with library_errors():
            return expense_of(await self._generated().remove_expense_async(expense_id, None))

    def summary(self) -> Summary:
        """
        Add up the group's totals.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The totals.

        """
        with library_errors():
            return summary_of(self._generated().summary())

    async def summary_async(self) -> Summary:
        """
        Add up the group's totals.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what summary refuses.

        Returns:
            The totals.

        """
        with library_errors():
            return summary_of(await self._generated().summary_async(None))

    def members(self) -> tuple[Member, ...]:
        """
        List every member, archived ones included, in identifier order, which is creation order for generated identifiers.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The members.

        """
        with library_errors():
            return tuple(member_of(member) for member in self._generated().members())

    async def members_async(self) -> tuple[Member, ...]:
        """
        List every member, archived ones included, in identifier order, which is creation order for generated identifiers.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what members refuses.

        Returns:
            The members.

        """
        with library_errors():
            return tuple(member_of(member) for member in await self._generated().members_async(None))

    def member(self, member_id: MemberId) -> Member | None:
        """
        Find the member with the identifier member_id, if the group holds them.

        It blocks the calling thread while another call holds the group's lock. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            member_id: The member's identifier.

        Returns:
            The member, or none.

        """
        with library_errors():
            return optional(self._generated().member(member_id), member_of)

    async def member_async(self, member_id: MemberId) -> Member | None:
        """
        Find the member with the identifier member_id, if the group holds them.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what member refuses.

        Args:
            member_id: The member's identifier.

        Returns:
            The member, or none.

        """
        with library_errors():
            return optional(await self._generated().member_async(member_id, None), member_of)

    def is_archived(self, member_id: MemberId) -> bool | None:
        """
        Tell whether the member with the identifier member_id is archived, if the group holds them.

        It blocks the calling thread while another call holds the group's lock. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            member_id: The member's identifier.

        Returns:
            Whether the member is archived, or none for a member the group does not hold.

        """
        with library_errors():
            return self._generated().is_archived(member_id)

    async def is_archived_async(self, member_id: MemberId) -> bool | None:
        """
        Tell whether the member with the identifier member_id is archived, if the group holds them.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what is_archived refuses.

        Args:
            member_id: The member's identifier.

        Returns:
            Whether the member is archived, or none for a member the group does not hold.

        """
        with library_errors():
            return await self._generated().is_archived_async(member_id, None)

    def add_member(self, member: Member) -> MemberId:
        """
        Add member under their identifier and return the identifier.

        It blocks the calling thread while another call holds the group's lock. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateMember``, a blank first name with ``MemberError.EmptyFirstName``,
        and an account with a blank name with ``PaymentAccountError.EmptyName``.

        Args:
            member: The member.

        Returns:
            The member's identifier.

        """
        with library_errors():
            return MemberId(self._generated().add_member(generated_member(member)))

    async def add_member_async(self, member: Member) -> MemberId:
        """
        Add member under their identifier and return the identifier.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what add_member refuses.

        Args:
            member: The member.

        Returns:
            The member's identifier.

        """
        with library_errors():
            return MemberId(await self._generated().add_member_async(generated_member(member), None))

    def replace_member(self, member: Member) -> Member:
        """
        Replace the details of the member that member names by their identifier, and return them as they were.

        It blocks the calling thread while another call holds the group's lock. The library refuses what adding one refuses, and a member the group
        does not hold with ``ExpenseGroupError.UnknownMember``.

        Args:
            member: The member.

        Returns:
            The member as they were.

        """
        with library_errors():
            return member_of(self._generated().replace_member(generated_member(member)))

    async def replace_member_async(self, member: Member) -> Member:
        """
        Replace the details of the member that member names by their identifier, and return them as they were.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what replace_member refuses.

        Args:
            member: The member.

        Returns:
            The member as they were.

        """
        with library_errors():
            return member_of(await self._generated().replace_member_async(generated_member(member), None))

    def archive_member(self, member_id: MemberId) -> None:
        """
        Archive the member with the identifier member_id: they keep their history and balance, and join no new expense.

        It blocks the calling thread while another call holds the group's lock. The library refuses a member the group does not hold with
        ``ExpenseGroupError.UnknownMember``.

        Args:
            member_id: The member's identifier.

        """
        with library_errors():
            self._generated().archive_member(member_id)

    async def archive_member_async(self, member_id: MemberId) -> None:
        """
        Archive the member with the identifier member_id: they keep their history and balance, and join no new expense.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what archive_member refuses.

        Args:
            member_id: The member's identifier.

        """
        with library_errors():
            await self._generated().archive_member_async(member_id, None)

    def unarchive_member(self, member_id: MemberId) -> None:
        """
        Make the archived member with the identifier member_id active again.

        It blocks the calling thread while another call holds the group's lock. The library refuses a member the group does not hold with
        ``ExpenseGroupError.UnknownMember``.

        Args:
            member_id: The member's identifier.

        """
        with library_errors():
            self._generated().unarchive_member(member_id)

    async def unarchive_member_async(self, member_id: MemberId) -> None:
        """
        Make the archived member with the identifier member_id active again.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what unarchive_member refuses.

        Args:
            member_id: The member's identifier.

        """
        with library_errors():
            await self._generated().unarchive_member_async(member_id, None)

    def remove_member(self, member_id: MemberId) -> Member:
        """
        Remove the member with the identifier member_id, who appears in no entry, and return them.

        It blocks the calling thread while another call holds the group's lock. The library refuses a member the group does not hold with
        ``ExpenseGroupError.UnknownMember``, and one entries still name with ``ExpenseGroupError.MemberInUse``, which names the entries; archive such
        a member instead.

        Args:
            member_id: The member's identifier.

        Returns:
            The member.

        """
        with library_errors():
            return member_of(self._generated().remove_member(member_id))

    async def remove_member_async(self, member_id: MemberId) -> Member:
        """
        Remove the member with the identifier member_id, who appears in no entry, and return them.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what remove_member refuses.

        Args:
            member_id: The member's identifier.

        Returns:
            The member.

        """
        with library_errors():
            return member_of(await self._generated().remove_member_async(member_id, None))

    def name(self) -> str:
        """
        Read the group's name.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The name.

        """
        with library_errors():
            return self._generated().name()

    async def name_async(self) -> str:
        """
        Read the group's name.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what name refuses.

        Returns:
            The name.

        """
        with library_errors():
            return await self._generated().name_async(None)

    def set_name(self, name: str) -> None:
        """
        Rename the group.

        It blocks the calling thread while another call holds the group's lock. The library refuses a blank name with ``ExpenseGroupError.EmptyName``.

        Args:
            name: The new name.

        """
        with library_errors():
            self._generated().set_name(name)

    async def set_name_async(self, name: str) -> None:
        """
        Rename the group.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what set_name refuses.

        Args:
            name: The new name.

        """
        with library_errors():
            await self._generated().set_name_async(name, None)

    def settlement(self, strategy: SettlementStrategy) -> SettlementPlan:
        """
        Plan the payments that settle every balance by strategy.

        It blocks the calling thread while another call holds the group's lock. The library refuses an exact plan of more open balances than its table
        fits with ``SettlementError.OutOfMemory``.

        Args:
            strategy: How the payments are planned.

        Returns:
            The settlement.

        """
        with library_errors():
            return settlement_of(self._generated().settlement(generated_strategy(strategy)))

    async def settlement_async(self, strategy: SettlementStrategy) -> SettlementPlan:
        """
        Plan the payments that settle every balance by strategy.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what settlement refuses.

        Args:
            strategy: How the payments are planned.

        Returns:
            The settlement.

        """
        with library_errors():
            return settlement_of(await self._generated().settlement_async(generated_strategy(strategy), None))

    def transfers(self) -> tuple[Transfer, ...]:
        """
        List every transfer, in identifier order, which is creation order for generated identifiers.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The transfers.

        """
        with library_errors():
            return tuple(transfer_of(transfer) for transfer in self._generated().transfers())

    async def transfers_async(self) -> tuple[Transfer, ...]:
        """
        List every transfer, in identifier order, which is creation order for generated identifiers.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what transfers refuses.

        Returns:
            The transfers.

        """
        with library_errors():
            return tuple(transfer_of(transfer) for transfer in await self._generated().transfers_async(None))

    def transfer(self, transfer_id: TransferId) -> Transfer | None:
        """
        Find the transfer with the identifier transfer_id, if the group holds it.

        It blocks the calling thread while another call holds the group's lock. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The transfer, or none.

        """
        with library_errors():
            return optional(self._generated().transfer(transfer_id), transfer_of)

    async def transfer_async(self, transfer_id: TransferId) -> Transfer | None:
        """
        Find the transfer with the identifier transfer_id, if the group holds it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what transfer refuses.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The transfer, or none.

        """
        with library_errors():
            return optional(await self._generated().transfer_async(transfer_id, None), transfer_of)

    def transfer_amount(self, transfer_id: TransferId) -> Money | None:
        """
        Work out the amount in the group currency of the transfer with the identifier transfer_id, if the group holds it.

        It blocks the calling thread while another call holds the group's lock. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The amount, or none.

        """
        with library_errors():
            return optional(self._generated().transfer_amount(transfer_id), money_of)

    async def transfer_amount_async(self, transfer_id: TransferId) -> Money | None:
        """
        Work out the amount in the group currency of the transfer with the identifier transfer_id, if the group holds it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what transfer_amount refuses.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The amount, or none.

        """
        with library_errors():
            return optional(await self._generated().transfer_amount_async(transfer_id, None), money_of)

    def add_transfer(self, transfer: Transfer) -> TransferId:
        """
        Add transfer under its identifier and return the identifier.

        It blocks the calling thread while another call holds the group's lock. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateTransfer``, a transfer that does not build, such as one from a
        member to themselves, with a ``TransferError``, and one that names a member the group does not hold with ``ExpenseGroupError.UnknownMember``.

        Args:
            transfer: The transfer.

        Returns:
            The transfer's identifier.

        """
        with library_errors():
            return TransferId(self._generated().add_transfer(generated_transfer(transfer)))

    async def add_transfer_async(self, transfer: Transfer) -> TransferId:
        """
        Add transfer under its identifier and return the identifier.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what add_transfer refuses.

        Args:
            transfer: The transfer.

        Returns:
            The transfer's identifier.

        """
        with library_errors():
            return TransferId(await self._generated().add_transfer_async(generated_transfer(transfer), None))

    def replace_transfer(self, transfer: Transfer) -> Transfer:
        """
        Replace the transfer that transfer names by its identifier, and return it as it was.

        It blocks the calling thread while another call holds the group's lock. The library refuses what adding one refuses, and a transfer the group
        does not hold with ``ExpenseGroupError.UnknownTransfer``.

        Args:
            transfer: The transfer.

        Returns:
            The transfer as it was.

        """
        with library_errors():
            return transfer_of(self._generated().replace_transfer(generated_transfer(transfer)))

    async def replace_transfer_async(self, transfer: Transfer) -> Transfer:
        """
        Replace the transfer that transfer names by its identifier, and return it as it was.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what replace_transfer refuses.

        Args:
            transfer: The transfer.

        Returns:
            The transfer as it was.

        """
        with library_errors():
            return transfer_of(await self._generated().replace_transfer_async(generated_transfer(transfer), None))

    def remove_transfer(self, transfer_id: TransferId) -> Transfer:
        """
        Remove the transfer with the identifier transfer_id and return it.

        It blocks the calling thread while another call holds the group's lock. The library refuses a transfer the group does not hold with
        ``ExpenseGroupError.UnknownTransfer``.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The transfer.

        """
        with library_errors():
            return transfer_of(self._generated().remove_transfer(transfer_id))

    async def remove_transfer_async(self, transfer_id: TransferId) -> Transfer:
        """
        Remove the transfer with the identifier transfer_id and return it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what remove_transfer refuses.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The transfer.

        """
        with library_errors():
            return transfer_of(await self._generated().remove_transfer_async(transfer_id, None))

    def description(self) -> str | None:
        """
        Read the group's description, if it has one.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The description, or none.

        """
        with library_errors():
            return self._generated().description()

    async def description_async(self) -> str | None:
        """
        Read the group's description, if it has one.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what description refuses.

        Returns:
            The description, or none.

        """
        with library_errors():
            return await self._generated().description_async(None)

    def set_description(self, description: str | None) -> None:
        """
        Set the group's description, trimmed; none or blank text clears it.

        It blocks the calling thread while another call holds the group's lock.

        Args:
            description: The description, or none.

        """
        with library_errors():
            self._generated().set_description(description)

    async def set_description_async(self, description: str | None) -> None:
        """
        Set the group's description, trimmed; none or blank text clears it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what set_description refuses.

        Args:
            description: The description, or none.

        """
        with library_errors():
            await self._generated().set_description_async(description, None)

    def language(self) -> Language | None:
        """
        Read the language the apps show the group in, if it has one.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The language, or none.

        """
        with library_errors():
            return optional(self._generated().language(), Language)

    async def language_async(self) -> Language | None:
        """
        Read the language the apps show the group in, if it has one.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what language refuses.

        Returns:
            The language, or none.

        """
        with library_errors():
            return optional(await self._generated().language_async(None), Language)

    def set_language(self, language: Language | None) -> None:
        """
        Set the language the apps show the group in; none clears it.

        It blocks the calling thread while another call holds the group's lock.

        Args:
            language: The language, or none.

        """
        with library_errors():
            self._generated().set_language(None if language is None else str(language))

    async def set_language_async(self, language: Language | None) -> None:
        """
        Set the language the apps show the group in; none clears it.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what set_language refuses.

        Args:
            language: The language, or none.

        """
        with library_errors():
            await self._generated().set_language_async(None if language is None else str(language), None)

    def save(self) -> str:
        """
        Save the group as JSON, which ExpenseGroup.load reads back.

        It blocks the calling thread while another call holds the group's lock.

        Returns:
            The saved group.

        """
        with library_errors():
            return self._generated().save()

    async def save_async(self) -> str:
        """
        Save the group as JSON, which ExpenseGroup.load reads back.

        It waits for the group's lock without blocking the event loop; cancelling the call's task cancels the call, which leaves the group as it was.
        It refuses what save refuses.

        Returns:
            The saved group.

        """
        with library_errors():
            return await self._generated().save_async(None)


def _generated_rates(rates: Iterable[CurrencyRate]) -> list[generated.CurrencyRate]:
    """
    Convert rates into the rates the generated module holds.

    Args:
        rates: The rates.

    Returns:
        The generated rates.

    """
    return [generated.CurrencyRate(currency=rate.currency.code, rate=str(rate.rate)) for rate in rates]
