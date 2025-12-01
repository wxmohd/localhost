use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Session data storage
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// Session data
    pub data: HashMap<String, String>,
    /// Creation time
    pub created_at: Instant,
    /// Last access time
    pub last_accessed: Instant,
    /// Session expiry duration
    pub expires_in: Duration,
}

impl Session {
    /// Creates a new session with the given ID
    pub fn new(id: &str) -> Self {
        let now = Instant::now();
        Session {
            id: id.to_string(),
            data: HashMap::new(),
            created_at: now,
            last_accessed: now,
            expires_in: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Gets a value from the session
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Sets a value in the session
    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
        self.last_accessed = Instant::now();
    }

    /// Removes a value from the session
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.last_accessed = Instant::now();
        self.data.remove(key)
    }

    /// Checks if the session has expired
    pub fn is_expired(&self) -> bool {
        self.last_accessed.elapsed() > self.expires_in
    }

    /// Touches the session (updates last accessed time)
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// In-memory session store
pub struct SessionStore {
    sessions: HashMap<String, Session>,
    /// Default session expiry
    default_expiry: Duration,
}

impl SessionStore {
    /// Creates a new session store
    pub fn new() -> Self {
        SessionStore {
            sessions: HashMap::new(),
            default_expiry: Duration::from_secs(3600),
        }
    }

    /// Creates a new session store with custom expiry
    pub fn with_expiry(expiry_seconds: u64) -> Self {
        SessionStore {
            sessions: HashMap::new(),
            default_expiry: Duration::from_secs(expiry_seconds),
        }
    }

    /// Generates a new session ID
    pub fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        
        // Simple ID generation (in production, use a proper random generator)
        format!("{:x}{:x}", timestamp, timestamp.wrapping_mul(0x5DEECE66D))
    }

    /// Creates a new session and returns its ID
    pub fn create(&mut self) -> String {
        let id = Self::generate_id();
        let mut session = Session::new(&id);
        session.expires_in = self.default_expiry;
        self.sessions.insert(id.clone(), session);
        id
    }

    /// Gets a session by ID
    pub fn get(&mut self, id: &str) -> Option<&Session> {
        // Clean up expired sessions occasionally
        self.cleanup_expired();
        
        if let Some(session) = self.sessions.get_mut(id) {
            if !session.is_expired() {
                session.touch();
                return self.sessions.get(id);
            } else {
                self.sessions.remove(id);
            }
        }
        None
    }

    /// Gets a mutable session by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Session> {
        if let Some(session) = self.sessions.get_mut(id) {
            if !session.is_expired() {
                session.touch();
                return self.sessions.get_mut(id);
            }
        }
        self.sessions.remove(id);
        None
    }

    /// Gets or creates a session
    pub fn get_or_create(&mut self, id: Option<&str>) -> &mut Session {
        let session_id = match id {
            Some(existing_id) if self.sessions.contains_key(existing_id) => {
                if let Some(session) = self.sessions.get(existing_id) {
                    if !session.is_expired() {
                        existing_id.to_string()
                    } else {
                        self.create()
                    }
                } else {
                    self.create()
                }
            }
            _ => self.create(),
        };

        self.sessions.get_mut(&session_id).unwrap()
    }

    /// Destroys a session
    pub fn destroy(&mut self, id: &str) {
        self.sessions.remove(id);
    }

    /// Cleans up expired sessions
    pub fn cleanup_expired(&mut self) {
        let expired: Vec<String> = self.sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired {
            self.sessions.remove(&id);
        }
    }

    /// Returns the number of active sessions
    pub fn count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
