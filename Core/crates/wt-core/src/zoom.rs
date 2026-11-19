//! The levels a page may be zoomed to (012). A shell does not know them:
//! it says "bigger" and "smaller", and is told where that left the page.

/// Actual size, in per cent: what a page nobody zoomed is at.
pub const ACTUAL_SIZE: u32 = 100;

/// Every level, smallest first.
pub const ZOOM_LEVELS: [u32; 13] = [50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300];

/// The next level above `percent`. `None` at the largest. A level that is
/// none of these (written by another build) goes to the first above it.
#[must_use]
pub fn zoom_in(percent: u32) -> Option<u32> {
    ZOOM_LEVELS.iter().copied().find(|level| *level > percent)
}

/// The next level below `percent`. `None` at the smallest.
#[must_use]
pub fn zoom_out(percent: u32) -> Option<u32> {
    ZOOM_LEVELS
        .iter()
        .rev()
        .copied()
        .find(|level| *level < percent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_levels_go_up_one_at_a_time_and_stop_at_the_largest() {
        let mut level = ZOOM_LEVELS[0];
        let mut climbed = vec![level];
        while let Some(next) = zoom_in(level) {
            level = next;
            climbed.push(level);
        }
        assert_eq!(climbed, ZOOM_LEVELS);
        assert_eq!(zoom_in(300), None);
    }

    #[test]
    fn and_down_one_at_a_time_and_stop_at_the_smallest() {
        let mut level = 300;
        let mut descended = vec![level];
        while let Some(next) = zoom_out(level) {
            level = next;
            descended.push(level);
        }
        descended.reverse();
        assert_eq!(descended, ZOOM_LEVELS);
        assert_eq!(zoom_out(50), None);
    }

    #[test]
    fn actual_size_is_a_level_with_one_on_each_side() {
        assert!(ZOOM_LEVELS.contains(&ACTUAL_SIZE));
        assert_eq!(zoom_in(ACTUAL_SIZE), Some(110));
        assert_eq!(zoom_out(ACTUAL_SIZE), Some(90));
    }

    #[test]
    fn a_level_that_is_none_of_them_goes_to_its_neighbours() {
        assert_eq!(zoom_in(120), Some(125));
        assert_eq!(zoom_out(120), Some(110));
        assert_eq!(zoom_in(10), Some(50));
        assert_eq!(zoom_out(10), None);
        assert_eq!(zoom_in(1_000), None);
        assert_eq!(zoom_out(1_000), Some(300));
    }
}
