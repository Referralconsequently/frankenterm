//! Integration tests: FleetMemoryController + TieredScrollback interaction.
//!
//! Validates that fleet pressure evaluation produces correct actions for
//! realistic 200+ pane swarm scenarios, and that TieredScrollback responds
//! correctly to those actions (eviction, enforce_warm_cap, etc.).
//!
//! Bead: ft-1memj.19

use frankenterm_core::backpressure::BackpressureTier;
use frankenterm_core::fleet_memory_controller::{
    FleetMemoryAction, FleetMemoryConfig, FleetMemoryController, FleetPressureTier,
    PressureSignals,
};
use frankenterm_core::memory_budget::BudgetLevel;
use frankenterm_core::memory_pressure::MemoryPressureTier;
use frankenterm_core::scrollback_tiers::{ScrollbackConfig, TieredScrollback};

fn normal_signals(pane_count: usize) -> PressureSignals {
    PressureSignals {
        backpressure: BackpressureTier::Green,
        memory_pressure: MemoryPressureTier::Green,
        worst_budget: BudgetLevel::Normal,
        pane_count,
        paused_pane_count: 0,
    }
}

fn elevated_signals(pane_count: usize) -> PressureSignals {
    PressureSignals {
        backpressure: BackpressureTier::Yellow,
        memory_pressure: MemoryPressureTier::Green,
        worst_budget: BudgetLevel::Normal,
        pane_count,
        paused_pane_count: 0,
    }
}

fn critical_signals(pane_count: usize) -> PressureSignals {
    PressureSignals {
        backpressure: BackpressureTier::Red,
        memory_pressure: MemoryPressureTier::Orange,
        worst_budget: BudgetLevel::Normal,
        pane_count,
        paused_pane_count: 0,
    }
}

fn emergency_signals(pane_count: usize) -> PressureSignals {
    PressureSignals {
        backpressure: BackpressureTier::Black,
        memory_pressure: MemoryPressureTier::Red,
        worst_budget: BudgetLevel::OverBudget,
        pane_count,
        paused_pane_count: 0,
    }
}

// ---------------------------------------------------------------------------
// Fleet controller evaluation tests
// ---------------------------------------------------------------------------

#[test]
fn fleet_normal_with_few_panes_recommends_no_action() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let actions = ctrl.evaluate(&normal_signals(10));
    assert_eq!(ctrl.compound_tier(), FleetPressureTier::Normal);
    assert!(
        actions.iter().all(|a| matches!(a, FleetMemoryAction::None)),
        "Normal tier should produce no active actions: {actions:?}"
    );
}

#[test]
fn fleet_elevated_with_many_panes_recommends_eviction() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    // Need consecutive evaluations to overcome hysteresis
    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..hysteresis {
        ctrl.evaluate(&elevated_signals(200));
    }
    let actions = ctrl.evaluate(&elevated_signals(200));
    assert_eq!(ctrl.compound_tier(), FleetPressureTier::Elevated);
    assert!(
        actions.contains(&FleetMemoryAction::ThrottlePolling),
        "Elevated tier should throttle: {actions:?}"
    );
    // With 200 panes, eviction should also be recommended
    assert!(
        actions.contains(&FleetMemoryAction::EvictWarmScrollback),
        "Elevated tier with 200 panes should evict warm scrollback: {actions:?}"
    );
}

#[test]
fn fleet_elevated_with_few_panes_skips_eviction() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..hysteresis {
        ctrl.evaluate(&elevated_signals(50));
    }
    let actions = ctrl.evaluate(&elevated_signals(50));
    assert_eq!(ctrl.compound_tier(), FleetPressureTier::Elevated);
    assert!(
        actions.contains(&FleetMemoryAction::ThrottlePolling),
        "Elevated tier should throttle: {actions:?}"
    );
    assert!(
        !actions.contains(&FleetMemoryAction::EvictWarmScrollback),
        "Elevated tier with <= 100 panes should NOT evict: {actions:?}"
    );
}

#[test]
fn fleet_critical_always_evicts_and_pauses() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..hysteresis {
        ctrl.evaluate(&critical_signals(200));
    }
    let actions = ctrl.evaluate(&critical_signals(200));
    assert_eq!(ctrl.compound_tier(), FleetPressureTier::Critical);
    assert!(actions.contains(&FleetMemoryAction::EvictWarmScrollback));
    assert!(actions.contains(&FleetMemoryAction::PauseIdlePanes));
    assert!(actions.contains(&FleetMemoryAction::ThrottlePolling));
}

#[test]
fn fleet_emergency_triggers_cleanup() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..hysteresis {
        ctrl.evaluate(&emergency_signals(200));
    }
    let actions = ctrl.evaluate(&emergency_signals(200));
    assert_eq!(ctrl.compound_tier(), FleetPressureTier::Emergency);
    assert!(
        actions.contains(&FleetMemoryAction::EmergencyCleanup),
        "Emergency should trigger cleanup: {actions:?}"
    );
}

// ---------------------------------------------------------------------------
// Hysteresis behavior
// ---------------------------------------------------------------------------

#[test]
fn fleet_hysteresis_prevents_oscillation() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let escalation = ctrl.config().escalation_threshold;
    let deescalation = ctrl.config().deescalation_threshold;

    // Escalate to Elevated
    for _ in 0..=escalation {
        ctrl.evaluate(&elevated_signals(200));
    }
    assert_eq!(ctrl.compound_tier(), FleetPressureTier::Elevated);

    // Single Normal reading should NOT de-escalate
    ctrl.evaluate(&normal_signals(200));
    assert_eq!(
        ctrl.compound_tier(),
        FleetPressureTier::Elevated,
        "Single normal reading should not de-escalate due to hysteresis"
    );

    // Need `deescalation_threshold` consecutive Normal readings
    for _ in 0..deescalation {
        ctrl.evaluate(&normal_signals(200));
    }
    assert_eq!(
        ctrl.compound_tier(),
        FleetPressureTier::Normal,
        "Should de-escalate after {deescalation} consecutive normal readings"
    );
}

// ---------------------------------------------------------------------------
// Scrollback + fleet action integration
// ---------------------------------------------------------------------------

#[test]
fn evict_warm_scrollback_clears_warm_tier() {
    let config = ScrollbackConfig {
        hot_lines: 100,
        page_size: 50,
        warm_max_bytes: 10 * 1024 * 1024,
        ..ScrollbackConfig::default()
    };
    let mut scrollback = TieredScrollback::new(config);

    // Fill hot + overflow to warm
    for i in 0..500 {
        scrollback.push_line(format!("Line {i}: some content that takes up space in the buffer"));
    }

    let snap_before = scrollback.snapshot();
    assert!(snap_before.warm_pages > 0, "Should have warm pages after overflow");
    assert!(snap_before.warm_bytes > 0, "Should have warm bytes");

    // Simulate EvictWarmScrollback action
    scrollback.evict_all_warm();

    let snap_after = scrollback.snapshot();
    assert_eq!(snap_after.warm_pages, 0, "Warm pages should be zero after eviction");
    assert_eq!(snap_after.warm_bytes, 0, "Warm bytes should be zero after eviction");
    assert!(snap_after.cold_pages > 0, "Cold pages should increase after eviction");
    assert_eq!(snap_after.hot_lines, snap_before.hot_lines, "Hot lines should be unchanged");
}

#[test]
fn fleet_driven_eviction_cycle_for_200_panes() {
    let config = ScrollbackConfig {
        hot_lines: 100,
        page_size: 50,
        warm_max_bytes: 1024 * 1024, // 1 MB per pane
        ..ScrollbackConfig::default()
    };

    // Simulate 200 panes with warm data
    let mut panes: Vec<TieredScrollback> = (0..200)
        .map(|_| TieredScrollback::new(config.clone()))
        .collect();

    // Fill each pane with enough lines to generate warm pages
    for pane in &mut panes {
        for i in 0..300 {
            pane.push_line(format!("Agent output line {i}: working on task..."));
        }
    }

    // Verify warm data exists
    let total_warm_before: usize = panes.iter().map(|p| p.snapshot().warm_bytes).sum();
    assert!(total_warm_before > 0, "Should have warm data across panes");

    // Fleet controller detects pressure and recommends eviction
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..=hysteresis {
        ctrl.evaluate(&critical_signals(200));
    }
    let actions = ctrl.evaluate(&critical_signals(200));
    assert!(actions.contains(&FleetMemoryAction::EvictWarmScrollback));

    // Execute eviction on all panes
    for pane in &mut panes {
        pane.evict_all_warm();
    }

    // Verify warm tier is empty across fleet
    let total_warm_after: usize = panes.iter().map(|p| p.snapshot().warm_bytes).sum();
    assert_eq!(total_warm_after, 0, "All warm data should be evicted");

    // Hot tier should be unaffected
    for pane in &panes {
        let snap = pane.snapshot();
        assert!(snap.hot_lines > 0, "Hot lines should be preserved");
    }
}

#[test]
fn enforce_warm_cap_respects_byte_limit() {
    let config = ScrollbackConfig {
        hot_lines: 100,
        page_size: 50,
        warm_max_bytes: 1024, // Very small warm cap
        ..ScrollbackConfig::default()
    };
    let mut scrollback = TieredScrollback::new(config);

    // Push enough to overflow warm cap
    for i in 0..1000 {
        scrollback.push_line(format!("Line {i}: data that will overflow the warm tier cap"));
    }

    let snap = scrollback.snapshot();
    // enforce_warm_cap evicts pages until warm_bytes <= warm_max_bytes,
    // so the invariant is exact (not approximate).
    assert!(
        snap.warm_bytes <= 1024,
        "Warm bytes ({}) should respect cap (1024) after enforce_warm_cap",
        snap.warm_bytes
    );
    // Cold tier should have received the evicted warm pages.
    assert!(
        snap.cold_pages > 0,
        "Cold pages should be non-zero after warm cap enforcement"
    );
}

#[test]
fn fleet_snapshot_captures_evaluation_state() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());

    ctrl.evaluate(&normal_signals(10));
    let snap = ctrl.snapshot();
    assert_eq!(snap.compound_tier, FleetPressureTier::Normal);
    assert_eq!(snap.total_evaluations, 1);
    assert_eq!(snap.total_transitions, 0);

    // Escalate
    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..=hysteresis {
        ctrl.evaluate(&emergency_signals(200));
    }
    let snap = ctrl.snapshot();
    assert_eq!(snap.compound_tier, FleetPressureTier::Emergency);
    assert!(snap.total_transitions > 0);
}

#[test]
fn scrollback_snapshot_serde_roundtrip() {
    let config = ScrollbackConfig::default();
    let mut scrollback = TieredScrollback::new(config);
    for i in 0..200 {
        scrollback.push_line(format!("Line {i}"));
    }

    let snap = scrollback.snapshot();
    let json = serde_json::to_string(&snap).expect("serialize snapshot");
    let deserialized: frankenterm_core::scrollback_tiers::ScrollbackTierSnapshot =
        serde_json::from_str(&json).expect("deserialize snapshot");
    assert_eq!(snap, deserialized);
}

#[test]
fn fleet_audit_trail_records_decisions() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());

    // Multiple evaluations
    for _ in 0..5 {
        ctrl.evaluate(&normal_signals(10));
    }

    let trail = ctrl.audit_trail();
    assert_eq!(trail.len(), 5, "Should have 5 audit records");
    for record in trail {
        assert_eq!(record.compound_tier, FleetPressureTier::Normal);
    }
}

#[test]
fn memory_budget_over_budget_escalates_fleet_tier() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let signals = PressureSignals {
        backpressure: BackpressureTier::Green,
        memory_pressure: MemoryPressureTier::Green,
        worst_budget: BudgetLevel::OverBudget,
        pane_count: 200,
        paused_pane_count: 0,
    };

    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..=hysteresis {
        ctrl.evaluate(&signals);
    }
    assert!(
        ctrl.compound_tier() >= FleetPressureTier::Critical,
        "OverBudget should escalate to at least Critical: {:?}",
        ctrl.compound_tier()
    );
}

#[test]
fn worst_of_semantics_takes_most_severe_signal() {
    let mut ctrl = FleetMemoryController::new(FleetMemoryConfig::default());
    let signals = PressureSignals {
        backpressure: BackpressureTier::Green,     // Normal
        memory_pressure: MemoryPressureTier::Red,  // Emergency
        worst_budget: BudgetLevel::Normal,          // Normal
        pane_count: 50,
        paused_pane_count: 0,
    };

    let hysteresis = ctrl.config().escalation_threshold;
    for _ in 0..=hysteresis {
        ctrl.evaluate(&signals);
    }
    assert_eq!(
        ctrl.compound_tier(),
        FleetPressureTier::Emergency,
        "Worst-of should pick Emergency from Red memory pressure"
    );
}
