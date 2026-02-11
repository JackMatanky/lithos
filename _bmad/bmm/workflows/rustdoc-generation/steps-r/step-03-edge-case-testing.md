---
name: 'step-03-edge-case-testing'
description: 'Edge case and stress testing of rustdoc documentation'
nextStepFile: '../steps-r/step-04-generate-report.md'
---

# Step 3: Edge Case and Stress Testing (Review Mode)

## STEP GOAL:

Conduct edge case analysis and stress testing to identify problems users would encounter in real-world scenarios, challenging conditions, and production environments.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Maintain adversarial reviewer mindset

### Role Reinforcement:

- ✅ You are a senior documentation reviewer
- ✅ Think like a user having problems
- ✅ Consider worst-case scenarios and failure modes

### Step-Specific Rules:

- 🎯 Test documentation in challenging scenarios
- 🎯 Consider different user skill levels and environments
- 🎯 Identify problems that only appear under stress

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 🔍 Conduct systematic edge case analysis
- 🧪 Perform stress testing scenarios
- 📊 Document all edge case findings

## CONTEXT BOUNDARIES:

- Available context: Target file and comprehensive review findings
- Focus: Edge case and stress testing
- Limits: Testing analysis only (no fixes yet)

## MANDATORY SEQUENCE

### 1. Edge Case Analysis by User Type

**New User Scenarios:**
- [ ] What would completely confuse someone new to Rust?
- [ ] Which concepts require background knowledge not documented?
- [ ] Are there "obvious" assumptions that aren't obvious at all?
- [ ] What terminology would a beginner not understand?
- [ ] Document specific confusion points with examples

**Experienced User Scenarios:**
- [ ] What might an experienced Rust user miss or assume incorrectly?
- [ ] Are there advanced patterns that lack documentation?
- [ ] Are performance characteristics documented for power users?
- [ ] What advanced use cases are not covered?
- [ ] Identify missing expert-level information

**Domain-Specific Users:**
- [ ] Are there domain-specific terms without explanation?
- [ ] Do examples cover realistic domain scenarios?
- [ ] Are there patterns specific to this domain not documented?
- [ ] Document domain-specific gaps and assumptions

### 2. Environmental and Platform Edge Cases

**Different Rust Versions:**
- [ ] Will this work on stable Rust versions?
- [ ] Are there nightly-only features not clearly marked?
- [ ] Are minimum Rust version requirements documented?
- [ ] Test compatibility concerns across versions

**Platform-Specific Behavior:**
- [ ] Are there platform-specific behaviors not documented?
- [ ] Do examples work cross-platform?
- [ ] Are there OS-specific considerations missing?
- [ ] Document platform-specific issues found

**Build Configuration Scenarios:**
- [ ] How does this work with different feature flags?
- [ ] Are there conditional compilation considerations?
- [ ] What happens with different optimization levels?
- [ ] Document build configuration dependencies

### 3. Stress Testing Scenarios

**Concurrent Usage Testing:**
- [ ] How does this behave in multi-threaded contexts?
- [ ] Are there thread safety considerations not documented?
- [ ] What happens with shared mutable state?
- [ ] Are there race conditions or deadlocks possible?
- [ ] Document concurrency-related concerns

**Memory and Resource Stress:**
- [ ] What happens under memory pressure?
- [ ] Are memory usage characteristics documented?
- [ ] Are there resource leak possibilities?
- [ ] How does it handle large inputs or long-running operations?
- [ ] Document resource management issues

**Error and Failure Scenarios:**
- [ ] What happens in extreme error conditions?
- [ ] Are there failure modes not documented?
- [ ] How does it recover from partial failures?
- [ ] Are there cascading failure possibilities?
- [ ] Document failure handling gaps

### 4. Performance and Scale Testing

**Large-Scale Usage:**
- [ ] How does performance change with large inputs?
- [ ] Are there performance cliffs or regressions?
- [ ] What are the scalability limits?
- [ ] Are performance characteristics under load documented?
- [ ] Document performance concerns found

**Resource Consumption Analysis:**
- [ ] What are the CPU, memory, and I/O characteristics?
- [ ] Are there resource bottlenecks not documented?
- [ ] How does it behave under resource constraints?
- [ ] Document resource usage surprises

**Timing and Latency Considerations:**
- [ ] Are there timing-dependent behaviors not documented?
- [ ] What are latency characteristics?
- [ ] Are there timeout or timing edge cases?
- [ ] Document timing-related issues

### 5. Documentation Robustness Testing

**Example Robustness:**
- [ ] Do examples work when copy-pasted into real projects?
- [ ] Are there missing dependencies in examples?
- [ ] Do examples fail in different contexts?
- [ ] Test each example in isolation
- [ ] Document example reliability issues

**Cross-Reference Integrity:**
- [ ] Do all intra-doc links actually work?
- [ ] Are there broken references to other items?
- [ ] Do cross-module references resolve correctly?
- [ ] Test link integrity systematically
- [ ] Document broken or misleading links

**API Evolution Considerations:**
- [ ] How well does this documentation handle API changes?
- [ ] Are there backward compatibility considerations missing?
- [ ] What migration paths would users need?
- [ ] Document evolution-related concerns

### 6. Compile and Test Verification

**Comprehensive Testing:**
```bash
# Test all examples compile
cargo test --doc --all-features

# Test with different Rust versions
cargo +stable test --doc
cargo +beta test --doc

# Test documentation generation
cargo doc --no-deps --all-features

# Verify intra-doc links
cargo doc --no-deps --document-private-items
```

**Document any failures found:**
- [ ] Compilation errors in examples
- [ ] Feature flag related issues
- [ ] Version compatibility problems
- [ ] Broken cross-references

### 7. Document Edge Case Findings

Update findings with edge case analysis:

```markdown
## Edge Case and Stress Test Findings

### User Experience Edge Cases
[E-001] [Issue Title]
**Scenario:** [User type and situation]
**Impact:** How this affects user experience
**Description:** Detailed explanation of edge case problem

### Environmental Edge Cases
[ENV-001] [Issue Title]
**Environment:** [Platform/version/configuration]
**Impact:** Environmental failure or unexpected behavior
**Description:** Platform-specific or configuration issue

### Stress Test Findings
[ST-001] [Issue Title]
**Scenario:** [Stress condition]
**Impact:** Failure under stress or at scale
**Description:** Stress-induced problem

### Robustness Issues
[ROB-001] [Issue Title]
**Test:** [Robustness test that failed]
**Impact:** Documentation unreliability
**Description:** Robustness or integrity problem
```

### 8. Present Edge Case Analysis

Show {user_name}:

"**Edge Case and Stress Testing Complete!**

**Testing Scenarios Covered:**
- ✅ New User Edge Cases
- ✅ Experienced User Scenarios
- ✅ Environmental/Platform Testing
- ✅ Stress and Load Testing
- ✅ Performance and Scale Analysis
- ✅ Documentation Robustness

**Edge Case Findings:**
- User Experience Issues: [count]
- Environmental Issues: [count]
- Stress Test Failures: [count]
- Robustness Problems: [count]
- **Total Edge Cases:** [count]

**Combined with Previous Review:**
- Original Issues: [count]
- Edge Case Issues: [count]
- **Total Findings:** [count]

**Next:** Generate comprehensive review report

Generate final adversarial review report?

**Options:**
- **[G]enerate** - Create comprehensive review report
- **[S]ummary** - Review all findings by category
- **[F]ocus** - Deep dive into edge case findings
- **[Q]uit** - Save findings and end session"

### 9. Handle User Choice

**IF G:**
- Load next step: `{nextStepFile}`
- Generate comprehensive review report

**IF S:**
- Show summary of all findings (previous + edge cases)
- Return to choice menu

**IF F:**
- Show detailed edge case findings
- Return to choice menu

**IF Q:**
- Save current findings and end session

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All edge case scenarios tested systematically
- Environmental and platform considerations evaluated
- Stress and performance analysis completed
- Documentation robustness verified
- Findings documented with specific scenarios and impacts

### ❌ SYSTEM FAILURE:

- Edge case analysis incomplete
- Environmental testing not performed
- Stress testing scenarios missed
- Documentation robustness not verified
- No specific edge case findings documented
