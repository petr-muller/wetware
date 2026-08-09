//! Fuzzy matching helper shared by the TUI's entity picker and the composer's
//! entity whisperer.
//!
//! Wraps `nucleo-matcher` so both call sites score candidates identically.

use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
};

/// Score every haystack against `query`, keeping only the ones that match.
///
/// Results are ordered by descending score; equal scores keep their original
/// relative order, so the output is stable for a given input. An empty query
/// matches everything, in the original order and with a score of 0.
pub fn score_all(query: &str, haystacks: &[&str]) -> Vec<(usize, u32)> {
    if query.is_empty() {
        return (0..haystacks.len()).map(|i| (i, 0)).collect();
    }

    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(query, CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy);

    let mut scored: Vec<(usize, u32)> = haystacks
        .iter()
        .enumerate()
        .filter_map(|(i, haystack)| {
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(haystack, &mut buf);
            pattern.score(haystack, &mut matcher).map(|score| (i, score))
        })
        .collect();

    // Stable sort so equal scores keep input order, making results deterministic.
    scored.sort_by_key(|&(_, score)| std::cmp::Reverse(score));
    scored
}

/// Score every haystack and return just the matching indices, best first.
pub fn match_indices(query: &str, haystacks: &[&str]) -> Vec<usize> {
    score_all(query, haystacks).into_iter().map(|(i, _)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTITIES: [&str; 4] = ["Alice", "Alicia", "Bob", "Machine Learning"];

    #[test]
    fn test_empty_query_matches_everything_in_order() {
        let matches = match_indices("", &ENTITIES);
        assert_eq!(matches, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_query_filters_out_non_matching_candidates() {
        let matches = match_indices("ali", &ENTITIES);

        assert!(matches.contains(&0), "Alice");
        assert!(matches.contains(&1), "Alicia");
        assert!(!matches.contains(&2), "Bob has no 'a'");
        // Note "Machine Learning" *does* match: a-l-i appear in that order.
        assert!(matches.contains(&3));
    }

    #[test]
    fn test_exact_prefix_outranks_looser_match() {
        let matches = match_indices("alice", &ENTITIES);
        assert_eq!(matches.first(), Some(&0));
    }

    #[test]
    fn test_matching_is_case_insensitive() {
        assert_eq!(match_indices("BOB", &ENTITIES), vec![2]);
    }

    #[test]
    fn test_non_contiguous_subsequence_matches() {
        // Fuzzy, not substring: "ml" matches "Machine Learning".
        assert_eq!(match_indices("ml", &ENTITIES), vec![3]);
    }

    #[test]
    fn test_no_match_returns_empty() {
        assert!(match_indices("zzzz", &ENTITIES).is_empty());
    }

    #[test]
    fn test_equal_scores_keep_input_order() {
        let haystacks = ["dup", "dup", "dup"];
        assert_eq!(match_indices("dup", &haystacks), vec![0, 1, 2]);
    }

    #[test]
    fn test_score_all_reports_descending_scores() {
        let scored = score_all("ali", &ENTITIES);
        assert!(scored.windows(2).all(|w| w[0].1 >= w[1].1));
    }
}
