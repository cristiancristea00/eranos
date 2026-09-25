package eranos;

import java.util.Objects;

/**
 * What the expenses of a category add up to, in the group currency.
 *
 * @param category the category
 * @param total the total
 */
public record CategoryTotal(CategoryId category, Money total) {
    /**
     * The category total of these components.
     *
     * @param category the category
     * @param total the total
     */
    public CategoryTotal {
        Objects.requireNonNull(category, "category");
        Objects.requireNonNull(total, "total");
    }

    /**
     * The category total the generated code holds as {@code categoryTotal}.
     *
     * @param categoryTotal the generated value
     * @return the value
     */
    static CategoryTotal converted(eranos.internal.CategoryTotal categoryTotal) {
        return new CategoryTotal(
                new CategoryId(categoryTotal.category()), Money.converted(categoryTotal.total()));
    }
}
