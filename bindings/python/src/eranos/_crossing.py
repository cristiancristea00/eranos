"""Cross values to and from the generated module where its types say less than its values hold."""

from collections.abc import Callable


def variant[Kind](kind: type[Kind], value: object) -> Kind:
    """
    Narrow value, a case of the generated data enum kind, to the enum.

    The generated module makes each case a subclass of its enum after it defines them, which a type checker cannot follow;
    at run time the check holds for every case.

    Args:
        kind: The generated data enum.
        value: A case of it.

    Returns:
        The case, as the enum.

    """
    assert isinstance(value, kind)
    return value


def optional[Value, Converted](value: Value | None, convert: Callable[[Value], Converted]) -> Converted | None:
    """
    Convert value with convert, if present.

    Args:
        value: The generated value, or none.
        convert: What converts it.

    Returns:
        The converted value, or none.

    """
    return None if value is None else convert(value)
