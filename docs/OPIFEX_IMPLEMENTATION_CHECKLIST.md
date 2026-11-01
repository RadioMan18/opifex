# Opifex Implementation Checklist

This checklist converts the Opifex design direction into an executable build plan. It is designed for a human or another agent to work through in priority order without re-reading the full design spec.

Related design spec:
- [OPIFEX_UI_DESIGN_SPEC.md](./OPIFEX_UI_DESIGN_SPEC.md)

## Execution Principles

- Ship the structural fixes before cosmetic refinements.
- Fix layout, information hierarchy, and state clarity before adding richer visual polish.
- Keep the product language consistent across chat, canvas, and plan surfaces.
- Treat the top bar simplification as a foundational fix, not a later tweak.
- Validate with real usage scenarios: create agent, review plan, edit block, save version, retry flow.

## Actual Vue Component Targets

Use these exact files first; they are the front door for the redesign work already in the app:

- App shell and global layout: `src/App.vue`
- Chat workflow: `src/components/ChatPane.vue`
- Live plan review: `src/components/PlanPane.vue`
- Builder canvas: `src/components/AgentCanvas.vue`
- Block palette: `src/components/BlockPalette.vue`
- Node config inspector: `src/components/NodeInspector.vue`
- Agent library / first-run state: `src/components/AgentLibrary.vue`
- Model selection: `src/components/ModelSelector.vue`
- Pin / workflow settings: `src/components/PinPanel.vue`
- Agent store state logic: `src/stores/agentStore.ts`
- Graph/canvas state logic: `src/stores/graphStore.ts`
- Canvas node visuals: `src/components/canvas/AgentNode.vue`, `src/components/canvas/ConnectorNode.vue`, `src/components/canvas/StepEdge.vue`
- Inspector sub-panels: `src/components/inspector/SkillEditor.vue`, `KnowledgeSourceEditor.vue`, `TriggerEditor.vue`, `AgentToolDrawer.vue`
- Shared types: `src/types/agent.ts`

This is the shortest path for the design work without broad repo search:

- Top-bar and app shell fixes live in `src/App.vue`.
- Chat experience changes live in `src/components/ChatPane.vue`.
- Plan/agent review changes live in `src/components/PlanPane.vue`.
- Builder and node editing changes live in `src/components/AgentCanvas.vue` and `src/components/NodeInspector.vue`.
- Palette and block organization live in `src/components/BlockPalette.vue`.
- Shared state and behavior changes live in the stores and type definitions.

---

## Priority 0 — Foundation and Shell Fixes

### P0.1: Simplify the top bar action model

- [x] Replace the crowded right-side cluster with a strict priority system:
  - Primary: New Agent, active mode toggle
  - Secondary: utility menu / overflow
  - Tertiary: less frequent or advanced settings
- [x] Move utility actions into a compact menu pattern (ellipsis / kebab menu or utility dropdown)
- [x] Remove edge crowding and clipped controls from the right side of the app shell
- [x] Ensure the layout uses a stable grid structure, not compensating negative spacing
- [x] Validate the shell at narrow and wide desktop sizes

Files likely involved:
- `src/App.vue`
- shell-level layout styles
- shared action/button components

Acceptance criteria:
- the right-side cluster no longer compresses or clips
- the active action set is visually prioritized
- secondary controls do not compete with primary actions

### P0.2: Establish the app shell hierarchy

- [x] Define the core three-part shell:
  - top bar
  - primary workspace
  - side/detail panel
- [x] Ensure the hierarchy reads as a serious workspace, not a dashboard or chatbot wrapper
- [x] Standardize panel spacing, border treatment, and header rhythm across the app shell
- [x] Separate app-wide navigation from workspace actions

Files likely involved:
- `src/App.vue`
- global layout styles
- shared page shell components

Acceptance criteria:
- users immediately understand where the app content lives
- the top bar feels stable and premium
- whitespace and panel borders convey hierarchy without excess decoration

### P0.3: Define and implement the base design tokens

- [x] Create a shared token set for:
  - background colors
  - panel surfaces
  - borders
  - text colors
  - accent color
  - success/warning/error states
  - spacing scale
  - radius scale
  - shadow / elevation
- [x] Use the same tokens across all major surfaces
- [x] Replace ad hoc color values with semantic tokens
- [x] Standardize spacing and radius rules across the app

Files likely involved:
- `src/styles/*` or equivalent
- shared theme file
- root app styling

Acceptance criteria:
- color usage is consistent across chat, canvas, and plan surfaces
- no arbitrary one-off values remain in core shell-level styling
- spacing rhythm is consistent across components

---

## Priority 1 — Core Experience and Information Architecture

### P1.1: Clarify the product story across surfaces

- [x] Present the app as one coherent workflow:
  - chat interviews the user
  - canvas defines the system
  - plan shows the artifact being built
- [x] Ensure these are visually and conceptually linked, not treated as unrelated views
- [x] Give each surface a distinct role but shared language

Files likely involved:
- `src/App.vue`
- `src/components/ChatPane.vue`
- `src/components/PlanPane.vue`
- canvas or builder components

Acceptance criteria:
- users can tell when they are in chat mode vs builder mode vs review mode
- the product reads as a single system-building flow

### P1.2: Rework the Chat Pane into a guided expert workflow

- [x] Rename or restyle assistant states so they read as purposeful system-building phases
- [x] Add structured message states such as:
  - understanding goal
  - clarifying requirements
  - building plan
  - validating plan
  - ready to edit
- [x] Improve header treatment so the assistant or workflow state feels intentional
- [x] Make the message list feel like a conversation with structure, not a plain transcript
- [x] Add stronger affordances for actions such as Build Agent / Retry / Continue / Review Plan

Files likely involved:
- `src/components/ChatPane.vue`
- message card components
- assistant status indicators

Acceptance criteria:
- users understand what the system is doing at each step
- assistant messages feel trustworthy and deliberate
- the chat workspace no longer feels generic or utility-only

### P1.3: Reframe the Plan Panel as the primary product artifact

- [x] Group the plan into clear, high-signal sections:
  - Objective
  - Role and scope
  - Tools and capabilities
  - Data and knowledge sources
  - Workflow steps
  - Triggers
  - Outputs
  - Risks / assumptions
  - Version state
- [x] Give each section strong heading hierarchy and consistent card treatment
- [x] Distinguish complete, incomplete, warning, and risky states visually
- [x] Ensure the plan is easy to scan at a glance

Files likely involved:
- `src/components/PlanPane.vue`
- plan rendering utilities
- any JSON-to-card mapping components

Acceptance criteria:
- the plan panel is a clear product artifact, not a side stack of raw data
- users can review the build with confidence
- warnings and missing info are visible and understandable

### P1.4: Standardize component states

- [x] Implement a consistent state system for:
  - idle
  - active
  - loading
  - selected
  - warning
  - error
  - complete
  - saved
  - draft
  - disabled
- [x] Apply the same semantic state treatment across buttons, cards, panels, and status pills
- [x] Ensure state changes are visible without requiring interpretation of code

Files likely involved:
- shared UI component layer
- status badge components
- button variants

Acceptance criteria:
- the UI communicates state clearly everywhere
- state semantics are consistent across the product

---

## Priority 2 — Canvas and Builder Experience

### P2.1: Rework the Agent Canvas foundation

- [x] Give the canvas a stronger workspace identity
- [x] Establish a clear left-palette / center-workspace / right-inspector layout
- [x] Ensure selected blocks are visually obvious
- [x] Improve connection readability between steps or blocks
- [x] Reduce the feeling of a generic graph editor

Files likely involved:
- canvas builder components
- block library/palette
- canvas state/selection layers

Acceptance criteria:
- the hierarchy of the canvas is clear at a glance
- block categories are recognizable and scannable
- active selection and editing context feel deliberate

### P2.2: Improve block palette organization

- [x] Categorize blocks into meaningful groups such as:
  - Triggers
  - LLM
  - Tools
  - Agents
  - Outputs
  - Control Flow
  - Utilities
- [x] Give each block a compact name, category, and clear icon/shape
- [x] Avoid a visually dense, unlabeled wall of blocks
- [x] Keep the palette calm and structured

Files likely involved:
- block palette component
- palette categories
- block metadata definitions

Acceptance criteria:
- the palette communicates structure instantly
- users can find the right kind of block without scanning too much
- the palette is usable without visual fatigue

### P2.3: Upgrade the inspector and editing drawer

- [x] Group settings into semantic categories
- [x] Reduce long form-heavy panels
- [x] Keep the most important controls visible and high-priority
- [x] Use inline validation for missing or risky values
- [x] Make destructive or risky operations explicit

Files likely involved:
- inspector drawer
- block property editors
- validation utilities

Acceptance criteria:
- editing tasks are less cognitively heavy
- users know exactly what they are changing
- form structure matches the logic they are editing

---

## Priority 3 — Product Polish and Trust Signals

### P3.1: Strengthen empty and onboarding states

- [x] Add a strong empty state for no agents
- [x] Add a strong onboarding state for first-time users
- [x] Offer starter templates and “Create Agent” clearly
- [x] Make the first-run path feel guided, not overloaded

Files likely involved:
- agent library component
- first-run onboarding screens
- starter template cards

Acceptance criteria:
- the first run feels intentional and low-friction
- users understand what to do next without reading documentation

### P3.2: Improve status and confirmation patterns

- [x] Add consistent save / export / import / duplicate confirmation flows
- [x] Use non-blocking status messages for successful actions
- [x] Add minimal but clear confirmation copy for destructive operations
- [x] Ensure important states are readable without overusing modals

Files likely involved:
- common notifications/toasts
- confirmation dialogs
- save and export actions

Acceptance criteria:
- recurring user actions produce clear feedback
- the app feels trustworthy and operationally stable

### P3.3: Enrich the review and validation story

- [x] Show summary banners or inline warnings for uncertain states
- [x] Clarify assumptions and missing inputs in the plan
- [x] Add visible confidence or readiness markers
- [x] Ensure the system invites careful review before launch

Files likely involved:
- plan section metadata
- validation layer
- assistant output summarization

Acceptance criteria:
- users can tell when an agent is ready or incomplete
- uncertainty is visible without being alarming

---

## Priority 4 — Motion, Accessibility, and Density Discipline

### P4.1: Add motion deliberately

- [ ] Use subtle animation for panel transitions, selection states, and loading feedback
- [ ] Keep motion restrained and purposeful
- [ ] Avoid decorative or distracting movement
- [ ] Respect reduced-motion preferences

Files likely involved:
- shared motion settings
- component hover/selection transitions
- loading animations

Acceptance criteria:
- motion helps comprehension rather than distracting from the work

### P4.2: Accessibility pass

- [ ] Verify text contrast in all states
- [ ] Improve focus ring visibility and keyboard flow
- [ ] Ensure all actionable controls have clear labels
- [ ] Test keyboard navigation for top bar, library, canvas, and inspector
- [ ] Confirm reduced-motion compatibility

Files likely involved:
- all major components
- accessibility utilities
- shared controls

Acceptance criteria:
- the UI is legible and navigable without pointer-only interaction
- all critical controls are clear and operable

### P4.3: Reduce density and increase clarity

- [ ] Remove unnecessary controls from the visible main shell
- [ ] Reduce noise in the right-side action rail
- [ ] Keep each surface visually calm enough to support decision-making
- [ ] Ensure spacing supports scanning rather than crowding

Files likely involved:
- all app shell and panel layout code
- utility menu and overflow patterns

Acceptance criteria:
- the interface feels premium and controlled rather than crowded
- the product reads as a serious design tool, not a packed utility panel

---

## Suggested Execution Order for an Agent

### Phase 1 — Structural fix

1. Top bar simplification and utility menu
2. App shell hierarchy and panel spacing
3. Shared tokens and state system
4. Main layout validation

### Phase 2 — Experience clarity

5. Chat experience improvements
6. Plan panel hierarchy and review treatment
7. Agent library and empty states
8. Shared action and feedback patterns

### Phase 3 — Builder polish

9. Canvas workspace hierarchy
10. Block palette structure
11. Inspector clarity and editing ergonomics
12. Validation and warnings

### Phase 4 — Premium quality pass

13. Motion tuning
14. Accessibility adjustments
15. Responsive density checks
16. Final UX review against the design standard

---

## Reference Acceptance Review

Before considering the redesign complete, validate against these prompts:

- Can a first-time user understand the product in under 30 seconds?
- Can a user create a new agent without feeling lost?
- Does the plan read as a trustworthy artifact, not a raw dump?
- Does the top bar feel calm and intentional instead of crowded?
- Do chat, canvas, and plan surfaces feel like one coherent system?
- Does the product communicate technical confidence and craft?

If the answer to any of those is no, the design is not ready.

---

## Recommended Implementation Notes for the Next Agent

- Do not start with micro-adjustments. Fix the layout structure first.
- Treat utility actions as a second-order concern; do not flood the top bar with them.
- Use the shared design tokens before creating any more ad hoc styling.
- The plan panel should feel premium and editorial, not spreadsheet-like.
- The chat should feel like an expert interview, not a generic chat box.
- The canvas should feel like a design studio, not just a graph editor.
- The final result should be premium, high-trust, and operationally clear.
