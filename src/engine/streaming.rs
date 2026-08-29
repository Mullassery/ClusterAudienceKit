//! Real-time streaming segmentation engine

use crate::engine::clustering;
use crate::engine::drift_detection::{DriftDetector, DriftMethod, DriftSeverity, FeatureDrift};
use crate::{ClusterClusterAudienceKitError, Result};
use ndarray::Array2;
use std::collections::HashMap;

/// Streaming event types
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamEventType {
    Purchase,
    Engagement,
    PageView,
    Custom,
}

/// Customer streaming event
#[derive(Clone, Debug)]
pub struct StreamingEvent {
    pub customer_id: String,
    pub event_type: StreamEventType,
    pub value: f64,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

impl StreamingEvent {
    pub fn new(
        customer_id: String,
        event_type: StreamEventType,
        value: f64,
        timestamp: i64,
    ) -> Self {
        Self {
            customer_id,
            event_type,
            value,
            timestamp,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Incremental RFM state for streaming
#[derive(Clone, Debug)]
pub struct StreamingRFMState {
    pub customer_id: String,
    pub recency: i64,     // Days since last event
    pub frequency: usize, // Total event count
    pub monetary: f64,    // Total value
    pub last_event_timestamp: i64,
    pub event_count: usize,
    pub total_value: f64,
}

impl StreamingRFMState {
    pub fn new(customer_id: String) -> Self {
        Self {
            customer_id,
            recency: 0,
            frequency: 0,
            monetary: 0.0,
            last_event_timestamp: 0,
            event_count: 0,
            total_value: 0.0,
        }
    }

    pub fn update(&mut self, event: &StreamingEvent) {
        self.frequency += 1;
        self.monetary += event.value;
        self.last_event_timestamp = event.timestamp;
        self.event_count += 1;
        self.total_value += event.value;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.recency = (now - event.timestamp) / 86400; // Convert to days
    }

    pub fn get_rfm_score(&self) -> (f64, f64, f64) {
        let r = 100.0 - (self.recency as f64).min(100.0);
        let f = (self.frequency as f64).min(100.0);
        let m = (self.monetary / 1000.0).min(100.0); // Normalize by 1000

        (r, f, m)
    }
}

/// Streaming window for aggregation
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum StreamingWindow {
    Minute,
    Hour,
    Day,
    Week,
}

impl StreamingWindow {
    pub fn seconds(&self) -> i64 {
        match self {
            StreamingWindow::Minute => 60,
            StreamingWindow::Hour => 3600,
            StreamingWindow::Day => 86400,
            StreamingWindow::Week => 604800,
        }
    }
}

/// Streaming batch configuration
#[derive(Clone, Debug)]
pub struct StreamingConfig {
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub window: StreamingWindow,
    pub decay_factor: f64, // For exponential smoothing
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout_ms: 5000,
            window: StreamingWindow::Hour,
            decay_factor: 0.95,
        }
    }
}

/// Streaming buffer for batching events
#[derive(Clone, Debug)]
pub struct StreamingBuffer {
    pub events: Vec<StreamingEvent>,
    pub max_size: usize,
    pub last_flush: i64,
}

impl StreamingBuffer {
    pub fn new(max_size: usize) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            events: Vec::new(),
            max_size,
            last_flush: now,
        }
    }

    pub fn push(&mut self, event: StreamingEvent) -> Result<bool> {
        if self.events.len() >= self.max_size {
            return Ok(false);
        }

        self.events.push(event);
        Ok(true)
    }

    pub fn is_full(&self) -> bool {
        self.events.len() >= self.max_size
    }

    pub fn should_flush(&self, timeout_ms: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let last_flush_ms = self.last_flush * 1000;
        (now - last_flush_ms) > timeout_ms as i64
    }

    pub fn size(&self) -> usize {
        self.events.len()
    }

    pub fn flush(&mut self) -> Result<Vec<StreamingEvent>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.last_flush = now;
        let events = self.events.drain(..).collect();
        Ok(events)
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Streaming segment update
#[derive(Clone, Debug)]
pub struct StreamingSegmentUpdate {
    pub customer_id: String,
    pub previous_segment: Option<String>,
    pub new_segment: Option<String>,
    pub segment_changed: bool,
    pub confidence: f64,
    pub timestamp: i64,
}

/// Streaming aggregator
#[derive(Clone, Debug)]
pub struct StreamingAggregator;

impl StreamingAggregator {
    /// Update RFM state incrementally with streaming event
    pub fn update_rfm_state(state: &mut StreamingRFMState, event: &StreamingEvent) -> Result<()> {
        state.update(event);
        Ok(())
    }

    /// Classify segment from streaming RFM state
    pub fn classify_segment_from_streaming(state: &StreamingRFMState) -> Result<String> {
        let (r, f, m) = state.get_rfm_score();
        let avg_score = (r + f + m) / 3.0;

        let segment = if avg_score >= 80.0 {
            "Champions"
        } else if avg_score >= 60.0 {
            "Loyal Customers"
        } else if avg_score >= 40.0 {
            "Potential Loyalists"
        } else if avg_score >= 20.0 {
            "At Risk"
        } else {
            "Lost"
        };

        Ok(segment.to_string())
    }

    /// Aggregate events by customer
    pub fn aggregate_by_customer(
        events: &[StreamingEvent],
    ) -> Result<HashMap<String, Vec<StreamingEvent>>> {
        let mut aggregated = HashMap::new();

        for event in events {
            aggregated
                .entry(event.customer_id.clone())
                .or_insert_with(Vec::new)
                .push(event.clone());
        }

        Ok(aggregated)
    }

    /// Calculate event rate (events per time window)
    pub fn calculate_event_rate(
        events: &[StreamingEvent],
        window_seconds: i64,
    ) -> Result<HashMap<String, f64>> {
        let mut rates = HashMap::new();

        if events.is_empty() {
            return Ok(rates);
        }

        let max_timestamp = events.iter().map(|e| e.timestamp).max().unwrap_or(0);
        let min_timestamp = events.iter().map(|e| e.timestamp).min().unwrap_or(0);
        let actual_window = ((max_timestamp - min_timestamp).max(1)) as f64 / 1000.0; // Convert to seconds

        for event in events {
            let rate = 1.0 / (actual_window / window_seconds as f64);
            rates.insert(event.customer_id.clone(), rate);
        }

        Ok(rates)
    }

    /// Detect high-value customers (exponential smoothing)
    pub fn detect_high_value(states: &[StreamingRFMState], threshold: f64) -> Result<Vec<String>> {
        Ok(states
            .iter()
            .filter(|s| s.monetary > threshold)
            .map(|s| s.customer_id.clone())
            .collect())
    }

    /// Detect churn signals from streaming
    pub fn detect_churn_signals(
        states: &[StreamingRFMState],
        recency_threshold_days: i64,
    ) -> Result<Vec<String>> {
        Ok(states
            .iter()
            .filter(|s| s.recency > recency_threshold_days)
            .map(|s| s.customer_id.clone())
            .collect())
    }

    /// Calculate windowed statistics
    pub fn calculate_window_stats(
        events: &[StreamingEvent],
        _window_seconds: i64,
    ) -> Result<HashMap<String, f64>> {
        let mut stats = HashMap::new();

        if events.is_empty() {
            return Ok(stats);
        }

        let total_value: f64 = events.iter().map(|e| e.value).sum();
        let event_count = events.len() as f64;

        stats.insert("total_value".to_string(), total_value);
        stats.insert("event_count".to_string(), event_count);
        stats.insert("average_value".to_string(), total_value / event_count);
        stats.insert(
            "max_value".to_string(),
            events
                .iter()
                .map(|e| e.value)
                .fold(f64::NEG_INFINITY, f64::max),
        );
        stats.insert(
            "min_value".to_string(),
            events.iter().map(|e| e.value).fold(f64::INFINITY, f64::min),
        );

        Ok(stats)
    }

    /// Apply exponential smoothing to metric
    pub fn exponential_smoothing(current_value: f64, new_value: f64, alpha: f64) -> Result<f64> {
        Ok(alpha * new_value + (1.0 - alpha) * current_value)
    }
}

/// Snapshot of the tracked population's RFM feature distributions, used as
/// the drift-detection baseline. Captured when re-clustering is first armed
/// and re-captured after every successful re-cluster, so each `check_drift`
/// call compares "now" against "as of the last time we adapted," not
/// against some fixed point at engine startup.
#[derive(Clone, Debug)]
struct RfmSnapshot {
    recency: Vec<f64>,
    frequency: Vec<f64>,
    monetary: Vec<f64>,
}

impl RfmSnapshot {
    fn capture(states: &HashMap<String, StreamingRFMState>) -> Self {
        Self {
            recency: states.values().map(|s| s.recency as f64).collect(),
            frequency: states.values().map(|s| s.frequency as f64).collect(),
            monetary: states.values().map(|s| s.monetary).collect(),
        }
    }
}

/// Configuration for drift-triggered re-clustering.
#[derive(Clone, Debug)]
pub struct ReclusterConfig {
    /// Minimum severity (on any of recency/frequency/monetary) that
    /// triggers a re-cluster.
    pub min_severity: DriftSeverity,
    /// Minimum tracked customers before drift is even evaluated -- a
    /// KS-test/Hellinger-distance result on a handful of points is noise,
    /// not signal.
    pub min_customers: usize,
    /// Number of clusters for the re-fit (capped at the tracked customer
    /// count if fewer customers than this are currently tracked).
    pub n_clusters: usize,
}

impl Default for ReclusterConfig {
    fn default() -> Self {
        Self {
            min_severity: DriftSeverity::High,
            min_customers: 30,
            n_clusters: 4,
        }
    }
}

/// Record of a completed drift-triggered re-cluster.
#[derive(Clone, Debug)]
pub struct ReclusterEvent {
    pub timestamp: i64,
    pub drifts: Vec<FeatureDrift>,
    pub customers_reclustered: usize,
    pub n_clusters: usize,
}

/// Streaming engine for real-time segmentation
pub struct StreamingSegmentationEngine {
    config: StreamingConfig,
    buffer: StreamingBuffer,
    rfm_states: HashMap<String, StreamingRFMState>,
    segment_assignments: HashMap<String, String>,
    recluster_config: ReclusterConfig,
    drift_baseline: Option<RfmSnapshot>,
    recluster_history: Vec<ReclusterEvent>,
}

impl StreamingSegmentationEngine {
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config: config.clone(),
            buffer: StreamingBuffer::new(config.batch_size),
            rfm_states: HashMap::new(),
            segment_assignments: HashMap::new(),
            recluster_config: ReclusterConfig::default(),
            drift_baseline: None,
            recluster_history: Vec::new(),
        }
    }

    /// Override the default drift-triggered re-clustering configuration.
    pub fn with_recluster_config(mut self, recluster_config: ReclusterConfig) -> Self {
        self.recluster_config = recluster_config;
        self
    }

    /// Process single event
    pub fn process_event(
        &mut self,
        event: StreamingEvent,
    ) -> Result<Option<StreamingSegmentUpdate>> {
        let pushed = self.buffer.push(event.clone())?;

        if !pushed {
            return Ok(None);
        }

        let rfm_state = self
            .rfm_states
            .entry(event.customer_id.clone())
            .or_insert_with(|| StreamingRFMState::new(event.customer_id.clone()));

        StreamingAggregator::update_rfm_state(rfm_state, &event)?;

        let new_segment = StreamingAggregator::classify_segment_from_streaming(rfm_state)?;
        let previous_segment = self.segment_assignments.get(&event.customer_id).cloned();

        let segment_changed = previous_segment != Some(new_segment.clone());

        if segment_changed {
            self.segment_assignments
                .insert(event.customer_id.clone(), new_segment.clone());
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Ok(Some(StreamingSegmentUpdate {
            customer_id: event.customer_id,
            previous_segment,
            new_segment: Some(new_segment),
            segment_changed,
            confidence: 0.85,
            timestamp: now,
        }))
    }

    /// Process batch of events. On top of the per-event RFM/segment update,
    /// this is the checkpoint where drift-triggered re-clustering is wired
    /// in: once enough customers are tracked, the first batch arms the
    /// drift baseline, and every batch after that checks the current RFM
    /// distribution against it (see `maybe_recluster`). Checking at the
    /// batch boundary -- not per-event -- keeps this cheap: a KS-test plus
    /// a potential k-means re-fit is O(n) to O(n * k * iterations), not
    /// something to redo on every single incoming event.
    pub fn process_batch(
        &mut self,
        events: Vec<StreamingEvent>,
    ) -> Result<Vec<StreamingSegmentUpdate>> {
        let mut updates = Vec::new();

        for event in events {
            if let Some(update) = self.process_event(event)? {
                updates.push(update);
            }
        }

        if self.drift_baseline.is_none() {
            if self.rfm_states.len() >= self.recluster_config.min_customers {
                self.set_recluster_baseline();
            }
        } else {
            self.maybe_recluster()?;
        }

        Ok(updates)
    }

    /// (Re-)arm drift detection against the current RFM distribution,
    /// without performing a re-cluster. `process_batch` calls this
    /// automatically the first time enough customers accumulate; call it
    /// directly to force a fresh baseline (e.g. after a known, deliberate
    /// population shift you don't want flagged as drift).
    pub fn set_recluster_baseline(&mut self) {
        self.drift_baseline = Some(RfmSnapshot::capture(&self.rfm_states));
    }

    /// Compare the current RFM distribution against the last-captured
    /// baseline using `DriftDetector`. Returns `None` (not an error) if no
    /// baseline has been armed yet, or if there aren't enough tracked
    /// customers for the comparison to be meaningful.
    pub fn check_drift(&self) -> Result<Option<Vec<FeatureDrift>>> {
        let Some(baseline) = &self.drift_baseline else {
            return Ok(None);
        };
        if self.rfm_states.len() < self.recluster_config.min_customers {
            return Ok(None);
        }

        let current = RfmSnapshot::capture(&self.rfm_states);

        let recency_drift = DriftDetector::detect_feature_drift(
            "recency".to_string(),
            &baseline.recency,
            &current.recency,
            DriftMethod::KolmogorovSmirnov,
        )?;
        let frequency_drift = DriftDetector::detect_feature_drift(
            "frequency".to_string(),
            &baseline.frequency,
            &current.frequency,
            DriftMethod::KolmogorovSmirnov,
        )?;
        // Monetary values are heavy-tailed (a handful of big spenders), so
        // Hellinger distance -- which compares fitted mean/variance rather
        // than the raw empirical CDF KS uses -- is the better-behaved
        // signal here.
        let monetary_drift = DriftDetector::detect_feature_drift(
            "monetary".to_string(),
            &baseline.monetary,
            &current.monetary,
            DriftMethod::HellingerDistance,
        )?;

        Ok(Some(vec![recency_drift, frequency_drift, monetary_drift]))
    }

    /// Check for drift against the current baseline and, if any tracked
    /// RFM feature has drifted at or beyond `recluster_config.min_severity`,
    /// re-fit real k-means clusters (`clustering::kmeans`) over every
    /// currently-tracked customer's (recency, frequency, monetary) vector
    /// and overwrite their segment assignments with the freshly computed
    /// cluster labels (`"cluster_0"`, `"cluster_1"`, ...). Re-arms the
    /// baseline to the post-recluster distribution on a successful trigger,
    /// so the next check compares against the new normal rather than
    /// immediately re-triggering on the same drift.
    ///
    /// Returns `Ok(None)` if no baseline is armed yet, there aren't enough
    /// customers, or no feature drifted past the threshold -- re-clustering
    /// is comparatively expensive, so this is deliberately conservative
    /// about when it actually fires.
    pub fn maybe_recluster(&mut self) -> Result<Option<ReclusterEvent>> {
        let Some(drifts) = self.check_drift()? else {
            return Ok(None);
        };

        let triggered = DriftDetector::get_drifted_features(
            &drifts,
            self.recluster_config.min_severity.clone(),
        );
        if triggered.is_empty() {
            return Ok(None);
        }

        let n_clusters = self
            .recluster_config
            .n_clusters
            .min(self.rfm_states.len().max(1));

        let customer_ids: Vec<String> = self.rfm_states.keys().cloned().collect();
        let rows: Vec<f64> = customer_ids
            .iter()
            .flat_map(|id| {
                let s = &self.rfm_states[id];
                [s.recency as f64, s.frequency as f64, s.monetary]
            })
            .collect();
        let data = Array2::from_shape_vec((customer_ids.len(), 3), rows).map_err(|e| {
            ClusterClusterAudienceKitError::DataValidation(format!(
                "failed to build RFM matrix for re-clustering: {e}"
            ))
        })?;

        let result = clustering::kmeans(&data, n_clusters, 300, 42, -1)?;

        for (customer_id, label) in customer_ids.iter().zip(result.labels.iter()) {
            self.segment_assignments
                .insert(customer_id.clone(), format!("cluster_{label}"));
        }

        self.drift_baseline = Some(RfmSnapshot::capture(&self.rfm_states));

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let event = ReclusterEvent {
            timestamp: now,
            drifts,
            customers_reclustered: customer_ids.len(),
            n_clusters,
        };
        self.recluster_history.push(event.clone());

        Ok(Some(event))
    }

    /// History of drift-triggered re-clusters this engine has performed.
    pub fn recluster_history(&self) -> &[ReclusterEvent] {
        &self.recluster_history
    }

    /// Flush buffer and return pending events
    pub fn flush_buffer(&mut self) -> Result<Vec<StreamingEvent>> {
        self.buffer.flush()
    }

    /// Get current segment for customer
    pub fn get_segment(&self, customer_id: &str) -> Option<String> {
        self.segment_assignments.get(customer_id).cloned()
    }

    /// Get RFM state for customer
    pub fn get_rfm_state(&self, customer_id: &str) -> Option<StreamingRFMState> {
        self.rfm_states.get(customer_id).cloned()
    }

    /// Get all segment assignments
    pub fn get_all_segments(&self) -> HashMap<String, String> {
        self.segment_assignments.clone()
    }

    /// Customer count
    pub fn customer_count(&self) -> usize {
        self.segment_assignments.len()
    }

    /// Buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.size()
    }

    /// The batching/windowing configuration this engine was constructed
    /// with (batch size, flush timeout, aggregation window, decay factor).
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }

    /// Segment distribution
    pub fn segment_distribution(&self) -> Result<HashMap<String, usize>> {
        let mut distribution = HashMap::new();

        for segment in self.segment_assignments.values() {
            *distribution.entry(segment.clone()).or_insert(0) += 1;
        }

        Ok(distribution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_event_creation() {
        let event = StreamingEvent::new(
            "cust_123".to_string(),
            StreamEventType::Purchase,
            100.0,
            1704067200,
        );

        assert_eq!(event.customer_id, "cust_123");
        assert_eq!(event.value, 100.0);
    }

    #[test]
    fn test_streaming_rfm_state_update() {
        let mut state = StreamingRFMState::new("cust_123".to_string());
        let event = StreamingEvent::new(
            "cust_123".to_string(),
            StreamEventType::Purchase,
            100.0,
            1704067200,
        );

        state.update(&event);
        assert_eq!(state.frequency, 1);
        assert_eq!(state.monetary, 100.0);
    }

    #[test]
    fn test_streaming_buffer() {
        let mut buffer = StreamingBuffer::new(2);
        let event1 = StreamingEvent::new(
            "cust_1".to_string(),
            StreamEventType::Purchase,
            100.0,
            1704067200,
        );
        let event2 = StreamingEvent::new(
            "cust_2".to_string(),
            StreamEventType::Purchase,
            200.0,
            1704067200,
        );

        assert!(buffer.push(event1).unwrap());
        assert!(buffer.push(event2).unwrap());
        assert!(buffer.is_full());
    }

    #[test]
    fn test_streaming_window() {
        assert_eq!(StreamingWindow::Minute.seconds(), 60);
        assert_eq!(StreamingWindow::Hour.seconds(), 3600);
        assert_eq!(StreamingWindow::Day.seconds(), 86400);
    }

    #[test]
    fn test_classify_segment_from_streaming() {
        let mut state = StreamingRFMState::new("cust_123".to_string());
        state.monetary = 1000.0;
        state.frequency = 10;

        let segment = StreamingAggregator::classify_segment_from_streaming(&state).unwrap();
        assert!(!segment.is_empty());
    }

    #[test]
    fn test_aggregate_by_customer() {
        let events = vec![
            StreamingEvent::new(
                "cust_1".to_string(),
                StreamEventType::Purchase,
                100.0,
                1704067200,
            ),
            StreamingEvent::new(
                "cust_1".to_string(),
                StreamEventType::Purchase,
                200.0,
                1704067300,
            ),
            StreamingEvent::new(
                "cust_2".to_string(),
                StreamEventType::Purchase,
                150.0,
                1704067400,
            ),
        ];

        let aggregated = StreamingAggregator::aggregate_by_customer(&events).unwrap();

        assert_eq!(aggregated.get("cust_1").map(|v| v.len()), Some(2));
        assert_eq!(aggregated.get("cust_2").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_detect_high_value() {
        let states = vec![
            StreamingRFMState {
                customer_id: "cust_1".to_string(),
                recency: 0,
                frequency: 10,
                monetary: 5000.0,
                last_event_timestamp: 0,
                event_count: 10,
                total_value: 5000.0,
            },
            StreamingRFMState {
                customer_id: "cust_2".to_string(),
                recency: 0,
                frequency: 2,
                monetary: 100.0,
                last_event_timestamp: 0,
                event_count: 2,
                total_value: 100.0,
            },
        ];

        let high_value = StreamingAggregator::detect_high_value(&states, 1000.0).unwrap();
        assert_eq!(high_value.len(), 1);
        assert_eq!(high_value[0], "cust_1");
    }

    #[test]
    fn test_detect_churn_signals() {
        let states = vec![
            StreamingRFMState {
                customer_id: "cust_1".to_string(),
                recency: 100,
                frequency: 10,
                monetary: 1000.0,
                last_event_timestamp: 0,
                event_count: 10,
                total_value: 1000.0,
            },
            StreamingRFMState {
                customer_id: "cust_2".to_string(),
                recency: 5,
                frequency: 2,
                monetary: 100.0,
                last_event_timestamp: 0,
                event_count: 2,
                total_value: 100.0,
            },
        ];

        let churned = StreamingAggregator::detect_churn_signals(&states, 30).unwrap();
        assert_eq!(churned.len(), 1);
        assert_eq!(churned[0], "cust_1");
    }

    #[test]
    fn test_exponential_smoothing() {
        let result = StreamingAggregator::exponential_smoothing(100.0, 120.0, 0.5).unwrap();
        assert_eq!(result, 110.0);
    }

    #[test]
    fn test_streaming_engine_process_event() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default());
        let event = StreamingEvent::new(
            "cust_123".to_string(),
            StreamEventType::Purchase,
            100.0,
            1704067200,
        );

        let update = engine.process_event(event).unwrap();
        assert!(update.is_some());
    }

    #[test]
    fn test_streaming_engine_segment_assignment() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default());
        let event = StreamingEvent::new(
            "cust_123".to_string(),
            StreamEventType::Purchase,
            100.0,
            1704067200,
        );

        engine.process_event(event).unwrap();
        let segment = engine.get_segment("cust_123");
        assert!(segment.is_some());
    }

    #[test]
    fn test_streaming_engine_segment_distribution() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default());

        let event1 = StreamingEvent::new(
            "cust_1".to_string(),
            StreamEventType::Purchase,
            100.0,
            1704067200,
        );
        let event2 = StreamingEvent::new(
            "cust_2".to_string(),
            StreamEventType::Purchase,
            100.0,
            1704067200,
        );

        engine.process_event(event1).unwrap();
        engine.process_event(event2).unwrap();

        let dist = engine.segment_distribution().unwrap();
        assert!(!dist.is_empty());
    }

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.batch_timeout_ms, 5000);
    }

    #[test]
    fn test_streaming_window_stats() {
        let events = vec![
            StreamingEvent::new(
                "cust_1".to_string(),
                StreamEventType::Purchase,
                100.0,
                1704067200,
            ),
            StreamingEvent::new(
                "cust_1".to_string(),
                StreamEventType::Purchase,
                200.0,
                1704067200,
            ),
            StreamingEvent::new(
                "cust_1".to_string(),
                StreamEventType::Purchase,
                50.0,
                1704067200,
            ),
        ];

        let stats = StreamingAggregator::calculate_window_stats(&events, 3600).unwrap();
        assert_eq!(stats.get("total_value"), Some(&350.0));
        assert_eq!(stats.get("event_count"), Some(&3.0));
    }

    fn purchase_events(customer_prefix: &str, count: usize, value: f64) -> Vec<StreamingEvent> {
        (0..count)
            .map(|i| {
                StreamingEvent::new(
                    format!("{customer_prefix}_{i}"),
                    StreamEventType::Purchase,
                    value,
                    1704067200 + i as i64,
                )
            })
            .collect()
    }

    fn low_customer_threshold_config() -> ReclusterConfig {
        ReclusterConfig {
            min_severity: DriftSeverity::Medium,
            min_customers: 6,
            n_clusters: 2,
        }
    }

    #[test]
    fn test_check_drift_none_before_any_baseline() {
        let engine = StreamingSegmentationEngine::new(StreamingConfig::default())
            .with_recluster_config(low_customer_threshold_config());
        assert!(engine.check_drift().unwrap().is_none());
    }

    #[test]
    fn test_check_drift_none_below_min_customers() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default())
            .with_recluster_config(low_customer_threshold_config());
        engine
            .process_batch(purchase_events("cust", 3, 10.0))
            .unwrap();

        // Fewer than min_customers (6) tracked, so process_batch shouldn't
        // have armed a baseline yet, and check_drift must stay None rather
        // than compare against a statistically meaningless sample.
        assert!(engine.check_drift().unwrap().is_none());
    }

    #[test]
    fn test_process_batch_arms_baseline_once_enough_customers_and_finds_no_drift_against_itself() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default())
            .with_recluster_config(low_customer_threshold_config());
        engine
            .process_batch(purchase_events("cust", 6, 10.0))
            .unwrap();

        // Baseline should now be armed (>= min_customers), and comparing
        // the same distribution against itself should show no meaningful
        // drift, so no re-cluster should have happened.
        let drifts = engine.check_drift().unwrap();
        assert!(drifts.is_some());
        assert!(engine.recluster_history().is_empty());
    }

    #[test]
    fn test_maybe_recluster_no_trigger_on_stable_distribution() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default())
            .with_recluster_config(low_customer_threshold_config());
        engine
            .process_batch(purchase_events("cust", 6, 10.0))
            .unwrap();

        let result = engine.maybe_recluster().unwrap();
        assert!(result.is_none());
        assert!(engine.recluster_history().is_empty());
    }

    #[test]
    fn test_maybe_recluster_triggers_on_drastic_monetary_shift() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default())
            .with_recluster_config(low_customer_threshold_config());

        // Baseline: 6 customers each spending ~10.
        engine
            .process_batch(purchase_events("cust", 6, 10.0))
            .unwrap();
        assert!(engine.recluster_history().is_empty());

        // New customers with drastically larger purchase values shift the
        // tracked population's monetary distribution hard enough to be real
        // drift, not noise -- this should both detect it and trigger a
        // genuine k-means re-fit over the RFM vectors of every tracked
        // customer (old and new).
        engine
            .process_batch(purchase_events("whale", 6, 5000.0))
            .unwrap();

        assert_eq!(engine.recluster_history().len(), 1);
        let event = &engine.recluster_history()[0];
        assert_eq!(event.customers_reclustered, 12);
        assert_eq!(event.n_clusters, 2);

        // Every tracked customer's segment should now be a real cluster
        // label, not the old fixed RFM-threshold segment name.
        for segment in engine.get_all_segments().values() {
            assert!(segment.starts_with("cluster_"));
        }
    }

    #[test]
    fn test_recluster_rearms_baseline_so_it_does_not_immediately_refire() {
        let mut engine = StreamingSegmentationEngine::new(StreamingConfig::default())
            .with_recluster_config(low_customer_threshold_config());

        engine
            .process_batch(purchase_events("cust", 6, 10.0))
            .unwrap();
        engine
            .process_batch(purchase_events("whale", 6, 5000.0))
            .unwrap();
        assert_eq!(engine.recluster_history().len(), 1);

        // Checking again immediately (no new events) compares the current
        // distribution against the just-recaptured baseline, so it should
        // find no further drift and must not trigger a second re-cluster.
        let result = engine.maybe_recluster().unwrap();
        assert!(result.is_none());
        assert_eq!(engine.recluster_history().len(), 1);
    }
}
