#![allow(non_camel_case_types, non_upper_case_globals)]

use memlink_msdk::prelude::*;

#[memlink_export]
pub fn echo(_ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

#[memlink_export]
pub fn add(_ctx: &CallContext, a: u32, b: u32) -> Result<u32> {
    Ok(a + b)
}

#[memlink_export]
pub fn greet(_ctx: &CallContext, name: String) -> Result<String> {
    Ok(format!("Hello, {}!", name))
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UserData {
    pub id: u32,
    pub name: String,
    pub score: u64,
}

#[memlink_export]
pub fn process_user(_ctx: &CallContext, user: UserData) -> Result<UserData> {
    Ok(UserData {
        id: user.id,
        name: user.name.to_uppercase(),
        score: user.score * 2,
    })
}

fn main() {
    println!("memlink-msdk example");
    println!("Exported functions: echo, add, greet, process_user");
}
