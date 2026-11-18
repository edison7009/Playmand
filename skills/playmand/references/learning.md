# Making experience accumulate

Using this skill does not mean the model remembers experience permanently. Verified launch, operation and debugging methods must be written into project records that can be read again; later tasks read the latest conclusions first instead of rediscovering them.

## Leave a project record first

Record only conclusions that are new or changed this round. Prefer updating an existing hand-over or verification file, and keep:

```text
Issue: observable symptom and trigger conditions
Environment: engine/plugin versions, platform, and conditions that affect reproduction
Shortest repro: initial state → action → actual result / expected result
Cause: the proven cause; list unproven guesses separately
Fix: what changed and why it works; keep failed attempts only if they help debugging
Evidence: regression test, logs, screenshots or measurements, with versions
Reuse scope: which conditions it applies to, and what is project-specific or a temporary workaround
```

When handing over a long task, add the current runnable version, working commands, what is done and not done, and the next step; do not copy the whole conversation. Record the real tool entry points; mark commands that were never run as candidates.

## What goes into the shared skill

- Changes the next development decision, with a repro and evidence: can become a candidate entry.
- API or plugin differences: note the versions and platforms they apply to; do not promote them to a cross-version rule.
- One project's colours, characters, durations, skeleton handling or save semantics: stays in the project; the shared skill keeps the diagnostic method.
- A one-off anomaly, an environment workaround, an unknown root cause, or a speed change without a comparison: keep it as an observation, not as a certain cause or gain.

Without authorization to maintain the shared skill, record candidate experience only in the current project; do not change the user's global skills on your own. When maintaining, merge similar entries in the shared source files, prefer replacing stale guidance, and avoid adding a permanent rule for every problem met. A new tool interface needs a real implementation and evidence of a call before it is written up as a usable operation.

## Check after updating

Check with a real task that triggers the new experience: does the AI notice the relevant conditions first, make fewer wrong attempts, and still pass the same player acceptance? Keep the same starting point, model and tool conditions as before the update; record time, retries, human takeover and pass/fail, leaving missing cost fields empty.

Verify in at least one other independent project or a minimal repro outside the original project before widening a "reusable" claim. A read-only review can find contradictions in instructions, but cannot stand in for independent AI testing or efficiency data.
