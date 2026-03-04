//! A priority queue in which elements of the same priority are popped in a LIFO order.

use smallvec::SmallVec;

/// A priority stack in which elements of the same priority are popped in a LIFO order.
// TODO(mingwei): Keep an upper bound on current priority to avoid scanning all stacks?
#[derive(Debug, Clone)]
pub struct PriorityStack<T> {
    /// Note: inner stack `Vec`s may be empty.
    stacks: Vec<SmallVec<[T; 1]>>,
}

impl<T> PriorityStack<T> {
    /// Creates a new, empty `PriorityStack`.
    pub fn new() -> Self {
        Self {
            stacks: Vec::default(),
        }
    }

    /// Creates a new, empty `PriorityStack` with pre-allocated capacity up to the given priority.
    pub fn with_priority_capacity(priority: usize) -> Self {
        Self {
            stacks: Vec::with_capacity(priority),
        }
    }

    /// Pushes an element onto the stack with the given priority.
    pub fn push(&mut self, priority: usize, item: T) {
        if priority >= self.stacks.len() {
            self.stacks.resize_with(priority + 1, Default::default);
        }
        self.stacks[priority].push(item);
    }

    /// Pops an element from the stack with the highest priority.
    pub fn pop(&mut self) -> Option<T> {
        self.stacks
            .iter_mut()
            .rev()
            .filter_map(SmallVec::pop)
            .next()
    }

    /// Pops an element from the stack and return `(priority, item)`.
    pub fn pop_prio(&mut self) -> Option<(usize, T)> {
        self.stacks
            .iter_mut()
            .enumerate()
            .rev()
            .filter_map(|(i, stack)| stack.pop().map(|x| (i, x)))
            .next()
    }

    /// Returns the item with the highest priority without removing it.
    pub fn peek(&self) -> Option<&T> {
        self.stacks
            .iter()
            .rev()
            .filter_map(|stack| stack.last())
            .next()
    }

    /// Returns the item with the highest priority and its priority without removing it.
    pub fn peek_prio(&self) -> Option<(usize, &T)> {
        self.stacks
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(i, stack)| stack.last().map(|x| (i, x)))
            .next()
    }

    /// Returns the number of elements in the `PriorityStack`.
    pub fn len(&self) -> usize {
        self.stacks.iter().map(SmallVec::len).sum()
    }

    /// Returns true if the `PriorityStack` is empty.
    pub fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }
}

impl<T> Default for PriorityStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Extend<(usize, T)> for PriorityStack<T> {
    fn extend<I: IntoIterator<Item = (usize, T)>>(&mut self, iter: I) {
        for (priority, item) in iter {
            self.push(priority, item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let ps: PriorityStack<i32> = PriorityStack::new();
        assert_eq!(ps.len(), 0);
        // is_empty checks stacks.is_empty() which is true for a fresh stack
        assert!(ps.is_empty());
        assert!(ps.peek().is_none());
    }

    #[test]
    fn test_push_pop_single() {
        let mut ps = PriorityStack::new();
        ps.push(0, "hello");

        assert_eq!(ps.len(), 1);
        assert_eq!(ps.pop(), Some("hello"));
        assert_eq!(ps.len(), 0);
        assert_eq!(ps.pop(), None);
    }

    #[test]
    fn test_highest_priority_first() {
        let mut ps = PriorityStack::new();
        ps.push(0, "low");
        ps.push(2, "high");
        ps.push(1, "mid");

        assert_eq!(ps.pop(), Some("high"));
        assert_eq!(ps.pop(), Some("mid"));
        assert_eq!(ps.pop(), Some("low"));
        assert_eq!(ps.pop(), None);
    }

    #[test]
    fn test_lifo_within_same_priority() {
        let mut ps = PriorityStack::new();
        ps.push(1, "A");
        ps.push(1, "B");
        ps.push(1, "C");

        assert_eq!(ps.pop(), Some("C"));
        assert_eq!(ps.pop(), Some("B"));
        assert_eq!(ps.pop(), Some("A"));
        assert_eq!(ps.pop(), None);
    }

    #[test]
    fn test_pop_prio() {
        let mut ps = PriorityStack::new();
        ps.push(0, 10);
        ps.push(3, 30);
        ps.push(1, 20);

        assert_eq!(ps.pop_prio(), Some((3, 30)));
        assert_eq!(ps.pop_prio(), Some((1, 20)));
        assert_eq!(ps.pop_prio(), Some((0, 10)));
        assert_eq!(ps.pop_prio(), None);
    }

    #[test]
    fn test_peek() {
        let mut ps = PriorityStack::new();
        assert_eq!(ps.peek(), None);

        ps.push(0, 1);
        ps.push(2, 3);
        assert_eq!(ps.peek(), Some(&3));

        // peek does not remove the item
        assert_eq!(ps.peek(), Some(&3));
        assert_eq!(ps.len(), 2);
    }

    #[test]
    fn test_peek_prio() {
        let mut ps = PriorityStack::new();
        assert_eq!(ps.peek_prio(), None);

        ps.push(1, "mid");
        ps.push(5, "high");
        ps.push(0, "low");

        assert_eq!(ps.peek_prio(), Some((5, &"high")));
        // peek_prio does not remove
        assert_eq!(ps.len(), 3);
    }

    #[test]
    fn test_len_tracks_correctly() {
        let mut ps = PriorityStack::new();
        assert_eq!(ps.len(), 0);

        ps.push(0, "a");
        assert_eq!(ps.len(), 1);

        ps.push(1, "b");
        assert_eq!(ps.len(), 2);

        ps.push(0, "c");
        assert_eq!(ps.len(), 3);

        ps.pop();
        assert_eq!(ps.len(), 2);

        ps.pop();
        assert_eq!(ps.len(), 1);

        ps.pop();
        assert_eq!(ps.len(), 0);
    }

    #[test]
    fn test_extend() {
        let mut ps = PriorityStack::new();
        ps.extend(vec![(0, "low"), (2, "high"), (1, "mid")]);

        assert_eq!(ps.len(), 3);
        assert_eq!(ps.pop(), Some("high"));
        assert_eq!(ps.pop(), Some("mid"));
        assert_eq!(ps.pop(), Some("low"));
    }

    #[test]
    fn test_mixed_priorities_complex() {
        let mut ps = PriorityStack::new();

        // Push at various priorities
        ps.push(1, "1a");
        ps.push(0, "0a");
        ps.push(2, "2a");
        ps.push(1, "1b");
        ps.push(2, "2b");

        // Pop highest priority first (2), LIFO within same priority
        assert_eq!(ps.pop(), Some("2b"));
        assert_eq!(ps.pop(), Some("2a"));

        // Push more while partially consumed
        ps.push(3, "3a");
        ps.push(1, "1c");

        // Now priority 3 is highest
        assert_eq!(ps.pop(), Some("3a"));

        // Back to priority 1 (LIFO: 1c, 1b, 1a)
        assert_eq!(ps.pop(), Some("1c"));
        assert_eq!(ps.pop(), Some("1b"));
        assert_eq!(ps.pop(), Some("1a"));

        // Finally priority 0
        assert_eq!(ps.pop(), Some("0a"));

        // Empty
        assert_eq!(ps.pop(), None);
        assert_eq!(ps.len(), 0);
    }
}
