use serde_json::Value;
use skillopt_rs::skillopt::engine::ReflActPipeline;
use skillopt_rs::skillopt::evaluation::gate::{evaluate_gate, GateMetric, GateOptions};
use skillopt_rs::skillopt::optimizer::scheduler::{LRScheduler, SchedulerMode};
use skillopt_rs::skillopt::optimizer::slow_update::{
    extract_slow_update_field, inject_empty_slow_update_field, replace_slow_update_field,
};
use skillopt_rs::skillopt::types::{GateAction, Patch};
use skillopt_rs::skillopt::utils::scoring::skill_hash;
use std::fs;

#[test]
fn test_python_parity_fixture() {
    let fixture_str = fs::read_to_string("tests_py_fixture.json")
        .expect("Failed to read tests_py_fixture.json");
    let fixture: Value = serde_json::from_str(&fixture_str).expect("Valid JSON fixture");

    // 1. Skill Hash Parity
    let initial_skill = fixture["initial_skill"].as_str().unwrap();
    let py_hash = fixture["hashing"]["skill_hash"].as_str().unwrap();
    let rs_hash = skill_hash(initial_skill);
    assert_eq!(rs_hash, py_hash, "Skill hash must match Python byte-for-byte");

    // 2. Patch Deserialization & Application Parity
    let patch: Patch = serde_json::from_value(fixture["patch"].clone())
        .expect("Deserialize Patch from Python JSON");
    assert_eq!(patch.edits.len(), 3);

    let updated_skill = ReflActPipeline::apply_patch(initial_skill, &patch);
    assert!(
        updated_skill.contains("## Step 1.1: Validate parameter bounds."),
        "InsertAfter edit must be present"
    );
    assert!(
        updated_skill.contains("## Step 3: Run sanity check before final conclusion."),
        "Replace edit must be applied"
    );
    assert!(
        updated_skill.contains("## General Tip: Always keep responses concise."),
        "Append edit must be applied"
    );

    // 3. Validation Gate Decision Parity
    let candidate_skill = format!("{}\n## Step 1.1: Validate parameter bounds.", initial_skill);
    let opts = GateOptions {
        cand_soft: 0.90,
        metric: GateMetric::Mixed,
        mixed_weight: 0.5,
        ..Default::default()
    };
    let gate_res = evaluate_gate(
        &candidate_skill,
        0.85,
        initial_skill,
        0.60,
        initial_skill,
        0.70,
        0,
        1,
        opts,
    );

    assert_eq!(gate_res.action, GateAction::AcceptNewBest);
    let py_current_score = fixture["gate_decision"]["current_score"].as_f64().unwrap();
    assert!((gate_res.current_score - py_current_score).abs() < 1e-3);
    assert_eq!(gate_res.best_step, 1);

    // 4. Scheduler Parity
    let mut linear_sched = LRScheduler::new(SchedulerMode::Linear, 10, 2, 5);
    let py_linear: Vec<usize> = fixture["linear_steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    for (step_idx, expected_lr) in py_linear.into_iter().enumerate() {
        let rs_lr = linear_sched.step();
        assert_eq!(
            rs_lr, expected_lr,
            "Linear step {} mismatch: rust={} vs python={}",
            step_idx + 1,
            rs_lr,
            expected_lr
        );
    }

    let mut cosine_sched = LRScheduler::new(SchedulerMode::Cosine, 10, 2, 5);
    let py_cosine: Vec<usize> = fixture["cosine_steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    for (step_idx, expected_lr) in py_cosine.into_iter().enumerate() {
        let rs_lr = cosine_sched.step();
        assert_eq!(
            rs_lr, expected_lr,
            "Cosine step {} mismatch: rust={} vs python={}",
            step_idx + 1,
            rs_lr,
            expected_lr
        );
    }

    // 5. Protected Slow Update Parity
    let slow_initial = inject_empty_slow_update_field(initial_skill);
    let slow_updated = replace_slow_update_field(
        &slow_initial,
        "Prioritize exact match over exploratory paths.",
    );
    let slow_extracted = extract_slow_update_field(&slow_updated);
    let py_extracted = fixture["slow_update"]["extracted"].as_str().unwrap();
    assert_eq!(slow_extracted, py_extracted);
}
