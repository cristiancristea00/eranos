"""Hold text that the library checked and wrote in its canonical form."""

from typing import override


class CheckedText:
    """
    Text that the library checked and wrote in its canonical form: the base of each kind of it.

    Two values compare equal when they are of one kind and the library wrote them alike; each prints as the library wrote
    it.
    """

    __slots__ = ('_text',)

    def __init__(self, text: str) -> None:
        """
        Hold text, which the library checked and wrote.

        Args:
            text: The text as the library wrote it.

        """
        self._text = text

    @override
    def __eq__(self, other: object) -> bool:
        """
        Tell whether other is of the same kind and written alike.

        Args:
            other: The object to compare.

        Returns:
            Whether the two are equal.

        """
        return type(other) is type(self) and str(other) == self._text

    @override
    def __hash__(self) -> int:
        """
        Hash the kind and the text.

        Returns:
            The hash.

        """
        return hash((type(self), self._text))

    @override
    def __repr__(self) -> str:
        """
        Write the call that makes the value.

        Returns:
            The text, such as ``"Email('ana@example.com')"``.

        """
        return f'{type(self).__name__}({self._text!r})'

    @override
    def __str__(self) -> str:
        """
        Write the text as the library did.

        Returns:
            The text.

        """
        return self._text
