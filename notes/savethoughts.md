for each levelpack maintain a save file

/[set]/state.json

```json
{
  "version": 1,
  "current_level": "The Ancient Temple",
  "unlocked_levels": [
    "The Ancient Temple",
    "The Lost City"
  ],
  "completed_levels": [
    "The Ancient Temple"
  ],
  "high_scores": {
    "ticks": {
      "The Ancient Temple": 120,
      "The Lost City": 150
    },
    "steps": {
      "The Ancient Temple": 50,
      "The Lost City": 60
    },
    "attempts": {
      "The Ancient Temple": 3,
      "The Lost City": 4
    }
  },
  "settings": {
    "background_music": false,
    "sound_effects": true,
    "developer_mode": false
  }
}
```

```json
```

Replays are stored in

/[set]/replays/level[n].attempt[x].json

```json
{
  "version": 1,
  "record": {
    "ticks": 1086,
    "realtime": 18.816668,
    "steps": 90,
    "bonks": 0,
    "seed": "f304c97992d3cbb5",
    "replay": "/*base64 compressed replay data*/"
  },
  "level": {/*(Optional) the full level data for full compatibility*/
  }
}
```
