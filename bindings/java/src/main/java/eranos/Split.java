package eranos;

import java.util.List;

/**
 * How an amount divides among members: equally, by exact amounts, by parts, or by percentages.
 *
 * <p>The library divides every amount so that the shares add up to it exactly, and gives any minor
 * unit left over by the rounding to a member it picks from the expense's identifier.
 */
public sealed interface Split {
    /**
     * The members the split divides among, in the order it lists them.
     *
     * @return the members
     */
    List<MemberId> members();

    /**
     * Every member pays an exact amount; the amounts add up to what they divide.
     *
     * @param shares what every member pays
     */
    record Amounts(List<MemberAmount> shares) implements Split {
        /**
         * The split of {@code shares}, which it copies.
         *
         * @param shares what every member pays
         */
        public Amounts {
            shares = List.copyOf(shares);
        }

        @Override
        public List<MemberId> members() {
            return shares.stream().map(MemberAmount::member).toList();
        }
    }

    /**
     * Every member pays the same.
     *
     * @param members the members
     */
    record Equal(List<MemberId> members) implements Split {
        /**
         * The split among {@code members}, which it copies.
         *
         * @param members the members
         */
        public Equal {
            members = List.copyOf(members);
        }
    }

    /**
     * Every member pays in proportion to their parts.
     *
     * @param shares the parts of every member
     */
    record Parts(List<MemberParts> shares) implements Split {
        /**
         * The split of {@code shares}, which it copies.
         *
         * @param shares the parts of every member
         */
        public Parts {
            shares = List.copyOf(shares);
        }

        @Override
        public List<MemberId> members() {
            return shares.stream().map(MemberParts::member).toList();
        }
    }

    /**
     * Every member pays a percentage; the percentages add up to 100 %.
     *
     * @param shares the percentage of every member
     */
    record Percentages(List<MemberPercentage> shares) implements Split {
        /**
         * The split of {@code shares}, which it copies.
         *
         * @param shares the percentage of every member
         */
        public Percentages {
            shares = List.copyOf(shares);
        }

        @Override
        public List<MemberId> members() {
            return shares.stream().map(MemberPercentage::member).toList();
        }
    }
}
