"""Hold a write handle, which holds a group's exclusive lock for the reads and changes of one piece of work."""

from collections.abc import Iterable
from types import TracebackType
from typing import Self, final

from eranos._contact import Language
from eranos._crossing import optional
from eranos._currency_code import CurrencyCode, currency_of
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
from eranos._identifiers import CategoryId, ExpenseId, ExpenseGroupId, MemberId, TransferId
from eranos._internal import eranos as generated
from eranos._money import Money, money_of
from eranos._ratios import CurrencyRate, generated_rates
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


# One method per call of the library: the class is its surface.
# ruff: ignore[too-many-public-methods]
@final
class ExpenseGroupWriter:
    """
    A write handle: it holds the group's exclusive lock until it is released, so its reads and changes run with no other call in between.

    Every other call waits for it, so a blocking call of the group on the thread that holds a writer waits forever: release the writer
    first. It is a context manager, which releases the lock at the end of its block::

        with group.write() as writer:
            writer.add_member(member)
            report(writer.summary())
    """

    __slots__ = ('_handle',)

    def __init__(self, handle: generated.ExpenseGroupWriter) -> None:
        """
        Wrap handle, a write handle of the generated module: an app takes one with ExpenseGroup.write.

        Args:
            handle: The generated write handle.

        """
        self._handle = handle

    def __enter__(self) -> Self:
        """
        Enter the write handle's block.

        Returns:
            The write handle.

        """
        return self

    def __exit__(self, kind: type[BaseException] | None, error: BaseException | None, trace: TracebackType | None) -> None:
        """
        Release the lock at the end of the block.

        Args:
            kind: The class of the error that ends the block, or none.
            error: The error that ends the block, or none.
            trace: The error's traceback, or none.

        """
        self.release()

    def release(self) -> None:
        """Release the lock at once; a later call raises ``RuntimeError``."""
        self._handle.release()

    def id(self) -> ExpenseGroupId:
        """
        Read the group's identifier.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The identifier.

        """
        with library_errors():
            return ExpenseGroupId(self._handle.id())

    def categories(self) -> tuple[Category, ...]:
        """
        List every category: the built-in one first, then the others in identifier order, which is creation order for generated identifiers.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The categories.

        """
        with library_errors():
            return tuple(category_of(category) for category in self._handle.categories())

    def category(self, category_id: CategoryId) -> Category | None:
        """
        Find the category with the identifier category_id, if the group holds it.

        A call after the writer's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            category_id: The category's identifier.

        Returns:
            The category, or none.

        """
        with library_errors():
            return optional(self._handle.category(category_id), category_of)

    def currency(self) -> CurrencyCode:
        """
        Read the group currency, which every total is in.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The currency.

        """
        with library_errors():
            return currency_of(self._handle.currency())

    def expenses(self) -> tuple[Expense, ...]:
        """
        List every expense, in identifier order, which is creation order for generated identifiers.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The expenses.

        """
        with library_errors():
            return tuple(expense_of(expense) for expense in self._handle.expenses())

    def expense(self, expense_id: ExpenseId) -> Expense | None:
        """
        Find the expense with the identifier expense_id, if the group holds it.

        A call after the writer's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense, or none.

        """
        with library_errors():
            return optional(self._handle.expense(expense_id), expense_of)

    def expense_shares(self, expense_id: ExpenseId) -> ExpenseShares | None:
        """
        Work out how the expense with the identifier expense_id divides, in the group currency, if the group holds it.

        A call after the writer's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense's shares, or none.

        """
        with library_errors():
            return optional(self._handle.expense_shares(expense_id), expense_shares_of)

    def summary(self) -> Summary:
        """
        Add up the group's totals.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The totals.

        """
        with library_errors():
            return summary_of(self._handle.summary())

    def members(self) -> tuple[Member, ...]:
        """
        List every member, archived ones included, in identifier order, which is creation order for generated identifiers.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The members.

        """
        with library_errors():
            return tuple(member_of(member) for member in self._handle.members())

    def member(self, member_id: MemberId) -> Member | None:
        """
        Find the member with the identifier member_id, if the group holds them.

        A call after the writer's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            member_id: The member's identifier.

        Returns:
            The member, or none.

        """
        with library_errors():
            return optional(self._handle.member(member_id), member_of)

    def is_archived(self, member_id: MemberId) -> bool | None:
        """
        Tell whether the member with the identifier member_id is archived, if the group holds them.

        A call after the writer's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            member_id: The member's identifier.

        Returns:
            Whether the member is archived, or none for a member the group does not hold.

        """
        with library_errors():
            return self._handle.is_archived(member_id)

    def name(self) -> str:
        """
        Read the group's name.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The name.

        """
        with library_errors():
            return self._handle.name()

    def settlement(self, strategy: SettlementStrategy) -> SettlementPlan:
        """
        Plan the payments that settle every balance by strategy.

        A call after the writer's release raises ``RuntimeError``. The library refuses an exact plan of more open balances than its table fits with
        ``SettlementError.OutOfMemory``.

        Args:
            strategy: How the payments are planned.

        Returns:
            The settlement.

        """
        with library_errors():
            return settlement_of(self._handle.settlement(generated_strategy(strategy)))

    def transfers(self) -> tuple[Transfer, ...]:
        """
        List every transfer, in identifier order, which is creation order for generated identifiers.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The transfers.

        """
        with library_errors():
            return tuple(transfer_of(transfer) for transfer in self._handle.transfers())

    def transfer(self, transfer_id: TransferId) -> Transfer | None:
        """
        Find the transfer with the identifier transfer_id, if the group holds it.

        A call after the writer's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The transfer, or none.

        """
        with library_errors():
            return optional(self._handle.transfer(transfer_id), transfer_of)

    def transfer_amount(self, transfer_id: TransferId) -> Money | None:
        """
        Work out the amount in the group currency of the transfer with the identifier transfer_id, if the group holds it.

        A call after the writer's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The amount, or none.

        """
        with library_errors():
            return optional(self._handle.transfer_amount(transfer_id), money_of)

    def note(self) -> str | None:
        """
        Read the group's note, if it has one.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The note, or none.

        """
        with library_errors():
            return self._handle.note()

    def language(self) -> Language | None:
        """
        Read the language the apps show the group in, if it has one.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The language, or none.

        """
        with library_errors():
            return optional(self._handle.language(), Language)

    def save(self) -> str:
        """
        Save the group as JSON, which ExpenseGroup.load reads back.

        A call after the writer's release raises ``RuntimeError``.

        Returns:
            The saved group.

        """
        with library_errors():
            return self._handle.save()

    def add_category(self, category: Category) -> CategoryId:
        """
        Add category under its identifier and return the identifier.

        A call after the writer's release raises ``RuntimeError``. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateCategory``, a blank name with ``CategoryError.EmptyName``, and a
        name another category has, compared caselessly, with ``ExpenseGroupError.DuplicateCategoryName``.

        Args:
            category: The category.

        Returns:
            The category's identifier.

        """
        with library_errors():
            return CategoryId(self._handle.add_category(generated_category(category)))

    def replace_category(self, category: Category) -> Category:
        """
        Rename the category that category names by its identifier, and return it as it was.

        A call after the writer's release raises ``RuntimeError``. The library refuses a blank name with ``CategoryError.EmptyName``, and
        the category the group does not hold, the built-in one, or a name another category has with ``ExpenseGroupError.UnknownCategory``,
        ``ExpenseGroupError.BuiltInCategory``, and ``ExpenseGroupError.DuplicateCategoryName``.

        Args:
            category: The category.

        Returns:
            The category as it was.

        """
        with library_errors():
            return category_of(self._handle.replace_category(generated_category(category)))

    def remove_category(self, category_id: CategoryId) -> Category:
        """
        Remove the category with the identifier category_id and return it; its expenses move to the built-in one.

        A call after the writer's release raises ``RuntimeError``. The library refuses a category the group does not hold with
        ``ExpenseGroupError.UnknownCategory``, and the built-in one with ``ExpenseGroupError.BuiltInCategory``.

        Args:
            category_id: The category's identifier.

        Returns:
            The category.

        """
        with library_errors():
            return category_of(self._handle.remove_category(category_id))

    def change_currency(self, currency: CurrencyCode, rates: Iterable[CurrencyRate]) -> None:
        """
        Move the group to currency, converting every entry with the rate rates give its currency.

        A call after the writer's release raises ``RuntimeError``. The library refuses a currency in use without a rate with
        ``ExpenseGroupError.MissingExchangeRate``, a rate of the new group currency with ``ExpenseGroupError.UnexpectedExchangeRate``, and a currency
        given two rates with ``ExpenseGroupError.DuplicateExchangeRate``.

        Args:
            currency: The new group currency.
            rates: The rate into the new group currency of every other currency the entries use.

        """
        with library_errors():
            self._handle.change_currency(currency.code, generated_rates(rates))

    def add_expense(self, expense: Expense) -> ExpenseId:
        """
        Add expense under its identifier and return the identifier.

        A call after the writer's release raises ``RuntimeError``. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateExpense``, an expense that does not build, such as one whose
        amounts do not add up, with an ``ExpenseError``, and one that does not fit the group, such as one that names a member it does not hold or an
        archived one, with ``ExpenseGroupError.UnknownMember`` and ``ExpenseGroupError.ArchivedMember``.

        Args:
            expense: The expense.

        Returns:
            The expense's identifier.

        """
        with library_errors():
            return ExpenseId(self._handle.add_expense(generated_expense(expense)))

    def replace_expense(self, expense: Expense) -> Expense:
        """
        Replace the expense that expense names by its identifier, and return it as it was.

        A call after the writer's release raises ``RuntimeError``. The library refuses what adding one refuses, and an expense the group
        does not hold with ``ExpenseGroupError.UnknownExpense``.

        Args:
            expense: The expense.

        Returns:
            The expense as it was.

        """
        with library_errors():
            return expense_of(self._handle.replace_expense(generated_expense(expense)))

    def remove_expense(self, expense_id: ExpenseId) -> Expense:
        """
        Remove the expense with the identifier expense_id and return it.

        A call after the writer's release raises ``RuntimeError``. The library refuses an expense the group does not hold with
        ``ExpenseGroupError.UnknownExpense``.

        Args:
            expense_id: The expense's identifier.

        Returns:
            The expense.

        """
        with library_errors():
            return expense_of(self._handle.remove_expense(expense_id))

    def add_member(self, member: Member) -> MemberId:
        """
        Add member under their identifier and return the identifier.

        A call after the writer's release raises ``RuntimeError``. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateMember``, a blank first name with ``MemberError.EmptyFirstName``,
        and an account with a blank name with ``PaymentAccountError.EmptyName``.

        Args:
            member: The member.

        Returns:
            The member's identifier.

        """
        with library_errors():
            return MemberId(self._handle.add_member(generated_member(member)))

    def replace_member(self, member: Member) -> Member:
        """
        Replace the details of the member that member names by their identifier, and return them as they were.

        A call after the writer's release raises ``RuntimeError``. The library refuses what adding one refuses, and a member the group
        does not hold with ``ExpenseGroupError.UnknownMember``.

        Args:
            member: The member.

        Returns:
            The member as they were.

        """
        with library_errors():
            return member_of(self._handle.replace_member(generated_member(member)))

    def archive_member(self, member_id: MemberId) -> None:
        """
        Archive the member with the identifier member_id: they keep their history and balance, and join no new expense.

        A call after the writer's release raises ``RuntimeError``. The library refuses a member the group does not hold with
        ``ExpenseGroupError.UnknownMember``.

        Args:
            member_id: The member's identifier.

        """
        with library_errors():
            self._handle.archive_member(member_id)

    def unarchive_member(self, member_id: MemberId) -> None:
        """
        Make the archived member with the identifier member_id active again.

        A call after the writer's release raises ``RuntimeError``. The library refuses a member the group does not hold with
        ``ExpenseGroupError.UnknownMember``.

        Args:
            member_id: The member's identifier.

        """
        with library_errors():
            self._handle.unarchive_member(member_id)

    def remove_member(self, member_id: MemberId) -> Member:
        """
        Remove the member with the identifier member_id, who appears in no entry, and return them.

        A call after the writer's release raises ``RuntimeError``. The library refuses a member the group does not hold with
        ``ExpenseGroupError.UnknownMember``, and one entries still name with ``ExpenseGroupError.MemberInUse``, which names the entries; archive such
        a member instead.

        Args:
            member_id: The member's identifier.

        Returns:
            The member.

        """
        with library_errors():
            return member_of(self._handle.remove_member(member_id))

    def set_name(self, name: str) -> None:
        """
        Rename the group.

        A call after the writer's release raises ``RuntimeError``. The library refuses a blank name with ``ExpenseGroupError.EmptyName``.

        Args:
            name: The new name.

        """
        with library_errors():
            self._handle.set_name(name)

    def add_transfer(self, transfer: Transfer) -> TransferId:
        """
        Add transfer under its identifier and return the identifier.

        A call after the writer's release raises ``RuntimeError``. The library refuses text that is not an identifier with an
        ``IdError``, an identifier the group holds with ``ExpenseGroupError.DuplicateTransfer``, a transfer that does not build, such as one from a
        member to themselves, with a ``TransferError``, and one that names a member the group does not hold with ``ExpenseGroupError.UnknownMember``.

        Args:
            transfer: The transfer.

        Returns:
            The transfer's identifier.

        """
        with library_errors():
            return TransferId(self._handle.add_transfer(generated_transfer(transfer)))

    def replace_transfer(self, transfer: Transfer) -> Transfer:
        """
        Replace the transfer that transfer names by its identifier, and return it as it was.

        A call after the writer's release raises ``RuntimeError``. The library refuses what adding one refuses, and a transfer the group
        does not hold with ``ExpenseGroupError.UnknownTransfer``.

        Args:
            transfer: The transfer.

        Returns:
            The transfer as it was.

        """
        with library_errors():
            return transfer_of(self._handle.replace_transfer(generated_transfer(transfer)))

    def remove_transfer(self, transfer_id: TransferId) -> Transfer:
        """
        Remove the transfer with the identifier transfer_id and return it.

        A call after the writer's release raises ``RuntimeError``. The library refuses a transfer the group does not hold with
        ``ExpenseGroupError.UnknownTransfer``.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The transfer.

        """
        with library_errors():
            return transfer_of(self._handle.remove_transfer(transfer_id))

    def set_note(self, note: str | None) -> None:
        """
        Set the group's note, trimmed; none or blank text clears it.

        A call after the writer's release raises ``RuntimeError``.

        Args:
            note: The note, or none.

        """
        with library_errors():
            self._handle.set_note(note)

    def set_language(self, language: Language | None) -> None:
        """
        Set the language the apps show the group in; none clears it.

        A call after the writer's release raises ``RuntimeError``.

        Args:
            language: The language, or none.

        """
        with library_errors():
            self._handle.set_language(None if language is None else str(language))
