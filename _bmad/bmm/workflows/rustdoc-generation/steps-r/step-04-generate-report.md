---
name: 'step-04-generate-report'
description: 'Generate comprehensive adversarial review report'
---

# Step 4: Generate Review Report (Review Mode)

## STEP GOAL:

Create comprehensive adversarial review report with detailed findings, severity classification, and actionable recommendations for achieving documentation excellence.

## MANDATORY EXECUTION RULES (READ FIRST):

### Universal Rules:

- 📖 Read the complete step file before taking any action
- ✅ Speak in `{communication_language}`
- 🎯 Generate actionable, prioritized recommendations

### Role Reinforcement:

- ✅ You are a senior documentation reviewer
- ✅ Provide clear, specific, and actionable feedback
- ✅ Focus on quality improvement and user experience

### Step-Specific Rules:

- 🎯 Categorize findings by severity and category
- 🎯 Provide specific fix recommendations
- 🎯 Prioritize issues by user impact

## EXECUTION PROTOCOLS:

- 🎯 Follow the MANDATORY SEQUENCE exactly
- 📊 Compile all review findings into comprehensive report
- 🎯 Classify and prioritize all issues
- ✅ Save report with proper naming and structure

## CONTEXT BOUNDARIES:

- Available context: All review findings from previous steps
- Focus: Report generation and prioritization
- Limits: Report generation (no fixes in this step)

## MANDATORY SEQUENCE

### 1. Compile All Review Findings

**Gather findings from all review steps:**
- Content and clarity issues
- Technical accuracy problems
- User experience and navigation issues
- Quality and polish problems
- Edge case and stress test findings
- Environmental and platform considerations

**Categorize by severity:**

**Critical Issues (🚨 Fix Immediately):**
- Documentation that could lead to incorrect usage
- Examples that don't compile or are dangerous
- Missing critical safety information
- Breaking inconsistencies between docs and code

**Major Issues (⚠️ Fix Soon):**
- Significant confusion points
- Missing important examples or use cases
- Performance characteristics not documented where relevant
- Navigation or discoverability problems

**Minor Issues (💡 Nice to Fix):**
- Grammar, spelling, or formatting errors
- Inconsistent terminology
- Missing cross-references
- Minor clarity improvements

**Informational (💭 Consider for Future):**
- Enhancement suggestions
- Additional examples that would be helpful
- Documentation improvements for future versions

### 2. Create Comprehensive Review Report

**Generate report at:** `{output_folder}/documentation-artifacts/report-review-{target-file}-{timestamp}.md`

**Report Structure:**
```markdown
---
project: {project_name}
targetFile: [file reviewed]
reviewType: adversarial-comprehensive
date: [current date]
reviewer: rustdoc-specialist
severity: critical-major-minor-info
findings:
  critical: [count]
  major: [count]
  minor: [count]
  info: [count]
categories:
  contentClarity: [issues]
  technicalAccuracy: [issues]
  userExperience: [issues]
  qualityPolish: [issues]
  edgeCases: [issues]
---

# Adversarial Rustdoc Review Report

## Executive Summary

**Target:** [file_path]
**Review Date:** [date]
**Review Type:** Adversarial Comprehensive Review
**Overall Assessment:** [EXCELLENT / GOOD / NEEDS IMPROVEMENT / CRITICAL ISSUES]

### Severity Breakdown
| Severity | Count | Priority | User Impact |
|----------|-------|----------|-------------|
| Critical | [x] | Fix Immediately | User-blocking issues |
| Major | [x] | Fix Soon | Significant user problems |
| Minor | [x] | Nice to Fix | Quality improvements |
| Info | [x] | Consider for Future | Enhancement ideas |

### Category Breakdown
| Category | Critical | Major | Minor | Info | Total |
|----------|----------|-------|-------|------|-------|
| Content Clarity | [x] | [x] | [x] | [x] | [x] |
| Technical Accuracy | [x] | [x] | [x] | [x] | [x] |
| User Experience | [x] | [x] | [x] | [x] | [x] |
| Quality & Polish | [x] | [x] | [x] | [x] | [x] |
| Edge Cases | [x] | [x] | [x] | [x] | [x] |

## Critical Issues (Must Fix Immediately)

### [C-001] [Issue Title]
**Location:** [file.rs:line]
**Severity:** Critical
**Category:** [Content/Technical/UX/Quality/EdgeCase]
**User Impact:** [How this affects users seriously]
**Issue:** [Detailed description of the problem]
**Root Cause:** [Why this issue exists]
**Recommended Fix:** [Specific, actionable solution]
**Code Example:** [Show fix if applicable]
**Testing:** [How to verify the fix works]

## Major Issues (Should Fix Soon)

### [M-001] [Issue Title]
**Location:** [file.rs:line]
**Severity:** Major
**Category:** [Content/Technical/UX/Quality/EdgeCase]
**User Impact:** [How this affects users]
**Issue:** [Detailed description]
**Root Cause:** [Analysis of why this happens]
**Recommended Fix:** [Actionable solution]
**Priority Reason:** [Why this should be fixed soon]

## Minor Issues (Nice to Fix)

### [m-001] [Issue Title]
**Location:** [file.rs:line]
**Severity:** Minor
**Category:** [Content/Technical/UX/Quality]
**User Impact:** [Minor impact on user experience]
**Issue:** [Description]
**Recommended Fix:** [Suggestion]
**Time Estimate:** [Approximate effort to fix]

## Informational Items (Consider for Future)

### [I-001] [Suggestion Title]
**Location:** [file.rs:line]
**Severity:** Informational
**Category:** [Content/Technical/UX/Quality]
**Suggestion:** [Improvement idea]
**Rationale:** [Why this would be valuable]
**Value Proposition:** [Benefits to users]

## Content Quality Analysis

### Summaries and Descriptions
- **Overall Quality:** [Excellent/Good/Fair/Poor]
- **Clarity Issues Found:** [count]
- **Completeness Gaps:** [count]
- **Most Common Confusion Points:** [list]

### Examples Quality
- **Total Examples Reviewed:** [count]
- **Working Examples:** [count]
- **Realistic Scenarios:** [count]/[count]
- **Edge Case Coverage:** [count]/[count]
- **Missing Examples:** [list]
- **Compilation Failures:** [count]

### User Experience Assessment
- **Discoverability:** [Excellent/Good/Fair/Poor]
- **Navigation Clarity:** [Excellent/Good/Fair/Poor]
- **Progressive Disclosure:** [Yes/Partial/No]
- **Context Sufficiency:** [Yes/Partial/No]
- **Top Navigation Problems:** [list]

## Technical Accuracy Verification

### API Consistency Check
- **Type Signatures:** [All Correct / Issues Found: x]
- **Error Documentation:** [Complete / Missing Types / Incomplete]
- **Lifetime Documentation:** [Accurate / Missing / Inaccurate]
- **Trait Bounds:** [Documented / Partially / Missing]
- **Inconsistencies Found:** [list]

### Compilation and Testing Results
```bash
# Results of comprehensive testing
cargo test --doc --all-features: [PASS/FAIL]
cargo doc --no-deps --document-private-items: [PASS/FAIL]
Failed examples: [count]
Broken intra-doc links: [count]
Feature flag issues: [count]
```

## Edge Case and Stress Test Summary

### User Type Edge Cases
- **New User Issues:** [count]
- **Experienced User Issues:** [count]
- **Domain-Specific Issues:** [count]

### Environmental Testing Results
- **Platform-Specific Issues:** [count]
- **Version Compatibility Issues:** [count]
- **Build Configuration Issues:** [count]

### Stress Test Findings
- **Concurrency Issues:** [count]
- **Memory/Resource Issues:** [count]
- **Performance Concerns:** [count]
- **Failure Mode Issues:** [count]

## Quality Metrics

### Documentation Completeness
- **Public Items Documented:** [percentage]%
- **Required Sections Present:** [percentage]%
- **Cross-Reference Integrity:** [percentage]%
- **Example Coverage:** [percentage]%

### User Experience Metrics
- **Information Findability:** [rating 1-5]
- **Learning Curve:** [steep/moderate/gentle]
- **Example Usability:** [rating 1-5]
- **Overall Satisfaction:** [predicted rating]

## Recommendations for Excellence

### Immediate Actions (Critical + Major)
**Priority 1 - Critical Fixes:**
1. [Fix critical issue 1] - [brief justification]
2. [Fix critical issue 2] - [brief justification]
3. [Fix critical issue 3] - [brief justification]

**Priority 2 - Major Improvements:**
1. [Fix major issue 1] - [user impact]
2. [Fix major issue 2] - [user impact]
3. [Fix major issue 3] - [user impact]

### Quality Enhancements (Minor)
**Polish and Professionalism:**
1. [Minor improvement 1] - [benefit]
2. [Minor improvement 2] - [benefit]
3. [Minor improvement 3] - [benefit]

### Future Enhancements (Informational)
**Next Level Excellence:**
1. [Enhancement suggestion 1] - [long-term value]
2. [Enhancement suggestion 2] - [long-term value]
3. [Enhancement suggestion 3] - [long-term value]

## Implementation Roadmap

### Phase 1: Critical Fixes (Week 1)
- [ ] Fix critical compilation issues
- [ ] Resolve dangerous or incorrect documentation
- [ ] Add missing safety information
- [ ] Verify all examples compile

### Phase 2: Major Improvements (Week 2-3)
- [ ] Address user experience problems
- [ ] Add missing important examples
- [ ] Improve discoverability and navigation
- [ ] Document performance characteristics

### Phase 3: Quality Polish (Week 4+)
- [ ] Fix grammar and formatting issues
- [ ] Improve consistency and terminology
- [ ] Add cross-references and links
- [ ] Enhance edge case coverage

### Phase 4: Future Enhancements (Ongoing)
- [ ] Implement informational suggestions
- [ ] Add advanced examples and patterns
- [ ] Improve progressive disclosure
- [ ] Consider user feedback and iterate

## Review Methodology

This adversarial review was conducted using:
- **Content Analysis:** Clarity, completeness, and user perspective
- **Technical Verification:** Accuracy, compilation, and consistency
- **User Experience Testing:** Discoverability, navigation, and real-world usage
- **Quality Assurance:** Professional polish and attention to detail
- **Edge Case Analysis:** Stress testing and scenario analysis
- **Robustness Verification:** Cross-platform and environmental testing

**Review Standards Applied:**
- RFC 1574 compliance verification
- User-centered design principles
- Technical accuracy validation
- Professional documentation standards
- Real-world scenario testing
- Comprehensive quality assessment

## Next Steps

### Immediate Actions (This Week)
1. **Address all Critical Issues** - Fix immediately before any release
2. **Plan Major Fixes** - Schedule improvements for next development cycle
3. **Review Test Results** - Verify compilation and example fixes

### Quality Improvement (Next Month)
1. **Implement Minor Fixes** - Polish and professional improvements
2. **Consider Informational Items** - Plan future enhancements
3. **Establish Review Process** - Make this adversarial review part of development

### Long-term Excellence
1. **User Feedback Integration** - Collect and incorporate user experiences
2. **Continuous Improvement** - Regular adversarial reviews
3. **Documentation Evolution** - Keep documentation current and excellent

---

**Report Generation Complete!**

This adversarial review identified [total] issues across [number] categories.
Critical and major issues should be addressed immediately.
Minor improvements can be implemented as part of regular maintenance.
Informational items provide a roadmap for future excellence.

**Quality is a journey, not a destination.** Use this review as a guide for creating documentation that users will find clear, accurate, and genuinely helpful.
```

### 3. Present Final Review Report

Show {user_name}:

"**🎉 Adversarial Review Report Generated!**

**Target:** [file_path]
**Overall Assessment:** [EXCELLENT/GOOD/NEEDS IMPROVEMENT/CRITICAL]

**Final Summary:**
- **Critical Issues:** [count] 🚨 Fix Immediately
- **Major Issues:** [count] ⚠️ Fix Soon
- **Minor Issues:** [count] 💡 Nice to Fix
- **Informational:** [count] 💭 Future Enhancements

**Top Priority Areas:**
1. [Top critical issue or area]
2. [Second priority]
3. [Third priority]

**Report saved to:** `{output_folder}/documentation-artifacts/report-review-{target-file}-{timestamp}.md`

**Ready for Action?**

**Options:**
- **[F]ix critical issues** - Start with urgent fixes
- **[R]eview full report** - Deep dive into all findings
- **[P]lan improvements** - Create implementation roadmap
- **[Q]uit** - End review with report location

### 4. Handle User Choice

**IF F:**
- Load EDIT MODE workflow
- Pre-populate with critical issues from review
- Focus on immediate fixes

**IF R:**
- Display complete review report
- Allow user to navigate by category and severity

**IF P:**
- Show implementation roadmap
- Help user plan systematic improvements
- Provide timeline and resource estimates

**IF Q:**
- End review session with summary
- Provide report location for future reference

## 🚨 SYSTEM SUCCESS/FAILURE METRICS:

### ✅ SUCCESS:

- All review findings compiled into comprehensive report
- Issues classified by severity and category
- Specific, actionable recommendations provided
- Implementation roadmap created
- Report saved with proper naming and structure

### ❌ SYSTEM FAILURE:

- Report not generated or incomplete
- Issues not properly classified or prioritized
- No actionable recommendations provided
- Report not saved correctly
- User not presented with next step options
