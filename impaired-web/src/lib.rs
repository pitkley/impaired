// Copyright Pit Kleyersburg <pitkley@googlemail.com>
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use impaired::{Comparisons, RetainItemIterator, Scores};
use ouroboros::self_referencing;
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};
use wasm_bindgen::prelude::*;

fn hash_one<T: Hash>(x: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

pub type ItemHash = u64;

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Clone, Eq, PartialEq, Hash)]
pub struct Item {
    pub hash: ItemHash,
    pub item: String,
}

impl Item {
    fn new(s: String) -> Self {
        Self {
            hash: hash_one(&s),
            item: s,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct Comparison {
    pub left: Item,
    pub right: Item,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Serialize, Clone)]
pub struct Score {
    pub item: Item,
    pub score: u32,
}

#[self_referencing]
struct OngoingComparison {
    items: HashMap<ItemHash, impaired::Item<String>>,
    #[borrows(items)]
    #[covariant]
    comparisons: Comparisons<'this, String>,
    #[borrows(comparisons)]
    #[not_covariant]
    iterator: RetainItemIterator<'this, String>,
    #[borrows()]
    #[covariant]
    scores: Scores<'this, String>,
}

thread_local! {
    static PUSHED_ITEMS: RefCell<Vec<Item>> = RefCell::new(Vec::new());
    static ONGOING_COMPARISON: RefCell<Option<OngoingComparison>> = RefCell::new(None);
}

fn pushed_items<F, R>(action: F) -> R
where
    F: FnOnce(&Vec<Item>) -> R,
{
    PUSHED_ITEMS.with(|pushed_items_rc| action(&pushed_items_rc.borrow()))
}

fn pushed_items_mut<F, R>(action: F) -> R
where
    F: FnOnce(&mut Vec<Item>) -> R,
{
    PUSHED_ITEMS.with(|pushed_items_rc| action(&mut pushed_items_rc.borrow_mut()))
}

fn ongoing_comparison<F, R>(action: F) -> R
where
    F: FnOnce(&Option<OngoingComparison>) -> R,
{
    ONGOING_COMPARISON.with(|ongoing_comparison_rc| action(&ongoing_comparison_rc.borrow()))
}

fn ongoing_comparison_mut<F, R>(action: F) -> R
where
    F: FnOnce(&mut Option<OngoingComparison>) -> R,
{
    ONGOING_COMPARISON.with(|ongoing_comparison_rc| action(&mut ongoing_comparison_rc.borrow_mut()))
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    Ok(())
}

#[wasm_bindgen(js_name = pushItem)]
pub fn push_item(item: String) {
    let item = Item::new(item);
    pushed_items_mut(|pushed_items| pushed_items.push(item));
}

#[wasm_bindgen(js_name = resetComparison)]
pub fn reset_comparison() {
    ongoing_comparison_mut(Option::take);
}

#[wasm_bindgen(js_name = startComparison)]
pub fn start_comparison() {
    ongoing_comparison_mut(|ongoing_comparison| {
        ongoing_comparison.replace(
            OngoingComparisonBuilder {
                items: {
                    let mut map = HashMap::new();
                    pushed_items_mut(|pushed_items| {
                        for item in pushed_items.iter() {
                            map.insert(item.hash, impaired::Item(item.item.clone()));
                        }
                        pushed_items.clear();
                    });
                    map
                },
                comparisons_builder: |items: &HashMap<u64, impaired::Item<String>>| {
                    Comparisons::new(items.values())
                },
                iterator_builder: |comparisons: &Comparisons<String>| {
                    comparisons.retain_item_iterator()
                },
                scores: Scores::new(),
            }
            .build(),
        )
    });
}

#[wasm_bindgen(js_name = hasOngoingComparison)]
pub fn has_ongoing_comparison() -> bool {
    ongoing_comparison(|ongoing_comparison| ongoing_comparison.is_some())
}

#[wasm_bindgen(js_name = nextComparison)]
pub fn next_comparison() -> Option<Comparison> {
    if !has_ongoing_comparison() {
        start_comparison();
    }
    ongoing_comparison_mut(|ongoing_comparison| {
        ongoing_comparison.as_mut().and_then(|ongoing_comparison| {
            ongoing_comparison.with_iterator_mut(|iterator| {
                iterator.next().map(|(comparison, _)| Comparison {
                    left: Item {
                        hash: hash_one(comparison.left),
                        item: comparison.left.0.to_owned(),
                    },
                    right: Item {
                        hash: hash_one(comparison.right),
                        item: comparison.right.0.to_owned(),
                    },
                })
            })
        })
    })
}

#[wasm_bindgen(js_name = trackResult)]
pub fn track_result(winner: Item, loser: Item) {
    ongoing_comparison_mut(|ongoing_comparison| {
        if let Some(ongoing_comparison) = ongoing_comparison.as_mut() {
            ongoing_comparison.with_mut(|fields| {
                if let (Some(winner), Some(loser)) = (
                    fields.items.get(&winner.hash),
                    fields.items.get(&loser.hash),
                ) {
                    fields.iterator.winner(winner);
                    fields.scores.track(winner, loser);
                }
            })
        }
    });
}

#[wasm_bindgen(js_name = getScores)]
pub fn get_scores() -> Result<JsValue, serde_wasm_bindgen::Error> {
    ongoing_comparison(|ongoing_comparison| {
        let mut results = Vec::new();
        if let Some(ongoing_comparison) = ongoing_comparison {
            let scores: &Scores<String> = ongoing_comparison.borrow_scores();
            for (item, score) in scores.iter() {
                results.push(Score {
                    item: Item::new(item.0.clone()),
                    score: *score as u32,
                });
            }
        }

        (&results).serialize(&Serializer::new().serialize_large_number_types_as_bigints(true))
    })
}

#[wasm_bindgen(js_name = getItems)]
pub fn get_items() -> Result<JsValue, serde_wasm_bindgen::Error> {
    if !has_ongoing_comparison() {
        pushed_items(|pushed_items| {
            pushed_items.serialize(&Serializer::new().serialize_large_number_types_as_bigints(true))
        })
    } else {
        ongoing_comparison(|ongoing_comparison| {
            if let Some(ongoing_comparison) = ongoing_comparison {
                let items = ongoing_comparison.borrow_items();
                items
                    .values()
                    .map(|impaired_item| Item::new(impaired_item.0.clone()))
                    .collect::<Vec<_>>()
                    .serialize(&Serializer::new().serialize_large_number_types_as_bigints(true))
            } else {
                serde_wasm_bindgen::to_value(&())
            }
        })
    }
}
