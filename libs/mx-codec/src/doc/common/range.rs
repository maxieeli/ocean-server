use std::{collections::VecDeque, mem, ops::Range};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OrderRange {
    Range(Range<u64>),
    Fragment(VecDeque<Range<u64>>)
}

impl Default for OrderRange {
    fn default() -> Self {
        Self::Range(0..0)
    }
}

impl From<Range<u64>> for OrderRange {
    fn from(range: Range<u64>) -> Self {
        Self::Range(range)
    }
}

impl From<Vec<Range<u64>>> for OrderRange {
    fn from(value: Vec<Range<u64>>) -> Self {
        Self::Fragment(value.into_iter().collect())
    }
}

impl From<VecDeque<Range<u64>>> for OrderRange {
    fn from(value: VecDeque<Range<u64>>) -> Self {
        Self::Fragment(value)
    }
}

#[inline]
fn is_continuous_range(lhs: &Range<u64>, rhs: &Range<u64>) -> bool {
    lhs.end >= rhs.start && lhs.start <= rhs.end
}

impl OrderRange {
    pub fn ranges_len(&self) -> usize {
        match self {
            OrderRange::Range(_) => 1,
            OrderRange::Fragment(ranges) => ranges.len()
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            OrderRange::Range(range) => range.is_empty(),
            OrderRange::Fragment(vec) => vec.is_empty(),
        }
    }
    pub fn contains(&self, clock: u64) -> bool {
        match self {
            OrderRange::Range(range) => range.contains(&clock),
            OrderRange::Fragment(ranges) => ranges.iter().any(|r| r.contains(&clock)),
        }
    }
    fn check_range_covered(old_vec: &[Range<u64>], new_vec: &[Range<u64>]) -> bool {
        let mut old_iter = old_vec.iter();
        let mut next_old = old_iter.next();
        let mut new_iter = new_vec.iter().peekable();
        let mut next_new = new_iter.next();
        'new_loop: while let Some(new_range) = next_new {
            while let Some(old_range) = next_old {
                if old_range.start < new_range.start || old_range.end > new_range.end {
                    if new_iter.peek().is_some() {
                        next_new = new_iter.next();
                        continue 'new_loop;
                    } else {
                        return false
                    }
                }
                next_old = old_iter.next();
                if let Some(next_old) = &next_old {
                    if next_old.start > new_range.end {
                        continue;
                    }
                }
            }
            next_new = new_iter.next()
        }
        true
    }

    // diff_range returns the difference between the old range and the new range
    // current range must be covered by the new range
    pub fn diff_range(&self, new_range: &OrderRange) -> Vec<Range<u64>> {
        let old_vec = self.clone().into_iter().collect::<Vec<_>>();
        let new_vec = new_range.clone().into_iter().collect::<Vec<_>>();
        if !Self::check_range_covered(&old_vec, &new_vec) {
            return Vec::new();
        }
        let mut diffs = Vec::new();
        let mut old_idx = 0;
        for new_range in &new_vec {
            let mut overlap_ranges = Vec::new();
            while old_idx < old_vec.len() && old_vec[old_idx].start <= new_range.end {
                overlap_ranges.push(old_vec[old_idx].clone());
                old_idx += 1;
            }
            if overlap_ranges.is_empty() {
                diffs.push(new_range.clone())
            } else {
                let mut last_end = overlap_ranges[0].start;
                if last_end > new_range.start {
                    diffs.push(new_range.start..last_end);
                }
                for overlap in &overlap_ranges {
                    if overlap.start > last_end {
                        diffs.push(last_end..overlap.start);
                    }
                    last_end = overlap.end;
                }
                if new_range.end > last_end {
                    diffs.push(last_end..new_range.end);
                }
            }
        }
        diffs
    }
    /// Push new range to current one.
    /// Range will be merged if overlap exists or turned into fragment if it's
    /// not continuous.
    pub fn push(&mut self, range: Range<u64>) {
        match self {
            OrderRange::Range(r) => {
                if r.start == r.end {
                    *self = range.into()
                } else if is_continuous_range(r, &range) {
                    r.end = r.end.max(range.end)
                    r.start = r.start.min(range.start)
                } else {
                    *self = OrderRange::Fragment(if r.start < range.start {
                        VecDeque::from([r.clone(), range])
                    } else {
                        VecDeque::from([range, r.clone()])
                    })
                }
            }
            OrderRange::Fragment(ranges) => {
                if ranges.is_empty() {
                    *self = OrderRange::Range(range)
                } else {
                    OrderRange::push_inner(ranges, range);
                    self.make_single();
                }
            }
        }
    }

    pub fn pop(&mut self) -> Option<Range<u64>> {
        if self.is_empty() {
            None
        } else {
            match self {
                OrderRange::Range(range) => Some(mem::replace(range, 0..0)),
                OrderRange::Fragment(list) => list.pop_front(),
            }
        }
    }
    pub fn merge(&mut self, other: Self) {
        self.extend(&other);
    }
    fn make_fragment(&mut self) {
        if let OrderRange::Range(range) = self {
            *self = OrderRange::Fragment(if range.is_empty() {
                VecDeque::new()
            } else {
                VecDeque::from([range.clone()])
            });
        }
    }
    fn make_single(&mut self) {
        if let OrderRange::Fragment(ranges) = self {
            if ranges.len() == 1 {
                *self = OrderRange::Range(ranges[0].clone());
            }
        }
    }
    // merge all available ranges list into one
    pub fn squash(&mut self) {
        // merge all available ranges
        if let OrderRange::Fragment(ranges) = self {
            if ranges.is_empty() {
                *self = OrderRange::Range(0..0);
                return
            }
            let mut changed = false;
            let mut merged = VecDeque::with_capacity(ranges.len());
            let mut cur = ranges[0].clone();
            for next in ranges.iter().skip(1) {
                if is_continuous_range(&cur, next) {
                    cur.start = cur.start.min(next.start);
                    cur.end = cur.end.max(next.end);
                    changed = true;
                } else {
                    merged.push_back(cur);
                    cur = next.clone();
                }
            }
            merged.push_back(cur);
            if merged.len() == 1 {
                *self = OrderRange::Range(merged[0].clone());
            } else if changed {
                mem::swap(ranges, &mut merged);
            }
        }
    }

    fn push_inner(list: &mut VecDeque<Range<u64>>, range: Range<u64>) {
        if list.is_empty() {
            list.push_back(range);
        } else {
            let search_result = list.binary_search_by(|r| {
                if is_continuous_range(r, &range) {
                    std::cmp::Ordering::Equal
                } else if r.end < range.start {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
            match search_result {
                Ok(idx) => {
                    let old = &mut list[idx];
                    list[idx] = old.start.min(range.start)..old.end.max(range.end);
                    Self::squash_arround(list, idx);
                }
                Err(idx) => {
                    list.insert(idx, range);
                    Self::squash_around(list, idx);
                }
            }
        }
    }
    fn squash_around(list: &mut VecDeque<Range<u64>>, idx: usize) {
        if idx > 0 {
            let prev = &list[idx - 1];
            let cur = &list[idx];
            if is_continuous_range(prev, cur) {
                list[idx - 1] = prev.start.min(cur.start)..prev.end.max(cur.end);
                list.remove(idx);
            }
        }
        if idx < list.len() - 1 {
            let next = &list[idx + 1];
            let cur = &list[idx];
            if is_continuous_range(cur, next) {
                list[idx] = cur.start.min(next.start)..cur.end.max(next.end);
                list.remove(idx + 1);
            }
        }
    }
}

impl<'a> IntoInterator for &'a OrderRange {
    type Item = Range<u64>;
    type IntoIter = OrderRangeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        OrderRangeIter { range: self, idx: 0 }
    }
}

impl Extend<Range<u64>> for OrderRange {
    fn extend<T: IntoIterator<Item = Range<u64>>>(&mut self, other: T) {
        self.make_fragment();
        match self {
            OrderRange::Fragment(ranges) => {
                for range in other {
                    OrderRange::push_inner(ranges, range);
                }
                self.make_single();
            }
            _ => unreachable!(),
        }
    }
}

pub struct OrderRangeIter<'a> {
    range: &'a OrderRange,
    idx: usize,
}

impl<'a> Iterator for OrderRangeIter<'a> {
    type Item = Range<u64>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.range {
            OrderRange::Range(range) => {
                if self.idx == 0 {
                    self.idx += 1;
                    Some(range.clone())
                } else {
                    None
                }
            }
            OrderRange::Fragment(ranges) => {
                if self.idx < ranges.len() {
                    let range = ranges[self.idx].clone();
                    self.idx += 1;
                    Some(range)
                } else {
                    None
                }
            }
        }
    }
}
