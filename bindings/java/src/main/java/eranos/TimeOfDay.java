package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/** A time of day, to the minute, without a time zone. */
public final class TimeOfDay {
    private final int hour;
    private final int minute;

    private TimeOfDay(int hour, int minute) {
        this.hour = hour;
        this.minute = minute;
    }

    /**
     * The time of {@code hour} and {@code minute}.
     *
     * @param hour the hour, from 0 to 23
     * @param minute the minute, from 0 to 59
     * @return the time
     * @throws EranosException.Calendar with {@link CalendarFailure.InvalidTime} for an hour or a
     *     minute out of range
     */
    public static TimeOfDay of(int hour, int minute) throws EranosException {
        TimeOfDay time = new TimeOfDay(hour, minute);
        Converting.run(() -> Eranos.timeOfDayCheck(time.generated()));
        return time;
    }

    /**
     * The time written {@code text} in ISO 8601: "18:30".
     *
     * @param text the time's text
     * @return the time
     * @throws EranosException.Calendar with {@link CalendarFailure.InvalidTime}
     */
    public static TimeOfDay parse(String text) throws EranosException {
        return Converting.call(() -> converted(Eranos.timeOfDayParse(text)));
    }

    /**
     * The time the generated code holds as {@code time}.
     *
     * @param time the generated time
     * @return the time
     */
    static TimeOfDay converted(eranos.internal.TimeOfDay time) {
        return new TimeOfDay(time.hour(), time.minute());
    }

    /**
     * The time as the generated code holds it.
     *
     * @return the generated time
     */
    eranos.internal.TimeOfDay generated() {
        return new eranos.internal.TimeOfDay(hour, minute);
    }

    /**
     * The hour.
     *
     * @return the hour, from 0 to 23
     */
    public int hour() {
        return hour;
    }

    /**
     * The minute.
     *
     * @return the minute, from 0 to 59
     */
    public int minute() {
        return minute;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof TimeOfDay time && hour == time.hour && minute == time.minute;
    }

    @Override
    public int hashCode() {
        return Objects.hash(hour, minute);
    }

    /**
     * The time as the library writes it, in ISO 8601.
     *
     * @return the text: "18:30"
     */
    @Override
    public String toString() {
        return Eranos.timeOfDayFormat(generated());
    }
}
