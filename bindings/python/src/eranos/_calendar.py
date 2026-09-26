"""Hold dates and times of day, without a time zone, which the library checks when they are made."""

import dataclasses
import functools
from typing import final, override

from eranos._errors import library_errors
from eranos._internal import eranos as generated


@final
@functools.total_ordering
@dataclasses.dataclass(frozen=True, slots=True)
class CalendarDate:
    """
    A day of the calendar, without a time zone: the day an entry happened, wherever it happened.

    The library checks the date when it is made, and refuses a day the calendar does not have, such as 30 February, with
    ``CalendarError.InvalidDate``, in a ``CalendarError``. Dates order by day.
    """

    day: int
    """The day of the month, from 1."""

    month: int
    """The month, from 1 for January."""

    year: int
    """The year, from 1 to 9999."""

    def __post_init__(self) -> None:
        """Check the date with the library."""
        with library_errors():
            generated.calendar_date_format(generated_date(self))

    @classmethod
    def parse(cls, text: str) -> CalendarDate:
        """
        Parse the date written text in ISO 8601: ``"2026-09-23"``.

        The library refuses text that is not a date with ``CalendarError.InvalidDate``, in a ``CalendarError``.

        Args:
            text: The date's text.

        Returns:
            The date.

        """
        with library_errors():
            return date_of(generated.calendar_date_parse(text))

    def __lt__(self, other: CalendarDate) -> bool:
        """
        Tell whether this date comes before other.

        Args:
            other: The date to compare.

        Returns:
            Whether this one is earlier.

        """
        return (self.year, self.month, self.day) < (other.year, other.month, other.day)

    @override
    def __str__(self) -> str:
        """
        Write the date as the library does, in ISO 8601.

        Returns:
            The text, such as ``"2026-09-23"``.

        """
        return generated.calendar_date_format(generated_date(self))


@final
@functools.total_ordering
@dataclasses.dataclass(frozen=True, slots=True)
class TimeOfDay:
    """
    A time of day, to the minute, without a time zone.

    The library checks the time when it is made, and refuses an hour or a minute out of range with
    ``CalendarError.InvalidTime``, in a ``CalendarError``. Times order by minute of the day.
    """

    hour: int
    """The hour, from 0 to 23."""

    minute: int
    """The minute, from 0 to 59."""

    def __post_init__(self) -> None:
        """Check the time with the library."""
        with library_errors():
            generated.time_of_day_format(generated_time(self))

    @classmethod
    def parse(cls, text: str) -> TimeOfDay:
        """
        Parse the time written text in ISO 8601: ``"18:30"``.

        The library refuses text that is not a time with ``CalendarError.InvalidTime``, in a ``CalendarError``.

        Args:
            text: The time's text.

        Returns:
            The time.

        """
        with library_errors():
            return time_of(generated.time_of_day_parse(text))

    def __lt__(self, other: TimeOfDay) -> bool:
        """
        Tell whether this time comes before other.

        Args:
            other: The time to compare.

        Returns:
            Whether this one is earlier.

        """
        return (self.hour, self.minute) < (other.hour, other.minute)

    @override
    def __str__(self) -> str:
        """
        Write the time as the library does, in ISO 8601.

        Returns:
            The text, such as ``"18:30"``.

        """
        return generated.time_of_day_format(generated_time(self))


def date_of(value: generated.CalendarDate) -> CalendarDate:
    """
    Convert the date the generated module holds as value.

    Args:
        value: The generated date.

    Returns:
        The date.

    """
    return CalendarDate(value.day, value.month, value.year)


def generated_date(date: CalendarDate) -> generated.CalendarDate:
    """
    Convert date into the date the generated module holds.

    Args:
        date: The date.

    Returns:
        The generated date.

    """
    return generated.CalendarDate(day=date.day, month=date.month, year=date.year)


def time_of(value: generated.TimeOfDay) -> TimeOfDay:
    """
    Convert the time the generated module holds as value.

    Args:
        value: The generated time.

    Returns:
        The time.

    """
    return TimeOfDay(value.hour, value.minute)


def generated_time(time: TimeOfDay) -> generated.TimeOfDay:
    """
    Convert time into the time the generated module holds.

    Args:
        time: The time.

    Returns:
        The generated time.

    """
    return generated.TimeOfDay(hour=time.hour, minute=time.minute)
