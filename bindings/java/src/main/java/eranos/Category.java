package eranos;

import java.util.Objects;

/**
 * A label the group gives expenses: "Food".
 *
 * <p>A category has the identifier given to {@link #Category(CategoryId, String)}, or one that
 * {@link #Category(String)} generates, and {@link #renamed} keeps it. The built-in category,
 * Uncategorised, holds every expense without another.
 */
public final class Category {
    private final CategoryId id;
    private final String name;

    /**
     * A category to add, called {@code name}, under an identifier generated when it is called;
     * {@link #Category(CategoryId, String)} takes one instead.
     *
     * @param name the name, which no other category of the group may share
     */
    public Category(String name) {
        this(CategoryId.generate(), name);
    }

    /**
     * A category to add, called {@code name}, under the identifier {@code id}, such as the key an
     * app's own store gives it; {@link #Category(String)} generates one instead.
     *
     * @param id the identifier
     * @param name the name, which no other category of the group may share
     */
    public Category(CategoryId id, String name) {
        this.id = Objects.requireNonNull(id, "id");
        this.name = Objects.requireNonNull(name, "name");
    }

    /**
     * The category the generated code holds as {@code category}.
     *
     * @param category the generated category
     * @return the category
     */
    static Category converted(eranos.internal.Category category) {
        return new Category(new CategoryId(category.id()), category.name());
    }

    /**
     * The category as the generated code holds it.
     *
     * @return the generated category
     */
    eranos.internal.Category generated() {
        return new eranos.internal.Category(id.value(), name);
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
     * @return the identifier
     */
    public CategoryId id() {
        return id;
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
                && id.equals(category.id)
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
