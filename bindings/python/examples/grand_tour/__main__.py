"""Tell the story of the tour: a week in Portugal, a section per part of the library."""

import asyncio

from examples.grand_tour.edits import another_currency, comings_and_goings, edits
from examples.grand_tour.expenses import expenses
from examples.grand_tour.members import add_categories, members, the_group
from examples.grand_tour.saving import saving_and_loading
from examples.grand_tour.sharing import sharing
from examples.grand_tour.totals import new_currency, removing_a_category, settling_up, summary
from examples.grand_tour.values import values
from examples.report import print_lines, title


def main() -> None:
    """Print the report, checking its key numbers."""
    title('Lisbon and Porto: a grand tour of eranos')
    print_lines(
        "Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each",
        'section below uses another part of the library, from creating the group to saving',
        'it, and checks its key numbers.',
    )
    with the_group() as group:
        people = members(group)
        categories = add_categories(group)
        seafood = expenses(group, people, categories)
        another_currency(group, people, categories)
        edits(group, people, categories, seafood)
        comings_and_goings(group, people)
        summary(group, people, categories)
        removing_a_category(group, categories)
        new_currency(group, people)
        settling_up(group)
        saving_and_loading(group, people, categories)
        values(people)
        asyncio.run(sharing(group, people))


if __name__ == '__main__':
    main()
