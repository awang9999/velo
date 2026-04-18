// Buffer data model
// Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 2.10

use crate::mode::{MajorMode, MinorMode, PlainTextMode};
use ropey::Rope;
use std::path::PathBuf;
use tree_sitter::Parser;
use velo_types::{Position, Selection, VeloError};

/// Buffer is the in‑memory representation of an open file or unsaved document,
/// backed by ropey::Rope for efficient text manipulation.
pub struct Buffer {
    /// The actual text content
    pub rope: Rope,
    /// None for unsaved buffers
    pub file_path: Option<PathBuf>,
    /// Modified since last save
    pub is_dirty: bool,
    /// Primary cursor location
    pub cursor: Position,
    /// Multi‑cursor / selection ranges
    pub selections: Vec<Selection>,
    /// Active major mode for this buffer
    pub major_mode: Box<dyn MajorMode>,
    /// Active minor modes for this buffer
    pub minor_modes: Vec<Box<dyn MinorMode>>,
    /// Incremental tree‑sitter parse tree
    pub syntax_tree: Option<tree_sitter::Tree>,
    /// Parser used for incremental parsing
    pub parser: tree_sitter::Parser,
}

impl Buffer {
    /// Update the syntax tree based on current buffer content and major mode grammar.
    fn update_syntax_tree(&mut self) {
        if let Some(_lang) = self.major_mode.grammar() {
            // `Parser::parse` takes `AsRef<[u8]>` for the text and an `Option<&Tree>`
            // for the previous parse tree.  We re‑parse the whole buffer (no incremental
            // diff) whenever the buffer changes.
            let source = self.rope.to_string(); // own the string – it satisfies `AsRef<[u8]>`
            self.syntax_tree = self.parser.parse(source, None);
        } else {
            self.syntax_tree = None;
        }
    }

    /// Create a new empty buffer with plain‑text mode
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            is_dirty: false,
            cursor: Position::new(0, 0),
            selections: Vec::new(),
            major_mode: Box::new(PlainTextMode::new()),
            minor_modes: Vec::new(),
            syntax_tree: None,
            parser: Parser::new(),
        }
    }

    /// Create a buffer from text content
    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            file_path: None,
            is_dirty: false,
            cursor: Position::new(0, 0),
            selections: Vec::new(),
            major_mode: Box::new(PlainTextMode::new()),
            minor_modes: Vec::new(),
            syntax_tree: None,
            parser: Parser::new(),
        }
    }

    /// Create a buffer from a file path
    pub fn from_file(path: PathBuf, text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            file_path: Some(path),
            is_dirty: false,
            cursor: Position::new(0, 0),
            selections: Vec::new(),
            major_mode: Box::new(PlainTextMode::new()),
            minor_modes: Vec::new(),
            syntax_tree: None,
            parser: Parser::new(),
        }
    }

    /// Insert text at the given position
    /// Requirement 2.9: Sets is_dirty = true
    pub fn insert(&mut self, pos: Position, text: &str) -> Result<(), VeloError> {
        let char_idx = self.position_to_char_idx(pos)?;
        self.rope.insert(char_idx, text);
        self.is_dirty = true;
        self.update_syntax_tree();
        Ok(())
    }

    /// Delete a range of text
    /// Requirement 2.9: Sets is_dirty = true
    pub fn delete(&mut self, start: Position, end: Position) -> Result<(), VeloError> {
        let start_idx = self.position_to_char_idx(start)?;
        let end_idx = self.position_to_char_idx(end)?;

        if start_idx > end_idx {
            return Err(VeloError::InvalidRange);
        }

        self.rope.remove(start_idx..end_idx);
        self.is_dirty = true;
        self.update_syntax_tree();
        Ok(())
    }

    /// Insert a single character at the cursor position
    /// Requirement 2.9: Sets is_dirty = true
    pub fn insert_char(&mut self, ch: char) -> Result<(), VeloError> {
        // NOTE: previously this used an undefined variable `pos`.  It should use the
        // current cursor position.
        let char_idx = self.position_to_char_idx(self.cursor)?;
        self.rope.insert_char(char_idx, ch);
        self.is_dirty = true;
        self.update_syntax_tree();
        Ok(())
    }

    /// Delete a single character at the cursor position
    /// Requirement 2.9: Sets is_dirty = true
    pub fn delete_char(&mut self) -> Result<(), VeloError> {
        let char_idx = self.position_to_char_idx(self.cursor)?;
        if char_idx >= self.rope.len_chars() {
            return Err(VeloError::InvalidPosition);
        }

        self.rope.remove(char_idx..char_idx + 1);
        self.is_dirty = true;
        self.update_syntax_tree();
        Ok(())
    }

    /// Replace a range of text with new text
    /// Requirement 2.9: Sets is_dirty = true
    pub fn replace(&mut self, start: Position, end: Position, text: &str) -> Result<(), VeloError> {
        self.delete(start, end)?;
        self.insert(start, text)?;
        // is_dirty is already set by delete and insert
        self.update_syntax_tree();
        Ok(())
    }

    /// Save the buffer to its file path
    /// Requirement 2.10: Sets is_dirty = false
    pub fn save(&mut self) -> Result<(), VeloError> {
        let path = self.file_path.as_ref().ok_or(VeloError::NoFilePath)?;

        let content = self.rope.to_string();
        std::fs::write(path, content).map_err(|e| VeloError::IoError(e.to_string()))?;

        self.is_dirty = false;
        Ok(())
    }

    /// Save the buffer to a specific file path
    /// Requirement 2.10: Sets is_dirty = false
    pub fn save_as(&mut self, path: PathBuf) -> Result<(), VeloError> {
        let content = self.rope.to_string();
        std::fs::write(&path, content).map_err(|e| VeloError::IoError(e.to_string()))?;

        self.file_path = Some(path);
        self.is_dirty = false;
        Ok(())
    }

    /// Get the text content as a string
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get the number of lines in the buffer
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get the text of a specific line
    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.rope.len_lines() {
            return None;
        }

        let line = self.rope.line(line_idx);
        Some(line.to_string())
    }

    /// Convert a Position to a character index in the rope
    pub fn position_to_char_idx(&self, pos: Position) -> Result<usize, VeloError> {
        if pos.line >= self.rope.len_lines() {
            return Err(VeloError::InvalidPosition);
        }

        let line_start = self.rope.line_to_char(pos.line);
        let line = self.rope.line(pos.line);
        let line_len = line.len_chars();

        if pos.column > line_len {
            return Err(VeloError::InvalidPosition);
        }

        Ok(line_start + pos.column)
    }

    /// Convert a character index to a Position
    #[allow(dead_code)]
    fn char_idx_to_position(&self, char_idx: usize) -> Result<Position, VeloError> {
        if char_idx > self.rope.len_chars() {
            return Err(VeloError::InvalidPosition);
        }

        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        let column = char_idx - line_start;

        Ok(Position::new(line, column))
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_new() {
        let buffer = Buffer::new();
        assert_eq!(buffer.text(), "");
        assert_eq!(buffer.file_path, None);
        assert!(!buffer.is_dirty);
        assert_eq!(buffer.cursor, Position::new(0, 0));
        assert!(buffer.selections.is_empty());
        assert!(buffer.minor_modes.is_empty());
        assert!(buffer.syntax_tree.is_none());
    }

    #[test]
    fn test_buffer_default() {
        let buffer = Buffer::default();
        assert_eq!(buffer.text(), "");
        assert!(!buffer.is_dirty);
    }

    #[test]
    fn test_buffer_from_text() {
        let buffer = Buffer::from_text("Hello, world!");
        assert_eq!(buffer.text(), "Hello, world!");
        assert_eq!(buffer.file_path, None);
        assert!(!buffer.is_dirty);
    }

    #[test]
    fn test_buffer_from_file() {
        let path = PathBuf::from("test.txt");
        let buffer = Buffer::from_file(path.clone(), "File content");
        assert_eq!(buffer.text(), "File content");
        assert_eq!(buffer.file_path, Some(path));
        assert!(!buffer.is_dirty);
    }

    #[test]
    fn test_insert_sets_dirty() {
        let mut buffer = Buffer::from_text("Hello");
        assert!(!buffer.is_dirty);

        buffer.insert(Position::new(0, 5), " world").unwrap();
        assert!(buffer.is_dirty);
        assert_eq!(buffer.text(), "Hello world");
    }

    #[test]
    fn test_delete_sets_dirty() {
        let mut buffer = Buffer::from_text("Hello world");
        assert!(!buffer.is_dirty);

        buffer
            .delete(Position::new(0, 5), Position::new(0, 11))
            .unwrap();
        assert!(buffer.is_dirty);
        assert_eq!(buffer.text(), "Hello");
    }

    #[test]
    fn test_insert_char_sets_dirty() {
        let mut buffer = Buffer::from_text("Hello");
        buffer.cursor = Position::new(0, 5);
        assert!(!buffer.is_dirty);

        buffer.insert_char('!').unwrap();
        assert!(buffer.is_dirty);
        assert_eq!(buffer.text(), "Hello!");
    }

    #[test]
    fn test_delete_char_sets_dirty() {
        let mut buffer = Buffer::from_text("Hello!");
        buffer.cursor = Position::new(0, 5);
        assert!(!buffer.is_dirty);

        buffer.delete_char().unwrap();
        assert!(buffer.is_dirty);
        assert_eq!(buffer.text(), "Hello");
    }

    #[test]
    fn test_replace_sets_dirty() {
        let mut buffer = Buffer::from_text("Hello world");
        assert!(!buffer.is_dirty);

        buffer
            .replace(Position::new(0, 0), Position::new(0, 5), "Hi")
            .unwrap();
        assert!(buffer.is_dirty);
        assert_eq!(buffer.text(), "Hi world");
    }

    #[test]
    fn test_save_clears_dirty() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("velo_test_save.txt");

        let mut buffer = Buffer::from_file(temp_file.clone(), "Initial content");
        buffer.insert(Position::new(0, 15), " modified").unwrap();
        assert!(buffer.is_dirty);

        buffer.save().unwrap();
        assert!(!buffer.is_dirty);

        let content = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(content, "Initial content modified");

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_save_as_clears_dirty() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("velo_test_save_as.txt");

        let mut buffer = Buffer::from_text("Test content");
        buffer.insert(Position::new(0, 12), " saved").unwrap();
        assert!(buffer.is_dirty);

        buffer.save_as(temp_file.clone()).unwrap();
        assert!(!buffer.is_dirty);
        assert_eq!(buffer.file_path, Some(temp_file.clone()));

        let content = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(content, "Test content saved");

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_save_without_file_path_fails() {
        let mut buffer = Buffer::from_text("No path");
        buffer.is_dirty = true;

        let result = buffer.save();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::NoFilePath));
    }

    #[test]
    fn test_line_count() {
        let buffer = Buffer::from_text("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn test_line() {
        let buffer = Buffer::from_text("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line(0), Some("Line 1\n".to_string()));
        assert_eq!(buffer.line(1), Some("Line 2\n".to_string()));
        assert_eq!(buffer.line(2), Some("Line 3".to_string()));
        assert_eq!(buffer.line(3), None);
    }

    #[test]
    fn test_delete_invalid_range() {
        let mut buffer = Buffer::from_text("Hello");
        let result = buffer.delete(Position::new(0, 5), Position::new(0, 0));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::InvalidRange));
    }

    #[test]
    fn test_insert_invalid_position() {
        let mut buffer = Buffer::from_text("Hello");
        let result = buffer.insert(Position::new(10, 0), "text");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::InvalidPosition));
    }

    #[test]
    fn test_delete_char_at_end_fails() {
        let mut buffer = Buffer::from_text("Hi");
        buffer.cursor = Position::new(0, 2);
        let result = buffer.delete_char();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::InvalidPosition));
    }

    #[test]
    fn test_multiline_insert() {
        let mut buffer = Buffer::from_text("Line 1\nLine 3");
        buffer.insert(Position::new(1, 0), "Line 2\n").unwrap();
        assert_eq!(buffer.text(), "Line 1\nLine 2\nLine 3");
        assert!(buffer.is_dirty);
    }

    #[test]
    fn test_multiline_delete() {
        let mut buffer = Buffer::from_text("Line 1\nLine 2\nLine 3");
        buffer
            .delete(Position::new(1, 0), Position::new(2, 0))
            .unwrap();
        assert_eq!(buffer.text(), "Line 1\nLine 3");
        assert!(buffer.is_dirty);
    }

    #[test]
    fn test_buffer_has_major_mode() {
        let buffer = Buffer::new();
        assert_eq!(buffer.major_mode.name(), "plain-text");
    }

    #[test]
    fn test_buffer_has_empty_minor_modes() {
        let buffer = Buffer::new();
        assert!(buffer.minor_modes.is_empty());
    }

    #[test]
    fn test_buffer_has_no_syntax_tree_initially() {
        let buffer = Buffer::new();
        assert!(buffer.syntax_tree.is_none());
    }

    #[test]
    fn test_buffer_has_rope() {
        let buffer = Buffer::from_text("Test");
        assert_eq!(buffer.rope.to_string(), "Test");
    }

    #[test]
    fn test_buffer_has_selections() {
        let buffer = Buffer::new();
        assert!(buffer.selections.is_empty());
    }
}
