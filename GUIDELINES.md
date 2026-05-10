# Coding Guidelines

These rules exist for one reason: to keep the codebase simple, readable, and human.
Every decision should make the next person's job easier, not harder.

---

## File Header

Every file begins with a header comment. No exceptions.

```
/* -----------------------------------------------------------------------------
 * auth/login.js
 * Handles login form submission, token storage, and redirect on success.
 * -------------------------------------------------------------------------- */
```

The header contains: the file path relative to the project root, and one sentence
describing what the file does. Not what it contains — what it *does*.

---

## Section Comments

Break every file into named sections using block comments. A reader should be able
to scan the file top to bottom in under 60 seconds and know exactly where everything
lives.

```js
/* --- Constants ------------------------------------------------------------ */

const MAX_RETRIES = 3;
const BASE_URL    = "https://api.example.com";

/* --- Helpers -------------------------------------------------------------- */

function buildUrl(path) { ... }
function parseResponse(raw) { ... }

/* --- Main ----------------------------------------------------------------- */

function fetchUser(id) { ... }
```

Section comment rules:
- The label is short: `Constants`, `Types`, `Helpers`, `State`, `Handlers`, `Render`, `Exports`, `Init`, `Main`
- Use them every time a new logical group starts
- Never let a section grow beyond what fits on one screen without introducing a subsection or splitting the file

---

## File Length

**Hard limit: 100–200 lines per file.**

If a file is growing past that, it is doing too much. Split it.
Name the new files after what they do, not what they are.

```
auth/
  login.js        ← form submit, token storage
  session.js      ← read/write/clear session state
  redirect.js     ← post-login routing logic
```

Flat is fine. Nesting for the sake of organization is not.

---

## Radical Minimalism

**Do not add a dependency if you can write the function in under 20 lines.**

If the wheel you are reinventing fits the codebase better than an off-the-shelf
version, reinvent it. You get full control, zero upgrade risk, and no transitive
dependencies dragging along with it.

This applies to:
- Utility libraries (lodash, ramda, date-fns, etc.)
- Micro-packages that solve one small problem
- Anything where the import costs more than the implementation

The exception: crypto, security primitives, and anything where correctness is
non-negotiable. Use battle-tested libraries there.

---

## Prefer Simple Over Clever

When two approaches produce identical output and performance, always choose
the one a junior developer reads without stopping.

**Loops over iterators:**

```js
// Preferred
const results = [];
for (let i = 0; i < items.length; i++) {
  results.push(transform(items[i]));
}

// Avoid
const results = items.map(transform);
```

**Explicit conditions over ternary chains:**

```js
// Preferred
let label;
if (score >= 90) {
  label = "excellent";
} else if (score >= 70) {
  label = "good";
} else {
  label = "needs work";
}

// Avoid
const label = score >= 90 ? "excellent" : score >= 70 ? "good" : "needs work";
```

**Named variables over inline expressions:**

```js
// Preferred
const isExpired  = token.expiresAt < Date.now();
const isAdmin    = user.role === "admin";
const canProceed = !isExpired && isAdmin;

// Avoid
if (token.expiresAt < Date.now() && user.role === "admin") { ... }
```

The goal is not to write less code. The goal is to write code that does not
require the reader to pause and mentally execute it.

---

## Naming

Names should say what something is or what it does — fully, without abbreviation.

```js
// Preferred
function getUserById(id) { ... }
const pendingRequests = [];
let hasSubmittedForm  = false;

// Avoid
function getUser(id) { ... }
const pending = [];
let submitted = false;
```

- Functions that return booleans start with `is`, `has`, `can`, or `should`
- Functions that fetch data start with `get` or `fetch`
- Functions that write or mutate start with `set`, `update`, `save`, or `delete`
- Event handlers start with `on` or `handle`

---

## Function Rules

- One function does one thing
- If you cannot describe it in one sentence without the word "and", split it
- No function exceeds 30–40 lines; if it does, extract the inner logic
- No nested functions unless they are trivially small closures
- Arguments beyond 2–3 should be collected into a named object

```js
// Preferred
function createUser({ name, email, role }) { ... }

// Avoid
function createUser(name, email, role, isVerified, sendWelcome) { ... }
```

---

## Comments in Code

Comment *why*, not *what*. The code already says what it does.

```js
// Avoid
// increment the counter
count++;

// Preferred (only when the reason is not obvious)
// Retry count starts at 1 because the first attempt already happened above.
let retries = 1;
```

If you feel the urge to comment what a block does, that is a signal to either
name it better or extract it into a function.

---

## Error Handling

Handle errors where they happen. Do not let them bubble silently.
Do not swallow errors with empty catch blocks.

```js
// Preferred
const response = await fetchUser(id);
if (!response.ok) {
  logError("fetchUser failed", { id, status: response.status });
  return null;
}

// Avoid
try {
  const response = await fetchUser(id);
} catch (e) {}
```

Return `null` or an empty value on failure when appropriate.
Return an error object when the caller needs to react differently based on the failure.
Never throw from utilities; only throw from top-level orchestrators where it is caught.

---

## Project Structure

Group files by feature, not by type.

```
/auth
  login.js
  session.js
  logout.js

/users
  list.js
  profile.js
  avatar.js

/shared
  http.js
  storage.js
  format.js
```

`/shared` holds only truly generic utilities used across three or more features.
If something is used by two features, keep it in the one that owns it.

---

## The Core Rule

Before committing any code, ask:

> *Could someone who did not write this read and understand it in one pass?*

If the answer is no, simplify it. Not the tests, not the docs — the code itself.

Complexity is not a sign of skill. It is a maintenance liability. The best code
looks like it was obvious to write.