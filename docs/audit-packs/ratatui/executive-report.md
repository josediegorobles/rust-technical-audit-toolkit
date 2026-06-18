# Rust Technical Audit Report

## Executive Summary

Repository `ratatui/ratatui` received an overall technical due diligence score of **47/100**.

Cargo workspace with 5 declared member(s) and 42 package manifest(s).

### Weighted Scoring Model

| Area | Weight | Score |
| --- | ---: | ---: |
| Dependency Health | 20% | 49 |
| Code Quality | 25% | 8 |
| Architecture | 25% | 50 |
| Testing | 15% | 100 |
| Risk Posture | 15% | 52 |

## Architecture

- Style: modular Cargo workspace
- Separation of concerns: Limited explicit layer separation detected.
- Detected layers: none detected
- Domain boundaries: none detected

## Dependency Health

- Total direct dependencies: 63
- Critical dependencies: serde in ratatui/Cargo.toml, tokio in examples/apps/async-github/Cargo.toml, serde in ratatui-core/Cargo.toml, serde in ratatui-widgets/Cargo.toml
- Maintenance risks: ratatui uses a non-registry or broad version declaration in ratatui-termwiz/Cargo.toml, ratatui uses a non-registry or broad version declaration in ratatui-termina/Cargo.toml, ratatui uses a non-registry or broad version declaration in ratatui-widgets/Cargo.toml
- Outdated indicators: colorgrad is pinned to pre-1.0 API surface in examples/apps/volatility-surface/Cargo.toml, crossterm_0_28 is pinned to pre-1.0 API surface in ratatui-crossterm/Cargo.toml, crossterm_0_29 is pinned to pre-1.0 API surface in ratatui-crossterm/Cargo.toml

## Code Quality

- Lines of Rust code: 54584
- Rust modules: 240
- Function count: 3285
- Average function size: 13.0 lines
- Large modules: ratatui-termion/src/lib.rs (745 lines), ratatui/tests/widgets_block.rs (485 lines), ratatui/tests/widgets_table.rs (842 lines), ratatui/tests/terminal.rs (482 lines), ratatui/tests/widgets_paragraph.rs (353 lines), ratatui/tests/widgets_chart.rs (410 lines), ratatui/tests/widgets_list.rs (371 lines), ratatui/src/lib.rs (521 lines), ratatui/src/init.rs (572 lines), ratatui/src/widgets.rs (771 lines), ratatui-termwiz/src/lib.rs (891 lines), ratatui-termina/src/lib.rs (822 lines), examples/apps/demo/src/ui.rs (405 lines), examples/apps/demo/src/app.rs (344 lines), examples/apps/volatility-surface/src/display/surface_3d.rs (327 lines), examples/apps/constraints/src/main.rs (388 lines), examples/apps/constraint-explorer/src/main.rs (605 lines), examples/apps/chart/src/main.rs (352 lines), examples/apps/table/src/main.rs (374 lines), examples/apps/flex/src/main.rs (617 lines), ratatui-core/src/layout.rs (333 lines), ratatui-core/src/backend.rs (433 lines), ratatui-core/src/layout/rect.rs (1114 lines), ratatui-core/src/layout/layout.rs (2926 lines), ratatui-core/src/layout/rect/iter.rs (356 lines), ratatui-core/src/layout/constraint.rs (526 lines), ratatui-core/src/terminal.rs (487 lines), ratatui-core/src/style.rs (1069 lines), ratatui-core/src/terminal/render.rs (865 lines), ratatui-core/src/terminal/buffers.rs (448 lines), ratatui-core/src/terminal/inline.rs (929 lines), ratatui-core/src/terminal/resize.rs (340 lines), ratatui-core/src/style/color.rs (788 lines), ratatui-core/src/style/anstyle.rs (361 lines), ratatui-core/src/style/stylize.rs (668 lines), ratatui-core/src/style/palette/material.rs (608 lines), ratatui-core/src/style/palette/tailwind.rs (653 lines), ratatui-core/src/backend/test.rs (1089 lines), ratatui-core/src/symbols/merge.rs (748 lines), ratatui-core/src/symbols/border.rs (709 lines), ratatui-core/src/text/line.rs (1729 lines), ratatui-core/src/text/span.rs (904 lines), ratatui-core/src/text/text.rs (1495 lines), ratatui-core/src/buffer/cell.rs (458 lines), ratatui-core/src/buffer/buffer.rs (1582 lines), ratatui-core/src/buffer/diff.rs (585 lines), ratatui-crossterm/src/lib.rs (1171 lines), ratatui-widgets/src/chart.rs (1705 lines), ratatui-widgets/src/barchart/bar.rs (336 lines), ratatui-widgets/src/borders.rs (307 lines), ratatui-widgets/src/list.rs (657 lines), ratatui-widgets/src/tabs.rs (805 lines), ratatui-widgets/src/gauge.rs (623 lines), ratatui-widgets/src/canvas/line.rs (603 lines), ratatui-widgets/src/canvas/world.rs (6298 lines), ratatui-widgets/src/scrollbar.rs (1219 lines), ratatui-widgets/src/table/state.rs (745 lines), ratatui-widgets/src/table/row.rs (345 lines), ratatui-widgets/src/canvas.rs (1221 lines), ratatui-widgets/src/block.rs (2371 lines), ratatui-widgets/src/list/rendering.rs (1272 lines), ratatui-widgets/src/list/state.rs (357 lines), ratatui-widgets/src/list/item.rs (353 lines), ratatui-widgets/src/table.rs (2717 lines), ratatui-widgets/src/paragraph.rs (1338 lines), ratatui-widgets/src/calendar.rs (458 lines), ratatui-widgets/src/reflow.rs (729 lines), ratatui-widgets/src/sparkline.rs (751 lines), ratatui-widgets/src/barchart.rs (1562 lines), ratatui-widgets/src/block/shadow.rs (453 lines)
- Potential God modules: ratatui-termion/src/lib.rs, ratatui/tests/widgets_table.rs, ratatui/src/lib.rs, ratatui/src/init.rs, ratatui/src/widgets.rs, ratatui-termwiz/src/lib.rs, ratatui-termina/src/lib.rs, examples/apps/constraint-explorer/src/main.rs, examples/apps/flex/src/main.rs, ratatui-core/src/layout/rect.rs, ratatui-core/src/layout/layout.rs, ratatui-core/src/layout/constraint.rs, ratatui-core/src/style.rs, ratatui-core/src/terminal/render.rs, ratatui-core/src/terminal/inline.rs, ratatui-core/src/style/color.rs, ratatui-core/src/style/stylize.rs, ratatui-core/src/style/palette/material.rs, ratatui-core/src/style/palette/tailwind.rs, ratatui-core/src/backend/test.rs, ratatui-core/src/symbols/merge.rs, ratatui-core/src/symbols/border.rs, ratatui-core/src/text/line.rs, ratatui-core/src/text/span.rs, ratatui-core/src/text/text.rs, ratatui-core/src/buffer/cell.rs, ratatui-core/src/buffer/buffer.rs, ratatui-core/src/buffer/diff.rs, ratatui-crossterm/src/lib.rs, ratatui-widgets/src/chart.rs, ratatui-widgets/src/list.rs, ratatui-widgets/src/tabs.rs, ratatui-widgets/src/gauge.rs, ratatui-widgets/src/canvas/line.rs, ratatui-widgets/src/canvas/world.rs, ratatui-widgets/src/scrollbar.rs, ratatui-widgets/src/table/state.rs, ratatui-widgets/src/canvas.rs, ratatui-widgets/src/block.rs, ratatui-widgets/src/list/rendering.rs, ratatui-widgets/src/table.rs, ratatui-widgets/src/paragraph.rs, ratatui-widgets/src/reflow.rs, ratatui-widgets/src/sparkline.rs, ratatui-widgets/src/barchart.rs, ratatui-widgets/src/block/shadow.rs

## Testing

- Unit and integration testing structures detected.
- Unit test files: 89
- Integration test files: 19
- Test functions: 1069

## Risks

### Potential God modules (high)

Evidence: ratatui-termion/src/lib.rs, ratatui/tests/widgets_table.rs, ratatui/src/lib.rs, ratatui/src/init.rs, ratatui/src/widgets.rs, ratatui-termwiz/src/lib.rs, ratatui-termina/src/lib.rs, examples/apps/constraint-explorer/src/main.rs, examples/apps/flex/src/main.rs, ratatui-core/src/layout/rect.rs, ratatui-core/src/layout/layout.rs, ratatui-core/src/layout/constraint.rs, ratatui-core/src/style.rs, ratatui-core/src/terminal/render.rs, ratatui-core/src/terminal/inline.rs, ratatui-core/src/style/color.rs, ratatui-core/src/style/stylize.rs, ratatui-core/src/style/palette/material.rs, ratatui-core/src/style/palette/tailwind.rs, ratatui-core/src/backend/test.rs, ratatui-core/src/symbols/merge.rs, ratatui-core/src/symbols/border.rs, ratatui-core/src/text/line.rs, ratatui-core/src/text/span.rs, ratatui-core/src/text/text.rs, ratatui-core/src/buffer/cell.rs, ratatui-core/src/buffer/buffer.rs, ratatui-core/src/buffer/diff.rs, ratatui-crossterm/src/lib.rs, ratatui-widgets/src/chart.rs, ratatui-widgets/src/list.rs, ratatui-widgets/src/tabs.rs, ratatui-widgets/src/gauge.rs, ratatui-widgets/src/canvas/line.rs, ratatui-widgets/src/canvas/world.rs, ratatui-widgets/src/scrollbar.rs, ratatui-widgets/src/table/state.rs, ratatui-widgets/src/canvas.rs, ratatui-widgets/src/block.rs, ratatui-widgets/src/list/rendering.rs, ratatui-widgets/src/table.rs, ratatui-widgets/src/paragraph.rs, ratatui-widgets/src/reflow.rs, ratatui-widgets/src/sparkline.rs, ratatui-widgets/src/barchart.rs, ratatui-widgets/src/block/shadow.rs

Recommendation: Extract cohesive submodules and isolate orchestration from domain behavior.

### Dependency maintenance risk (medium)

Evidence: ratatui uses a non-registry or broad version declaration in ratatui-termwiz/Cargo.toml; ratatui uses a non-registry or broad version declaration in ratatui-termina/Cargo.toml; ratatui uses a non-registry or broad version declaration in ratatui-widgets/Cargo.toml

Recommendation: Review dependency sourcing, version policy, and upgrade ownership.

### Excessive dependency concentration (medium)

Evidence: 63 direct dependencies were detected.

Recommendation: Identify strategic dependencies and remove low-value transitive surface area.

### Circular dependency risk (medium)

Evidence: ratatui/benches/main.rs centralizes 26 module declarations; ratatui-widgets/src/lib.rs centralizes 37 module declarations

Recommendation: Review module directionality and enforce dependency rules at crate boundaries.

## Recommendations

- Document intended architectural boundaries and enforce them through crate or module ownership.
- Create a dependency ownership and upgrade policy.
- Prioritize decomposition of large modules and high-branching functions.
