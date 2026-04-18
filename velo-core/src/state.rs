// EditorState as single source of truth
// Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7

use crate::{Buffer, MajorModeRegistry, VeloConfig};
use velo_plugin::PluginRegistry;
use velo_types::VeloError;

/// EditorState is the single source of truth for all mutable editor state
/// Requirements 3.1, 3.2, 3.3, 3.4, 3.5
pub struct EditorState {
    pub buffers: Vec<Buffer>,
    pub active_buffer_id: usize,
    pub config: VeloConfig,
    pub major_mode_registry: MajorModeRegistry,
    pub plugin_registry: PluginRegistry,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            active_buffer_id: 0,
            config: VeloConfig::default(),
            major_mode_registry: MajorModeRegistry::new(),
            plugin_registry: PluginRegistry::new(),
        }
    }

    /// Open a new buffer and set it as active
    /// Requirement 3.6: Maintains valid active_buffer_id
    pub fn open_buffer(&mut self, buffer: Buffer) -> usize {
        let buffer_id = self.buffers.len();
        self.buffers.push(buffer);
        self.active_buffer_id = buffer_id;
        buffer_id
    }

    /// Close a buffer by ID
    /// Requirement 3.6: Maintains valid active_buffer_id
    pub fn close_buffer(&mut self, buffer_id: usize) -> Result<(), VeloError> {
        if buffer_id >= self.buffers.len() {
            return Err(VeloError::InvalidBufferId);
        }

        self.buffers.remove(buffer_id);

        // Update active_buffer_id to maintain validity
        if self.buffers.is_empty() {
            // No buffers left, reset to 0
            self.active_buffer_id = 0;
        } else if self.active_buffer_id >= self.buffers.len() {
            // Active buffer was removed or is now out of bounds
            self.active_buffer_id = self.buffers.len() - 1;
        } else if buffer_id < self.active_buffer_id {
            // A buffer before the active one was removed, adjust index
            self.active_buffer_id -= 1;
        }
        // If buffer_id > active_buffer_id, no adjustment needed

        Ok(())
    }

    /// Get a reference to the active buffer
    /// Requirement 3.7: Safe access with error handling
    pub fn active_buffer(&self) -> Result<&Buffer, VeloError> {
        if self.buffers.is_empty() {
            return Err(VeloError::NoBuffersOpen);
        }

        self.buffers
            .get(self.active_buffer_id)
            .ok_or(VeloError::InvalidBufferId)
    }

    /// Get a mutable reference to the active buffer
    /// Requirement 3.7: Safe access with error handling
    pub fn active_buffer_mut(&mut self) -> Result<&mut Buffer, VeloError> {
        if self.buffers.is_empty() {
            return Err(VeloError::NoBuffersOpen);
        }

        self.buffers
            .get_mut(self.active_buffer_id)
            .ok_or(VeloError::InvalidBufferId)
    }

    /// Get a reference to a buffer by ID
    pub fn buffer(&self, buffer_id: usize) -> Result<&Buffer, VeloError> {
        self.buffers
            .get(buffer_id)
            .ok_or(VeloError::InvalidBufferId)
    }

    /// Get a mutable reference to a buffer by ID
    pub fn buffer_mut(&mut self, buffer_id: usize) -> Result<&mut Buffer, VeloError> {
        self.buffers
            .get_mut(buffer_id)
            .ok_or(VeloError::InvalidBufferId)
    }

    /// Set the active buffer by ID
    /// Requirement 3.6: Maintains valid active_buffer_id
    pub fn set_active_buffer(&mut self, buffer_id: usize) -> Result<(), VeloError> {
        if buffer_id >= self.buffers.len() {
            return Err(VeloError::InvalidBufferId);
        }

        self.active_buffer_id = buffer_id;
        Ok(())
    }

    /// Get the number of open buffers
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_state_new() {
        let state = EditorState::new();
        assert_eq!(state.buffers.len(), 0);
        assert_eq!(state.active_buffer_id, 0);
        assert!(state.major_mode_registry.registered_modes().contains(&"plain-text"));
    }

    #[test]
    fn test_editor_state_default() {
        let state = EditorState::default();
        assert_eq!(state.buffers.len(), 0);
        assert_eq!(state.active_buffer_id, 0);
    }

    #[test]
    fn test_editor_state_has_config() {
        let state = EditorState::new();
        assert_eq!(state.config.editor.tab_width, 4);
        assert_eq!(state.config.theme.name, "velo-dark");
    }

    #[test]
    fn test_open_buffer_sets_active() {
        let mut state = EditorState::new();
        let buffer = Buffer::new();
        
        let buffer_id = state.open_buffer(buffer);
        
        assert_eq!(buffer_id, 0);
        assert_eq!(state.active_buffer_id, 0);
        assert_eq!(state.buffers.len(), 1);
    }

    #[test]
    fn test_open_multiple_buffers() {
        let mut state = EditorState::new();
        
        let id1 = state.open_buffer(Buffer::new());
        let id2 = state.open_buffer(Buffer::new());
        let id3 = state.open_buffer(Buffer::new());
        
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(state.active_buffer_id, 2);
        assert_eq!(state.buffers.len(), 3);
    }

    #[test]
    fn test_close_buffer_updates_active_id() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        
        // Active is buffer 2, close it
        state.close_buffer(2).unwrap();
        
        // Active should now be buffer 1 (last buffer)
        assert_eq!(state.active_buffer_id, 1);
        assert_eq!(state.buffers.len(), 2);
    }

    #[test]
    fn test_close_buffer_before_active() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        
        // Active is buffer 2, close buffer 0
        state.close_buffer(0).unwrap();
        
        // Active should be adjusted to buffer 1 (was buffer 2)
        assert_eq!(state.active_buffer_id, 1);
        assert_eq!(state.buffers.len(), 2);
    }

    #[test]
    fn test_close_buffer_after_active() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        
        state.set_active_buffer(1).unwrap();
        
        // Active is buffer 1, close buffer 2
        state.close_buffer(2).unwrap();
        
        // Active should remain buffer 1
        assert_eq!(state.active_buffer_id, 1);
        assert_eq!(state.buffers.len(), 2);
    }

    #[test]
    fn test_close_last_buffer() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        
        state.close_buffer(0).unwrap();
        
        // No buffers left, active_buffer_id should be 0
        assert_eq!(state.active_buffer_id, 0);
        assert_eq!(state.buffers.len(), 0);
    }

    #[test]
    fn test_close_invalid_buffer() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        
        let result = state.close_buffer(5);
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, VeloError::InvalidBufferId));
        }
    }

    #[test]
    fn test_active_buffer_when_empty() {
        let state = EditorState::new();
        
        let result = state.active_buffer();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, VeloError::NoBuffersOpen));
        }
    }

    #[test]
    fn test_active_buffer_mut_when_empty() {
        let mut state = EditorState::new();
        
        let result = state.active_buffer_mut();
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, VeloError::NoBuffersOpen));
        }
    }

    #[test]
    fn test_active_buffer_success() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::from_text("Test content"));
        
        let buffer = state.active_buffer().unwrap();
        
        assert_eq!(buffer.text(), "Test content");
    }

    #[test]
    fn test_active_buffer_mut_success() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::from_text("Test content"));
        
        let buffer = state.active_buffer_mut().unwrap();
        buffer.insert(velo_types::Position::new(0, 12), " modified").unwrap();
        
        assert_eq!(buffer.text(), "Test content modified");
    }

    #[test]
    fn test_buffer_by_id() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::from_text("Buffer 0"));
        state.open_buffer(Buffer::from_text("Buffer 1"));
        state.open_buffer(Buffer::from_text("Buffer 2"));
        
        let buffer = state.buffer(1).unwrap();
        
        assert_eq!(buffer.text(), "Buffer 1");
    }

    #[test]
    fn test_buffer_by_id_invalid() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        
        let result = state.buffer(5);
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, VeloError::InvalidBufferId));
        }
    }

    #[test]
    fn test_buffer_mut_by_id() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::from_text("Buffer 0"));
        state.open_buffer(Buffer::from_text("Buffer 1"));
        
        let buffer = state.buffer_mut(1).unwrap();
        buffer.insert(velo_types::Position::new(0, 8), " modified").unwrap();
        
        assert_eq!(buffer.text(), "Buffer 1 modified");
    }

    #[test]
    fn test_set_active_buffer() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        
        state.set_active_buffer(1).unwrap();
        
        assert_eq!(state.active_buffer_id, 1);
    }

    #[test]
    fn test_set_active_buffer_invalid() {
        let mut state = EditorState::new();
        state.open_buffer(Buffer::new());
        
        let result = state.set_active_buffer(5);
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, VeloError::InvalidBufferId));
        }
    }

    #[test]
    fn test_buffer_count() {
        let mut state = EditorState::new();
        
        assert_eq!(state.buffer_count(), 0);
        
        state.open_buffer(Buffer::new());
        assert_eq!(state.buffer_count(), 1);
        
        state.open_buffer(Buffer::new());
        assert_eq!(state.buffer_count(), 2);
        
        state.close_buffer(0).unwrap();
        assert_eq!(state.buffer_count(), 1);
    }

    #[test]
    fn test_active_buffer_id_validity_after_operations() {
        let mut state = EditorState::new();
        
        // Open 3 buffers
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        
        // Close middle buffer
        state.close_buffer(1).unwrap();
        
        // Active buffer should still be valid
        assert!(state.active_buffer().is_ok());
        assert!(state.active_buffer_id < state.buffers.len());
    }

    #[test]
    fn test_active_buffer_id_validity_complex_sequence() {
        let mut state = EditorState::new();
        
        // Open buffers
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        state.open_buffer(Buffer::new());
        
        // Set active to buffer 1
        state.set_active_buffer(1).unwrap();
        
        // Close buffer 3
        state.close_buffer(3).unwrap();
        assert_eq!(state.active_buffer_id, 1);
        assert!(state.active_buffer().is_ok());
        
        // Close buffer 0
        state.close_buffer(0).unwrap();
        assert_eq!(state.active_buffer_id, 0); // Was 1, now adjusted to 0
        assert!(state.active_buffer().is_ok());
        
        // Close buffer 1 (was originally buffer 2)
        state.close_buffer(1).unwrap();
        assert_eq!(state.active_buffer_id, 0);
        assert!(state.active_buffer().is_ok());
        
        // Close last buffer
        state.close_buffer(0).unwrap();
        assert_eq!(state.active_buffer_id, 0);
        assert_eq!(state.buffers.len(), 0);
    }
}
