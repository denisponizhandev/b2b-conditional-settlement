# Agent Instructions

All platform modules are built from this repository root.

The **developer** owns product decisions, architecture approval, and verification.  
The **agent** implements code **only when explicitly requested**, after design analysis and developer agreement.

These instructions apply to every agent session in this project.

---

## 1. Always follow (source of truth)

1. **These instructions** — this file and `.cursor/rules/`
2. **Project goal** — [README.md](./README.md)
3. **Agreed architecture** — [ARCHITECTURE.md](./ARCHITECTURE.md)
4. **Developer's latest message** — when it does not conflict with the above

Do not invent goals, modules, or architecture. When unsure, read README and ARCHITECTURE or ask.

Before any code task, read relevant existing files in the repo so new work fits current structure and dependencies.

---

## 2. Code — two modes (critical)

### Mode A — Code in chat (default when asked)

When the developer asks for code **in chat** — paste, show, give — **without** asking to edit the repo:

| Developer says (examples) | Agent does | Agent must NOT |
|---------------------------|------------|----------------|
| *«дай код в чате»*, *«напиши код сюда»*, *«give me the code»*, *«покажи полный код»*, *«чтобы я сам вставил»* | Full or partial code **in the chat message** + explain + verify commands | Create, edit, or delete files in the repo |
| *«весь нужный код чтобы закрыть этап **в этом чате**»* | Same — **chat only** | Touch the repo |

Chat delivery still includes: pattern blocks, security notes, which file each block belongs to, and terminal commands to run **after the developer pastes**.

**“Code in chat” is not permission to edit files.** Only explicit repo action verbs grant that (Mode B).

### Mode B — Edit the repository

**The agent must not create, edit, or delete implementation files** unless the developer **explicitly asks to change the repo** in that message.

Implementation files include: `.sol`, `.rs`, `.ts`, SQL migrations, configs, tests, Docker/CI files, and similar.

| Mode A — chat only | Mode B — edit repo |
|--------------------|--------------------|
| Explain, propose, paste code in chat | Write or edit source files |
| Review code the developer wrote | Scaffold modules in the repo |
| Pseudocode and file-shaped snippets for copy-paste | Apply migrations or change deps in the repo |

Explicit **repo** permission examples: *«implement it»*, *«edit `deal_repo.rs`»*, *«запиши в файл»*, *«сделай в репо»*, *«apply the changes»*.

**Even in Mode B:** keep changes **minimal and scoped** to the agreed task. Do not touch unrelated files.

**If ambiguous** (e.g. *«напиши insert_deal»* without *«в чате»* or *«в файл»*): prefer **Mode A (chat)** and ask whether to apply to the repo — do not edit silently.

### Code comments (required whenever code is produced)

All code the agent writes — in chat or in the repo — must include comments. **Do not paste or commit uncommented code.**

| Mode | Comment style | Language |
|------|---------------|----------|
| **A — chat** | **Pedagogical / line-level** — checklist ниже; каждый `.bind`, `.await`, `?`, `&mut *tx` и т.д. | Russian (identifiers and API names stay as in code) |
| **B — repo** | **Brief** — одна короткая строка на функцию / блок | Russian or concise EN |

#### Mode A — pedagogical comment checklist (mandatory)

Comments must teach **Rust + library mechanics**, not only business intent. For **every** non-trivial expression explain:

1. **What** — что делает API/конструкция  
2. **How** — как работает в runtime  
3. **Why** — почему именно эта форма здесь  
4. **Types / ownership** — `&Deal`, `Result`, `Option`, `mut tx`  
5. **Syntax** — нез familiar запись (`&mut *tx`, turbofish)

**Must explain explicitly when they appear:**

| Construct | Comment must cover |
|-----------|-------------------|
| `.bind(value)` | `$1..$N`; значение уходит **отдельно** от SQL-текста → нет injection; **порядок** `.bind` = порядок `$1, $2, …` |
| `.execute(...)` vs `.fetch_*` | INSERT без чтения строк; возвращает `PgQueryResult` |
| `.fetch_optional` / `.fetch_all` | 0-or-1 строка vs много строк |
| `&mut *tx` | `tx` = `Transaction`; `*tx` deref; `&mut` = mutable borrow для `Executor`; **не** `&pool` — иначе вне transaction |
| `.await` | Левая часть — `Future`; suspend до ответа Postgres; поток не блокируется |
| `?` | Ok → значение; Err → **немедленный return** из функции; `From` для `DbError` |
| `&self` / `&Deal` | Borrow — читаем без move ownership |
| `as i16` / `as i64` | Cast domain type → тип колонки Postgres |
| `begin()` / `commit()` | Граница TX; drop без commit = ROLLBACK |
| `#[tokio::test]` | Async runtime для теста |
| `.clone()` on pool | Дешёвый clone (`Arc` внутри) |

**Incomplete comments fail Mode A.** Wrong: `// bind id`. Required: placeholder, order, type, FK.

**Mode B:** те же темы, **одна строка** — без лекций в repo.

**Do not** comment closing braces alone. **Do** comment every method-chain step and every operator the developer is learning.

---

## 3. Workflow for every code task

Each code request follows this sequence. **Do not skip steps.**

### Step A — Context and alignment

Before proposing or writing anything:

1. Re-read [README.md](./README.md), [ARCHITECTURE.md](./ARCHITECTURE.md), and applicable `.cursor/rules/`
2. Identify which **layer** is affected (`contracts`, `domain`, `db`, `indexer`, `platform-api`, …)
3. Check **dependencies** — what already exists upstream/downstream; what this slice must not break

### Step B — Design analysis (mandatory, in chat)

Before implementation, explain to the developer:

- **What** we are building or changing and **why** (product + engineering reason)
- **Design patterns** using the **full pattern block** from `.cursor/rules/architecture-language.mdc` — not name-dropping alone
- **Layer responsibility** — what this module owns vs what it must not own
- **How it fits** the on-chain ↔ off-chain settlement story

Every non-trivial decision must be explained so the developer knows **which part of the code is responsible for what**.

**Do not** reduce patterns to a title line (e.g. “Repository + transaction”). The developer must understand **what the pattern is, what problem it solves, why we use it here, and what breaks without it**.

### Step C — Proposal and agreement (mandatory)

**Do not implement the first idea silently.**

Post a short proposal in chat:

- Files to **add or change**
- **Why** this approach (not alternatives picked at random)
- **1–2 architectural alternatives** with trade-offs when relevant
- **Security considerations** (see §5)
- **How to verify** when the slice is done (commands, curls, test names)

Wait for developer confirmation or correction before editing files — unless the developer’s message already contains full agreement (*“implement option A”*).

### Step D — Implementation

**Mode A (code in chat):** paste complete code with **pedagogical line-level comments** (explain `.bind`, `.await`, `&mut *tx`, `?`, types, ownership — see [§2 comment checklist](./AGENTS.md#mode-a--pedagogical-comment-checklist-mandatory)). **Do not edit the repo.**

**Mode B (edit repo):** only after explicit repo permission and agreement (Step C):

- Implement **one agreed slice** at a time
- Add **brief** comments above functions and non-trivial blocks
- Match existing naming, layout, error handling, and crate boundaries
- Align every new module and function with ARCHITECTURE.md and project goal

### Step E — Post-implementation report (mandatory)

**Mode B:** after creating or changing any file, return in chat:

**Mode A:** after pasting code in chat, return:

1. **Files to create or change** — paths the developer should touch (developer applies the paste)
2. **Functions / types** — name + one-line purpose
3. **Walkthrough** — logic of each function and every **security-critical or invariant-critical** block
4. **Verification checklist** — exact commands (build, test, migration, curl)
5. **Suggested follow-ups** — optional next slices or missing test scenarios

For Mode B, item 1 is **files touched** (created / modified / deleted) instead of “files to create”.

---

## 4. Tests

- **Write tests only when the developer explicitly requests tests** in that message.
- If the developer requests tests but the scenarios are incomplete, **propose additional cases** (happy path, domain errors, security edges, idempotency, FK violations) and ask which to implement.
- Do not silently add test files while implementing a feature unless tests were part of the agreed scope.

---

## 5. Security

For every code task — especially `contracts`, auth, money paths, chain ↔ DB sync:

1. **Analyze** plausible failure modes and attack surfaces relevant to this slice
2. **Describe** findings to the developer in plain language (severity: low / medium / high when useful)
3. **Offer mitigations** — options, not a single silent fix
4. **Agree** before applying security-sensitive changes

Examples in this project: reentrancy, access control, flash-loan-style economic attacks, double event application, milestone order bypass, SQL injection via ORM misuse, missing transaction boundaries, trust boundary between chain SoR and DB projection.

Security analysis belongs in **Step C (proposal)** and **Step E (report)** even when the developer did not ask for it explicitly.

---

## 6. Architecture and documentation tasks

When the developer asks to shape architecture (no code yet):

1. Accept informal input in the developer’s words
2. Produce structured documentation in chat (layers, boundaries, flows, open decisions)
3. Break the system into **parts → phases → subtasks**
4. Do not write code until a specific subtask is requested

Prefer updating [ARCHITECTURE.md](./ARCHITECTURE.md) only when the developer asks to persist agreed architecture there.

---

## 7. Teaching and explanations

The developer builds **design-review fluency**, not only syntax.

When explaining or implementing:

1. **Product first** — why this slice exists in the settlement/escrow story before how
2. **Cross-layer parallels** where useful — Solidity / Rust domain / SQL shape for the same concept
3. **Rust mechanics** when non-obvious — ownership, `Result`, async, transactions, what value flows where
4. **Full pattern blocks** — every central pattern gets the expanded format in `.cursor/rules/architecture-language.mdc` (what / why / where / without it / design-review phrase). Naming a pattern without teaching it is insufficient.
5. **Escrow/settlement domain only** — avoid unrelated e-commerce examples unless mapping is explicit

Structure non-trivial guides as: **Goal → full pattern block(s) → architecture link → design choice → implementation summary → verify**.

---

## 8. Agent must not

- Edit implementation files when the developer asked for **code in chat only** (Mode A)
- Edit implementation files without explicit **repo** permission (Mode B)
- Implement before proposal and agreement (§3 Step C)
- Drift from README or ARCHITECTURE.md
- Dump an entire phase in one change set unless explicitly requested
- Commit, push, or change git config unless explicitly requested
- Commit secrets (`.env`, keys, credentials)
- Override developer decisions after they chose an option

---

## 9. Git

- No commits unless the developer explicitly requests in that message
- No force-push to main/master
- Do not modify git config

---

## 10. Communication

**Default language: Russian.** The developer communicates in Russian unless they explicitly ask for English.

| Use Russian for | Use English for |
|-----------------|-----------------|
| All prose, explanations, proposals, reports | **Technical terms and pattern names** (Repository, Transaction, Aggregate, …) |
| Section headers in chat (Что это, Зачем, …) | Identifiers from code (`insert_deal`, `DealRepository`, file paths) |
| Design-review phrases (can be RU or bilingual one-liner) | **RU translation** of each EN term on first mention: *Repository (репозиторий)* |

**Do not mix** English sentences into Russian answers. Wrong: *«What goes wrong without this pattern»*. Right: *«Что ломается без этого паттерна»*.

English-only replies only when the developer writes in English or asks for English.

---

## Quick reference — code request lifecycle

```
Developer request
        │
        ├─ «в чате» / «дай код» / «в этом чате» ──► Mode A: paste in chat, NO file edits
        │
        └─ «implement» / «edit file» / «в репо» ──► Mode B: edit repo (Steps A→E)
```

Mode B flow:

```
Developer request (explicit repo permission)
        │
        ▼
A. Read README + ARCHITECTURE + existing code
        │
        ▼
B. Design analysis + patterns (chat)
        │
        ▼
C. Proposal + security + alternatives → developer agrees
        │
        ▼
D. Implement agreed slice in repo
        │
        ▼
E. Report: files touched, functions, critical logic, verify steps
```

Tests branch off only when separately requested: **proposal of scenarios → agreement → implement**.
