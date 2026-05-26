//! Short-term working memory boundary.
//!
//! The current SQLite-backed blackboard and run-local working context helpers
//! still live at their historical call sites. This module is the stable import
//! target for the Phase 5b split as those helpers move out of `memory::mod`.
