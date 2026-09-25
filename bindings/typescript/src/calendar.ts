/**
 * Dates and times of day, without a time zone.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import * as internal from './internal/eranos.js';

/** A day of the calendar, without a time zone: the day an entry happened wherever it happened. */
export class CalendarDate {
    private constructor(
        /** The day of the month, from 1. */
        readonly day: number,
        /** The month, from 1 for January. */
        readonly month: number,
        /** The year, from 1 to 9999. */
        readonly year: number,
    ) {}

    /**
     * The date of `day`, `month`, and `year`.
     *
     * @param day - The day of the month, from 1
     * @param month - The month, from 1 for January
     * @param year - The year, from 1 to 9999
     * @returns The date
     * @throws {@link CalendarError} with the failure `invalidDate` for a day that does not exist, such as 30 February
     */
    static of(day: number, month: number, year: number): CalendarDate {
        const date = new CalendarDate(day, month, year);
        calling(() => internal.calendarDateFormat(date.generated()));
        return date;
    }

    /**
     * The date written `text` in ISO 8601: "2026-09-23".
     *
     * @param text - The date's text
     * @returns The date
     * @throws {@link CalendarError} with the failure `invalidDate`
     */
    static parse(text: string): CalendarDate {
        return CalendarDate.converted(calling(() => internal.calendarDateParse(text)));
    }

    /**
     * The date the generated code holds as `date`.
     *
     * @internal
     */
    static converted(date: internal.CalendarDate): CalendarDate {
        return new CalendarDate(date.day, date.month, date.year);
    }

    /**
     * The date as the generated code holds it.
     *
     * @internal
     */
    generated(): internal.CalendarDate {
        return internal.CalendarDate.create({ day: this.day, month: this.month, year: this.year });
    }

    /**
     * Whether `other` is the same day.
     *
     * @param other - The date to compare
     * @returns Whether the two are equal
     */
    equals(other: CalendarDate): boolean {
        return this.day === other.day && this.month === other.month && this.year === other.year;
    }

    /**
     * The date as the library writes it, in ISO 8601.
     *
     * @returns The text: "2026-09-23"
     */
    toString(): string {
        try {
            return internal.calendarDateFormat(this.generated());
        } catch {
            return `${this.year.toString()}/${this.month.toString()}/${this.day.toString()}`;
        }
    }
}

/** A time of day, to the minute, without a time zone. */
export class TimeOfDay {
    private constructor(
        /** The hour, from 0 to 23. */
        readonly hour: number,
        /** The minute, from 0 to 59. */
        readonly minute: number,
    ) {}

    /**
     * The time of `hour` and `minute`.
     *
     * @param hour - The hour, from 0 to 23
     * @param minute - The minute, from 0 to 59
     * @returns The time
     * @throws {@link CalendarError} with the failure `invalidTime` for an hour or a minute out of range
     */
    static of(hour: number, minute: number): TimeOfDay {
        const time = new TimeOfDay(hour, minute);
        calling(() => internal.timeOfDayFormat(time.generated()));
        return time;
    }

    /**
     * The time written `text` in ISO 8601: "18:30".
     *
     * @param text - The time's text
     * @returns The time
     * @throws {@link CalendarError} with the failure `invalidTime`
     */
    static parse(text: string): TimeOfDay {
        return TimeOfDay.converted(calling(() => internal.timeOfDayParse(text)));
    }

    /**
     * The time the generated code holds as `time`.
     *
     * @internal
     */
    static converted(time: internal.TimeOfDay): TimeOfDay {
        return new TimeOfDay(time.hour, time.minute);
    }

    /**
     * The time as the generated code holds it.
     *
     * @internal
     */
    generated(): internal.TimeOfDay {
        return internal.TimeOfDay.create({ hour: this.hour, minute: this.minute });
    }

    /**
     * Whether `other` is the same time of day.
     *
     * @param other - The time to compare
     * @returns Whether the two are equal
     */
    equals(other: TimeOfDay): boolean {
        return this.hour === other.hour && this.minute === other.minute;
    }

    /**
     * The time as the library writes it, in ISO 8601.
     *
     * @returns The text: "18:30"
     */
    toString(): string {
        try {
            return internal.timeOfDayFormat(this.generated());
        } catch {
            return `${this.hour.toString()}h${this.minute.toString()}`;
        }
    }
}
