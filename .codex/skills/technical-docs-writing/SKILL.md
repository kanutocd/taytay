---
name: technical-docs-writing
description: Write, rewrite, audit, or review technical documentation using a Mintlify-guides-inspired standard. Use for READMEs, guides, tutorials, how-tos, references, verification docs, API docs, troubleshooting docs, and docs quality reviews when the user asks for world-class docs, clearer docs, better docs structure, technical writing, documentation rewrite, documentation rubric, or docs intended for both humans and AI agents.
---

# Technical Docs Writing

## Core Standard

Use this skill to make documentation help a specific reader complete a real task or understand a real system. Base the standard on the Mintlify Guides principles: know the audience, choose the right content type, organize for navigation and retrieval, write clearly, use examples intentionally, maintain accuracy, and measure whether readers succeed.

Optimize for humans and AI agents:

- Use clear, descriptive headings that answer likely search questions.
- Make each page and major section self-contained enough to be retrieved independently.
- Define product terms, acronyms, trust boundaries, prerequisites, and expected outputs before relying on them.
- Use consistent names for the same concept.
- Provide complete, copyable examples when examples are useful.
- Prefer concise, active, second-person prose over product-centric exposition.

## Language Requirements

Use this writing style unless the user provides a stricter or more specific style guide:

- Use clear, simple language.
- Use short, direct sentences.
- Use active voice.
- Put the most important information first.
- Focus on practical, actionable information.
- Explain what, why, and how.
- Use examples, code snippets, commands, and expected output when useful.
- Document assumptions, requirements, limitations, and tradeoffs clearly.

Avoid:

- marketing language, hype, and buzzwords,
- metaphors, cliches, and generalizations,
- unsupported claims,
- unnecessary adjectives and adverbs,
- repeated information,
- AI disclaimers and meta-commentary,
- conclusion or summary setup phrases,
- em dashes. Use periods, commas, parentheses, or colons instead,
- paired not-only/but-also constructions.

Use Markdown formatting intentionally:

- Use descriptive headings.
- Use ordered lists for procedures.
- Use fenced code blocks for commands and code.
- Use Note, Warning, Tip, and Important callouts only when they improve clarity.

## Workflow

1. Identify the primary audience.
   Name one dominant reader for the page: new user, implementer, verifier, operator, decision maker, maintainer, or AI agent retrieving content for one of those readers. Do not write one page equally for everyone; split or link when goals conflict.

2. Choose the content type.
   Use one primary Diataxis type per page or section:
   - Tutorial: teach through a guided sequence.
   - How-to: solve one concrete task for a reader with some context.
   - Reference: provide exact facts, contracts, tables, commands, parameters, and constraints.
   - Explanation: build conceptual understanding, tradeoffs, architecture, and trust model.

3. Map the reader journey.
   Put the most common path first. Start with the outcome, prerequisites, and shortest correct path. Move advanced modes, alternatives, limitations, and audit details after the happy path unless they are required for safety.

4. Ground claims in source truth.
   Read the code, config, tests, schemas, or product behavior before documenting it. Remove claims that cannot be verified. Do not invent behavior, names, guarantees, performance numbers, or security properties.

5. Write the first pass.
   Use direct headings, short paragraphs, semantic Markdown, tables for structured data, numbered lists for ordered procedures, and bullets for unordered facts. Add links with descriptive anchor text.

6. Review as a user.
   Ask: can the target reader start from this page, know whether it applies to them, run the commands, recognize success, and recover from expected errors without reading source code?

7. Review as an AI retrieval target.
   Ask: if a section is quoted alone, does it contain enough context to answer a user question accurately? Are names explicit instead of "this", "it", or "the above"?

## Page Patterns

For a practical README:

1. Start with a one-paragraph definition: what this is, who uses it, and what problem it solves.
2. Add "When to use it" and "When not to use it" when the feature has sharp boundaries.
3. Give the shortest working example before deep explanation.
4. List prerequisites before commands.
5. Explain the trust, security, data, or operational model after the reader has seen the concrete flow.
6. Put configuration contracts in tables.
7. Put deployment options in separate sections with decision criteria.
8. End with tests, verification, troubleshooting, or limitations.

For reference docs:

- Keep prose minimal.
- Use stable tables and exact accepted values.
- Include defaults, required/optional status, validation rules, side effects, and examples.
- Avoid burying constraints in paragraphs when a table can expose them.

For verification or security docs:

- Separate trust inputs, threat boundaries, assumptions, and non-goals.
- Tell the verifier exactly which artifacts to compare and which command extracts each value.
- Distinguish signed evidence from corroborating logs.
- Avoid vague claims such as "secure", "safe", or "trusted" unless the document defines the precise property.

## Quality Rubric

Score each dimension 0-2. A publishable doc has no zeroes and should score at least 16/20. A world-class doc scores 18/20 or higher.

| Dimension | 0 | 1 | 2 |
| --- | --- | --- | --- |
| Audience fit | Audience is unclear or mixed | Audience can be inferred | Primary reader and goal are explicit |
| Task success | Reader cannot complete the task | Task works with outside context | Task is complete, ordered, and success is recognizable |
| Content type discipline | Tutorial/how-to/reference/explanation are muddled | Mostly one type with digressions | Structure matches the reader's goal |
| Accuracy | Claims are unverified or stale | Mostly accurate with gaps | Claims are grounded in source truth |
| Structure | Dense or hard to scan | Usable but uneven | Headings, lists, tables, and links form a clear map |
| Examples | Missing, partial, or non-runnable | Useful but incomplete | Complete, realistic, and easy to adapt |
| Terminology | Inconsistent or undefined terms | Minor inconsistency | Terms are defined and used consistently |
| AI retrievability | Sections depend on hidden context | Some sections stand alone | Sections are self-contained and explicit |
| Maintenance | Likely to rot or over-specifies volatile details | Some maintenance risk | Evergreen where possible and clear about volatile facts |
| Style | Wordy, passive, product-centric | Clear with rough spots | Concise, active, reader-centered |

## Editing Rules

- Prefer one strong page over many shallow fragments, but split when audiences or content types conflict.
- Keep warnings specific: state the consequence and the condition that triggers it.
- Put advanced options after the recommended path.
- Make examples realistic without embedding secrets, personal data, or unverifiable production values.
- Use present tense for stable behavior and dates for historical events.
- Avoid colloquialisms, clever headings, and vague cross-references.
- Do not add screenshots, diagrams, or media unless text alone cannot explain the workflow; if media is used, provide alt text and expect maintenance.
- Remove outdated or misleading content instead of preserving it with caveats.

## Final Check

Before finishing a docs rewrite or review, report:

- The primary audience and content type used.
- The main structural changes.
- Any source files or commands used to verify accuracy.
- Remaining risks, especially commands not run, missing product facts, or intentionally deferred docs.
