--  SPDX-License-Identifier: MPL-2.0
--
--  Sentinel.Classifier — the safety-critical decision kernel.
--
--  Given three observed signals about a host, decide whether 0patch's
--  lsass-targeted micropatch coverage is live, silently inert, or moot.
--  This is the one piece that must never be wrong, so it is pure, total,
--  and proved with SPARK (see `just prove`).
--
--  The tri-state semantics mirror DESIGN.md §3:
--    Open_Covered  : LSA Protection off AND loader in lsass, patches present
--                    -> lsass patches live (cred-dump protection forfeited)
--    Blocked_Inert : patches present AND (LSA on OR loader blocked)
--                    -> the SILENT-FAILURE state: believed protected, isn't
--    Moot          : no outstanding lsass-targeted patches
--    Indeterminate : inputs insufficient to assert any of the above

pragma SPARK_Mode (On);

package Sentinel.Classifier is

   --  Canonical integer encodings (shared verbatim with the Idris2 ABI spec,
   --  the Zig `sentinel.h`, and the Rust `model.rs`). DO NOT renumber.
   type Lsa_Protection is (Protection_Unknown, Protection_Off, Protection_On);
   --  0, 1, 2
   type Loader_Presence is (Loader_Unknown, Loader_Blocked, Loader_Loaded);
   --  0, 1, 2
   type Patch_Status is (Patch_Unknown, Patch_Absent, Patch_Present);
   --  0, 1, 2

   type Verdict is (Indeterminate, Moot, Open_Covered, Blocked_Inert);
   --  0, 1, 2, 3

   type Signals is record
      Protection : Lsa_Protection;
      Loader     : Loader_Presence;
      Patches    : Patch_Status;
   end record;

   function Classify (S : Signals) return Verdict
     with
       Post =>
         --  Moot iff (and only iff) there are no lsass-targeted patches.
         ((Classify'Result = Moot) = (S.Patches = Patch_Absent))

         --  Open_Covered is only ever reported for a genuinely covered host.
         and then
           (if Classify'Result = Open_Covered then
              S.Protection = Protection_Off
              and then S.Loader = Loader_Loaded
              and then S.Patches = Patch_Present)

         --  The dangerous verdict is only reported when patches truly exist
         --  and something is in fact keeping the loader out of lsass.
         and then
           (if Classify'Result = Blocked_Inert then
              S.Patches = Patch_Present
              and then
                (S.Protection = Protection_On
                 or else S.Loader = Loader_Blocked));

end Sentinel.Classifier;
