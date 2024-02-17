use std::{
    cell::RefCell,
    cmp::max,
    collections::VecDeque,
    ops::{Deref, DerefMut},
};
use super::*;

const MAX_SEARCH_MARKER: usize = 80;

#[derive(Clone, Debug)]
pub(crate) struct SearchMarker {
    pub ptr: Somr<Item>,
    pub index: u64,
}

impl SearchMarker {
    fn new(ptr: Somr<Item>, index: u64) -> Self {
        SearchMarker { ptr, index }
    }
    fn overwrite_marker(&mut self, ptr: Somr<Item>, index: u64) {
        self.ptr = ptr;
        self.index = index;
    }
}

unsafe impl Sync for MarkerList {}

/// in yjs, a timestamp field is used to sort markers and the oldest marker is
/// deleted once the limit is reached. this was designed for optimization
/// purposes for v8. In Rust, we can simply use a [VecDeque] and trust the
/// compiler to optimize. the [VecDeque] can naturally maintain the insertion
/// order, allowing us to know which marker is the oldest without using an extra
/// timestamp field.
///
/// NOTE:
/// A [MarkerList] is always belonging to a [YType],
/// which means whenever [MakerList] is used, we actually have a [YType]
/// instance behind [RwLock] guard already, so it's safe to make the list
/// internal mutable.
#[derive(Debug)]
pub(crate) struct MarkerList(RefCell<VecDeque<SearchMarker>>);

impl Deref for MarkerList {
    type Target = RefCell<VecDeque<SearchMarker>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MarkerList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for MarkerList {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkerList {
    pub fn new() -> Self {
        MarkerList(RefCell::new(VecDeque::new()))
    }
    // mark pos and push to the end of linked list
    fn mark_position(list: &mut VecDeque<SearchMarker>, ptr: Somr<Item>, index: u64) -> Option<SearchMarker> {
        if list.len() >= MAX_SEARCH_MARKER {
            let mut oldest_marker = list.pop_front().unwrap();
            oldest_marker.overwrite_marker(ptr, index);
            list.push_back(oldest_marker);
        } else {
            let marker = SearchMarker::new(ptr, index);
            list.push_back(marker);
        }
        list.back().cloned()
    }
    // update mark position if the index is within the range of the marker
    pub fn update_marker_changes(&self, index: u64, len: i64) {
        let mut list = self.borrow_mut();
        for marker in list.iter_mut() {
            if len > 0 {
                while let Some(ptr) = marker.ptr.get() {
                    if !ptr.indexable() {
                        let left_ref = ptr.left.clone();
                        if let Some(left) = left_ref.get() {
                            if left.indexable() {
                                marker.index -= left.len();
                            }
                            marker.ptr = left_ref;
                        } else {
                            // remove marker
                            marker.index = 0;
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            if marker.ptr.is_some() && (index < marker.index || (len > 0 && index == marker.index)) {
                marker.index = max(index as i64, marker.index as i64 + len) as u64;
            }
        }
        list.retain(|marker| marker.index > 0);
    }

    // find and return the marker that is closest to the index
    pub fn find_marker(&self, parent: &YType, index: u64) -> Option<SearchMarker> {
        if parent.start.is_none() || index == 0 {
            return None;
        }
        let mut list = self.borrow_mut();
        let marker = list.iter_mut().min_by_key(|m| (index as i64 - m.index as i64).abs());
        let mut marker_index = marker.as_ref().map(|m| m.index).unwrap_or(0);
        let mut item_ptr = marker
            .as_ref()
            .map(|m| m.ptr.clone())
            .unwrap_or_else(|| parent.start.clone());
        // TODO: this logic here is a bit messy
        // i think it can be implemented with more streamlined code, and then optimized
        {
            // iterate to the right if possible
            while let Some(item) = item_ptr.clone().get() {
                if marker_index >= index {
                    break;
                }
                let right_ref: ItemRef = item.right.clone();
                if right_ref.is_some() {
                    if item.indexable() {
                        if index < marker_index + item.len() {
                            break;
                        }

                        marker_index += item.len();
                    }
                    item_ptr = right_ref;
                } else {
                    break;
                }
            }
            // iterate to the left if necessary (might be that marker_index > index)
            while let Some(item) = item_ptr.clone().get() {
                if marker_index <= index {
                    break;
                }
                let left_ref: ItemRef = item.left.clone();
                if let Some(left) = left_ref.get() {
                    if left.indexable() {
                        marker_index -= left.len();
                    }
                    item_ptr = left_ref;
                } else {
                    break;
                }
            }
            // we want to make sure that item_ptr can't be merged with left, because that
            // would screw up everything in that case just return what we have
            // (it is most likely the best marker anyway) iterate to left until
            // item_ptr can't be merged with left
            while let Some(item) = item_ptr.clone().get() {
                let left_ref: ItemRef = item.left.clone();
                if let Some(left) = left_ref.get() {
                    if left.id.client == item.id.client && left.id.clock + left.len() == item.id.clock {
                        if left.indexable() {
                            marker_index -= left.len();
                        }
                        item_ptr = left_ref;
                        continue;
                    }
                    break;
                } else {
                    break;
                }
            }
        }
        match marker {
            Some(marker)
                if (marker.index as f64 - marker_index as f64).abs() < parent.len as f64 / MAX_SEARCH_MARKER as f64 =>
            {
                // adjust existing marker
                marker.overwrite_marker(item_ptr, marker_index);
                Some(marker.clone())
            }
            _ => {
                // create new marker
                Self::mark_position(&mut list, item_ptr, marker_index)
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_last_marker(&self) -> Option<SearchMarker> {
        self.borrow().back().cloned()
    }
    pub fn replace_marker(&self, raw: Somr<Item>, new: Somr<Item>, len_shift: i64) {
        let mut list = self.borrow_mut();

        for marker in list.iter_mut() {
            if marker.ptr == raw {
                marker.ptr = new.clone();
                marker.index = ((marker.index as i64) + len_shift) as u64;
            }
        }
    }
}
