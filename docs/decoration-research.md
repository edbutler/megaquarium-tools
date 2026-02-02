# Decoration Research

## Source File

Decorations are defined in:
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

- **`id`**: Unique identifier for the decoration
- **`tags`**: Categories like "scenery", "cave"
- **`map`**: Defines the footprint/shape in the tank grid (`x` = occupied, `o` = origin)
- **`hosting.hostedOn`**: Where it can be placed (e.g., "tank")
- **`aquascaping.stats`**: Provides decoration values that satisfy animal requirements (e.g., `isCave: 8`, `isRock: 4`)
- **`aquascaping.decorationBonus`**: Affects the tank's decoration/prestige score (negative values reduce it)
- **`fbx`**: Visual/rendering properties
- **`unlockable`**: When it becomes available in the game
