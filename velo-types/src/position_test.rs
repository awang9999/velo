// Tests for Position, Range, and Selection types
// Requirements 4.1, 4.2, 4.3, 4.4, 4.5

#[cfg(test)]
mod tests {
    use crate::{Position, Range, Selection, VeloError};

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(5, 10);
        let pos2 = Position::new(5, 10);
        let pos3 = Position::new(5, 11);
        
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_ordering() {
        let pos1 = Position::new(1, 5);
        let pos2 = Position::new(2, 3);
        let pos3 = Position::new(2, 10);
        
        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
        assert!(pos1 < pos3);
    }

    #[test]
    fn test_range_valid_different_lines() {
        let start = Position::new(1, 5);
        let end = Position::new(3, 2);
        let range = Range::new(start, end);
        
        assert!(range.is_ok());
        let range = range.unwrap();
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_range_valid_same_line() {
        let start = Position::new(5, 10);
        let end = Position::new(5, 20);
        let range = Range::new(start, end);
        
        assert!(range.is_ok());
        let range = range.unwrap();
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_range_valid_same_position() {
        let pos = Position::new(5, 10);
        let range = Range::new(pos, pos);
        
        assert!(range.is_ok());
    }

    #[test]
    fn test_range_invalid_same_line_wrong_order() {
        let start = Position::new(5, 20);
        let end = Position::new(5, 10);
        let range = Range::new(start, end);
        
        assert!(range.is_err());
        assert!(matches!(range.unwrap_err(), VeloError::InvalidRange));
    }

    #[test]
    fn test_range_invalid_different_lines_wrong_order() {
        let start = Position::new(10, 5);
        let end = Position::new(5, 10);
        let range = Range::new(start, end);
        
        assert!(range.is_err());
        assert!(matches!(range.unwrap_err(), VeloError::InvalidRange));
    }

    #[test]
    fn test_range_unchecked() {
        let start = Position::new(10, 5);
        let end = Position::new(5, 10);
        // This should not panic even with invalid ordering
        let range = Range::new_unchecked(start, end);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_selection_creation() {
        let anchor = Position::new(1, 5);
        let head = Position::new(3, 10);
        let selection = Selection::new(anchor, head);
        
        assert_eq!(selection.anchor, anchor);
        assert_eq!(selection.head, head);
    }

    #[test]
    fn test_selection_equality() {
        let sel1 = Selection::new(Position::new(1, 5), Position::new(3, 10));
        let sel2 = Selection::new(Position::new(1, 5), Position::new(3, 10));
        let sel3 = Selection::new(Position::new(1, 5), Position::new(3, 11));
        
        assert_eq!(sel1, sel2);
        assert_ne!(sel1, sel3);
    }
}
