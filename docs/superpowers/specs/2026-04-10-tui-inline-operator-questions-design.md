# TUI Inline Operator Questions Design

## Goal

Replace the blocking operator-question modal in the TUI with an inline conversation pattern that keeps the transcript visible, keeps the answer controls accessible at the bottom of the chat view, and remains usable even when model-produced question text is formatted poorly.

## Problem

The current `ask_questions` flow renders a modal overlay (`OperatorQuestionOverlay`) that blocks the transcript. This creates two concrete UX failures:

1. The operator cannot inspect the surrounding assistant proposal while deciding.
2. The question UI depends too heavily on model formatting because the protocol provides compact option tokens separately from a free-form `content` blob that often contains both the question and the detailed answer text.

The current modal also duplicates interaction behavior that already exists elsewhere in the TUI. The bottom conversation action surface used by concierge welcome actions is a better interaction class for this feature.

## User Requirements

- The operator must be able to scroll the chat and read the model’s proposal while the question remains answerable.
- The answer controls must live in the bottom chat action surface, not in a blocking modal.
- The transcript must remain the source of truth for the full question content.
- The UI must degrade gracefully when the model formats the question body inconsistently.
- Mouse and keyboard interactions must continue to work.

## Decision

Operator questions will be represented as assistant messages inside the active thread transcript, with compact answer actions exposed through the existing bottom conversation action bar.

The modal-based question overlay will be removed from the interaction path.

## Architecture

### 1. Event handling

When `ClientEvent::OperatorQuestion` arrives:

- Do not push `ModalKind::OperatorQuestionOverlay`.
- Append a new assistant message into the active thread.
- Attach `MessageAction` entries derived from the compact option tokens.
- Mark the message as an operator-question message so it can render with question-specific styling and lifecycle behavior.

This makes the question visible where the decision context already lives: the transcript.

### 2. Bottom action surface

The bottom action bar already renders actions from the most recent assistant message with `actions`.

Operator-question answers should reuse this path:

- each token becomes one action button in the bottom bar
- the selected action remains keyboard-navigable through existing left/right handling
- mouse hit-testing reuses the existing concierge action bar behavior

No parallel operator-question-specific input system should be introduced.

### 3. Transcript card rendering

Operator-question messages should render as a dedicated assistant card in the transcript.

The renderer should attempt to derive three visual regions from `content`:

- question/title
- descriptive body
- answer legend that maps compact tokens such as `A` and `B` to their longer meaning

This parser must be best-effort only. If parsing succeeds, the card becomes clearer and easier to scan. If parsing fails, the transcript still shows the original wrapped content cleanly, and the bottom action bar remains fully usable.

### 4. Answer dispatch

Each bottom-bar action should encode enough information to answer the daemon question without additional state lookup.

Recommended shape:

- `action_type = "operator_question_answer:<question_id>:<token>"`

`run_concierge_action` should decode this action type and send:

- `DaemonCommand::AnswerOperatorQuestion { question_id, answer: token }`

This keeps all answer dispatch on the existing message-action path.

### 5. Resolution behavior

When `ClientEvent::OperatorQuestionResolved` arrives:

- find the pending operator-question message for that `question_id`
- remove its answer actions so it no longer owns the bottom bar
- mark the transcript card as answered
- show the selected token and expanded label if known

The resolved question should stay visible in the transcript for historical continuity instead of disappearing.

## Data Model Changes

`AgentMessage` needs explicit operator-question metadata instead of relying on raw string heuristics alone.

Recommended additions:

- `operator_question_id: Option<String>`
- `operator_question_answer: Option<String>`
- `is_operator_question: bool`

Optional future-friendly extension:

- `operator_question_option_legend: Vec<(String, String)>`

The legend can be derived at event time or render time. The minimal version can keep the raw content and compute the legend lazily in rendering helpers.

## Rendering Strategy

### Transcript

Operator-question transcript cards should:

- use assistant styling, not modal chrome
- visibly indicate pending vs answered state
- prioritize readability over decorative framing
- preserve full original context when parsing is uncertain

The card should never hide the question body behind interaction controls.

### Bottom bar

The bottom bar should stay compact:

- token buttons only
- same navigation model as concierge welcome actions
- same mouse hit-testing behavior

This keeps the bottom surface stable and predictable.

## Parsing Heuristics

The `ask_questions` contract currently encourages models to place the full question and option descriptions in one `content` string while reserving `options` for compact tokens only. Because model output varies, parsing must be tolerant.

The parser should try, in order:

1. detect leading question/title text
2. detect token-prefixed answer descriptions such as `A - ...`, `A. ...`, or `A) ...`
3. build a token-to-description legend only for advertised tokens

If no trustworthy structure is found:

- render the raw content as wrapped markdown/plain text
- still expose the tokens via the bottom bar

This avoids coupling correctness to formatter quality.

## Interaction Rules

- The newest pending operator-question message owns the bottom action bar.
- Older resolved questions remain in transcript history but do not expose active bottom actions.
- The transcript stays scrollable while the question is pending.
- Keyboard left/right and enter continue to work through the existing chat action selection path.
- Mouse clicks on bottom-bar buttons continue to work through the existing concierge action hit-testing path.

## Error Handling

- Unknown or malformed action payloads should be ignored safely.
- If a resolution event arrives for a missing question, do not crash; ignore it.
- If parsing fails, do not block answering.
- If a newer question supersedes an older pending one, only the newest question should remain actionable.

## Out of Scope

- Changing the daemon/tool contract for `ask_questions`
- Redesigning the whole chat transcript layout
- Adding multi-question workflows
- Building a new dedicated decision panel

## Implementation Notes

Likely touch points:

- `crates/amux-tui/src/app/events/events_activity.rs`
- `crates/amux-tui/src/app/model_impl_part2.rs`
- `crates/amux-tui/src/app/rendering.rs`
- `crates/amux-tui/src/state/chat_types.rs`
- `crates/amux-tui/src/state/chat.rs`
- `crates/amux-tui/src/widgets/message.rs`
- `crates/amux-tui/src/widgets/concierge.rs`
- question-related tests in `crates/amux-tui/src/app/tests/`

The old modal rendering path can be removed only after the new inline path is fully wired and covered by tests.

## Testing

Required coverage:

- operator question event appends a transcript message instead of opening a modal
- appended message exposes bottom-bar actions through `chat.active_actions()`
- keyboard selection submits the expected token
- mouse click submits the expected token
- resolved event removes actions and marks the transcript card answered
- malformed question content still renders readable transcript content and working actions
- concierge welcome behavior remains unchanged

## Why This Design

This design matches the operator’s workflow:

- read proposal in transcript
- inspect surrounding context
- answer from the bottom action surface without losing view state

It also uses existing TUI interaction patterns instead of layering another partial control system on top of them.
