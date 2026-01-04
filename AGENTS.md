# BMAD-METHOD

## Critical Files

The following critical files are essential and **MUST** be reviewed before starting:

- Coding Standards: docs/architecture/coding-standards.md
- Data Models: docs/architecture/data-models.md
- Components: docs/architecture/components.md
- Tech Stack: docs/architecture/tech-stack.md

## Agents

The following agents are available and the full definition **MUST** be reviewed when the agent is activated:

```yaml
agents:
  - id: agent-builder
    title: Agent Builder
    path: _bmad/bmb/agents/agent-builder.md
    source: "[_bmad/bmb/agents/agent-builder.md](_bmad/bmb/agents/agent-builder.md)"
    whenToUse: Use to create, edit, and validate new BMAD compliant agents.
    activation:
      - 'Mention "As agent-builder, ..."'
      - "*agent-builder"
    fullDefinition: MUST open the source file from path
  - id: module-builder
    title: Module Builder
    path: _bmad/bmb/agents/module-builder.md
    source: "[_bmad/bmb/agents/module-builder.md](_bmad/bmb/agents/module-builder.md)"
    whenToUse: Use to package agents and workflows into complete BMAD modules.
    activation:
      - 'Mention "As module-builder, ..."'
      - "*module-builder"
    fullDefinition: MUST open the source file from path
  - id: workflow-builder
    title: Workflow Builder
    path: _bmad/bmb/agents/workflow-builder.md
    source: "[_bmad/bmb/agents/workflow-builder.md](_bmad/bmb/agents/workflow-builder.md)"
    whenToUse: Use to design, create, and edit structured workflows.
    activation:
      - 'Mention "As workflow-builder, ..."'
      - "*workflow-builder"
    fullDefinition: MUST open the source file from path
  - id: analyst
    title: Business Analyst
    path: _bmad/bmm/agents/analyst.md
    source: "[_bmad/bmm/agents/analyst.md](_bmad/bmm/agents/analyst.md)"
    whenToUse: Use for market research, brainstorming, competitive analysis, creating project briefs, and initial discovery.
    activation:
      - 'Mention "As analyst, ..."'
      - "*analyst"
    fullDefinition: MUST open the source file from path
  - id: architect
    title: Architect
    path: _bmad/bmm/agents/architect.md
    source: "[_bmad/bmm/agents/architect.md](_bmad/bmm/agents/architect.md)"
    whenToUse: Use for system design, architecture documents, technology selection, and infrastructure planning.
    activation:
      - 'Mention "As architect, ..."'
      - "*architect"
    fullDefinition: MUST open the source file from path
  - id: dev
    title: Full Stack Developer
    path: _bmad/bmm/agents/dev.md
    source: "[_bmad/bmm/agents/dev.md](_bmad/bmm/agents/dev.md)"
    whenToUse: Use for code implementation, debugging, refactoring, and general development tasks.
    activation:
      - 'Mention "As dev, ..."'
      - "*dev"
    fullDefinition: MUST open the source file from path
  - id: pm
    title: Product Manager
    path: _bmad/bmm/agents/pm.md
    source: "[_bmad/bmm/agents/pm.md](_bmad/bmm/agents/pm.md)"
    whenToUse: Use for product strategy, PRDs, roadmap planning, and stakeholder requirements.
    activation:
      - 'Mention "As pm, ..."'
      - "*pm"
    fullDefinition: MUST open the source file from path
  - id: quick-flow-solo-dev
    title: Quick Flow Solo Dev
    path: _bmad/bmm/agents/quick-flow-solo-dev.md
    source: "[_bmad/bmm/agents/quick-flow-solo-dev.md](_bmad/bmm/agents/quick-flow-solo-dev.md)"
    whenToUse: Use for rapid prototyping or solo development cycles combining multiple roles.
    activation:
      - 'Mention "As quick-flow-solo-dev, ..."'
      - "*quick-flow-solo-dev"
    fullDefinition: MUST open the source file from path
  - id: sm
    title: Scrum Master
    path: _bmad/bmm/agents/sm.md
    source: "[_bmad/bmm/agents/sm.md](_bmad/bmm/agents/sm.md)"
    whenToUse: Use for agile process guidance, sprint planning, and removing blockers.
    activation:
      - 'Mention "As sm, ..."'
      - "*sm"
    fullDefinition: MUST open the source file from path
  - id: tea
    title: Test Engineering Architect
    path: _bmad/bmm/agents/tea.md
    source: "[_bmad/bmm/agents/tea.md](_bmad/bmm/agents/tea.md)"
    whenToUse: Use for test strategy, quality gates, and test automation architecture.
    activation:
      - 'Mention "As tea, ..."'
      - "*tea"
    fullDefinition: MUST open the source file from path
  - id: tech-writer
    title: Tech Writer
    path: _bmad/bmm/agents/tech-writer.md
    source: "[_bmad/bmm/agents/tech-writer.md](_bmad/bmm/agents/tech-writer.md)"
    whenToUse: Use for documentation, API references, and user guides.
    activation:
      - 'Mention "As tech-writer, ..."'
      - "*tech-writer"
    fullDefinition: MUST open the source file from path
  - id: ux-designer
    title: UX Designer
    path: _bmad/bmm/agents/ux-designer.md
    source: "[_bmad/bmm/agents/ux-designer.md](_bmad/bmm/agents/ux-designer.md)"
    whenToUse: Use for UI/UX design, wireframes, and user experience planning.
    activation:
      - 'Mention "As ux-designer, ..."'
      - "*ux-designer"
    fullDefinition: MUST open the source file from path
  - id: bmad-master
    title: Bmad Master
    path: _bmad/core/agents/bmad-master.md
    source: "[_bmad/core/agents/bmad-master.md](_bmad/core/agents/bmad-master.md)"
    whenToUse: Use for general-purpose orchestration or when a specific role is unsure.
    activation:
      - 'Mention "As bmad-master, ..."'
      - "*bmad-master"
    fullDefinition: MUST open the source file from path
```

## Tasks

```yaml
tasks:
  - id: meal-prep-nutrition
    path: _bmad/bmb/workflows/create-workflow/data/examples/meal-prep-nutrition/workflow.md
    source: "[_bmad/bmb/workflows/create-workflow/data/examples/meal-prep-nutrition/workflow.md](_bmad/bmb/workflows/create-workflow/data/examples/meal-prep-nutrition/workflow.md)"
    howToUse: Creates personalized meal plans through collaborative nutrition planning.
    fullBrief: MUST open the source file from path
  - id: agent-workflow
    path: _bmad/bmb/workflows/agent/workflow.md
    source: "[_bmad/bmb/workflows/agent/workflow.md](_bmad/bmb/workflows/agent/workflow.md)"
    howToUse: Tri-modal workflow for creating, editing, and validating BMAD Core compliant agents.
    fullBrief: MUST open the source file from path
  - id: create-module
    path: _bmad/bmb/workflows/create-module/workflow.md
    source: "[_bmad/bmb/workflows/create-module/workflow.md](_bmad/bmb/workflows/create-module/workflow.md)"
    howToUse: Interactive workflow to build complete BMAD modules with agents and workflows.
    fullBrief: MUST open the source file from path
  - id: create-workflow
    path: _bmad/bmb/workflows/create-workflow/workflow.md
    source: "[_bmad/bmb/workflows/create-workflow/workflow.md](_bmad/bmb/workflows/create-workflow/workflow.md)"
    howToUse: Create structured standalone workflows using markdown-based step architecture.
    fullBrief: MUST open the source file from path
  - id: edit-workflow
    path: _bmad/bmb/workflows/edit-workflow/workflow.md
    source: "[_bmad/bmb/workflows/edit-workflow/workflow.md](_bmad/bmb/workflows/edit-workflow/workflow.md)"
    howToUse: Intelligent workflow editor that helps modify existing workflows.
    fullBrief: MUST open the source file from path
  - id: workflow-compliance
    path: _bmad/bmb/workflows/workflow-compliance-check/workflow.md
    source: "[_bmad/bmb/workflows/workflow-compliance-check/workflow.md](_bmad/bmb/workflows/workflow-compliance-check/workflow.md)"
    howToUse: Systematic validation of workflows against BMAD standards.
    fullBrief: MUST open the source file from path
  - id: check-readiness
    path: _bmad/bmm/workflows/3-solutioning/check-implementation-readiness/workflow.md
    source: "[_bmad/bmm/workflows/3-solutioning/check-implementation-readiness/workflow.md](_bmad/bmm/workflows/3-solutioning/check-implementation-readiness/workflow.md)"
    howToUse: Critical validation of PRD, Architecture, and Epics before implementation.
    fullBrief: MUST open the source file from path
  - id: code-review
    path: _bmad/bmm/workflows/4-implementation/code-review/workflow.yaml
    source: "[_bmad/bmm/workflows/4-implementation/code-review/workflow.yaml](_bmad/bmm/workflows/4-implementation/code-review/workflow.yaml)"
    howToUse: Perform an adversarial senior developer code review.
    fullBrief: MUST open the source file from path
  - id: correct-course
    path: _bmad/bmm/workflows/4-implementation/correct-course/workflow.yaml
    source: "[_bmad/bmm/workflows/4-implementation/correct-course/workflow.yaml](_bmad/bmm/workflows/4-implementation/correct-course/workflow.yaml)"
    howToUse: Navigate significant changes during sprint execution.
    fullBrief: MUST open the source file from path
  - id: create-architecture
    path: _bmad/bmm/workflows/3-solutioning/create-architecture/workflow.md
    source: "[_bmad/bmm/workflows/3-solutioning/create-architecture/workflow.md](_bmad/bmm/workflows/3-solutioning/create-architecture/workflow.md)"
    howToUse: Collaborative architectural decision facilitation for AI-agent consistency.
    fullBrief: MUST open the source file from path
  - id: create-epics-stories
    path: _bmad/bmm/workflows/3-solutioning/create-epics-and-stories/workflow.md
    source: "[_bmad/bmm/workflows/3-solutioning/create-epics-and-stories/workflow.md](_bmad/bmm/workflows/3-solutioning/create-epics-and-stories/workflow.md)"
    howToUse: Transform PRD and Architecture into comprehensive epics and user stories.
    fullBrief: MUST open the source file from path
  - id: create-dataflow
    path: _bmad/bmm/workflows/excalidraw-diagrams/create-dataflow/workflow.yaml
    source: "[_bmad/bmm/workflows/excalidraw-diagrams/create-dataflow/workflow.yaml](_bmad/bmm/workflows/excalidraw-diagrams/create-dataflow/workflow.yaml)"
    howToUse: Create data flow diagrams (DFD) in Excalidraw format.
    fullBrief: MUST open the source file from path
  - id: create-diagram
    path: _bmad/bmm/workflows/excalidraw-diagrams/create-diagram/workflow.yaml
    source: "[_bmad/bmm/workflows/excalidraw-diagrams/create-diagram/workflow.yaml](_bmad/bmm/workflows/excalidraw-diagrams/create-diagram/workflow.yaml)"
    howToUse: Create system architecture, ERD, or UML diagrams in Excalidraw.
    fullBrief: MUST open the source file from path
  - id: create-flowchart
    path: _bmad/bmm/workflows/excalidraw-diagrams/create-flowchart/workflow.yaml
    source: "[_bmad/bmm/workflows/excalidraw-diagrams/create-flowchart/workflow.yaml](_bmad/bmm/workflows/excalidraw-diagrams/create-flowchart/workflow.yaml)"
    howToUse: Create process flowcharts in Excalidraw format.
    fullBrief: MUST open the source file from path
  - id: create-wireframe
    path: _bmad/bmm/workflows/excalidraw-diagrams/create-wireframe/workflow.yaml
    source: "[_bmad/bmm/workflows/excalidraw-diagrams/create-wireframe/workflow.yaml](_bmad/bmm/workflows/excalidraw-diagrams/create-wireframe/workflow.yaml)"
    howToUse: Create website or app wireframes in Excalidraw format.
    fullBrief: MUST open the source file from path
  - id: create-prd
    path: _bmad/bmm/workflows/2-plan-workflows/prd/workflow.md
    source: "[_bmad/bmm/workflows/2-plan-workflows/prd/workflow.md](_bmad/bmm/workflows/2-plan-workflows/prd/workflow.md)"
    howToUse: Create a comprehensive PRD through collaborative discovery.
    fullBrief: MUST open the source file from path
  - id: create-product-brief
    path: _bmad/bmm/workflows/1-analysis/create-product-brief/workflow.md
    source: "[_bmad/bmm/workflows/1-analysis/create-product-brief/workflow.md](_bmad/bmm/workflows/1-analysis/create-product-brief/workflow.md)"
    howToUse: Create product briefs through collaborative step-by-step discovery.
    fullBrief: MUST open the source file from path
  - id: create-story
    path: _bmad/bmm/workflows/4-implementation/create-story/workflow.yaml
    source: "[_bmad/bmm/workflows/4-implementation/create-story/workflow.yaml](_bmad/bmm/workflows/4-implementation/create-story/workflow.yaml)"
    howToUse: Create the next user story from epics+stories with context analysis.
    fullBrief: MUST open the source file from path
  - id: create-tech-spec
    path: _bmad/bmm/workflows/bmad-quick-flow/create-tech-spec/workflow.md
    source: "[_bmad/bmm/workflows/bmad-quick-flow/create-tech-spec/workflow.md](_bmad/bmm/workflows/bmad-quick-flow/create-tech-spec/workflow.md)"
    howToUse: Produce implementation-ready tech specs through investigation.
    fullBrief: MUST open the source file from path
  - id: create-ux-design
    path: _bmad/bmm/workflows/2-plan-workflows/create-ux-design/workflow.md
    source: "[_bmad/bmm/workflows/2-plan-workflows/create-ux-design/workflow.md](_bmad/bmm/workflows/2-plan-workflows/create-ux-design/workflow.md)"
    howToUse: Plan application UX patterns, look and feel.
    fullBrief: MUST open the source file from path
  - id: dev-story
    path: _bmad/bmm/workflows/4-implementation/dev-story/workflow.yaml
    source: "[_bmad/bmm/workflows/4-implementation/dev-story/workflow.yaml](_bmad/bmm/workflows/4-implementation/dev-story/workflow.yaml)"
    howToUse: Execute a story by implementing tasks, writing tests, and validating.
    fullBrief: MUST open the source file from path
  - id: document-project
    path: _bmad/bmm/workflows/document-project/workflow.yaml
    source: "[_bmad/bmm/workflows/document-project/workflow.yaml](_bmad/bmm/workflows/document-project/workflow.yaml)"
    howToUse: Analyze and document brownfield projects.
    fullBrief: MUST open the source file from path
  - id: generate-context
    path: _bmad/bmm/workflows/generate-project-context/workflow.md
    source: "[_bmad/bmm/workflows/generate-project-context/workflow.md](_bmad/bmm/workflows/generate-project-context/workflow.md)"
    howToUse: Create a concise project-context.md file for AI agents.
    fullBrief: MUST open the source file from path
  - id: quick-dev
    path: _bmad/bmm/workflows/bmad-quick-flow/quick-dev/workflow.md
    source: "[_bmad/bmm/workflows/bmad-quick-flow/quick-dev/workflow.md](_bmad/bmm/workflows/bmad-quick-flow/quick-dev/workflow.md)"
    howToUse: Flexible development - execute tech-specs or direct instructions.
    fullBrief: MUST open the source file from path
  - id: research
    path: _bmad/bmm/workflows/1-analysis/research/workflow.md
    source: "[_bmad/bmm/workflows/1-analysis/research/workflow.md](_bmad/bmm/workflows/1-analysis/research/workflow.md)"
    howToUse: Conduct comprehensive research across multiple domains.
    fullBrief: MUST open the source file from path
  - id: retrospective
    path: _bmad/bmm/workflows/4-implementation/retrospective/workflow.yaml
    source: "[_bmad/bmm/workflows/4-implementation/retrospective/workflow.yaml](_bmad/bmm/workflows/4-implementation/retrospective/workflow.yaml)"
    howToUse: Run after epic completion to review success and lessons learned.
    fullBrief: MUST open the source file from path
  - id: sprint-planning
    path: _bmad/bmm/workflows/4-implementation/sprint-planning/workflow.yaml
    source: "[_bmad/bmm/workflows/4-implementation/sprint-planning/workflow.yaml](_bmad/bmm/workflows/4-implementation/sprint-planning/workflow.yaml)"
    howToUse: Generate and manage the sprint status tracking file.
    fullBrief: MUST open the source file from path
  - id: sprint-status
    path: _bmad/bmm/workflows/4-implementation/sprint-status/workflow.yaml
    source: "[_bmad/bmm/workflows/4-implementation/sprint-status/workflow.yaml](_bmad/bmm/workflows/4-implementation/sprint-status/workflow.yaml)"
    howToUse: Summarize sprint status, surface risks, and route next steps.
    fullBrief: MUST open the source file from path
  - id: test-atdd
    path: _bmad/bmm/workflows/testarch/atdd/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/atdd/workflow.yaml](_bmad/bmm/workflows/testarch/atdd/workflow.yaml)"
    howToUse: Generate failing acceptance tests before implementation (ATDD).
    fullBrief: MUST open the source file from path
  - id: test-automate
    path: _bmad/bmm/workflows/testarch/automate/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/automate/workflow.yaml](_bmad/bmm/workflows/testarch/automate/workflow.yaml)"
    howToUse: Expand test automation coverage or analyze existing codebase.
    fullBrief: MUST open the source file from path
  - id: test-ci
    path: _bmad/bmm/workflows/testarch/ci/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/ci/workflow.yaml](_bmad/bmm/workflows/testarch/ci/workflow.yaml)"
    howToUse: Scaffold CI/CD quality pipeline with test execution.
    fullBrief: MUST open the source file from path
  - id: test-framework
    path: _bmad/bmm/workflows/testarch/framework/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/framework/workflow.yaml](_bmad/bmm/workflows/testarch/framework/workflow.yaml)"
    howToUse: Initialize production-ready test framework architecture.
    fullBrief: MUST open the source file from path
  - id: test-nfr
    path: _bmad/bmm/workflows/testarch/nfr-assess/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/nfr-assess/workflow.yaml](_bmad/bmm/workflows/testarch/nfr-assess/workflow.yaml)"
    howToUse: Assess non-functional requirements (performance, security, etc.).
    fullBrief: MUST open the source file from path
  - id: test-design
    path: _bmad/bmm/workflows/testarch/test-design/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/test-design/workflow.yaml](_bmad/bmm/workflows/testarch/test-design/workflow.yaml)"
    howToUse: System-level testability review or Epic-level test planning.
    fullBrief: MUST open the source file from path
  - id: test-review
    path: _bmad/bmm/workflows/testarch/test-review/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/test-review/workflow.yaml](_bmad/bmm/workflows/testarch/test-review/workflow.yaml)"
    howToUse: Review test quality using comprehensive knowledge base.
    fullBrief: MUST open the source file from path
  - id: test-trace
    path: _bmad/bmm/workflows/testarch/trace/workflow.yaml
    source: "[_bmad/bmm/workflows/testarch/trace/workflow.yaml](_bmad/bmm/workflows/testarch/trace/workflow.yaml)"
    howToUse: Generate requirements-to-tests traceability matrix.
    fullBrief: MUST open the source file from path
  - id: workflow-init
    path: _bmad/bmm/workflows/workflow-status/init/workflow.yaml
    source: "[_bmad/bmm/workflows/workflow-status/init/workflow.yaml](_bmad/bmm/workflows/workflow-status/init/workflow.yaml)"
    howToUse: Initialize a new BMM project by determining level and type.
    fullBrief: MUST open the source file from path
  - id: workflow-status
    path: _bmad/bmm/workflows/workflow-status/workflow.yaml
    source: "[_bmad/bmm/workflows/workflow-status/workflow.yaml](_bmad/bmm/workflows/workflow-status/workflow.yaml)"
    howToUse: Answers "what should I do now?" by reading project status.
    fullBrief: MUST open the source file from path
  - id: brainstorming
    path: _bmad/core/workflows/brainstorming/workflow.md
    source: "[_bmad/core/workflows/brainstorming/workflow.md](_bmad/core/workflows/brainstorming/workflow.md)"
    howToUse: Facilitate interactive brainstorming sessions.
    fullBrief: MUST open the source file from path
  - id: party-mode
    path: _bmad/core/workflows/party-mode/workflow.md
    source: "[_bmad/core/workflows/party-mode/workflow.md](_bmad/core/workflows/party-mode/workflow.md)"
    howToUse: Orchestrate group discussions between all installed BMAD agents.
    fullBrief: MUST open the source file from path
  - id: index-docs
    path: _bmad/core/tasks/index-docs.xml
    source: "[_bmad/core/tasks/index-docs.xml](_bmad/core/tasks/index-docs.xml)"
    howToUse: Generates or updates an index.md of all documents in the specified directory.
    fullBrief: MUST open the source file from path
```

# Common Commands

File Formatting: `golangci-lint fmt`

File Linting: `golangci-lint run --fix`
