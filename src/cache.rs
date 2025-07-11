pub mod meta;

use crate::mdd::Member;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub static MEMBER_CACHE: Lazy<Arc<RwLock<HashMap<u64, Member>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));
