// Copyright Pit Kleyersburg <pitkley@googlemail.com>
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use getch::Getch;
use impaired::{Comparisons, Item, Scores};
use itertools::Itertools;
use std::{
    env,
    io::{stdout, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items: Vec<Item<String>> = env::args().skip(1).map(Item).collect();
    if items.is_empty() {
        let (example_name, _) = file!()
            .split_once('.')
            .expect("Failed to get example file name");
        eprintln!(
            "USAGE: cargo run --example {} -- item1 item2 ...",
            example_name
        );
        std::process::exit(1);
    }
    let getch = Getch::new();

    let comparisons: Comparisons<_> = Comparisons::new(items.iter());
    let mut scores: Scores<_> = Scores::new();

    for (comparison, result_tracker) in comparisons.retain_item_iterator() {
        println!("A: '{}'  vs.", comparison.left);
        println!("B: '{}'", comparison.right);
        print!("=> Choose by typing 'a' or 'b': ");
        stdout().flush()?;
        loop {
            let char = getch.getch()?;
            match char.to_ascii_lowercase() as char {
                'a' => {
                    scores.track(comparison.left, comparison.right);
                    result_tracker.winner(comparison.left);
                }
                'b' => {
                    scores.track(comparison.right, comparison.left);
                    result_tracker.winner(comparison.right);
                }
                _ => {
                    continue;
                }
            }
            println!("\n");
            break;
        }
    }

    println!("\nFinal scores:");
    for (item, score) in scores.iter().sorted_by(|(_, a), (_, b)| b.cmp(a)) {
        println!("- {}: {} votes", item, score);
    }

    Ok(())
}
