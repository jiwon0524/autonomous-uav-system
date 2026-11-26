"""Logical target identity and lifecycle; independent of perception and missions."""
from .target_manager import (
    ExternalTargetId, IdentityConflict, InvalidTargetTransition, Observation,
    PrimaryTargetPolicy, SelectionConflict, StaleObservation, Target, TargetChange,
    TargetEvent, TargetManager, TargetNotFound, TargetStatus, TargetType,
)

__all__ = [
    "ExternalTargetId", "IdentityConflict", "InvalidTargetTransition", "Observation",
    "PrimaryTargetPolicy", "SelectionConflict", "StaleObservation", "Target",
    "TargetChange", "TargetEvent", "TargetManager", "TargetNotFound", "TargetStatus",
    "TargetType",
]
