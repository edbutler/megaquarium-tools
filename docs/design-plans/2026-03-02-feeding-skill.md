# Feeding Skill Design

## Summary

The Megaquarium game tracks a `needsFeedingSkill` property on animal species, indicating how much player skill is required to feed them successfully (0 = no special skill, up to 3 = highly demanding). Currently the tool ignores this property entirely. This feature threads the skill value from game data files all the way through to the tank summary output displayed by the `check` and `validate` commands, so that a player configuring a tank can see at a glance what feeding skill level their staff will need.

The implementation follows the data flow already established in the codebase: load the value in `data.rs`, carry it on the `Diet::Food` variant in `animal.rs`, serialize it in `sexpr_impl.rs` using the existing conditional keyword-argument pattern, and aggregate the per-food maximum in `check.rs` before printing it in `report.rs`. Because skill is additive across animals sharing a food type (the tank needs staff capable of the hardest feeder for each food), the summary reports the maximum skill required per food type rather than summing or averaging.

## Definition of Done
Add a `skill` field (u8, 0-3) to `Diet::Food` representing `needsFeedingSkill` from game data files. This field is loaded from all animal/coral data files, round-trips through s-expression serialization (e.g., `(food flakes 3 skill 2)`), and in the tank summary output (validate/check commands), the maximum feeding skill needed per food type across all animals in the tank is displayed.

## Acceptance Criteria

### feeding-skill.AC1: Skill loaded from game data
- **feeding-skill.AC1.1 Success:** Species with `needsFeedingSkill: {value: 2}` in game data gets `Diet::Food { skill: 2, .. }`
- **feeding-skill.AC1.2 Success:** Species without `needsFeedingSkill` in game data gets `Diet::Food { skill: 0, .. }`
- **feeding-skill.AC1.3 Edge:** Scavenger and DoesNotEat variants are unaffected by the change

### feeding-skill.AC2: S-expression output includes skill
- **feeding-skill.AC2.1 Success:** `Diet::Food { food: "krill", period: 2, skill: 2 }` serializes to `(food krill 2 #:skill 2)`
- **feeding-skill.AC2.2 Success:** `Diet::Food { food: "flakes", period: 3, skill: 0 }` serializes to `(food flakes 3)` (no skill annotation)

### feeding-skill.AC3: Tank summary shows max skill per food type
- **feeding-skill.AC3.1 Success:** Two species eating flakes with skill 1 and skill 2 → food line shows `(skill 2)`
- **feeding-skill.AC3.2 Success:** All species eating a food type have skill 0 → no skill annotation on that food line
- **feeding-skill.AC3.3 Edge:** Multiple food types each display their own independent max skill

## Glossary

- **`Diet::Food`**: A variant of the `Diet` enum in `animal.rs` representing a species that eats a named food on a timed feeding period. The other variants, `Scavenger` and `DoesNotEat`, have no feeding-skill concept.
- **`needsFeedingSkill`**: A property key in Megaquarium's raw game data files. Its value is an integer 0-3 indicating how difficult the species is to feed.
- **`FoodAmount`**: A struct in `check.rs` used to accumulate feeding requirements across all animals in a tank — how much of a given food is needed and (after this change) what skill level is required.
- **`minimum_required_food()`**: A function in `check.rs` that aggregates per-animal diet data into per-food-type totals for the tank summary. It currently groups by food name; this change also tracks the maximum skill seen per group.
- **s-expression (sexp)**: A parenthesized list notation (e.g., `(food flakes 3 #:skill 2)`) used by this tool to serialize species and aquarium data. Borrowed from Lisp/Racket syntax conventions.
- **`#:keyword` annotation**: The s-expression convention used in this codebase for optional named fields (e.g., `#:armored? #t`). A `#:skill N` annotation is added to food entries when skill > 0, and omitted when skill is 0.
- **round-trip**: The property that a value can be serialized to s-expression output and parsed back without loss of information.

## Architecture

Add `skill: u8` to `Diet::Food` to carry the `needsFeedingSkill` value from game data through to output. The value flows through three layers:

1. **Loading** (`src/data.rs`): After constructing `Diet::Food` from the `eats` block, read `needsFeedingSkill` from the animal's traits and attach as `skill`. Default 0 when absent.
2. **Serialization** (`src/sexpr_impl.rs`): `to_sexp` for `Diet::Food` conditionally appends `#:skill N` when skill > 0.
3. **Tank summary** (`src/check.rs`, `src/report.rs`): `FoodAmount` gains a `skill: u8` field. `minimum_required_food()` tracks max skill per food type. `report.rs` appends `(skill N)` to food lines when skill > 0.

Contracts:

```rust
// animal.rs
pub enum Diet {
    Food { food: String, period: u16, skill: u8 },
    Scavenger,
    DoesNotEat,
}

// check.rs
pub struct FoodAmount {
    pub food: String,
    pub count: u16,
    pub skill: u8,
}
```

## Existing Patterns

Investigation found that conditional s-expression fields follow a consistent pattern: emit nothing when the value is the default/absent case, emit `#:keyword value` when present. Examples: `#:armored? #t`, `#:greedy #t`. The `#:skill N` approach follows this pattern.

`FoodAmount` aggregation in `minimum_required_food()` already groups by food name. Adding max-skill tracking fits naturally into the existing fold.

`report.rs` conditionally appends annotations — the pattern of "print extra info when non-default" is already used for other fields.

## Implementation Phases

<!-- START_PHASE_1 -->
### Phase 1: Data Model and Loading
**Goal:** Add skill field to Diet::Food and load from game data

**Components:**
- `src/animal.rs` — Add `skill: u8` to `Diet::Food` variant
- `src/data.rs` — Read `needsFeedingSkill` from traits after constructing Diet, attach to `Diet::Food`

**Dependencies:** None

**Done when:** `cargo build` succeeds, all existing tests pass with updated Diet construction (skill: 0 where not specified), loaded species from game data carry correct skill values

**Covers:** feeding-skill.AC1.1, feeding-skill.AC1.2, feeding-skill.AC1.3
<!-- END_PHASE_1 -->

<!-- START_PHASE_2 -->
### Phase 2: S-Expression Output
**Goal:** Serialize skill in species s-expression output

**Components:**
- `src/sexpr_impl.rs` — Update `Diet::Food` branch in `to_sexp` to conditionally append `#:skill N`

**Dependencies:** Phase 1

**Done when:** Tests verify `(food flakes 3)` when skill=0 and `(food flakes 3 #:skill 2)` when skill=2

**Covers:** feeding-skill.AC2.1, feeding-skill.AC2.2
<!-- END_PHASE_2 -->

<!-- START_PHASE_3 -->
### Phase 3: Tank Summary Output
**Goal:** Display max feeding skill per food type in validate/check output

**Components:**
- `src/check.rs` — Add `skill: u8` to `FoodAmount`, update `minimum_required_food()` to track max skill per food type
- `src/report.rs` — Conditionally append `(skill N)` to food requirement lines

**Dependencies:** Phase 1

**Done when:** Tank summary shows `- 3x flakes (skill 2)` when max skill > 0, shows `- 3x flakes` when skill is 0

**Covers:** feeding-skill.AC3.1, feeding-skill.AC3.2, feeding-skill.AC3.3
<!-- END_PHASE_3 -->
