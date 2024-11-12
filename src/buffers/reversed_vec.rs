use std::{cmp::Ordering, mem::MaybeUninit, ptr};

use crate::{decimals::decimal_type::DecimalType, level::Level};

#[derive(Debug, Clone)]
pub struct ReversedVec<const N: usize, V: DecimalType> {
    buf: Box<[Level<V>; N]>,
    limit: V,
    /// Track actual number of valid levels
    pub len: usize,
    /// Cache the first level for fast access
    cached_first: Option<Level<V>>,
}

impl<const N: usize, V> ReversedVec<N, V>
where
    V: DecimalType + PartialOrd + Copy + Ord,
{
    #[inline]
    #[must_use]
    pub fn new(is_reversed: bool) -> Self {
        // Safety: Initialize array using MaybeUninit for better performance
        let buf = unsafe {
            let mut buf = Box::new(MaybeUninit::<[Level<V>; N]>::uninit());
            let bound = Level::bound(!is_reversed);

            // Initialize using ptr::write for better performance
            for i in 0..N {
                ptr::addr_of_mut!((*buf.as_mut_ptr())[i]).write(bound);
            }
            buf.assume_init()
        };

        Self { buf, limit: if is_reversed { V::MAX } else { V::MIN }, len: 0, cached_first: None }
    }

    #[inline(always)]
    unsafe fn invalidate_cache(&mut self) {
        self.cached_first = if self.len > 0 {
            let first = self.get_unchecked(0);
            (first.price != self.limit).then_some(*first)
        } else {
            None
        };
    }

    #[inline(always)]
    #[must_use]
    /// # Safety
    /// `index` must be less than `self.len`
    pub unsafe fn get_unchecked(&self, index: usize) -> &Level<V> {
        self.buf.get_unchecked(index)
    }

    #[inline(always)]
    /// # Safety
    /// `index` must be less than `self.len`
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Level<V> {
        self.buf.get_unchecked_mut(index)
    }

    #[inline(always)]
    pub fn find_index(&self, price: V, is_reversed: bool) -> Result<usize, usize> {
        // Fast path for empty buffer
        if self.len == 0 {
            return Err(0);
        }

        // Fast path for beyond bounds
        unsafe {
            if is_reversed {
                if price < self.get_unchecked(0).price {
                    return Err(0);
                }
                if price > self.get_unchecked(self.len - 1).price {
                    return Err(self.len);
                }
            } else {
                if price > self.get_unchecked(0).price {
                    return Err(0);
                }
                if price < self.get_unchecked(self.len - 1).price {
                    return Err(self.len);
                }
            }
        }

        // Use SIMD-friendly search for larger arrays
        if self.len >= 32 {
            return self.simd_search(price, is_reversed);
        }

        // Regular binary search for small ranges
        let mut left = 0;
        let mut right = self.len;

        while left < right {
            let mid = left + (right - left) / 2;
            unsafe {
                let level_price = self.get_unchecked(mid).price;
                match price.cmp(&level_price) {
                    Ordering::Equal => return Ok(mid),
                    Ordering::Less if is_reversed => right = mid,
                    Ordering::Greater if !is_reversed => right = mid,
                    Ordering::Less | Ordering::Greater => left = mid + 1,
                }
            }
        }

        Err(left)
    }

    #[inline(always)]
    fn simd_search(&self, price: V, is_reversed: bool) -> Result<usize, usize> {
        let mut size = self.len;
        let mut left = 0;

        while size > 1 {
            let half = size / 2;
            let mid = left + half;

            unsafe {
                let level_price = self.get_unchecked(mid).price;
                let cmp = price.cmp(&level_price);

                // Update left based on comparison and direction
                left =
                    if (is_reversed && cmp == Ordering::Greater) || (!is_reversed && cmp == Ordering::Less) { mid } else { left };
                size -= half;
            }
        }

        unsafe {
            if self.get_unchecked(left).price == price {
                Ok(left)
            } else {
                Err(left + 1)
            }
        }
    }

    #[inline(always)]
    pub fn bulk_insert(&mut self, levels: &[Level<V>]) {
        let available_space = N - self.len;
        let insert_count = levels.len().min(available_space);

        if insert_count > 0 {
            unsafe {
                ptr::copy_nonoverlapping(levels.as_ptr(), self.buf.as_mut_ptr().add(self.len), insert_count);
                self.len += insert_count;
                self.invalidate_cache();
            }
        }
    }

    #[inline(always)]
    fn move_back(&mut self, start: usize) {
        if self.len == 0 {
            return;
        }

        unsafe {
            if start >= self.len - 1 {
                *self.get_unchecked_mut(self.len - 1) = Level::bound(self.limit == V::MAX);
                self.len -= 1;
                self.invalidate_cache();
                return;
            }

            // Use ptr::copy for better performance
            ptr::copy(self.buf.as_ptr().add(start + 1), self.buf.as_mut_ptr().add(start), self.len - start - 1);
            *self.get_unchecked_mut(self.len - 1) = Level::bound(self.limit == V::MAX);
            self.len -= 1;

            if start == 0 {
                self.invalidate_cache();
            }
        }
    }

    #[inline(always)]
    pub fn remove(&mut self, index: usize) -> V {
        unsafe {
            let is_reversed = self.limit == V::MAX;
            let level = self.get_unchecked_mut(index);
            let removed = level.price;
            *level = Level::bound(is_reversed);
            self.move_back(index);
            removed
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn first(&self) -> Option<Level<V>> {
        self.cached_first
    }

    #[inline(always)]
    pub fn insert(&mut self, index: usize, level: Level<V>) {
        if index >= N {
            return;
        }

        unsafe {
            match index {
                // Fast path for empty buffer or append
                i if i == self.len => {
                    *self.get_unchecked_mut(self.len) = level;
                    self.len += 1;
                    self.invalidate_cache();
                }
                // Fast path for insert at beginning
                0 => {
                    ptr::copy(self.buf.as_ptr(), self.buf.as_mut_ptr().add(1), self.len);
                    *self.get_unchecked_mut(0) = level;
                    self.len = (self.len + 1).min(N);
                    self.invalidate_cache();
                }
                // Regular insert
                _ => {
                    ptr::copy(self.buf.as_ptr().add(index), self.buf.as_mut_ptr().add(index + 1), self.len - index);
                    *self.get_unchecked_mut(index) = level;
                    self.len = (self.len + 1).min(N);
                    if index == 0 {
                        self.invalidate_cache();
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn modify(&mut self, index: usize, size: V) {
        debug_assert!(index < self.len, "index out of bounds");
        unsafe {
            self.get_unchecked_mut(index).size = size;
            if index == 0 {
                self.invalidate_cache();
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "fixed_decimal")]
mod tests {
    use crate::decimals::fixed_decimal::FixedDecimal;

    use super::*;

    #[test]
    fn test_reversed_vec_basic_operations() {
        let mut buffer = ReversedVec::<10, FixedDecimal>::new(true);

        buffer.insert(0, Level { price: FixedDecimal::from_int(10), size: FixedDecimal::from_int(1) });
        buffer.insert(1, Level { price: FixedDecimal::from_int(90), size: FixedDecimal::from_int(2) });
        buffer.insert(2, Level { price: FixedDecimal::from_int(80), size: FixedDecimal::from_int(3) });
        insta::assert_debug_snapshot!(&buffer);

        let removed = buffer.remove(1);
        assert_eq!(removed, FixedDecimal::from_int(90));
        assert_eq!(buffer.len, 2);
        insta::assert_debug_snapshot!(&buffer);
    }
}
