# Graph Report - C:\Users\User\Documents\GitHub\stealthmon  (2026-06-15)

## Corpus Check
- 16 files · ~105,624 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 828 nodes · 2293 edges · 19 communities detected
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 55 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 18|Community 18]]

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
Cohesion: 0.03
Nodes (79): addBox(), ao(), at(), be(), beforeDatasetDraw(), beforeDatasetsDraw(), beforeDraw(), beforeLayout() (+71 more)

### Community 1 - "Community 1"
Cohesion: 0.04
Nodes (43): app_distribution(), characters(), daily_avg(), parse_range(), range_to_days(), range_to_hours(), RangeParams, routes() (+35 more)

### Community 2 - "Community 2"
Cohesion: 0.04
Nodes (27): a(), buildLookupTable(), determineDataLimits(), ei(), En, Fo(), _generate(), getDecimalForValue() (+19 more)

### Community 3 - "Community 3"
Cohesion: 0.05
Nodes (29): aa(), b(), buildTicks(), co(), Do(), eo(), Fn(), g() (+21 more)

### Community 4 - "Community 4"
Cohesion: 0.07
Nodes (8): afterDatasetsUpdate(), an(), cn(), ke(), mn(), onClick(), u(), wn()

### Community 5 - "Community 5"
Cohesion: 0.07
Nodes (25): _(), afterDraw(), afterEvent(), afterUpdate(), ai(), ba(), ea(), f() (+17 more)

### Community 6 - "Community 6"
Cohesion: 0.06
Nodes (11): Ae(), Bi(), ci(), d(), Fi(), Ie(), js(), Ue() (+3 more)

### Community 7 - "Community 7"
Cohesion: 0.06
Nodes (34): Bs(), ca(), _calculateBarIndexPixels(), _calculateBarValuePixels(), da(), es(), fa(), ga() (+26 more)

### Community 8 - "Community 8"
Cohesion: 0.08
Nodes (6): As(), k(), ns(), rt(), updateRangeFromParsed(), vs()

### Community 9 - "Community 9"
Cohesion: 0.1
Nodes (13): bt, gt(), jt(), kt(), mt(), qt(), _t(), te() (+5 more)

### Community 10 - "Community 10"
Cohesion: 0.1
Nodes (30): captureScreenshotForVisit(), chromeCallback(), getTab(), isLoggableUrl(), logCompletedVisit(), retryPendingCaptureForActiveTab(), scheduleScreenshotCapture(), shouldSkipDuplicate() (+22 more)

### Community 11 - "Community 11"
Cohesion: 0.09
Nodes (10): ce(), de, dn(), dt(), fe(), ls, nn(), Oe() (+2 more)

### Community 12 - "Community 12"
Cohesion: 0.16
Nodes (2): addElements(), tn

### Community 13 - "Community 13"
Cohesion: 0.22
Nodes (3): Cs, os(), pi()

### Community 14 - "Community 14"
Cohesion: 0.22
Nodes (1): rs

### Community 15 - "Community 15"
Cohesion: 0.67
Nodes (0): 

### Community 16 - "Community 16"
Cohesion: 1.0
Nodes (0): 

### Community 17 - "Community 17"
Cohesion: 1.0
Nodes (0): 

### Community 18 - "Community 18"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **10 isolated node(s):** `InputEvent`, `CharacterStat`, `HourlyStat`, `DailyStat`, `AppShare` (+5 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 16`** (2 nodes): `test_hwnd.rs`, `main()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 17`** (2 nodes): `mouse_distance.rs`, `pixels_to_feet()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 18`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `js()` connect `Community 6` to `Community 0`, `Community 2`, `Community 3`, `Community 5`, `Community 7`, `Community 11`?**
  _High betweenness centrality (0.091) - this node is a cross-community bridge._
- **Why does `shouldSkipDuplicate()` connect `Community 10` to `Community 1`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **Why does `an()` connect `Community 4` to `Community 0`, `Community 2`, `Community 3`, `Community 6`, `Community 11`, `Community 12`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._
- **What connects `InputEvent`, `CharacterStat`, `HourlyStat` to the rest of the system?**
  _10 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.03 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._