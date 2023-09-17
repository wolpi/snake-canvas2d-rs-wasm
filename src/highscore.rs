use crate::utils::log;
use core::cmp::Ordering;
use std::cmp::Ordering::Greater;
use serde::{Deserialize, Serialize};
use chrono::offset::Local;

use wasm_bindgen::prelude::*;

const STORAGE_KEY: &str = "highscore";
const MAX_ENTRIES: usize = 20;


#[derive(Serialize, Deserialize, Eq, PartialEq, Ord)]
struct HighscoreEntry {
    name: String,
    score: u32,
    mode: String,
    time: String,
}

impl PartialOrd for HighscoreEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.mode == other.mode {
            if self.score == other.score {
                return Some(self.time.cmp(&other.time));
            }
            return Some(other.score.cmp(&self.score));
        }
        return Some(self.mode.cmp(&other.mode));
    }
}


pub fn add_score(name :&str, score :u32, mode :&str) {
    let window = web_sys::window().unwrap();
    let local_storage_opt = window.local_storage().unwrap();
    if local_storage_opt.is_some() {
        let local_storage = local_storage_opt.unwrap();
        let json_result = local_storage.get_item(STORAGE_KEY);
        let mut entries :Vec<HighscoreEntry>;
        if json_result.is_ok() {
            let json_opt = json_result.unwrap();
            if json_opt.is_some() {
                let json = json_opt.unwrap();
                entries = serde_json::from_str(&json).unwrap();
            } else {
                entries = Vec::new();
            }
        } else {
            entries = Vec::new();
        }
        let new_entry = HighscoreEntry {
            name: name.to_string(),
            score: score,
            mode: mode.to_string(),
            time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        entries.push(new_entry);
        entries.sort();
        while entries.len() > MAX_ENTRIES {
            entries.pop();
        }
        let json = serde_json::to_string(&entries).unwrap();
        let result = local_storage.set_item(STORAGE_KEY, &json);
        if result.is_err() {
            log!("could not save highscore to local_storage: {}", result.err().unwrap().as_string().unwrap());
        }
    }
}

pub fn print_highscores() {
    let window = web_sys::window().unwrap();
    let local_storage_opt = window.local_storage().unwrap();
    if local_storage_opt.is_some() {
        let local_storage = local_storage_opt.unwrap();
        let document = window.document().unwrap();
        let table = document.get_element_by_id("highscores-table").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
        while table.child_element_count() > 1 {
            table.last_element_child().unwrap().remove();
        }
        let json_result = local_storage.get_item(STORAGE_KEY);
        let entries :Vec<HighscoreEntry>;
        if json_result.is_ok() {
            let json_opt = json_result.unwrap();
            if json_opt.is_some() {
                let json = json_opt.unwrap();
                entries = serde_json::from_str(&json).unwrap();
                let latest = find_latest_entry(&entries);
                let mut i = 0;
                for entry in entries {
                    let result = print_entry(
                        &document,
                        &table,
                        &entry,
                        i+1,
                        i == latest);
                    if result.is_err() {
                        log!("could not crate highscore table elements: {}", result.err().unwrap().as_string().unwrap());
                        break;
                    }
                    i += 1;
                }
            }
        }
    }
}

fn find_latest_entry(entries :&Vec<HighscoreEntry>) -> u32 {
    let mut latest_idx = 0;
    let mut i = 0;
    let mut latest_entry:&HighscoreEntry = &HighscoreEntry{
        name: "".to_string(), score: 0, mode: "".to_string(), time: "0".to_string()
    };
    for entry in entries {
        if entry.time.cmp(&latest_entry.time) == Greater {
            latest_idx = i;
            latest_entry = entry;
        }
        i += 1;
    }
    return latest_idx;
}
fn print_entry(
        document :&web_sys::Document,
        table :&web_sys::HtmlElement,
        entry :&HighscoreEntry,
        rank :u32,
        highlight :bool)
        -> Result<(), wasm_bindgen::JsValue> {
    let tr = document.create_element("tr").unwrap();
    table.append_child(&tr)?;
    if highlight {
        tr.set_class_name("latest");
    }
    let td_rank = document.create_element("td").unwrap();
    let td_name = document.create_element("td").unwrap();
    let td_score = document.create_element("td").unwrap();
    let td_mode = document.create_element("td").unwrap();
    let td_time = document.create_element("td").unwrap();
    tr.append_child(&td_rank)?;
    tr.append_child(&td_name)?;
    tr.append_child(&td_score)?;
    tr.append_child(&td_mode)?;
    tr.append_child(&td_time)?;
    td_rank.set_text_content(Some(&rank.to_string()));
    td_name.set_text_content(Some(&entry.name));
    td_score.set_text_content(Some(&entry.score.to_string()));
    td_mode.set_text_content(Some(&entry.mode));
    td_time.set_text_content(Some(&entry.time));
    Ok(())
}
