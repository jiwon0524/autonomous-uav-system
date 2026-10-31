"""Domain fixtures only: no ROS, telemetry, simulated sensors or flight commands."""
from dataclasses import FrozenInstanceError
import unittest

from src.autonomy.core import (
    InvalidTransition, MissionEvent as E, MissionState as S, MissionStateMachine,
)

PATROL = [E.START_MISSION, E.TAKEOFF_COMPLETED]
TRACKING = PATROL + [E.TARGET_DETECTED, E.SELECT_TARGET, E.START_TRACKING]
PATHS = {
    S.IDLE: [], S.TAKEOFF: [E.START_MISSION], S.PATROL: PATROL,
    S.TARGET_DETECTED: PATROL + [E.TARGET_DETECTED],
    S.TARGET_SELECTED: PATROL + [E.TARGET_DETECTED, E.SELECT_TARGET],
    S.TRACKING: TRACKING, S.TARGET_LOST: TRACKING + [E.TARGET_LOST],
    S.REACQUIRING: TRACKING + [E.TARGET_LOST, E.START_REACQUISITION],
    S.AVOIDING: TRACKING + [E.OBSTACLE_DETECTED],
    S.REPLANNING: TRACKING + [E.OBSTACLE_DETECTED, E.REPLAN_REQUIRED],
    S.RETURN_HOME: PATROL + [E.REQUEST_RETURN_HOME],
    S.LANDING: PATROL + [E.REQUEST_RETURN_HOME, E.HOME_REACHED],
    S.HOLD: TRACKING + [E.REQUEST_HOLD],
    S.MISSION_ABORTED: TRACKING + [E.ABORT_MISSION],
}


def at(state):
    machine = MissionStateMachine()
    for event in PATHS[state]:
        machine.handle_event(event)
    return machine


class MissionTests(unittest.TestCase):
    def test_takeoff_patrol(self):
        machine = MissionStateMachine()
        self.assertIsNone(machine.last_transition)
        self.assertEqual(machine.handle_event(E.START_MISSION).state, S.TAKEOFF)
        self.assertEqual(machine.handle_event(E.TAKEOFF_COMPLETED).state, S.PATROL)

    def test_detection_selection_tracking(self):
        machine = at(S.PATROL)
        for event, state in [(E.TARGET_DETECTED, S.TARGET_DETECTED),
                             (E.SELECT_TARGET, S.TARGET_SELECTED),
                             (E.START_TRACKING, S.TRACKING)]:
            self.assertEqual(machine.handle_event(event).state, state)

    def test_lost_reacquiring_reacquired(self):
        machine = at(S.TRACKING)
        for event, state in [(E.TARGET_LOST, S.TARGET_LOST),
                             (E.START_REACQUISITION, S.REACQUIRING),
                             (E.TARGET_REACQUIRED, S.TRACKING)]:
            self.assertEqual(machine.handle_event(event).state, state)
        self.assertEqual(machine.handle_event(E.TRACKING_COMPLETED).state, S.PATROL)

    def test_avoid_replan_resumes_original_activity(self):
        for original in (S.PATROL, S.TRACKING):
            with self.subTest(original=original):
                machine = at(original)
                self.assertEqual(machine.handle_event(E.OBSTACLE_DETECTED).state, S.AVOIDING)
                self.assertEqual(machine.handle_event(E.REPLAN_REQUIRED).state, S.REPLANNING)
                self.assertEqual(machine.handle_event(E.PLAN_READY).state, original)

    def test_obstacle_cleared_without_replanning(self):
        machine = at(S.PATROL)
        machine.handle_event(E.OBSTACLE_DETECTED)
        self.assertEqual(machine.handle_event(E.OBSTACLE_CLEARED).state, S.PATROL)

    def test_return_landing_idle_from_active_states(self):
        for origin in (S.PATROL, S.TRACKING, S.REACQUIRING, S.AVOIDING, S.REPLANNING, S.HOLD):
            with self.subTest(origin=origin):
                machine = at(origin)
                self.assertEqual(machine.handle_event(E.REQUEST_RETURN_HOME).state, S.RETURN_HOME)
                self.assertEqual(machine.handle_event(E.HOME_REACHED).state, S.LANDING)
                self.assertEqual(machine.handle_event(E.LANDING_COMPLETED).state, S.IDLE)
                with self.assertRaises(InvalidTransition):
                    machine.handle_event(E.RESUME_MISSION)

    def test_every_invalid_state_event_pair_is_atomic(self):
        self.assertEqual(set(PATHS), set(S))
        for state in S:
            for event in E:
                machine = at(state)
                if event in machine.allowed_events():
                    continue
                before = (machine.state, machine.last_transition, machine.allowed_events())
                with self.subTest(state=state, event=event):
                    with self.assertRaises(InvalidTransition) as error:
                        machine.handle_event(event)
                    self.assertEqual(error.exception.state, state)
                    self.assertEqual(error.exception.event, event)
                    self.assertEqual(before, (machine.state, machine.last_transition, machine.allowed_events()))

    def test_allowed_event_contract(self):
        for state in S:
            for event in at(state).allowed_events():
                with self.subTest(state=state, event=event):
                    machine = at(state)
                    self.assertNotEqual(machine.handle_event(event).state, state)

    def test_invalid_input_and_readonly_state(self):
        machine = at(S.TRACKING)
        before = machine.last_transition
        for event in ('TARGET_LOST', None, S.TARGET_LOST):
            with self.assertRaises(TypeError):
                machine.handle_event(event)
        with self.assertRaises(TypeError):
            machine.handle_event(E.TARGET_LOST, reason=None)
        self.assertIs(machine.last_transition, before)
        self.assertEqual(machine.state, S.TRACKING)
        with self.assertRaises(AttributeError):
            machine.state = S.LANDING

    def test_hold_preserves_avoidance_context_even_after_rejection(self):
        machine = at(S.REPLANNING)
        machine.handle_event(E.REQUEST_HOLD)
        with self.assertRaises(InvalidTransition):
            machine.handle_event(E.PLAN_READY)
        self.assertEqual(machine.handle_event(E.RESUME_MISSION).state, S.REPLANNING)
        self.assertEqual(machine.handle_event(E.PLAN_READY).state, S.TRACKING)

    def test_abort_requires_explicit_reset(self):
        machine = at(S.TRACKING)
        machine.handle_event(E.ABORT_MISSION)
        self.assertEqual(machine.allowed_events(), frozenset({E.RESET_MISSION}))
        with self.assertRaises(InvalidTransition):
            machine.handle_event(E.START_MISSION)
        self.assertEqual(machine.handle_event(E.RESET_MISSION).state, S.IDLE)

    def test_transition_metadata_and_immutable_record(self):
        machine = MissionStateMachine()
        record = machine.handle_event(E.START_MISSION, reason='operator approved')
        self.assertEqual((record.previous_state, record.state), (S.IDLE, S.TAKEOFF))
        self.assertEqual(record.reason, 'operator approved')
        self.assertEqual(record.sequence, 1)
        with self.assertRaises(FrozenInstanceError):
            record.reason = 'changed'
        self.assertEqual(machine.handle_event(E.TAKEOFF_COMPLETED).sequence, 2)

    def test_machine_instances_do_not_share_state(self):
        first, second = MissionStateMachine(), MissionStateMachine()
        first.handle_event(E.START_MISSION)
        self.assertEqual(second.state, S.IDLE)
        self.assertIsNone(second.last_transition)


if __name__ == '__main__':
    unittest.main()
