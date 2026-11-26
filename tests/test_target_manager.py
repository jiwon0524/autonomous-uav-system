"""Explicit isolated domain fixtures, never connected to runtime adapters."""
from dataclasses import FrozenInstanceError
from datetime import datetime, timezone
from uuid import UUID, uuid4
import unittest

from src.autonomy.targets import (
    ExternalTargetId, IdentityConflict, InvalidTargetTransition, Observation,
    PrimaryTargetPolicy, SelectionConflict, StaleObservation, TargetEvent,
    TargetManager, TargetNotFound, TargetStatus as S, TargetType as T,
)


def tracked(manager):
    target = manager.register_target(T.PERSON, observation=Observation(1))
    manager.select_target(target.id)
    return manager.start_tracking(target.id)


class TargetManagerTests(unittest.TestCase):
    def test_empty_manager_creates_nothing(self):
        manager = TargetManager()
        self.assertEqual(manager.list_targets(), ())
        self.assertIsNone(manager.get_selected_target())
        self.assertEqual(manager.last_changes, ())

    def test_registration_and_lookup(self):
        manager = TargetManager()
        observation = Observation(1, datetime(2026, 10, 31, tzinfo=timezone.utc))
        target = manager.register_target(T.PERSON, observation=observation)
        self.assertIsInstance(target.id, UUID)
        self.assertEqual(manager.get_target(target.id), target)
        self.assertEqual(target.last_observation, observation)
        self.assertEqual(target.status, S.DETECTED)
        self.assertFalse(target.selected)
        self.assertEqual(target.last_change.event, TargetEvent.REGISTERED)

    def test_external_identity_deduplication_updates_same_target(self):
        manager = TargetManager()
        external = ExternalTargetId('opaque-adapter-session', 'object-key')
        target = manager.register_target(T.PERSON, external_id=external, observation=Observation(0))
        manager.select_target(target.id)
        for sequence in range(1, 31):
            duplicate = manager.register_target(T.PERSON, external_id=external, observation=Observation(sequence))
            self.assertEqual(duplicate.id, target.id)
            self.assertEqual(duplicate.status, S.SELECTED)
        self.assertEqual(len(manager.list_targets()), 1)
        self.assertEqual(manager.get_target(target.id).last_observation.sequence, 30)

    def test_namespaced_external_ids_and_multiple_target_types(self):
        manager = TargetManager()
        first = manager.register_target(T.PERSON, external_id=ExternalTargetId('source-a', '1'))
        second = manager.register_target(T.VEHICLE, external_id=ExternalTargetId('source-b', '1'))
        third = manager.register_target(T.OBJECT)
        self.assertEqual(len(manager.list_targets()), 3)
        self.assertEqual(len({first.id, second.id, third.id}), 3)

    def test_no_external_identity_does_not_guess_object_matching(self):
        manager = TargetManager()
        self.assertNotEqual(manager.register_target(T.OBJECT).id, manager.register_target(T.OBJECT).id)

    def test_duplicate_identity_type_conflict_is_atomic(self):
        manager = TargetManager()
        external = ExternalTargetId('source', 'id')
        manager.register_target(T.PERSON, external_id=external)
        before = (manager.list_targets(), manager.last_changes)
        with self.assertRaises(IdentityConflict):
            manager.register_target(T.VEHICLE, external_id=external)
        self.assertEqual(before, (manager.list_targets(), manager.last_changes))

    def test_primary_selection_and_tracking(self):
        manager = TargetManager()
        target = manager.register_target(T.VEHICLE)
        selected = manager.select_target(target.id)
        self.assertTrue(selected.selected)
        self.assertEqual(selected.status, S.SELECTED)
        self.assertEqual(manager.get_selected_target(), selected)
        self.assertEqual(manager.start_tracking(target.id).status, S.TRACKING)
        before = manager.last_changes
        manager.select_target(target.id)
        self.assertIs(manager.last_changes, before)

    def test_primary_replacement_releases_old_target_without_erasing_observation(self):
        manager = TargetManager()
        old = tracked(manager)
        new = manager.register_target(T.VEHICLE)
        manager.select_target(new.id)
        released = manager.get_target(old.id)
        self.assertFalse(released.selected)
        self.assertEqual(released.status, S.DETECTED)
        self.assertEqual(released.last_observation, old.last_observation)
        self.assertEqual(manager.get_selected_target().id, new.id)
        self.assertEqual([change.event for change in manager.last_changes], [TargetEvent.SUPERSEDED, TargetEvent.SELECTED])
        self.assertEqual(sum(target.selected for target in manager.list_targets()), 1)

    def test_primary_replacement_preserves_lost_semantics(self):
        for status in (S.LOST, S.REACQUIRING):
            with self.subTest(status=status):
                manager = TargetManager()
                old = tracked(manager)
                manager.mark_lost(old.id)
                if status == S.REACQUIRING:
                    manager.start_reacquiring(old.id)
                new = manager.register_target(T.OBJECT)
                manager.select_target(new.id)
                self.assertEqual(manager.get_target(old.id).status, S.LOST)
                manager.select_target(old.id)
                self.assertEqual(manager.get_selected_target().status, S.LOST)
                with self.assertRaises(InvalidTargetTransition):
                    manager.start_tracking(old.id)
                manager.start_reacquiring(old.id)
                self.assertEqual(manager.reacquire(old.id, Observation(2)).status, S.TRACKING)

    def test_reject_replacement_policy_is_atomic(self):
        manager = TargetManager(primary_policy=PrimaryTargetPolicy.REJECT)
        old = tracked(manager)
        new = manager.register_target(T.OBJECT)
        before = (manager.list_targets(), manager.last_changes)
        with self.assertRaises(SelectionConflict):
            manager.select_target(new.id)
        self.assertEqual(before, (manager.list_targets(), manager.last_changes))
        self.assertEqual(manager.get_selected_target().id, old.id)
        manager.deselect_target()
        self.assertEqual(manager.select_target(new.id).status, S.SELECTED)

    def test_deselection(self):
        manager = TargetManager()
        target = tracked(manager)
        self.assertEqual(manager.deselect_target(target.id).status, S.DETECTED)
        self.assertIsNone(manager.get_selected_target())
        self.assertIsNone(manager.deselect_target())
        with self.assertRaises(InvalidTargetTransition):
            manager.deselect_target(target.id)

    def test_lost_reacquiring_reacquired_preserves_identity(self):
        manager = TargetManager()
        target = tracked(manager)
        lost = manager.mark_lost(target.id)
        self.assertTrue(lost.selected)
        self.assertEqual(lost.last_observation, target.last_observation)
        self.assertEqual(manager.start_reacquiring(target.id).status, S.REACQUIRING)
        restored = manager.reacquire(target.id, Observation(2))
        self.assertEqual(restored.id, target.id)
        self.assertEqual(restored.status, S.TRACKING)
        self.assertEqual(restored.last_change.event, TargetEvent.REACQUIRED)
        self.assertEqual(restored.last_change.previous_status, S.REACQUIRING)
        self.assertEqual(restored.last_observation.sequence, 2)
        self.assertEqual(len(manager.list_targets()), 1)

    def test_observation_update_does_not_implicitly_reacquire(self):
        manager = TargetManager()
        target = tracked(manager)
        manager.mark_lost(target.id)
        self.assertEqual(manager.update_observation(target.id, Observation(2)).status, S.LOST)

    def test_observation_preserves_last_state_transition(self):
        manager = TargetManager()
        target = tracked(manager)
        lost = manager.mark_lost(target.id)
        observed = manager.update_observation(target.id, Observation(2))
        self.assertEqual(observed.last_change.event, TargetEvent.OBSERVED)
        self.assertEqual(observed.last_status_change, lost.last_status_change)
        self.assertEqual(observed.last_status_change.previous_status, S.TRACKING)
        self.assertEqual(observed.last_status_change.status, S.LOST)

    def test_stale_and_conflicting_observations_do_not_corrupt_state(self):
        manager = TargetManager()
        target = manager.register_target(T.OBJECT, observation=Observation(5))
        before = (manager.list_targets(), manager.last_changes)
        manager.update_observation(target.id, Observation(5))
        self.assertEqual(before, (manager.list_targets(), manager.last_changes))
        for observation in (Observation(4), Observation(5, datetime(2026, 10, 31, tzinfo=timezone.utc))):
            with self.assertRaises(StaleObservation):
                manager.update_observation(target.id, observation)
            self.assertEqual(before, (manager.list_targets(), manager.last_changes))

    def test_reacquisition_requires_fresh_observation(self):
        manager = TargetManager()
        target = tracked(manager)
        manager.mark_lost(target.id)
        manager.start_reacquiring(target.id)
        before = (manager.list_targets(), manager.last_changes)
        with self.assertRaises(StaleObservation):
            manager.reacquire(target.id, Observation(1))
        self.assertEqual(before, (manager.list_targets(), manager.last_changes))

    def test_nonexistent_target_operations_rejected(self):
        manager = TargetManager()
        unknown = uuid4()
        operations = [manager.get_target, manager.select_target, manager.deselect_target,
                      manager.start_tracking, manager.mark_lost, manager.start_reacquiring,
                      lambda target_id: manager.reacquire(target_id, Observation(1)),
                      lambda target_id: manager.update_observation(target_id, Observation(1))]
        for operation in operations:
            with self.subTest(operation=operation):
                with self.assertRaises(TargetNotFound):
                    operation(unknown)
                self.assertEqual(manager.list_targets(), ())
                self.assertEqual(manager.last_changes, ())

    def test_illegal_status_operations_rejected_without_mutation(self):
        manager = TargetManager()
        target = manager.register_target(T.OBJECT)
        for operation in (manager.start_tracking, manager.mark_lost, manager.start_reacquiring,
                          lambda target_id: manager.reacquire(target_id, Observation(1))):
            before = (manager.list_targets(), manager.last_changes, manager.get_selected_target())
            with self.assertRaises(InvalidTargetTransition):
                operation(target.id)
            self.assertEqual(before, (manager.list_targets(), manager.last_changes, manager.get_selected_target()))
        manager.select_target(target.id)
        with self.assertRaises(InvalidTargetTransition):
            manager.mark_lost(target.id)

    def test_returned_snapshots_are_immutable_and_not_live_views(self):
        manager = TargetManager()
        target = manager.register_target(T.PERSON, external_id=ExternalTargetId('source', '1'), observation=Observation(1))
        for obj, field, value in [(target, 'status', S.TRACKING), (target.external_id, 'value', 'other'),
                                   (target.last_observation, 'sequence', 100), (target.last_change, 'event', TargetEvent.LOST)]:
            with self.assertRaises(FrozenInstanceError):
                setattr(obj, field, value)
        targets = manager.list_targets()
        self.assertIsInstance(targets, tuple)
        manager.select_target(target.id)
        self.assertEqual(targets[0].status, S.DETECTED)
        self.assertEqual(manager.get_target(target.id).status, S.SELECTED)

    def test_bad_inputs_and_id_factory_collision_are_rejected(self):
        with self.assertRaises(TypeError):
            TargetManager(primary_policy='REPLACE')
        for sequence in (True, 1.5, '1'):
            with self.assertRaises(TypeError):
                Observation(sequence)
        with self.assertRaises(ValueError):
            Observation(-1)
        with self.assertRaises(ValueError):
            Observation(1, datetime(2026, 10, 31))
        with self.assertRaises(ValueError):
            ExternalTargetId('', '1')
        manager = TargetManager(id_factory=lambda: UUID(int=1))
        manager.register_target(T.PERSON)
        before = (manager.list_targets(), manager.last_changes)
        with self.assertRaises(IdentityConflict):
            manager.register_target(T.OBJECT)
        self.assertEqual(before, (manager.list_targets(), manager.last_changes))
        with self.assertRaises(TypeError):
            manager.get_target('1')
        with self.assertRaises(TypeError):
            manager.register_target('PERSON')


if __name__ == '__main__':
    unittest.main()
