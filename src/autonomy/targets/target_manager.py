"""Pure logical target management. No inference, matching, I/O or flight control.

A single owner dispatches operations serially. Re-identification must be confirmed
by an external adapter; an ID alone does not prove real-world object identity.
"""
from collections.abc import Callable
from dataclasses import dataclass, replace
from datetime import datetime
from enum import Enum, auto
from uuid import UUID, uuid4


class TargetType(Enum):
    PERSON = auto()
    VEHICLE = auto()
    OBJECT = auto()


class TargetStatus(Enum):
    DETECTED = auto()
    SELECTED = auto()
    TRACKING = auto()
    LOST = auto()
    REACQUIRING = auto()


class TargetEvent(Enum):
    REGISTERED = auto()
    OBSERVED = auto()
    SELECTED = auto()
    DESELECTED = auto()
    SUPERSEDED = auto()
    TRACKING_STARTED = auto()
    LOST = auto()
    REACQUISITION_STARTED = auto()
    REACQUIRED = auto()


class PrimaryTargetPolicy(Enum):
    REPLACE = auto()
    REJECT = auto()


@dataclass(frozen=True)
class ExternalTargetId:
    """Opaque source namespace and value, without assumptions about any SDK."""
    source: str
    value: str

    def __post_init__(self):
        for name in ("source", "value"):
            field = getattr(self, name)
            if not isinstance(field, str):
                raise TypeError(f"{name} must be a string")
            if not field.strip():
                raise ValueError(f"{name} must not be blank")


@dataclass(frozen=True)
class Observation:
    """Caller-owned per-target ordering. No wall-clock access or sensor payload."""
    sequence: int
    observed_at: datetime | None = None

    def __post_init__(self):
        if type(self.sequence) is not int:
            raise TypeError("sequence must be an integer")
        if self.sequence < 0:
            raise ValueError("sequence must not be negative")
        if self.observed_at is not None:
            if not isinstance(self.observed_at, datetime):
                raise TypeError("observed_at must be a datetime or None")
            if self.observed_at.tzinfo is None or self.observed_at.utcoffset() is None:
                raise ValueError("observed_at must be timezone-aware")


@dataclass(frozen=True)
class TargetChange:
    target_id: UUID
    event: TargetEvent
    previous_status: TargetStatus | None
    status: TargetStatus
    was_selected: bool
    is_selected: bool
    sequence: int


@dataclass(frozen=True)
class Target:
    id: UUID
    type: TargetType
    status: TargetStatus
    selected: bool
    external_id: ExternalTargetId | None
    last_observation: Observation | None
    last_change: TargetChange | None = None
    last_status_change: TargetChange | None = None


class TargetNotFound(KeyError):
    pass


class InvalidTargetTransition(ValueError):
    pass


class IdentityConflict(ValueError):
    pass


class SelectionConflict(ValueError):
    pass


class StaleObservation(ValueError):
    pass


class TargetManager:
    """Owns immutable snapshots; operations validate before committing changes.

    Only explicit registration creates a target. Observations never select,
    start tracking, or recover LOST targets automatically.
    """

    def __init__(self, *, primary_policy: PrimaryTargetPolicy = PrimaryTargetPolicy.REPLACE,
                 id_factory: Callable[[], UUID] = uuid4):
        if not isinstance(primary_policy, PrimaryTargetPolicy):
            raise TypeError("primary_policy must be a PrimaryTargetPolicy")
        if not callable(id_factory):
            raise TypeError("id_factory must be callable")
        self.__policy = primary_policy
        self.__id_factory = id_factory
        self.__targets: dict[UUID, Target] = {}
        self.__external: dict[ExternalTargetId, UUID] = {}
        self.__primary: UUID | None = None
        self.__sequence = 0
        self.__last_changes: tuple[TargetChange, ...] = ()

    @property
    def last_changes(self) -> tuple[TargetChange, ...]:
        """Changes from the latest mutating operation, including both switched targets."""
        return self.__last_changes

    def get_target(self, target_id: UUID) -> Target:
        if not isinstance(target_id, UUID):
            raise TypeError("target_id must be a UUID")
        try:
            return self.__targets[target_id]
        except KeyError:
            raise TargetNotFound(target_id) from None

    def list_targets(self) -> tuple[Target, ...]:
        return tuple(self.__targets.values())

    def get_selected_target(self) -> Target | None:
        return self.get_target(self.__primary) if self.__primary is not None else None

    @staticmethod
    def _check_observation(observation: Observation | None):
        if observation is not None and not isinstance(observation, Observation):
            raise TypeError("observation must be an Observation or None")

    @staticmethod
    def _observation_is_new(target: Target, observation: Observation) -> bool:
        previous = target.last_observation
        if previous is None:
            return True
        if observation.sequence < previous.sequence:
            raise StaleObservation("observation sequence is older than the last observation")
        if observation.sequence == previous.sequence:
            if observation != previous:
                raise StaleObservation("same sequence contains conflicting observation metadata")
            return False
        return True

    def _commit(self, updates: list[tuple[Target, TargetEvent]], primary: UUID | None):
        targets = dict(self.__targets)
        changes = []
        sequence = self.__sequence
        for target, event in updates:
            old = targets.get(target.id)
            sequence += 1
            change = TargetChange(target.id, event, old.status if old else None,
                                  target.status, old.selected if old else False,
                                  target.selected, sequence)
            status_changed = old is None or old.status != target.status or old.selected != target.selected
            last_status_change = change if status_changed else old.last_status_change
            targets[target.id] = replace(target, last_change=change, last_status_change=last_status_change)
            changes.append(change)
        self.__targets = targets
        self.__primary = primary
        self.__sequence = sequence
        self.__last_changes = tuple(changes)

    def register_target(self, target_type: TargetType, *, external_id: ExternalTargetId | None = None,
                        observation: Observation | None = None) -> Target:
        if not isinstance(target_type, TargetType):
            raise TypeError("target_type must be a TargetType")
        if external_id is not None and not isinstance(external_id, ExternalTargetId):
            raise TypeError("external_id must be an ExternalTargetId or None")
        self._check_observation(observation)
        if external_id is not None and external_id in self.__external:
            existing = self.get_target(self.__external[external_id])
            if existing.type != target_type:
                raise IdentityConflict("external identity is already registered with a different type")
            return self.update_observation(existing.id, observation) if observation is not None else existing
        target_id = self.__id_factory()
        if not isinstance(target_id, UUID):
            raise TypeError("id_factory must return a UUID")
        if target_id in self.__targets:
            raise IdentityConflict("id_factory returned an existing internal ID")
        target = Target(target_id, target_type, TargetStatus.DETECTED, False, external_id, observation)
        self._commit([(target, TargetEvent.REGISTERED)], self.__primary)
        if external_id is not None:
            self.__external[external_id] = target_id
        return self.get_target(target_id)

    def update_observation(self, target_id: UUID, observation: Observation) -> Target:
        target = self.get_target(target_id)
        self._check_observation(observation)
        if observation is None:
            raise TypeError("observation must not be None")
        if self._observation_is_new(target, observation):
            self._commit([(replace(target, last_observation=observation), TargetEvent.OBSERVED)], self.__primary)
        return self.get_target(target_id)

    @staticmethod
    def _release(target: Target) -> Target:
        # Loss must not be disguised as a fresh detection when switching primary.
        status = (TargetStatus.LOST if target.status in
                  {TargetStatus.LOST, TargetStatus.REACQUIRING} else TargetStatus.DETECTED)
        return replace(target, selected=False, status=status)

    def select_target(self, target_id: UUID) -> Target:
        target = self.get_target(target_id)
        if target.selected:
            return target
        if target.status not in {TargetStatus.DETECTED, TargetStatus.LOST}:
            raise InvalidTargetTransition("only detected or lost targets can be selected")
        previous = self.get_selected_target()
        if previous is not None and self.__policy == PrimaryTargetPolicy.REJECT:
            raise SelectionConflict("release the current primary before selecting another")
        updates = [(self._release(previous), TargetEvent.SUPERSEDED)] if previous else []
        status = TargetStatus.SELECTED if target.status == TargetStatus.DETECTED else TargetStatus.LOST
        updates.append((replace(target, selected=True, status=status), TargetEvent.SELECTED))
        self._commit(updates, target.id)
        return self.get_target(target_id)

    def deselect_target(self, target_id: UUID | None = None) -> Target | None:
        target = self.get_selected_target() if target_id is None else self.get_target(target_id)
        if target is None:
            return None
        if not target.selected:
            raise InvalidTargetTransition("target is not the selected primary")
        self._commit([(self._release(target), TargetEvent.DESELECTED)], None)
        return self.get_target(target.id)

    def _require_status(self, target_id: UUID, expected: TargetStatus) -> Target:
        target = self.get_target(target_id)
        if not target.selected or target.status != expected:
            raise InvalidTargetTransition(f"selected {expected.name} target required; got {target.status.name}")
        return target

    def start_tracking(self, target_id: UUID) -> Target:
        target = self._require_status(target_id, TargetStatus.SELECTED)
        self._commit([(replace(target, status=TargetStatus.TRACKING), TargetEvent.TRACKING_STARTED)], self.__primary)
        return self.get_target(target_id)

    def mark_lost(self, target_id: UUID) -> Target:
        target = self._require_status(target_id, TargetStatus.TRACKING)
        self._commit([(replace(target, status=TargetStatus.LOST), TargetEvent.LOST)], self.__primary)
        return self.get_target(target_id)

    def start_reacquiring(self, target_id: UUID) -> Target:
        target = self._require_status(target_id, TargetStatus.LOST)
        self._commit([(replace(target, status=TargetStatus.REACQUIRING), TargetEvent.REACQUISITION_STARTED)], self.__primary)
        return self.get_target(target_id)

    def reacquire(self, target_id: UUID, observation: Observation) -> Target:
        target = self._require_status(target_id, TargetStatus.REACQUIRING)
        self._check_observation(observation)
        if observation is None:
            raise TypeError("observation must not be None")
        if not self._observation_is_new(target, observation):
            raise StaleObservation("reacquisition requires a newer observation")
        restored = replace(target, status=TargetStatus.TRACKING, last_observation=observation)
        self._commit([(restored, TargetEvent.REACQUIRED)], self.__primary)
        return self.get_target(target_id)
