//! Word wrapping for the composer's content field.
//!
//! `tui_input::Input` holds a single logical line with no newlines, but a thought
//! easily runs past the right edge. This module lays that line out over several
//! visual rows and reports where the cursor lands, so rendering and cursor
//! placement are derived from one computation and cannot disagree.
//!
//! Widths are counted in display columns (via `unicode-width`), not characters, so
//! wide glyphs do not overflow the field and get clipped.

use unicode_width::UnicodeWidthChar;

/// A single visual row: a half-open range of character indices into the content.
///
/// Rows tile the content exactly — every character belongs to exactly one row —
/// which is what makes mapping a cursor index to a row unambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// First character index on this row
    pub start: usize,
    /// One past the last character index on this row
    pub end: usize,
}

/// Content laid out over visual rows, with the cursor located within them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapped {
    /// Visual rows, always at least one (empty content yields one empty row)
    pub rows: Vec<Row>,
    /// Index into `rows` of the row holding the cursor
    pub cursor_row: usize,
    /// Display-column offset of the cursor within `cursor_row`
    pub cursor_col: usize,
}

/// Display width of a character, treating control characters as zero-width.
fn char_width(c: char) -> usize {
    c.width().unwrap_or(0)
}

/// Lay `text` out over rows no wider than `width` display columns, and locate
/// `cursor` (a character index) within them.
///
/// Breaks at spaces where possible, keeping the space at the end of the row it
/// breaks after so the rows stay contiguous. A word longer than `width` is broken
/// hard rather than overflowing.
pub fn wrap(text: &str, cursor: usize, width: usize) -> Wrapped {
    let chars: Vec<char> = text.chars().collect();
    let width = width.max(1);
    let mut rows = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let mut end = start;
        let mut used = 0;
        // Last index at which we saw a space, as a preferred break point.
        let mut last_space: Option<usize> = None;

        while end < chars.len() {
            let w = char_width(chars[end]);
            if used + w > width {
                break;
            }
            used += w;
            if chars[end] == ' ' {
                last_space = Some(end);
            }
            end += 1;
        }

        if end == chars.len() {
            rows.push(Row { start, end });
            break;
        }

        // Overflowed. Prefer breaking after the last space on this row; if the row
        // is one long word, break it hard at the column limit.
        let break_at = match last_space {
            Some(space) if space + 1 > start => space + 1,
            _ => end.max(start + 1),
        };
        rows.push(Row { start, end: break_at });
        start = break_at;
    }

    // An empty field still needs a row to put the cursor on. So does a full last
    // row when the cursor sits past its end — there is nowhere else to draw it.
    // Only then, so text that merely happens to end on a boundary gains no
    // spurious blank row.
    let cursor_needs_a_new_row = cursor >= chars.len() && rows.last().is_some_and(|r| row_is_full(&chars, *r, width));
    if rows.is_empty() || cursor_needs_a_new_row {
        rows.push(Row {
            start: chars.len(),
            end: chars.len(),
        });
    }

    let (cursor_row, cursor_col) = locate_cursor(&chars, &rows, cursor);
    Wrapped {
        rows,
        cursor_row,
        cursor_col,
    }
}

/// Whether a row exactly fills the available width.
fn row_is_full(chars: &[char], row: Row, width: usize) -> bool {
    chars[row.start..row.end].iter().copied().map(char_width).sum::<usize>() >= width
}

/// Map a character index to a row and a display-column offset within it.
fn locate_cursor(chars: &[char], rows: &[Row], cursor: usize) -> (usize, usize) {
    let cursor = cursor.min(chars.len());
    for (i, row) in rows.iter().enumerate() {
        // The final row owns the position one past the last character.
        let owns_end = i == rows.len() - 1;
        if (cursor >= row.start && cursor < row.end) || (owns_end && cursor >= row.start) {
            let col = chars[row.start..cursor.min(row.end.max(row.start))]
                .iter()
                .copied()
                .map(char_width)
                .sum();
            return (i, col);
        }
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The text of each row, for readable assertions.
    fn row_texts(text: &str, wrapped: &Wrapped) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        wrapped
            .rows
            .iter()
            .map(|r| chars[r.start..r.end].iter().collect())
            .collect()
    }

    #[test]
    fn test_short_text_stays_on_one_row() {
        let wrapped = wrap("hello", 5, 20);
        assert_eq!(row_texts("hello", &wrapped), vec!["hello"]);
        assert_eq!((wrapped.cursor_row, wrapped.cursor_col), (0, 5));
    }

    #[test]
    fn test_empty_text_still_has_a_row_for_the_cursor() {
        let wrapped = wrap("", 0, 20);
        assert_eq!(wrapped.rows.len(), 1);
        assert_eq!((wrapped.cursor_row, wrapped.cursor_col), (0, 0));
    }

    #[test]
    fn test_breaks_at_a_space_not_mid_word() {
        let text = "the quick brown fox";
        let wrapped = wrap(text, 0, 10);
        // "the quick " fits in 10; "brown" would not.
        assert_eq!(row_texts(text, &wrapped), vec!["the quick ", "brown fox"]);
    }

    #[test]
    fn test_rows_tile_the_text_exactly() {
        let text = "alpha beta gamma delta epsilon zeta";
        let wrapped = wrap(text, 0, 12);

        assert_eq!(wrapped.rows[0].start, 0);
        for pair in wrapped.rows.windows(2) {
            assert_eq!(pair[0].end, pair[1].start, "rows must be contiguous");
        }
        assert_eq!(row_texts(text, &wrapped).concat(), text);
    }

    #[test]
    fn test_word_longer_than_the_width_is_broken_hard() {
        let text = "supercalifragilistic";
        let wrapped = wrap(text, 0, 8);
        assert_eq!(row_texts(text, &wrapped), vec!["supercal", "ifragili", "stic"]);
    }

    #[test]
    fn test_cursor_follows_onto_the_wrapped_row() {
        let text = "the quick brown fox";
        // Cursor at the very end.
        let wrapped = wrap(text, text.chars().count(), 10);
        assert_eq!(wrapped.cursor_row, 1);
        assert_eq!(wrapped.cursor_col, "brown fox".len());
    }

    #[test]
    fn test_cursor_at_a_row_boundary_belongs_to_the_later_row() {
        let text = "the quick brown fox";
        // Index 10 is the first char of "brown", i.e. the start of row 1.
        let wrapped = wrap(text, 10, 10);
        assert_eq!((wrapped.cursor_row, wrapped.cursor_col), (1, 0));
    }

    #[test]
    fn test_text_ending_exactly_at_the_boundary_gets_a_row_for_the_cursor() {
        let text = "abcdefghij"; // exactly 10
        let wrapped = wrap(text, 10, 10);
        assert_eq!(wrapped.rows.len(), 2, "cursor needs somewhere to sit");
        assert_eq!((wrapped.cursor_row, wrapped.cursor_col), (1, 0));
    }

    #[test]
    fn test_wide_glyphs_wrap_by_display_width() {
        // Each CJK glyph is two columns wide, so only three fit in a width of 7.
        let text = "日本語日本語";
        let wrapped = wrap(text, 0, 7);
        assert_eq!(row_texts(text, &wrapped), vec!["日本語", "日本語"]);
    }

    #[test]
    fn test_cursor_column_counts_display_width() {
        let text = "日本語";
        let wrapped = wrap(text, 2, 10);
        assert_eq!(wrapped.cursor_col, 4, "two wide glyphs are four columns");
    }

    #[test]
    fn test_zero_width_is_treated_as_one_column() {
        // Guards against a division-free infinite loop on a degenerate area.
        let wrapped = wrap("abc", 0, 0);
        assert_eq!(wrapped.rows.len(), 3);
    }

    #[test]
    fn test_cursor_past_the_end_is_clamped() {
        let wrapped = wrap("abc", 99, 10);
        assert_eq!((wrapped.cursor_row, wrapped.cursor_col), (0, 3));
    }

    #[test]
    fn test_run_of_spaces_does_not_lose_characters() {
        let text = "a     b     c";
        let wrapped = wrap(text, 0, 6);
        assert_eq!(row_texts(text, &wrapped).concat(), text);
    }
}
