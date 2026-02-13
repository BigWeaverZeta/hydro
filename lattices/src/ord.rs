use std::cmp::Ordering;

use crate::{DeepReveal, IsBot, IsTop, LatticeFrom, LatticeOrd, Merge};

/// A totally ordered max lattice. Merging returns the larger value.
///
/// Note that the [`Default::default()`] value for numeric type is `MIN`, not zero.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Max<T>(T);
impl<T> Max<T> {
    /// Create a new `Max` lattice instance from a `T`.
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Create a new `Max` lattice instance from an `Into<T>` value.
    pub fn from(val: impl Into<T>) -> Self {
        Self::new(val.into())
    }

    /// Reveal the inner value as a shared reference.
    pub fn as_reveal_ref(&self) -> &T {
        &self.0
    }

    /// Reveal the inner value as an exclusive reference.
    pub fn as_reveal_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Gets the inner by value, consuming self.
    pub fn into_reveal(self) -> T {
        self.0
    }
}

impl<T> DeepReveal for Max<T> {
    type Revealed = T;

    fn deep_reveal(self) -> Self::Revealed {
        self.0
    }
}

impl<T> Merge<Max<T>> for Max<T>
where
    T: Ord,
{
    fn merge(&mut self, other: Max<T>) -> bool {
        if self.0 < other.0 {
            self.0 = other.0;
            true
        } else {
            false
        }
    }
}

impl<T> LatticeFrom<Max<T>> for Max<T> {
    fn lattice_from(other: Max<T>) -> Self {
        other
    }
}

impl<T> LatticeOrd<Self> for Max<T> where Self: PartialOrd<Self> {}

/// A totally ordered min lattice. Merging returns the smaller value.
///
/// This means the lattice order is the reverse of what you might naturally expect: 0 is greater
/// than 1.
///
/// Note that the [`Default::default()`] value for numeric type is `MAX`, not zero.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Min<T>(T);
impl<T> Min<T> {
    /// Create a new `Min` lattice instance from a `T`.
    pub fn new(val: T) -> Self {
        Self(val)
    }

    /// Create a new `Min` lattice instance from an `Into<T>` value.
    pub fn new_from(val: impl Into<T>) -> Self {
        Self::new(val.into())
    }

    /// Reveal the inner value as a shared reference.
    pub fn as_reveal_ref(&self) -> &T {
        &self.0
    }

    /// Reveal the inner value as an exclusive reference.
    pub fn as_reveal_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Gets the inner by value, consuming self.
    pub fn into_reveal(self) -> T {
        self.0
    }
}

impl<T> DeepReveal for Min<T> {
    type Revealed = T;

    fn deep_reveal(self) -> Self::Revealed {
        self.0
    }
}

impl<T> Merge<Min<T>> for Min<T>
where
    T: Ord,
{
    fn merge(&mut self, other: Min<T>) -> bool {
        if other.0 < self.0 {
            self.0 = other.0;
            true
        } else {
            false
        }
    }
}

impl<T> LatticeFrom<Min<T>> for Min<T> {
    fn lattice_from(other: Min<T>) -> Self {
        other
    }
}

impl<T> PartialOrd for Min<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0).map(Ordering::reverse)
    }
}
impl<T> LatticeOrd<Self> for Min<T> where Self: PartialOrd<Self> {}

impl<T> Ord for Min<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

// IsTop, IsBot, Default impls
impl IsTop for Max<()> {
    fn is_top(&self) -> bool {
        true
    }
}
impl IsBot for Max<()> {
    fn is_bot(&self) -> bool {
        true
    }
}
impl IsTop for Min<()> {
    fn is_top(&self) -> bool {
        true
    }
}
impl IsBot for Min<()> {
    fn is_bot(&self) -> bool {
        true
    }
}

impl IsTop for Max<bool> {
    fn is_top(&self) -> bool {
        self.0
    }
}
impl IsBot for Max<bool> {
    fn is_bot(&self) -> bool {
        !self.0
    }
}
impl Default for Max<bool> {
    fn default() -> Self {
        Self(false)
    }
}
impl IsTop for Min<bool> {
    fn is_top(&self) -> bool {
        !self.0
    }
}
impl IsBot for Min<bool> {
    fn is_bot(&self) -> bool {
        self.0
    }
}
impl Default for Min<bool> {
    fn default() -> Self {
        Self(true)
    }
}

impl IsTop for Max<char> {
    fn is_top(&self) -> bool {
        char::MAX == self.0
    }
}
impl IsBot for Max<char> {
    fn is_bot(&self) -> bool {
        '\x00' == self.0
    }
}
impl Default for Max<char> {
    fn default() -> Self {
        Self('\x00')
    }
}
impl IsTop for Min<char> {
    fn is_top(&self) -> bool {
        '\x00' == self.0
    }
}
impl IsBot for Min<char> {
    fn is_bot(&self) -> bool {
        char::MAX == self.0
    }
}
impl Default for Min<char> {
    fn default() -> Self {
        Self(char::MAX)
    }
}

macro_rules! impls_numeric {
    (
        $( $x:ty ),*
    ) => {
        $(
            impl IsTop for Max<$x> {
                fn is_top(&self) -> bool {
                    <$x>::MAX == self.0
                }
            }
            impl IsBot for Max<$x> {
                fn is_bot(&self) -> bool {
                    <$x>::MIN == self.0
                }
            }

            impl Default for Max<$x> {
                fn default() -> Self {
                    Self(<$x>::MIN)
                }
            }

            impl IsTop for Min<$x> {
                fn is_top(&self) -> bool {
                    <$x>::MIN == self.0
                }
            }
            impl IsBot for Min<$x> {
                fn is_bot(&self) -> bool {
                    <$x>::MAX == self.0
                }
            }
            impl Default for Min<$x> {
                fn default() -> Self {
                    Self(<$x>::MAX)
                }
            }
        )*
    };
}
impls_numeric! {
    isize, i8, i16, i32, i64, i128, usize, u8, u16, u32, u64, u128
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering::*;

    use super::*;
    use crate::test::check_all;

    #[test]
    fn ordering() {
        assert_eq!(Max::new(0).cmp(&Max::new(0)), Equal);
        assert_eq!(Max::new(0).cmp(&Max::new(1)), Less);
        assert_eq!(Max::new(1).cmp(&Max::new(0)), Greater);

        assert_eq!(Min::new(0).cmp(&Min::new(0)), Equal);
        assert_eq!(Min::new(0).cmp(&Min::new(1)), Greater);
        assert_eq!(Min::new(1).cmp(&Min::new(0)), Less);
    }

    #[test]
    fn eq() {
        assert!(Max::new(0).eq(&Max::new(0)));
        assert!(!Max::new(0).eq(&Max::new(1)));
        assert!(!Max::new(1).eq(&Max::new(0)));

        assert!(Min::new(0).eq(&Min::new(0)));
        assert!(!Min::new(0).eq(&Min::new(1)));
        assert!(!Min::new(1).eq(&Min::new(0)));
    }

    #[test]
    fn consistency_max_bool() {
        let items = &[Max::new(false), Max::new(true)];
        check_all(items);
    }

    #[test]
    fn consistency_min_bool() {
        let items = &[Min::new(false), Min::new(true)];
        check_all(items);
    }

    #[test]
    fn consistency_max_char() {
        let items: Vec<_> = "\x00\u{10FFFF}✨🤦‍♀️踊るx".chars().map(Max::new).collect();
        check_all(&items);
    }

    #[test]
    fn consistency_min_char() {
        let items: Vec<_> = "\x00\u{10FFFF}✨🤦‍♀️踊るx".chars().map(Min::new).collect();
        check_all(&items);
    }

    #[test]
    fn consistency_max_i32() {
        let items = &[
            Max::new(0),
            Max::new(1),
            Max::new(i32::MIN),
            Max::new(i32::MAX),
        ];
        check_all(items);
    }

    #[test]
    fn consistency_min_i32() {
        let items = &[
            Min::new(0),
            Min::new(1),
            Min::new(i32::MIN),
            Min::new(i32::MAX),
        ];
        check_all(items);
    }

    #[test]
    fn max_merge_takes_larger() {
        let mut a = Max::new(3);
        assert!(a.merge(Max::new(5)));
        assert_eq!(*a.as_reveal_ref(), 5);

        assert!(!a.merge(Max::new(2)));
        assert_eq!(*a.as_reveal_ref(), 5);

        assert!(!a.merge(Max::new(5)));
        assert_eq!(*a.as_reveal_ref(), 5);
    }

    #[test]
    fn min_merge_takes_smaller() {
        let mut a = Min::new(5);
        assert!(a.merge(Min::new(3)));
        assert_eq!(*a.as_reveal_ref(), 3);

        assert!(!a.merge(Min::new(7)));
        assert_eq!(*a.as_reveal_ref(), 3);

        assert!(!a.merge(Min::new(3)));
        assert_eq!(*a.as_reveal_ref(), 3);
    }

    #[test]
    fn max_is_bot_and_top_numeric() {
        assert!(Max::new(u8::MIN).is_bot());
        assert!(!Max::new(u8::MIN).is_top());
        assert!(Max::new(u8::MAX).is_top());
        assert!(!Max::new(u8::MAX).is_bot());
        assert!(!Max::new(42_u8).is_bot());
        assert!(!Max::new(42_u8).is_top());
    }

    #[test]
    fn min_is_bot_and_top_numeric() {
        assert!(Min::new(u8::MAX).is_bot());
        assert!(!Min::new(u8::MAX).is_top());
        assert!(Min::new(u8::MIN).is_top());
        assert!(!Min::new(u8::MIN).is_bot());
    }

    #[test]
    fn max_default_is_min_value() {
        assert_eq!(*Max::<i32>::default().as_reveal_ref(), i32::MIN);
        assert_eq!(*Max::<u64>::default().as_reveal_ref(), u64::MIN);
        assert_eq!(*Max::<bool>::default().as_reveal_ref(), false);
    }

    #[test]
    fn min_default_is_max_value() {
        assert_eq!(*Min::<i32>::default().as_reveal_ref(), i32::MAX);
        assert_eq!(*Min::<u64>::default().as_reveal_ref(), u64::MAX);
        assert_eq!(*Min::<bool>::default().as_reveal_ref(), true);
    }

    #[test]
    fn max_deep_reveal() {
        assert_eq!(Max::new(42).deep_reveal(), 42);
    }

    #[test]
    fn min_deep_reveal() {
        assert_eq!(Min::new(42).deep_reveal(), 42);
    }

    #[test]
    fn max_merge_owned() {
        let result = Merge::merge_owned(Max::new(3), Max::new(7));
        assert_eq!(*result.as_reveal_ref(), 7);
    }

    #[test]
    fn min_merge_owned() {
        let result = Merge::merge_owned(Min::new(3), Min::new(7));
        assert_eq!(*result.as_reveal_ref(), 3);
    }

    #[test]
    fn max_bool_lattice() {
        assert!(Max::new(false).is_bot());
        assert!(Max::new(true).is_top());
        let mut a = Max::new(false);
        assert!(a.merge(Max::new(true)));
        assert_eq!(*a.as_reveal_ref(), true);
        assert!(!a.merge(Max::new(false)));
    }

    #[test]
    fn min_bool_lattice() {
        assert!(Min::new(true).is_bot());
        assert!(Min::new(false).is_top());
        let mut a = Min::new(true);
        assert!(a.merge(Min::new(false)));
        assert_eq!(*a.as_reveal_ref(), false);
        assert!(!a.merge(Min::new(true)));
    }

    #[test]
    fn consistency_max_u8() {
        let items = &[
            Max::new(0_u8),
            Max::new(1_u8),
            Max::new(128_u8),
            Max::new(u8::MIN),
            Max::new(u8::MAX),
        ];
        check_all(items);
    }

    #[test]
    fn consistency_min_u8() {
        let items = &[
            Min::new(0_u8),
            Min::new(1_u8),
            Min::new(128_u8),
            Min::new(u8::MIN),
            Min::new(u8::MAX),
        ];
        check_all(items);
    }

    #[test]
    fn max_as_reveal_mut() {
        let mut m = Max::new(10);
        *m.as_reveal_mut() = 20;
        assert_eq!(m.into_reveal(), 20);
    }

    #[test]
    fn min_as_reveal_mut() {
        let mut m = Min::new(10);
        *m.as_reveal_mut() = 5;
        assert_eq!(m.into_reveal(), 5);
    }

    #[test]
    fn max_from() {
        let m = Max::from(42_u32);
        assert_eq!(*m.as_reveal_ref(), 42_u32);
    }

    #[test]
    fn min_new_from() {
        let m = Min::new_from(42_u32);
        assert_eq!(*m.as_reveal_ref(), 42_u32);
    }

    #[test]
    fn max_unit_is_both_bot_and_top() {
        let m = Max::new(());
        assert!(m.is_bot());
        assert!(m.is_top());
    }

    #[test]
    fn min_unit_is_both_bot_and_top() {
        let m = Min::new(());
        assert!(m.is_bot());
        assert!(m.is_top());
    }

    #[test]
    fn min_partial_ord_reversed() {
        // Min reverses the ordering: smaller inner values are "greater" in the lattice
        assert!(Min::new(0) > Min::new(1));
        assert!(Min::new(1) < Min::new(0));
        assert_eq!(Min::new(5).partial_cmp(&Min::new(5)), Some(Equal));
    }
}
