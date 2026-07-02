# Rust Technical Audit Report

## Executive Summary

Repository `ratatui/ratatui` received an overall technical due diligence score of **81/100**.

Cargo workspace with 5 declared member(s) and 42 package manifest(s).

### Weighted Scoring Model

| Area | Weight | Score |
| --- | ---: | ---: |
| Dependency Health | 20% | 64 |
| Code Quality | 25% | 92 |
| Architecture | 25% | 84 |
| Testing | 15% | 100 |
| Risk Posture | 15% | 62 |

## Architecture

- Style: modular Cargo workspace
- Separation of concerns: Top-level module cycles detected; dependency direction should be reviewed.
- Detected layers: multi-crate workspace, nested module tree, crate-relative module fan-out
- Domain boundaries: crate:advanced-widget-impl, crate:async-github, crate:calendar-explorer, crate:canvas, crate:chart, crate:color-explorer, crate:colors-rgb, crate:constraint-explorer, crate:constraints, crate:custom-widget, crate:demo, crate:demo2, crate:flex, crate:gauge, crate:hello-world, crate:hyperlink, crate:inline, crate:input-form, crate:minimal, crate:modifiers, crate:mouse-drawing, crate:panic, crate:popup, crate:ratatui, crate:ratatui-core, crate:ratatui-crossterm, crate:ratatui-macros, crate:ratatui-state-examples, crate:ratatui-termina, crate:ratatui-termion, crate:ratatui-termwiz, crate:ratatui-widgets, crate:release-header, crate:scrollbar, crate:table, crate:todo-list, crate:tracing, crate:user-input, crate:volatility-surface, crate:weather, crate:widget-ref-container, crate:xtask, module:app, module:as_ref, module:backend, module:barchart, module:bin, module:block, module:borders, module:buffer, module:calendar, module:canvas, module:chart, module:clear, module:colors, module:commands, module:crossterm, module:destroy, module:display, module:fill, module:gauge, module:init, module:layout, module:line, module:list, module:logo, module:mascot, module:paragraph, module:polyfills, module:prelude, module:reflow, module:row, module:scrollbar, module:span, module:sparkline, module:style, module:symbols, module:table, module:tabs, module:termina, module:terminal, module:termion, module:termwiz, module:text, module:theme, module:ui, module:volatility, module:widgets

## Dependency Health

- Total direct dependencies: 54
- Critical dependencies: serde in ratatui/Cargo.toml, tokio in examples/apps/async-github/Cargo.toml, serde in examples/apps/input-form/Cargo.toml, serde in ratatui-core/Cargo.toml, serde in ratatui-widgets/Cargo.toml
- Maintenance risks: ratatui uses a non-registry or broad version declaration in ratatui-termwiz/Cargo.toml, ratatui uses a non-registry or broad version declaration in ratatui-termina/Cargo.toml, ratatui uses a non-registry or broad version declaration in ratatui-widgets/Cargo.toml
- Outdated indicators: colorgrad is pinned to pre-1.0 API surface in examples/apps/volatility-surface/Cargo.toml, crossterm_0_28 is pinned to pre-1.0 API surface in ratatui-crossterm/Cargo.toml, crossterm_0_29 is pinned to pre-1.0 API surface in ratatui-crossterm/Cargo.toml

## Code Quality

- Lines of Rust code: 54587
- Rust modules: 240
- Function count: 2998
- Average function size: 13.5 lines
- Large modules: ratatui/tests/widgets_table.rs (842 lines), ratatui-termwiz/src/lib.rs (891 lines), ratatui-termina/src/lib.rs (822 lines), ratatui-core/src/layout/rect.rs (1114 lines), ratatui-core/src/layout/layout.rs (2929 lines), ratatui-core/src/style.rs (1069 lines), ratatui-core/src/terminal/render.rs (865 lines), ratatui-core/src/terminal/inline.rs (929 lines), ratatui-core/src/backend/test.rs (1089 lines), ratatui-core/src/text/line.rs (1729 lines), ratatui-core/src/text/span.rs (904 lines), ratatui-core/src/text/text.rs (1495 lines), ratatui-core/src/buffer/buffer.rs (1582 lines), ratatui-crossterm/src/lib.rs (1171 lines), ratatui-widgets/src/chart.rs (1705 lines), ratatui-widgets/src/tabs.rs (805 lines), ratatui-widgets/src/canvas/world.rs (6298 lines), ratatui-widgets/src/scrollbar.rs (1219 lines), ratatui-widgets/src/canvas.rs (1221 lines), ratatui-widgets/src/block.rs (2371 lines), ratatui-widgets/src/list/rendering.rs (1272 lines), ratatui-widgets/src/table.rs (2717 lines), ratatui-widgets/src/paragraph.rs (1338 lines), ratatui-widgets/src/barchart.rs (1562 lines)
- Potential God modules: ratatui-termion/src/lib.rs, ratatui-termwiz/src/lib.rs, ratatui-termina/src/lib.rs, examples/apps/constraint-explorer/src/main.rs, ratatui-core/src/layout/rect.rs, ratatui-core/src/layout/layout.rs, ratatui-core/src/style/stylize.rs, ratatui-core/src/backend/test.rs, ratatui-core/src/text/line.rs, ratatui-core/src/text/span.rs, ratatui-core/src/text/text.rs, ratatui-core/src/buffer/cell.rs, ratatui-core/src/buffer/buffer.rs, ratatui-crossterm/src/lib.rs, ratatui-widgets/src/chart.rs, ratatui-widgets/src/tabs.rs, ratatui-widgets/src/gauge.rs, ratatui-widgets/src/canvas/world.rs, ratatui-widgets/src/scrollbar.rs, ratatui-widgets/src/canvas.rs, ratatui-widgets/src/block.rs, ratatui-widgets/src/list/rendering.rs, ratatui-widgets/src/table.rs, ratatui-widgets/src/paragraph.rs, ratatui-widgets/src/sparkline.rs, ratatui-widgets/src/barchart.rs

## Testing

- Unit and integration testing structures detected.
- Unit test files: 89
- Integration test files: 19
- Test functions: 1069

## Risks

### Potential God modules (high)

Evidence: ratatui-termion/src/lib.rs, ratatui-termwiz/src/lib.rs, ratatui-termina/src/lib.rs, examples/apps/constraint-explorer/src/main.rs, ratatui-core/src/layout/rect.rs, ratatui-core/src/layout/layout.rs, ratatui-core/src/style/stylize.rs, ratatui-core/src/backend/test.rs, ratatui-core/src/text/line.rs, ratatui-core/src/text/span.rs, ratatui-core/src/text/text.rs, ratatui-core/src/buffer/cell.rs, ratatui-core/src/buffer/buffer.rs, ratatui-crossterm/src/lib.rs, ratatui-widgets/src/chart.rs, ratatui-widgets/src/tabs.rs, ratatui-widgets/src/gauge.rs, ratatui-widgets/src/canvas/world.rs, ratatui-widgets/src/scrollbar.rs, ratatui-widgets/src/canvas.rs, ratatui-widgets/src/block.rs, ratatui-widgets/src/list/rendering.rs, ratatui-widgets/src/table.rs, ratatui-widgets/src/paragraph.rs, ratatui-widgets/src/sparkline.rs, ratatui-widgets/src/barchart.rs

Recommendation: Extract cohesive submodules and isolate orchestration from domain behavior.

### Dependency maintenance risk (medium)

Evidence: ratatui uses a non-registry or broad version declaration in ratatui-termwiz/Cargo.toml; ratatui uses a non-registry or broad version declaration in ratatui-termina/Cargo.toml; ratatui uses a non-registry or broad version declaration in ratatui-widgets/Cargo.toml

Recommendation: Review dependency sourcing, version policy, and upgrade ownership.

### Circular module dependency (medium)

Evidence: cycle among top-level modules: buffer -> layout -> style -> text -> widgets

Recommendation: Break top-level module cycles or move shared contracts behind an explicit boundary.

## Recommendations

- Create a dependency ownership and upgrade policy.
