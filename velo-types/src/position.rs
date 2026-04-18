// Position, Range, and Selection types
// Requirements 4.1, 4.2, 4.3, 4.4, 4.5

use crate::VeloError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    /// Creates a new Range with validation.
    /// Returns VeloError::InvalidRange if start > end.
    /// Requirements 4.4, 4.5
    pub fn new(start: Position, end: Position) -> Result<Self, VeloError> {
        // Validate: start.line < end.line OR (start.line == end.line AND start.column <= end.column)
        if start.line < end.line || (start.line == end.line && start.column <= end.column) {
            Ok(Self { start, end })
        } else {
            Err(VeloError::InvalidRange)
        }
    }

    /// Creates a new Range without validation (for internal use when invariants are known).
    pub fn new_unchecked(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    pub anchor: Position,
    pub head: Position,
}

impl Selection {
    pub fn new(anchor: Position, head: Position) -> Self {
        Self { anchor, head }
    }
}
