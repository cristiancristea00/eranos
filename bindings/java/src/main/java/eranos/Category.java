package eranos;

import java.util.Objects;
import java.util.Optional;

/**
 * A label the group gives expenses: "Food".
 *
 * <p>A new category has no identifier; {@link ExpenseGroup#addCategory} gives it one. The built-in
 * category, Uncategorised, holds every expense without another.
 */
public final class Category {
    private final CategoryId id;
    private final String name;

    private Category(CategoryId id, String name) {
        this.id = id;
        this.name = Objects.requireNonNull(name, "name");
    }

    /**
     * A category to add, called {@code name}.
     *
     * @param name the name, which no other category of the group may share
     */
    public Category(String name) {
        this(null, name);
    }

    /**
     * The category the generated code holds as {@code category}.
     *
     * @param category the generated category
     * @return the category
     */
    static Category converted(eranos.internal.Category category) {
        return new Category(
                null == category.id() ? null : new CategoryId(category.id()), category.name());
    }

    /**
     * The category as the generated code holds it.
     *
     * @return the generated category
     */
    eranos.internal.Category generated() {
        return new eranos.internal.Category(null == id ? null : id.value(), name);
    }

    /**
     * This category called {@code name}, with its identifier, so it can replace this one.
     *
     * @param name the new name
     * @return the renamed category
     */
    public Category renamed(String name) {
        return new Category(id, name);
    }

    /**
     * The identifier.
     *
     * @return the identifier, empty for a category to add
     */
    public Optional<CategoryId> id() {
        return Optional.ofNullable(id);
    }

    /**
     * The name, which no other category of the group shares.
     *
     * @return the name
     */
    public String name() {
        return name;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Category category
                && Objects.equals(id, category.id)
                && name.equals(category.name);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, name);
    }

    @Override
    public String toString() {
        return "Category[id=" + id + ", name=" + name + "]";
    }
}
