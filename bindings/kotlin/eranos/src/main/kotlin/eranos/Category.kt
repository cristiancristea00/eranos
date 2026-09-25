package eranos

/**
 * A label the group gives expenses: "Food".
 *
 * A new category has no identifier; [Group.add] gives it one. The built-in category, Uncategorised, holds every expense
 * without another.
 */
public class Category internal constructor(
    /** The identifier: `null` for a category to add, set for one the group holds. */
    public val id: CategoryId?,
    /** The name, which no other category of the group shares. */
    public val name: String,
) {
    /** A category to add, called [name]. */
    public constructor(name: String) : this(null, name)

    /** The category as the generated code holds it. */
    internal val generated: eranos.internal.Category
        get() = eranos.internal.Category(id?.value, name)

    /** This category renamed [name]; the identifier stays, so the copy can replace the category. */
    public fun copy(name: String = this.name): Category = Category(id, name)

    /** Whether [other] is of the same class, with the same [id] and [name]. */
    override fun equals(other: Any?): Boolean = other is Category && id == other.id && name == other.name

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, name)

    /** A description of every field, for logs. */
    override fun toString(): String = "Category(id=$id, name=$name)"
}

/** The category the generated code holds as [this]. */
internal fun eranos.internal.Category.converted(): Category = Category(id?.let(::CategoryId), name)
