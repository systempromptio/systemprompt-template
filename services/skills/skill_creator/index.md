---
title: "Skill Creator"
slug: "skill-creator"
description: "Socratic interview to design and create a custom business skill with guided questions."
author: "systemprompt"
published_at: "2026-02-23"
type: "skill"
category: "marketplace"
keywords: "skills, creator, socratic, builder, custom, business"
---

# Skill Creator

Guide the user through creating a custom skill by asking questions — never by asking them to fill in YAML fields or write config files. You translate plain-language answers into a complete skill.

## What is a Skill?

A skill is a reusable set of instructions that tells an AI agent how to perform a specific task. It includes:
- A clear purpose (what the agent should do)
- Detailed instructions (how to do it)
- Rules and guardrails (what to always/never do)
- Examples (what good output looks like)

## Interview Flow

Ask these questions one at a time. Each answer shapes the skill.

### Step 1: Purpose

**Ask:** "What should this skill help an AI agent do? Describe it as if you were explaining the task to a new employee."

- Listen for: the core capability, scope boundaries, expected outputs
- If vague, ask: "Can you give me a specific example of when someone would use this?"

### Step 2: Examples

**Ask:** "Give me 3 examples of real situations where someone would use this skill. Be as specific as you can."

- Listen for: diverse use cases, edge cases, common patterns
- If they give fewer than 3: "What about [suggest a related scenario]?"
- These become the `examples` field and the Examples section of the skill content

### Step 3: Instructions

**Ask:** "Now imagine you are training someone to do this task. What specific instructions would you give them? Think about: what steps to follow, what to pay attention to, what format the output should be in."

- Listen for: step-by-step processes, formatting requirements, priorities
- Follow-up: "Is there anything that is easy to get wrong? What mistakes should the agent watch out for?"
- This becomes the Instructions section of the skill content

### Step 4: Reference Materials

**Ask:** "Do you have any existing templates, examples of good output, or reference documents that show what 'done well' looks like?"

- Listen for: templates, style guides, example documents
- If yes: "Can you describe or paste the most important one?"
- If no: "That's fine — I will create a template section based on what you have described."
- This becomes the Reference section of the skill content

### Step 5: Category and Tags

**Ask:** "What category does this fall into? For example: writing, operations, analysis, customer-service, development, marketing, sales, or something else?"

- Listen for: primary domain and related domains
- Follow-up: "Any other areas this touches? For example, does it also involve [suggest based on their answers]?"
- This becomes the `tags` field

### Step 6: Voice and Persona

**Ask:** "Should this skill enforce a particular voice or persona? For example: formal and professional, casual and friendly, technical and precise, or your company's specific brand voice?"

- Listen for: tone, formality level, personality traits, brand guidelines
- If they have brand guidelines: "What are the key rules of your brand voice?"
- If not: "No problem. I will keep it neutral and professional."
- This becomes the Voice section of the skill content (if applicable)

## Synthesis

After all questions, present the complete skill for review:

> "Here is the skill I have designed based on our conversation:
>
> **Name:** [derived from Step 1]
> **Description:** [one-sentence summary starting with a verb]
> **Tags:** [from Step 5]
>
> **Content preview:**
> [Show the first few sections of the index.md content]
>
> Does this look right? Anything you would change?"

Wait for confirmation. Make adjustments if requested.

## Skill Content Template

Generate the index.md content following this structure:

```markdown
---
title: "[Skill Name]"
slug: "[skill-slug]"
description: "[One-sentence description starting with verb]"
author: "[user or systemprompt]"
published_at: "[today's date]"
type: "skill"
category: "[from Step 5]"
keywords: "[comma-separated from tags]"
---

# [Skill Name]

[Purpose statement from Step 1 — 2-3 sentences explaining what this skill does and when to use it]

## Instructions

[Detailed instructions from Step 3, formatted as clear steps or guidelines]

## Rules

**Always:**
- [Rules derived from interview — things the agent must do]

**Never:**
- [Anti-patterns derived from interview — things to avoid]

## Examples

[Examples from Step 2, formatted as scenarios with expected behavior]

## Reference

[Templates or reference materials from Step 4, if provided]

## Voice

[Voice guidelines from Step 6, if applicable]
```

## Creation

After the user confirms, create the skill:

**Using MCP tools (preferred):**
- Call `create_skill` with: name, description, content (the full index.md), tags

**Fallback CLI:**
1. Create the config file: `services/skills/{skill_id}/config.yaml`
2. Create the content file: `services/skills/{skill_id}/index.md`
3. Sync: `core skills sync --direction to-db -y`
4. Verify: `core skills show {skill_id}`

## Naming Conventions

- **skill_id:** lowercase_with_underscores, under 30 characters, descriptive
  - Good: `proposal_writing`, `code_review`, `customer_onboarding`
  - Bad: `pw`, `mySkill`, `proposal-writing-for-consulting-firms`
- **name:** Title case, human-readable
- **description:** Starts with a verb (Create, Review, Analyse, Generate, Help with...)
- **tags:** lowercase, 5-7 tags, include category + action + domain

## Quality Checks

Before finalising, verify:
- [ ] Description starts with a verb
- [ ] Tags are all lowercase
- [ ] Examples cover at least 3 diverse use cases
- [ ] Instructions are specific enough to act on (not just "write good content")
- [ ] Rules include at least 2 always and 2 never items
- [ ] Skill ID follows naming conventions
