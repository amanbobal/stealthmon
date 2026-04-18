# Graph Report - C:\Users\User\Documents\GitHub\stealthmon  (2026-04-19)

## Corpus Check
- 12 files · ~79,955 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 789 nodes · 2223 edges · 18 communities detected
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 49 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]

## God Nodes (most connected - your core abstractions)
1. `js()` - 71 edges
2. `an()` - 61 edges
3. `ns()` - 55 edges
4. `n()` - 38 edges
5. `no` - 32 edges
6. `s()` - 29 edges
7. `va` - 28 edges
8. `o()` - 27 edges
9. `updateElements()` - 25 edges
10. `a()` - 24 edges

## Surprising Connections (you probably didn't know these)
- `init()` --calls--> `setup_logging()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\assets\chart.umd.min.js → C:\Users\User\Documents\GitHub\stealthmon\src\main.rs
- `main()` --calls--> `start_window_collector()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\src\main.rs → C:\Users\User\Documents\GitHub\stealthmon\src\collectors\window.rs
- `main()` --calls--> `start_input_collector()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\src\main.rs → C:\Users\User\Documents\GitHub\stealthmon\src\collectors\input.rs
- `main()` --calls--> `process_input_events()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\src\main.rs → C:\Users\User\Documents\GitHub\stealthmon\src\collectors\input.rs
- `main()` --calls--> `start_server()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\src\main.rs → C:\Users\User\Documents\GitHub\stealthmon\src\server\mod.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.04
Nodes (67): ao(), be(), beforeDatasetDraw(), beforeDatasetsDraw(), beforeDraw(), beforeLayout(), c(), ct() (+59 more)

### Community 1 - "Community 1"
Cohesion: 0.05
Nodes (16): addBox(), afterDatasetsUpdate(), an(), cn(), configure(), dn(), je(), ke() (+8 more)

### Community 2 - "Community 2"
Cohesion: 0.05
Nodes (24): a(), buildLookupTable(), determineDataLimits(), En, Fo(), _generate(), getDecimalForValue(), _getTimestampsForTable() (+16 more)

### Community 3 - "Community 3"
Cohesion: 0.05
Nodes (14): Ae(), Bi(), ci(), d(), fe(), Fi(), gs(), Ie() (+6 more)

### Community 4 - "Community 4"
Cohesion: 0.05
Nodes (37): app_distribution(), characters(), daily_avg(), parse_range(), range_to_days(), range_to_hours(), RangeParams, routes() (+29 more)

### Community 5 - "Community 5"
Cohesion: 0.07
Nodes (28): afterDraw(), afterEvent(), afterUpdate(), ai(), ba(), da(), ea(), f() (+20 more)

### Community 6 - "Community 6"
Cohesion: 0.05
Nodes (20): As(), beforeUpdate(), buildTicks(), Fn(), go(), ii(), initialize(), k() (+12 more)

### Community 7 - "Community 7"
Cohesion: 0.07
Nodes (21): _(), aa(), b(), bo, co(), Do(), eo(), g() (+13 more)

### Community 8 - "Community 8"
Cohesion: 0.07
Nodes (27): Bs(), ca(), _calculateBarIndexPixels(), _calculateBarValuePixels(), es(), getBasePixel(), getLabelAndValue(), getLabelForValue() (+19 more)

### Community 9 - "Community 9"
Cohesion: 0.09
Nodes (13): bt, gt(), jt(), kt(), mt(), qt(), _t(), te() (+5 more)

### Community 10 - "Community 10"
Cohesion: 0.1
Nodes (12): at(), bn, ce(), de, dt(), e(), he(), Oe() (+4 more)

### Community 11 - "Community 11"
Cohesion: 0.11
Nodes (4): addElements(), ia(), qs(), tn

### Community 12 - "Community 12"
Cohesion: 0.15
Nodes (5): Cs, nn(), os(), pi(), sn

### Community 13 - "Community 13"
Cohesion: 0.27
Nodes (2): h(), jn

### Community 14 - "Community 14"
Cohesion: 0.22
Nodes (1): rs

### Community 15 - "Community 15"
Cohesion: 1.0
Nodes (0): 

### Community 16 - "Community 16"
Cohesion: 1.0
Nodes (0): 

### Community 17 - "Community 17"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **10 isolated node(s):** `InputEvent`, `CharacterStat`, `HourlyStat`, `DailyStat`, `AppShare` (+5 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 15`** (2 nodes): `test_hwnd.rs`, `main()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 16`** (2 nodes): `mouse_distance.rs`, `pixels_to_feet()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 17`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `js()` connect `Community 3` to `Community 0`, `Community 1`, `Community 2`, `Community 5`, `Community 6`, `Community 7`, `Community 8`?**
  _High betweenness centrality (0.096) - this node is a cross-community bridge._
- **Why does `an()` connect `Community 1` to `Community 0`, `Community 3`, `Community 7`, `Community 8`, `Community 11`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **Why does `ns()` connect `Community 6` to `Community 0`, `Community 1`, `Community 2`, `Community 3`, `Community 8`, `Community 10`, `Community 12`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **What connects `InputEvent`, `CharacterStat`, `HourlyStat` to the rest of the system?**
  _10 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._