use std::{
    io,
    time::Duration,
};

use crate::event::{
    filter::Filter,
    internal::{self, InternalEvent},
};

/// A terminal capability query that can be issued via [`QueryBatch`].
pub(crate) trait TerminalQuery: Clone + Send + Sync + 'static {
    type Response;
    fn query_bytes(&self) -> Vec<u8>;
    fn matches(&self, event: &InternalEvent) -> bool;
    fn extract(&self, event: Option<InternalEvent>) -> io::Result<Self::Response>;
}

/// A typed handle to retrieve one query's result from a [`QueryResults`].
///
/// Obtained by calling [`QueryBatch::add`].
pub struct QueryHandle<T> {
    idx: usize,
    extract: Box<dyn Fn(Option<InternalEvent>) -> io::Result<T>>,
}

/// Results returned by [`QueryBatch::execute`].
pub struct QueryResults {
    results: Vec<Option<InternalEvent>>,
}

impl QueryResults {
    /// Extracts the response for the given handle.
    pub fn get<T>(&self, handle: &QueryHandle<T>) -> io::Result<T> {
        (handle.extract)(self.results[handle.idx].clone())
    }
}

struct BatchFilter {
    matchers: Vec<Box<dyn Fn(&InternalEvent) -> bool + Send + Sync>>,
}

impl Filter for BatchFilter {
    fn eval(&self, event: &InternalEvent) -> bool {
        // Always pass DA1 — it's the sentinel that ends the batch read loop.
        matches!(event, InternalEvent::PrimaryDeviceAttributes(_))
            || self.matchers.iter().any(|m| m(event))
    }
}

/// Sends multiple terminal queries in one write and collects all responses.
///
/// All query bytes are written together followed by a single DA1 sentinel
/// (`ESC [ c`). Responses are collected until the DA1 reply arrives.
///
/// # Example
///
/// ```no_run
/// # #[cfg(unix)] {
/// use crossterm::graphics::{KittyQuery, SixelQuery};
/// use crossterm::query::QueryBatch;
///
/// let mut batch = QueryBatch::new();
/// let kitty = batch.add(KittyQuery);
/// let sixel = batch.add(SixelQuery);
/// let results = batch.execute()?;
/// println!("kitty={} sixel={}", results.get(&kitty)?, results.get(&sixel)?);
/// # }
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct QueryBatch {
    bytes: Vec<Vec<u8>>,
    matchers: Vec<Box<dyn Fn(&InternalEvent) -> bool + Send + Sync>>,
    results: Vec<Option<InternalEvent>>,
}

impl Default for QueryBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryBatch {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            matchers: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Registers a query and returns a handle for retrieving its result.
    #[allow(private_bounds, private_interfaces)]
    pub fn add<Q: TerminalQuery>(&mut self, query: Q) -> QueryHandle<Q::Response> {
        let idx = self.bytes.len();
        let q = query.clone();
        self.bytes.push(query.query_bytes());
        self.matchers.push(Box::new(move |e| q.matches(e)));
        self.results.push(None);
        QueryHandle {
            idx,
            extract: Box::new(move |e| query.extract(e)),
        }
    }

    /// Writes all queries plus a DA1 sentinel, then reads until the DA1 reply
    /// arrives and returns the collected results.
    pub fn execute(mut self) -> io::Result<QueryResults> {
        let filter = BatchFilter { matchers: self.matchers };

        // Drain any stale events from prior queries.
        while internal::poll(Some(Duration::ZERO), &filter)? {
            internal::read(&filter)?;
        }

        let mut bytes: Vec<u8> = self.bytes.into_iter().flatten().collect();
        bytes.extend_from_slice(b"\x1B[c"); // DA1 sentinel
        crate::event::write_query(&bytes)?;

        let timeout = Duration::from_secs(2);
        loop {
            if !internal::poll(Some(timeout), &filter)? {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "terminal query timed out",
                ));
            }
            let event = internal::read(&filter)?;
            let is_da1 = matches!(event, InternalEvent::PrimaryDeviceAttributes(_));
            for (matcher, result) in filter.matchers.iter().zip(self.results.iter_mut()) {
                if matcher(&event) && result.is_none() {
                    *result = Some(event.clone());
                }
            }
            if is_da1 {
                break;
            }
        }

        Ok(QueryResults { results: self.results })
    }
}
