# Test Inventory

## crates/test-utils/tests/cqrs_commands.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `command_handler_saves_entity_to_repository` | N/A | Yes | Pass |
| `command_handler_publishes_domain_event` | N/A | Yes | Pass |
| `command_handler_validates_input` | N/A | Yes | Pass |
| `command_handler_handles_repository_failure` | N/A | Yes | Pass |
| `command_handler_records_all_interactions` | N/A | Yes | Pass |

## crates/test-utils/tests/cqrs_queries.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `query_handler_returns_user_by_id` | N/A | Yes | Pass |
| `query_handler_returns_none_for_nonexistent_user` | N/A | Yes | Pass |
| `list_query_handler_returns_all_users` | N/A | Yes | Pass |
| `stub_store_can_be_dynamically_updated` | N/A | Yes | Pass |
| `stub_store_supports_count_operations` | N/A | Yes | Pass |
| `query_handler_performs_within_time_bounds` | N/A | Yes | Pass |
| `stub_store_can_be_cleared_for_test_isolation` | N/A | Yes | Pass |

## crates/test-utils/src/assertions.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `test_async_operation` |  | Yes | Pass |
| `test_eventual_condition` |  | Yes | Pass |
| `test_assert_eq_detailed_success` | N/A | Yes | Pass |
| `test_assert_eq_detailed_failure` | N/A | Yes | Pass |
| `test_assert_async_completed_success` | N/A | Yes | Pass |
| `test_assert_async_completed_timeout` | N/A | Yes | Pass |
| `test_structural_compare_success` | N/A | Yes | Pass |
| `test_structural_compare_failure` | N/A | Yes | Pass |
| `test_domain_assert_contains_same_items` | N/A | Yes | Pass |
| `test_domain_assert_in_range` | N/A | Yes | Pass |
| `test_domain_assert_in_range_failure` | N/A | Yes | Pass |

## crates/test-utils/src/events.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `given_when_then_matches_expected_events` | N/A | Yes | Pass |
| `given_when_then_reports_mismatched_events` | N/A | Yes | Pass |
| `payload_assertion_detects_mismatch` | N/A | Yes | Pass |
| `sequence_assertion_detects_out_of_order` | N/A | Yes | Pass |
| `sequence_assertion_accepts_increasing_sequences` | N/A | Yes | Pass |
| `timing_assertion_rejects_backwards_timestamps` | N/A | Yes | Pass |
| `timing_assertion_enforces_max_span` | N/A | Yes | Pass |
| `payload_assertion_to_value_serializes_payload` | N/A | Yes | Pass |
| `event_test_error_formats_message` | N/A | Yes | Pass |
| `payload_assertion_reports_expected_serialization_failure` | N/A | Yes | Pass |
| `payload_assertion_reports_actual_serialization_failure` | N/A | Yes | Pass |
| `payload_to_value_reports_serialization_failure` | N/A | Yes | Pass |
| `event_test_result_exposes_published_events` | N/A | Yes | Pass |

## crates/test-utils/src/cqrs.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `test_command_handler` | N/A | Yes | Pass |
| `test_query_handler` | N/A | Yes | Pass |
| `test_aggregate_command` | N/A | Yes | Pass |
| `test_eventual_consistency` |  | Yes | Pass |
| `test_order_saga` |  | Yes | Pass |
| `mock_repository_saves_entity` | N/A | Yes | Pass |
| `mock_repository_records_interactions` | N/A | Yes | Pass |
| `mock_repository_fails_on_configured_error` | N/A | Yes | Pass |
| `stub_query_store_returns_data` | N/A | Yes | Pass |
| `event_verifier_records_events` | N/A | Yes | Pass |
| `eventual_consistency_tester_waits_for_condition` | N/A | Yes | Pass |
| `eventual_consistency_tester_times_out` | N/A | Yes | Pass |
| `eventual_consistency_tester_waits_for_value` | N/A | Yes | Pass |
| `saga_tester_tracks_participants` | N/A | Yes | Pass |
| `saga_tester_verifies_all_updated` | N/A | Yes | Pass |
| `saga_tester_records_events` | N/A | Yes | Pass |
| `saga_tester_verifies_event_sequence` | N/A | Yes | Pass |

## crates/test-utils/src/async_utils.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `test_with_timeout` |  | Yes | Pass |
| `test_blocking_operation` |  | Yes | Pass |
| `test_with_cancellation` |  | Yes | Pass |
| `default_test_timeout_is_five_seconds` | N/A | Yes | Pass |
| `short_test_timeout_is_one_second` | N/A | Yes | Pass |
| `long_test_timeout_is_thirty_seconds` | N/A | Yes | Pass |

## crates/test-utils/src/fixtures.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `test_builder_pattern` | N/A | Yes | Pass |
| `test_fake_data_generation` | N/A | Yes | Pass |
| `test_serialization_helper` | N/A | Yes | Pass |
| `test_fixture_composition` | N/A | Yes | Pass |
| `test_fixture_functions` | N/A | Yes | Pass |

## crates/test-utils/src/temp.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `test_with_temp_dir` |  | Yes | Pass |
| `test_temp_dir_creation` | N/A | Yes | Pass |
| `test_temp_dir_cleanup` | N/A | Yes | Pass |
| `test_unique_name_generation` | N/A | Yes | Pass |
| `test_path_utils_join` | N/A | Yes | Pass |
| `test_test_output_creation` | N/A | Yes | Pass |
| `test_test_output_file_path` | N/A | Yes | Pass |

## crates/test-utils/src/mocks/event_bus.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `event_plane_variants_match_expected_values` | N/A | Yes | Pass |

## crates/test-utils/src/cqrs/observability.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `test_command_metrics` |  | Yes | Pass |
| `metrics_collector_records_commands` | N/A | Yes | Pass |
| `metrics_collector_calculates_avg_duration` | N/A | Yes | Pass |
| `metrics_collector_tracks_events` | N/A | Yes | Pass |
| `trace_collector_records_traces` | N/A | Yes | Pass |
| `trace_collector_verifies_event_flow` | N/A | Yes | Pass |

## crates/test-utils/src/cqrs/security.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `test_command_authorization` |  | Yes | Pass |
| `mock_auth_grants_and_checks_permissions` | N/A | Yes | Pass |
| `mock_auth_records_audit_trail` | N/A | Yes | Pass |
| `mock_auth_counts_denials` | N/A | Yes | Pass |
| `mock_auth_filters_audit_by_user` | N/A | Yes | Pass |
| `input_sanitizer_detects_sql_injection` | N/A | Yes | Pass |
| `input_sanitizer_detects_xss` | N/A | Yes | Pass |
| `input_sanitizer_detects_path_traversal` | N/A | Yes | Pass |
| `input_sanitizer_sanitizes_input` | N/A | Yes | Pass |

## crates/app/tests/dummy_integration.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `app_integration_environment_ready` | Ensures the integration test harness is operational. | Yes | Pass |

## crates/app/tests/integration_tests.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `maintains_event_bus_api_contract_across_boundaries` | by validating that adapter implementations fulfill port contracts expected by the app layer. | Yes | Pass |
| `propagates_errors_across_module_boundaries` | this test should be enhanced to verify specific error types. | Yes | Pass |
| `validates_integration_performance_meets_baseline` | Current baseline: <50ms for batch operations across hexagonal boundaries. | Yes | Pass |

## crates/cli/src/main.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `main_runs_successfully` | N/A | Yes | Pass |

## crates/adapters/src/lib.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `adapters_compilation_works` | N/A | Yes | Pass |

## crates/domain/src/lib.rs
| Test Name | Behavior | Deterministic | Status |
|-----------|----------|---------------|--------|
| `domain_error_is_send_and_sync` | Initial placeholder error. | Yes | Pass |
