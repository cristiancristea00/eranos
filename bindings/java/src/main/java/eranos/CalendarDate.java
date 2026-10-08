package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/** A day of the calendar, without a time zone: the day an entry happened wherever it happened. */
public final class CalendarDate {
    private final int year;
    private final int month;
    private final int day;

    private CalendarDate(int year, int month, int day) {
        this.year = year;
        this.month = month;
        this.day = day;
    }

    /**
     * The date of {@code year}, {@code month}, and {@code day}.
     *
     * @param year the year, from 1 to 9999
     * @param month the month, from 1 for January
     * @param day the day of the month, from 1
     * @return the date
     * @throws EranosException.Calendar with {@link CalendarFailure.InvalidDate} for a day that does
     *     not exist, such as 30 February
     */
    public static CalendarDate of(int year, int month, int day) throws EranosException {
        CalendarDate date = new CalendarDate(year, month, day);
        Converting.run(() -> Eranos.calendarDateCheck(date.generated()));
        return date;
    }

    /**
     * The date written {@code text} in ISO 8601: "2026-09-23".
     *
     * @param text the date's text
     * @return the date
     * @throws EranosException.Calendar with {@link CalendarFailure.InvalidDate}
     */
    public static CalendarDate parse(String text) throws EranosException {
        return Converting.call(() -> converted(Eranos.calendarDateParse(text)));
    }

    /**
     * The date the generated code holds as {@code date}.
     *
     * @param date the generated date
     * @return the date
     */
    static CalendarDate converted(eranos.internal.CalendarDate date) {
        return new CalendarDate(date.year(), date.month(), date.day());
    }

    /**
     * The date as the generated code holds it.
     *
     * @return the generated date
     */
    eranos.internal.CalendarDate generated() {
        return new eranos.internal.CalendarDate(year, month, day);
    }

    /**
     * The year.
     *
     * @return the year, from 1 to 9999
     */
    public int year() {
        return year;
    }

    /**
     * The month.
     *
     * @return the month, from 1 for January
     */
    public int month() {
        return month;
    }

    /**
     * The day of the month.
     *
     * @return the day, from 1
     */
    public int day() {
        return day;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof CalendarDate date
                && year == date.year
                && month == date.month
                && day == date.day;
    }

    @Override
    public int hashCode() {
        return Objects.hash(year, month, day);
    }

    /**
     * The date as the library writes it, in ISO 8601.
     *
     * @return the text: "2026-09-23"
     */
    @Override
    public String toString() {
        return Eranos.calendarDateFormat(generated());
    }
}
