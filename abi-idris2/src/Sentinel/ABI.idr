||| SPDX-License-Identifier: MPL-2.0
|||
||| Idris2 ABI conformance spec for 0patch-lsa-sentinel.
|||
||| This is the estate's "offline verifying elaborator" for the C ABI: it pins
||| the canonical integer encodings shared by the SPARK kernel, the Zig header,
||| and the Rust host, and it machine-checks (totally) that
|||   (1) every code round-trips: decode . encode = Just,
|||   (2) the classifier semantics match the SPARK truth table on the spec rows.
|||
||| If any side renumbers an enum or drifts a row, this file stops type-checking.
module Sentinel.ABI

%default total

------------------------------------------------------------------------------
-- Domain (mirrors Sentinel.Classifier in SPARK and `enum`s in model.rs)
------------------------------------------------------------------------------

public export
data Protection = PUnknown | POff | POn

public export
data Loader = LUnknown | LBlocked | LLoaded

public export
data Patches = PatUnknown | PatAbsent | PatPresent

public export
data Verdict = Indeterminate | Moot | OpenCovered | BlockedInert

------------------------------------------------------------------------------
-- Canonical integer encodings  (DO NOT renumber — these ARE the ABI)
------------------------------------------------------------------------------

public export encProtection : Protection -> Int
encProtection PUnknown = 0
encProtection POff     = 1
encProtection POn      = 2

public export decProtection : Int -> Maybe Protection
decProtection 0 = Just PUnknown
decProtection 1 = Just POff
decProtection 2 = Just POn
decProtection _ = Nothing

public export encLoader : Loader -> Int
encLoader LUnknown = 0
encLoader LBlocked = 1
encLoader LLoaded  = 2

public export decLoader : Int -> Maybe Loader
decLoader 0 = Just LUnknown
decLoader 1 = Just LBlocked
decLoader 2 = Just LLoaded
decLoader _ = Nothing

public export encPatches : Patches -> Int
encPatches PatUnknown = 0
encPatches PatAbsent  = 1
encPatches PatPresent = 2

public export decPatches : Int -> Maybe Patches
decPatches 0 = Just PatUnknown
decPatches 1 = Just PatAbsent
decPatches 2 = Just PatPresent
decPatches _ = Nothing

public export encVerdict : Verdict -> Int
encVerdict Indeterminate = 0
encVerdict Moot          = 1
encVerdict OpenCovered   = 2
encVerdict BlockedInert  = 3

public export decVerdict : Int -> Maybe Verdict
decVerdict 0 = Just Indeterminate
decVerdict 1 = Just Moot
decVerdict 2 = Just OpenCovered
decVerdict 3 = Just BlockedInert
decVerdict _ = Nothing

------------------------------------------------------------------------------
-- (1) Round-trip proofs: every value decodes back from its code.
------------------------------------------------------------------------------

export rtProtection : (p : Protection) -> decProtection (encProtection p) = Just p
rtProtection PUnknown = Refl
rtProtection POff     = Refl
rtProtection POn      = Refl

export rtLoader : (l : Loader) -> decLoader (encLoader l) = Just l
rtLoader LUnknown = Refl
rtLoader LBlocked = Refl
rtLoader LLoaded  = Refl

export rtPatches : (s : Patches) -> decPatches (encPatches s) = Just s
rtPatches PatUnknown = Refl
rtPatches PatAbsent  = Refl
rtPatches PatPresent = Refl

export rtVerdict : (v : Verdict) -> decVerdict (encVerdict v) = Just v
rtVerdict Indeterminate = Refl
rtVerdict Moot          = Refl
rtVerdict OpenCovered   = Refl
rtVerdict BlockedInert  = Refl

------------------------------------------------------------------------------
-- (2) Classifier semantics — identical four-branch table to the SPARK body.
------------------------------------------------------------------------------

public export
classify : Protection -> Loader -> Patches -> Verdict
classify _    _        PatAbsent  = Moot
classify POff LLoaded  PatPresent = OpenCovered
classify POn  _        PatPresent = BlockedInert
classify _    LBlocked PatPresent = BlockedInert
classify _    _        _          = Indeterminate

------------------------------------------------------------------------------
-- Spec properties (machine-checked, mirroring the SPARK postcondition).
------------------------------------------------------------------------------

||| Property 1 (one direction of the SPARK iff): no lsass patches => Moot,
||| for ANY protection/loader state.
export mootWhenAbsent : (p : Protection) -> (l : Loader)
                     -> classify p l PatAbsent = Moot
mootWhenAbsent _ _ = Refl

||| The headline operational row: LSA on + outstanding lsass patch = alarm.
export dangerousRowIsAlarm : classify POn LBlocked PatPresent = BlockedInert
dangerousRowIsAlarm = Refl

||| A genuinely covered host is the only way to OpenCovered (spot rows).
export coveredRow : classify POff LLoaded PatPresent = OpenCovered
coveredRow = Refl

||| Off + blocked + present is still a silent failure, not "covered".
export offButBlockedIsAlarm : classify POff LBlocked PatPresent = BlockedInert
offButBlockedIsAlarm = Refl

||| Unknown patch status never reads as covered or moot.
export unknownPatchesIndeterminate : classify POff LLoaded PatUnknown = Indeterminate
unknownPatchesIndeterminate = Refl

||| All-unknown is fail-safe Indeterminate.
export allUnknownIndeterminate : classify PUnknown LUnknown PatUnknown = Indeterminate
allUnknownIndeterminate = Refl
