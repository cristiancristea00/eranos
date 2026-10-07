package eranos

import eranos.internal.calendarDateFormat
import eranos.internal.calendarDateParse
import eranos.internal.timeOfDayFormat
import eranos.internal.timeOfDayParse

/** A day of the calendar, without a time zone: the day an entry happened wherever it happened. */
public class CalendarDate internal constructor(
    /** The year, from 1 to 9999. */
    public val year: Int,
    /** The month, from 1 for January. */
    public val month: Int,
    /** The day of the month, from 1. */
    public val day: Int,
) {
    /** The date as the generated code holds it. */
    internal val generated: eranos.internal.CalendarDate
        get() = eranos.internal.CalendarDate(year, month, day)

    /** Whether [other] is of the same class, with the same [year], [month], and [day]. */
    override fun equals(other: Any?): Boolean = other is CalendarDate && year == other.year && month == other.month && day == other.day

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(year, month, day)

    /** The date as the library writes it, in ISO 8601: "2026-09-23". */
    override fun toString(): String =
        try {
            calendarDateFormat(generated)
        } catch (_: eranos.internal.EranosException) {
            "$year/$month/$day"
        }

    /** The date of a year, a month, and a day, and the parsing of one from its text. */
    public companion object {
        /**
         * The date of [year], [month], and [day].
         *
         * @throws EranosException.Calendar with [CalendarFailure.InvalidDate] for a day that does not exist, such as
         *   30 February.
         */
        @Throws(EranosException::class)
        public fun of(
            year: Int,
            month: Int,
            day: Int,
        ): CalendarDate =
            converting {
                val date = CalendarDate(year, month, day)
                calendarDateFormat(date.generated)
                date
            }

        /**
         * The date written [text] in ISO 8601: "2026-09-23".
         *
         * @throws EranosException.Calendar with [CalendarFailure.InvalidDate].
         */
        @Throws(EranosException::class)
        public fun parse(text: String): CalendarDate = converting { calendarDateParse(text).converted() }
    }
}

/** The date the generated code holds as [this]. */
internal fun eranos.internal.CalendarDate.converted(): CalendarDate = CalendarDate(year, month, day)

/** A time of day, to the minute, without a time zone. */
public class TimeOfDay internal constructor(
    /** The hour, from 0 to 23. */
    public val hour: Int,
    /** The minute, from 0 to 59. */
    public val minute: Int,
) {
    /** The time as the generated code holds it. */
    internal val generated: eranos.internal.TimeOfDay
        get() = eranos.internal.TimeOfDay(hour, minute)

    /** Whether [other] is of the same class, with the same [hour] and [minute]. */
    override fun equals(other: Any?): Boolean = other is TimeOfDay && hour == other.hour && minute == other.minute

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(hour, minute)

    /** The time as the library writes it, in ISO 8601: "18:30". */
    override fun toString(): String =
        try {
            timeOfDayFormat(generated)
        } catch (_: eranos.internal.EranosException) {
            "${hour}h$minute"
        }

    /** The time of an hour and a minute, and the parsing of one from its text. */
    public companion object {
        /**
         * The time of [hour] and [minute].
         *
         * @throws EranosException.Calendar with [CalendarFailure.InvalidTime] for an hour or a minute out of range.
         */
        @Throws(EranosException::class)
        public fun of(
            hour: Int,
            minute: Int,
        ): TimeOfDay =
            converting {
                val time = TimeOfDay(hour, minute)
                timeOfDayFormat(time.generated)
                time
            }

        /**
         * The time written [text] in ISO 8601: "18:30".
         *
         * @throws EranosException.Calendar with [CalendarFailure.InvalidTime].
         */
        @Throws(EranosException::class)
        public fun parse(text: String): TimeOfDay = converting { timeOfDayParse(text).converted() }
    }
}

/** The time the generated code holds as [this]. */
internal fun eranos.internal.TimeOfDay.converted(): TimeOfDay = TimeOfDay(hour, minute)
