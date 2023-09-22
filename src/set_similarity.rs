use std::collections::HashSet;

pub struct SetSimilarity {
    pub less: i32,
    pub more: i32,
}

pub fn compare_sets(
    set1: &HashSet<String>,
    set2: &HashSet<String>,
    less_score: i32,
    more_score: i32,
) -> SetSimilarity {
    let mut more = 0;
    let mut less = 0;
    for item in set1 {
        if set2.contains(item) {
            more += more_score;
        } else {
            less += less_score;
        }
    }
    SetSimilarity { less, more }
}
