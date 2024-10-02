use bon::Builder;
use prometheus::{
    core::{AtomicU64, GenericCounter},
    Histogram, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry,
};

#[derive(Clone, Metronom)]
pub struct Metrics {
    #[metronom(
        name="honeycomb_ctrl_reconciliations_total"
        help="Total number of reconciliations attempted"
        labels=["target", "kind"]
    )]
    reconciliations: IntCounterVec,

    #[metronom(
        name="honeycomb_ctrl_updates_total"
        help="Total number of hc api updates"
        labels=["target", "kind", "action"]
    )]
    updates: IntCounterVec,

    #[metronom(
        name = "honeycomb_ctrl_reconciliations_errors_total"
        help = "Total number of reconciliation errors"
        labels = ["target", "kind", "error"]
    )]
    errors: IntCounterVec,

    #[metronom(
        name = "honeycomb_ctrl_reconciliations_duration_seconds",
        help = "The duration of reconcile to complete in seconds"
        buckets = [0.01, 0.1, 0.25, 0.5, 1., 5., 15., 60.]
    )]
    reconcile_duration: HistogramVec,
}

trait LabelValues {
    fn values(&self) -> Vec<&str>;
}

#[derive(Builder)]
struct ReconciliationsLabels {
    target: String,
    kind: String,
}

impl LabelValues for ReconciliationsLabels {
    fn values(&self) -> Vec<&str> {
        vec![&self.target, &self.kind]
    }
}

#[derive(Builder)]
struct UpdatesLabels {
    target: String,
    kind: String,
    action: String,
}

impl LabelValues for UpdatesLabels {
    fn values(&self) -> Vec<&str> {
        vec![&self.target, &self.kind, &self.action]
    }
}

#[derive(Builder)]
struct ErrorsLabels {
    target: String,
    kind: String,
    error: String,
}

impl LabelValues for ErrorsLabels {
    fn values(&self) -> Vec<&str> {
        vec![&self.target, &self.kind, &self.error]
    }
}

#[derive(Builder)]
struct ReconcileDurationLabels {}

impl LabelValues for ReconcileDurationLabels {
    fn values(&self) -> Vec<&str> {
        vec![]
    }
}

impl Metrics {
    fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let reconciliations = IntCounterVec::new(
            Opts::new(
                "honeycomb_ctrl_reconciliations_total",
                "Total number of reconciliations attempted",
            ),
            &["target", "kind"],
        )
        .unwrap();

        let updates = IntCounterVec::new(
            Opts::new(
                "honeycomb_ctrl_updates_total",
                "Total number of hc api updates",
            ),
            &["target", "kind", "action"],
        )
        .unwrap();

        let errors = IntCounterVec::new(
            Opts::new(
                "honeycomb_ctrl_reconciliations_errors_total",
                "Total number of reconciliation errors",
            ),
            &["target", "kind", "error"],
        )
        .unwrap();

        let reconcile_duration = HistogramVec::new(
            HistogramOpts::new(
                "honeycomb_ctrl_reconciliations_duration_seconds",
                "The duration of reconcile to complete in seconds",
            )
            .buckets(vec![0.01, 0.1, 0.25, 0.5, 1., 5., 15., 60.]),
            &[],
        )
        .unwrap();

        let metrics = Self {
            reconciliations,
            updates,
            errors,
            reconcile_duration,
        };

        registry.register(Box::new(metrics.reconciliations.clone()))?;
        registry.register(Box::new(metrics.updates.clone()))?;
        registry.register(Box::new(metrics.errors.clone()))?;
        registry.register(Box::new(metrics.reconcile_duration.clone()))?;

        Ok(metrics)
    }

    fn reconciliations_with(&self, values: ReconciliationsLabels) -> GenericCounter<AtomicU64> {
        self.reconciliations.with_label_values(&values.values())
    }

    fn updates_with(&self, values: UpdatesLabels) -> GenericCounter<AtomicU64> {
        self.updates.with_label_values(&values.values())
    }

    fn errors_with(&self, values: ErrorsLabels) -> GenericCounter<AtomicU64> {
        self.updates.with_label_values(&values.values())
    }

    fn reconcile_duration_with(&self, values: ReconcileDurationLabels) -> Histogram {
        self.reconcile_duration.with_label_values(&values.values())
    }
}
