"""Pure event-driven mission domain. No ROS, I/O, timers, or flight commands.

One owner should dispatch events serially. External adapters must establish that
completion events are true; this module cannot verify aircraft or sensor state.
"""
from dataclasses import dataclass
from enum import Enum, auto
from types import MappingProxyType


class MissionState(Enum):
    IDLE = auto()
    TAKEOFF = auto()
    PATROL = auto()
    TARGET_DETECTED = auto()
    TARGET_SELECTED = auto()
    TRACKING = auto()
    TARGET_LOST = auto()
    REACQUIRING = auto()
    AVOIDING = auto()
    REPLANNING = auto()
    RETURN_HOME = auto()
    LANDING = auto()
    HOLD = auto()
    MISSION_ABORTED = auto()


class MissionEvent(Enum):
    START_MISSION = auto()
    TAKEOFF_COMPLETED = auto()
    TARGET_DETECTED = auto()
    SELECT_TARGET = auto()
    START_TRACKING = auto()
    DISMISS_TARGET = auto()
    TARGET_LOST = auto()
    START_REACQUISITION = auto()
    TARGET_REACQUIRED = auto()
    TRACKING_COMPLETED = auto()
    OBSTACLE_DETECTED = auto()
    REPLAN_REQUIRED = auto()
    PLAN_READY = auto()
    OBSTACLE_CLEARED = auto()
    REQUEST_RETURN_HOME = auto()
    HOME_REACHED = auto()
    LANDING_COMPLETED = auto()
    REQUEST_HOLD = auto()
    RESUME_MISSION = auto()
    ABORT_MISSION = auto()
    RESET_MISSION = auto()


@dataclass(frozen=True)
class Transition:
    previous_state: MissionState
    state: MissionState
    event: MissionEvent
    reason: str
    sequence: int


class InvalidTransition(ValueError):
    """Rejected event; the machine and its last transition are unchanged."""

    def __init__(self, state: MissionState, event: MissionEvent):
        self.state = state
        self.event = event
        super().__init__(f"{event.name} is not allowed in {state.name}")


S = MissionState
E = MissionEvent
_TRANSITIONS = MappingProxyType({
    (S.IDLE, E.START_MISSION): S.TAKEOFF,
    (S.TAKEOFF, E.TAKEOFF_COMPLETED): S.PATROL,
    (S.PATROL, E.TARGET_DETECTED): S.TARGET_DETECTED,
    (S.TARGET_DETECTED, E.SELECT_TARGET): S.TARGET_SELECTED,
    (S.TARGET_DETECTED, E.DISMISS_TARGET): S.PATROL,
    (S.TARGET_SELECTED, E.DISMISS_TARGET): S.PATROL,
    (S.TARGET_SELECTED, E.START_TRACKING): S.TRACKING,
    (S.TRACKING, E.TARGET_LOST): S.TARGET_LOST,
    (S.TARGET_LOST, E.START_REACQUISITION): S.REACQUIRING,
    (S.REACQUIRING, E.TARGET_REACQUIRED): S.TRACKING,
    (S.TRACKING, E.TRACKING_COMPLETED): S.PATROL,
    (S.AVOIDING, E.REPLAN_REQUIRED): S.REPLANNING,
    (S.RETURN_HOME, E.HOME_REACHED): S.LANDING,
    (S.LANDING, E.LANDING_COMPLETED): S.IDLE,
    (S.MISSION_ABORTED, E.RESET_MISSION): S.IDLE,
})
_AVOIDABLE = frozenset({S.PATROL, S.TRACKING})
_INTERRUPTIBLE = frozenset(set(S) - {S.IDLE, S.LANDING, S.MISSION_ABORTED, S.HOLD})
_ABORTABLE = frozenset(set(S) - {S.IDLE, S.MISSION_ABORTED})


class MissionStateMachine:
    """Only accepted events can change state; no public state setter exists."""

    def __init__(self):
        self._state = S.IDLE
        self._last_transition: Transition | None = None
        self._sequence = 0
        self._avoid_return: MissionState | None = None
        self._hold_return: MissionState | None = None

    @property
    def state(self) -> MissionState:
        return self._state

    @property
    def last_transition(self) -> Transition | None:
        return self._last_transition

    def allowed_events(self) -> frozenset[MissionEvent]:
        events = {event for state, event in _TRANSITIONS if state == self._state}
        if self._state in _AVOIDABLE:
            events.add(E.OBSTACLE_DETECTED)
        if self._state in _INTERRUPTIBLE:
            events.update({E.REQUEST_HOLD, E.REQUEST_RETURN_HOME})
        if self._state == S.HOLD:
            events.update({E.RESUME_MISSION, E.REQUEST_RETURN_HOME})
        if self._state in _ABORTABLE:
            events.add(E.ABORT_MISSION)
        if self._state == S.AVOIDING:
            events.add(E.OBSTACLE_CLEARED)
        if self._state == S.REPLANNING:
            events.add(E.PLAN_READY)
        # A return request while already returning has no new transition.
        if self._state == S.RETURN_HOME:
            events.discard(E.REQUEST_RETURN_HOME)
        return frozenset(events)

    def handle_event(self, event: MissionEvent, *, reason: str = "") -> Transition:
        if not isinstance(event, MissionEvent):
            raise TypeError("event must be a MissionEvent")
        if not isinstance(reason, str):
            raise TypeError("reason must be a string")
        if event not in self.allowed_events():
            raise InvalidTransition(self._state, event)

        previous = self._state
        avoid_return = self._avoid_return
        hold_return = self._hold_return
        if event == E.OBSTACLE_DETECTED:
            next_state, avoid_return = S.AVOIDING, previous
        elif event == E.REQUEST_HOLD:
            next_state, hold_return = S.HOLD, previous
        elif event == E.RESUME_MISSION:
            if hold_return is None:
                raise InvalidTransition(previous, event)
            next_state, hold_return = hold_return, None
        elif event in {E.OBSTACLE_CLEARED, E.PLAN_READY}:
            if avoid_return is None:
                raise InvalidTransition(previous, event)
            next_state, avoid_return = avoid_return, None
        elif event == E.REQUEST_RETURN_HOME:
            next_state = S.RETURN_HOME
            avoid_return = hold_return = None
        elif event == E.ABORT_MISSION:
            next_state = S.MISSION_ABORTED
            avoid_return = hold_return = None
        else:
            next_state = _TRANSITIONS[(previous, event)]

        if next_state == S.IDLE:
            avoid_return = hold_return = None
        transition = Transition(previous, next_state, event, reason or event.name, self._sequence + 1)
        # Commit only after validation and transition resolution succeed.
        self._state = next_state
        self._avoid_return = avoid_return
        self._hold_return = hold_return
        self._sequence = transition.sequence
        self._last_transition = transition
        return transition
