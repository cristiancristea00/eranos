package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/** A day of the calendar, without a time zone: the day an entry happened wherever it happened. */
public final class CalendarDate {
    private final int day;
    private final int month;
    private final int year;

    private CalendarDate(int day, int month, int year) {
        this.day = day;
        this.month = month;
        this.year = year;
    }

    /**
     * The date of {@code day}, {@code month}, and {@code year}.
     *
     * @param day the day of the month, from 1
     * @param month the month, from 1 for January
     * @param year the year, from 1 to 9999
     * @return the date
     * @throws EranosException.Calendar with {@link CalendarFailure.InvalidDate} for a day that does
     *     not exist, such as 30 February
     */
    public static CalendarDate of(int day, int month, int year) throws EranosException {
        CalendarDate date = new CalendarDate(day, month, year);
        return Converting.call(
                () -> {
                    Eranos.calendarDateFormat(date.generated());
                    return date;
                });
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
        return new CalendarDate(date.day(), date.month(), date.year());
    }

    /**
     * The date as the generated code holds it.
     *
     * @return the generated date
     */
    eranos.internal.CalendarDate generated() {
        return new eranos.internal.CalendarDate(day, month, year);
    }

    /**
     * The day of the month.
     *
     * @return the day, from 1
     */
    public int day() {
        return day;
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
     * The year.
     *
     * @return the year, from 1 to 9999
     */
    public int year() {
        return year;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof CalendarDate date
                && day == date.day
                && month == date.month
                && year == date.year;
    }

    @Override
    public int hashCode() {
        return Objects.hash(day, month, year);
    }

    /**
     * The date as the library writes it, in ISO 8601.
     *
     * @return the text: "2026-09-23"
     */
    @Override
    public String toString() {
        try {
            return Eranos.calendarDateFormat(generated());
        } catch (eranos.internal.EranosException exception) {
            return year + "/" + month + "/" + day;
        }
    }
}
