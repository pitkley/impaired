// Copyright Pit Kleyersburg <pitkley@googlemail.com>
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::fmt::Debug;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    hash::Hash,
    ops::{Deref, DerefMut},
};

/// An item for use in pairwise comparisons.
///
/// ```rust
/// # use impaired::Item;
/// let item = Item("Rust");
/// println!("{}", item);
/// ```
///
/// The underlying item can be accessed by dereferencing `Item`.
///
/// ```
/// # use impaired::Item;
/// let item = Item("Rust");
/// assert_eq!(*item, "Rust");
/// # assert_eq!(item.0, "Rust");
/// # assert_eq!(item.0, *item);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Item<T>(pub T);

impl<T> Deref for Item<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Display> Display for Item<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A type alias for a tuple for two items, representing a comparison.
pub type Comparison<'a, T> = (&'a Item<T>, &'a Item<T>);

/// A list of comparisons.
///
/// This is a thin wrapper around a [`Vec`](std::vec::Vec) of [`Comparison`s](Comparison).
#[derive(Debug, Default)]
pub struct Comparisons<'a, T>(Vec<Comparison<'a, T>>);

impl<'a, T> Comparisons<'a, T> {
    /// Create a new set of comparisons from a list of [`Item`s](Item).
    ///
    /// The comparisons created will be exhaustive across the list of items provided, ensuring that
    /// for each provided item there is exactly one comparison against every other item.
    ///
    /// ```rust
    /// # use impaired::{Comparisons, Item};
    /// let rust = Item("Rust");
    /// let cpp = Item("C++");
    /// let java = Item("Java");
    /// let comparisons = Comparisons::new([&rust, &cpp, &java]);
    /// assert_eq!(comparisons.len(), 3);
    /// assert_eq!(*comparisons, vec![
    ///     (&java, &rust),
    ///     (&java, &cpp),
    ///     (&cpp, &rust),
    /// ]);
    /// ```
    ///
    /// `Comparisons` automatically dereferences into the underlying `Vec` of
    /// [`Comparison`s](Comparison), such that you can interact with the comparisons, e.g. for
    /// iteration:
    ///
    /// ```rust
    /// # use impaired::{Comparisons, Item};
    /// # let rust = Item("Rust");
    /// # let cpp = Item("C++");
    /// # let java = Item("Java");
    /// let comparisons = Comparisons::new([&rust, &cpp, &java]);
    /// for comparison in comparisons.iter() {
    ///     println!("Comparing '{}' against '{}'", comparison.0, comparison.1);
    /// }
    /// ```
    ///
    /// ## Order of comparisons
    ///
    /// Currently there is no guarantee about the order of the items returned. Do not rely on the
    /// order in your implementation.
    ///
    /// If you need to follow a specific order, you can dereference the comparisons into the inner
    /// [`Vec`](std::vec::Vec) of [`Comparison`](Comparison) and ensure that a specific order is
    /// followed that way:
    ///
    /// ```rust
    /// # use impaired::{Comparison, Comparisons, Item};
    /// # let rust = Item("Rust");
    /// # let cpp = Item("C++");
    /// # let java = Item("Java");
    /// let comparisons = Comparisons::new([&rust, &cpp, &java]);
    /// let inner: &Vec<Comparison<&str>> = &*comparisons;
    /// ```
    pub fn new(items: impl IntoIterator<Item = &'a Item<T>>) -> Self {
        let mut comparisons = Vec::new();
        let mut it: Vec<&'a Item<T>> = items.into_iter().collect();
        while let Some(item) = it.pop() {
            for other in &it {
                comparisons.push((item, *other));
            }
        }

        Self(comparisons)
    }
}

impl<'a, T> Deref for Comparisons<'a, T> {
    type Target = Vec<Comparison<'a, T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Track scores for a pairwise-comparison.
///
/// The score of an item is simply the number of times this item was chosen over another item. This
/// allows you to later look at all the items and their scores, sorting them from best-to-worst (or
/// vice versa).
///
/// This is a thin wrapper around a [`HashMap`](std::collections::HashMap), mapping [`Item`s](Item)
/// to a score.
///
/// ## Example
///
/// The following example simulates a fictitious comparison of three programming languages, printing
/// the scores, i.e. the comparison results, from best to worst at the end.
///
/// ```rust
/// # use impaired::{Comparison, Item, Scores};
/// use itertools::Itertools;
///
/// let rust = Item("Rust");
/// let cpp = Item("C++");
/// let java = Item("Java");
///
/// let mut scores = Scores::new();
/// scores.track(&rust, &cpp);
/// scores.track(&rust, &java);
/// scores.track(&java, &cpp);
///
/// for (item, count) in scores.iter().sorted_by(|(_, a), (_, b)| b.cmp(a)) {
///     println!("{} ({}x)", item, count);
/// }
/// ```
///
/// ## Accessing the scores
///
/// `Scores` automatically dereferences into a [`HashMap`](std::collections::HashMap) mapping an
/// [`Item`](Item) to its score (a [`usize`](usize)), allowing you to interact with the results
/// as you require.
///
/// ```rust
/// # use impaired::{Comparison, Item, Scores};
/// # use itertools::Itertools;
/// # let rust = Item("Rust");
/// # let cpp = Item("C++");
/// let mut scores = Scores::new();
/// # scores.track(&rust, &cpp);
///
/// // Access the score for an item directly
/// println!("{}", scores[&rust]);
/// println!("{}", scores[&cpp]);
///
/// // Iterate over the items and their scores
/// for (item, count) in scores.iter().sorted_by(|(_, a), (_, b)| b.cmp(a)) {
///     println!("{} ({}x)", item, count);
/// }
/// ```
#[derive(Debug, Default)]
pub struct Scores<'a, T>(HashMap<&'a Item<T>, usize>);

impl<'a, T> Scores<'a, T>
where
    T: Eq + Hash,
{
    /// Constructs a new, empty set of scores.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Track the result of a single pairwise comparison.
    ///
    /// The winning item's score will be increased by one, the loosing item's score will be kept as
    /// is (although it will be set to zero if it hasn't been tracked yet).
    ///
    /// ```rust
    /// # use impaired::{Comparison, Item, Scores};
    /// let rust = Item("Rust");
    /// let cpp = Item("C++");
    ///
    /// let mut scores = Scores::new();
    /// assert!(scores.get(&rust).is_none());
    /// assert!(scores.get(&cpp).is_none());
    ///
    /// scores.track(&rust, &cpp);
    /// assert_eq!(scores[&rust], 1);
    /// assert_eq!(scores[&cpp], 0);
    /// ```
    pub fn track(&mut self, winner: &'a Item<T>, looser: &'a Item<T>) {
        self.0
            .entry(winner)
            .and_modify(|count| *count += 1)
            .or_insert(1);
        self.0.entry(looser).or_insert(0);
    }
}

impl<'a, T> Deref for Scores<'a, T> {
    type Target = HashMap<&'a Item<T>, usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for Scores<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
