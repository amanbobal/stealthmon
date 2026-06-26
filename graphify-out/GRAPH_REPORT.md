# Graph Report - C:\Users\User\Documents\GitHub\stealthmon  (2026-06-26)

## Corpus Check
- 17 files · ~112,150 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 886 nodes · 2420 edges · 18 communities detected
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 78 edges (avg confidence: 0.8)
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
- `parse_release_version()` --calls--> `parse()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\src\updater.rs → C:\Users\User\Documents\GitHub\stealthmon\assets\chart.umd.min.js
- `main()` --calls--> `start_window_collector()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\src\main.rs → C:\Users\User\Documents\GitHub\stealthmon\src\collectors\window.rs
- `importFile()` --calls--> `importVisits()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\web-logger-extension\src\dashboard.js → C:\Users\User\Documents\GitHub\stealthmon\web-logger-extension\src\shared-db.js
- `refresh()` --calls--> `listVisits()`  [INFERRED]
  C:\Users\User\Documents\GitHub\stealthmon\web-logger-extension\src\dashboard.js → C:\Users\User\Documents\GitHub\stealthmon\web-logger-extension\src\shared-db.js

## Communities

### Community 0 - "Community 0"
Cohesion: 0.03
Nodes (62): app_distribution(), characters(), daily_avg(), ingest_web_history(), most_visited_website(), parse_range(), range_to_days(), range_to_hours() (+54 more)

### Community 1 - "Community 1"
Cohesion: 0.04
Nodes (79): _(), aa(), addBox(), afterDatasetsUpdate(), afterEvent(), ao(), at(), b() (+71 more)

### Community 2 - "Community 2"
Cohesion: 0.04
Nodes (35): Ae(), As(), Bs(), _calculateBarIndexPixels(), _calculateBarValuePixels(), ci(), Fi(), Fn() (+27 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (35): afterDraw(), ai(), ba(), da(), ea(), fa(), ft(), gs() (+27 more)

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (10): an(), cn(), f(), generateLabels(), jn, ke(), onClick(), removeBox() (+2 more)

### Community 5 - "Community 5"
Cohesion: 0.05
Nodes (23): buildLookupTable(), ei(), En, Fo(), _generate(), getDecimalForValue(), _getTimestampsForTable(), ia() (+15 more)

### Community 6 - "Community 6"
Cohesion: 0.05
Nodes (18): bo, buildTicks(), ca(), co(), Do(), eo(), et(), getBasePixel() (+10 more)

### Community 7 - "Community 7"
Cohesion: 0.07
Nodes (7): afterUpdate(), d(), js(), Ue(), Us(), w(), Xs()

### Community 8 - "Community 8"
Cohesion: 0.06
Nodes (14): bn, Cs, dn(), fe(), k(), nn(), on(), os() (+6 more)

### Community 9 - "Community 9"
Cohesion: 0.08
Nodes (16): Bi(), bt, gt(), it(), jt(), kt(), mt(), pt() (+8 more)

### Community 10 - "Community 10"
Cohesion: 0.08
Nodes (13): a(), beforeLayout(), ce(), de, determineDataLimits(), dt(), getValueForPixel(), ko (+5 more)

### Community 11 - "Community 11"
Cohesion: 0.12
Nodes (33): captureScreenshotForVisit(), chromeCallback(), getTab(), isLoggableUrl(), logCompletedVisit(), pingStealthMon(), queueStealthMonSync(), retryPendingCaptureForActiveTab() (+25 more)

### Community 12 - "Community 12"
Cohesion: 0.13
Nodes (3): addElements(), qs(), tn

### Community 13 - "Community 13"
Cohesion: 0.16
Nodes (14): chromeDownload(), downloadFormattedChunks(), downloadText(), escapeHtml(), exportFilenameBase(), exportRows(), importFile(), parseCsv() (+6 more)

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
- **17 isolated node(s):** `UpdateStatus`, `GitHubRelease`, `GitHubAsset`, `InputEvent`, `CharacterStat` (+12 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 15`** (2 nodes): `test_hwnd.rs`, `main()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 16`** (2 nodes): `mouse_distance.rs`, `pixels_to_feet()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 17`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `importVisits()` connect `Community 11` to `Community 9`, `Community 5`, `Community 13`?**
  _High betweenness centrality (0.086) - this node is a cross-community bridge._
- **Why does `js()` connect `Community 7` to `Community 1`, `Community 2`, `Community 3`, `Community 4`, `Community 6`, `Community 10`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **Why does `an()` connect `Community 4` to `Community 1`, `Community 2`, `Community 3`, `Community 5`, `Community 6`, `Community 8`, `Community 12`?**
  _High betweenness centrality (0.069) - this node is a cross-community bridge._
- **What connects `UpdateStatus`, `GitHubRelease`, `GitHubAsset` to the rest of the system?**
  _17 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.03 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._