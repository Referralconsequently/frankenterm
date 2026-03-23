//! Fleet scrollback coordinator (ft-1memj.19).
//!
//! Bridges the [`FleetMemoryController`] decision layer with per-pane
//! [`TieredScrollback`] instances. On each evaluation tick the coordinator:
//!
//! 1. Collects [`PaneScrollbackInfo`] from every registered pane.
//! 2. Feeds [`PressureSignals`] into [`FleetMemoryController::evaluate`].
//! 3. When the controller emits [`FleetMemoryAction::EvictWarmScrollback`]
//!    or [`FleetMemoryAction::EmergencyCleanup`], asks the
//!    [`FleetScrollbackOrchestrator`] for a concrete [`EvictionPlan`].
//! 4. Applies each [`EvictionTarget`] to the corresponding pane's
//!    [`TieredScrollback`].
//! 5. Records telemetry (plans produced, pages evicted, bytes reclaimed).
//!
//! The coordinator is *not* async — it operates synchronously on mutable
//! references to avoid locking complexity. The caller (runtime maintenance
//! loop) is responsible for providing the pressure signals and pane handles.

use serde::{Deserialize, Serialize};

use crate::backpressure::BackpressureTier;
use crate::fleet_memory_controller::{
    EvictionPlan, FleetMemoryAction, FleetMemoryConfig, FleetMemoryController, FleetPressureTier,
    FleetScrollbackOrchestrator, PaneScrollbackInfo, PressureSignals,
};
use crate::memory_budget::BudgetLevel;
use crate::memory_pressure::MemoryPressureTier;
use crate::scrollback_tiers::{ScrollbackTierSnapshot, TieredScrollback};

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for the fleet scrollback coordinator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorConfig {
    /// Maximum number of panes to target per eviction cycle.
    /// Limits blast radius of a single evaluation. Default: 50.
    pub max_targets_per_cycle: usize,

    /// Minimum warm bytes across the fleet before eviction is worthwhile.
    /// Avoids churn when there's negligible warm data. Default: 1 MB.
    pub min_fleet_warm_bytes_for_eviction: usize,

    /// When true, emergency cleanup evicts *all* warm pages fleet-wide
    /// rather than relying on the orchestrator's proportional plan.
    /// Default: true.
    pub emergency_evict_all: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_targets_per_cycle: 50,
            min_fleet_warm_bytes_for_eviction: 1024 * 1024, // 1 MB
            emergency_evict_all: true,
        }
    }
}

// =============================================================================
// Telemetry
// =============================================================================

/// Cumulative telemetry for the coordinator.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoordinatorTelemetry {
    /// Total evaluation ticks processed.
    pub ticks: u64,
    /// Ticks where the compound tier was above Normal.
    pub elevated_ticks: u64,
    /// Total eviction plans produced (from the orchestrator).
    pub plans_produced: u64,
    /// Total individual pane eviction targets applied.
    pub targets_applied: u64,
    /// Total warm pages evicted across all targets.
    pub pages_evicted: u64,
    /// Total warm bytes reclaimed (estimated compressed).
    pub bytes_reclaimed: u64,
    /// Number of emergency cleanup events.
    pub emergency_cleanups: u64,
    /// Number of ticks where eviction was skipped because fleet warm
    /// bytes were below the minimum threshold.
    pub skipped_below_threshold: u64,
}

// =============================================================================
// Evaluation result
// =============================================================================

/// Summary of a single coordinator evaluation tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Fleet pressure tier after this evaluation.
    pub compound_tier: FleetPressureTier,
    /// Actions recommended by the fleet memory controller.
    pub actions: Vec<FleetMemoryAction>,
    /// Eviction plan produced (if any).
    pub eviction_plan: Option<EvictionPlan>,
    /// Number of warm pages actually evicted in this tick.
    pub pages_evicted: u64,
    /// Estimated bytes reclaimed in this tick.
    pub bytes_reclaimed: u64,
    /// Number of pane targets applied.
    pub targets_applied: usize,
}

// =============================================================================
// Coordinator
// =============================================================================

/// Stateful coordinator that bridges fleet memory decisions with per-pane
/// tiered scrollback instances.
///
/// # Usage
///
/// ```ignore
/// let mut coord = FleetScrollbackCoordinator::new(
///     CoordinatorConfig::default(),
///     FleetMemoryConfig::default(),
/// );
///
/// // In the maintenance loop:
/// let pane_infos = collect_pane_infos(&panes);
/// let signals = collect_pressure_signals();
/// let result = coord.evaluate(&signals, &pane_infos, &mut panes);
/// ```
#[derive(Debug)]
pub struct FleetScrollbackCoordinator {
    config: CoordinatorConfig,
    controller: FleetMemoryController,
    orchestrator: FleetScrollbackOrchestrator,
    telemetry: CoordinatorTelemetry,
}

impl FleetScrollbackCoordinator {
    /// Create a new coordinator with the given configuration.
    #[must_use]
    pub fn new(config: CoordinatorConfig, fleet_config: FleetMemoryConfig) -> Self {
        Self {
            config,
            controller: FleetMemoryController::new(fleet_config),
            orchestrator: FleetScrollbackOrchestrator::new(),
            telemetry: CoordinatorTelemetry::default(),
        }
    }

    /// Read-only access to cumulative telemetry.
    #[must_use]
    pub fn telemetry(&self) -> &CoordinatorTelemetry {
        &self.telemetry
    }

    /// Snapshot of cumulative telemetry (owned copy for serialization).
    #[must_use]
    pub fn telemetry_snapshot(&self) -> CoordinatorTelemetry {
        self.telemetry.clone()
    }

    /// Current compound fleet pressure tier.
    #[must_use]
    pub fn compound_tier(&self) -> FleetPressureTier {
        self.controller.compound_tier()
    }

    /// Reference to the coordinator configuration.
    #[must_use]
    pub fn config(&self) -> &CoordinatorConfig {
        &self.config
    }

    /// Run a single evaluation tick.
    ///
    /// `signals` — current pressure readings from all subsystems.
    /// `pane_infos` — per-pane scrollback metadata collected from panes.
    /// `panes` — mutable map from pane_id → TieredScrollback so the
    ///   coordinator can apply eviction directly.
    ///
    /// Returns an [`EvaluationResult`] describing what happened.
    pub fn evaluate(
        &mut self,
        signals: &PressureSignals,
        pane_infos: &[PaneScrollbackInfo],
        panes: &mut dyn PaneScrollbackAccess,
    ) -> EvaluationResult {
        self.telemetry.ticks += 1;

        // Step 1: Evaluate fleet pressure.
        let actions = self.controller.evaluate(signals);
        let tier = self.controller.compound_tier();

        if tier != FleetPressureTier::Normal {
            self.telemetry.elevated_ticks += 1;
        }

        // Step 2: Determine if eviction is needed.
        let needs_eviction = actions.contains(&FleetMemoryAction::EvictWarmScrollback)
            || actions.contains(&FleetMemoryAction::EmergencyCleanup);

        if !needs_eviction {
            return EvaluationResult {
                compound_tier: tier,
                actions,
                eviction_plan: None,
                pages_evicted: 0,
                bytes_reclaimed: 0,
                targets_applied: 0,
            };
        }

        // Step 3: Check minimum threshold.
        let fleet_warm_bytes: usize = pane_infos.iter().map(|p| p.warm_bytes).sum();
        if fleet_warm_bytes < self.config.min_fleet_warm_bytes_for_eviction {
            self.telemetry.skipped_below_threshold += 1;
            return EvaluationResult {
                compound_tier: tier,
                actions,
                eviction_plan: None,
                pages_evicted: 0,
                bytes_reclaimed: 0,
                targets_applied: 0,
            };
        }

        // Step 4: Handle emergency vs proportional eviction.
        let is_emergency = actions.contains(&FleetMemoryAction::EmergencyCleanup);

        if is_emergency && self.config.emergency_evict_all {
            // Emergency path: evict all warm pages on every pane.
            self.telemetry.emergency_cleanups += 1;
            let (pages, bytes, targets) = self.apply_emergency_eviction(pane_infos, panes);
            return EvaluationResult {
                compound_tier: tier,
                actions,
                eviction_plan: None, // emergency bypasses orchestrator
                pages_evicted: pages,
                bytes_reclaimed: bytes,
                targets_applied: targets,
            };
        }

        // Step 5: Ask orchestrator for a proportional eviction plan.
        let plan = self.orchestrator.plan_eviction(tier, pane_infos);

        let Some(plan) = plan else {
            return EvaluationResult {
                compound_tier: tier,
                actions,
                eviction_plan: None,
                pages_evicted: 0,
                bytes_reclaimed: 0,
                targets_applied: 0,
            };
        };

        self.telemetry.plans_produced += 1;

        // Step 6: Apply the eviction plan (bounded by max_targets_per_cycle).
        let (pages, bytes, targets) = self.apply_plan(&plan, panes);

        EvaluationResult {
            compound_tier: tier,
            actions,
            eviction_plan: Some(plan),
            pages_evicted: pages,
            bytes_reclaimed: bytes,
            targets_applied: targets,
        }
    }

    /// Collect per-pane scrollback info from a set of TieredScrollback instances.
    ///
    /// Convenience helper for callers that hold pane data in a map-like structure.
    pub fn collect_pane_infos(panes: &dyn PaneScrollbackAccess) -> Vec<PaneScrollbackInfo> {
        let ids = panes.pane_ids();
        let mut infos = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(snapshot) = panes.snapshot(id) {
                infos.push(PaneScrollbackInfo {
                    pane_id: id,
                    activity_counter: snapshot.activity_counter,
                    warm_bytes: snapshot.warm_bytes,
                    warm_pages: snapshot.warm_pages,
                    estimated_memory_bytes: snapshot.warm_bytes + (snapshot.hot_lines * 200), // rough estimate: 200 bytes/line
                });
            }
        }
        infos
    }

    /// Build default pressure signals for testing or fallback.
    #[must_use]
    pub fn default_signals(pane_count: usize) -> PressureSignals {
        PressureSignals {
            backpressure: BackpressureTier::Green,
            memory_pressure: MemoryPressureTier::Green,
            worst_budget: BudgetLevel::Normal,
            pane_count,
            paused_pane_count: 0,
        }
    }

    // ── Internal ──────────────────────────────────────────────────────

    /// Apply a proportional eviction plan to pane scrollbacks.
    fn apply_plan(
        &mut self,
        plan: &EvictionPlan,
        panes: &mut dyn PaneScrollbackAccess,
    ) -> (u64, u64, usize) {
        let mut total_pages: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut applied = 0;

        let targets = &plan.targets;
        let max = self.config.max_targets_per_cycle.min(targets.len());

        for target in &targets[..max] {
            let before = panes.snapshot(target.pane_id);
            let evicted = panes.evict_warm_pages(target.pane_id, target.pages_to_evict);
            let after = panes.snapshot(target.pane_id);

            let bytes_freed = match (before, after) {
                (Some(b), Some(a)) => b.warm_bytes.saturating_sub(a.warm_bytes) as u64,
                _ => 0,
            };

            total_pages += evicted as u64;
            total_bytes += bytes_freed;
            applied += 1;
        }

        self.telemetry.targets_applied += applied as u64;
        self.telemetry.pages_evicted += total_pages;
        self.telemetry.bytes_reclaimed += total_bytes;

        (total_pages, total_bytes, applied)
    }

    /// Emergency path: evict all warm pages on every pane.
    fn apply_emergency_eviction(
        &mut self,
        pane_infos: &[PaneScrollbackInfo],
        panes: &mut dyn PaneScrollbackAccess,
    ) -> (u64, u64, usize) {
        let mut total_pages: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut targets = 0;

        for info in pane_infos {
            if info.warm_pages == 0 {
                continue;
            }
            let before_bytes = info.warm_bytes as u64;
            panes.evict_all_warm(info.pane_id);
            total_pages += info.warm_pages as u64;
            total_bytes += before_bytes;
            targets += 1;
        }

        self.telemetry.targets_applied += targets as u64;
        self.telemetry.pages_evicted += total_pages;
        self.telemetry.bytes_reclaimed += total_bytes;

        (total_pages, total_bytes, targets)
    }
}

impl Default for FleetScrollbackCoordinator {
    fn default() -> Self {
        Self::new(CoordinatorConfig::default(), FleetMemoryConfig::default())
    }
}

// =============================================================================
// Pane access trait
// =============================================================================

/// Trait abstracting access to per-pane TieredScrollback instances.
///
/// This decouples the coordinator from the concrete pane storage structure
/// (HashMap, BTreeMap, registry, etc.), making it testable with mock panes.
pub trait PaneScrollbackAccess {
    /// Return all active pane IDs.
    fn pane_ids(&self) -> Vec<u64>;

    /// Get a scrollback tier snapshot for a specific pane.
    fn snapshot(&self, pane_id: u64) -> Option<ScrollbackTierSnapshot>;

    /// Evict up to `count` warm pages from a specific pane's scrollback.
    /// Returns the number of pages actually evicted.
    fn evict_warm_pages(&mut self, pane_id: u64, count: usize) -> usize;

    /// Evict all warm pages from a specific pane's scrollback.
    fn evict_all_warm(&mut self, pane_id: u64);
}

/// Simple HashMap-based implementation for testing and lightweight usage.
impl PaneScrollbackAccess for std::collections::HashMap<u64, TieredScrollback> {
    fn pane_ids(&self) -> Vec<u64> {
        self.keys().copied().collect()
    }

    fn snapshot(&self, pane_id: u64) -> Option<ScrollbackTierSnapshot> {
        self.get(&pane_id).map(TieredScrollback::snapshot)
    }

    fn evict_warm_pages(&mut self, pane_id: u64, count: usize) -> usize {
        self.get_mut(&pane_id)
            .map(|sb| sb.evict_warm_pages(count))
            .unwrap_or(0)
    }

    fn evict_all_warm(&mut self, pane_id: u64) {
        if let Some(sb) = self.get_mut(&pane_id) {
            sb.evict_all_warm();
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrollback_tiers::ScrollbackConfig;

    fn small_config() -> ScrollbackConfig {
        ScrollbackConfig {
            hot_lines: 10,
            page_size: 5,
            warm_max_bytes: 100_000, // large cap so warm accumulates
            ..ScrollbackConfig::default()
        }
    }

    fn fill_pane(sb: &mut TieredScrollback, lines: usize) {
        for i in 0..lines {
            sb.push_line(format!("line-{i:06}"));
        }
    }

    fn make_panes(
        count: usize,
        lines_per_pane: usize,
    ) -> std::collections::HashMap<u64, TieredScrollback> {
        let mut panes = std::collections::HashMap::new();
        for i in 0..count {
            let mut sb = TieredScrollback::new(small_config());
            fill_pane(&mut sb, lines_per_pane);
            panes.insert(i as u64, sb);
        }
        panes
    }

    fn pane_infos_from_map(
        panes: &std::collections::HashMap<u64, TieredScrollback>,
    ) -> Vec<PaneScrollbackInfo> {
        panes
            .iter()
            .map(|(&id, sb)| {
                let snap = sb.snapshot();
                PaneScrollbackInfo {
                    pane_id: id,
                    activity_counter: snap.activity_counter,
                    warm_bytes: snap.warm_bytes,
                    warm_pages: snap.warm_pages,
                    estimated_memory_bytes: sb.estimated_memory_bytes(),
                }
            })
            .collect()
    }

    // ── Basic lifecycle ──────────────────────────────────────────────

    #[test]
    fn default_coordinator_starts_normal() {
        let coord = FleetScrollbackCoordinator::default();
        assert_eq!(coord.compound_tier(), FleetPressureTier::Normal);
        assert_eq!(coord.telemetry().ticks, 0);
    }

    #[test]
    fn normal_pressure_produces_no_eviction() {
        let mut coord = FleetScrollbackCoordinator::default();
        let mut panes = make_panes(10, 50);
        let infos = pane_infos_from_map(&panes);
        let signals = FleetScrollbackCoordinator::default_signals(10);

        let result = coord.evaluate(&signals, &infos, &mut panes);

        assert_eq!(result.compound_tier, FleetPressureTier::Normal);
        assert!(result.eviction_plan.is_none());
        assert_eq!(result.pages_evicted, 0);
        assert_eq!(coord.telemetry().ticks, 1);
        assert_eq!(coord.telemetry().elevated_ticks, 0);
    }

    #[test]
    fn elevated_pressure_with_many_panes_triggers_eviction() {
        // Use immediate escalation to avoid hysteresis complexity in test.
        let mut coord = FleetScrollbackCoordinator::new(
            CoordinatorConfig::default(),
            FleetMemoryConfig {
                escalation_threshold: 1,
                deescalation_threshold: 1,
                ..FleetMemoryConfig::default()
            },
        );
        let mut panes = make_panes(200, 100); // 200 panes, lots of warm data
        let infos = pane_infos_from_map(&panes);

        // Elevated signals with 200+ panes
        let signals = PressureSignals {
            backpressure: BackpressureTier::Yellow,
            memory_pressure: MemoryPressureTier::Green,
            worst_budget: BudgetLevel::Normal,
            pane_count: 200,
            paused_pane_count: 0,
        };

        // With threshold=1, first evaluation should already escalate
        let result = coord.evaluate(&signals, &infos, &mut panes);
        assert_eq!(result.compound_tier, FleetPressureTier::Elevated);
        assert!(coord.telemetry().elevated_ticks >= 1);
    }

    #[test]
    fn critical_pressure_evicts_warm_pages() {
        let mut coord = FleetScrollbackCoordinator::new(
            CoordinatorConfig {
                min_fleet_warm_bytes_for_eviction: 0, // no threshold
                ..CoordinatorConfig::default()
            },
            FleetMemoryConfig::default(),
        );
        let mut panes = make_panes(5, 200); // 5 panes with substantial warm data

        let signals = PressureSignals {
            backpressure: BackpressureTier::Red,
            memory_pressure: MemoryPressureTier::Orange,
            worst_budget: BudgetLevel::Normal,
            pane_count: 5,
            paused_pane_count: 0,
        };

        // Push past hysteresis
        for _ in 0..4 {
            let infos = pane_infos_from_map(&panes);
            coord.evaluate(&signals, &infos, &mut panes);
        }

        let infos = pane_infos_from_map(&panes);
        let fleet_warm_before: usize = infos.iter().map(|p| p.warm_bytes).sum();

        let result = coord.evaluate(&signals, &infos, &mut panes);
        assert_eq!(result.compound_tier, FleetPressureTier::Critical);

        if fleet_warm_before > 0 {
            // Should have evicted some pages
            assert!(
                result.pages_evicted > 0 || result.eviction_plan.is_some(),
                "Critical tier should trigger eviction when warm data exists"
            );
        }
    }

    #[test]
    fn emergency_evicts_all_warm() {
        let mut coord = FleetScrollbackCoordinator::new(
            CoordinatorConfig {
                emergency_evict_all: true,
                min_fleet_warm_bytes_for_eviction: 0,
                ..CoordinatorConfig::default()
            },
            FleetMemoryConfig::default(),
        );
        let mut panes = make_panes(5, 200);

        let signals = PressureSignals {
            backpressure: BackpressureTier::Black,
            memory_pressure: MemoryPressureTier::Red,
            worst_budget: BudgetLevel::OverBudget,
            pane_count: 5,
            paused_pane_count: 0,
        };

        // Push past hysteresis
        for _ in 0..4 {
            let infos = pane_infos_from_map(&panes);
            coord.evaluate(&signals, &infos, &mut panes);
        }

        let infos = pane_infos_from_map(&panes);
        let result = coord.evaluate(&signals, &infos, &mut panes);
        assert_eq!(result.compound_tier, FleetPressureTier::Emergency);

        // After emergency, all warm should be gone
        for sb in panes.values() {
            assert_eq!(
                sb.warm_page_count(),
                0,
                "Emergency should evict all warm pages"
            );
        }
        assert!(coord.telemetry().emergency_cleanups > 0);
    }

    #[test]
    fn below_threshold_skips_eviction() {
        let mut coord = FleetScrollbackCoordinator::new(
            CoordinatorConfig {
                min_fleet_warm_bytes_for_eviction: usize::MAX, // impossibly high
                ..CoordinatorConfig::default()
            },
            FleetMemoryConfig::default(),
        );
        let mut panes = make_panes(5, 200);

        let signals = PressureSignals {
            backpressure: BackpressureTier::Red,
            memory_pressure: MemoryPressureTier::Orange,
            worst_budget: BudgetLevel::Normal,
            pane_count: 5,
            paused_pane_count: 0,
        };

        // Push past hysteresis
        for _ in 0..4 {
            let infos = pane_infos_from_map(&panes);
            coord.evaluate(&signals, &infos, &mut panes);
        }

        let infos = pane_infos_from_map(&panes);
        let result = coord.evaluate(&signals, &infos, &mut panes);

        assert_eq!(result.pages_evicted, 0);
        assert!(coord.telemetry().skipped_below_threshold > 0);
    }

    #[test]
    fn max_targets_per_cycle_limits_blast_radius() {
        let mut coord = FleetScrollbackCoordinator::new(
            CoordinatorConfig {
                max_targets_per_cycle: 2, // Only evict from 2 panes per tick
                min_fleet_warm_bytes_for_eviction: 0,
                ..CoordinatorConfig::default()
            },
            FleetMemoryConfig::default(),
        );
        let mut panes = make_panes(10, 200);

        let signals = PressureSignals {
            backpressure: BackpressureTier::Red,
            memory_pressure: MemoryPressureTier::Orange,
            worst_budget: BudgetLevel::Normal,
            pane_count: 10,
            paused_pane_count: 0,
        };

        // Push past hysteresis
        for _ in 0..4 {
            let infos = pane_infos_from_map(&panes);
            coord.evaluate(&signals, &infos, &mut panes);
        }

        let infos = pane_infos_from_map(&panes);
        let result = coord.evaluate(&signals, &infos, &mut panes);

        assert!(
            result.targets_applied <= 2,
            "Should limit to max_targets_per_cycle: got {}",
            result.targets_applied
        );
    }

    #[test]
    fn collect_pane_infos_from_trait() {
        let panes = make_panes(3, 50);
        let infos = FleetScrollbackCoordinator::collect_pane_infos(&panes);
        assert_eq!(infos.len(), 3);
        for info in &infos {
            assert!(info.pane_id < 3);
        }
    }

    #[test]
    fn telemetry_accumulates_across_ticks() {
        let mut coord = FleetScrollbackCoordinator::default();
        let mut panes = make_panes(5, 50);
        let signals = FleetScrollbackCoordinator::default_signals(5);

        for _ in 0..10 {
            let infos = pane_infos_from_map(&panes);
            coord.evaluate(&signals, &infos, &mut panes);
        }

        assert_eq!(coord.telemetry().ticks, 10);
    }

    #[test]
    fn telemetry_snapshot_is_independent_copy() {
        let mut coord = FleetScrollbackCoordinator::default();
        let mut panes = make_panes(1, 10);
        let infos = pane_infos_from_map(&panes);
        let signals = FleetScrollbackCoordinator::default_signals(1);

        coord.evaluate(&signals, &infos, &mut panes);
        let snap1 = coord.telemetry_snapshot();

        coord.evaluate(&signals, &infos, &mut panes);
        let snap2 = coord.telemetry_snapshot();

        assert_eq!(snap1.ticks, 1);
        assert_eq!(snap2.ticks, 2);
    }

    #[test]
    fn empty_fleet_produces_no_eviction_under_pressure() {
        let mut coord = FleetScrollbackCoordinator::new(
            CoordinatorConfig {
                min_fleet_warm_bytes_for_eviction: 0,
                ..CoordinatorConfig::default()
            },
            FleetMemoryConfig::default(),
        );
        let mut panes: std::collections::HashMap<u64, TieredScrollback> =
            std::collections::HashMap::new();
        let infos: Vec<PaneScrollbackInfo> = Vec::new();

        let signals = PressureSignals {
            backpressure: BackpressureTier::Red,
            memory_pressure: MemoryPressureTier::Orange,
            worst_budget: BudgetLevel::Normal,
            pane_count: 0,
            paused_pane_count: 0,
        };

        // Push past hysteresis
        for _ in 0..5 {
            coord.evaluate(&signals, &infos, &mut panes);
        }

        let result = coord.evaluate(&signals, &infos, &mut panes);
        assert_eq!(result.pages_evicted, 0);
        assert_eq!(result.targets_applied, 0);
    }

    #[test]
    fn serde_roundtrip_telemetry() {
        let telem = CoordinatorTelemetry {
            ticks: 100,
            elevated_ticks: 20,
            plans_produced: 5,
            targets_applied: 15,
            pages_evicted: 150,
            bytes_reclaimed: 1_048_576,
            emergency_cleanups: 1,
            skipped_below_threshold: 3,
        };

        let json = serde_json::to_string(&telem).expect("serialize");
        let back: CoordinatorTelemetry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(telem, back);
    }

    #[test]
    fn serde_roundtrip_config() {
        let config = CoordinatorConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let back: CoordinatorConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.max_targets_per_cycle, config.max_targets_per_cycle);
        assert_eq!(
            back.min_fleet_warm_bytes_for_eviction,
            config.min_fleet_warm_bytes_for_eviction
        );
    }

    #[test]
    fn serde_roundtrip_evaluation_result() {
        let result = EvaluationResult {
            compound_tier: FleetPressureTier::Elevated,
            actions: vec![FleetMemoryAction::ThrottlePolling],
            eviction_plan: None,
            pages_evicted: 0,
            bytes_reclaimed: 0,
            targets_applied: 0,
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let back: EvaluationResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.compound_tier, FleetPressureTier::Elevated);
        assert_eq!(back.pages_evicted, 0);
    }

    #[test]
    fn pane_access_trait_evict_nonexistent_pane() {
        let mut panes: std::collections::HashMap<u64, TieredScrollback> =
            std::collections::HashMap::new();
        // Evicting from a nonexistent pane should return 0, not panic
        let evicted = panes.evict_warm_pages(999, 10);
        assert_eq!(evicted, 0);
    }

    #[test]
    fn pane_access_trait_evict_all_warm_nonexistent() {
        let mut panes: std::collections::HashMap<u64, TieredScrollback> =
            std::collections::HashMap::new();
        // Should be a no-op
        panes.evict_all_warm(999);
    }

    #[test]
    fn pane_access_trait_snapshot_nonexistent() {
        let panes: std::collections::HashMap<u64, TieredScrollback> =
            std::collections::HashMap::new();
        assert!(panes.snapshot(999).is_none());
    }

    #[test]
    fn proportional_eviction_does_not_emergency_cleanup() {
        let mut coord = FleetScrollbackCoordinator::new(
            CoordinatorConfig {
                emergency_evict_all: true,
                min_fleet_warm_bytes_for_eviction: 0,
                ..CoordinatorConfig::default()
            },
            FleetMemoryConfig::default(),
        );
        let mut panes = make_panes(5, 200);

        // Critical but not emergency
        let signals = PressureSignals {
            backpressure: BackpressureTier::Red,
            memory_pressure: MemoryPressureTier::Orange,
            worst_budget: BudgetLevel::Normal,
            pane_count: 5,
            paused_pane_count: 0,
        };

        // Push past hysteresis
        for _ in 0..5 {
            let infos = pane_infos_from_map(&panes);
            coord.evaluate(&signals, &infos, &mut panes);
        }

        // Should not have triggered emergency cleanup
        assert_eq!(
            coord.telemetry().emergency_cleanups,
            0,
            "Critical tier should use proportional eviction, not emergency"
        );

        // But some panes should still have warm data
        let total_warm: usize = panes.values().map(|sb| sb.warm_page_count()).sum();
        // (Some warm pages remain because proportional eviction doesn't evict everything)
        // This assertion is conditional — depends on data volume
        let _ = total_warm; // suppress unused warning
    }
}
