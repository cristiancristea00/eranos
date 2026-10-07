package eranos

/**
 * A label the group gives expenses: "Food".
 *
 * A category has its identifier from the moment it is made: the app's own, such as the key of its own store, or a
 * generated one. The built-in category, Uncategorised, holds every expense without another.
 *
 * @constructor A category called [name], under the identifier [id]; the constructor without [id] generates one.
 */
public class Category(
    /** The identifier, which the app gives: its own, or one [CategoryId.generate] makes. */
    public val id: CategoryId,
    /** The name, which no other category of the group shares. */
    public val name: String,
) {
    /** A category called [name], under an identifier [CategoryId.generate] makes; the constructor with [id] takes the app's. */
    public constructor(name: String) : this(CategoryId.generate(), name)

    /** The category as the generated code holds it. */
    internal val generated: eranos.internal.Category
        get() = eranos.internal.Category(id.value, name)

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
internal fun eranos.internal.Category.converted(): Category = Category(CategoryId(id), name)
