// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// Copyright (C) 2021 Subspace Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the offences module.

#![cfg(test)]

use super::*;
use crate::mock::{
    new_test_ext, offence_reports, with_on_offence_fractions, Event, Offence, Offences, System,
    KIND,
};
use frame_system::{EventRecord, Phase};
use schnorrkel::Keypair;
use sp_core::Public;
use sp_runtime::Perbill;

fn generate_farmer_id() -> FarmerId {
    let keypair = Keypair::generate();
    FarmerId::from_slice(&keypair.public.to_bytes())
}

#[test]
fn should_report_an_farmer_and_trigger_on_offence() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            time_slot,
            offenders: vec![generate_farmer_id()],
        };

        // when
        Offences::report_offence(offence).unwrap();

        // then
        with_on_offence_fractions(|f| {
            assert_eq!(f.clone(), vec![Perbill::from_percent(25)]);
        });
    });
}

#[test]
fn should_not_report_the_same_farmer_twice_in_the_same_slot() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            time_slot,
            offenders: vec![generate_farmer_id()],
        };
        Offences::report_offence(offence.clone()).unwrap();
        with_on_offence_fractions(|f| {
            assert_eq!(f.clone(), vec![Perbill::from_percent(25)]);
            f.clear();
        });

        // when
        // report for the second time
        assert_eq!(
            Offences::report_offence(offence),
            Err(OffenceError::DuplicateReport)
        );

        // then
        with_on_offence_fractions(|f| {
            assert_eq!(f.clone(), vec![]);
        });
    });
}

#[test]
fn should_report_in_different_time_slot() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let mut offence = Offence {
            time_slot,
            offenders: vec![generate_farmer_id()],
        };
        Offences::report_offence(offence.clone()).unwrap();
        with_on_offence_fractions(|f| {
            assert_eq!(f.clone(), vec![Perbill::from_percent(25)]);
            f.clear();
        });

        // when
        // report for the second time
        offence.time_slot += 1;
        Offences::report_offence(offence).unwrap();

        // then
        with_on_offence_fractions(|f| {
            assert_eq!(f.clone(), vec![Perbill::from_percent(25)]);
        });
    });
}

#[test]
fn should_deposit_event() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            time_slot,
            offenders: vec![generate_farmer_id()],
        };

        // when
        Offences::report_offence(offence).unwrap();

        // then
        assert_eq!(
            System::events(),
            vec![EventRecord {
                phase: Phase::Initialization,
                event: Event::offences(crate::Event::Offence(KIND, time_slot.encode())),
                topics: vec![],
            }]
        );
    });
}

#[test]
fn doesnt_deposit_event_for_dups() {
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let offence = Offence {
            time_slot,
            offenders: vec![generate_farmer_id()],
        };
        Offences::report_offence(offence.clone()).unwrap();
        with_on_offence_fractions(|f| {
            assert_eq!(f.clone(), vec![Perbill::from_percent(25)]);
            f.clear();
        });

        // when
        // report for the second time
        assert_eq!(
            Offences::report_offence(offence),
            Err(OffenceError::DuplicateReport)
        );

        // then
        // there is only one event.
        assert_eq!(
            System::events(),
            vec![EventRecord {
                phase: Phase::Initialization,
                event: Event::offences(crate::Event::Offence(KIND, time_slot.encode())),
                topics: vec![],
            }]
        );
    });
}

#[test]
fn reports_if_an_offence_is_dup() {
    type TestOffence = Offence<FarmerId>;

    new_test_ext().execute_with(|| {
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let farmer_0 = generate_farmer_id();
        let farmer_1 = generate_farmer_id();

        let offence = |time_slot, offenders| TestOffence {
            time_slot,
            offenders,
        };

        let mut test_offence = offence(time_slot, vec![farmer_0.clone()]);

        // the report for farmer 0 at time slot 42 should not be a known
        // offence
        assert!(
            !<Offences as ReportOffence<_, TestOffence>>::is_known_offence(
                &test_offence.offenders,
                &test_offence.time_slot
            )
        );

        // we report an offence for farmer 0 at time slot 42
        Offences::report_offence(test_offence.clone()).unwrap();

        // the same report should be a known offence now
        assert!(
            <Offences as ReportOffence<_, TestOffence>>::is_known_offence(
                &test_offence.offenders,
                &test_offence.time_slot
            )
        );

        // and reporting it again should yield a duplicate report error
        assert_eq!(
            Offences::report_offence(test_offence.clone()),
            Err(OffenceError::DuplicateReport)
        );

        // after adding a new offender to the offence report
        test_offence.offenders.push(farmer_1.clone());

        // it should not be a known offence anymore
        assert!(
            !<Offences as ReportOffence<_, TestOffence>>::is_known_offence(
                &test_offence.offenders,
                &test_offence.time_slot
            )
        );

        // and reporting it again should work without any error
        assert_eq!(Offences::report_offence(test_offence.clone()), Ok(()));

        // creating a new offence for the same farmers on the next slot
        // should be considered a new offence and therefore not known
        let test_offence_next_slot = offence(time_slot + 1, vec![farmer_0, farmer_1]);
        assert!(
            !<Offences as ReportOffence<_, TestOffence>>::is_known_offence(
                &test_offence_next_slot.offenders,
                &test_offence_next_slot.time_slot
            )
        );
    });
}

#[test]
fn should_properly_count_offences() {
    // We report two different farmers for the same issue. Ultimately, the 1st farmer
    // should have `count` equal 2 and the count of the 2nd one should be equal to 1.
    new_test_ext().execute_with(|| {
        // given
        let time_slot = 42;
        assert_eq!(offence_reports(KIND, time_slot), vec![]);

        let farmer_1 = generate_farmer_id();
        let farmer_2 = generate_farmer_id();

        let offence1 = Offence {
            time_slot,
            offenders: vec![farmer_1.clone()],
        };
        let offence2 = Offence {
            time_slot,
            offenders: vec![farmer_2.clone()],
        };
        Offences::report_offence(offence1).unwrap();
        with_on_offence_fractions(|f| {
            assert_eq!(f.clone(), vec![Perbill::from_percent(25)]);
            f.clear();
        });

        // when
        // report for the second time
        Offences::report_offence(offence2).unwrap();

        // then
        // the 1st farmer should have count 2 and the 2nd one should be reported only once.
        assert_eq!(
            offence_reports(KIND, time_slot),
            vec![
                OffenceDetails { offender: farmer_1 },
                OffenceDetails { offender: farmer_2 },
            ]
        );
    });
}
