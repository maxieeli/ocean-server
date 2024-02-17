use super::*;

impl_type!(Array);

impl ListType for Array {}
pub struct ArrayIter<'a>(ListIterator<'a>);

impl Iterator for ArrayIter<'_> {
    type Item = Value;
    fn next(&mut self) -> Option<Self::Item> {
        for item in self.0.by_ref() {
            if let Some(item) = item.get() {
                if item.countable() {
                    return Some(Value::from(&item.content));
                }
            }
        }
        None
    }
}

impl Array {
    #[inline]
    pub fn len(&self) -> u64 {
        self.content_len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: u64) -> Option<Value> {
        let (item, offset) = self.get_item_at(index)?;
        if let Some(item) = item.get() {
            return match &item.content {
                Content::Any(any) => return any.get(offset as usize).map(|any| Value::Any(any.clone())),
                _ => Some(Value::from(&item.content)),
            };
        }
        None
    }
    pub fn iter(&self) -> ArrayIter {
        ArrayIter(self.iter_item())
    }
    pub fn push<V: Into<Value>>(&mut self, val: V) -> JwstCodecResult {
        self.insert(self.len(), val)
    }
    pub fn insert<V: Into<Value>>(&mut self, idx: u64, val: V) -> JwstCodecResult {
        self.insert_at(idx, val.into().into())
    }
    pub fn remove(&mut self, idx: u64, len: u64) -> JwstCodecResult {
        self.remove_at(idx, len)
    }
}

impl serde::Serialize for Array {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.len() as usize))?;
        for item in self.iter() {
            seq.serialize_element(&item)?;
        }
        seq.end()
    }
}
