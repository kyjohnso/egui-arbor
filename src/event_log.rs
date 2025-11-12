//! Event logging system for tracking outliner interactions.
//!
//! This module provides types for logging and tracking user interactions with the outliner,
//! including selections, visibility changes, lock toggles, drag-drop operations, and renames.
//!
//! # Examples
//!
//! ```
//! use egui_arbor::event_log::{EventLog, EventType};
//!
//! let mut log = EventLog::<u64>::new(10); // Keep last 10 events
//!
//! log.log("Selected node 5", EventType::Selection, Some(5));
//! log.log("Renamed node 3", EventType::Rename, Some(3));
//!
//! for entry in log.entries() {
//!     println!("{}: {}", entry.event_type_str(), entry.message);
//! }
//! ```

use std::collections::VecDeque;
use std::time::SystemTime;

/// Type of event that occurred in the outliner.
///
/// This enum categorizes different types of user interactions for logging
/// and filtering purposes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventType {
    /// Node selection or deselection event.
    Selection,

    /// Visibility toggle event (show/hide).
    Visibility,

    /// Lock state toggle event.
    Lock,

    /// Drag-and-drop operation event.
    DragDrop,

    /// Node rename event.
    Rename,

    /// Custom event type with a string identifier.
    Custom(String),
}

impl EventType {
    /// Returns a string representation of the event type.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::EventType;
    ///
    /// assert_eq!(EventType::Selection.as_str(), "Selection");
    /// assert_eq!(EventType::Custom("MyEvent".into()).as_str(), "MyEvent");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            EventType::Selection => "Selection",
            EventType::Visibility => "Visibility",
            EventType::Lock => "Lock",
            EventType::DragDrop => "DragDrop",
            EventType::Rename => "Rename",
            EventType::Custom(s) => s.as_str(),
        }
    }
}

/// A single log entry recording an event.
///
/// Each entry contains:
/// - A timestamp of when the event occurred
/// - A human-readable message describing the event
/// - The type of event
/// - An optional node ID associated with the event
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogEntry<Id> {
    /// When the event occurred.
    pub timestamp: SystemTime,

    /// Human-readable description of the event.
    pub message: String,

    /// The type of event that occurred.
    pub event_type: EventType,

    /// The ID of the node involved in the event, if applicable.
    pub node_id: Option<Id>,
}

impl<Id> LogEntry<Id> {
    /// Creates a new log entry.
    ///
    /// # Arguments
    ///
    /// * `message` - Human-readable description of the event
    /// * `event_type` - The type of event
    /// * `node_id` - Optional ID of the node involved
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{LogEntry, EventType};
    ///
    /// let entry = LogEntry::new(
    ///     "Selected node".to_string(),
    ///     EventType::Selection,
    ///     Some(42u64),
    /// );
    /// ```
    pub fn new(message: String, event_type: EventType, node_id: Option<Id>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            message,
            event_type,
            node_id,
        }
    }

    /// Returns the event type as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{LogEntry, EventType};
    ///
    /// let entry = LogEntry::<u64>::new("test".into(), EventType::Selection, None);
    /// assert_eq!(entry.event_type_str(), "Selection");
    /// ```
    pub fn event_type_str(&self) -> &str {
        self.event_type.as_str()
    }

    /// Returns the elapsed time since this event occurred.
    ///
    /// # Returns
    ///
    /// `Ok(Duration)` if the elapsed time could be calculated, or an error if
    /// the system time has moved backwards.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{LogEntry, EventType};
    ///
    /// let entry = LogEntry::<u64>::new("test".into(), EventType::Selection, None);
    /// if let Ok(elapsed) = entry.elapsed() {
    ///     println!("Event occurred {} seconds ago", elapsed.as_secs());
    /// }
    /// ```
    pub fn elapsed(&self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        self.timestamp.elapsed()
    }

    /// Formats the elapsed time as a human-readable string.
    ///
    /// Returns strings like "5s ago", "2m ago", "1h ago", etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{LogEntry, EventType};
    ///
    /// let entry = LogEntry::<u64>::new("test".into(), EventType::Selection, None);
    /// println!("{}", entry.format_elapsed());
    /// ```
    pub fn format_elapsed(&self) -> String {
        match self.elapsed() {
            Ok(duration) => {
                let secs = duration.as_secs();
                if secs < 60 {
                    format!("{}s ago", secs)
                } else if secs < 3600 {
                    format!("{}m ago", secs / 60)
                } else if secs < 86400 {
                    format!("{}h ago", secs / 3600)
                } else {
                    format!("{}d ago", secs / 86400)
                }
            }
            Err(_) => "unknown".to_string(),
        }
    }
}

/// Event log for tracking outliner interactions.
///
/// This structure maintains a circular buffer of recent events, automatically
/// discarding old events when the maximum capacity is reached.
///
/// # Examples
///
/// ```
/// use egui_arbor::event_log::{EventLog, EventType};
///
/// let mut log = EventLog::<u64>::new(100);
///
/// log.log("Node 5 selected", EventType::Selection, Some(5));
/// log.log("Node 3 renamed to 'New Name'", EventType::Rename, Some(3));
///
/// assert_eq!(log.len(), 2);
///
/// for entry in log.entries() {
///     println!("{}: {}", entry.event_type_str(), entry.message);
/// }
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventLog<Id> {
    /// The log entries, with most recent first.
    entries: VecDeque<LogEntry<Id>>,

    /// Maximum number of entries to keep.
    max_entries: usize,
}

impl<Id> EventLog<Id> {
    /// Creates a new event log with the specified maximum capacity.
    ///
    /// # Arguments
    ///
    /// * `max_entries` - Maximum number of entries to keep. When this limit is
    ///   reached, the oldest entries are discarded.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::EventLog;
    ///
    /// let log = EventLog::<u64>::new(50);
    /// ```
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Logs a new event.
    ///
    /// The event is added to the front of the log (most recent). If the log
    /// is at capacity, the oldest event is removed.
    ///
    /// # Arguments
    ///
    /// * `message` - Human-readable description of the event
    /// * `event_type` - The type of event
    /// * `node_id` - Optional ID of the node involved
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{EventLog, EventType};
    ///
    /// let mut log = EventLog::new(10);
    /// log.log("Selected node 5", EventType::Selection, Some(5u64));
    /// ```
    pub fn log(&mut self, message: impl Into<String>, event_type: EventType, node_id: Option<Id>) {
        self.entries
            .push_front(LogEntry::new(message.into(), event_type, node_id));

        if self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
    }

    /// Returns a slice of all log entries, with most recent first.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{EventLog, EventType};
    ///
    /// let mut log = EventLog::new(10);
    /// log.log("Event 1", EventType::Selection, Some(1u64));
    /// log.log("Event 2", EventType::Rename, Some(2u64));
    ///
    /// assert_eq!(log.entries().count(), 2);
    /// let entries: Vec<_> = log.entries().collect();
    /// assert_eq!(entries[0].message, "Event 2"); // Most recent first
    /// ```
    pub fn entries(&self) -> impl Iterator<Item = &LogEntry<Id>> {
        self.entries.iter()
    }

    /// Returns the number of entries in the log.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{EventLog, EventType};
    ///
    /// let mut log = EventLog::<u64>::new(10);
    /// assert_eq!(log.len(), 0);
    ///
    /// log.log("Event", EventType::Selection, None);
    /// assert_eq!(log.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the log contains no entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::EventLog;
    ///
    /// let log = EventLog::<u64>::new(10);
    /// assert!(log.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clears all entries from the log.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{EventLog, EventType};
    ///
    /// let mut log = EventLog::<u64>::new(10);
    /// log.log("Event", EventType::Selection, None);
    /// assert!(!log.is_empty());
    ///
    /// log.clear();
    /// assert!(log.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the maximum number of entries this log can hold.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::EventLog;
    ///
    /// let log = EventLog::<u64>::new(50);
    /// assert_eq!(log.max_entries(), 50);
    /// ```
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Sets the maximum number of entries this log can hold.
    ///
    /// If the new maximum is less than the current number of entries,
    /// the oldest entries are removed to fit the new limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{EventLog, EventType};
    ///
    /// let mut log = EventLog::new(10);
    /// for i in 0..10 {
    ///     log.log(format!("Event {}", i), EventType::Selection, Some(i));
    /// }
    ///
    /// log.set_max_entries(5);
    /// assert_eq!(log.len(), 5);
    /// ```
    pub fn set_max_entries(&mut self, max_entries: usize) {
        self.max_entries = max_entries;
        while self.entries.len() > max_entries {
            self.entries.pop_back();
        }
    }

    /// Filters entries by event type.
    ///
    /// Returns an iterator over entries matching the specified event type.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_arbor::event_log::{EventLog, EventType};
    ///
    /// let mut log = EventLog::new(10);
    /// log.log("Selected", EventType::Selection, Some(1u64));
    /// log.log("Renamed", EventType::Rename, Some(2u64));
    /// log.log("Selected again", EventType::Selection, Some(3u64));
    ///
    /// let selections: Vec<_> = log.filter_by_type(&EventType::Selection).collect();
    /// assert_eq!(selections.len(), 2);
    /// ```
    pub fn filter_by_type(&self, event_type: &EventType) -> impl Iterator<Item = &LogEntry<Id>> {
        self.entries
            .iter()
            .filter(move |entry| &entry.event_type == event_type)
    }
}

impl<Id> Default for EventLog<Id> {
    /// Creates a new event log with a default capacity of 100 entries.
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::Selection.as_str(), "Selection");
        assert_eq!(EventType::Visibility.as_str(), "Visibility");
        assert_eq!(EventType::Lock.as_str(), "Lock");
        assert_eq!(EventType::DragDrop.as_str(), "DragDrop");
        assert_eq!(EventType::Rename.as_str(), "Rename");
        assert_eq!(EventType::Custom("Test".into()).as_str(), "Test");
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            "Test message".to_string(),
            EventType::Selection,
            Some(42u64),
        );

        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.event_type, EventType::Selection);
        assert_eq!(entry.node_id, Some(42));
    }

    #[test]
    fn test_log_entry_event_type_str() {
        let entry = LogEntry::<u64>::new("Test".into(), EventType::Rename, None);

        assert_eq!(entry.event_type_str(), "Rename");
    }

    #[test]
    fn test_event_log_new() {
        let log = EventLog::<u64>::new(10);
        assert_eq!(log.max_entries(), 10);
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_event_log_log() {
        let mut log = EventLog::new(10);

        log.log("Event 1", EventType::Selection, Some(1u64));
        assert_eq!(log.len(), 1);

        log.log("Event 2", EventType::Rename, Some(2u64));
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_event_log_max_capacity() {
        let mut log = EventLog::new(3);

        log.log("Event 1", EventType::Selection, Some(1u64));
        log.log("Event 2", EventType::Selection, Some(2u64));
        log.log("Event 3", EventType::Selection, Some(3u64));
        log.log("Event 4", EventType::Selection, Some(4u64));

        assert_eq!(log.len(), 3);

        // Most recent entries should be kept
        let entries: Vec<_> = log.entries().collect();
        assert_eq!(entries[0].message, "Event 4");
        assert_eq!(entries[1].message, "Event 3");
        assert_eq!(entries[2].message, "Event 2");
    }

    #[test]
    fn test_event_log_clear() {
        let mut log = EventLog::<u64>::new(10);

        log.log("Event 1", EventType::Selection, None);
        log.log("Event 2", EventType::Selection, None);
        assert_eq!(log.len(), 2);

        log.clear();
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());
    }

    #[test]
    fn test_event_log_set_max_entries() {
        let mut log = EventLog::new(10);

        for i in 0..10 {
            log.log(format!("Event {}", i), EventType::Selection, Some(i));
        }
        assert_eq!(log.len(), 10);

        log.set_max_entries(5);
        assert_eq!(log.len(), 5);
        assert_eq!(log.max_entries(), 5);

        // Most recent entries should be kept
        let entries: Vec<_> = log.entries().collect();
        assert_eq!(entries[0].node_id, Some(9));
        assert_eq!(entries[4].node_id, Some(5));
    }

    #[test]
    fn test_event_log_filter_by_type() {
        let mut log = EventLog::new(10);

        log.log("Select 1", EventType::Selection, Some(1u64));
        log.log("Rename 1", EventType::Rename, Some(2u64));
        log.log("Select 2", EventType::Selection, Some(3u64));
        log.log("Lock 1", EventType::Lock, Some(4u64));
        log.log("Select 3", EventType::Selection, Some(5u64));

        let selections: Vec<_> = log.filter_by_type(&EventType::Selection).collect();
        assert_eq!(selections.len(), 3);

        let renames: Vec<_> = log.filter_by_type(&EventType::Rename).collect();
        assert_eq!(renames.len(), 1);

        let visibility: Vec<_> = log.filter_by_type(&EventType::Visibility).collect();
        assert_eq!(visibility.len(), 0);
    }

    #[test]
    fn test_event_log_default() {
        let log = EventLog::<u64>::default();
        assert_eq!(log.max_entries(), 100);
        assert!(log.is_empty());
    }

    #[test]
    fn test_log_entry_format_elapsed() {
        let entry = LogEntry::<u64>::new("Test".into(), EventType::Selection, None);

        let formatted = entry.format_elapsed();
        assert!(formatted.ends_with("ago"));
    }
}
