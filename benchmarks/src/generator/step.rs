/// `usize` Key generator separated by a step value.
///
/// The values yielded on each iteration are increasing by a `step`.
/// Once the iterator has yielded a set amount of elements, it cycles.
/// On each cycle, the starting point is
/// incremented by one (starting from `0`) up to the
/// `step`. The starting point of a cycle will also cycle
/// everytime the iterator has yielded all of its `step_size * cycle_size`
/// distinct values in `[0, step_size * cycle_size[`.
#[derive(Clone, Copy, Debug)]
pub struct StepGenerator {
    // Increment between two keys.
    step_size: usize,
    // Number of keys to output in a cycle.
    cycle_size: usize,
    // Whether to increment the start value when starting a new cycle.
    // This value is in [ 0, step_size [
    cycle_increment: usize,
    // Current value: in [ 0, cycle_size [
    pos: usize,
    // The initial position at the beginning of the cycle.
    // value in: [ 0, step_size [
    initial_pos: usize,
}

impl StepGenerator {
    /// Create a new `Step` `Iterator`.
    ///
    /// The values yielded on each iteration are increasing by `step_size`,
    /// unless the end of the cycle is reached.
    /// The end of a cycle is reached every time `cycle_size` values
    /// have been yielded.
    /// On each cycle, the starting point is
    /// incremented by one (starting from `0`) up to
    /// `step_size`. The starting point of a cycle will also cycle
    /// everytime the iterator has yielded all of its `step_size * cycle_size`
    /// distinct values in `[0, step_size * cycle_size[`.
    pub fn new(cycle_size: usize, step_size: usize) -> Self {
        StepGenerator {
            step_size,
            cycle_size,
            cycle_increment: 1usize,
            pos: 0usize,
            initial_pos: 0usize,
        }
    }
}

impl Iterator for StepGenerator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.initial_pos + self.pos * self.step_size;

        self.pos += 1;
        if self.pos > self.cycle_size {
            self.pos = 0;
            self.initial_pos =
                (self.initial_pos + self.cycle_increment) % self.step_size;
        }
        Some(val)
    }
}
