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
#[derive(Clone)]
pub struct Item {
    pub hash: ItemHash,
    pub item: String,
}

#[wasm_bindgen(getter_with_clone)]
pub struct Comparison {
    pub left: Item,
    pub right: Item,
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
    static ONGOING_COMPARISON: RefCell<Option<OngoingComparison>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let rust = impaired::Item("Rust".to_owned());
    let cpp = impaired::Item("C++".to_owned());
    let java = impaired::Item("Java".to_owned());
    ONGOING_COMPARISON.with(|comparison| {
        comparison.borrow_mut().replace(
            OngoingComparisonBuilder {
                items: {
                    let mut map = HashMap::new();
                    map.insert(hash_one(&rust), rust);
                    map.insert(hash_one(&cpp), cpp);
                    map.insert(hash_one(&java), java);
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
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(js_name = nextComparison)]
pub fn next_comparison() -> Option<Comparison> {
    ONGOING_COMPARISON.with(|ongoing_comparison_rc| {
        ongoing_comparison_rc
            .borrow_mut()
            .as_mut()
            .and_then(|ongoing_comparison| {
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
    ONGOING_COMPARISON.with(|ongoing_comparison_rc| {
        if let Some(ongoing_comparison) = ongoing_comparison_rc.borrow_mut().as_mut() {
            ongoing_comparison.with_mut(|fields| {
                match (
                    fields.items.get(&winner.hash),
                    fields.items.get(&loser.hash),
                ) {
                    (Some(winner), Some(loser)) => {
                        log(&format!(
                            "Tracking result for winner={winner}, loser={loser}"
                        ));
                        fields.iterator.winner(winner);
                        fields.scores.track(winner, loser);
                    }
                    _ => {
                        log("Did not find one of the provided items, can't track result.");
                    }
                };
            })
        }
    });
}

#[wasm_bindgen(js_name = printScores)]
pub fn print_scores() {
    ONGOING_COMPARISON.with(|ongoing_comparison_rc| {
        if let Some(ongoing_comparison) = ongoing_comparison_rc.borrow_mut().as_mut() {
            let scores: &Scores<String> = ongoing_comparison.borrow_scores();
            for (item, score) in scores.iter() {
                log(&format!("- {} ({} points)", item, score));
            }
        }
    })
}
