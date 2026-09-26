"""Build the package's documentation with Sphinx, from the docstrings of its interface."""

import pathlib
import sys
import typing

from docutils import nodes
from sphinx.addnodes import pending_xref
from sphinx.application import Sphinx
from sphinx.environment import BuildEnvironment
from sphinx.ext.autodoc import Options
from sphinx.pycode import ModuleAnalyzer


sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / 'src'))

project = 'eranos'
extensions = ['sphinx.ext.autodoc', 'sphinx.ext.napoleon']
napoleon_google_docstring = True
napoleon_numpy_docstring = False
autodoc_member_order = 'bysource'
autodoc_typehints = 'signature'
html_theme = 'alabaster'
# The build runs offline, with no inventory of Python's own documentation to link its types to, and the generated
# module's types are not part of the package's interface.
nitpick_ignore = [
    ('py:class', name)
    for name in (
        'collections.abc.Iterable',
        'eranos._internal.eranos.Group',
        'eranos._internal.eranos.GroupReader',
        'types.TracebackType',
    )
]


def public(app: Sphinx, environment: BuildEnvironment, node: pending_xref, content: nodes.TextElement) -> nodes.reference | None:
    """
    Resolve a reference to a name of one of the package's private modules as the name the package exports.

    Args:
        app: The application.
        environment: The build's environment.
        node: The reference.
        content: What the reference shows.

    Returns:
        The resolved reference, or none for a name the package does not export.

    """
    target = node.get('reftarget', '')
    if not target.startswith('eranos._'):
        return None
    exported = f'eranos.{target.rsplit(".", 1)[-1]}'
    return environment.get_domain('py').resolve_xref(environment, node['refdoc'], app.builder, node['reftype'], exported, node, content)


def defining_module(value: object) -> str | None:
    """
    Name the private module that defines value, if it is a union of the package's, whose docstring sits beside it there.

    Args:
        value: What autodoc documents.

    Returns:
        The module's name, or none for anything else.

    """
    module = value.__module__ if isinstance(value, typing.TypeAliasType) else None
    return module if module is not None and module.startswith('eranos._') else None


# Sphinx calls the handler with these arguments.
# ruff: ignore[too-many-arguments, too-many-positional-arguments]
def included(app: Sphinx, what: str, name: str, value: object, skip: bool, options: Options) -> bool | None:
    """
    Keep the package's unions, which autodoc takes for undocumented imports.

    Args:
        app: The application.
        what: The kind of the object that holds the member.
        name: The member's name.
        value: The member.
        skip: Whether autodoc would skip it.
        options: The directive's options.

    Returns:
        False for a union, and none to leave the choice to autodoc.

    """
    return False if defining_module(value) is not None else None


# Sphinx calls the handler with these arguments.
# ruff: ignore[too-many-arguments, too-many-positional-arguments]
def documented(app: Sphinx, what: str, name: str, value: object, options: Options, lines: list[str]) -> None:
    """
    Take the docstring of a union from beside its definition, in the private module that defines it.

    Args:
        app: The application.
        what: The kind of the object.
        name: Its full name.
        value: The object.
        options: The directive's options.
        lines: Its docstring's lines, which this replaces.

    """
    module = defining_module(value)
    if module is not None:
        found = ModuleAnalyzer.for_module(module).find_attr_docs().get(('', name.rsplit('.', 1)[-1]))
        lines[:] = list(found or [])


def setup(app: Sphinx) -> None:
    """
    Document the package's unions, and resolve the private modules' names as the package's.

    Args:
        app: The application.

    """
    app.connect('autodoc-skip-member', included)
    app.connect('autodoc-process-docstring', documented)
    app.connect('missing-reference', public)
