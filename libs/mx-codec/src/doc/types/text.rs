use super::list::ListType;
use crate::{impl_type, Content, MxCodecResult};

impl_type!(Text);

impl ListType for Text {}

impl Text {
    #[inline]
    pub fn len(&self) -> u64 {
        self.content_len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    pub fn insert<T: ToString>(&mut self, char_index: u64, str: T) -> MxCodecResult {
        self.insert_at(char_index, Content::String(str.to_string()))
    }

    #[inline]
    pub fn remove(&mut self, char_index: u64, len: u64) -> MxCodecResult {
        self.remove_at(char_index, len)
    }
}

impl ToString for Text {
    fn to_string(&self) -> String {
        let mut ret = String::with_capacity(self.len() as usize);
        self.iter_item().fold(&mut ret, |ret, item| {
            if let Content::String(str) = &item.get().unwrap().content {
                ret.push_str(str);
            }
            ret
        });
        ret
    }
}

impl serde::Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}
