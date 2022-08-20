use colored::Colorize;
use slotmap::{DefaultKey, SlotMap};
use std::{
    env,
    fmt::Display,
    mem::size_of,
    ops::{Deref, DerefMut},
};

use crate::{exceptions::Exception, interpreter::type_hint::Type, parser::data::Data};

const SIZE_OF_DATA: usize = size_of::<Data>();
pub struct Variables {
    map: SlotMap<DefaultKey, Variable>,
    size: usize,
    used: usize,
}

impl Variables {
    pub(crate) fn new() -> Self {
        let size = string_to_bytes(
            env::var("CRYSCRIPT_VAR_MEMORY").unwrap_or_else(|_| "MB1024".to_string()),
        );
        Self {
            map: SlotMap::with_capacity(size / SIZE_OF_DATA + 1),
            size,
            used: 0,
        }
    }
    pub(crate) fn access_variable(&mut self, key: DefaultKey) -> VariableReference {
        VariableReference::new(self.map.get_mut(key).unwrap())
    }
    pub(crate) fn declare_variable(&mut self, var: Variable) -> DefaultKey {
        self.used += SIZE_OF_DATA;
        if self.used > (self.size as f32 * 0.8) as usize {
            println!("{} {}", self.used, (self.size as f32 * 0.8));
            self.garbage_collector()
        } else if self.used > self.size {
            panic!("too much memory")
        }
        self.map.insert(var)
    }

    pub(crate) fn update_variable(&mut self, key: DefaultKey, data: Data) {
        self.map.get_mut(key).unwrap().data = data
    }

    pub(crate) fn remove_variable(&mut self, key: DefaultKey) {
        self.used -= SIZE_OF_DATA;
        drop(self.map.remove(key).unwrap());
    }

    pub(crate) fn garbage_collector(&mut self) {
        let mut v = vec![];
        for (key, var) in self.map.iter() {
            if var.used_places == 0 {
                v.push(key)
            }
        }
        for i in v {
            self.remove_variable(i)
        }
        if self.used > self.size {
            Exception::new(
                "Err: Memory overflow, consider increasing maximum memory of the program"
                    .red()
                    .bold()
                    .to_string(),
            )
            .run()
        }
    }
}

#[derive(Debug)]
pub(crate) struct VariableReference {
    pub(crate) reference: *mut Variable,
}

impl VariableReference {
    pub(crate) fn new(reference: *mut Variable) -> Self {
        let mut v = Self { reference };
        (*v).used_places += 1;
        v
    }
}

impl Deref for VariableReference {
    type Target = Variable;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.reference }
    }
}

impl DerefMut for VariableReference {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.reference }
    }
}

impl Drop for VariableReference {
    fn drop(&mut self) {
        (*self).used_places -= 1
    }
}

impl Clone for VariableReference {
    fn clone(&self) -> Self {
        Self::new(self.reference)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub(crate) struct VariableKey(pub String);

#[derive(Debug, Clone)]
pub(crate) struct Variable {
    pub(crate) data: Data,
    pub(crate) type_hint: Type,
    pub(crate) is_final: bool,
    pub(crate) identifier: String,
    pub(crate) used_places: usize,
}

impl Variable {
    pub(crate) fn new(data: Data, type_hint: Type, is_final: bool, identifier: String) -> Self {
        Self {
            data,
            type_hint,
            is_final,
            used_places: 0,
            identifier,
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "data: {} type: {} final: {}",
            self.data.data_type, self.type_hint.type_value, self.is_final
        )
    }
}

fn string_to_bytes(string: String) -> usize {
    let split = string.split_at(2);
    let prefix = split.0;
    let number = split.1.parse::<usize>().unwrap();
    number
        * match prefix {
            "BT" => 1,
            "KB" => 1_000,
            "MB" => 1_000_000,
            "GB" => 1_000_000_000,
            "TB" => 1_000_000_000_000,
            "PB" => 1_000_000_000_000_000,
            "EB" => 1_000_000_000_000_000_000,
            _ => panic!("invalid memory type"),
        }
}
