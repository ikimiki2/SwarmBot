/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/27/21, 3:15 PM
 */

use std::collections::HashMap;
use std::hash::Hash;

pub struct PathConstructor;


fn path_trace<T: Copy + Hash + Eq>(from: T, lookup: &HashMap<T, T>, into: &mut Vec<T>) {
    let mut on = from;
    into.push(on);
    while let Some(&prev) = lookup.get(&on) {
        into.push(prev);
        on = prev;
    }
}

impl PathConstructor {
    pub fn build_path<T: Copy + Hash + Eq>(forward: &HashMap<T, T>, backward: &HashMap<T, T>, split: T) -> Vec<T> {
        let mut vec = Vec::new();
        path_trace(split, forward, &mut vec);
        vec.reverse();
        path_trace(split, backward, &mut vec);
        vec
    }

    pub fn build_path_forward<T: Copy + Hash + Eq>(forward: &HashMap<T, T>, goal: T) -> Vec<T> {
        let mut vec = Vec::new();
        path_trace(goal, forward, &mut vec);
        vec.reverse();
        vec
    }
}
