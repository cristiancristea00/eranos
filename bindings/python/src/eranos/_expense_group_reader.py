"""Hold a read handle, which holds a group's shared lock for the reads of one report."""

from types import TracebackType
from typing import Self, final

from eranos._contact import Language
from eranos._crossing import optional
from eranos._currency import Currency
from eranos._entries import Category, Expense, Member, Transfer, category_of, expense_of, member_of, transfer_of
from eranos._errors import library_errors
from eranos._identifiers import CategoryId, ExpenseId, ExpenseGroupId, MemberId, TransferId
from eranos._internal import eranos as generated
from eranos._money import Money, money_of
from eranos._totals import ExpenseShares, Settlement, SettlementStrategy, Summary, expense_shares_of, generated_strategy, settlement_of, summary_of


# One method per read of the library: the class is its surface.
# ruff: ignore[too-many-public-methods]
@final
class ExpenseGroupReader:
    """
    A read handle: it holds the group's shared lock until it is released, so the reads of one report see no change in between.

    A change waits for it. It is a context manager, which releases the lock at the end of its block::

        with group.read() as reader:
            report(reader.summary(), reader.members())
    """

    __slots__ = ('_handle',)

    def __init__(self, handle: generated.ExpenseGroupReader) -> None:
        """
        Wrap handle, a read handle of the generated module: an app takes one with ExpenseGroup.read.

        Args:
            handle: The generated read handle.

        """
        self._handle = handle

    def __enter__(self) -> Self:
        """
        Enter the read handle's block.

        Returns:
            The read handle.

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
        """Release the lock at once; a later read raises ``RuntimeError``."""
        self._handle.release()

    def id(self) -> ExpenseGroupId:
        """
        Read the group's identifier.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The identifier.

        """
        with library_errors():
            return ExpenseGroupId(self._handle.id())

    def categories(self) -> tuple[Category, ...]:
        """
        List every category, the built-in one first.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The categories.

        """
        with library_errors():
            return tuple(category_of(category) for category in self._handle.categories())

    def category(self, category_id: CategoryId) -> Category | None:
        """
        Find the category with the identifier category_id, if the group holds it.

        A read after the reader's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            category_id: The category's identifier.

        Returns:
            The category, or none.

        """
        with library_errors():
            return optional(self._handle.category(category_id), category_of)

    def currency(self) -> Currency:
        """
        Read the group currency, which every total is in.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The currency.

        """
        with library_errors():
            return Currency(self._handle.currency())

    def expenses(self) -> tuple[Expense, ...]:
        """
        List every expense, in the order they were created.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The expenses.

        """
        with library_errors():
            return tuple(expense_of(expense) for expense in self._handle.expenses())

    def expense(self, expense_id: ExpenseId) -> Expense | None:
        """
        Find the expense with the identifier expense_id, if the group holds it.

        A read after the reader's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

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

        A read after the reader's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

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

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The totals.

        """
        with library_errors():
            return summary_of(self._handle.summary())

    def members(self) -> tuple[Member, ...]:
        """
        List every member, archived ones included, in the order they were added.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The members.

        """
        with library_errors():
            return tuple(member_of(member) for member in self._handle.members())

    def member(self, member_id: MemberId) -> Member | None:
        """
        Find the member with the identifier member_id, if the group holds them.

        A read after the reader's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            member_id: The member's identifier.

        Returns:
            The member, or none.

        """
        with library_errors():
            return optional(self._handle.member(member_id), member_of)

    def is_archived(self, member_id: MemberId) -> bool:
        """
        Tell whether the member with the identifier member_id is archived: false for a member the group does not hold.

        A read after the reader's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            member_id: The member's identifier.

        Returns:
            Whether the member is archived.

        """
        with library_errors():
            return self._handle.is_archived(member_id)

    def name(self) -> str:
        """
        Read the group's name.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The name.

        """
        with library_errors():
            return self._handle.name()

    def settlement(self, strategy: SettlementStrategy) -> Settlement:
        """
        Plan the payments that settle every balance by strategy.

        A read after the reader's release raises ``RuntimeError``. The library refuses an exact plan of more open balances than its table fits with
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
        List every transfer, in the order they were created.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The transfers.

        """
        with library_errors():
            return tuple(transfer_of(transfer) for transfer in self._handle.transfers())

    def transfer(self, transfer_id: TransferId) -> Transfer | None:
        """
        Find the transfer with the identifier transfer_id, if the group holds it.

        A read after the reader's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

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

        A read after the reader's release raises ``RuntimeError``. The library refuses an identifier that is not one with an ``IdError``.

        Args:
            transfer_id: The transfer's identifier.

        Returns:
            The amount, or none.

        """
        with library_errors():
            return optional(self._handle.transfer_amount(transfer_id), money_of)

    def description(self) -> str | None:
        """
        Read the group's description, if it has one.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The description, or none.

        """
        with library_errors():
            return self._handle.description()

    def language(self) -> Language | None:
        """
        Read the language the apps show the group in, if it has one.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The language, or none.

        """
        with library_errors():
            return optional(self._handle.language(), Language)

    def save(self) -> str:
        """
        Save the group as JSON, which ExpenseGroup.load reads back.

        A read after the reader's release raises ``RuntimeError``.

        Returns:
            The saved group.

        """
        with library_errors():
            return self._handle.save()
