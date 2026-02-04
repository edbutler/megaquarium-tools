# Fixture Research

## Source File

Fixtures are defined in:
```
C:/Program Files (x86)/Steam/steamapps/common/Megaquarium/Megaquarium_Data/GameData/Data/scenery.data
```

## Example: 45_rocky_overhang

```json
{
  "id": "45_rocky_overhang",
  "tags": [
    "scenery",
    "cave"
  ],
  "map": {
    "map": "
    xx
    ox
    ",
    "originChar": "x"
  },
  "hosting": {
    "hostedOn": [
      "tank"
    ]
  },
  "immobile": {
    "rotatable": true
  },
  "fbx": {
    "name": "45_rocky_overhang",
    "shadowDetailLevel": 2,
    "guiScale": 0.45,
    "center": {"x": 0.5, "z": 0.5},
    "guiOffset": {
      "y": 20
    },
    "guiRotation": {
      "x": 30.0,
      "y": 125.0,
      "z": 325.0
    }
  },
  "aquascaping": {
    "stats": {
      "isCave": {"value": 8},
      "isRock": {"value": 4}
    },
    "decorationBonus": -3,
    "interactionPosition": {"x": 1, "y": 0.3, "z": 1},
    "interactionAngle": 45
  },
  "unlockable": {
    "autoUnlock": true,
    "availableLevel": 4,
    "category": "scenery"
  },
  "sound": {
    "placeSound": "column_place"
  }
}
```

## Key Fields

- **`id`**: Unique identifier for the fixture
- **`tags`**: Categories like "scenery", "cave"
- **`map`**: Defines the footprint/shape in the tank grid (`x` = occupied, `o` = origin)
- **`hosting.hostedOn`**: Where it can be placed (e.g., "tank")
- **`aquascaping.stats`**: Provides fixture values that satisfy animal requirements (e.g., `isCave: 8`, `isRock: 4`)
- **`aquascaping.fixtureBonus`**: Affects the tank's fixture/prestige score (negative values reduce it)
- **`fbx`**: Visual/rendering properties
- **`unlockable`**: When it becomes available in the game

## Example Animal: 43_bartlett_anthias

Source file: `GameData/Data/animals.data`

```json
{
  "id": "43_bartlett_anthias",
  "tags": [
    "animal",
    "dottybacksGrammasAnthias"
  ],
  "menuTag": "dottybacksGrammasAnthias",
  "perSpecNumbering": true,
  "hosting": {
    "hostedOn": [
      "tank"
    ]
  },
  "contained": {
    "advancedSpotMovement": {
      "speed": 0.006,
      "behaviours": {
        "flickMove": 50,
        "swim": 50
      },
      "flickMoveTurnMagnitude": 60.0,
      "swimLength": 120,
      "slowDownPeriod": 15.0,
      "speedingUpPeriod": 15.0,
      "pauseLength": 100,
      "flickMoveDistance": 0.1,
      "turnFrames": 70,
      "grouping": {
        "minDistance": 0.3,
        "turnSpeed": 0.3,
        "ySpeed": 0.001,
        "catchUpSpeed": 1.3
      },
      "iterations": 1
    },
    "shoalSize": 1
  },
  "bar": {},
  "fbx": {
    "name": "43_bartlett_anthias",
    "scale": 0.217,
    "guiScale": 1.12,
    "guiOffset": {
      "x": 27.0,
      "y": -15.0
    },
    "guiRotation": {
      "x": 30.0,
      "y": 125.0,
      "z": 325.0
    },
    "defaultAnimation": "mill",
    "animationsById": {
      "move": {"name": "swim"},
      "static": {"name": "mill"},
      "dart": {"name": "dart"},
      "feed": {"name": "feed"},
      "turnLeft": {"name": "turnleft"},
      "turnRight": {"name": "turnright"}
    },
    "baseHue": 0.55
  },
  "animal": {
    "stages": [
      {
        "stageName": "adult",
        "size": 3
      }
    ],
    "stats": {
      "ecology": {},
      "eats": {
        "item": "krill"
      },
      "waterQuality": {
        "value": 75
      },
      "shoaler": {
        "req": 7
      },
      "isFish": {},
      "isTropical": {},
      "dislikesFoodCompetitors": {},
      "needsFeedingSkill": {
        "value": 1
      }
    }
  },
  "unlockable": {
    "number": 2,
    "availableLevel": 7,
    "category": "small",
    "howRare": 1
  },
  "sound": {
    "placeSound": "small_fish_place",
    "moveSound": "small_fish_move",
    "turnLeftSound": "small_fish_turn",
    "turnRightSound": "small_fish_turn"
  }
}
```

## Comparison: Fixtures vs Animals

### Common Fields (Shared Structure)

| Field | Fixture | Animal | Notes |
|-------|------------|--------|-------|
| `id` | Yes | Yes | Unique identifier |
| `tags` | Yes | Yes | Categories/groupings |
| `hosting.hostedOn` | Yes | Yes | Both hosted on "tank" |
| `fbx` | Yes | Yes | Visual/rendering properties |
| `unlockable` | Yes | Yes | Unlock progression |
| `sound` | Yes | Yes | Audio effects |

### Fixture-Specific Fields

| Field | Purpose |
|-------|---------|
| `map` | Grid footprint (shape in tank) |
| `immobile.rotatable` | Whether it can be rotated |
| `aquascaping.stats` | **Provides** fixture values (isCave, isRock, etc.) |
| `aquascaping.fixtureBonus` | Affects prestige score |
| `aquascaping.interactionPosition/Angle` | Staff interaction point |

### Animal-Specific Fields

| Field | Purpose |
|-------|---------|
| `menuTag` | UI grouping |
| `perSpecNumbering` | Naming convention |
| `contained` | Movement behavior (speed, swim patterns, shoaling visuals) |
| `bar` | Unknown (possibly health bar?) |
| `animal.stages` | Growth stages with sizes |
| `animal.stats` | **Requires** things like caves, has behavioral traits |
| `animal.stats.eats` | Food requirements |
| `animal.stats.waterQuality` | Water quality requirement |
| `animal.stats.shoaler` | Shoaling requirements |

### Key Insight: Stats Symmetry

The fixture `aquascaping.stats` **provides** values like `isCave: 8`, while the animal `animal.stats` **requires** values like `likesCave: 7` (seen in other fish). This is how the game matches fixtures to animal needs.

**Fixture provides:**
```json
"aquascaping": {
  "stats": {
    "isCave": {"value": 8},
    "isRock": {"value": 4}
  }
}
```

**Animal requires (example from another fish):**
```json
"animal": {
  "stats": {
    "likesCave": {"value": 7}
  }
}
```

The naming convention appears to be:
- Fixtures use `is*` prefix (isRock, isCave, isPlant, etc.)
- Animals use `likes*` prefix (likesRock, likesCave, likesPlant, etc.)

## Additional Findings

- Every scenery item has an `aquascaping` field (20/20 objects in scenery.data)
- Scenery items use `aquascaping.stats` to define their properties with `is*` prefixes (e.g., `isPlant`, `isCave`, `isRock`)
- The `tags` array always has `"scenery"` as the first element, with the second tag indicating subcategory (`"plant"`, `"rock"`, `"cave"`)
