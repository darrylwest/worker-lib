/// concrete implementation for cache-like operations.
/// use cases:
/// * in-memory spot-cache for storing user/communication sessions
/// * in-memory spot-cache for storing semi-static configuration settings
/// * in-memory spat-cache for storing critical errors and state
/// * long-term cache connected to a backing source, e.g., redis, filesystem, etc
///
/// The main advantage that this supervisor/worker provides is a common interface to
/// various cache requirements.  Tbink of it as a Level 1 application cache similar
/// to CPUs: level 1 is closest to the app, and the fastestest.  Level 2 is two
/// steps away, e.g., hosted Redis and Level 3 is a SQL or Mongo hosted database.
///
pub mod supervisor;
pub mod worker;
