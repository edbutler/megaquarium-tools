# Human Test Plan: Feeding Skill Feature

## Prerequisites
- Rust nightly toolchain installed
- Game data files accessible (Steam installation of Megaquarium)
- `cargo test` passing (all 59 tests green)

## Phase 1: Report Output for Non-Zero Feeding Skill

| Step | Action | Expected |
|------|--------|----------|
| 1 | Run `cargo run -- check 33_arabian_butterflyfish=1` | Output includes a food line with `(skill 1)` annotation |
| 2 | Confirm the skill number matches the value from game data (should be 1 for Arabian butterflyfish) | Skill annotation shows the correct value |

## Phase 2: Report Output for Zero Feeding Skill

| Step | Action | Expected |
|------|--------|----------|
| 1 | Run `cargo run -- check 1_azure_demoiselle=1` | Output includes a food line without any `(skill ...)` annotation |
| 2 | Confirm no `skill` text appears in the food section of the output | No skill annotation present |

## Phase 3: Multiple Species with Different Skills on Same Food

| Step | Action | Expected |
|------|--------|----------|
| 1 | Identify two species that eat the same food type but have different feeding skill values | Two species identified |
| 2 | Run `cargo run -- check species_a=1 species_b=1` with those two species | The food line shows `(skill N)` where N is the maximum of the two skill values |

## End-to-End: Full Check with Mixed Diet Types

**Purpose:** Validate that a `check` command involving multiple food types each displays independent skill annotations.

1. Run `cargo run -- check 33_arabian_butterflyfish=1 1_azure_demoiselle=1`
2. Verify that the food type associated with the Arabian butterflyfish shows a `(skill 1)` annotation.
3. Verify that the food type associated with the Azure demoiselle shows no skill annotation.
4. Confirm the food counts are correct for each food type.

## Human Verification Required

| Criterion | Why Manual | Steps |
|-----------|------------|-------|
| AC3.1 / AC3.2 / AC3.3: Report output formatting | `report.rs` writes directly to stdout; tests verify `FoodAmount` struct values but not the printed format | Run the commands in Phases 1-3 above and visually confirm the `(skill N)` annotation appears/is omitted correctly |

## Traceability

| Acceptance Criterion | Automated Test | Manual Step |
|----------------------|----------------|-------------|
| AC1.1: Skill loaded from game data | `data::test::test_feeding_skill_ac1_1_skill_loaded_from_game_data` | -- |
| AC1.2: Skill defaults to 0 | `data::test::test_feeding_skill_ac1_2_skill_defaults_to_zero` | -- |
| AC1.3: Other diet types unaffected | `data::test::test_feeding_skill_ac1_3_other_diet_types_unaffected` | -- |
| AC2.1: Non-zero skill serialization | `sexpr_impl::tests::test_species_with_diet_food_skill_nonzero_to_sexp` | -- |
| AC2.2: Zero skill omitted in serialization | `sexpr_impl::tests::test_species_with_diet_food_to_sexp` | -- |
| AC3.1: Max skill aggregation | `check::test::test_skill_aggregation_two_species_same_food_different_skills` | Phase 1, Phase 3 |
| AC3.2: Zero skill no annotation | `check::test::test_skill_aggregation_zero_skill` | Phase 2 |
| AC3.3: Independent skill per food type | `check::test::test_skill_aggregation_multiple_food_types_independent_skills` | End-to-End scenario |
