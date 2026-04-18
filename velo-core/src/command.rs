// Command trait and system
// Requirements 5.1, 5.2, 5.3

use crate::EditorState;
use std::path::Path;
use velo_plugin::PLUGIN_DIR;
use velo_types::VeloError;

/// Command trait - the only way to mutate EditorState
///
/// Commands enforce unidirectional data flow and make undo/redo straightforward.
/// Every user action is modeled as a Command.
///
/// Requirements:
/// - 5.1: execute method for applying the command
/// - 5.2: undo method for reverting the command (optional)
/// - 5.3: name method for identifying the command
pub trait Command {
    /// Execute the command, mutating the EditorState
    /// Requirement 5.1
    fn execute(&self, state: &mut EditorState) -> Result<(), VeloError>;

    /// Undo the command, restoring EditorState to its pre-execute state
    /// Requirement 5.2
    ///
    /// This is an optional operation. The default implementation returns an error
    /// indicating that undo is not supported for this command.
    fn undo(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let _ = state;
        Err(VeloError::UndoNotSupported)
    }

    /// Get the name of this command
    /// Requirement 5.3
    fn name(&self) -> &str;
}

// Built-in command implementations
// Requirements 5.4, 5.6

use std::cell::RefCell;
use std::path::PathBuf;
use velo_types::Position;

/// InsertChar command - inserts a character at the cursor position
/// Requirement 5.4, 5.6
pub struct InsertChar {
    pub ch: char,
    /// Stored for undo: the position where the char was inserted
    undo_info: RefCell<Option<Position>>,
}

impl InsertChar {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            undo_info: RefCell::new(None),
        }
    }
}

impl Command for InsertChar {
    fn execute(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let buffer = state.active_buffer_mut()?;
        let cursor_pos = buffer.cursor;

        // Store position for undo
        *self.undo_info.borrow_mut() = Some(cursor_pos);

        buffer.insert_char(self.ch)?;

        // Move cursor forward by 1 column
        buffer.cursor = Position::new(cursor_pos.line, cursor_pos.column + 1);

        Ok(())
    }

    fn undo(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let buffer = state.active_buffer_mut()?;
        let original_pos = self.undo_info.borrow().ok_or(VeloError::Other(
            "No undo information available".to_string(),
        ))?;

        // Move cursor to the position after the inserted char
        buffer.cursor = Position::new(original_pos.line, original_pos.column + 1);

        // Delete the character that was inserted (cursor needs to be at the char position)
        buffer.cursor = original_pos;
        buffer.delete_char()?;

        Ok(())
    }

    fn name(&self) -> &str {
        "insert_char"
    }
}

/// DeleteChar command - deletes a character at the cursor position
/// Requirement 5.4, 5.6
pub struct DeleteChar {
    /// Stored for undo: the deleted character and position
    undo_info: RefCell<Option<(char, Position)>>,
}

impl DeleteChar {
    pub fn new() -> Self {
        Self {
            undo_info: RefCell::new(None),
        }
    }
}

impl Command for DeleteChar {
    fn execute(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let buffer = state.active_buffer_mut()?;
        let cursor_pos = buffer.cursor;

        // Get the character before deleting
        let char_idx = buffer.position_to_char_idx(cursor_pos)?;
        if char_idx >= buffer.rope.len_chars() {
            return Err(VeloError::InvalidPosition);
        }

        let ch = buffer.rope.char(char_idx);

        // Store for undo
        *self.undo_info.borrow_mut() = Some((ch, cursor_pos));

        // Delete the character
        buffer.delete_char()?;

        Ok(())
    }

    fn undo(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let (ch, pos) = self.undo_info.borrow().ok_or(VeloError::Other(
            "No undo information available".to_string(),
        ))?;

        let buffer = state.active_buffer_mut()?;

        // Move cursor to the deletion position
        buffer.cursor = pos;

        // Re-insert the deleted character
        buffer.insert_char(ch)?;

        // Restore cursor position
        buffer.cursor = pos;

        Ok(())
    }

    fn name(&self) -> &str {
        "delete_char"
    }
}

/// MoveCursor command - moves the cursor to a new position
/// Requirement 5.4, 5.6
pub struct MoveCursor {
    pub new_position: Position,
    /// Stored for undo: the previous cursor position
    undo_info: RefCell<Option<Position>>,
}

impl MoveCursor {
    pub fn new(new_position: Position) -> Self {
        Self {
            new_position,
            undo_info: RefCell::new(None),
        }
    }
}

impl Command for MoveCursor {
    fn execute(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let buffer = state.active_buffer_mut()?;

        // Validate the new position is within bounds
        let _ = buffer.position_to_char_idx(self.new_position)?;

        // Store old position for undo
        *self.undo_info.borrow_mut() = Some(buffer.cursor);

        // Move cursor
        buffer.cursor = self.new_position;

        Ok(())
    }

    fn undo(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let old_pos = self.undo_info.borrow().ok_or(VeloError::Other(
            "No undo information available".to_string(),
        ))?;

        let buffer = state.active_buffer_mut()?;
        buffer.cursor = old_pos;

        Ok(())
    }

    fn name(&self) -> &str {
        "move_cursor"
    }
}

/// SaveFile command - saves the active buffer to disk
/// Requirement 5.4, 5.6
pub struct SaveFile;

impl SaveFile {
    pub fn new() -> Self {
        Self
    }
}

impl Command for SaveFile {
    fn execute(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let buffer = state.active_buffer_mut()?;
        buffer.save()?;
        Ok(())
    }

    fn undo(&self, _state: &mut EditorState) -> Result<(), VeloError> {
        // Saving a file cannot be undone
        Err(VeloError::UndoNotSupported)
    }

    fn name(&self) -> &str {
        "save_file"
    }
}

/// OpenFile command - opens a file into a new buffer
/// Requirement 5.4, 5.6
pub struct OpenFile {
    pub path: PathBuf,
    /// Stored for undo: the buffer ID that was created
    undo_info: RefCell<Option<usize>>,
}

impl OpenFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            undo_info: RefCell::new(None),
        }
    }
}

impl Command for OpenFile {
    fn execute(&self, state: &mut EditorState) -> Result<(), VeloError> {
        // Read the file content
        let content =
            std::fs::read_to_string(&self.path).map_err(|e| VeloError::IoError(e.to_string()))?;

        // Create a new buffer with the file content
        let buffer = crate::Buffer::from_file(self.path.clone(), &content);

        // Open the buffer and store the ID for undo
        let buffer_id = state.open_buffer(buffer);
        *self.undo_info.borrow_mut() = Some(buffer_id);

        Ok(())
    }

    fn undo(&self, state: &mut EditorState) -> Result<(), VeloError> {
        let buffer_id = self.undo_info.borrow().ok_or(VeloError::Other(
            "No undo information available".to_string(),
        ))?;

        // Close the buffer that was opened
        state.close_buffer(buffer_id)?;

        Ok(())
    }

    fn name(&self) -> &str {
        "open_file"
    }
}

/// `:plugin install <name>` – install a new plugin
///
/// The command simply creates an empty `.so` file in the plugin directory
/// (in a real editor you’d download the binary from a server).  The UI can
/// show a “Restart required” message after the command finishes.
pub struct InstallPlugin {
    /// The plugin name (used to build the file name and the URL).
    pub name: String,
}

impl InstallPlugin {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Command for InstallPlugin {
    fn execute(&self, _state: &mut EditorState) -> Result<(), VeloError> {
        // 1. Construct the destination path.
        let dest_path = Path::new(PLUGIN_DIR).join(format!("{}.so", self.name));

        // 2. Ensure the plugin directory exists.
        std::fs::create_dir_all(PLUGIN_DIR).ok();

        // 3. In a real implementation you would download the binary here.
        //    For this demo we just create an empty file to represent the plugin.
        std::fs::write(&dest_path, b"").map_err(|e| VeloError::IoError(e.to_string()))?;

        // 4. Notify the user that a restart is required.
        println!(
            "Plugin '{}' installed to {:?}. Restart Velo to activate it.",
            self.name, dest_path
        );

        Ok(())
    }

    // We don’t support undo – the user can simply delete the file manually if needed.
    fn undo(&self, _state: &mut EditorState) -> Result<(), VeloError> {
        Err(VeloError::UndoNotSupported)
    }

    fn name(&self) -> &str {
        "install_plugin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Buffer, EditorState};
    use std::fs;
    use std::path::Path;
    use velo_types::Position;

    #[test]
    fn test_insert_char_execute() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hello");
        state.open_buffer(buffer);

        // Set cursor to end of text
        state.active_buffer_mut().unwrap().cursor = Position::new(0, 5);

        let cmd = InsertChar::new('!');
        cmd.execute(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "Hello!");
        assert_eq!(buffer.cursor, Position::new(0, 6));
        assert!(buffer.is_dirty);
    }

    #[test]
    fn test_insert_char_undo() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hello");
        state.open_buffer(buffer);

        state.active_buffer_mut().unwrap().cursor = Position::new(0, 5);

        let cmd = InsertChar::new('!');
        cmd.execute(&mut state).unwrap();
        cmd.undo(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "Hello");
        assert_eq!(buffer.cursor, Position::new(0, 5));
    }

    #[test]
    fn test_insert_char_middle_of_text() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Helo");
        state.open_buffer(buffer);

        state.active_buffer_mut().unwrap().cursor = Position::new(0, 2);

        let cmd = InsertChar::new('l');
        cmd.execute(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "Hello");
        assert_eq!(buffer.cursor, Position::new(0, 3));
    }

    #[test]
    fn test_delete_char_execute() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hello!");
        state.open_buffer(buffer);

        state.active_buffer_mut().unwrap().cursor = Position::new(0, 5);

        let cmd = DeleteChar::new();
        cmd.execute(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "Hello");
        assert!(buffer.is_dirty);
    }

    #[test]
    fn test_delete_char_undo() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hello!");
        state.open_buffer(buffer);

        state.active_buffer_mut().unwrap().cursor = Position::new(0, 5);

        let cmd = DeleteChar::new();
        cmd.execute(&mut state).unwrap();
        cmd.undo(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "Hello!");
        assert_eq!(buffer.cursor, Position::new(0, 5));
    }

    #[test]
    fn test_delete_char_at_end_fails() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hi");
        state.open_buffer(buffer);

        state.active_buffer_mut().unwrap().cursor = Position::new(0, 2);

        let cmd = DeleteChar::new();
        let result = cmd.execute(&mut state);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::InvalidPosition));
    }

    #[test]
    fn test_move_cursor_execute() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hello\nWorld");
        state.open_buffer(buffer);

        let cmd = MoveCursor::new(Position::new(1, 3));
        cmd.execute(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.cursor, Position::new(1, 3));
    }

    #[test]
    fn test_move_cursor_undo() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hello\nWorld");
        state.open_buffer(buffer);

        let original_pos = Position::new(0, 0);
        state.active_buffer_mut().unwrap().cursor = original_pos;

        let cmd = MoveCursor::new(Position::new(1, 3));
        cmd.execute(&mut state).unwrap();
        cmd.undo(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.cursor, original_pos);
    }

    #[test]
    fn test_move_cursor_invalid_position() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Hello");
        state.open_buffer(buffer);

        let cmd = MoveCursor::new(Position::new(10, 0));
        let result = cmd.execute(&mut state);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::InvalidPosition));
    }

    #[test]
    fn test_save_file_execute() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("velo_test_save_cmd.txt");

        let mut state = EditorState::new();
        let mut buffer = Buffer::from_file(temp_file.clone(), "Initial");
        buffer.insert(Position::new(0, 7), " content").unwrap();
        state.open_buffer(buffer);

        let cmd = SaveFile::new();
        cmd.execute(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert!(!buffer.is_dirty);

        let content = fs::read_to_string(&temp_file).unwrap();
        assert_eq!(content, "Initial content");

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_save_file_undo_not_supported() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Test");
        state.open_buffer(buffer);

        let cmd = SaveFile::new();
        let result = cmd.undo(&mut state);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::UndoNotSupported));
    }

    #[test]
    fn test_save_file_no_path_fails() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Test");
        state.open_buffer(buffer);

        let cmd = SaveFile::new();
        let result = cmd.execute(&mut state);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::NoFilePath));
    }

    #[test]
    fn test_open_file_execute() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("velo_test_open_cmd.txt");

        fs::write(&temp_file, "File content").unwrap();

        let mut state = EditorState::new();

        let cmd = OpenFile::new(temp_file.clone());
        cmd.execute(&mut state).unwrap();

        assert_eq!(state.buffer_count(), 1);
        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "File content");
        assert_eq!(buffer.file_path, Some(temp_file.clone()));

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_open_file_undo() {
        use std::env;
        use std::fs;

        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("velo_test_open_undo_cmd.txt");

        fs::write(&temp_file, "File content").unwrap();

        let mut state = EditorState::new();

        let cmd = OpenFile::new(temp_file.clone());
        cmd.execute(&mut state).unwrap();
        assert_eq!(state.buffer_count(), 1);

        cmd.undo(&mut state).unwrap();
        assert_eq!(state.buffer_count(), 0);

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_open_file_nonexistent_fails() {
        let mut state = EditorState::new();

        let cmd = OpenFile::new(PathBuf::from("/nonexistent/file.txt"));
        let result = cmd.execute(&mut state);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VeloError::IoError(_)));
    }

    #[test]
    fn test_command_names() {
        assert_eq!(InsertChar::new('a').name(), "insert_char");
        assert_eq!(DeleteChar::new().name(), "delete_char");
        assert_eq!(MoveCursor::new(Position::new(0, 0)).name(), "move_cursor");
        assert_eq!(SaveFile::new().name(), "save_file");
        assert_eq!(OpenFile::new(PathBuf::from("test.txt")).name(), "open_file");
    }

    #[test]
    fn test_insert_char_multiline() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Line 1\nLine 2");
        state.open_buffer(buffer);

        state.active_buffer_mut().unwrap().cursor = Position::new(1, 6);

        let cmd = InsertChar::new('!');
        cmd.execute(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "Line 1\nLine 2!");
    }

    #[test]
    fn test_delete_char_multiline() {
        let mut state = EditorState::new();
        let buffer = Buffer::from_text("Line 1\nLine 2!");
        state.open_buffer(buffer);

        state.active_buffer_mut().unwrap().cursor = Position::new(1, 6);

        let cmd = DeleteChar::new();
        cmd.execute(&mut state).unwrap();

        let buffer = state.active_buffer().unwrap();
        assert_eq!(buffer.text(), "Line 1\nLine 2");
    }

    #[test]
    fn test_commands_require_active_buffer() {
        let mut state = EditorState::new();

        let insert_result = InsertChar::new('a').execute(&mut state);
        assert!(insert_result.is_err());
        assert!(matches!(
            insert_result.unwrap_err(),
            VeloError::NoBuffersOpen
        ));

        let delete_result = DeleteChar::new().execute(&mut state);
        assert!(delete_result.is_err());
        assert!(matches!(
            delete_result.unwrap_err(),
            VeloError::NoBuffersOpen
        ));

        let move_result = MoveCursor::new(Position::new(0, 0)).execute(&mut state);
        assert!(move_result.is_err());
        assert!(matches!(move_result.unwrap_err(), VeloError::NoBuffersOpen));

        let save_result = SaveFile::new().execute(&mut state);
        assert!(save_result.is_err());
        assert!(matches!(save_result.unwrap_err(), VeloError::NoBuffersOpen));
    }

    #[test]
    fn test_install_plugin() {
        // Use a temporary name to avoid clashing with real plugins
        let plugin_name = "temp_test_plugin";
        let cmd = InstallPlugin::new(plugin_name.to_string());
        let result = cmd.execute(&mut EditorState::new());
        assert!(result.is_ok());

        // The file should now exist
        let expected_path = Path::new(PLUGIN_DIR).join(format!("{}.so", plugin_name));
        assert!(expected_path.exists());

        // The file should be empty
        let metadata = fs::metadata(&expected_path).expect("Failed to read metadata");
        assert_eq!(metadata.len(), 0);

        // Clean up after the test
        fs::remove_file(&expected_path).ok();
    }
}
