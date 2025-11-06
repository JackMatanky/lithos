# BMAD-METHOD

## Critical Files

The following critical files are essential and **MUST** be reviewed before starting:

- Coding Standards: docs/architecture/coding-standards.md
- Data Models: docs/architecture/data-models.md
- Components: docs/architecture/components.md

## Agents

The following agents are available and the full definition **MUST** be reviewed when the agent is activated:

```yaml
agents:
  - id: analyst
    title: Business Analyst
    path: .bmad-core/agents/analyst.md
    source: "[.bmad-core/agents/analyst.md](.bmad-core/agents/analyst.md)"
    whenToUse: Use for market research, brainstorming, competitive analysis, creating project briefs, initial project discovery, and documenting existing projects (brownfield)
    activation:
      - 'Mention "As analyst, ..." to get role-aligned behavior'
      - "*analyst"
    fullDefinition: MUST open the source file from path
  - id: architect
    title: Architect
    path: .bmad-core/agents/architect.md
    source: "[.bmad-core/agents/architect.md](.bmad-core/agents/architect.md)"
    whenToUse: Use for system design, architecture documents, technology selection, API design, and infrastructure planning
    activation:
      - 'Mention "As architect, ..." to get role-aligned behavior'
      - "*architect"
    fullDefinition: MUST open the source file from path
  - id: bmad-master
    title: BMad Master Task Executor
    path: .bmad-core/agents/bmad-master.md
    source: "[.bmad-core/agents/bmad-master.md](.bmad-core/agents/bmad-master.md)"
    whenToUse: Use when you need comprehensive expertise across all domains, running 1 off tasks that do not require a persona, or just wanting to use the same agent for many things.
    activation:
      - 'Mention "As bmad-master, ..." to get role-aligned behavior'
      - "*bmad-master"
    fullDefinition: MUST open the source file from path
  - id: bmad-orchestrator
    title: BMad Master Orchestrator
    path: .bmad-core/agents/bmad-orchestrator.md
    source: "[.bmad-core/agents/bmad-orchestrator.md](.bmad-core/agents/bmad-orchestrator.md)"
    whenToUse: Use for workflow coordination, multi-agent tasks, role switching guidance, and when unsure which specialist to consult
    activation:
      - 'Mention "As bmad-orchestrator, ..." to get role-aligned behavior'
      - "*bmad-orchestrator"
    fullDefinition: MUST open the source file from path
  - id: dev
    title: Full Stack Developer
    path: .bmad-core/agents/dev.md
    source: "[.bmad-core/agents/dev.md](.bmad-core/agents/dev.md)"
    whenToUse: Use for code implementation, debugging, refactoring, and development best practices
    activation:
      - 'Mention "As dev, ..." to get role-aligned behavior'
      - "*dev"
    fullDefinition: MUST open the source file from path
  - id: pm
    title: Product Manager
    path: .bmad-core/agents/pm.md
    source: "[.bmad-core/agents/pm.md](.bmad-core/agents/pm.md)"
    whenToUse: Use for creating PRDs, product strategy, feature prioritization, roadmap planning, and stakeholder communication
    activation:
      - 'Mention "As pm, ..." to get role-aligned behavior'
      - "*pm"
    fullDefinition: MUST open the source file from path
  - id: po
    title: Product Owner
    path: .bmad-core/agents/po.md
    source: "[.bmad-core/agents/po.md](.bmad-core/agents/po.md)"
    whenToUse: Use for backlog management, story refinement, acceptance criteria, sprint planning, and prioritization decisions
    activation:
      - 'Mention "As po, ..." to get role-aligned behavior'
      - "*po"
    fullDefinition: MUST open the source file from path
  - id: qa
    title: Test Architect & Quality Advisor
    path: .bmad-core/agents/qa.md
    source: "[.bmad-core/agents/qa.md](.bmad-core/agents/qa.md)"
    whenToUse: Use for comprehensive test architecture review, quality gate decisions, and code improvement. Provides thorough analysis including requirements traceability, risk assessment, and test strategy. Advisory only - teams choose their quality bar.
    activation:
      - 'Mention "As qa, ..." to get role-aligned behavior'
      - "*qa"
    fullDefinition: MUST open the source file from path
  - id: sm
    title: Scrum Master
    path: .bmad-core/agents/sm.md
    source: "[.bmad-core/agents/sm.md](.bmad-core/agents/sm.md)"
    whenToUse: Use for story creation, epic management, retrospectives in party-mode, and agile process guidance
    activation:
      - 'Mention "As sm, ..." to get role-aligned behavior'
      - "*sm"
    fullDefinition: MUST open the source file from path
  - id: ux-expert
    title: UX Expert
    path: .bmad-core/agents/ux-expert.md
    source: "[.bmad-core/agents/ux-expert.md](.bmad-core/agents/ux-expert.md)"
    whenToUse: Use for UI/UX design, wireframes, prototypes, front-end specifications, and user experience optimization
    activation:
      - 'Mention "As ux-expert, ..." to get role-aligned behavior'
      - "*ux-expert"
    fullDefinition: MUST open the source file from path
```

## Tasks

```yaml
tasks:
  - id: validate-next-story
    path: .bmad-core/tasks/validate-next-story.md
    source: "[.bmad-core/tasks/validate-next-story.md](.bmad-core/tasks/validate-next-story.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: trace-requirements
    path: .bmad-core/tasks/trace-requirements.md
    source: "[.bmad-core/tasks/trace-requirements.md](.bmad-core/tasks/trace-requirements.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: test-design
    path: .bmad-core/tasks/test-design.md
    source: "[.bmad-core/tasks/test-design.md](.bmad-core/tasks/test-design.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: shard-doc
    path: .bmad-core/tasks/shard-doc.md
    source: "[.bmad-core/tasks/shard-doc.md](.bmad-core/tasks/shard-doc.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: risk-profile
    path: .bmad-core/tasks/risk-profile.md
    source: "[.bmad-core/tasks/risk-profile.md](.bmad-core/tasks/risk-profile.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: review-story
    path: .bmad-core/tasks/review-story.md
    source: "[.bmad-core/tasks/review-story.md](.bmad-core/tasks/review-story.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: qa-gate
    path: .bmad-core/tasks/qa-gate.md
    source: "[.bmad-core/tasks/qa-gate.md](.bmad-core/tasks/qa-gate.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: nfr-assess
    path: .bmad-core/tasks/nfr-assess.md
    source: "[.bmad-core/tasks/nfr-assess.md](.bmad-core/tasks/nfr-assess.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: kb-mode-interaction
    path: .bmad-core/tasks/kb-mode-interaction.md
    source: "[.bmad-core/tasks/kb-mode-interaction.md](.bmad-core/tasks/kb-mode-interaction.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: index-docs
    path: .bmad-core/tasks/index-docs.md
    source: "[.bmad-core/tasks/index-docs.md](.bmad-core/tasks/index-docs.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: generate-ai-frontend-prompt
    path: .bmad-core/tasks/generate-ai-frontend-prompt.md
    source: "[.bmad-core/tasks/generate-ai-frontend-prompt.md](.bmad-core/tasks/generate-ai-frontend-prompt.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: facilitate-brainstorming-session
    path: .bmad-core/tasks/facilitate-brainstorming-session.md
    source: "[.bmad-core/tasks/facilitate-brainstorming-session.md](.bmad-core/tasks/facilitate-brainstorming-session.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: execute-checklist
    path: .bmad-core/tasks/execute-checklist.md
    source: "[.bmad-core/tasks/execute-checklist.md](.bmad-core/tasks/execute-checklist.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: document-project
    path: .bmad-core/tasks/document-project.md
    source: "[.bmad-core/tasks/document-project.md](.bmad-core/tasks/document-project.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: create-next-story
    path: .bmad-core/tasks/create-next-story.md
    source: "[.bmad-core/tasks/create-next-story.md](.bmad-core/tasks/create-next-story.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: create-doc
    path: .bmad-core/tasks/create-doc.md
    source: "[.bmad-core/tasks/create-doc.md](.bmad-core/tasks/create-doc.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: create-deep-research-prompt
    path: .bmad-core/tasks/create-deep-research-prompt.md
    source: "[.bmad-core/tasks/create-deep-research-prompt.md](.bmad-core/tasks/create-deep-research-prompt.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: create-brownfield-story
    path: .bmad-core/tasks/create-brownfield-story.md
    source: "[.bmad-core/tasks/create-brownfield-story.md](.bmad-core/tasks/create-brownfield-story.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: correct-course
    path: .bmad-core/tasks/correct-course.md
    source: "[.bmad-core/tasks/correct-course.md](.bmad-core/tasks/correct-course.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: brownfield-create-story
    path: .bmad-core/tasks/brownfield-create-story.md
    source: "[.bmad-core/tasks/brownfield-create-story.md](.bmad-core/tasks/brownfield-create-story.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: brownfield-create-epic
    path: .bmad-core/tasks/brownfield-create-epic.md
    source: "[.bmad-core/tasks/brownfield-create-epic.md](.bmad-core/tasks/brownfield-create-epic.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: apply-qa-fixes
    path: .bmad-core/tasks/apply-qa-fixes.md
    source: "[.bmad-core/tasks/apply-qa-fixes.md](.bmad-core/tasks/apply-qa-fixes.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: advanced-elicitation
    path: .bmad-core/tasks/advanced-elicitation.md
    source: "[.bmad-core/tasks/advanced-elicitation.md](.bmad-core/tasks/advanced-elicitation.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
  - id: trace-requirements
    path: .bmad-core/tasks/trace-requirements.md
    source: "[.bmad-core/tasks/trace-requirements.md](.bmad-core/tasks/trace-requirements.md)"
    howToUse: Reference the task in your prompt or execute via your configured commands.
    fullBrief: MUST open the source file from path
```
