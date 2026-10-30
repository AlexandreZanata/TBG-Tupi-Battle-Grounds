# Art implementation verification transcript

These are the actual command outputs from the isolated `art/missing-assets`
worktree. The phase is blocked on the Augment byte budget and live Practice
observation; successful compilation does not resolve either requirement.

## `cargo fmt --all -- --check`

Exit: `0`. Elapsed: 0.31 s.

```text
[no output]
```

## `cargo clippy --workspace -- -D warnings`

Exit: `0`. Elapsed: 3.51 s.

```text
    Checking tbg-core v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-core)
    Checking tbg-server v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-server)
    Checking tbg-balance v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-balance)
    Checking tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.44s
```

## `cargo test --workspace`

Exit: `0`. Elapsed: 142.59 s.

```text
   Compiling tbg-core v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-core)
   Compiling tbg-server v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-server)
   Compiling tbg-balance v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-balance)
   Compiling tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-client)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.52s
     Running unittests src/lib.rs (target/debug/deps/tbg_balance-ebafe18167df35dc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/tbg_balance-05b7be23bcf2ab27)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/arcana_effects.rs (target/debug/deps/arcana_effects-a94903ef7acb5b05)

running 6 tests
test hunters_focus_adds_damage_only_against_a_controlled_target ... ok
test emergency_care_boosts_low_hp_heals_and_purification_cleanses_control ... ok
test shieldwall_and_combined_arms_reduce_tank_damage_and_stop_without_dps ... ok
test quickdraw_stays_inside_the_attack_speed_cap ... ok
test battle_rush_moves_dps_faster_only_with_a_living_tank ... ok
test quickdraw_and_deadeye_change_attack_timing_in_combat ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/contract.rs (target/debug/deps/contract-be684a232b16f550)

running 7 tests
test c5_every_tunable_number_is_registered ... ok
test c6_the_simulation_reads_the_ruleset ... ok
test c9_statistics_are_sound ... ok
test c3_results_do_not_depend_on_thread_count ... ok
test c11_every_policy_plays_legal_complete_matches ... ok
test c8_a_flat_roster_flags_nothing ... ok
test c7_planted_imbalances_are_caught_for_every_unit has been running for over 60 seconds
test c7_planted_imbalances_are_caught_for_every_unit ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 64.14s

     Running tests/value.rs (target/debug/deps/value-cbd6d0b7c2e5da9b)

running 12 tests
test lab_spawn_at_places_a_unit_exactly ... ok
test interventions_reject_negative_hp_and_zero_cost ... ok
test reference_lambda_is_the_median_and_must_be_positive ... ok
test zero_intervention_is_the_canon_ruleset ... ok
test static_baseline_is_cost_aware ... ok
test doubled_damage_beats_a_twenty_percent_increase ... ok
test null_scenario_has_no_side_bias ... ok
test scenario_is_deterministic ... ok
test damage_plus_attack_speed_has_a_nonzero_interaction ... ok
test unrelated_intervention_has_zero_interaction ... ok
test bot_avoid_parses_and_never_buys_the_avoided_unit ... ok
test pve_of_a_noop_is_zero ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running unittests src/main.rs (target/debug/deps/tbg_client-257c0d9564cc84c5)

running 48 tests
test arena::strongholds::tests::every_form_has_its_own_silhouette ... ok
test arena::strongholds::tests::war_banner_radius_is_the_authoritative_one ... ok
test arena::strongholds::tests::artillery_recoil_matches_the_authoritative_cadence ... ok
test art::anim_tests::walk_delivers_two_footsteps_without_boundary_duplicates ... ok
test art::anim_tests::strike_time_is_the_strike_key_pose ... ok
test arena::strongholds::tests::damage_states_follow_the_kinds_own_max_hp ... ok
test art::anim_tests::planted_foot_holds_world_x_while_the_figure_moves ... ok
test art::anim_tests::all_easings_preserve_keys_and_hold_preserves_contact ... ok
test art::archer_v2_tests::full_draw_is_within_both_arms_reach ... ok
test config::stronghold_tests::picker_names_parse_and_classic_never_does ... ok
test config::tests::invite_token_accepts_uppercase_hex ... ok
test config::arcana_tests::the_editor_document_round_trips_and_legacy_rows_migrate ... ok
test arena::strongholds::tests::damage_only_removes_pieces_and_keeps_the_kind ... ok
test config::tests::invite_token_hex_round_trip ... ok
test config::tests::invite_token_rejects_bad_length_and_chars ... ok
test config::tests::native_challenge_link_contains_the_token ... ok
test net::login::tests::no_login_without_key_or_inside_a_match ... ok
test net::login::tests::fresh_session_with_stored_key_logs_in ... ok
test config::tests::removing_a_consumed_invite_preserves_server_and_lag ... ok
test net::login::tests::resumed_session_never_logs_in_again ... ok
test net::socket::tests::challenge_result_reconnects_without_consuming_the_invite_again ... ok
test net::socket::tests::acknowledged_challenge_reconnects_without_recreating_the_invite ... ok
test sync::timeline::tests::a_late_view_never_moves_time_backwards_and_keeps_its_events ... ok
test sync::timeline::tests::spawn_and_death_follow_the_render_time ... ok
test ui::fortify::tests::offers_follow_the_server_checks ... ok
test ui::battle_hud::tests::stronghold_numbers_follow_the_view_seat ... ok
test ui::fortify::tests::phone_text_names_the_next_step_and_its_price ... ok
test ui::hud_top::tests::keeps_short_names_and_fits_long_ones ... ok
test ui::augment_data_tests::art_manifest_matches_every_canonical_augment_and_round_timing ... ok
test art::anim_tests::archer_budget_and_release_anchor_obey_p15 ... ok
test ui::inspect::tests::report_follows_live_hp_for_own_and_enemy_units ... ok
test sync::timeline::tests::a_reconnect_snaps_once_and_a_new_match_starts_over ... ok
test ui::augment_data_tests::all_augment_illustrations_and_rarity_frames_are_shipped ... ok
test sync::timeline::tests::an_augment_pause_freezes_and_resumes_without_a_jump ... ok
test ui::army_tests::army_label_counts_living_units_and_summons_apart ... ok
test sync::timeline::tests::a_burst_of_views_in_one_frame_plays_in_order ... ok
test ui::stronghold_data_tests::picker_data_matches_core_constants ... ok
test ui::stronghold_data_tests::every_kind_tier_and_branch_matches_core_stats ... ok
test art::archer_v2_tests::every_candidate_form_fits_the_one_slot_budget ... ok
test sync::timeline::tests::stable_100ms_views_move_without_stalls_or_jumps ... ok
test ui::evolve::tests::open_panel_tracks_energy_without_reopening_or_replacing_buttons ... ok
test ui::queue::tests::challenge_waiting_queries_and_copy_share_controls_work_together ... ok
test ui::result::tests::rematch_controls_follow_the_offer_without_query_conflicts ... ok
test arena::strongholds::tests::every_form_stays_within_its_triangle_budget ... ok
test sync::timeline::tests::high_jitter_degrades_predictably_and_never_runs_backwards ... ok
test sync::timeline::tests::moderate_jitter_stays_smooth_and_beats_the_old_formula ... ok
test ui::glyphs::tests::ui_strings_are_ascii ... ok
test sync::timeline::tests::the_real_vps_trace_is_smooth ... ok

test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running unittests src/lib.rs (target/debug/deps/tbg_core-86973f9a50b6ba5c)

running 3 tests
test arcana::tests::every_new_node_names_its_prerequisites ... ok
test arcana::tests::children_and_supremes_are_rejected_without_prerequisites ... ok
test arcana::tests::legacy_migration_preserves_ranks_and_fills_originals ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/accounts.rs (target/debug/deps/accounts-c40947e32285b995)

running 6 tests
test br_p01_bad_shapes_rejected ... ok
test br_p01_reserved_and_blocked_rejected ... ok
test br_p02_key_normalization ... ok
test br_p02_bad_keys_rejected ... ok
test br_p02_key_round_trip ... ok
test br_p01_valid_names_pass ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/arcana.rs (target/debug/deps/arcana-2f838895d3eb4b85)

running 7 tests
test br_ar05_ar06_ar07_ar08_ar09_conditions_are_exact ... ok
test br_ar10_deadeye_adds_range_only_to_conventional_ranged ... ok
test br_ar03_children_and_supremes_require_their_parents_and_completion ... ok
test br_ar13_legacy_six_node_presets_migrate_to_twelve_points ... ok
test br_ar04_ar05_bonus_by_base_archetype_without_conditional_double_counting ... ok
test summons_keep_neutral_arcanas_and_presets_apply_per_seat ... ok
test br_ar02_ar03_exactly_1842_complete_presets_with_costs_and_prerequisites ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/augment_effects.rs (target/debug/deps/augment_effects-75d6ec47fc4e30e5)

running 14 tests
test attack_speed_and_move_caps_hold_under_many_copies ... ok
test faction_and_evolved_filters_only_reach_their_units ... ok
test forced_march_stamps_only_new_units ... ok
test bounty_and_stone_mend_credit_the_killer ... ok
test damage_reduction_aura_respects_position_condition_and_cap ... ok
test forgo_discount_and_convergence_are_dynamic ... ok
test great_wounds_cuts_direct_healing ... ok
test lifesteal_heals_the_primary_attacker_only ... ok
test second_wind_survives_one_lethal_hit_per_copy ... ok
test lingering_hex_lengthens_root_without_changing_strength ... ok
test situational_damage_uses_hit_time_conditions ... ok
test stat_augments_add_in_bp_and_truncate_once ... ok
test tower_fury_counts_only_stronghold_kills ... ok
test last_bastion_raises_one_shield_and_expires ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/augment_pause.rs (target/debug/deps/augment_pause-82e3b02d8593ef9c)

running 9 tests
test br_ag01_the_next_draw_counts_only_effective_combat ... ok
test br_ag05_a_confirmed_pick_cannot_change_or_repeat ... ok
test br_ag14_a_pick_stays_hidden_until_the_round_resolves ... ok
test br_ag13_both_confirmations_resume_at_once_for_both ... ok
test br_ag12_gameplay_intentions_are_rejected_while_paused ... ok
test br_ag13_one_confirmation_never_resumes_the_battle ... ok
test br_ag12_a_round_freezes_units_attacks_cooldowns_and_the_economy ... ok
test br_ag13_the_deadline_resumes_with_an_idle_player ... ok
test br_ag13_a_replay_keeps_paused_choices_in_order ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/augments_infinite.rs (target/debug/deps/augments_infinite-076ea58f86cc16ac)

running 10 tests
test br_ag08_counts_are_bounded_and_compact ... ok
test br_ag04_expiry_grants_forgo_once_when_configured ... ok
test br_ag07_a_short_player_may_wait_and_buy_later ... ok
test br_ag06_forgo_marks_the_round_and_pays_at_most_once ... ok
test br_ag05_one_purchase_per_round_and_repeats_are_allowed_later ... ok
test field_kit_copies_stack_on_existing_units ... ok
test br_ag03_each_cycle_offers_all_forty_and_reshuffles ... ok
test br_ag01_first_round_is_at_one_minute_and_repeats_forever ... ok
test br_ag08_every_catalog_id_can_be_bought_and_stacked ... ok
test br_ag02_four_distinct_options_and_both_seats_see_them ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/contract_constants.rs (target/debug/deps/contract_constants-5d9bd6b282fc3906)

running 6 tests
test account_and_protocol_limits ... ok
test open_ended_match_and_lane ... ok
test shop_slots_stronghold_and_evolution ... ok
test uncapped_energy_and_interest ... ok
test synergies ... ok
test ranked_rating_constants_and_table ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/contract_determinism.rs (target/debug/deps/contract_determinism-f3b80e3e3e1a6af8)

running 2 tests
test surrender_is_only_a_forfeit ... ok
test bot_matches_are_frozen ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/contract_lab.rs (target/debug/deps/contract_lab-d6073d3c2f6e2167)

running 3 tests
test c1_simulation_sources_have_no_hidden_inputs ... ok
test c10_online_crates_only_play_the_canonical_ruleset ... ok
test c2_per_tick_traces_are_frozen ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/contract_protocol.rs (target/debug/deps/contract_protocol-2d0c8bb1c87cb451)

running 7 tests
test client_bytes_are_frozen ... ok
test v10_augment_intentions_are_appended ... ok
test error_code_order_is_frozen ... ok
test protocol_version ... ok
test v8_arcana_messages_are_appended_and_v11_resizes_the_preset ... ok
test match_mode_order_is_frozen ... ok
test client_variant_order_is_frozen ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/contract_protocol_server.rs (target/debug/deps/contract_protocol_server-3c1cd406c9d2eaae)

running 2 tests
test server_bytes_are_frozen ... ok
test server_variant_order_is_frozen ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/contract_roster.rs (target/debug/deps/contract_roster-4cf757e91c137172)

running 3 tests
test base_stats_match_game_design_table ... ok
test roster_composition ... ok
test evolution_forms_are_frozen ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/contract_strongholds.rs (target/debug/deps/contract_strongholds-4366580d91420467)

running 14 tests
test br_sk06_every_branch_changes_the_real_numbers ... ok
test br_sk01_four_free_kinds_default_bastion_classic_only_for_the_lab ... ok
test br_sk03_kind_bases_keep_their_roles ... ok
test br_sk05_tier_one_picks_the_branch_tier_two_repeats_it ... ok
test br_sk05_costs_are_three_and_four_energy_and_need_the_energy ... ok
test br_sk05_no_fortification_after_the_end ... ok
test br_sk08_both_seats_see_both_towers ... ok
test br_sk02_royal_mint_pays_more_with_a_higher_ceiling ... ok
test br_sk07_more_max_hp_keeps_the_ratio_never_heals ... ok
test br_sk06_artillery_branches_change_the_real_shot ... ok
test br_sk04_the_aura_is_read_at_hit_time_and_ends_outside ... ok
test br_sk02_spoils_pay_eligible_enemy_deaths_within_the_window_cap ... ok
test br_sk04_war_banner_reaches_only_conventional_allies_in_its_radius ... ok
test br_sk09_kind_matches_still_end_by_destruction_and_replay_identically ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/economy_interest.rs (target/debug/deps/economy_interest-ffa62ea447e98274)

running 11 tests
test br_e05_every_five_energy_adds_a_quarter_of_the_base_regen ... ok
test br_e03_overspend_is_rejected_and_spend_is_the_only_path ... ok
test br_e03_no_regen_before_running_or_after_ended ... ok
test br_e05_the_boost_follows_the_balance_at_once ... ok
test br_e07_energy_saturates_without_overflow ... ok
test br_e06_the_remainder_survives_a_serialize_roundtrip ... ok
test br_e06_a_lab_forgo_reward_is_paid_once_per_round ... ok
test br_e08_the_bounty_window_resets_every_ten_seconds_of_combat ... ok
test br_e05_the_rate_is_exact_over_time ... ok
test br_e05_the_cap_limits_the_boost_not_the_energy ... ok
test br_e06_banking_compounds_and_spending_resets_it ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/open_army.rs (target/debug/deps/open_army-335180dc8331263a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/rating.rs (target/debug/deps/rating-507d4ac1a068a7e4)

running 5 tests
test br_r03_favorite_gains_little_underdog_loses_little ... ok
test br_r03_draw_moves_toward_each_other ... ok
test br_r03_equal_win_moves_by_k_half ... ok
test br_r05_constants ... ok
test br_r05_floor_holds ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/rules.rs (target/debug/deps/rules-629db6721660b2a7)

running 14 tests
test purchase_rejected_during_countdown ... ok
test protocol_roundtrip_and_garbage_rejected ... ok
test overspend_is_rejected_and_energy_unchanged ... ok
test forfeit_gives_win_to_other_seat ... ok
test evolution_branch_gating ... ok
test evolve_other_seats_unit_rejected ... ok
test view_hides_opponent_energy_and_offers ... ok
test stronghold_destroyed_wins ... ok
test synergy_levels_at_two_and_four_and_drop_on_death ... ok
test energy_regenerates_without_cap_and_pays_interest ... ok
test br_s04_a_purchase_redraws_all_five_offers_and_nothing_rotates ... ok
test unequal_strongholds_do_not_end_by_time ... ok
test empty_match_never_ends_by_time ... ok
test same_seed_and_script_same_hash ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running unittests src/lib.rs (target/debug/deps/tbg_server-efb70f16bdd4a8e7)

running 14 tests
test lobby::tests::cleanup_rematch_with_missing_session_notifies_remaining_player ... ok
test lobby::tests::cleanup_removes_expired_challenge_and_resets_owner ... ok
test lobby::tests::delayed_account_preset_cannot_change_a_waiting_challenge ... ok
test lobby::tests::login_takeover_moves_the_rematch_token ... ok
test lobby::tests::cleanup_expired_rematch_notifies_both_players ... ok
test lobby::tests::resume_cannot_start_an_expired_rematch_before_cleanup ... ok
test lobby::tests::reconnect_during_drain_keeps_rematch_from_starting ... ok
test lobby::tests::cleanup_removes_many_expired_challenges_in_one_sweep ... ok
test ws::admission::tests::forwarded_ip_trusted_only_from_local_proxy ... ok
test store::arcanas::tests::a_saved_preset_round_trips ... ok
test store::strongholds::tests::saved_kind_round_trips_and_bad_rows_are_ignored ... ok
test store::arcanas::tests::a_six_byte_row_migrates_in_place_to_the_twelve_point_tree ... ok
test store::leaderboard::tests::order_and_tiebreaks ... ok
test match_task::tests::challenge_initial_detached_forfeits ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s

     Running unittests src/bin/tbg-bots.rs (target/debug/deps/tbg_bots-400872541a2451a0)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/tbg_server-8d1002a1ebf05683)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/accounts.rs (target/debug/deps/accounts-45c032f228e89bfc)

running 12 tests
test br_p02_migrate_twice_is_noop ... ok
test br_p02_bad_secret_and_unknown_name_share_badkey ... ok
test br_p01_duplicate_name_other_case_taken ... ok
test br_p01_register_replies_key_that_parses ... ok
test br_p01_register_then_login ... ok
test br_p04_account_messages_refused_in_match ... ok
test br_p01_register_name_taken_and_invalid ... ok
test br_p04_resumed_session_reports_its_account ... ok
test br_p02_login_same_profile_and_bad_secret ... ok
test br_p01_register_rate_limited_per_ip ... ok
test br_p03_second_login_takes_over_live_match ... ok
test br_p03_expired_session_returns_as_guest_and_ranked_is_refused ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.14s

     Running tests/arcanas.rs (target/debug/deps/arcanas-504d89bbcd5363ea)

running 3 tests
test br_ar05_presets_are_frozen_at_pairing_and_public_in_match_start ... ok
test br_ar08_an_account_keeps_its_preset_across_devices ... ok
test br_ar06_only_complete_presets_while_idle ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/contract.rs (target/debug/deps/contract-0c754f0706b42e6a)

running 3 tests
test production_defaults ... ok
test br_r02_casual_pairs_first_come_first_served ... ok
test br_r02_ranked_is_fifo_and_never_mixes_with_casual ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s

     Running tests/contract_challenge.rs (target/debug/deps/contract_challenge-95ccbf53750ff6d9)

running 2 tests
test br_r08_quick_challenge_is_single_use_and_casual ... ok
test br_r09_rematch_requires_both_and_never_enters_a_queue ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running tests/contract_registration.rs (target/debug/deps/contract_registration-56cac6133855ae8e)

running 2 tests
test br_p01_taken_names_do_not_consume_the_register_limit ... ok
test register_limit_still_caps_created_accounts ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/lab_equivalence.rs (target/debug/deps/lab_equivalence-c05e6b9e1c143a32)

running 2 tests
test c4b_realtime_server_match_equals_headless_replay ... ignored, a full 1x match takes 2-4 minutes; run once per balance phase and release
test c4a_fast_server_match_equals_headless_replay ... ok

test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 11.31s

     Running tests/leaderboard.rs (target/debug/deps/leaderboard-bee7db5ec66feef8)

running 4 tests
test api_rate_limit_returns_429 ... ok
test br_r07_leaderboard_and_player_profile ... ok
test br_r07_in_game_leaderboard_matches_api ... ok
test br_r07_limit_clamped_to_100 ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.30s

     Running tests/online.rs (target/debug/deps/online-aad1c89807ce338a)

running 31 tests
test draining_server_refuses_new_matches ... ok
test challenge_invite_rate_limit ... ok
test challenge_cancel_removes_invite ... ok
test private_room_pairs_owner_and_guest ... ok
test challenge_pairs_owner_and_guest_without_code ... ok
test malformed_frame_closes_connection ... ok
test challenge_rejects_third_guest_and_owner_token ... ok
test challenge_only_one_guest_wins ... ok
test rate_limit_closes_flooder ... ok
test challenge_freezes_arcanas_through_reconnect_and_rematch ... ok
test challenge_owner_reload_resumes_waiting ... ok
test intent_is_acknowledged_next_tick ... ok
test challenge_guest_reload_resumes_match ... ok
test rematch_decline_closes ... ok
test two_guests_same_ip_join_different_challenges ... ok
test version_mismatch_rejected ... ok
test br_m06_surrender_forfeits_once ... ok
test rematch_requires_both ... ok
test rematch_does_not_enter_public_queue ... ok
test cannot_evolve_opponent_unit ... ok
test rematch_duplicate_requests_idempotent ... ok
test br_sk01_kinds_freeze_at_pairing_fortify_online_and_survive_reconnect ... ok
test rematch_reload_keeps_offer ... ok
test challenge_expires ... ok
test forfeit_after_window ... ok
test br_ag_online_paused_round_hidden_pick_reconnect_and_deadline ... ok
test br_u07_online_army_stays_in_sync ... ok
test records_insert_and_count ... ok
test reconnect_resumes_match ... ok
test two_bots_finish_a_match ... ok
test protocol_pong_keeps_socket_alive ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.02s

     Running tests/ranked.rs (target/debug/deps/ranked-c69ff190294f411d)

running 6 tests
test br_r01_ranked_pairing_and_decisive_rating ... ok
test br_r01_guest_ranked_needs_login ... ok
test br_m06_ranked_surrender_is_a_loss ... ok
test br_r04_ranked_forfeit_is_a_loss ... ok
test br_r06_casual_matches_never_touch_rating ... ok
test br_r02_queues_never_mix ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.12s

   Doc-tests tbg_balance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests tbg_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests tbg_server

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`

Exit: `0`. Elapsed: 41.55 s.

```text
    Checking web-sys v0.3.106
    Checking tbg-core v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-core)
    Checking wgpu-types v30.0.1
    Checking glow v0.17.0
    Checking cpal v0.17.3
    Checking winit v0.30.13
    Checking ewebsock v0.8.0
    Checking rodio v0.22.2
    Checking accesskit_winit v0.33.2
    Checking bevy_reflect v0.20.0
    Checking wgpu-naga-bridge v30.0.1
    Checking wgpu-hal v30.0.1
    Checking wgpu-core-deps-wasm v30.0.1
    Checking wgpu-core v30.0.1
    Checking bevy_ecs v0.20.0
    Checking bevy_math v0.20.0
    Checking bevy_curve v0.20.0
    Checking bevy_shape v0.20.0
    Checking bevy_color v0.20.0
    Checking wgpu v30.0.1
    Checking bevy_app v0.20.0
    Checking bevy_time v0.20.0
    Checking bevy_transform v0.20.0
    Checking bevy_input v0.20.0
    Checking bevy_log v0.20.0
    Checking bevy_a11y v0.20.0
    Checking bevy_state v0.20.0
    Checking bevy_clipboard v0.20.0
    Checking bevy_diagnostic v0.20.0
    Checking bevy_asset v0.20.0
    Checking bevy_window v0.20.0
    Checking bevy_image v0.20.0
    Checking bevy_mesh v0.20.0
    Checking bevy_shader v0.20.0
    Checking bevy_audio v0.20.0
    Checking bevy_text v0.20.0
    Checking bevy_camera v0.20.0
    Checking bevy_material v0.20.0
    Checking bevy_extract v0.20.0
    Checking bevy_light v0.20.0
    Checking bevy_input_focus v0.20.0
    Checking bevy_sprite v0.20.0
    Checking bevy_render v0.20.0
    Checking bevy_ui v0.20.0
    Checking bevy_winit v0.20.0
    Checking bevy_core_pipeline v0.20.0
    Checking bevy_sprite_render v0.20.0
    Checking bevy_pbr v0.20.0
    Checking bevy_ui_render v0.20.0
    Checking bevy_internal v0.20.0
    Checking bevy v0.20.0
    Checking tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-client)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 41.50s
```

## `node --test tests/arcana.test.js tests/lobby.test.js tests/stronghold.test.js`

Exit: `0`. Elapsed: 0.18 s.

```text
✔ BR-AR02: the fifteen-node table, costs and prerequisites match the canon (2.693074ms)
✔ BR-AR02: twelve points, bounded ranks, valid default and 1842 complete presets (100.544036ms)
✔ BR-AR03: children need a parent and supremes need the complete family (0.287866ms)
✔ BR-AR13: a schema-1 six-node preset migrates deterministically (0.195776ms)
✔ every manifest art asset exists locally and has a safe filename (0.768467ms)
✔ BR-AR03/10 preview: roster includes Nester, Rampart; Renewal only reaches an existing healer (0.157561ms)
✔ lobby text projection never targets an attribute the Arcana editor writes (1.597671ms)
✔ lobby and Stronghold picker projections never overlap (0.636506ms)
✔ BR-SK01: four selectable kinds, Bastion by default, Classic never offered (1.306586ms)
✔ prose holds no hand-written numbers: every figure comes from tested stats (0.721352ms)
✔ superlatives in the copy hold for the data (0.13434ms)
✔ BR-SK02/SK04/SK05: derived effects, passives and costs read as the rules say (4.337515ms)
✔ every kind has its tower drawing and the HUD understands every offer state (0.868056ms)
ℹ tests 13
ℹ suites 0
ℹ pass 13
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 147.71289
```

## `trunk build --release`

Exit: `0`. Elapsed: 120.26 s.

```text
   Compiling tbg-client v0.1.0 (/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-art-wt/crates/tbg-client)
    Finished `release` profile [optimized] target(s) in 1m 23s
```

## Additional art loader tests

`node --test --test-name-pattern='opening a round|canonical rarity' tests/art.test.js`
Exit: `0`.

```text
✔ BR-AG12: opening a round preloads exactly its four illustrations and required frames (6.952955ms)
✔ canonical rarity names and their visual palettes are preserved (0.629276ms)
ℹ tests 2
ℹ suites 0
ℹ pass 2
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 46.961649
```

## Complete Node test command with the new budget contract

`node --test tests/arcana.test.js tests/lobby.test.js tests/stronghold.test.js tests/art.test.js`
Exit: `1`. This later run reported file-level results:

```text
✔ tests/arcana.test.js (931.584569ms)
✖ tests/art.test.js (1533.348809ms)
✔ tests/lobby.test.js (302.514214ms)
✔ tests/stronghold.test.js (302.940333ms)
ℹ tests 4
ℹ suites 0
ℹ pass 3
ℹ fail 1
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 1612.330463

✖ failing tests:

test at tests/art.test.js:1:1
✖ tests/art.test.js (1533.348809ms)
  'test failed'
```

The failing art budget is retained as a strict assertion. Actual shipped
Augment art is 4,067,598 B against 4,000,000 B. The test is not weakened to
accept the excess.

## Live browser observation

The real lobby was observed at 1920×1080 and 1366×768, including all nine
new Arcana icons and all Stronghold portraits, banners and branch icons.
The request log confirmed no Augment catalog/art loads before an open
round, and no generated Stronghold pictures before the picker opens.
The actual `?gallery=strongholds` scene was inspected at 1920×1080 before
generation to match silhouettes.

Two actual Practice runs stopped with:

```text
page.waitForFunction: Timeout 240000ms exceeded.
```

The second run's last actual view:

```json
{"hidden":false,"clock":"0:07","council":"closed","energy":"1447","army":"2","hpOwn":"5175","hpEnemy":"5175"}
```

This is a blocked acceptance check. Component fixtures do not prove a
real Practice Augment round. Their images visibly carry `ART LAYOUT
FIXTURE — not a live match`.

## Final browser bundle after title layout correction

`cd crates/tbg-client && trunk build --release`
Exit: `0`.

```text
    Finished `release` profile [optimized] target(s) in 0.22s
```

## Labeled Augment component fixtures

`ART_PLAYWRIGHT=<local module> ART_CHROMIUM=<local browser> node scripts/art-browser.cjs fixture`
Exit: `0`. At both desktop sizes all 40 canonical names/descriptions fit;
three unaffordable cards are disabled at 3 Energy; a pick enables Confirm;
confirm emits only `aug:0` and waits for a projected view; Forgo emits its
existing intention. At 100 remaining ticks the clock reads `0:05` and the
progress bar is 25%. No browser errors or art HTTP failures occurred.

```json
{
  "screenshots": [
    {
      "name": "augments-layout-1920x1080.jpg",
      "bytes": 193187,
      "quality": 80
    },
    {
      "name": "augments-layout-1366x768.jpg",
      "bytes": 159909,
      "quality": 80
    }
  ],
  "checks": [
    {
      "viewport": "1920x1080",
      "layoutFixture": true,
      "livePractice": false,
      "fourCanonicalRarities": true,
      "componentContracts": {
        "overflow": [],
        "affordable": [
          true,
          false,
          false,
          false
        ],
        "pickOnly": true,
        "intentionOnly": true,
        "waitsForView": true,
        "forgo": true,
        "timer": "0:05",
        "progress": "25%"
      }
    },
    {
      "viewport": "1366x768",
      "layoutFixture": true,
      "livePractice": false,
      "fourCanonicalRarities": true,
      "componentContracts": {
        "overflow": [],
        "affordable": [
          true,
          false,
          false,
          false
        ],
        "pickOnly": true,
        "intentionOnly": true,
        "waitsForView": true,
        "forgo": true,
        "timer": "0:05",
        "progress": "25%"
      }
    }
  ],
  "errors": [],
  "assetFailures": []
}
```
